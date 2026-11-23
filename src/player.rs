use crate::opus::OpusStream;
use crate::ytm;
use std::collections::{HashMap, VecDeque};
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering::Relaxed};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, SyncSender, TryRecvError, TrySendError};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::errors::Error as DecodeError;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::{Time, TimeBase};

const RANGE_BYTES: u64 = 2 * 1024 * 1024;
const WINDOW_BYTES: u64 = 16 * 1024 * 1024;
const KEPT_BEHIND: u64 = 1024 * 1024;
const BUFFERED_AHEAD: u64 = 12 * 1024 * 1024;
const FULL_POLL: Duration = Duration::from_millis(10);
const PAUSED_POLL: Duration = Duration::from_millis(250);
const WINDOW_POLL: Duration = Duration::from_millis(150);
const MAX_STALLS: u32 = 16;
const QUEUED_CHUNKS: usize = 24;
const PRE_ROLL_MS: u64 = 80;
const SILENCE_SAMPLES: usize = 512;
const URL_LIFETIME: Duration = Duration::from_secs(3 * 60 * 60);
const URL_CACHE_ENTRIES: usize = 64;
const SELFTEST_FAR_SEEK_MS: u64 = 20 * 60 * 1000;
const DEVICE_POLL: Duration = Duration::from_secs(2);
const NO_OUTPUT: &str = "No audio output device was found";

type Wake = Arc<dyn Fn() + Send + Sync>;
type Chunks = Arc<Mutex<Receiver<Chunk>>>;
type Urls = Arc<Mutex<HashMap<String, (Instant, ytm::Stream)>>>;

#[derive(Default)]
struct Shared {
    generation: AtomicU64,
    epoch: AtomicU64,
    position_ms: AtomicU64,
    duration_ms: AtomicU64,
    buffered: AtomicU32,
    loading: AtomicBool,
    paused: AtomicBool,
    ended: AtomicBool,
    error: Mutex<Option<String>>,
    loudness_normalizer: AtomicBool,
    loudness_gain: AtomicU32,
    eq_enabled: AtomicBool,
    eq_gains: Mutex<[f32; 10]>,
    eq_preamp: AtomicU32,
}

struct Chunk {
    samples: Vec<f32>,
    start_ms: u64,
    epoch: u64,
    rate: u32,
    channels: u16,
    end: bool,
}

enum Cmd {
    Load { video_id: String, generation: u64, epoch: u64 },
    Opened(Track),
    Failed { generation: u64, message: String },
    Seek { ms: u64, epoch: u64 },
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

struct Fetch {
    window: Mutex<Window>,
    changed: Condvar,
    len: u64,
    failed: AtomicBool,
    cancelled: AtomicBool,
    generation: u64,
    shared: Arc<Shared>,
    video_id: String,
    target_ext: &'static str,
    yt: Arc<ytm::Client>,
}

impl Fetch {
    fn start(
        yt: Arc<ytm::Client>,
        url: String,
        len: u64,
        generation: u64,
        shared: Arc<Shared>,
        video_id: String,
        target_ext: &'static str,
    ) -> Arc<Fetch> {
        let (agent, user_agent) = (yt.agent.clone(), yt.player_agent());
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
            generation,
            shared,
            video_id,
            target_ext,
            yt,
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
        self.shared.generation.load(Relaxed) == self.generation && !self.cancelled.load(Relaxed)
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

    fn download(&self, agent: &ureq::Agent, initial_url: &str, user_agent: &str) -> bool {
        let mut stalls = 0;
        let mut block = [0u8; 32 * 1024];

        let cache_dir = crate::cache::get_cache_dir();
        let target_file = cache_dir.join(format!("{}.{}", self.video_id, self.target_ext));
        let part_file = crate::cache::temp_part_path(&self.video_id, "fetch");
        let mut cache_writer = if crate::cache::is_auto_cache() && !target_file.exists() && !self.video_id.is_empty() {
            std::fs::File::create(&part_file).ok()
        } else {
            None
        };
        let mut cache_bytes_written = 0u64;
        let mut current_url = initial_url.to_string();

        crate::log_stream!("Fetch", "Starting download loop for {}: total_len={}", self.video_id, self.len);
        while let Some(from) = self.next_range() {
            let mut at = from;
            let to = (from + RANGE_BYTES).min(self.len) - 1;
            let sep = if current_url.contains('?') { '&' } else { '?' };
            let url_with_range = format!("{current_url}{sep}range={from}-{to}");
            crate::log_stream!("Fetch", "Requesting chunk {from}-{to} for {}", self.video_id);
            let request = agent
                .get(&url_with_range)
                .set("User-Agent", user_agent);

            match request.call() {
                Ok(response) => {
                    let status = response.status();
                    crate::log_stream!("Fetch", "Chunk {from}-{to} HTTP {status} received");
                    let mut body = response.into_reader();
                    let mut chunk_bytes = Vec::with_capacity(64 * 1024);
                    while let Ok(read) = body.read(&mut block) {
                        if read == 0 || !self.current() {
                            break;
                        }
                        chunk_bytes.extend_from_slice(&block[..read]);
                        at += read as u64;
                    }
                    if !self.current() {
                        if let Some(f) = cache_writer.take() {
                            drop(f);
                            let _ = std::fs::remove_file(&part_file);
                        }
                        return false;
                    }
                    if !chunk_bytes.is_empty() {
                        crate::log_stream!("Fetch", "Chunk {from}-{to} buffered {} bytes (at={at})", chunk_bytes.len());
                        if let Some(writer) = &mut cache_writer {
                            use std::io::Write;
                            if from == cache_bytes_written && writer.write_all(&chunk_bytes).is_ok() {
                                cache_bytes_written += chunk_bytes.len() as u64;
                            } else {
                                cache_writer = None;
                                let _ = std::fs::remove_file(&part_file);
                            }
                        }
                        self.window.lock().unwrap().bytes.extend(chunk_bytes);
                        self.changed.notify_all();
                        self.shared.buffered.store((at * 1000 / self.len.max(1)) as u32, Relaxed);
                    }
                }
                Err(err) => {
                    crate::log_error!("Fetch", "Chunk {from}-{to} fetch ERROR for {}: {:?}", self.video_id, err);
                    if let Ok(stream) = self.yt.stream(&self.video_id) {
                        current_url = stream.url;
                        crate::log_warn!("Fetch", "Refreshed stream URL for {}", self.video_id);
                    }
                }
            }

            if at > from {
                stalls = 0;
                continue;
            }
            stalls += 1;
            crate::log_warn!("Fetch", "Chunk {from}-{to} stall #{stalls}/{MAX_STALLS} for {}", self.video_id);
            if stalls >= MAX_STALLS {
                crate::log_error!("Fetch", "Max stalls reached for {}, aborting fetch", self.video_id);
                if let Some(f) = cache_writer.take() {
                    drop(f);
                    let _ = std::fs::remove_file(&part_file);
                }
                return false;
            }
            let backoff_ms = (250 * stalls as u64).min(2000);
            std::thread::sleep(Duration::from_millis(backoff_ms));
        }

        if let Some(mut writer) = cache_writer.take() {
            use std::io::Write;
            let _ = writer.flush();
            drop(writer);
            if cache_bytes_written >= self.len && self.len > 0 {
                crate::cache::commit_temp_track(&part_file, &self.video_id, &self.target_ext, self.len);
            } else {
                let _ = std::fs::remove_file(&part_file);
            }
        }

        true
    }
}

struct Reader {
    fetch: Arc<Fetch>,
    at: u64,
    seekable: Arc<AtomicBool>,
}

impl Drop for Reader {
    fn drop(&mut self) {
        self.fetch.cancelled.store(true, Relaxed);
        self.fetch.changed.notify_all();
    }
}

impl Read for Reader {
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
                return Err(io::Error::other("cancelled"));
            }
            if self.at >= self.fetch.len {
                return Ok(0);
            }
            if self.fetch.failed.load(Relaxed) {
                return Err(io::Error::other("The connection was lost while streaming"));
            }
            self.fetch.changed.notify_all();
            window = self.fetch.changed.wait_timeout(window, WINDOW_POLL).unwrap().0;
        }
    }
}

