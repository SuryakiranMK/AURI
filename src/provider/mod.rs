// AURI Music Provider Module
// Unified abstraction for Local filesystem and YouTube Music streaming

pub mod buffer;
pub mod innertube;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackItem {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: u64,
    pub thumbnail_url: String,
    pub is_local: bool,
    pub file_path: Option<String>,
    pub loudness_db: Option<f64>,
    pub format_badge: String,
}

impl Default for TrackItem {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: "Unknown Title".to_string(),
            artist: "Unknown Artist".to_string(),
            album: "Unknown Album".to_string(),
            duration_secs: 0,
            thumbnail_url: String::new(),
            is_local: false,
            file_path: None,
            loudness_db: None,
            format_badge: "FLAC".to_string(),
        }
    }
}

