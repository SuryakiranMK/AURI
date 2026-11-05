// Project AURI: Music Provider Subsystem
// Abstract provider trait and audio streaming interfaces.

pub mod buffer;
pub mod innertube;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioStreamFormat {
    WebmOpus,
    Mp4Aac,
    Flac,
    Mp3,
    Wav,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedStream {
    pub video_id: String,
    pub stream_url: String,
    pub format: AudioStreamFormat,
    pub bitrate_bps: u32,
    pub duration_secs: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: f64,
    pub formatted_duration: String,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ProviderError {
    Network(String),
    Parse(String),
    Unavailable(String),
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(e) => write!(f, "Network error: {}", e),
            Self::Parse(e) => write!(f, "Parse error: {}", e),
            Self::Unavailable(e) => write!(f, "Track unavailable: {}", e),
        }
    }
}

impl std::error::Error for ProviderError {}

/// Abstract music provider trait (YouTube Music, Local, future Spotify/Soundcloud)
pub trait MusicProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn search(&self, query: &str) -> Result<Vec<SearchResultItem>, ProviderError>;
    fn resolve_stream(&self, track_id: &str) -> Result<ResolvedStream, ProviderError>;
}

