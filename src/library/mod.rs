// AURI Music Library & Playback Queue Subsystem
// Milestone v0.3.0: Local Library, Metadata Parsing, 16MB LRU Cache, Dynamic Queue.

pub mod artwork;
pub mod metadata;
pub mod queue;
pub mod scanner;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
pub use artwork::{ArtworkCache, DEFAULT_MAX_ARTWORK_CACHE_BYTES};
pub use metadata::TrackMetadata;
pub use queue::{PlayQueue, RepeatMode};
pub use scanner::LibraryScanner;

pub struct LibraryManager {
    queue: Arc<Mutex<PlayQueue>>,
    artwork_cache: Arc<Mutex<ArtworkCache>>,
    library_tracks: Arc<Mutex<Vec<TrackMetadata>>>,
}

impl LibraryManager {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(PlayQueue::new())),
            artwork_cache: Arc::new(Mutex::new(ArtworkCache::new(DEFAULT_MAX_ARTWORK_CACHE_BYTES))),
            library_tracks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn queue(&self) -> Arc<Mutex<PlayQueue>> {
        self.queue.clone()
    }

    pub fn artwork_cache(&self) -> Arc<Mutex<ArtworkCache>> {
        self.artwork_cache.clone()
    }

    pub fn library_tracks(&self) -> Arc<Mutex<Vec<TrackMetadata>>> {
        self.library_tracks.clone()
    }

    /// Scans standard and custom music paths, populating the library.
    pub fn scan_directories(&self, dirs: &[PathBuf]) -> usize {
        let mut total_found = Vec::new();
        for dir in dirs {
            if dir.exists() {
                let tracks = LibraryScanner::scan_dir(dir);
                total_found.extend(tracks);
            }
        }

        let count = total_found.len();
        *self.library_tracks.lock().unwrap() = total_found.clone();

        // If queue is empty, populate queue with discovered library
        let mut q = self.queue.lock().unwrap();
        if q.is_empty() && !total_found.is_empty() {
            q.set_queue(total_found, 0);
        }

        count
    }

    /// Appends a single file or directory to the active library and queue.
    pub fn add_path(&self, path: &Path) {
        if path.is_file() && LibraryScanner::is_supported_audio(path) {
            let track = metadata::MetadataExtractor::extract(path)
                .unwrap_or_else(|_| TrackMetadata::new(path.to_path_buf()));
            self.library_tracks.lock().unwrap().push(track.clone());
            self.queue.lock().unwrap().add_track(track);
        } else if path.is_dir() {
            let tracks = LibraryScanner::scan_dir(path);
            for t in tracks {
                self.library_tracks.lock().unwrap().push(t.clone());
                self.queue.lock().unwrap().add_track(t);
            }
        }
    }
}

