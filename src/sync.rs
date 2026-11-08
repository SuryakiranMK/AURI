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

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SyncPacket {
    pub room: String,
    pub video_id: String,
    pub title: String,
    pub subtitle: String,
    pub position_ms: u64,
    pub playing: bool,
    pub timestamp_ms: u64,
}

pub enum SyncCmd {
    Host { room: String, github_token: Option<String>, gist_id: Option<String> },
    Join { room: String, gist_id: Option<String> },
    Publish(SyncPacket),
    Disconnect,
}

pub struct SyncSession {
    cmd_sender: Sender<SyncCmd>,
    active: Arc<AtomicBool>,
}

impl SyncSession {
    pub fn new(on_packet: impl Fn(SyncPacket) + Send + 'static) -> Self {
        let (cmd_sender, cmd_inbox) = mpsc::channel();
        let active = Arc::new(AtomicBool::new(false));
        let active_clone = active.clone();

        std::thread::spawn(move || {
            run_sync_worker(cmd_inbox, on_packet, active_clone);
        });

        Self { cmd_sender, active }
    }

    pub fn host(&self, room: String, github_token: Option<String>, gist_id: Option<String>) {
        self.active.store(true, Ordering::Relaxed);
        let _ = self.cmd_sender.send(SyncCmd::Host { room, github_token, gist_id });
    }

    pub fn join(&self, room: String, gist_id: Option<String>) {
        self.active.store(true, Ordering::Relaxed);
        let _ = self.cmd_sender.send(SyncCmd::Join { room, gist_id });
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

fn run_sync_worker(
    inbox: Receiver<SyncCmd>,
    on_packet: impl Fn(SyncPacket) + Send + 'static,
    active: Arc<AtomicBool>,
) {
    let mut current_mode = SyncMode::Disconnected;
    let mut current_room = String::new();
    let mut current_token: Option<String> = None;
    let mut current_gist: Option<String> = None;
    let mut pending_publish: Option<SyncPacket> = None;

    loop {
        // Handle incoming commands
        while let Ok(cmd) = inbox.try_recv() {
            match cmd {
                SyncCmd::Host { room, github_token, gist_id } => {
                    current_mode = SyncMode::Host;
                    current_room = room;
                    current_token = github_token;
                    current_gist = gist_id;
                }
                SyncCmd::Join { room, gist_id } => {
                    current_mode = SyncMode::Guest;
                    current_room = room;
                    current_gist = gist_id;
                }
                SyncCmd::Publish(packet) => {
                    pending_publish = Some(packet);
                }
                SyncCmd::Disconnect => {
                    current_mode = SyncMode::Disconnected;
                    current_room.clear();
                    current_token = None;
                    current_gist = None;
                    pending_publish = None;
                }
            }
        }

        match current_mode {
            SyncMode::Disconnected => {
                // Idle wait
                match inbox.recv_timeout(Duration::from_millis(500)) {
                    Ok(cmd) => {
                        let _ = inbox.try_recv(); // process next loop
                        match cmd {
                            SyncCmd::Host { room, github_token, gist_id } => {
                                current_mode = SyncMode::Host;
                                current_room = room;
                                current_token = github_token;
                                current_gist = gist_id;
                            }
                            SyncCmd::Join { room, gist_id } => {
                                current_mode = SyncMode::Guest;
                                current_room = room;
                                current_gist = gist_id;
                            }
                            _ => {}
                        }
                    }
                    Err(_) => {}
                }
            }
            SyncMode::Host => {
                if let Some(packet) = pending_publish.take() {
                    let json = serde_json::to_string(&packet).unwrap_or_default();
                    // 1. Send to instant relay
                    if !current_room.is_empty() {
                        let url = format!("https://ntfy.sh/auri-sync-{}", current_room);
                        let _ = ureq::post(&url)
                            .timeout(Duration::from_millis(2000))
                            .send_string(&json);
                    }
                    // 2. If GitHub Gist configured, sync to Gist
                    if let (Some(token), Some(gist_id)) = (&current_token, &current_gist) {
                        let gist_url = format!("https://api.github.com/gists/{}", gist_id);
                        let body = serde_json::json!({
                            "files": {
                                "auri-sync.json": {
                                    "content": json
                                }
                            }
                        });
                        let _ = ureq::patch(&gist_url)
                            .set("Authorization", &format!("Bearer {}", token))
                            .set("User-Agent", "AURI-Player")
                            .timeout(Duration::from_millis(2500))
                            .send_json(body);
                    }
                }
                // Check for updates or sleep a bit
                let _ = inbox.recv_timeout(Duration::from_millis(1500));
            }
            SyncMode::Guest => {
                // Poll from relay or gist
                let mut fetched_packet = None;

                if let Some(gist_id) = &current_gist {
                    if !gist_id.is_empty() {
                        let gist_url = format!("https://api.github.com/gists/{}", gist_id);
                        if let Ok(resp) = ureq::get(&gist_url)
                            .set("User-Agent", "AURI-Player")
                            .timeout(Duration::from_millis(2500))
                            .call()
                        {
                            if let Ok(val) = resp.into_json::<serde_json::Value>() {
                                if let Some(content) = val["files"]["auri-sync.json"]["content"].as_str() {
                                    if let Ok(pkt) = serde_json::from_str::<SyncPacket>(content) {
                                        fetched_packet = Some(pkt);
                                    }
                                }
                            }
                        }
                    }
                }

                if fetched_packet.is_none() && !current_room.is_empty() {
                    let url = format!("https://ntfy.sh/auri-sync-{}/json?poll=1", current_room);
                    if let Ok(resp) = ureq::get(&url).timeout(Duration::from_millis(2500)).call() {
                        if let Ok(text) = resp.into_string() {
                            for line in text.lines().rev() {
                                if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                                    if let Some(msg) = val["message"].as_str() {
                                        if let Ok(pkt) = serde_json::from_str::<SyncPacket>(msg) {
                                            fetched_packet = Some(pkt);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(pkt) = fetched_packet {
                    if active.load(Ordering::Relaxed) {
                        on_packet(pkt);
                    }
                }

                let _ = inbox.recv_timeout(Duration::from_millis(2000));
            }
        }
    }
}

pub fn now_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |e| e.as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_packet_serialization() {
        let packet = SyncPacket {
            room: "TEST99".to_owned(),
            video_id: "dQw4w9WgXcQ".to_owned(),
            title: "Never Gonna Give You Up".to_owned(),
            subtitle: "Rick Astley".to_owned(),
            position_ms: 42000,
            playing: true,
            timestamp_ms: 123456789,
        };

        let json = serde_json::to_string(&packet).unwrap();
        let decoded: SyncPacket = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.room, "TEST99");
        assert_eq!(decoded.video_id, "dQw4w9WgXcQ");
        assert_eq!(decoded.position_ms, 42000);
        assert!(decoded.playing);
    }
}