impl Seek for Reader {
    fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
        let target = match to {
            SeekFrom::Start(offset) => offset as i64,
            SeekFrom::Current(delta) => self.at as i64 + delta,
            SeekFrom::End(delta) => self.fetch.len as i64 + delta,
        };
        self.at = target.clamp(0, self.fetch.len as i64) as u64;
        Ok(self.at)
    }
}

impl MediaSource for Reader {
    fn is_seekable(&self) -> bool {
        self.seekable.load(Relaxed)
    }

    fn byte_len(&self) -> Option<u64> {
        Some(self.fetch.len)
    }
}

struct Track {
    video_id: String,
    generation: u64,
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    id: u32,
    time_base: Option<TimeBase>,
    buffer: Option<SampleBuffer<f32>>,
    trim_ms: Option<u64>,
    finished: bool,
}

impl Track {
    fn open(yt: &Arc<ytm::Client>, urls: &Urls, video_id: &str, generation: u64, shared: &Arc<Shared>) -> Result<Track, String> {
        if let Some(cached_path) = crate::cache::get_cached_track_path(video_id) {
            crate::log_info!("Cache", "Found cached audio for {video_id} ({cached_path:?})");
            if let Ok(file) = std::fs::File::open(&cached_path) {
                let is_opus = cached_path.extension().map_or(true, |ext| ext == "opus");
                let source = MediaSourceStream::new(Box::new(file), Default::default());
                let mut hint = Hint::new();
                hint.with_extension(if is_opus { "webm" } else { "m4a" });
                if let Ok(probed) = symphonia::default::get_probe().format(&hint, source, &FormatOptions::default(), &MetadataOptions::default()) {
                    let format = probed.format;
                    if let Some(track) = format.default_track() {
                        let (id, time_base) = (track.id, track.codec_params.time_base);
                        let options = DecoderOptions::default();
                        let decoder: Result<Box<dyn Decoder>, _> = if is_opus {
                            OpusStream::try_new(&track.codec_params, &options).map(|s| Box::new(s) as Box<dyn Decoder>).map_err(|e| e.to_string())
                        } else {
                            symphonia::default::get_codecs().make(&track.codec_params, &options).map_err(|e| e.to_string())
                        };
                        if let Ok(decoder) = decoder {
                            if shared.generation.load(Relaxed) == generation {
                                shared.buffered.store(1000, Relaxed);
                            }
                            crate::log_audio!("Cache", "Loaded cached track decoder for {video_id}");
                            return Ok(Track {
                                video_id: video_id.to_owned(),
                                generation,
                                format,
                                decoder,
                                id,
                                time_base,
                                buffer: None,
                                trim_ms: None,
                                finished: false,
                            });
                        }
                    }
                }
            }
        }

        let mut stream = resolve(yt, urls, video_id)?;
        let fallback = stream.fallback.take();
        Self::open_stream(yt.clone(), stream, video_id, generation, shared).or_else(|message| {
            crate::log_warn!("Stream", "Primary stream open failed for {video_id}: {message}. Checking fallback...");
            match fallback {
                Some(aac) => Self::open_stream(yt.clone(), *aac, video_id, generation, shared),
                None => Err(message),
            }
        })
    }

