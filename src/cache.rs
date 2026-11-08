use std::path::PathBuf;

pub fn get_cache_dir() -> PathBuf {
    #[cfg(windows)]
    {
        if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            let dir = PathBuf::from(user_profile).join("Music").join("AURI").join("cache");
            let _ = std::fs::create_dir_all(&dir);
            return dir;
        }
    }
    let dir = std::env::temp_dir().join("auri_cache");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn get_cached_track_path(video_id: &str) -> Option<PathBuf> {
    let dir = get_cache_dir();
    let opus = dir.join(format!("{video_id}.opus"));
    if opus.exists() && std::fs::metadata(&opus).map_or(false, |m| m.len() > 10000) {
        return Some(opus);
    }
    let m4a = dir.join(format!("{video_id}.m4a"));
    if m4a.exists() && std::fs::metadata(&m4a).map_or(false, |m| m.len() > 10000) {
        return Some(m4a);
    }
    None
}

pub fn cache_size_bytes() -> u64 {
    let dir = get_cache_dir();
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total += meta.len();
                }
            }
        }
    }
    total
}

pub fn count_cached_tracks() -> usize {
    let dir = get_cache_dir();
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension() {
                if ext == "opus" || ext == "m4a" {
                    count += 1;
                }
            }
        }
    }
    count
}

pub fn purge_cache() {
    let dir = get_cache_dir();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}

pub fn trim_cache_if_needed(max_bytes: u64) {
    let dir = get_cache_dir();
    let mut files = Vec::new();
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    let len = meta.len();
                    total += len;
                    let mtime = meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    files.push((entry.path(), len, mtime));
                }
            }
        }
    }
    if total > max_bytes {
        files.sort_by_key(|(_, _, mtime)| *mtime);
        let target = max_bytes * 3 / 4;
        for (path, len, _) in files {
            if total <= target {
                break;
            }
            if std::fs::remove_file(path).is_ok() {
                total = total.saturating_sub(len);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_dir_creation() {
        let dir = get_cache_dir();
        assert!(dir.exists());
    }

    #[test]
    fn test_cache_size_calculation() {
        let size = cache_size_bytes();
        let _ = size;
        let count = count_cached_tracks();
        let _ = count;
    }
}
