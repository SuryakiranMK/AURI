use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader};
use std::net::{SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
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
    /// "state" (host playback), "hello" / "bye" (guest presence), "candidate" / "ping" (P2P hole punch).
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
    /// P2P candidate endpoints ("public_ip:port;local_ip:port")
    #[serde(default)]
    pub endpoint: String,
    /// True if delivered directly peer-to-peer via UDP (<15ms)
    #[serde(default)]
    pub p2p: bool,
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

/// Strictly validates the format of Listen Together room codes.
/// Supports standard Crockford format (XXXX-XXXX-XX) as well as
/// custom room codes (between 3 and 32 characters consisting of ASCII alphanumeric characters, hyphens, and underscores).
/// Prevents path traversal, shell injection, or invalid inputs.
pub fn is_valid_room_code(code: &str) -> bool {
    let clean = code.trim();
    if clean.len() < 3 || clean.len() > 32 {
        return false;
    }
    clean.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
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

/// RFC 5389 compliant STUN client to discover NAT-mapped public IP & port.
pub fn discover_public_endpoint(socket: &UdpSocket) -> Option<SocketAddr> {
    let stun_servers = [
        "stun.l.google.com:19302",
        "stun1.l.google.com:19302",
        "stun.cloudflare.com:3478",
    ];
    let mut request = [0u8; 20];
    request[0] = 0x00;
    request[1] = 0x01; // Binding Request
    request[4..8].copy_from_slice(&0x2112_A442u32.to_be_bytes()); // Magic Cookie
    let now = now_timestamp_ms();
    for i in 8..20 {
        request[i] = (now.wrapping_add(i as u64 * 37) & 0xFF) as u8;
    }

    for server in stun_servers {
        let Ok(mut addrs) = server.to_socket_addrs() else { continue };
        let Some(addr) = addrs.next() else { continue };
        if socket.send_to(&request, addr).is_err() {
            continue;
        }
        let _ = socket.set_read_timeout(Some(Duration::from_millis(1500)));
        let mut buf = [0u8; 512];
        let Ok((len, _)) = socket.recv_from(&mut buf) else { continue };
        if len >= 20 && buf[0] == 0x01 && buf[1] == 0x01 && &buf[4..8] == &0x2112_A442u32.to_be_bytes() {
            let mut offset = 20;
            while offset + 4 <= len {
                let attr_type = u16::from_be_bytes([buf[offset], buf[offset + 1]]);
                let attr_len = u16::from_be_bytes([buf[offset + 2], buf[offset + 3]]) as usize;
                offset += 4;
                if attr_type == 0x0020 && offset + attr_len <= len && attr_len >= 8 {
                    // XOR-MAPPED-ADDRESS
                    let family = buf[offset + 1];
                    let xport = u16::from_be_bytes([buf[offset + 2], buf[offset + 3]]);
                    let port = xport ^ 0x2112;
                    if family == 0x01 {
                        let xip = u32::from_be_bytes([buf[offset + 4], buf[offset + 5], buf[offset + 6], buf[offset + 7]]);
                        let ip = std::net::Ipv4Addr::from(xip ^ 0x2112_A442);
                        return Some(SocketAddr::V4(SocketAddrV4::new(ip, port)));
                    }
                }
                offset += attr_len;
            }
        }
    }
    None
}

/// Discover local LAN IP address by querying local routing table via dummy UDP connect
pub fn discover_local_endpoint(port: u16) -> Option<SocketAddr> {
    let dummy = UdpSocket::bind("0.0.0.0:0").ok()?;
    dummy.connect("8.8.8.8:80").ok()?;
    let local_ip = dummy.local_addr().ok()?.ip();
    Some(SocketAddr::new(local_ip, port))
}

fn parse_endpoints(raw: &str) -> Vec<SocketAddr> {
    raw.split(';')
        .filter_map(|s| s.trim().parse::<SocketAddr>().ok())
        .collect()
}

fn format_endpoints(public: Option<SocketAddr>, local: Option<SocketAddr>) -> String {
    let mut parts = Vec::new();
    if let Some(p) = public {
        parts.push(p.to_string());
    }
    if let Some(l) = local {
        parts.push(l.to_string());
    }
    parts.join(";")
}

fn p2p_receiver_loop(
    socket: UdpSocket,
    secret: Arc<RwLock<String>>,
    own_id: String,
    on_packet: Arc<dyn Fn(SyncPacket) + Send + Sync>,
    active_peers: Arc<RwLock<HashMap<String, (SocketAddr, Instant)>>>,
    active: Arc<AtomicBool>,
) {
    let _ = socket.set_read_timeout(Some(Duration::from_millis(500)));
    let mut buf = [0u8; 4096];
    while active.load(Ordering::Relaxed) {
        match socket.recv_from(&mut buf) {
            Ok((len, src)) => {
                if len == 0 {
                    continue;
                }
                let Ok(text) = std::str::from_utf8(&buf[..len]) else { continue };
                let current_secret = secret.read().unwrap().clone();
                if current_secret.is_empty() {
                    continue;
                }
                if let Some(mut packet) = verified_packet(text, &current_secret) {
                    if packet.sender != own_id {
                        packet.p2p = true;
                        if let Ok(mut peers) = active_peers.write() {
                            peers.insert(packet.sender.clone(), (src, Instant::now()));
                            peers.retain(|_, (_, seen)| seen.elapsed() < Duration::from_secs(30));
                        }
                        if packet.kind == "ping" {
                            let pong = SyncPacket {
                                kind: "pong".into(),
                                sender: own_id.clone(),
                                timestamp_ms: now_timestamp_ms(),
                                p2p: true,
                                ..Default::default()
                            };
                            let signed = signed_json(pong, &current_secret);
                            let _ = socket.send_to(signed.as_bytes(), src);
                        }
                        on_packet(packet);
                    }
                }
            }
            Err(_) => {}
        }
    }
}

/// Streams the room's messages over ntfy's chunked HTTP JSON endpoint: each
/// message arrives the instant it is published, no polling interval needed.
fn reader_loop(
    topic: String,
    secret: String,
    own_id: String,
    on_packet: Arc<dyn Fn(SyncPacket) + Send + Sync>,
    peer_candidates: Arc<RwLock<HashMap<String, Vec<SocketAddr>>>>,
    p2p_socket: Option<UdpSocket>,
    session_active: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
) {
    while session_active.load(Ordering::Relaxed) && !stop.load(Ordering::Relaxed) {
        let response = ureq::get(&format!("https://ntfy.sh/{topic}/json?since=latest"))
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
                                    // Punch NAT hole towards candidate endpoints
                                    if !packet.endpoint.is_empty() {
                                        let endpoints = parse_endpoints(&packet.endpoint);
                                        if !endpoints.is_empty() {
                                            if let Ok(mut cands) = peer_candidates.write() {
                                                cands.insert(packet.sender.clone(), endpoints.clone());
                                            }
                                            if let Some(sock) = &p2p_socket {
                                                let ping = SyncPacket {
                                                    kind: "ping".into(),
                                                    sender: own_id.clone(),
                                                    timestamp_ms: now_timestamp_ms(),
                                                    p2p: true,
                                                    ..Default::default()
                                                };
                                                let signed = signed_json(ping, &secret);
                                                for addr in endpoints {
                                                    let _ = sock.send_to(signed.as_bytes(), addr);
                                                }
                                            }
                                        }
                                    }
                                    if packet.kind != "ping" {
                                        on_packet(packet);
                                    }
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
    let mut last_hello = Instant::now() - Duration::from_secs(60);
    let mut last_relay_broadcast = Instant::now() - Duration::from_secs(60);

    // 1. Initialize P2P UDP Socket on ephemeral port
    let p2p_socket = UdpSocket::bind("0.0.0.0:0").ok();
    let p2p_port = p2p_socket.as_ref().and_then(|s| s.local_addr().ok()).map(|a| a.port()).unwrap_or(0);
    let current_secret = Arc::new(RwLock::new(String::new()));
    let active_p2p_peers = Arc::new(RwLock::new(HashMap::<String, (SocketAddr, Instant)>::new()));
    let peer_candidates = Arc::new(RwLock::new(HashMap::<String, Vec<SocketAddr>>::new()));

    // Discover my endpoints via STUN (public NAT mapped) and local interface
    let my_public_endpoint = Arc::new(RwLock::new(None));
    let my_local_endpoint = discover_local_endpoint(p2p_port);

    if let Some(sock) = &p2p_socket {
        if let Ok(cloned) = sock.try_clone() {
            let my_pub = my_public_endpoint.clone();
            std::thread::spawn(move || {
                if let Some(ep) = discover_public_endpoint(&cloned) {
                    if let Ok(mut lock) = my_pub.write() {
                        *lock = Some(ep);
                    }
                }
            });
        }
    }

    // Start background UDP receiver loop
    if let Some(sock) = &p2p_socket {
        if let Ok(recv_sock) = sock.try_clone() {
            let sec = current_secret.clone();
            let id = own_id.clone();
            let cb = on_packet.clone();
            let peers = active_p2p_peers.clone();
            let act = active.clone();
            std::thread::spawn(move || {
                p2p_receiver_loop(recv_sock, sec, id, cb, peers, act);
            });
        }
    }

    loop {
        let timeout = match mode {
            SyncMode::Disconnected => Duration::from_millis(500),
            SyncMode::Host => Duration::from_millis(150),
            SyncMode::Guest => Duration::from_millis(500),
        };

        let cmd = match inbox.recv_timeout(timeout) {
            Ok(cmd) => Some(cmd),
            Err(mpsc::RecvTimeoutError::Timeout) => None,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };

        let mut next_cmd = cmd;
        while let Some(c) = next_cmd.take().or_else(|| inbox.try_recv().ok()) {
            match c {
                SyncCmd::Host { room: code } => {
                    if let Some(stop) = reader_stop.take() {
                        stop.store(true, Ordering::Relaxed);
                    }
                    mode = SyncMode::Host;
                    guest_name.clear();
                    room = code;
                    if let Ok(mut lock) = current_secret.write() {
                        *lock = room.clone();
                    }
                    topic = topic_for(&room);
                    let stop = Arc::new(AtomicBool::new(false));
                    reader_stop = Some(stop.clone());
                    let (t, s, id, cb, cands, sock, act) = (
                        topic.clone(),
                        room.clone(),
                        own_id.clone(),
                        on_packet.clone(),
                        peer_candidates.clone(),
                        p2p_socket.as_ref().and_then(|s| s.try_clone().ok()),
                        active.clone(),
                    );
                    std::thread::spawn(move || reader_loop(t, s, id, cb, cands, sock, act, stop));
                }
                SyncCmd::Join { room: code, name } => {
                    if let Some(stop) = reader_stop.take() {
                        stop.store(true, Ordering::Relaxed);
                    }
                    mode = SyncMode::Guest;
                    guest_name = name;
                    room = code;
                    if let Ok(mut lock) = current_secret.write() {
                        *lock = room.clone();
                    }
                    topic = topic_for(&room);
                    let stop = Arc::new(AtomicBool::new(false));
                    reader_stop = Some(stop.clone());
                    let (t, s, id, cb, cands, sock, act) = (
                        topic.clone(),
                        room.clone(),
                        own_id.clone(),
                        on_packet.clone(),
                        peer_candidates.clone(),
                        p2p_socket.as_ref().and_then(|s| s.try_clone().ok()),
                        active.clone(),
                    );
                    std::thread::spawn(move || reader_loop(t, s, id, cb, cands, sock, act, stop));
                    // Send hello immediately upon joining so host recognizes guest instantly
                    let my_pub = *my_public_endpoint.read().unwrap();
                    let hello = SyncPacket {
                        kind: "hello".into(),
                        title: guest_name.clone(),
                        sender: own_id.clone(),
                        endpoint: format_endpoints(my_pub, my_local_endpoint),
                        timestamp_ms: now_timestamp_ms(),
                        ..Default::default()
                    };
                    post(&topic, signed_json(hello, &room));
                    last_hello = Instant::now();
                }
                SyncCmd::Publish(packet) => pending = Some(packet),
                SyncCmd::Disconnect => {
                    if mode == SyncMode::Guest && !topic.is_empty() {
                        let bye = SyncPacket {
                            kind: "bye".into(),
                            title: guest_name.clone(),
                            sender: own_id.clone(),
                            p2p: true,
                            ..Default::default()
                        };
                        let signed = signed_json(bye, &room);
                        if let Some(sock) = &p2p_socket {
                            if let Ok(peers) = active_p2p_peers.read() {
                                for (_, (addr, _)) in peers.iter() {
                                    let _ = sock.send_to(signed.as_bytes(), addr);
                                }
                            }
                        }
                        post(&topic, signed);
                    }
                    if let Some(stop) = reader_stop.take() {
                        stop.store(true, Ordering::Relaxed);
                    }
                    mode = SyncMode::Disconnected;
                    guest_name.clear();
                    room.clear();
                    if let Ok(mut lock) = current_secret.write() {
                        lock.clear();
                    }
                    if let Ok(mut peers) = active_p2p_peers.write() {
                        peers.clear();
                    }
                    if let Ok(mut cands) = peer_candidates.write() {
                        cands.clear();
                    }
                    topic.clear();
                    pending = None;
                }
            }
        }

        match mode {
            SyncMode::Disconnected => {}
            SyncMode::Host => {
                if let Some(packet) = pending.take() {
                    let my_pub = *my_public_endpoint.read().unwrap();
                    let packet = SyncPacket {
                        kind: "state".into(),
                        sender: own_id.clone(),
                        timestamp_ms: now_timestamp_ms(),
                        endpoint: format_endpoints(my_pub, my_local_endpoint),
                        ..packet
                    };
                    let signed = signed_json(packet, &room);

                    // 1. Direct P2P transmission over UDP to all active peers:
                    let mut p2p_sent = false;
                    if let Some(sock) = &p2p_socket {
                        if let Ok(mut peers) = active_p2p_peers.write() {
                            peers.retain(|_, (_, seen)| seen.elapsed() < Duration::from_secs(30));
                            for (_, (addr, _)) in peers.iter() {
                                let _ = sock.send_to(signed.as_bytes(), addr);
                                p2p_sent = true;
                            }
                        }
                        // Also punch any candidates that haven't confirmed yet
                        if let Ok(cands) = peer_candidates.read() {
                            for (_, addrs) in cands.iter() {
                                for addr in addrs {
                                    let _ = sock.send_to(signed.as_bytes(), addr);
                                }
                            }
                        }
                    }

                    // 2. Relay backup: send periodically or when direct P2P is not yet established
                    if !p2p_sent || last_relay_broadcast.elapsed() > Duration::from_secs(60) {
                        last_relay_broadcast = Instant::now();
                        post(&topic, signed);
                    }
                }
            }
            SyncMode::Guest => {
                if last_hello.elapsed() >= Duration::from_secs(10) {
                    last_hello = Instant::now();
                    let my_pub = *my_public_endpoint.read().unwrap();
                    let hello = SyncPacket {
                        kind: "hello".into(),
                        title: guest_name.clone(),
                        sender: own_id.clone(),
                        endpoint: format_endpoints(my_pub, my_local_endpoint),
                        timestamp_ms: now_timestamp_ms(),
                        ..Default::default()
                    };
                    let signed = signed_json(hello, &room);

                    let mut p2p_sent = false;
                    if let Some(sock) = &p2p_socket {
                        if let Ok(mut peers) = active_p2p_peers.write() {
                            peers.retain(|_, (_, seen)| seen.elapsed() < Duration::from_secs(30));
                            for (_, (addr, _)) in peers.iter() {
                                let _ = sock.send_to(signed.as_bytes(), addr);
                                p2p_sent = true;
                            }
                        }
                    }
                    if !p2p_sent {
                        post(&topic, signed);
                    }
                }
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
    fn test_sync_session_lifecycle() {
        let session = SyncSession::new(|_pkt| {});
        assert!(!session.active.load(Ordering::Relaxed));
        let room = new_room_code();
        session.host(room.clone());
        assert!(session.active.load(Ordering::Relaxed));
        session.publish(SyncPacket {
            kind: "state".to_owned(),
            video_id: "test123".to_owned(),
            ..Default::default()
        });
        session.disconnect();
        assert!(!session.active.load(Ordering::Relaxed));
    }

    #[test]
    fn test_parse_and_format_endpoints() {
        let ep1: SocketAddr = "1.2.3.4:5000".parse().unwrap();
        let ep2: SocketAddr = "127.0.0.1:6000".parse().unwrap();
        let formatted = format_endpoints(Some(ep1), Some(ep2));
        assert!(formatted.contains("1.2.3.4:5000"));
        assert!(formatted.contains("127.0.0.1:6000"));
        let parsed = parse_endpoints(&formatted);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], ep1);
        assert_eq!(parsed[1], ep2);
    }

    #[test]
    fn test_direct_p2p_udp_transmission_and_verification() {
        let server = UdpSocket::bind("127.0.0.1:0").unwrap();
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        let server_addr = server.local_addr().unwrap();

        let packet = SyncPacket {
            kind: "state".to_owned(),
            sender: "GUEST".to_owned(),
            video_id: "p2p_song_123".to_owned(),
            position_ms: 15000,
            playing: true,
            p2p: true,
            ..Default::default()
        };

        let signed = signed_json(packet, "P2P-SECRET");
        client.send_to(signed.as_bytes(), server_addr).unwrap();

        let mut buf = [0u8; 1024];
        let (len, _) = server.recv_from(&mut buf).unwrap();
        let received_text = std::str::from_utf8(&buf[..len]).unwrap();
        let verified = verified_packet(received_text, "P2P-SECRET").unwrap();

        assert_eq!(verified.sender, "GUEST");
        assert_eq!(verified.video_id, "p2p_song_123");
        assert!(verified.p2p);
    }

    #[test]
    fn test_p2p_ping_pong_packet_structure() {
        let ping = SyncPacket {
            kind: "ping".to_owned(),
            sender: "PEER_A".to_owned(),
            timestamp_ms: 123456,
            p2p: true,
            ..Default::default()
        };
        let signed_ping = signed_json(ping, "SHARED-ROOM-CODE");
        let verified_ping = verified_packet(&signed_ping, "SHARED-ROOM-CODE").unwrap();
        assert_eq!(verified_ping.kind, "ping");
        assert_eq!(verified_ping.sender, "PEER_A");
        assert!(verified_ping.p2p);

        let pong = SyncPacket {
            kind: "pong".to_owned(),
            sender: "PEER_B".to_owned(),
            timestamp_ms: 123470,
            p2p: true,
            ..Default::default()
        };
        let signed_pong = signed_json(pong, "SHARED-ROOM-CODE");
        let verified_pong = verified_packet(&signed_pong, "SHARED-ROOM-CODE").unwrap();
        assert_eq!(verified_pong.kind, "pong");
        assert_eq!(verified_pong.sender, "PEER_B");
        assert!(verified_pong.p2p);
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

    #[test]
    fn test_is_valid_room_code() {
        // Standard Crockford codes (XXXX-XXXX-XX)
        assert!(is_valid_room_code("ABCD-EFGH-12"));
        assert!(is_valid_room_code("1234-5678-90"));
        assert!(is_valid_room_code("K7QX-9M2T-B4"));

        // Custom room codes (3-32 chars: ASCII alphanumeric, hyphens, underscores)
        assert!(is_valid_room_code("abc"));
        assert!(is_valid_room_code("custom-room"));
        assert!(is_valid_room_code("party_room_123"));
        assert!(is_valid_room_code("a-b_c-12345678901234567890123456")); // exactly 32 chars
        assert!(is_valid_room_code("ABCD-EFGH"));
        assert!(is_valid_room_code("ABCD-EFGH-123"));

        // Invalid codes
        assert!(!is_valid_room_code(""));
        assert!(!is_valid_room_code("   "));
        assert!(!is_valid_room_code("ab")); // too short (< 3)
        assert!(!is_valid_room_code("a".repeat(33).as_str())); // too long (> 32)
        assert!(!is_valid_room_code("ABCD-EFGH-1;DROP")); // injection / illegal chars
        assert!(!is_valid_room_code("ABCD/EFGH/12")); // path traversal / slashes
        assert!(!is_valid_room_code("room code")); // spaces
        assert!(!is_valid_room_code("room@auri")); // illegal symbol
        assert!(!is_valid_room_code("room#1")); // illegal symbol
    }

    #[test]
    fn test_topic_for_hashes_safely_for_standard_and_custom_codes() {
        let std_code = "K7QX-9M2T-B4";
        let custom_code = "my-custom_room-99";
        let topic_std = topic_for(std_code);
        let topic_cust = topic_for(custom_code);

        assert!(topic_std.starts_with("auri-lt-"));
        assert!(topic_cust.starts_with("auri-lt-"));
        assert_eq!(topic_std.len(), 8 + 24); // "auri-lt-" (8) + 12 bytes hex (24)
        assert_eq!(topic_cust.len(), 8 + 24);
        assert!(!topic_std.contains(std_code));
        assert!(!topic_cust.contains(custom_code));
        assert_ne!(topic_std, topic_cust);
        // Topic remains deterministic
        assert_eq!(topic_for(custom_code), topic_cust);
    }
}
