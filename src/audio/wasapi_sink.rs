// AURI Native WASAPI Audio Sink
// Implements lock-free ring-buffered WASAPI output with MMCSS real-time scheduling

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use ringbuf::traits::{Consumer, Observer, Producer, Split};
use ringbuf::HeapRb;

use super::mmcss::MmcssRegistration;
use super::normalizer::{db_to_linear, SoftLimiter};
use super::resampler::CubicResampler;
use super::sink::{AudioEngineKind, AudioStatus, PlaybackState};

const BUFFER_CAPACITY_SAMPLES: usize = 192_000; // 2 seconds @ 48 kHz stereo

pub struct WasapiSink {
    _stream: Stream,
    producer: Arc<Mutex<ringbuf::HeapProd<f32>>>,
    resampler: Arc<Mutex<CubicResampler>>,
    _limiter: SoftLimiter,
    state: Arc<AtomicU32>, // 0: Stopped, 1: Playing, 2: Paused
    volume_bits: Arc<AtomicU32>, // f32 as u32 bits
    loudness_gain_db: Arc<Mutex<Option<f64>>>,
    gain_multiplier_bits: Arc<AtomicU32>, // f32 as u32 bits
    position_frames: Arc<AtomicU64>,
    duration_frames: Arc<AtomicU64>,
    sample_rate: u32,
    channels: u16,
    device_name: String,
    eof_flag: Arc<AtomicBool>,
    flush_flag: Arc<AtomicBool>,
    _mmcss: Option<MmcssRegistration>,
}

impl WasapiSink {
    pub fn new() -> Result<Self, String> {
        let mmcss = MmcssRegistration::register_audio_thread();
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "No default audio output device available".to_string())?;

        let device_name = device.to_string();
        let default_config = device
            .default_output_config()
            .map_err(|e| format!("Failed to get default audio config: {}", e))?;

        let sample_rate = default_config.sample_rate();
        let channels = default_config.channels();

        let config = StreamConfig {
            channels,
            sample_rate,
            buffer_size: cpal::BufferSize::Default,
        };

        let rb = HeapRb::<f32>::new(BUFFER_CAPACITY_SAMPLES);
        let (producer, mut consumer) = rb.split();
        let producer = Arc::new(Mutex::new(producer));

        let state = Arc::new(AtomicU32::new(1)); // Playing by default
        let volume_bits = Arc::new(AtomicU32::new(0.8_f32.to_bits()));
        let gain_multiplier_bits = Arc::new(AtomicU32::new(1.0_f32.to_bits()));
        let loudness_gain_db = Arc::new(Mutex::new(None));
        let position_frames = Arc::new(AtomicU64::new(0));
        let duration_frames = Arc::new(AtomicU64::new(0));
        let eof_flag = Arc::new(AtomicBool::new(false));
        let flush_flag = Arc::new(AtomicBool::new(false));

        let limiter = SoftLimiter::default();
        let limiter_clone = limiter.clone();

        let state_cb = Arc::clone(&state);
        let vol_cb = Arc::clone(&volume_bits);
        let gain_cb = Arc::clone(&gain_multiplier_bits);
        let pos_cb = Arc::clone(&position_frames);
        let eof_cb = Arc::clone(&eof_flag);
        let flush_cb = Arc::clone(&flush_flag);

        let err_fn = |err| eprintln!("[AURI Audio Sink Error] Stream callback error: {}", err);