    fn open_stream(yt: Arc<ytm::Client>, stream: ytm::Stream, video_id: &str, generation: u64, shared: &Arc<Shared>) -> Result<Track, String> {
        let ytm::Stream {
            url,
            len,
            duration_ms,
            opus,
            loudness_db,
            ..
        } = stream;
        crate::log_stream!("Stream", "Opening stream for {video_id}: len={len} bytes, duration={duration_ms}ms, opus={opus}, loudness={loudness_db:?}dB");
        if shared.generation.load(Relaxed) == generation {
            shared.duration_ms.store(duration_ms, Relaxed);
            let gain = crate::dsp::calculate_loudness_gain_db(loudness_db)
                .map(crate::dsp::db_to_linear)
                .unwrap_or(1.0);
            shared.loudness_gain.store(gain.to_bits(), Relaxed);
        }

        let target_ext = if opus { "opus" } else { "m4a" };
        let seekable = Arc::new(AtomicBool::new(false));
        let fetch = Fetch::start(
            yt,
            url,
            len,
            generation,
            shared.clone(),
            video_id.to_owned(),
            target_ext,
        );
        let reader = Reader {
            fetch,
            at: 0,
            seekable: seekable.clone(),
        };
        let source = MediaSourceStream::new(Box::new(reader), Default::default());
        let mut hint = Hint::new();
        hint.with_extension(if opus { "webm" } else { "m4a" });
        let format = match symphonia::default::get_probe()
            .format(&hint, source, &FormatOptions::default(), &MetadataOptions::default())
        {
            Ok(probed) => {
                crate::log_stream!("Decoder", "Symphonia probed format container successfully for {video_id}");
                probed.format
            }
            Err(e) => {
                crate::log_error!("Decoder", "Symphonia probe failed for {video_id}: {e}");
                return Err(e.to_string());
            }
        };
        seekable.store(true, Relaxed);
        let track = format.default_track().ok_or("This stream has no audio track")?;
        let (id, time_base) = (track.id, track.codec_params.time_base);
        let options = DecoderOptions::default();
        let decoder: Box<dyn Decoder> = if opus {
            match OpusStream::try_new(&track.codec_params, &options) {
                Ok(stream) => {
                    crate::log_audio!("Decoder", "Opus decoder stream initialized (rate={:?}, channels={:?})", track.codec_params.sample_rate, track.codec_params.channels);
                    Box::new(stream)
                }
                Err(e) => {
                    crate::log_error!("Decoder", "Opus decoder stream initialization failed: {e}");
                    return Err(e.to_string());
                }
            }
        } else {
            match symphonia::default::get_codecs().make(&track.codec_params, &options) {
                Ok(dec) => {
                    crate::log_audio!("Decoder", "AAC/Symphonia decoder initialized (rate={:?}, channels={:?})", track.codec_params.sample_rate, track.codec_params.channels);
                    dec
                }
                Err(e) => {
                    crate::log_error!("Decoder", "AAC decoder initialization failed: {e}");
                    return Err(e.to_string());
                }
            }
        };
        Ok(Track {
            video_id: video_id.to_owned(),
            generation,
            format,
            decoder,
            id,
            time_base,
            buffer: None,
            trim_ms: None,
            finished: false,
        })
    }

    fn next_chunk(&mut self, epoch: u64) -> Result<Option<Chunk>, String> {
        loop {
            let packet = match self.format.next_packet() {
                Ok(packet) => packet,
                Err(DecodeError::IoError(e)) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
                Err(e) => return Err(e.to_string()),
            };
            if packet.track_id() != self.id {
                continue;
            }
            let decoded = match self.decoder.decode(&packet) {
                Ok(decoded) => decoded,
                Err(DecodeError::DecodeError(_)) => continue,
                Err(e) => return Err(e.to_string()),
            };
            let spec = *decoded.spec();
            let channels = spec.channels.count();
            let buffer = match &mut self.buffer {
                Some(buffer) if buffer.capacity() >= decoded.frames() * channels => buffer,
                slot => slot.insert(SampleBuffer::new(decoded.frames() as u64, spec)),
            };
            buffer.copy_interleaved_ref(decoded);
            let time = self.time_base.map(|base| base.calc_time(packet.ts()));
            let mut start_ms = time.map_or(0, |t| t.seconds * 1000 + (t.frac * 1000.0) as u64);
            let samples = buffer.samples();
            let mut from = 0;
            if let Some(target) = self.trim_ms {
                let end_ms = start_ms + (samples.len() / channels) as u64 * 1000 / spec.rate as u64;
                if end_ms <= target {
                    continue;
                }
                self.trim_ms = None;
                if start_ms < target {
                    from = ((target - start_ms) * spec.rate as u64 / 1000) as usize * channels;
                    start_ms = target;
                }
            }
            return Ok(Some(Chunk {
                samples: samples[from.min(samples.len())..].to_vec(),
                start_ms,
                epoch,
                rate: spec.rate,
                channels: spec.channels.count() as u16,
                end: false,
            }));
        }
    }

    fn seek(&mut self, ms: u64) {
        let from = ms.saturating_sub(PRE_ROLL_MS);
        let time = Time::new(from / 1000, (from % 1000) as f64 / 1000.0);
        if self
            .format
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time,
                    track_id: Some(self.id),
                },
            )
            .is_ok()
        {
            self.decoder.reset();
            self.trim_ms = self.time_base.map(|_| ms);
            self.finished = false;
        }
    }
}

fn resolve(yt: &ytm::Client, urls: &Urls, video_id: &str) -> Result<ytm::Stream, String> {
    if let Some((at, stream)) = urls.lock().unwrap().get(video_id)
        && at.elapsed() < URL_LIFETIME
    {
        return Ok(stream.clone());
    }
    let stream = yt.stream(video_id)?;
    let mut urls = urls.lock().unwrap();
    if urls.len() >= URL_CACHE_ENTRIES {
        urls.clear();
    }
    urls.insert(video_id.to_owned(), (Instant::now(), stream.clone()));
    Ok(stream)
}

struct PcmSource {
    chunks: Chunks,
    samples: Vec<f32>,
    at: usize,
    rate: u32,
    channels: u16,
    shared: Arc<Shared>,
    wake: Wake,
    limiter: crate::dsp::SoftLimiter,
    eq: crate::dsp::Equalizer,
    last_l: f32,
    last_r: f32,
}

impl PcmSource {
    fn new(chunks: Chunks, shared: Arc<Shared>, wake: Wake) -> Self {
        Self {
            chunks,
            samples: vec![0.0; SILENCE_SAMPLES],
            at: 0,
            rate: 44_100,
            channels: 2,
            shared,
            wake,
            limiter: crate::dsp::SoftLimiter::default(),
            eq: crate::dsp::Equalizer::default(),
            last_l: 0.0,
            last_r: 0.0,
        }
    }

