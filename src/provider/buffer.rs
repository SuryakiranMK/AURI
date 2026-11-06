// AURI Sliding Stream Buffer
// Bounded 8 MB memory window buffer with chunked HTTP Range streaming

use std::collections::VecDeque;
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use symphonia::core::io::MediaSource;

const RANGE_CHUNK_BYTES: u64 = 1024 * 1024; // 1 MB per HTTP chunk
const WINDOW_MAX_BYTES: u64 = 8 * 1024 * 1024; // 8 MB max sliding window
const KEPT_BEHIND_BYTES: u64 = 1024 * 1024; // 1 MB kept behind read head
const BUFFER_AHEAD_BYTES: u64 = WINDOW_MAX_BYTES - KEPT_BEHIND_BYTES - RANGE_CHUNK_BYTES;
const POLL_INTERVAL: Duration = Duration::from_millis(150);

#[derive(Default)]
struct BufferWindow {
    start_offset: u64,
    bytes: VecDeque<u8>,
    reader_offset: u64,
}

impl BufferWindow {
    fn end_offset(&self) -> u64 {
        self.start_offset + self.bytes.len() as u64
    }
}

pub struct SlidingStreamBuffer {
    window: Arc<Mutex<BufferWindow>>,
    changed: Arc<Condvar>,
    content_len: u64,
    cancelled: Arc<AtomicBool>,
    failed: Arc<AtomicBool>,
    current_read_pos: u64,
}

impl SlidingStreamBuffer {
    pub fn new(stream_url: String, content_len: u64, user_agent: String) -> Self {
        let window = Arc::new(Mutex::new(BufferWindow {
            bytes: VecDeque::with_capacity(content_len.min(WINDOW_MAX_BYTES) as usize),
            ..Default::default()
        }));
        let changed = Arc::new(Condvar::new());
        let cancelled = Arc::new(AtomicBool::new(false));
        let failed = Arc::new(AtomicBool::new(false));

        let w_worker = Arc::clone(&window);
        let c_worker = Arc::clone(&changed);
        let cancel_worker = Arc::clone(&cancelled);
        let fail_worker = Arc::clone(&failed);

        // Background progressive chunk downloader
        std::thread::Builder::new()
            .name("auri-stream-fetcher".to_string())
            .spawn(move || {
                let agent = ureq::Agent::new_with_config(
                    ureq::config::Config::builder()
                        .timeout_global(Some(Duration::from_secs(15)))
                        .build(),
                );

                let mut block = [0u8; 16 * 1024];

                loop {
                    if cancel_worker.load(Ordering::Relaxed) {
                        break;
                    }

                    // Determine next range to fetch
                    let next_range = {
                        let mut win = w_worker.lock().unwrap();

                        if content_len > WINDOW_MAX_BYTES {
                            // Evict bytes far behind read head
                            let played = win
                                .reader_offset
                                .saturating_sub(win.start_offset + KEPT_BEHIND_BYTES)
                                .min(win.bytes.len() as u64);
                            win.bytes.drain(..played as usize);
                            win.start_offset += played;
                        }

                        let end = win.end_offset();
                        if end < content_len && end < win.reader_offset + BUFFER_AHEAD_BYTES {
                            Some(end)
                        } else if end >= content_len {
                            None
                        } else {
                            // Wait for reader to advance
                            let _ = c_worker.wait_timeout(win, POLL_INTERVAL);
                            continue;
                        }
                    };

                    match next_range {
                        Some(from) => {
                            let to = (from + RANGE_CHUNK_BYTES).min(content_len) - 1;
                            let url_with_range = format!("{}&range={}-{}", stream_url, from, to);

                            let req = agent
                                .get(&url_with_range)
                                .header("User-Agent", &user_agent);

                            match req.call() {
                                Ok(res) => {
                                    let mut reader = res.into_body().into_reader();
                                    while let Ok(n) = reader.read(&mut block) {
                                        if n == 0 || cancel_worker.load(Ordering::Relaxed) {
                                            break;
                                        }
                                        {
                                            let mut win = w_worker.lock().unwrap();
                                            win.bytes.extend(&block[..n]);
                                        }
                                        c_worker.notify_all();
                                    }
                                }
                                Err(e) => {
                                    eprintln!("[AURI Stream Fetcher Error] HTTP request failed: {}", e);
                                    fail_worker.store(true, Ordering::Relaxed);
                                    c_worker.notify_all();
                                    std::thread::sleep(Duration::from_millis(500));
                                }
                            }
                        }
                        None => {
                            // Entire file buffered or waiting
                            std::thread::sleep(Duration::from_millis(200));
                        }
                    }
                }
            })
            .expect("Failed to spawn stream fetcher thread");

        Self {
            window,
            changed,
            content_len,
            cancelled,
            failed,
            current_read_pos: 0,
        }
    }
}

impl Drop for SlidingStreamBuffer {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
        self.changed.notify_all();
    }
}

impl Read for SlidingStreamBuffer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut win = self.window.lock().unwrap();

        loop {
            win.reader_offset = self.current_read_pos;

            if self.current_read_pos >= win.start_offset && self.current_read_pos < win.end_offset() {
                let offset = (self.current_read_pos - win.start_offset) as usize;
                let mut count = 0;
                for (out, &byte) in buf.iter_mut().zip(win.bytes.range(offset..)) {
                    *out = byte;
                    count += 1;
                }
                self.current_read_pos += count as u64;
                win.reader_offset = self.current_read_pos;
                self.changed.notify_all();
                return Ok(count);
            }

            if self.current_read_pos >= self.content_len {
                return Ok(0); // EOF
            }

            if self.cancelled.load(Ordering::Relaxed) {
                return Err(io::Error::new(io::ErrorKind::Interrupted, "Stream cancelled"));
            }

            if self.failed.load(Ordering::Relaxed) {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "Stream connection failed",
                ));
            }

            self.changed.notify_all();
            win = self.changed.wait_timeout(win, POLL_INTERVAL).unwrap().0;
        }
    }
}

impl Seek for SlidingStreamBuffer {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let target = match pos {
            SeekFrom::Start(off) => off as i64,
            SeekFrom::Current(delta) => self.current_read_pos as i64 + delta,
            SeekFrom::End(delta) => self.content_len as i64 + delta,
        };

        let clamped = target.clamp(0, self.content_len as i64) as u64;
        self.current_read_pos = clamped;

        let mut win = self.window.lock().unwrap();
        win.reader_offset = self.current_read_pos;
        self.changed.notify_all();

        Ok(self.current_read_pos)
    }
}

impl MediaSource for SlidingStreamBuffer {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        Some(self.content_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_seek_math() {
        let mut buffer = SlidingStreamBuffer {
            window: Arc::new(Mutex::new(BufferWindow::default())),
            changed: Arc::new(Condvar::new()),
            content_len: 1000,
            cancelled: Arc::new(AtomicBool::new(false)),
            failed: Arc::new(AtomicBool::new(false)),
            current_read_pos: 100,
        };

        assert_eq!(buffer.seek(SeekFrom::Start(500)).unwrap(), 500);
        assert_eq!(buffer.seek(SeekFrom::Current(50)).unwrap(), 550);
        assert_eq!(buffer.seek(SeekFrom::End(-100)).unwrap(), 900);
        assert_eq!(buffer.seek(SeekFrom::End(200)).unwrap(), 1000); // Clamped at EOF
    }
}

