// AURI Audio Engine Architecture
// Pure Rust Decoders (Symphonia) + WASAPI Real-Time Sink (CPAL) + MMCSS Priority

pub mod mmcss;
pub mod resampler;
pub mod decoder;
pub mod generator;
pub mod wasapi_sink;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::audio::decoder::AudioDecoder;
use crate::audio::generator::ToneGenerator;
use crate::audio::resampler::LinearResampler;
use crate::audio::wasapi_sink::WasapiSink;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone)]
pub struct AudioStatus {
    pub state: PlaybackState,
    pub position_secs: f64,
    pub duration_secs: f64,
    pub volume: f32,
    pub track_title: String,
    pub track_artist: String,
    pub sample_rate: u32,
    pub channels: usize,
    pub device_name: String,
}

enum AudioCommand {
    PlayFile(PathBuf),
    PlayTone { freq_hz: f32, duration_secs: f32 },
    Pause,
    Resume,
    Toggle,
    Stop,
    Seek(f64),
    SetVolume(f32),
}

enum AudioSource {
    None,
    Decoder(AudioDecoder),
    Tone(ToneGenerator),
}

pub struct AudioEngine {
    cmd_tx: Sender<AudioCommand>,
    state: Arc<Mutex<PlaybackState>>,
    position: Arc<AtomicU32>, // f32 bits of position in seconds
    duration: Arc<AtomicU32>, // f32 bits of duration in seconds
    volume: Arc<AtomicU32>,   // f32 bits
    current_title: Arc<Mutex<String>>,
    current_artist: Arc<Mutex<String>>,
    device_name: String,
    sink_sample_rate: u32,
    sink_channels: usize,
    _worker: Option<JoinHandle<()>>,
}

impl AudioEngine {
    pub fn new() -> Result<Self, String> {
        let (cmd_tx, cmd_rx) = channel::<AudioCommand>();

        let state = Arc::new(Mutex::new(PlaybackState::Stopped));
        let position = Arc::new(AtomicU32::new(0.0f32.to_bits()));
        let duration = Arc::new(AtomicU32::new(0.0f32.to_bits()));
        let volume = Arc::new(AtomicU32::new(0.75f32.to_bits()));
        let current_title = Arc::new(Mutex::new("No Track Loaded".to_string()));
        let current_artist = Arc::new(Mutex::new("".to_string()));

        let sink = WasapiSink::new()?;
        let device_name = sink.device_name().to_string();
        let sink_sample_rate = sink.sample_rate();
        let sink_channels = sink.channels();

        let state_clone = state.clone();
        let position_clone = position.clone();
        let duration_clone = duration.clone();
        let volume_clone = volume.clone();
        let title_clone = current_title.clone();
        let artist_clone = current_artist.clone();

        sink.set_volume(0.75);

        let worker = thread::Builder::new()
            .name("auri-audio-worker".to_string())
            .spawn(move || {
                Self::audio_worker_loop(
                    sink,
                    cmd_rx,
                    state_clone,
                    position_clone,
                    duration_clone,
                    volume_clone,
                    title_clone,
                    artist_clone,
                );
            })
            .map_err(|e| format!("Failed to spawn audio worker thread: {}", e))?;

        Ok(Self {
            cmd_tx,
            state,
            position,
            duration,
            volume,
            current_title,
            current_artist,
            device_name,
            sink_sample_rate,
            sink_channels,
            _worker: Some(worker),
        })
    }

    pub fn play_file(&self, path: impl AsRef<Path>) {
        let _ = self.cmd_tx.send(AudioCommand::PlayFile(path.as_ref().to_path_buf()));
    }

    pub fn play_test_tone(&self, freq_hz: f32, duration_secs: f32) {
        let _ = self.cmd_tx.send(AudioCommand::PlayTone { freq_hz, duration_secs });
    }

    pub fn pause(&self) {
        let _ = self.cmd_tx.send(AudioCommand::Pause);
    }

    pub fn resume(&self) {
        let _ = self.cmd_tx.send(AudioCommand::Resume);
    }

    pub fn toggle_playback(&self) {
        let _ = self.cmd_tx.send(AudioCommand::Toggle);
    }

    pub fn stop(&self) {
        let _ = self.cmd_tx.send(AudioCommand::Stop);
    }

    pub fn seek(&self, position_secs: f64) {
        let _ = self.cmd_tx.send(AudioCommand::Seek(position_secs));
    }

    pub fn set_volume(&self, vol: f32) {
        let _ = self.cmd_tx.send(AudioCommand::SetVolume(vol));
    }

    pub fn get_status(&self) -> AudioStatus {
        let state = *self.state.lock().unwrap();
        let position_secs = f32::from_bits(self.position.load(Ordering::Relaxed)) as f64;
        let duration_secs = f32::from_bits(self.duration.load(Ordering::Relaxed)) as f64;
        let volume = f32::from_bits(self.volume.load(Ordering::Relaxed));
        let track_title = self.current_title.lock().unwrap().clone();
        let track_artist = self.current_artist.lock().unwrap().clone();

        AudioStatus {
            state,
            position_secs,
            duration_secs,
            volume,
            track_title,
            track_artist,
            sample_rate: self.sink_sample_rate,
            channels: self.sink_channels,
            device_name: self.device_name.clone(),
        }
    }

