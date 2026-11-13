use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SyncMode {
    #[default]
    Disconnected,
    Host,
    Guest,
}

/// One track in the shared queue. Guests rebuild their queue from these;
/// audio is fetched by each client through its own YouTube Music session.
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct SyncTrack {
    pub id: String,
    pub title: String,
    pub artist: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SyncPacket {
    /// "state" (host playback), "hello" / "bye" (guest presence).
    #[serde(default)]
    pub kind: String,
    /// Random per-instance id; never the room secret.
    #[serde(default)]
    pub sender: String,
    #[serde(default)]
    pub video_id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub position_ms: u64,
    #[serde(default)]
    pub playing: bool,
    #[serde(default)]
    pub timestamp_ms: u64,
    #[serde(default)]
    pub queue: Vec<SyncTrack>,
    #[serde(default)]
    pub index: usize,
    /// HMAC-SHA256 of the packet with this field emptied, keyed by the room
    /// code. Only people holding the code can control the room.
    #[serde(default)]
    pub signature: String,
}

pub enum SyncCmd {
    Host { room: String },
    Join { room: String, name: String },
    Publish(SyncPacket),
    Disconnect,
}

pub struct SyncSession {
    cmd_sender: Sender<SyncCmd>,
    active: Arc<AtomicBool>,
}

impl SyncSession {
    pub fn new(on_packet: impl Fn(SyncPacket) + Send + Sync + 'static) -> Self {
        let (cmd_sender, cmd_inbox) = mpsc::channel();
        let active = Arc::new(AtomicBool::new(false));
        let on_packet: Arc<dyn Fn(SyncPacket) + Send + Sync> = Arc::new(on_packet);
        let worker_active = active.clone();
        std::thread::spawn(move || run_worker(cmd_inbox, on_packet, worker_active));
        Self { cmd_sender, active }
    }

    pub fn host(&self, room: String) {
        self.active.store(true, Ordering::Relaxed);
        let _ = self.cmd_sender.send(SyncCmd::Host { room });
    }

    pub fn join(&self, room: String, name: String) {
        self.active.store(true, Ordering::Relaxed);
        let _ = self.cmd_sender.send(SyncCmd::Join { room, name });
    }

    pub fn publish(&self, packet: SyncPacket) {
        if self.active.load(Ordering::Relaxed) {
            let _ = self.cmd_sender.send(SyncCmd::Publish(packet));
        }
    }

    pub fn disconnect(&self) {
        self.active.store(false, Ordering::Relaxed);
        let _ = self.cmd_sender.send(SyncCmd::Disconnect);
    }
}

/// Room codes are the shared secret: 50 bits from the OS RNG, printed without
/// ambiguous characters. Unguessable, and enough to sign every packet.
pub fn new_room_code() -> String {
    use ring::rand::{SecureRandom, SystemRandom};
    const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut bytes = [0u8; 10];
    if SystemRandom::new().fill(&mut bytes).is_err() {
        // Practically unreachable; fall back to time-seeded bytes.
        let now = now_timestamp_ms();
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = (now.rotate_left(i as u32 * 5) & 0xFF) as u8;
        }
    }
    let chars: Vec<char> = bytes.iter().map(|b| ALPHABET[*b as usize % ALPHABET.len()] as char).collect();
    format!("{}{}{}{}-{}{}{}{}-{}{}", chars[0], chars[1], chars[2], chars[3], chars[4], chars[5], chars[6], chars[7], chars[8], chars[9])
}

/// The ntfy topic is derived from a hash of the room code, so the relay (or
/// anyone watching it) never learns the code itself.
fn topic_for(room: &str) -> String {
    let digest = ring::digest::digest(&ring::digest::SHA256, room.as_bytes());
    format!("auri-lt-{}", hex(&digest.as_ref()[..12]))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 || !text.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
        .collect()
}

fn signed_json(mut packet: SyncPacket, secret: &str) -> String {
    packet.signature = String::new();
    let json = serde_json::to_string(&packet).unwrap_or_default();
    let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, secret.as_bytes());
    let signature = ring::hmac::sign(&key, json.as_bytes());
    packet.signature = hex(signature.as_ref());
    serde_json::to_string(&packet).unwrap_or_default()
}

fn verified_packet(message: &str, secret: &str) -> Option<SyncPacket> {
    let mut packet: SyncPacket = serde_json::from_str(message).ok()?;
    let signature = std::mem::take(&mut packet.signature);
    let expected = unhex(&signature)?;
    // Re-serializing with the signature emptied reproduces the signed JSON
    // byte-for-byte (struct field order is stable in serde_json).
    let json = serde_json::to_string(&packet).unwrap_or_default();
    let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, secret.as_bytes());
    ring::hmac::verify(&key, json.as_bytes(), &expected).ok()?;
    Some(packet)
}

