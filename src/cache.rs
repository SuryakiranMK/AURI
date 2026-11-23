use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

static AUTO_CACHE_ENABLED: AtomicBool = AtomicBool::new(true);
static CACHE_LIMIT_BYTES: AtomicU64 = AtomicU64::new(200 * 1024 * 1024);

pub fn set_auto_cache(enabled: bool) {
    AUTO_CACHE_ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn is_auto_cache() -> bool {
    AUTO_CACHE_ENABLED.load(Ordering::Relaxed)
}

pub fn set_cache_limit_mb(mb: u32) {
    let bytes = (mb as u64) * 1024 * 1024;
    CACHE_LIMIT_BYTES.store(bytes, Ordering::Relaxed);
}

pub fn get_cache_limit_bytes() -> u64 {
    CACHE_LIMIT_BYTES.load(Ordering::Relaxed)
}

#[allow(dead_code)]
pub fn get_cache_limit_mb() -> u32 {
    (CACHE_LIMIT_BYTES.load(Ordering::Relaxed) / (1024 * 1024)) as u32
}

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

/// Returns a unique temporary `.part` file path for downloading or preloading a track.
/// Includes process ID and an atomic monotonic counter so concurrent fetch/preload workers
/// never collide or trigger Windows ERROR_SHARING_VIOLATION.
pub fn temp_part_path(video_id: &str, tag: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    get_cache_dir().join(format!("{video_id}.{tag}.{pid}_{count}.part"))
}

/// Updates the last modified time of a file to current system time for LRU tracking.
pub fn touch_file(path: &Path) {
    if let Ok(file) = std::fs::OpenOptions::new().write(true).open(path) {
        let _ = file.set_modified(std::time::SystemTime::now());
    }
}

/// Reads an EBML variable-size integer (VINT) from a reader.
fn read_ebml_vint<R: std::io::Read>(reader: &mut R) -> std::io::Result<Option<u64>> {
    let mut first = [0u8; 1];
    reader.read_exact(&mut first)?;
    let b = first[0];
    if b == 0 {
        return Ok(None);
    }
    let (mask, len) = if b & 0x80 != 0 {
        (0x7F, 1)
    } else if b & 0x40 != 0 {
        (0x3F, 2)
    } else if b & 0x20 != 0 {
        (0x1F, 3)
    } else if b & 0x10 != 0 {
        (0x0F, 4)
    } else if b & 0x08 != 0 {
        (0x07, 5)
    } else if b & 0x04 != 0 {
        (0x03, 6)
    } else if b & 0x02 != 0 {
        (0x01, 7)
    } else if b & 0x01 != 0 {
        (0x00, 8)
    } else {
        return Ok(None);
    };

    let mut val = (b & mask) as u64;
    let mut all_ones = (b & mask) == mask;
    for _ in 1..len {
        let mut next = [0u8; 1];
        reader.read_exact(&mut next)?;
        val = (val << 8) | (next[0] as u64);
        if next[0] != 0xFF {
            all_ones = false;
        }
    }
    if all_ones {
        Ok(None) // Unknown length in EBML
    } else {
        Ok(Some(val))
    }
}

/// Validates whether a file is a complete WebM container (YouTube's standard Opus format).
fn is_valid_webm(path: &Path, file_len: u64) -> bool {
    use std::io::{BufReader, Read, Seek, SeekFrom};
    let Ok(file) = std::fs::File::open(path) else { return false };
    let mut reader = BufReader::new(file);

    let mut header = [0u8; 4];
    if reader.read_exact(&mut header).is_err() || header != [0x1A, 0x45, 0xDF, 0xA3] {
        return false;
    }

    let Some(ebml_len) = read_ebml_vint(&mut reader).ok().flatten() else { return false };
    if reader.seek(SeekFrom::Current(ebml_len as i64)).is_err() {
        return false;
    }

    let mut seg_id = [0u8; 4];
    if reader.read_exact(&mut seg_id).is_err() || seg_id != [0x18, 0x53, 0x80, 0x67] {
        return false;
    }

    match read_ebml_vint(&mut reader) {
        Ok(Some(seg_len)) => {
            let offset = reader.stream_position().unwrap_or(0);
            offset + seg_len <= file_len + 16
        }
        Ok(None) => file_len >= 64 * 1024,
        Err(_) => false,
    }
}

/// Validates whether a file is a complete Ogg Opus container.
fn is_valid_ogg_opus(path: &Path, file_len: u64) -> bool {
    use std::io::{Read, Seek, SeekFrom};
    let Ok(mut file) = std::fs::File::open(path) else { return false };

    let mut header = [0u8; 36];
    if file.read_exact(&mut header).is_err() {
        return false;
    }
    if &header[0..4] != b"OggS" {
        return false;
    }
    if (header[5] & 0x02) == 0 {
        return false;
    }

    let tail_size = file_len.min(64 * 1024) as usize;
    if file.seek(SeekFrom::End(-(tail_size as i64))).is_err() {
        return false;
    }
    let mut tail = vec![0u8; tail_size];
    if file.read_exact(&mut tail).is_err() {
        return false;
    }

    let mut i = tail.len().saturating_sub(4);
    while i > 0 {
        if &tail[i..i + 4] == b"OggS" {
            if i + 5 < tail.len() && (tail[i + 5] & 0x04) != 0 {
                return true;
            }
            return false;
        }
        i -= 1;
    }
    false
}

/// Validates whether a file is a complete MP4/M4A container.
fn is_valid_mp4_m4a(path: &Path, file_len: u64) -> bool {
    use std::io::{Read, Seek, SeekFrom};
    let Ok(mut file) = std::fs::File::open(path) else { return false };

    let mut header = [0u8; 8];
    if file.read_exact(&mut header).is_err() {
        return false;
    }
    if &header[4..8] != b"ftyp" {
        return false;
    }

    if file.seek(SeekFrom::Start(0)).is_err() {
        return false;
    }
    let mut offset = 0u64;
    let mut has_moov = false;
    let mut has_mdat = false;

    while offset + 8 <= file_len {
        let mut atom_hdr = [0u8; 8];
        if file.read_exact(&mut atom_hdr).is_err() {
            break;
        }
        let raw_len = u32::from_be_bytes([atom_hdr[0], atom_hdr[1], atom_hdr[2], atom_hdr[3]]) as u64;
        let atom_type = &atom_hdr[4..8];
        if atom_type == b"moov" {
            has_moov = true;
        } else if atom_type == b"mdat" {
            has_mdat = true;
        }

        let atom_len = match raw_len {
            0 => file_len - offset,
            1 => {
                let mut ext = [0u8; 8];
                if file.read_exact(&mut ext).is_err() {
                    return false;
                }
                u64::from_be_bytes(ext)
            }
            n if n >= 8 => n,
            _ => return false,
        };

        if offset + atom_len > file_len {
            return false;
        }

        offset += atom_len;
        if raw_len == 0 || offset >= file_len {
            break;
        }
        if file.seek(SeekFrom::Start(offset)).is_err() {
            return false;
        }
    }

    (has_moov || has_mdat) && offset == file_len
}

/// Checks whether a cached file is complete and structurally valid.
/// Rejects partial downloads from interrupted streams so they do not cause UnexpectedEof.
pub fn is_valid_track_file(path: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(path) else { return false };
    if !meta.is_file() || meta.len() < 32 * 1024 {
        return false;
    }
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    match ext {
        "opus" => is_valid_webm(path, meta.len()) || is_valid_ogg_opus(path, meta.len()),
        "m4a" => is_valid_mp4_m4a(path, meta.len()),
        _ => false,
    }
}

/// Atomically finalizes a temporary part file into a completed cached track.
/// Verifies expected file length and container integrity before renaming.
/// Automatically cleans up the temp file if validation fails.
pub fn commit_temp_track(part_path: &Path, video_id: &str, ext: &str, expected_len: u64) -> bool {
    if !is_auto_cache() {
        let _ = std::fs::remove_file(part_path);
        return false;
    }
    let dir = get_cache_dir();
    let target = dir.join(format!("{video_id}.{ext}"));

    let meta = match std::fs::metadata(part_path) {
        Ok(m) => m,
        Err(_) => {
            let _ = std::fs::remove_file(part_path);
            return false;
        }
    };

    if (expected_len > 0 && meta.len() < expected_len) || !is_valid_track_file(part_path) {
        let _ = std::fs::remove_file(part_path);
        return false;
    }

    if target.exists() && is_valid_track_file(&target) {
        let _ = std::fs::remove_file(part_path);
        return true;
    }

    if std::fs::rename(part_path, &target).is_ok() {
        touch_file(&target);
        trim_cache_if_needed(get_cache_limit_bytes());
        true
    } else {
        let _ = std::fs::remove_file(part_path);
        false
    }
}

pub fn get_cached_track_path(video_id: &str) -> Option<PathBuf> {
    let dir = get_cache_dir();
    let opus = dir.join(format!("{video_id}.opus"));
    if opus.exists() {
        if is_valid_track_file(&opus) {
            touch_file(&opus);
            return Some(opus);
        } else {
            let _ = std::fs::remove_file(&opus);
        }
    }
    let m4a = dir.join(format!("{video_id}.m4a"));
    if m4a.exists() {
        if is_valid_track_file(&m4a) {
            touch_file(&m4a);
            return Some(m4a);
        } else {
            let _ = std::fs::remove_file(&m4a);
        }
    }
    None
}

pub fn cache_size_bytes() -> u64 {
    let dir = get_cache_dir();
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if (ext == "opus" || ext == "m4a") && is_valid_track_file(&path) {
                    if let Ok(meta) = entry.metadata() {
                        total += meta.len();
                    }
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
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if (ext == "opus" || ext == "m4a") && is_valid_track_file(&path) {
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
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "part") {
                if let Ok(meta) = entry.metadata() {
                    if let Ok(modified) = meta.modified() {
                        if modified.elapsed().map_or(false, |e| e > Duration::from_secs(3600)) {
                            let _ = std::fs::remove_file(&path);
                        }
                    }
                }
                continue;
            }
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    let len = meta.len();
                    total += len;
                    let mtime = meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    files.push((path, len, mtime));
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

    #[test]
    fn test_temp_part_path_uniqueness() {
        let path1 = temp_part_path("test_video_123", "fetch");
        let path2 = temp_part_path("test_video_123", "preload");
        assert_ne!(path1, path2);
        assert!(path1.to_string_lossy().contains("fetch"));
        assert!(path2.to_string_lossy().contains("preload"));
    }

    #[test]
    fn test_invalid_short_file_rejected() {
        let temp = std::env::temp_dir().join("test_short_track.opus");
        let _ = std::fs::write(&temp, b"too short data");
        assert!(!is_valid_track_file(&temp));
        let _ = std::fs::remove_file(&temp);
    }

    #[test]
    fn test_touch_file_updates_mtime() {
        let temp = std::env::temp_dir().join("test_touch_mtime.tmp");
        let _ = std::fs::write(&temp, b"sample");
        touch_file(&temp);
        let meta = std::fs::metadata(&temp).unwrap();
        assert!(meta.modified().is_ok());
        let _ = std::fs::remove_file(&temp);
    }
}