    fn audio_worker_loop(
        mut sink: WasapiSink,
        cmd_rx: Receiver<AudioCommand>,
        state: Arc<Mutex<PlaybackState>>,
        position: Arc<AtomicU32>,
        duration: Arc<AtomicU32>,
        volume: Arc<AtomicU32>,
        current_title: Arc<Mutex<String>>,
        current_artist: Arc<Mutex<String>>,
    ) {
        let sink_sample_rate = sink.sample_rate();
        let sink_channels = sink.channels();

        let mut source = AudioSource::None;
        let mut resampler = LinearResampler::new(sink_sample_rate, sink_sample_rate, sink_channels);
        let mut resample_buf = Vec::with_capacity(4096);

        loop {
            // Process all pending incoming commands
            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    AudioCommand::PlayFile(path) => {
                        println!("[AURI Audio] Opening file: {:?}", path);
                        match AudioDecoder::open_file(&path) {
                            Ok(decoder) => {
                                let total_dur = decoder.duration_secs();
                                duration.store((total_dur as f32).to_bits(), Ordering::Relaxed);
                                position.store(0.0f32.to_bits(), Ordering::Relaxed);

                                if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                                    *current_title.lock().unwrap() = file_name.to_string();
                                    *current_artist.lock().unwrap() = "Local File".to_string();
                                }

                                resampler = LinearResampler::new(decoder.sample_rate(), sink_sample_rate, sink_channels);
                                source = AudioSource::Decoder(decoder);

                                *state.lock().unwrap() = PlaybackState::Playing;
                                sink.set_playing(true);
                            }
                            Err(e) => {
                                eprintln!("[AURI Audio Error] Failed to decode file: {}", e);
                            }
                        }
                    }
                    AudioCommand::PlayTone { freq_hz, duration_secs } => {
                        println!("[AURI Audio] Generating test tone: {} Hz, {}s", freq_hz, duration_secs);
                        let gen = ToneGenerator::new(freq_hz, sink_sample_rate, duration_secs);
                        duration.store(duration_secs.to_bits(), Ordering::Relaxed);
                        position.store(0.0f32.to_bits(), Ordering::Relaxed);

                        *current_title.lock().unwrap() = format!("AURI Test Tone ({} Hz)", freq_hz as u32);
                        *current_artist.lock().unwrap() = "Pure Audio Core".to_string();

                        source = AudioSource::Tone(gen);
                        *state.lock().unwrap() = PlaybackState::Playing;
                        sink.set_playing(true);
                    }
                    AudioCommand::Pause => {
                        *state.lock().unwrap() = PlaybackState::Paused;
                        sink.set_playing(false);
                    }
                    AudioCommand::Resume => {
                        *state.lock().unwrap() = PlaybackState::Playing;
                        sink.set_playing(true);
                    }
                    AudioCommand::Toggle => {
                        let mut st = state.lock().unwrap();
                        if *st == PlaybackState::Playing {
                            *st = PlaybackState::Paused;
                            sink.set_playing(false);
                        } else {
                            *st = PlaybackState::Playing;
                            sink.set_playing(true);
                        }
                    }
                    AudioCommand::Stop => {
                        *state.lock().unwrap() = PlaybackState::Stopped;
                        sink.set_playing(false);
                        source = AudioSource::None;
                        position.store(0.0f32.to_bits(), Ordering::Relaxed);
                    }
                    AudioCommand::Seek(seconds) => {
                        match &mut source {
                            AudioSource::Decoder(dec) => {
                                let _ = dec.seek(seconds);
                                position.store((seconds as f32).to_bits(), Ordering::Relaxed);
                            }
                            AudioSource::Tone(gen) => {
                                gen.seek(seconds);
                                position.store((seconds as f32).to_bits(), Ordering::Relaxed);
                            }
                            AudioSource::None => {}
                        }
                    }
                    AudioCommand::SetVolume(vol) => {
                        volume.store(vol.to_bits(), Ordering::Relaxed);
                        sink.set_volume(vol);
                    }
                }
            }

            // If playing, pump audio frames into the sink
            let is_currently_playing = *state.lock().unwrap() == PlaybackState::Playing;

            if is_currently_playing {
                let vacant = sink.vacant_samples();
                if vacant > 2048 {
                    match &mut source {
                        AudioSource::Decoder(dec) => {
                            match dec.decode_next() {
                                Ok(Some(samples)) => {
                                    resample_buf.clear();
                                    resampler.resample(&samples, &mut resample_buf);
                                    sink.push_samples(&resample_buf);

                                    let cur_pos = dec.current_time_secs();
                                    position.store((cur_pos as f32).to_bits(), Ordering::Relaxed);
                                }
                                Ok(None) => {
                                    // End of track reached
                                    println!("[AURI Audio] Track playback completed");
                                    *state.lock().unwrap() = PlaybackState::Stopped;
                                    sink.set_playing(false);
                                    source = AudioSource::None;
                                }
                                Err(e) => {
                                    eprintln!("[AURI Audio Error] Decode failed: {}", e);
                                    *state.lock().unwrap() = PlaybackState::Stopped;
                                    sink.set_playing(false);
                                    source = AudioSource::None;
                                }
                            }
                        }
                        AudioSource::Tone(gen) => {
                            if let Some(samples) = gen.next_chunk(1024) {
                                sink.push_samples(&samples);
                                let cur_pos = gen.current_time_secs();
                                position.store((cur_pos as f32).to_bits(), Ordering::Relaxed);
                            } else {
                                println!("[AURI Audio] Tone playback completed");
                                *state.lock().unwrap() = PlaybackState::Stopped;
                                sink.set_playing(false);
                                source = AudioSource::None;
                            }
                        }
                        AudioSource::None => {
                            thread::sleep(Duration::from_millis(20));
                        }
                    }
                } else {
                    // Buffer healthy, sleep briefly to yield CPU
                    thread::sleep(Duration::from_millis(6));
                }
            } else {
                // Idle / Paused: Sleep to ensure 0% CPU consumption
                thread::sleep(Duration::from_millis(25));
            }
        }
    }
}