fn post(topic: &str, body: String) {
    let _ = ureq::post(&format!("https://ntfy.sh/{topic}"))
        .timeout(Duration::from_millis(3000))
        .send_string(&body);
}

/// Streams the room's messages over ntfy's chunked HTTP JSON endpoint: each
/// message arrives the instant it is published, no polling interval needed.
fn reader_loop(
    topic: String,
    secret: String,
    own_id: String,
    on_packet: Arc<dyn Fn(SyncPacket) + Send + Sync>,
    session_active: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
) {
    while session_active.load(Ordering::Relaxed) && !stop.load(Ordering::Relaxed) {
        let response = ureq::get(&format!("https://ntfy.sh/{topic}/json"))
            .timeout(Duration::from_secs(45))
            .call();
        match response {
            Ok(response) => {
                let mut reader = BufReader::new(response.into_reader());
                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line) {
                        Ok(0) => break, // EOF: reconnect
                        Ok(_) => {
                            if !session_active.load(Ordering::Relaxed) || stop.load(Ordering::Relaxed) {
                                return;
                            }
                            let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else { continue };
                            if value["event"] != "message" {
                                continue; // open / keepalive events
                            }
                            let Some(message) = value["message"].as_str() else { continue };
                            if let Some(packet) = verified_packet(message, &secret) {
                                if packet.sender != own_id {
                                    on_packet(packet);
                                }
                            }
                        }
                        Err(_) => break, // read timeout or socket error: reconnect
                    }
                }
            }
            Err(_) => {}
        }
        std::thread::sleep(Duration::from_millis(750));
    }
}

fn run_worker(
    inbox: Receiver<SyncCmd>,
    on_packet: Arc<dyn Fn(SyncPacket) + Send + Sync>,
    active: Arc<AtomicBool>,
) {
    let mut mode = SyncMode::Disconnected;
    let mut room = String::new();
    let mut topic = String::new();
    let own_id = hex(&now_timestamp_ms().to_le_bytes()) + &hex(&[std::process::id() as u8]);
    let mut reader_stop: Option<Arc<AtomicBool>> = None;
    let mut pending: Option<SyncPacket> = None;
    let mut guest_name = String::new();
    let mut last_hello = std::time::Instant::now() - Duration::from_secs(60);

    loop {
        while let Ok(cmd) = inbox.try_recv() {
            match cmd {
                SyncCmd::Host { room: code } => {
                    if let Some(stop) = reader_stop.take() {
                        stop.store(true, Ordering::Relaxed);
                    }
                    mode = SyncMode::Host;
                    guest_name.clear();
                    room = code;
                    topic = topic_for(&room);
                    let stop = Arc::new(AtomicBool::new(false));
                    reader_stop = Some(stop.clone());
                    let (t, s, id, cb, act) = (topic.clone(), room.clone(), own_id.clone(), on_packet.clone(), active.clone());
                    std::thread::spawn(move || reader_loop(t, s, id, cb, act, stop));
                }
                SyncCmd::Join { room: code, name } => {
                    if let Some(stop) = reader_stop.take() {
                        stop.store(true, Ordering::Relaxed);
                    }
                    mode = SyncMode::Guest;
                    guest_name = name;
                    room = code;
                    topic = topic_for(&room);
                    let stop = Arc::new(AtomicBool::new(false));
                    reader_stop = Some(stop.clone());
                    let (t, s, id, cb, act) = (topic.clone(), room.clone(), own_id.clone(), on_packet.clone(), active.clone());
                    std::thread::spawn(move || reader_loop(t, s, id, cb, act, stop));
                    last_hello = std::time::Instant::now() - Duration::from_secs(60);
                }
                SyncCmd::Publish(packet) => pending = Some(packet),
                SyncCmd::Disconnect => {
                    if mode == SyncMode::Guest && !topic.is_empty() {
                        let bye = SyncPacket {
                            kind: "bye".into(),
                            title: guest_name.clone(),
                            sender: own_id.clone(),
                            ..Default::default()
                        };
                        post(&topic, signed_json(bye, &room));
                    }
                    if let Some(stop) = reader_stop.take() {
                        stop.store(true, Ordering::Relaxed);
                    }
                    mode = SyncMode::Disconnected;
                    guest_name.clear();
                    room.clear();
                    topic.clear();
                    pending = None;
                }
            }
        }

        match mode {
            SyncMode::Disconnected => {
                let _ = inbox.recv_timeout(Duration::from_millis(500));
            }
            SyncMode::Host => {
                if let Some(packet) = pending.take() {
                    let packet = SyncPacket {
                        kind: "state".into(),
                        sender: own_id.clone(),
                        timestamp_ms: now_timestamp_ms(),
                        ..packet
                    };
                    post(&topic, signed_json(packet, &room));
                }
                let _ = inbox.recv_timeout(Duration::from_millis(250));
            }
            SyncMode::Guest => {
                if last_hello.elapsed() >= Duration::from_secs(15) {
                    last_hello = std::time::Instant::now();
                    let hello = SyncPacket {
                        kind: "hello".into(),
                        title: guest_name.clone(),
                        sender: own_id.clone(),
                        ..Default::default()
                    };
                    post(&topic, signed_json(hello, &room));
                }
                let _ = inbox.recv_timeout(Duration::from_millis(250));
            }
        }
    }
}