        let stream = match default_config.sample_format() {
            SampleFormat::F32 => device.build_output_stream(
                config.clone(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    if flush_cb.swap(false, Ordering::SeqCst) {
                        while consumer.try_pop().is_some() {}
                    }

                    let current_state = state_cb.load(Ordering::Relaxed);
                    let vol = f32::from_bits(vol_cb.load(Ordering::Relaxed));
                    let gain = f32::from_bits(gain_cb.load(Ordering::Relaxed));
                    let effective_volume = vol * gain;

                    if current_state != 1 {
                        // Paused or Stopped: output pure silence
                        data.fill(0.0);
                        return;
                    }

                    let channels_cnt = channels as usize;
                    let mut read_samples = 0;
                    for sample in data.iter_mut() {
                        if let Some(s) = consumer.try_pop() {
                            let scaled = s * effective_volume;
                            *sample = limiter_clone.process_sample(scaled);
                            read_samples += 1;
                        } else {
                            *sample = 0.0;
                        }
                    }

                    if read_samples > 0 {
                        let frames_read = (read_samples / channels_cnt) as u64;
                        pos_cb.fetch_add(frames_read, Ordering::Relaxed);
                    } else if consumer.is_empty() {
                        eof_cb.store(true, Ordering::Relaxed);
                    }
                },
                err_fn,
                None,
            ),
            SampleFormat::I16 => device.build_output_stream(
                config,
                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    if flush_cb.swap(false, Ordering::SeqCst) {
                        while consumer.try_pop().is_some() {}
                    }

                    let current_state = state_cb.load(Ordering::Relaxed);
                    let vol = f32::from_bits(vol_cb.load(Ordering::Relaxed));
                    let gain = f32::from_bits(gain_cb.load(Ordering::Relaxed));
                    let effective_volume = vol * gain;

                    if current_state != 1 {
                        data.fill(0);
                        return;
                    }

                    let channels_cnt = channels as usize;
                    let mut read_samples = 0;
                    for sample in data.iter_mut() {
                        if let Some(s) = consumer.try_pop() {
                            let scaled = s * effective_volume;
                            let limited = limiter_clone.process_sample(scaled);
                            *sample = (limited * 32767.0).clamp(-32768.0, 32767.0) as i16;
                            read_samples += 1;
                        } else {
                            *sample = 0;
                        }
                    }

                    if read_samples > 0 {
                        let frames_read = (read_samples / channels_cnt) as u64;
                        pos_cb.fetch_add(frames_read, Ordering::Relaxed);
                    } else if consumer.is_empty() {
                        eof_cb.store(true, Ordering::Relaxed);
                    }
                },
                err_fn,
                None,
            ),
            _ => return Err("Unsupported hardware audio format".to_string()),
        }
        .map_err(|e| format!("Failed to build audio stream: {}", e))?;

        stream
            .play()
            .map_err(|e| format!("Failed to start audio stream: {}", e))?;

        let resampler = Arc::new(Mutex::new(CubicResampler::new(
            sample_rate,
            sample_rate,
            channels as usize,
        )));

        Ok(Self {
            _stream: stream,
            producer,
            resampler,
            _limiter: limiter,
            state,
            volume_bits,
            loudness_gain_db,
            gain_multiplier_bits,
            position_frames,
            duration_frames,
            sample_rate,
            channels,
            device_name,
            eof_flag,
            flush_flag,
            _mmcss: mmcss,
        })
    }

    pub fn set_duration_secs(&self, secs: f64) {
        let frames = (secs * self.sample_rate as f64) as u64;
        self.duration_frames.store(frames, Ordering::Relaxed);
    }

    /// Feed decoded PCM samples into the audio sink with real-time backpressure
    pub fn push_samples(&self, samples: &[f32], source_rate: u32, source_channels: usize, is_active: Option<&AtomicBool>) {
        let mut resampler = self.resampler.lock().unwrap();
        let target_rate = self.sample_rate;
        let target_channels = self.channels as usize;

        let resampled = if source_rate != target_rate || source_channels != target_channels {
            *resampler = CubicResampler::new(source_rate, target_rate, source_channels);
            let mut buf = Vec::with_capacity(samples.len());
            resampler.resample(samples, &mut buf);
            buf
        } else {
            samples.to_vec()
        };

        let mut offset = 0;
        while offset < resampled.len() {
            if let Some(active) = is_active {
                if !active.load(Ordering::Relaxed) {
                    break;
                }
            }

            let mut prod = self.producer.lock().unwrap();
            while offset < resampled.len() {
                if prod.try_push(resampled[offset]).is_ok() {
                    offset += 1;
                } else {
                    break;
                }
            }
            drop(prod);

            if offset < resampled.len() {
                // Buffer is full (2 seconds of audio buffered ahead). Sleep briefly to pace with real-time playback.
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        self.eof_flag.store(false, Ordering::Relaxed);
    }

    /// Clear buffered audio (for seeks or track changes)
    pub fn clear_buffer(&self) {
        self.position_frames.store(0, Ordering::Relaxed);
        self.flush_flag.store(true, Ordering::SeqCst);
    }

    /// Synthesize a clean test tone (e.g. 440 Hz A4 chime) with smooth click-free envelope
    pub fn play_test_tone(&self, freq_hz: f32, duration_secs: f32) {
        let frames = (self.sample_rate as f32 * duration_secs) as usize;
        let mut samples = Vec::with_capacity(frames * self.channels as usize);
        let ch = self.channels as usize;

        for i in 0..frames {
            let t = i as f32 / self.sample_rate as f32;
            let raw_sin = (2.0 * std::f32::consts::PI * freq_hz * t).sin();
            // Envelope: 20ms attack, exponential decay
            let attack = (t / 0.02).min(1.0);
            let decay = (-3.0 * t / duration_secs).exp();
            let val = raw_sin * attack * decay * 0.4;
            for _ in 0..ch {
                samples.push(val);
            }
        }

        self.duration_frames.store(frames as u64, Ordering::Relaxed);
        self.position_frames.store(0, Ordering::Relaxed);
        self.push_samples(&samples, self.sample_rate, self.channels as usize, None);
    }
}

impl super::sink::AudioSink for WasapiSink {
    fn play(&self) -> Result<(), String> {
        self.state.store(1, Ordering::SeqCst);
        Ok(())
    }

    fn pause(&self) -> Result<(), String> {
        self.state.store(2, Ordering::SeqCst);
        Ok(())
    }

    fn toggle(&self) -> Result<(), String> {
        let current = self.state.load(Ordering::SeqCst);
        if current == 1 {
            self.pause()
        } else {
            self.play()
        }
    }

    fn stop(&self) -> Result<(), String> {
        self.state.store(0, Ordering::SeqCst);
        self.clear_buffer();
        Ok(())
    }

    fn seek(&self, position: Duration) -> Result<(), String> {
        let target_frames = (position.as_secs_f64() * self.sample_rate as f64) as u64;
        self.position_frames.store(target_frames, Ordering::Relaxed);
        Ok(())
    }

    fn set_volume(&self, volume: f32) {
        let clamped = volume.clamp(0.0, 1.0);
        self.volume_bits.store(clamped.to_bits(), Ordering::Relaxed);
    }

    fn set_loudness_gain(&self, gain_db: Option<f64>) {
        *self.loudness_gain_db.lock().unwrap() = gain_db;
        let mult = match gain_db {
            Some(db) => db_to_linear(db),
            None => 1.0,
        };
        self.gain_multiplier_bits.store(mult.to_bits(), Ordering::Relaxed);
    }

    fn get_status(&self) -> AudioStatus {
        let s = self.state.load(Ordering::Relaxed);
        let state = match s {
            1 => PlaybackState::Playing,
            2 => PlaybackState::Paused,
            _ => PlaybackState::Stopped,
        };

        let pos_frames = self.position_frames.load(Ordering::Relaxed);
        let dur_frames = self.duration_frames.load(Ordering::Relaxed);
        let pos_secs = pos_frames as f64 / self.sample_rate as f64;
        let dur_secs = dur_frames as f64 / self.sample_rate as f64;

        let volume = f32::from_bits(self.volume_bits.load(Ordering::Relaxed));
        let loudness_gain_db = *self.loudness_gain_db.lock().unwrap();

        AudioStatus {
            state,
            position: Duration::from_secs_f64(pos_secs),
            duration: Duration::from_secs_f64(dur_secs),
            volume,
            loudness_gain_db,
            engine: AudioEngineKind::NativeWasapi,
            device_name: self.device_name.clone(),
            sample_rate: self.sample_rate,
            channels: self.channels,
        }
    }

    fn take_eof(&self) -> bool {
        self.eof_flag.swap(false, Ordering::Relaxed)
    }
}