    fn refill(&mut self) {
        self.at = 0;
        while let Ok(chunk) = self.chunks.lock().unwrap().try_recv() {
            if chunk.epoch != self.shared.epoch.load(Relaxed) {
                continue;
            }
            if chunk.end {
                self.shared.ended.store(true, Relaxed);
                (self.wake)();
                break;
            }
            if chunk.samples.is_empty() {
                continue;
            }
            self.shared.position_ms.store(chunk.start_ms, Relaxed);
            self.rate = chunk.rate;

            // Handle mono by duplicating/interleaving to stereo so WASAPI output
            // (configured for 2 channels) plays at 1x speed without channel desync.
            if chunk.channels == 1 {
                let mut stereo = Vec::with_capacity(chunk.samples.len() * 2);
                for &sample in &chunk.samples {
                    stereo.push(sample);
                    stereo.push(sample);
                }
                self.samples = stereo;
                self.channels = 2;
            } else {
                self.channels = chunk.channels.max(2);
                self.samples = chunk.samples;
            }

            if self.shared.eq_enabled.load(Relaxed) {
                if let Ok(gains) = self.shared.eq_gains.try_lock() {
                    self.eq.set_gains(*gains);
                }
                let preamp = f32::from_bits(self.shared.eq_preamp.load(Relaxed));
                self.eq.set_preamp(preamp);
                self.eq.process_interleaved(&mut self.samples, self.channels as usize, self.rate);
            }
            if self.shared.loudness_normalizer.load(Relaxed) {
                let bits = self.shared.loudness_gain.load(Relaxed);
                let gain = if bits == 0 { 1.0 } else { f32::from_bits(bits) };
                self.limiter.process_buffer(&mut self.samples, gain);
            } else if self.shared.eq_enabled.load(Relaxed) {
                self.limiter.process_buffer(&mut self.samples, 1.0);
            }

            // Track final frame samples to enable gentle decay fade on buffer underrun
            if self.samples.len() >= 2 {
                self.last_l = self.samples[self.samples.len() - 2];
                self.last_r = self.samples[self.samples.len() - 1];
            } else if let Some(&s) = self.samples.first() {
                self.last_l = s;
                self.last_r = s;
            }

            return;
        }

        // Buffer underrun or silence: apply gentle cosine decay fade from last played
        // samples to avoid DC step clicks, then fill remaining with silence.
        self.samples.clear();
        if self.last_l.abs() > 1e-4 || self.last_r.abs() > 1e-4 {
            let fade_frames = 64.min(SILENCE_SAMPLES / 2);
            self.samples.reserve(SILENCE_SAMPLES);
            for i in 0..fade_frames {
                // Cosine decay factor from 1.0 down to 0.0
                let factor = 0.5 * (1.0 + (std::f32::consts::PI * (i as f32 / fade_frames as f32)).cos());
                self.samples.push(self.last_l * factor);
                self.samples.push(self.last_r * factor);
            }
            self.samples.resize(SILENCE_SAMPLES, 0.0);
            self.last_l = 0.0;
            self.last_r = 0.0;
        } else {
            self.samples.resize(SILENCE_SAMPLES, 0.0);
        }
    }
}

impl Iterator for PcmSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.at >= self.samples.len() {
            self.refill();
        }
        let sample = self.samples[self.at];
        self.at += 1;
        Some(sample)
    }
}

impl rodio::Source for PcmSource {
    fn current_span_len(&self) -> Option<usize> {
        Some(self.samples.len().saturating_sub(self.at))
    }

    fn channels(&self) -> rodio::ChannelCount {
        self.channels
    }

