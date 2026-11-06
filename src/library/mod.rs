// AURI Library Manager Module
// Local filesystem scanning and queue coordination

pub mod queue;

use std::fs;
use std::path::Path;
use std::sync::Arc;

pub use queue::{QueueManager, RepeatMode};
use crate::provider::TrackItem;

pub struct LibraryManager {
    queue: Arc<QueueManager>,
}

impl Default for LibraryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LibraryManager {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(QueueManager::new()),
        }
    }

    pub fn queue(&self) -> &Arc<QueueManager> {
        &self.queue
    }

    /// Discovers supported local audio files across specified paths
    pub fn scan_local_files<P: AsRef<Path>>(&self, paths: &[P]) -> Vec<TrackItem> {
        let mut results = Vec::new();
        for p in paths {
            if p.as_ref().exists() {
                scan_dir_recursive(p.as_ref(), &mut results);
            }
        }
        results
    }
}

fn scan_dir_recursive(dir: &Path, out: &mut Vec<TrackItem>) {
    let supported_exts = ["flac", "mp3", "m4a", "aac", "ogg", "opus", "wav"];

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_dir_recursive(&path, out);
            } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if supported_exts.contains(&ext.to_lowercase().as_str()) {
                    let file_name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Unknown Track");

                    // Parse "Artist - Title" heuristic from filename
                    let (artist, title) = if let Some((art, tit)) = file_name.split_once(" - ") {
                        (art.trim().to_string(), tit.trim().to_string())
                    } else {
                        ("Local Artist".to_string(), file_name.to_string())
                    };

                    let format_badge = ext.to_uppercase();

                    out.push(TrackItem {
                        id: format!("local:{}", path.display()),
                        title,
                        artist,
                        album: "Local Library".to_string(),
                        duration_secs: 180, // Updated accurately on open
                        thumbnail_url: String::new(),
                        is_local: true,
                        file_path: Some(path.to_string_lossy().to_string()),
                        loudness_db: None,
                        format_badge,
                    });
                }
            }
        }
    }
}

