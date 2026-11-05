// Pure Rust Symphonia Metadata Extractor
// Extracts ID3v2, Vorbis Comments, MP4/AAC tags, and FLAC metadata with zero heavy dependencies.

use std::fs::File;
use std::path::{Path, PathBuf};
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, StandardTag};

#[derive(Clone, Debug, PartialEq)]
pub struct TrackMetadata {
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: f64,
    pub year: Option<u32>,
    pub track_num: Option<u32>,
    pub has_embedded_artwork: bool,
}

impl TrackMetadata {
    pub fn new(path: PathBuf) -> Self {
        let default_title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown Track")
            .to_string();

        Self {
            path,
            title: default_title,
            artist: "Unknown Artist".to_string(),
            album: "Unknown Album".to_string(),
            duration_secs: 0.0,
            year: None,
            track_num: None,
            has_embedded_artwork: false,
        }
    }

    pub fn formatted_duration(&self) -> String {
        let total_secs = self.duration_secs.max(0.0) as u64;
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{}:{:02}", mins, secs)
    }
}

pub struct MetadataExtractor;

impl MetadataExtractor {
    pub fn extract(path: &Path) -> Result<TrackMetadata, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            hint.with_extension(ext);
        }

        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();

        let mut probed = symphonia::default::get_probe()
            .probe(&hint, mss, format_opts, metadata_opts)
            .map_err(|e| format!("Probe failed: {}", e))?;

        let mut metadata = TrackMetadata::new(path.to_path_buf());

        // 1. Calculate duration from default audio track
        if let Some(track) = probed.default_track(TrackType::Audio) {
            let time_base = track.time_base;
            if let (Some(dur), Some(tb)) = (track.duration, time_base) {
                if let Some(time) = tb.calc_duration(dur) {
                    metadata.duration_secs = time.as_secs_f64();
                }
            } else if let (Some(n_frames), Some(tb)) = (track.num_frames, time_base) {
                if let Some(time) = tb.calc_duration(symphonia::core::units::Duration::from(n_frames)) {
                    metadata.duration_secs = time.as_secs_f64();
                }
            }
        }

        // 2. Read metadata revisions
        if let Some(rev) = probed.metadata().current() {
            if !rev.media.visuals.is_empty() {
                metadata.has_embedded_artwork = true;
            }

            for tag in &rev.media.tags {
                if let Some(std_tag) = &tag.std {
                    match std_tag {
                        StandardTag::TrackTitle(val) => metadata.title = val.to_string(),
                        StandardTag::Artist(val) => metadata.artist = val.to_string(),
                        StandardTag::Album(val) => metadata.album = val.to_string(),
                        StandardTag::TrackNumber(val) => metadata.track_num = Some(*val as u32),
                        StandardTag::ReleaseDate(val) => {
                            if let Ok(year) = val.chars().take(4).collect::<String>().parse::<u32>() {
                                metadata.year = Some(year);
                            }
                        }
                        _ => {}
                    }
                } else {
                    // Raw tag fallback
                    let key = tag.raw.key.to_uppercase();
                    let val = tag.raw.value.to_string();
                    match key.as_str() {
                        "TITLE" | "TIT2" => metadata.title = val,
                        "ARTIST" | "TPE1" => metadata.artist = val,
                        "ALBUM" | "TALB" => metadata.album = val,
                        _ => {}
                    }
                }
            }
        }

        // 3. Fallback: Parse "Artist - Title" from filename if tags are generic
        if metadata.artist == "Unknown Artist" {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if let Some((artist_part, title_part)) = stem.split_once(" - ") {
                    metadata.artist = artist_part.trim().to_string();
                    metadata.title = title_part.trim().to_string();
                }
            }
        }

        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_metadata_fallback() {
        let path = PathBuf::from("C:/Music/Daft Punk - One More Time.flac");
        let meta = TrackMetadata::new(path);
        assert_eq!(meta.title, "Daft Punk - One More Time");
        assert_eq!(meta.artist, "Unknown Artist");
    }

    #[test]
    fn test_formatted_duration() {
        let mut meta = TrackMetadata::new(PathBuf::from("test.mp3"));
        meta.duration_secs = 214.0;
        assert_eq!(meta.formatted_duration(), "3:34");

        meta.duration_secs = 59.0;
        assert_eq!(meta.formatted_duration(), "0:59");
    }
}