    fn sample_rate(&self) -> rodio::SampleRate {
        self.rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

fn decode_loop(
    commands: Receiver<Cmd>,
    replies: Sender<Cmd>,
    pcm: SyncSender<Chunk>,
    shared: Arc<Shared>,
    yt: Arc<ytm::Client>,
    urls: Urls,
    wake: Wake,
) {
    let mut track: Option<Track> = None;
    let mut pending: Option<Chunk> = None;
    let mut resume: Option<u64> = None;
    let mut epoch = 0;
    loop {
        if let Some(chunk) = pending.take() {
            match pcm.try_send(chunk) {
                Ok(()) => {}
                Err(TrySendError::Full(chunk)) => pending = Some(chunk),
                Err(TrySendError::Disconnected(_)) => return,
            }
        }
        let decoding = track.as_ref().is_some_and(|t| !t.finished);
        let command = if pending.is_some() {
            let wait = if shared.paused.load(Relaxed) { PAUSED_POLL } else { FULL_POLL };
            match commands.recv_timeout(wait) {
                Ok(command) => Some(command),
                Err(RecvTimeoutError::Timeout) => None,
                Err(RecvTimeoutError::Disconnected) => return,
            }
        } else if decoding {
            match commands.try_recv() {
                Ok(command) => Some(command),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => return,
            }
        } else {
            match commands.recv() {
                Ok(command) => Some(command),
                Err(_) => return,
            }
        };
        let current = |generation: u64| shared.generation.load(Relaxed) == generation;
        match command {
            Some(Cmd::Load {
                video_id,
                generation,
                epoch: started,
            }) => {
                crate::log_stream!("Player", "Cmd::Load: initiating playback for video_id={video_id}, gen={generation}");
                (track, pending, resume, epoch) = (None, None, None, started);
                let (replies, shared, yt, urls) = (replies.clone(), shared.clone(), yt.clone(), urls.clone());
                std::thread::spawn(move || {
                    let opened = Track::open(&yt, &urls, &video_id, generation, &shared).or_else(|message| {
                        urls.lock().unwrap().remove(&video_id);
                        if shared.generation.load(Relaxed) == generation && yt.refresh_clients() {
                            Track::open(&yt, &urls, &video_id, generation, &shared)
                        } else {
                            Err(message)
                        }
                    });
                    let reply = match opened {
                        Ok(track) => Cmd::Opened(track),
                        Err(message) => {
                            urls.lock().unwrap().remove(&video_id);
                            Cmd::Failed { generation, message }
                        }
                    };
                    let _ = replies.send(reply);
                });
            }
            Some(Cmd::Opened(mut opened)) if current(opened.generation) => {
                crate::log_audio!("Player", "Cmd::Opened: track ready video_id={}, duration_ms={}", opened.video_id, shared.duration_ms.load(Relaxed));
                if let Some(ms) = resume.take() {
                    opened.seek(ms);
                }
                track = Some(opened);
                shared.loading.store(false, Relaxed);
                wake();
            }
            Some(Cmd::Failed { generation, message }) if current(generation) => {
                crate::log_error!("Player", "Cmd::Failed: playback failed for generation {generation}: {message}");
                *shared.error.lock().unwrap() = Some(message);
                shared.loading.store(false, Relaxed);
                wake();
            }
            Some(Cmd::Seek { ms, epoch: flushed }) => {
                (pending, epoch) = (None, flushed);
                match &mut track {
                    Some(track) => track.seek(ms),
                    None => resume = Some(ms),
                }
            }
            _ => {}
        }
        if pending.is_some() {
            continue;
        }
        let Some(playing) = track.as_mut().filter(|t| !t.finished) else {
            continue;
        };
        match playing.next_chunk(epoch) {
            Ok(Some(chunk)) => pending = Some(chunk),
            Ok(None) => {
                playing.finished = true;
                pending = Some(Chunk {
                    samples: Vec::new(),
                    start_ms: 0,
                    epoch,
                    rate: 0,
                    channels: 0,
                    end: true,
                });
            }
            Err(message) => {
                if current(playing.generation) {
                    urls.lock().unwrap().remove(&playing.video_id);
                    *shared.error.lock().unwrap() = Some(message);
                    wake();
                }
                track = None;
            }
        }
    }
}

#[cfg(windows)]
pub mod wasapi_mute {
    use windows::Win32::System::Com::*;
    use windows::Win32::Media::Audio::*;
    use windows::Win32::Media::Audio::Endpoints::*;

    const PKEY_DEVICE_FRIENDLY_NAME: windows::Win32::Foundation::PROPERTYKEY = windows::Win32::Foundation::PROPERTYKEY {
        fmtid: windows::core::GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
        pid: 14,
    };

    pub fn get_endpoint_mute(device_name: &str) -> Option<bool> {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
            let collection = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE).ok()?;
            let count = collection.GetCount().ok()?;
            for i in 0..count {
                if let Ok(dev) = collection.Item(i) {
                    if let Ok(store) = dev.OpenPropertyStore(STGM(0)) {
                        if let Ok(val) = store.GetValue(&PKEY_DEVICE_FRIENDLY_NAME as *const _) {
                            let name = val.to_string();
                            if name == device_name {
                                if let Ok(ep_vol) = dev.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None) {
                                    return ep_vol.GetMute().ok().map(|m| m.as_bool());
                                }
                            }
                        }
                    }
                }
            }
            None
        }
    }

    pub fn set_endpoint_mute(device_name: &str, mute: bool) -> bool {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let enumerator: IMMDeviceEnumerator = match CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) {
                Ok(e) => e,
                Err(_) => return false,
            };
            let collection = match enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE) {
                Ok(c) => c,
                Err(_) => return false,
            };
            let count = match collection.GetCount() {
                Ok(c) => c,
                Err(_) => return false,
            };
            for i in 0..count {
                if let Ok(dev) = collection.Item(i) {
                    if let Ok(store) = dev.OpenPropertyStore(STGM(0)) {
                        if let Ok(val) = store.GetValue(&PKEY_DEVICE_FRIENDLY_NAME as *const _) {
                            let name = val.to_string();
                            if name == device_name {
                                if let Ok(ep_vol) = dev.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None) {
                                    return ep_vol.SetMute(mute, std::ptr::null()).is_ok();
                                }
                            }
                        }
                    }
                }
            }
            false
        }
    }
}

#[cfg(not(windows))]
pub mod wasapi_mute {
    pub fn get_endpoint_mute(_device_name: &str) -> Option<bool> { None }
    pub fn set_endpoint_mute(_device_name: &str, _mute: bool) -> bool { false }
}

fn default_device() -> Option<String> {
    use rodio::cpal::traits::{DeviceTrait, HostTrait};
    rodio::cpal::default_host().default_output_device()?.name().ok()
}

pub fn available_output_devices() -> Vec<String> {
    use rodio::cpal::traits::{DeviceTrait, HostTrait};
    let host = rodio::cpal::default_host();
    match host.output_devices() {
        Ok(devices) => {
            let mut list: Vec<String> = devices.filter_map(|d| d.name().ok()).collect();
            list.sort();
            list.dedup();
            list
        }
        Err(_) => Vec::new(),
    }
}

pub struct Player {
    commands: Sender<Cmd>,
    shared: Arc<Shared>,
    output: Option<(rodio::OutputStream, rodio::Sink)>,
    device: Option<String>,
    target_device: Option<String>,
    device_lost: Arc<AtomicBool>,
    device_polled: Instant,
    chunks: Chunks,
    wake: Wake,
    volume: f32,
    yt: Arc<ytm::Client>,
    urls: Urls,
    unmuted_device: Option<String>,
}

impl Player {
    pub fn new(yt: Arc<ytm::Client>, wake: impl Fn() + Send + Sync + 'static) -> Self {
        let wake: Wake = Arc::new(wake);
        let shared = Arc::new(Shared::default());
        let urls = Urls::default();
        let (commands, inbox) = mpsc::channel();
        let (pcm, chunks) = mpsc::sync_channel(QUEUED_CHUNKS);
        let worker = (commands.clone(), shared.clone(), yt.clone(), urls.clone(), wake.clone());
        std::thread::spawn(move || decode_loop(inbox, worker.0, pcm, worker.1, worker.2, worker.3, worker.4));
        let mut player = Self {
            commands,
            shared,
            output: None,
            device: None,
            target_device: None,
            device_lost: Arc::default(),
            device_polled: Instant::now(),
            chunks: Arc::new(Mutex::new(chunks)),
            wake,
            volume: 1.0,
            yt,
            urls,
            unmuted_device: None,
        };
        player.open_output();
        if player.output.is_none() {
            crate::log_error!("Audio", "Audio output initialization failed: No output audio device available or device is busy");
            *player.shared.error.lock().unwrap() = Some(NO_OUTPUT.to_owned());
        } else {
            crate::log_audio!("Audio", "Audio output initialized successfully on device: {:?}", player.device);
        }
        player
    }