pub fn now_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |e| e.as_millis() as u64)
}

/// Running clock-skew estimator for guests. The minimum of (receive time -
/// host timestamp) over recent packets approximates (best latency - clock
/// skew), so subtracting it cancels both clock skew and latency jitter
/// without the two machines ever needing synchronized clocks.
pub struct SkewEstimator {
    samples: VecDeque<i64>,
}

impl Default for SkewEstimator {
    fn default() -> Self {
        Self { samples: VecDeque::with_capacity(32) }
    }
}

impl SkewEstimator {
    /// `received_at` is the guest's clock, `host_timestamp` the host's.
    /// Returns the estimated host position right now.
    pub fn target_position(&mut self, position_ms: u64, host_timestamp: u64, received_at: u64) -> u64 {
        let raw = received_at as i64 - host_timestamp as i64;
        self.samples.push_back(raw);
        if self.samples.len() > 32 {
            self.samples.pop_front();
        }
        let floor = self.samples.iter().copied().min().unwrap_or(raw);
        (position_ms as i64 + (raw - floor)).max(0) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_packet_serialization() {
        let packet = SyncPacket {
            kind: "state".to_owned(),
            sender: "HOST".to_owned(),
            video_id: "dQw4w9WgXcQ".to_owned(),
            title: "Never Gonna Give You Up".to_owned(),
            subtitle: "Rick Astley".to_owned(),
            position_ms: 42000,
            playing: true,
            timestamp_ms: 123456789,
            ..Default::default()
        };
        let json = serde_json::to_string(&packet).unwrap();
        let decoded: SyncPacket = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.sender, "HOST");
        assert_eq!(decoded.video_id, "dQw4w9WgXcQ");
        assert_eq!(decoded.position_ms, 42000);
        assert!(decoded.playing);
    }

    #[test]
    fn test_signed_packets_verify_and_tampering_is_rejected() {
        let packet = SyncPacket {
            kind: "state".to_owned(),
            sender: "HOST".to_owned(),
            video_id: "abc123".to_owned(),
            playing: true,
            ..Default::default()
        };
        let json = signed_json(packet, "SECRET-CODE");
        assert!(verified_packet(&json, "SECRET-CODE").is_some());
        assert!(verified_packet(&json, "WRONG-CODE").is_none());
        let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
        value["video_id"] = serde_json::json!("evil666");
        assert!(verified_packet(&value.to_string(), "SECRET-CODE").is_none());
    }

    #[test]
    fn test_room_codes_are_random_and_topics_hide_the_code() {
        let a = new_room_code();
        let b = new_room_code();
        assert_ne!(a, b);
        assert_eq!(a.len(), 12); // XXXX-XXXX-XX
        assert!(a.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'));
        let topic = topic_for(&a);
        assert!(topic.starts_with("auri-lt-"));
        assert!(!topic.contains(&a));
        assert_eq!(topic_for(&a), topic);
    }

    #[test]
    fn test_skew_estimator_cancels_clock_offset() {
        let mut estimator = SkewEstimator::default();
        // Host clock runs 30s ahead of the guest. The first packet sets the baseline.
        let host_ts = 1_000_000u64;
        let guest_now = host_ts - 30_000 + 150;
        assert_eq!(estimator.target_position(50_000, host_ts, guest_now), 50_000);
        // A lower-latency packet (100ms) becomes the new floor.
        let host_ts2 = host_ts + 2_000;
        let guest_now2 = host_ts2 - 30_000 + 100;
        assert_eq!(estimator.target_position(52_000, host_ts2, guest_now2), 52_000);
        // A later packet with 400ms latency: the 30s clock skew cancels, and
        // the 300ms jitter above the best observed latency is compensated.
        let host_ts3 = host_ts + 4_000;
        let guest_now3 = host_ts3 - 30_000 + 400;
        assert_eq!(estimator.target_position(54_000, host_ts3, guest_now3), 54_300);
    }
}
