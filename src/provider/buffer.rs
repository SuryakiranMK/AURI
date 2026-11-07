// AURI 8 MB Sliding-Window Progressive Stream Buffer
// Derived from Tubefast's zero-allocation sliding HTTP range fetcher

use std::collections::VecDeque;
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use symphonia::core::io::MediaSource;

const RANGE_BYTES: u64 = 1024 * 1024; // 1 MB per chunk
const WINDOW_BYTES: u64 = 8 * 1024 * 1024; // 8 MB max window in RAM
const KEPT_BEHIND: u64 = 1024 * 1024; // 1 MB preserved behind playhead
const BUFFERED_AHEAD: u64 = WINDOW_BYTES - KEPT_BEHIND - RANGE_BYTES;
const WINDOW_POLL: Duration = Duration::from_millis(150);
const MAX_STALLS: u32 = 5;

#[derive(Default)]
pub struct StreamStats {
    pub buffered_pct: AtomicU32,
    pub failed: AtomicBool,
}

#[derive(Default)]
struct Window {
    start: u64,
    bytes: VecDeque<u8>,
    reader: u64,
}

impl Window {
    fn end(&self) -> u64 {
        self.start + self.bytes.len() as u64
    }
}

pub struct Fetch {
    window: Mutex<Window>,
    changed: Condvar,
    len: u64,
    failed: AtomicBool,
    cancelled: AtomicBool,
    stats: Arc<StreamStats>,
}

impl Fetch {
    pub fn start(
        agent: ureq::Agent,
        url: String,
        len: u64,
        user_agent: String,
        stats: Arc<StreamStats>,
    ) -> Arc<Fetch> {
        let window = Window {
            bytes: VecDeque::with_capacity(len.min(WINDOW_BYTES) as usize),
            ..Window::default()
        };
        let fetch = Arc::new(Fetch {
            window: Mutex::new(window),
            changed: Condvar::new(),
            len,
            failed: AtomicBool::new(false),
            cancelled: AtomicBool::new(false),
            stats,
        });

        let worker = fetch.clone();
        std::thread::spawn(move || {
            let complete = worker.download(&agent, &url, &user_agent);
            worker.failed.store(!complete, Relaxed);
            worker.changed.notify_all();
        });
        fetch
    }

    fn current(&self) -> bool {
        !self.cancelled.load(Relaxed)
    }

    fn next_range(&self) -> Option<u64> {
        let windowed = self.len > WINDOW_BYTES;
        let mut window = self.window.lock().unwrap();
        while self.current() {
            if windowed {
                if window.reader < window.start || window.reader > window.end() + RANGE_BYTES {
                    window.bytes.clear();
                    window.start = window.reader;
                }
                let played = window
                    .reader
                    .saturating_sub(window.start + KEPT_BEHIND)
                    .min(window.bytes.len() as u64);
                window.bytes.drain(..played as usize);
                window.start += played;
            }
            let end = window.end();
            if end < self.len && (!windowed || end < window.reader + BUFFERED_AHEAD) {
                return Some(end);
            }
            if !windowed {
                return None;
            }
            window = self.changed.wait_timeout(window, WINDOW_POLL).unwrap().0;
        }
        None
    }

    fn download(&self, agent: &ureq::Agent, url: &str, user_agent: &str) -> bool {
        let mut stalls = 0;
        let mut block = [0u8; 16 * 1024];
        while let Some(from) = self.next_range() {
            let mut at = from;
            let to = (from + RANGE_BYTES).min(self.len) - 1;
            let request = agent
                .get(&format!("{url}&range={from}-{to}"))
                .set("User-Agent", user_agent);

            if let Ok(response) = request.call() {
                let mut body = response.into_reader();
                while let Ok(read) = body.read(&mut block) {
                    if read == 0 || !self.current() {
                        break;
                    }
                    self.window.lock().unwrap().bytes.extend(&block[..read]);
                    at += read as u64;
                    self.changed.notify_all();
                    if self.len > 0 {
                        self.stats
                            .buffered_pct
                            .store(((at * 100) / self.len) as u32, Relaxed);
                    }
                }
            }
            if at > from {
                stalls = 0;
                continue;
            }
            stalls += 1;
            if stalls >= MAX_STALLS {
                self.stats.failed.store(true, Relaxed);
                return false;
            }
            std::thread::sleep(Duration::from_millis(300 * stalls as u64));
        }
        true
    }
}

pub struct StreamReader {
    fetch: Arc<Fetch>,
    at: u64,
    seekable: Arc<AtomicBool>,
}

impl StreamReader {
    pub fn new(fetch: Arc<Fetch>, seekable: Arc<AtomicBool>) -> Self {
        Self {
            fetch,
            at: 0,
            seekable,
        }
    }
}

impl Drop for StreamReader {
    fn drop(&mut self) {
        self.fetch.cancelled.store(true, Relaxed);
        self.fetch.changed.notify_all();
    }
}

impl Read for StreamReader {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let mut window = self.fetch.window.lock().unwrap();
        loop {
            window.reader = self.at;
            if self.at >= window.start && self.at < window.end() {
                let offset = (self.at - window.start) as usize;
                let mut count = 0;
                for (slot, byte) in out.iter_mut().zip(window.bytes.range(offset..)) {
                    *slot = *byte;
                    count += 1;
                }
                self.at += count as u64;
                window.reader = self.at;
                drop(window);
                self.fetch.changed.notify_all();
                return Ok(count);
            }
            if !self.fetch.current() {
                return Err(io::Error::other("Stream cancelled"));
            }
            if self.at >= self.fetch.len {
                return Ok(0);
            }
            if self.fetch.failed.load(Relaxed) {
                return Err(io::Error::other("Network connection lost while streaming"));
            }
            self.fetch.changed.notify_all();
            window = self.fetch.changed.wait_timeout(window, WINDOW_POLL).unwrap().0;
        }
    }
}

impl Seek for StreamReader {
    fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
        let target = match to {
            SeekFrom::Start(offset) => offset as i64,
            SeekFrom::Current(delta) => self.at as i64 + delta,
            SeekFrom::End(delta) => self.fetch.len as i64 + delta,
        };
        self.at = target.clamp(0, self.fetch.len as i64) as u64;
        self.fetch.window.lock().unwrap().reader = self.at;
        self.fetch.changed.notify_all();
        Ok(self.at)
    }
}

impl MediaSource for StreamReader {
    fn is_seekable(&self) -> bool {
        self.seekable.load(Relaxed)
    }

    fn byte_len(&self) -> Option<u64> {
        Some(self.fetch.len)
    }
}