    fn open_output(&mut self) {
        self.device_lost.store(false, Relaxed);
        let lost = self.device_lost.clone();
        let notice_loss = move |error| {
            lost.fetch_or(matches!(error, rodio::cpal::StreamError::DeviceNotAvailable), Relaxed);
        };

        use rodio::cpal::traits::{DeviceTrait, HostTrait};
        let host = rodio::cpal::default_host();

        let stream = if let Some(target_name) = &self.target_device {
            let target_dev = host.output_devices().ok().and_then(|mut devs| {
                devs.find(|d| d.name().map(|n| n == *target_name).unwrap_or(false))
            });
            if let Some(dev) = target_dev {
                let lost_fallback = self.device_lost.clone();
                rodio::OutputStreamBuilder::from_device(dev)
                    .and_then(|b| b.with_error_callback(notice_loss).open_stream_or_fallback())
                    .or_else(|_| {
                        rodio::OutputStreamBuilder::from_default_device()
                            .and_then(|device| device.with_error_callback(move |e| {
                                lost_fallback.fetch_or(matches!(e, rodio::cpal::StreamError::DeviceNotAvailable), Relaxed);
                            }).open_stream_or_fallback())
                    })
                    .or_else(|_| rodio::OutputStreamBuilder::open_default_stream())
            } else {
                crate::log_warn!("Audio", "Selected audio device '{target_name}' not available, falling back to default device");
                rodio::OutputStreamBuilder::from_default_device()
                    .and_then(|device| device.with_error_callback(notice_loss).open_stream_or_fallback())
                    .or_else(|_| rodio::OutputStreamBuilder::open_default_stream())
            }
        } else {
            rodio::OutputStreamBuilder::from_default_device()
                .and_then(|device| device.with_error_callback(notice_loss).open_stream_or_fallback())
                .or_else(|_| rodio::OutputStreamBuilder::open_default_stream())
        };

        self.output = stream.ok().map(|mut stream| {
            stream.log_on_drop(false);
            let sink = rodio::Sink::connect_new(stream.mixer());
            sink.pause();
            self.shared.paused.store(true, Relaxed);
            sink.set_volume(self.volume * self.volume);
            sink.append(PcmSource::new(self.chunks.clone(), self.shared.clone(), self.wake.clone()));
            (stream, sink)
        });
        self.device = if self.target_device.is_some() {
            self.target_device.clone()
        } else {
            default_device()
        }.filter(|_| self.output.is_some());

        if let Some(dev) = &self.device {
            crate::log_audio!("Audio", "Bound audio output stream to device: {dev}");
        } else {
            crate::log_warn!("Audio", "Could not bind audio output device");
        }
    }

    #[allow(dead_code)]
    pub fn target_device(&self) -> Option<String> {
        self.target_device.clone()
    }

    pub fn set_target_device(&mut self, target: Option<String>) {
        if self.target_device == target && self.output.is_some() && !self.device_lost.load(Relaxed) {
            return;
        }

        // If AURI previously auto-unmuted an endpoint (e.g. laptop speakers while headphones
        // were plugged in), restore its muted state in Windows when switching away so we do
        // not disrupt the user's natural Windows / hardware jack flow.
        if let Some(prev) = self.unmuted_device.take() {
            if Some(&prev) != target.as_ref() {
                crate::log_audio!("Audio", "Restoring Windows mute state for '{prev}'");
                wasapi_mute::set_endpoint_mute(&prev, true);
            } else {
                // Switching to the same device, keep tracking
                self.unmuted_device = Some(prev);
            }
        }

        // If switching to an explicit device that Windows currently has muted,
        // automatically unmute it for this playback session and record that we unmuted it.
        if let Some(ref target_name) = target {
            if wasapi_mute::get_endpoint_mute(target_name) == Some(true) {
                crate::log_audio!("Audio", "Target endpoint '{target_name}' is muted in Windows; auto-unmuting for playback");
                if wasapi_mute::set_endpoint_mute(target_name, false) {
                    self.unmuted_device = Some(target_name.clone());
                }
            }
        }

        crate::log_audio!("Audio", "Switching target audio device to: {:?}", target);
        self.target_device = target;
        let paused = self.paused();
        let played = self.output.take().is_some();
        self.open_output();
        self.set_paused(paused);
        if played && self.output.is_none() {
            crate::log_error!("Audio", "Failed to bind selected audio output device");
            *self.shared.error.lock().unwrap() = Some(NO_OUTPUT.to_owned());
        }
    }

    pub fn follow_default_device(&mut self) {
        if self.device_polled.elapsed() < DEVICE_POLL {
            return;
        }
        self.device_polled = Instant::now();
        if self.target_device.is_some() {
            if !self.device_lost.load(Relaxed) {
                return;
            }
            crate::log_warn!("Audio", "Target audio device disconnected, reconnecting...");
            let paused = self.paused();
            let played = self.output.take().is_some();
            self.open_output();
            self.set_paused(paused);
            if played && self.output.is_none() {
                crate::log_error!("Audio", "Lost audio output device, no replacement device available");
                *self.shared.error.lock().unwrap() = Some(NO_OUTPUT.to_owned());
            }
            return;
        }
        if default_device() == self.device && !self.device_lost.load(Relaxed) {
            return;
        }
        crate::log_audio!("Audio", "Audio output device changed. Previous: {:?}, New: {:?}", self.device, default_device());
        let paused = self.paused();
        let played = self.output.take().is_some();
        self.open_output();
        self.set_paused(paused);
        if played && self.output.is_none() {
            crate::log_error!("Audio", "Lost audio output device on switch, no replacement device available");
            *self.shared.error.lock().unwrap() = Some(NO_OUTPUT.to_owned());
        }
    }

    pub fn load(&self, video_id: &str) {
        let generation = self.shared.generation.fetch_add(1, Relaxed) + 1;
        let epoch = self.shared.epoch.fetch_add(1, Relaxed) + 1;
        self.shared.position_ms.store(0, Relaxed);
        self.shared.duration_ms.store(0, Relaxed);
        self.shared.buffered.store(0, Relaxed);
        self.shared.ended.store(false, Relaxed);
        self.shared.loading.store(true, Relaxed);
        self.shared.error.lock().unwrap().take();
        let _ = self.commands.send(Cmd::Load {
            video_id: video_id.to_owned(),
            generation,
            epoch,
        });
        self.set_paused(false);
    }

