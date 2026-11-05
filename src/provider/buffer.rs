// Project AURI: Sliding Stream Buffer
// Bounded 8 MB in-memory stream buffer implementing std::io::Read + Seek.
// Conforms strictly to PERFORMANCE_TARGETS.md (< 8 MB streaming buffer).

use std::collections::BTreeMap;
use std::io::{Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const CHUNK_SIZE: usize = 256 * 1024; // 256 KB network chunk
const MAX_BUFFER_CAPACITY: usize = 8 * 1024 * 1024; // 8 MB max sliding window

/// Thread-safe in-memory sliding buffer for progressive audio streaming
pub struct SlidingStreamBuffer {
    stream_url: String,
    total_size: Option<u64>,
    cursor: u64,
    chunks: BTreeMap<u64, Vec<u8>>,
    buffered_bytes: usize,
    is_cancelled: Arc<AtomicBool>,
}

impl SlidingStreamBuffer {
    /// Creates a new streaming buffer for the given direct audio URL
    pub fn new(stream_url: String, total_size: Option<u64>) -> Self {
        Self {
            stream_url,
            total_size,
            cursor: 0,
            chunks: BTreeMap::new(),
            buffered_bytes: 0,
            is_cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Pre-buffers the initial chunk (e.g. 512 KB) for instant playback startup
    pub fn prefetch_initial(&mut self) -> Result<(), std::io::Error> {
        let chunk_index = 0;
        self.ensure_chunk(chunk_index)?;
        // Prefetch second chunk in background or sequentially
        let _ = self.ensure_chunk(1);
        Ok(())
    }

    /// Returns the cancellation token handle
    pub fn cancel_token(&self) -> Arc<AtomicBool> {
        self.is_cancelled.clone()
    }

    /// Fetches a specific chunk by index using HTTP Range request
    fn fetch_range(&self, start: u64, end: u64) -> Result<Vec<u8>, std::io::Error> {
        if self.is_cancelled.load(Ordering::Relaxed) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "Stream fetch cancelled",
            ));
        }

        let range_header = format!("bytes={}-{}", start, end);
        let resp = ureq::get(&self.stream_url)
            .header("Range", &range_header)
            .call()
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::ConnectionReset,
                    format!("HTTP range fetch failed: {}", e),
                )
            })?;

        let mut body = resp.into_body();
        let mut buffer = Vec::with_capacity((end - start + 1) as usize);
        body.as_reader().read_to_end(&mut buffer)?;

        Ok(buffer)
    }

    /// Ensures that the specified chunk index is available in memory
    fn ensure_chunk(&mut self, chunk_index: u64) -> Result<(), std::io::Error> {
        let chunk_start = chunk_index * CHUNK_SIZE as u64;

        if self.chunks.contains_key(&chunk_start) {
            return Ok(());
        }

        let chunk_end = if let Some(total) = self.total_size {
            (chunk_start + CHUNK_SIZE as u64 - 1).min(total.saturating_sub(1))
        } else {
            chunk_start + CHUNK_SIZE as u64 - 1
        };

        if chunk_start > chunk_end {
            return Ok(());
        }

        let data = self.fetch_range(chunk_start, chunk_end)?;
        let data_len = data.len();

        // Enforce max 8 MB sliding window by evicting chunks far behind the cursor
        while self.buffered_bytes + data_len > MAX_BUFFER_CAPACITY && !self.chunks.is_empty() {
            // Find chunk furthest behind current cursor
            let evict_key = self
                .chunks
                .keys()
                .filter(|&&k| k + (CHUNK_SIZE as u64) < self.cursor)
                .next()
                .cloned();

            if let Some(key) = evict_key {
                if let Some(removed) = self.chunks.remove(&key) {
                    self.buffered_bytes = self.buffered_bytes.saturating_sub(removed.len());
                }
            } else {
                // If no chunks behind cursor, evict earliest available chunk
                let first_key = *self.chunks.keys().next().unwrap();
                if let Some(removed) = self.chunks.remove(&first_key) {
                    self.buffered_bytes = self.buffered_bytes.saturating_sub(removed.len());
                }
            }
        }

        self.buffered_bytes += data_len;
        self.chunks.insert(chunk_start, data);

        Ok(())
    }
}

impl Read for SlidingStreamBuffer {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        if let Some(total) = self.total_size {
            if self.cursor >= total {
                return Ok(0); // EOF
            }
        }

        let chunk_index = self.cursor / CHUNK_SIZE as u64;
        let offset_in_chunk = (self.cursor % CHUNK_SIZE as u64) as usize;

        self.ensure_chunk(chunk_index)?;

        let chunk_start = chunk_index * CHUNK_SIZE as u64;
        let chunk_data = match self.chunks.get(&chunk_start) {
            Some(data) => data,
            None => return Ok(0),
        };

        if offset_in_chunk >= chunk_data.len() {
            return Ok(0); // End of this chunk or EOF
        }

        let available = chunk_data.len() - offset_in_chunk;
        let to_copy = buf.len().min(available);

        buf[..to_copy].copy_from_slice(&chunk_data[offset_in_chunk..offset_in_chunk + to_copy]);
        self.cursor += to_copy as u64;

        Ok(to_copy)
    }
}

impl Seek for SlidingStreamBuffer {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(offset) => offset as i64,
            SeekFrom::Current(offset) => (self.cursor as i64) + offset,
            SeekFrom::End(offset) => match self.total_size {
                Some(total) => (total as i64) + offset,
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Unsupported,
                        "Cannot seek from end without total stream length",
                    ))
                }
            },
        };

        if new_pos < 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Seek before stream start",
            ));
        }

        self.cursor = new_pos as u64;
        Ok(self.cursor)
    }
}

impl symphonia::core::io::MediaSource for SlidingStreamBuffer {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        self.total_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_seek_math() {
        let mut buffer = SlidingStreamBuffer::new("http://localhost".to_string(), Some(1000));
        assert_eq!(buffer.seek(SeekFrom::Start(100)).unwrap(), 100);
        assert_eq!(buffer.seek(SeekFrom::Current(50)).unwrap(), 150);
        assert_eq!(buffer.seek(SeekFrom::End(-100)).unwrap(), 900);
    }
}
