// Recursive Local Music File Scanner
// Discovers FLAC, MP3, AAC, M4A, Opus, OGG, and WAV tracks.

use std::fs;
use std::path::{Path, PathBuf};
use crate::library::metadata::{MetadataExtractor, TrackMetadata};

const SUPPORTED_EXTENSIONS: &[&str] = &["mp3", "flac", "m4a", "aac", "ogg", "opus", "wav"];

pub struct LibraryScanner;

impl LibraryScanner {
    /// Returns default OS music directories (e.g. %USERPROFILE%\Music on Windows).
    pub fn default_music_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        #[cfg(windows)]
        {
            if let Ok(userprofile) = std::env::var("USERPROFILE") {
                let music_dir = PathBuf::from(userprofile).join("Music");
                if music_dir.exists() && music_dir.is_dir() {
                    dirs.push(music_dir);
                }
            }
        }

        // Also check local sample directory if present
        let local_samples = PathBuf::from("samples");
        if local_samples.exists() && local_samples.is_dir() {
            dirs.push(local_samples);
        }

        dirs
    }

    /// Recursively scans a directory and extracts metadata for all supported audio files.
    pub fn scan_dir(dir: &Path) -> Vec<TrackMetadata> {
        let mut results = Vec::new();
        Self::scan_recursive(dir, &mut results);
        results
    }

    fn scan_recursive(dir: &Path, results: &mut Vec<TrackMetadata>) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::scan_recursive(&path, results);
            } else if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    let ext_lower = ext.to_ascii_lowercase();
                    if SUPPORTED_EXTENSIONS.contains(&ext_lower.as_str()) {
                        // Extract metadata or construct minimal fallback
                        let track = MetadataExtractor::extract(&path)
                            .unwrap_or_else(|_| TrackMetadata::new(path));
                        results.push(track);
                    }
                }
            }
        }
    }

    /// Checks if a file path has a supported audio extension.
    pub fn is_supported_audio(path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            SUPPORTED_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str())
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported_audio() {
        assert!(LibraryScanner::is_supported_audio(Path::new("song.flac")));
        assert!(LibraryScanner::is_supported_audio(Path::new("song.MP3")));
        assert!(LibraryScanner::is_supported_audio(Path::new("song.m4a")));
        assert!(LibraryScanner::is_supported_audio(Path::new("song.opus")));
        assert!(!LibraryScanner::is_supported_audio(Path::new("video.mp4")));
        assert!(!LibraryScanner::is_supported_audio(Path::new("document.pdf")));
    }
}

