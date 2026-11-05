// Windows WASAPI Output Sink using CPAL
// Manages real-time audio playback stream with MMCSS scheduling.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{OutputCallbackInfo, SampleFormat, Stream, StreamConfig};
use ringbuf::traits::{Consumer, Observer, Producer, Split};
use ringbuf::HeapRb;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use crate::audio::mmcss::MmcssRegistration;

pub struct WasapiSink {
    device_name: String,
    config: StreamConfig,
    _stream: Stream,
    producer: ringbuf::wrap::caching::Caching<Arc<HeapRb<f32>>, true, false>,
    is_playing: Arc<AtomicBool>,
    volume: Arc<AtomicU32>, // f32 bits
    _mmcss: Arc<MmcssRegistration>,
}

impl WasapiSink {
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "No default audio output device available".to_string())?;

        let device_name = device
            .description()
            .map(|d| d.name().to_string())
            .unwrap_or_else(|_| "Default Audio Device".to_string());
        println!("[AURI Audio Sink] Selected device: {}", device_name);

        let default_config = device
            .default_output_config()
            .map_err(|e| format!("Failed to get default audio output config: {}", e))?;

        println!(
            "[AURI Audio Sink] Device format: {:?}, sample rate: {} Hz, channels: {}",
            default_config.sample_format(),
            default_config.sample_rate(),
            default_config.channels()
        );

        let sample_rate = default_config.sample_rate();
        let channels = default_config.channels();
        let config = StreamConfig {
            channels,
            sample_rate,
            buffer_size: cpal::BufferSize::Default,
        };

        // Ring buffer: 1 second capacity of stereo audio (e.g. 48000 * 2 = 96000 samples)
        let buffer_capacity = (config.sample_rate as usize * config.channels as usize) * 2;
        let rb = HeapRb::<f32>::new(buffer_capacity);
        let (producer, mut consumer) = rb.split();

        let is_playing = Arc::new(AtomicBool::new(false));
        let is_playing_clone = is_playing.clone();

        let volume = Arc::new(AtomicU32::new(1.0f32.to_bits()));
        let volume_clone = volume.clone();

        let mmcss = Arc::new(MmcssRegistration::register_audio());

        let err_fn = |err| eprintln!("[AURI Audio Error] Audio stream error: {}", err);

        let stream = match default_config.sample_format() {
            SampleFormat::F32 => device.build_output_stream(
                config.clone(),
                move |data: &mut [f32], _: &OutputCallbackInfo| {
                    let playing = is_playing_clone.load(Ordering::Relaxed);
                    let vol = f32::from_bits(volume_clone.load(Ordering::Relaxed));

                    for sample in data.iter_mut() {
                        if playing {
                            if let Some(s) = consumer.try_pop() {
                                *sample = s * vol;
                            } else {
                                *sample = 0.0;
                            }
                        } else {
                            *sample = 0.0;
                        }
                    }
                },
                err_fn,
                None,
            ),
            SampleFormat::I16 => device.build_output_stream(
                config.clone(),
                move |data: &mut [i16], _: &OutputCallbackInfo| {
                    let playing = is_playing_clone.load(Ordering::Relaxed);
                    let vol = f32::from_bits(volume_clone.load(Ordering::Relaxed));

                    for sample in data.iter_mut() {
                        if playing {
                            if let Some(s) = consumer.try_pop() {
                                *sample = (s * vol * 32767.0).clamp(-32768.0, 32767.0) as i16;
                            } else {
                                *sample = 0;
                            }
                        } else {
                            *sample = 0;
                        }
                    }
                },
                err_fn,
                None,
            ),
            SampleFormat::U16 => device.build_output_stream(
                config.clone(),
                move |data: &mut [u16], _: &OutputCallbackInfo| {
                    let playing = is_playing_clone.load(Ordering::Relaxed);
                    let vol = f32::from_bits(volume_clone.load(Ordering::Relaxed));

                    for sample in data.iter_mut() {
                        if playing {
                            if let Some(s) = consumer.try_pop() {
                                let scaled = (s * vol * 32767.0).clamp(-32768.0, 32767.0);
                                *sample = (scaled + 32768.0) as u16;
                            } else {
                                *sample = 32768;
                            }
                        } else {
                            *sample = 32768;
                        }
                    }
                },
                err_fn,
                None,
            ),
            format => return Err(format!("Unsupported audio sample format: {:?}", format)),
        }
        .map_err(|e| format!("Failed to build CPAL WASAPI output stream: {}", e))?;

        stream
            .play()
            .map_err(|e| format!("Failed to start WASAPI stream: {}", e))?;

        Ok(Self {
            device_name,
            config,
            _stream: stream,
            producer,
            is_playing,
            volume,
            _mmcss: mmcss,
        })
    }

    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate
    }

    pub fn channels(&self) -> usize {
        self.config.channels as usize
    }

    pub fn set_playing(&self, playing: bool) {
        self.is_playing.store(playing, Ordering::Relaxed);
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::Relaxed)
    }

    pub fn set_volume(&self, vol: f32) {
        let clamped = vol.clamp(0.0, 1.5);
        self.volume.store(clamped.to_bits(), Ordering::Relaxed);
    }

    pub fn volume(&self) -> f32 {
        f32::from_bits(self.volume.load(Ordering::Relaxed))
    }

    /// Pushes interleaved f32 audio samples into the lock-free ring buffer.
    /// Returns the number of samples actually pushed.
    pub fn push_samples(&mut self, samples: &[f32]) -> usize {
        self.producer.push_slice(samples)
    }

    pub fn vacant_samples(&self) -> usize {
        self.producer.vacant_len()
    }

    pub fn is_buffer_full(&self) -> bool {
        self.producer.is_full()
    }
}
