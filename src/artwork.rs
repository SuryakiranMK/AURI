// AURI Artwork Downloader & Disk Cache
// High-performance asynchronous thumbnail manager for YouTube Music & Local Tracks

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

pub fn get_cache_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("auri_artwork");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

/// Fetch song artwork from disk cache or download via HTTP.
/// Returns the local PathBuf to the JPEG/PNG image.
pub fn fetch_artwork(id: &str, custom_url: Option<&str>) -> Option<PathBuf> {
    if id.is_empty() {
        return None;
    }

    let cache_dir = get_cache_dir();
    let safe_id = sanitize_id(id);
    let target_file = cache_dir.join(format!("{}.jpg", safe_id));

    // Return if already cached and valid size (> 500 bytes)
    if let Ok(meta) = fs::metadata(&target_file) {
        if meta.len() > 500 {
            return Some(target_file);
        }
    }

    // Determine download URL
    let download_url = if let Some(u) = custom_url {
        if !u.is_empty() {
            u.to_string()
        } else {
            format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", id)
        }
    } else {
        format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", id)
    };

    // Download with timeout
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(5))
        .build();

    let response = match agent.get(&download_url).call() {
        Ok(resp) => resp,
        Err(_) => {
            // Fallback for YouTube standard thumb
            let fallback_url = format!("https://img.youtube.com/vi/{}/0.jpg", id);
            agent.get(&fallback_url).call().ok()?
        }
    };

    let mut bytes = Vec::new();
    let mut reader = response.into_reader();
    if std::io::copy(&mut reader, &mut bytes).is_err() || bytes.len() < 500 {
        return None;
    }

    let tmp_file = cache_dir.join(format!("{}.tmp", safe_id));
    if let Ok(mut f) = File::create(&tmp_file) {
        if f.write_all(&bytes).is_ok() {
            drop(f);
            let _ = fs::rename(&tmp_file, &target_file);
            return Some(target_file);
        }
    }

    None
}