    pub fn preload(&self, video_id: &str) {
        if !crate::cache::is_auto_cache() {
            return;
        }
        let (yt, urls, video_id) = (self.yt.clone(), self.urls.clone(), video_id.to_owned());
        std::thread::spawn(move || {
            if crate::cache::get_cached_track_path(&video_id).is_some() {
                return;
            }
            if let Ok(stream) = resolve(&yt, &urls, &video_id) {
                let cache_dir = crate::cache::get_cache_dir();
                let target_ext = if stream.opus { "opus" } else { "m4a" };
                let target_file = cache_dir.join(format!("{video_id}.{target_ext}"));
                if !target_file.exists() {
                    let part = crate::cache::temp_part_path(&video_id, "preload");
                    if let Ok(resp) = yt.agent.get(&stream.url).call() {
                        if let Ok(mut out_f) = std::fs::File::create(&part) {
                            if std::io::copy(&mut resp.into_reader(), &mut out_f).is_ok() {
                                crate::cache::commit_temp_track(&part, &video_id, target_ext, stream.len);
                            } else {
                                let _ = std::fs::remove_file(&part);
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn seek(&self, ms: u64) {
        let epoch = self.shared.epoch.fetch_add(1, Relaxed) + 1;
        self.shared.position_ms.store(ms, Relaxed);
        self.shared.ended.store(false, Relaxed);
        let _ = self.commands.send(Cmd::Seek { ms, epoch });
    }

    pub fn set_paused(&self, paused: bool) {
        self.shared.paused.store(paused, Relaxed);
        if let Some((_, sink)) = &self.output {
            if paused { sink.pause() } else { sink.play() }
        }
    }

    pub fn paused(&self) -> bool {
        self.output.as_ref().is_none_or(|(_, sink)| sink.is_paused())
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        if let Some((_, sink)) = &self.output {
            sink.set_volume(volume * volume);
        }
    }

    pub fn set_loudness_normalizer(&self, enabled: bool) {
        self.shared.loudness_normalizer.store(enabled, Relaxed);
    }

    pub fn set_eq_enabled(&self, enabled: bool) {
        self.shared.eq_enabled.store(enabled, Relaxed);
    }

    pub fn set_eq_gains(&self, gains: [f32; 10]) {
        if let Ok(mut g) = self.shared.eq_gains.lock() {
            *g = gains;
        }
    }

    pub fn set_eq_preamp(&self, preamp_db: f32) {
        self.shared.eq_preamp.store(preamp_db.to_bits(), Relaxed);
    }

    #[allow(dead_code)]
    pub fn loudness_normalizer(&self) -> bool {
        self.shared.loudness_normalizer.load(Relaxed)
    }

    #[allow(dead_code)]
    pub fn set_loudness_gain(&self, gain: f32) {
        self.shared.loudness_gain.store(gain.to_bits(), Relaxed);
    }

    pub fn position_ms(&self) -> u64 {
        self.shared.position_ms.load(Relaxed)
    }

    pub fn duration_ms(&self) -> u64 {
        self.shared.duration_ms.load(Relaxed)
    }

    pub fn buffered(&self) -> f32 {
        self.shared.buffered.load(Relaxed) as f32 / 1000.0
    }

    pub fn loading(&self) -> bool {
        self.shared.loading.load(Relaxed)
    }

    pub fn take_ended(&self) -> bool {
        self.shared.ended.swap(false, Relaxed)
    }

    pub fn take_error(&self) -> Option<String> {
        self.shared.error.lock().unwrap().take()
    }

    pub fn device_name(&self) -> Option<String> {
        self.device.clone()
    }

    pub fn current_error(&self) -> Option<String> {
        self.shared.error.lock().unwrap().clone()
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        if let Some(prev) = self.unmuted_device.take() {
            wasapi_mute::set_endpoint_mute(&prev, true);
        }
    }
}

pub fn selftest(query: &str) -> Result<String, String> {
    let started = Instant::now();
    let yt = Arc::new(ytm::Client::new(String::new()));
    let song = yt.search(query)?.songs().into_iter().next().ok_or("search returned no songs")?;
    let searched = started.elapsed();
    let radio = yt.radio(&song.video_id)?;
    let shared = Arc::new(Shared::default());
    let opening = Instant::now();
    let mut track = Track::open(&yt, &Urls::default(), &song.video_id, 0, &shared)?;
    let opened = opening.elapsed();
    let mut frames = 0u64;
    let mut peak = 0f32;
    let mut rate = 0;
    while frames < 3 * 44_100 {
        let chunk = track.next_chunk(0)?.ok_or("stream ended early")?;
        frames += (chunk.samples.len() / chunk.channels.max(1) as usize) as u64;
        peak = chunk.samples.iter().fold(peak, |peak, sample| peak.max(sample.abs()));
        rate = chunk.rate;
    }
    if peak <= 0.001 {
        return Err("decoded audio is silent".to_owned());
    }
    let duration = shared.duration_ms.load(Relaxed);
    let far = if duration > SELFTEST_FAR_SEEK_MS + 60_000 {
        SELFTEST_FAR_SEEK_MS
    } else {
        60_000
    };
    let seeking = Instant::now();
    for target in [60_000, 5_000, far, 5_000] {
        track.seek(target);
        let landed = track.next_chunk(0)?.ok_or("seek ran past the end")?.start_ms;
        if landed.abs_diff(target) > 1000 {
            return Err(format!("seek to {target} ms landed at {landed} ms"));
        }
    }
    Ok(format!(
        "ok: \"{}\" by {} | search {searched:.0?} | open {opened:.0?} | {rate} Hz, peak {peak:.2} | seeks to 60s, 5s, {}s, 5s in {:.0?} | radio {} tracks | duration {duration} ms",
        song.title,
        song.subtitle,
        far / 1000,
        seeking.elapsed(),
        radio.len(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reopened_output_keeps_playing_the_queued_audio() {
        let shared = Arc::new(Shared::default());
        let (pcm, chunks) = mpsc::sync_channel(QUEUED_CHUNKS);
        let chunks: Chunks = Arc::new(Mutex::new(chunks));
        let open = || PcmSource::new(chunks.clone(), shared.clone(), Arc::new(|| {}));
        let queue = |start_ms| {
            let chunk = Chunk {
                samples: vec![0.5; 4],
                start_ms,
                epoch: 0,
                rate: 48_000,
                channels: 2,
                end: false,
            };
            pcm.try_send(chunk).is_ok()
        };
        assert!(queue(1000) && queue(2000));
        let mut first = open();
        assert_eq!(first.by_ref().take(SILENCE_SAMPLES + 4).filter(|sample| *sample == 0.5).count(), 4);
        drop(first);
        let mut second = open();
        assert_eq!(second.by_ref().take(SILENCE_SAMPLES + 4).filter(|sample| *sample == 0.5).count(), 4);
        assert_eq!(shared.position_ms.load(Relaxed), 2000);
        assert!(queue(3000));
    }

    #[test]
    fn test_mono_audio_interleaving_to_stereo() {
        use rodio::Source;
        let shared = Arc::new(Shared::default());
        let (pcm, chunks) = mpsc::sync_channel(QUEUED_CHUNKS);
        let chunks: Chunks = Arc::new(Mutex::new(chunks));
        let mut source = PcmSource::new(chunks, shared, Arc::new(|| {}));

        // Drain initial silence
        for _ in 0..SILENCE_SAMPLES {
            source.next();
        }

        // Queue a mono chunk (channels = 1) with 3 samples
        let mono_chunk = Chunk {
            samples: vec![0.25, 0.5, 0.75],
            start_ms: 500,
            epoch: 0,
            rate: 44_100,
            channels: 1,
            end: false,
        };
        pcm.send(mono_chunk).unwrap();

        // Reading samples should yield duplicated stereo pairs [0.25, 0.25, 0.5, 0.5, 0.75, 0.75]
        let mut read = Vec::new();
        for _ in 0..6 {
            read.push(source.next().unwrap());
        }
        assert_eq!(read, vec![0.25, 0.25, 0.5, 0.5, 0.75, 0.75]);
        assert_eq!(source.channels(), 2);
    }

    #[test]
    fn test_current_span_len_tracks_remaining_samples() {
        use rodio::Source;
        let shared = Arc::new(Shared::default());
        let (pcm, chunks) = mpsc::sync_channel(QUEUED_CHUNKS);
        let chunks: Chunks = Arc::new(Mutex::new(chunks));
        let mut source = PcmSource::new(chunks, shared, Arc::new(|| {}));

        assert_eq!(source.current_span_len(), Some(SILENCE_SAMPLES));
        source.next();
        assert_eq!(source.current_span_len(), Some(SILENCE_SAMPLES - 1));

        // Skip to end of silence
        for _ in 1..SILENCE_SAMPLES {
            source.next();
        }
        assert_eq!(source.current_span_len(), Some(0));

        // Queue a 4-sample stereo chunk
        pcm.send(Chunk {
            samples: vec![0.1; 4],
            start_ms: 0,
            epoch: 0,
            rate: 48_000,
            channels: 2,
            end: false,
        }).unwrap();

        // First sample triggers refill
        source.next();
        assert_eq!(source.current_span_len(), Some(3));
    }

    #[test]
    fn test_buffer_underrun_gentle_decay_fade() {
        let shared = Arc::new(Shared::default());
        let (pcm, chunks) = mpsc::sync_channel(QUEUED_CHUNKS);
        let chunks: Chunks = Arc::new(Mutex::new(chunks));
        let mut source = PcmSource::new(chunks, shared, Arc::new(|| {}));

        // Drain initial silence
        for _ in 0..SILENCE_SAMPLES {
            source.next();
        }

        // Queue a chunk that ends on high non-zero amplitude (0.8)
        pcm.send(Chunk {
            samples: vec![0.8; 4],
            start_ms: 0,
            epoch: 0,
            rate: 48_000,
            channels: 2,
            end: false,
        }).unwrap();

        for _ in 0..4 {
            source.next();
        }

        // Next read causes buffer underrun. Instead of a hard 0.0 step, it should gently fade down
        let mut underrun_samples = Vec::new();
        for _ in 0..SILENCE_SAMPLES {
            underrun_samples.push(source.next().unwrap());
        }

        // First frame should start near 0.8 (e.g. > 0.7) and smoothly decay towards 0.0
        assert!(underrun_samples[0] > 0.7, "Expected gentle decay to start near 0.8, got {}", underrun_samples[0]);
        // Last sample of fade region should be 0.0
        assert_eq!(underrun_samples[SILENCE_SAMPLES - 1], 0.0);
    }

    #[test]
    fn test_download_live_audio_chunks() {
        let yt = ytm::Client::new(String::new());
        let song = yt.search("Aeroplane Scene").unwrap().songs().into_iter().next().unwrap();
        let stream = yt.stream(&song.video_id).unwrap();
        let user_agent = yt.player_agent();
        let agent = yt.agent.clone();

        // Verify full song chunked download seamlessly retrieves all bytes across multiple chunks
        let mut downloaded_bytes = 0u64;
        let mut chunk_index = 0;
        while downloaded_bytes < stream.len {
            let from = downloaded_bytes;
            let to = (from + RANGE_BYTES).min(stream.len) - 1;
            let sep = if stream.url.contains('?') { '&' } else { '?' };
            let url = format!("{}{sep}range={from}-{to}", stream.url);
            let req = agent.get(&url).set("User-Agent", &user_agent);
            let resp = req.call();
            assert!(resp.is_ok(), "Chunk {chunk_index} ({from}-{to}) failed: {:?}", resp.err());
            let mut reader = resp.unwrap().into_reader();
            let mut chunk = Vec::new();
            reader.read_to_end(&mut chunk).unwrap();
            assert_eq!(chunk.len() as u64, to - from + 1, "Chunk {chunk_index} wrong size");
            downloaded_bytes += chunk.len() as u64;
            chunk_index += 1;
        }
        assert!(chunk_index > 1, "Expected multiple chunks for full song");
        assert_eq!(downloaded_bytes, stream.len);
    }

    #[test]
    fn test_available_output_devices_query() {
        let devs = available_output_devices();
        // Enumeration should not panic and return a deduplicated, sorted list
        for window in devs.windows(2) {
            assert!(window[0] < window[1], "devices should be sorted and unique");
        }
    }

    #[test]
    fn test_wasapi_mute_query_and_toggle() {
        assert_eq!(wasapi_mute::get_endpoint_mute("NonExistentAudioDevice123"), None);
        assert!(!wasapi_mute::set_endpoint_mute("NonExistentAudioDevice123", true));
    }
}
