// AURI Audio Sink Trait & Data Structures
// Decouples the audio playback implementation from the rest of the application

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioEngineKind {
    NativeWasapi,
    StudioMpv,
}

impl AudioEngineKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            AudioEngineKind::NativeWasapi => "Native WASAPI (Ultra-Low RAM)",
            AudioEngineKind::StudioMpv => "Studio libmpv (Audiophile)",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AudioStatus {
    pub state: PlaybackState,
    pub position: Duration,
    pub duration: Duration,
    pub volume: f32,
    pub loudness_gain_db: Option<f64>,
    pub engine: AudioEngineKind,
    pub device_name: String,
    pub sample_rate: u32,
    pub channels: u16,
}

pub trait AudioSink: Send + Sync {
    fn play(&self) -> Result<(), String>;
    fn pause(&self) -> Result<(), String>;
    fn toggle(&self) -> Result<(), String>;
    fn stop(&self) -> Result<(), String>;
    fn seek(&self, position: Duration) -> Result<(), String>;
    fn set_volume(&self, volume: f32);
    fn set_loudness_gain(&self, gain_db: Option<f64>);
    fn get_status(&self) -> AudioStatus;
    fn take_eof(&self) -> bool;
}

