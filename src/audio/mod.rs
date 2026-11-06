// AURI Audio Engine Module
// High-fidelity playback coordinator with dual-engine support and real-time WASAPI output

pub mod decoder;
pub mod mmcss;
pub mod normalizer;
pub mod resampler;
pub mod sink;
pub mod wasapi_sink;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use symphonia::core::io::MediaSource;

pub use decoder::{AudioDecoder, DecodedAudioPacket};
pub use normalizer::{calculate_loudness_gain_db, SoftLimiter};
pub use resampler::CubicResampler;
pub use sink::{AudioEngineKind, AudioSink, AudioStatus, PlaybackState};
pub use wasapi_sink::WasapiSink;

enum DecodeCommand {
    OpenFile {
        path: PathBuf,
        loudness_db: Option<f64>,
    },
    OpenStream {
        source: Box<dyn MediaSource>,
        extension_hint: String,
        loudness_db: Option<f64>,
    },
    Seek(f64),
    Stop,
}

pub struct AudioEngine {
    sink: Arc<WasapiSink>,
    engine_kind: Arc<Mutex<AudioEngineKind>>,
    cmd_sender: std::sync::mpsc::Sender<DecodeCommand>,
    _worker_thread: Option<JoinHandle<()>>,
    is_decoding: Arc<AtomicBool>,
}

impl AudioEngine {
    pub fn new() -> Result<Self, String> {
        let sink = Arc::new(WasapiSink::new()?);
        let engine_kind = Arc::new(Mutex::new(AudioEngineKind::NativeWasapi));
        let (tx, rx) = std::sync::mpsc::channel::<DecodeCommand>();

        let sink_clone = Arc::clone(&sink);
        let is_decoding = Arc::new(AtomicBool::new(false));
        let is_decoding_clone = Arc::clone(&is_decoding);

        let worker = thread::Builder::new()
            .name("auri-audio-decoder".to_string())
            .spawn(move || {
                let mut current_decoder: Option<AudioDecoder> = None;

                loop {
                    // Check for incoming commands
                    if let Ok(cmd) = rx.try_recv() {
                        match cmd {
                            DecodeCommand::OpenFile { path, loudness_db } => {
                                sink_clone.clear_buffer();
                                sink_clone.set_loudness_gain(calculate_loudness_gain_db(loudness_db));
                                match AudioDecoder::open_file(&path) {
                                    Ok(dec) => {
                                        current_decoder = Some(dec);
                                        is_decoding_clone.store(true, Ordering::SeqCst);
                                        let _ = sink_clone.play();
                                    }
                                    Err(err) => {
                                        eprintln!("[AURI Audio] Failed to open audio file {:?}: {}", path, err);
                                        current_decoder = None;
                                        is_decoding_clone.store(false, Ordering::SeqCst);
                                    }
                                }
                            }
                            DecodeCommand::OpenStream {
                                source,
                                extension_hint,
                                loudness_db,
                            } => {
                                sink_clone.clear_buffer();
                                sink_clone.set_loudness_gain(calculate_loudness_gain_db(loudness_db));
                                match AudioDecoder::open_stream(source, &extension_hint) {
                                    Ok(dec) => {
                                        current_decoder = Some(dec);
                                        is_decoding_clone.store(true, Ordering::SeqCst);
                                        let _ = sink_clone.play();
                                    }
                                    Err(err) => {
                                        eprintln!("[AURI Audio] Failed to open streaming audio: {}", err);
                                        current_decoder = None;
                                        is_decoding_clone.store(false, Ordering::SeqCst);
                                    }
                                }
                            }
                            DecodeCommand::Seek(time_secs) => {
                                if let Some(ref mut dec) = current_decoder {
                                    sink_clone.clear_buffer();
                                    let _ = dec.seek(time_secs);
                                    let _ = sink_clone.seek(Duration::from_secs_f64(time_secs));
                                }
                            }
                            DecodeCommand::Stop => {
                                current_decoder = None;
                                is_decoding_clone.store(false, Ordering::SeqCst);
                                let _ = sink_clone.stop();
                            }
                        }
                    }

                    // Decode audio packets if active
                    if let Some(ref mut dec) = current_decoder {
                        match dec.next_packet() {
                            Ok(Some(packet)) => {
                                sink_clone.push_samples(
                                    &packet.samples,
                                    packet.sample_rate,
                                    packet.channels,
                                );
                            }
                            Ok(None) => {
                                // EOF reached
                                current_decoder = None;
                                is_decoding_clone.store(false, Ordering::SeqCst);
                            }
                            Err(e) => {
                                eprintln!("[AURI Audio] Decode error: {}", e);
                                current_decoder = None;
                                is_decoding_clone.store(false, Ordering::SeqCst);
                            }
                        }
                    } else {
                        // Idle sleep to conserve 0.0% CPU when no track is decoding
                        thread::sleep(Duration::from_millis(15));
                    }
                }
            })
            .map_err(|e| format!("Failed to spawn audio worker thread: {}", e))?;

        Ok(Self {
            sink,
            engine_kind,
            cmd_sender: tx,
            _worker_thread: Some(worker),
            is_decoding,
        })
    }

    pub fn play_file<P: AsRef<Path>>(&self, path: P, loudness_db: Option<f64>) {
        let _ = self.cmd_sender.send(DecodeCommand::OpenFile {
            path: path.as_ref().to_path_buf(),
            loudness_db,
        });
    }

    pub fn play_stream(
        &self,
        source: Box<dyn MediaSource>,
        extension_hint: &str,
        loudness_db: Option<f64>,
    ) {
        let _ = self.cmd_sender.send(DecodeCommand::OpenStream {
            source,
            extension_hint: extension_hint.to_string(),
            loudness_db,
        });
    }

    pub fn play_test_tone(&self, freq_hz: f32, duration_secs: f32) {
        self.sink.play_test_tone(freq_hz, duration_secs);
    }

    pub fn play(&self) -> Result<(), String> {
        self.sink.play()
    }

    pub fn pause(&self) -> Result<(), String> {
        self.sink.pause()
    }

    pub fn toggle_playback(&self) -> Result<(), String> {
        self.sink.toggle()
    }

    pub fn stop(&self) -> Result<(), String> {
        let _ = self.cmd_sender.send(DecodeCommand::Stop);
        self.sink.stop()
    }

    pub fn seek(&self, time_secs: f64) -> Result<(), String> {
        let _ = self.cmd_sender.send(DecodeCommand::Seek(time_secs));
        self.sink.seek(Duration::from_secs_f64(time_secs))
    }

    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }

    pub fn get_status(&self) -> AudioStatus {
        let mut status = self.sink.get_status();
        status.engine = *self.engine_kind.lock().unwrap();
        status
    }

    pub fn take_eof(&self) -> bool {
        self.sink.take_eof()
    }

    pub fn is_decoding(&self) -> bool {
        self.is_decoding.load(Ordering::Relaxed)
    }

    pub fn set_engine(&self, engine: AudioEngineKind) {
        *self.engine_kind.lock().unwrap() = engine;
    }
}

