// Bounded LRU Album Artwork Cache
// Strictly enforces max 16 MB RAM budget as specified in PERFORMANCE_TARGETS.md.

use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

pub const DEFAULT_MAX_ARTWORK_CACHE_BYTES: usize = 16 * 1024 * 1024; // 16 MB

pub struct ArtworkCache {
    max_bytes: usize,
    current_bytes: usize,
    entries: HashMap<PathBuf, Arc<Vec<u8>>>,
    lru_order: VecDeque<PathBuf>,
}

impl ArtworkCache {
    pub fn new(max_bytes: usize) -> Self {
        Self {
            max_bytes,
            current_bytes: 0,
            entries: HashMap::new(),
            lru_order: VecDeque::new(),
        }
    }

    pub fn current_bytes(&self) -> usize {
        self.current_bytes
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Retrieves cached artwork bytes or attempts to load from embedded visual / neighbor file.
    pub fn get_or_load(&mut self, audio_path: &Path) -> Option<Arc<Vec<u8>>> {
        let key = audio_path.to_path_buf();

        // 1. Cache hit
        if let Some(cached) = self.entries.get(&key).cloned() {
            // Update LRU position: move key to the back of lru_order
            self.touch(&key);
            return Some(cached);
        }

        // 2. Cache miss: extract from audio file or adjacent image
        if let Some(data) = Self::extract_artwork(audio_path) {
            let data_size = data.len();

            // Evict until there is space
            while self.current_bytes + data_size > self.max_bytes && !self.lru_order.is_empty() {
                if let Some(oldest_key) = self.lru_order.pop_front() {
                    if let Some(removed) = self.entries.remove(&oldest_key) {
                        self.current_bytes = self.current_bytes.saturating_sub(removed.len());
                    }
                }
            }

            let shared_data = Arc::new(data);
            self.current_bytes += data_size;
            self.entries.insert(key.clone(), shared_data.clone());
            self.lru_order.push_back(key);

            Some(shared_data)
        } else {
            None
        }
    }

    /// Stores pre-loaded artwork directly into cache (useful for testing or stream cover fetches).
    pub fn insert(&mut self, key: PathBuf, data: Vec<u8>) {
        let data_size = data.len();
        while self.current_bytes + data_size > self.max_bytes && !self.lru_order.is_empty() {
            if let Some(oldest_key) = self.lru_order.pop_front() {
                if let Some(removed) = self.entries.remove(&oldest_key) {
                    self.current_bytes = self.current_bytes.saturating_sub(removed.len());
                }
            }
        }

        self.current_bytes += data_size;
        let shared = Arc::new(data);
        self.entries.insert(key.clone(), shared);
        self.lru_order.push_back(key);
    }

    fn touch(&mut self, key: &Path) {
        if let Some(pos) = self.lru_order.iter().position(|k| k == key) {
            self.lru_order.remove(pos);
            self.lru_order.push_back(key.to_path_buf());
        }
    }

    /// Extracts embedded artwork from the audio file, or falls back to folder cover art.
    pub fn extract_artwork(audio_path: &Path) -> Option<Vec<u8>> {
        // Attempt 1: Check embedded container metadata
        if let Ok(file) = File::open(audio_path) {
            let mss = MediaSourceStream::new(Box::new(file), Default::default());
            let mut hint = Hint::new();
            if let Some(ext) = audio_path.extension().and_then(|s| s.to_str()) {
                hint.with_extension(ext);
            }

            let format_opts = FormatOptions::default();
            let metadata_opts = MetadataOptions::default();

            if let Ok(mut probed) = symphonia::default::get_probe().probe(&hint, mss, format_opts, metadata_opts) {
                if let Some(rev) = probed.metadata().current() {
                    if let Some(visual) = rev.media.visuals.first() {
                        return Some(visual.data.to_vec());
                    }
                }
            }
        }

        // Attempt 2: Check folder neighbor artwork files
        if let Some(parent) = audio_path.parent() {
            let candidate_names = [
                "cover.jpg", "cover.png", "folder.jpg", "folder.png",
                "album.jpg", "album.png", "artwork.jpg", "front.jpg",
            ];

            for name in candidate_names {
                let candidate = parent.join(name);
                if candidate.is_file() {
                    if let Ok(mut img_file) = File::open(&candidate) {
                        let mut buf = Vec::new();
                        if img_file.read_to_end(&mut buf).is_ok() && !buf.is_empty() {
                            return Some(buf);
                        }
                    }
                }
            }
        }

        None
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.lru_order.clear();
        self.current_bytes = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artwork_cache_eviction() {
        // Create cache with 1000 bytes max
        let mut cache = ArtworkCache::new(1000);

        let key1 = PathBuf::from("track1.mp3");
        let key2 = PathBuf::from("track2.mp3");
        let key3 = PathBuf::from("track3.mp3");

        // Insert 400 bytes
        cache.insert(key1.clone(), vec![0u8; 400]);
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.current_bytes(), 400);

        // Insert another 400 bytes -> total 800 <= 1000
        cache.insert(key2.clone(), vec![0u8; 400]);
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.current_bytes(), 800);

        // Insert 400 bytes -> total 1200 > 1000, key1 must be evicted
        cache.insert(key3.clone(), vec![0u8; 400]);
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.current_bytes(), 800);

        // Key1 should no longer be present
        assert!(!cache.entries.contains_key(&key1));
        assert!(cache.entries.contains_key(&key2));
        assert!(cache.entries.contains_key(&key3));
    }
}
