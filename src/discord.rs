use crate::ytm::{self, Item};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::io::{self, Read, Write};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const APPLICATION_ID: &str = "1558230893519306802";
const PIPES: u8 = 10;
const HANDSHAKE: u32 = 0;
const FRAME: u32 = 1;
const CLOSE: u32 = 2;
#[allow(dead_code)]
const PING: u32 = 3;
const LISTENING: u8 = 2;
const DISCONNECTED_RETRY: Duration = Duration::from_millis(1500);
const CONNECTED_TIMEOUT: Duration = Duration::from_secs(30);
const SEEK_TOLERANCE_MS: u64 = 2000;
const COVER_SIDE: u32 = 512;
const SHORTEST_LINE: usize = 2;
const LONGEST_LINE: usize = 128;

#[derive(Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub enum Headline {
    #[default]
    App,
    Artist,
    Song,
    Custom,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscordFormat {
    pub show_when_paused: bool,
    pub hide_details: bool,
    pub status_text: crate::app::DiscordStatusField,
    pub app_name: String,
    pub line1: crate::app::DiscordLineContent,
    pub line2: crate::app::DiscordLineContent,
    pub line3: crate::app::DiscordLineContent,
    pub album_art: bool,
    pub provider_badge: bool,
    pub progress_bar: bool,
    pub line1_clickable: bool,
    pub line2_clickable: bool,
    pub art_clickable: bool,
    pub button1: crate::app::DiscordButtonOption,
    pub button2: crate::app::DiscordButtonOption,
}

impl Default for DiscordFormat {
    fn default() -> Self {
        Self {
            show_when_paused: false,
            hide_details: false,
            status_text: crate::app::DiscordStatusField::Line2Artist,
            app_name: String::new(),
            line1: crate::app::DiscordLineContent::Title,
            line2: crate::app::DiscordLineContent::Artist,
            line3: crate::app::DiscordLineContent::Album,
            album_art: true,
            provider_badge: true,
            progress_bar: true,
            line1_clickable: false,
            line2_clickable: false,
            art_clickable: false,
            button1: crate::app::DiscordButtonOption::ListenOnYtm,
            button2: crate::app::DiscordButtonOption::GetApp,
        }
    }
}

pub struct Discord {
    updates: Sender<Option<Value>>,
    shown: Option<(String, String, Headline, String, u64, DiscordFormat, bool, Option<(String, usize)>)>,
}

impl Discord {
    pub fn start(report: impl Fn(bool) + Send + 'static, on_join: impl Fn(String) + Send + Sync + 'static) -> Self {
        let (updates, inbox) = mpsc::channel();
        std::thread::spawn(move || run(inbox, report, on_join));
        Self { updates, shown: None }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        track: Option<(&Item, &str)>,
        headline: Headline,
        name: &str,
        position_ms: u64,
        duration_ms: u64,
        fmt: DiscordFormat,
        playing: bool,
        party: Option<(&str, usize)>,
    ) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_millis() as u64);
        let started = now.saturating_sub(position_ms);
        let name = if headline == Headline::Custom { name } else { "" };
        let party_owned = party.map(|(room, size)| (room.to_owned(), size));
        let unchanged = match (&self.shown, track) {
            (Some((video_id, art, lead, named, shown, old_fmt, was_playing, old_party)), Some((item, cover))) => {
                *video_id == item.video_id
                    && art == cover
                    && *lead == headline
                    && named == name
                    && *old_fmt == fmt
                    && *was_playing == playing
                    && *old_party == party_owned
                    && shown.abs_diff(started) < SEEK_TOLERANCE_MS
            }
            (None, None) => true,
            _ => false,
        };
        if unchanged {
            return;
        }
        self.shown = track.map(|(item, cover)| {
            (item.video_id.clone(), cover.to_owned(), headline, name.to_owned(), started, fmt.clone(), playing, party_owned.clone())
        });
        let _ = self
            .updates
            .send(track.map(|(item, cover)| activity(item, cover, headline, name, started, duration_ms, fmt, playing, party)));
    }
}

fn run(updates: Receiver<Option<Value>>, report: impl Fn(bool), on_join: impl Fn(String) + Send + Sync + 'static) {
    let on_join: Arc<dyn Fn(String) + Send + Sync> = Arc::new(on_join);
    let (mut pipe, mut activity, mut delivered, mut nonce) = (None, None, false, 0u64);
    let mut linked = None;
    loop {
        if pipe.is_none() {
            (pipe, delivered) = (connect(&on_join), false);
        }
        if let Some(open) = &mut pipe {
            if !delivered {
                nonce += 1;
                let args = json!({ "pid": std::process::id(), "activity": activity });
                // Write-only: the event reader thread owns all inbound frames.
                let sent = open.write_all(&frame(
                    FRAME,
                    &json!({ "cmd": "SET_ACTIVITY", "args": args, "nonce": nonce.to_string() }),
                ));
                delivered = sent.is_ok();
                if !delivered {
                    pipe = None;
                }
            }
        }
        if linked != Some(pipe.is_some()) {
            linked = Some(pipe.is_some());
            report(pipe.is_some());
        }
        let wait_time = if pipe.is_none() {
            DISCONNECTED_RETRY
        } else {
            CONNECTED_TIMEOUT
        };
        match updates.recv_timeout(wait_time) {
            Ok(update) => (activity, delivered) = (updates.try_iter().last().unwrap_or(update), false),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
    }
}

fn connect(on_join: &Arc<dyn Fn(String) + Send + Sync>) -> Option<Pipe> {
    (0..PIPES).find_map(|number| {
        let mut pipe = open(number)?;
        exchange(&mut pipe, HANDSHAKE, &json!({ "v": 1, "client_id": APPLICATION_ID })).ok()?;
        // Listen Together invites: subscribe to joins, then hand all inbound
        // frames to a dedicated reader so SET_ACTIVITY stays write-only.
        pipe.write_all(&frame(FRAME, &json!({ "cmd": "SUBSCRIBE", "evt": "ACTIVITY_JOIN", "nonce": "join-sub" })))
            .ok()?;
        let mut reader = pipe.try_clone().ok()?;
        let on_join = on_join.clone();
        std::thread::spawn(move || read_events(&mut reader, on_join));
        Some(pipe)
    })
}

fn read_events(pipe: &mut impl Read, on_join: Arc<dyn Fn(String) + Send + Sync>) {
    loop {
        let mut header = [0u8; 8];
        if pipe.read_exact(&mut header).is_err() {
            return;
        }
        let length = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;
        if length > 1024 * 1024 {
            return;
        }
        let mut body = vec![0u8; length];
        if pipe.read_exact(&mut body).is_err() {
            return;
        }
        let Ok(value) = serde_json::from_slice::<Value>(&body) else { continue };
        if value["evt"] == "ACTIVITY_JOIN" {
            if let Some(secret) = value["data"]["secret"].as_str().filter(|s| !s.is_empty()) {
                on_join(secret.to_owned());
            }
        }
    }
}

#[cfg(windows)]
type Pipe = std::fs::File;

#[cfg(windows)]
fn open(number: u8) -> Option<Pipe> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(format!(r"\\.\pipe\discord-ipc-{number}"))
        .ok()
}

#[cfg(unix)]
type Pipe = std::os::unix::net::UnixStream;

#[cfg(unix)]
const SANDBOXES: [&str; 3] = ["", "app/com.discordapp.Discord", "snap.discord"];

#[cfg(unix)]
fn open(number: u8) -> Option<Pipe> {
    let folder = ["XDG_RUNTIME_DIR", "TMPDIR", "TMP", "TEMP"]
        .into_iter()
        .find_map(std::env::var_os)
        .unwrap_or_else(|| "/tmp".into());
    let folder = std::path::Path::new(&folder);
    SANDBOXES
        .iter()
        .find_map(|sandbox| Pipe::connect(folder.join(sandbox).join(format!("discord-ipc-{number}"))).ok())
}

fn frame(opcode: u32, payload: &Value) -> Vec<u8> {
    let body = payload.to_string();
    [&opcode.to_le_bytes()[..], &(body.len() as u32).to_le_bytes(), body.as_bytes()].concat()
}

fn exchange(pipe: &mut (impl Read + Write), opcode: u32, payload: &Value) -> io::Result<()> {
    pipe.write_all(&frame(opcode, payload))?;
    let mut word = [0; 4];
    pipe.read_exact(&mut word)?;
    let answer = u32::from_le_bytes(word);
    pipe.read_exact(&mut word)?;
    let length = u64::from(u32::from_le_bytes(word));
    let skipped = io::copy(&mut pipe.take(length), &mut io::sink())?;
    if answer == CLOSE || skipped < length {
        return Err(io::ErrorKind::ConnectionAborted.into());
    }
    Ok(())
}

fn format_content(content: crate::app::DiscordLineContent, title: &str, artist: &str, album: &str) -> Option<String> {
    match content {
        crate::app::DiscordLineContent::Title => line(title),
        crate::app::DiscordLineContent::Artist => line(artist),
        crate::app::DiscordLineContent::Album => line(album),
        crate::app::DiscordLineContent::ArtistAlbum => {
            if !artist.is_empty() && !album.is_empty() {
                line(&format!("{artist} • {album}"))
            } else if !artist.is_empty() {
                line(artist)
            } else {
                line(album)
            }
        }
        crate::app::DiscordLineContent::Off => None,
    }
}

fn album_of(item: &Item) -> &str {
    if !item.extra.is_empty() {
        &item.extra
    } else {
        let parts: Vec<&str> = item.subtitle.split(" • ").collect();
        if parts.len() >= 2 {
            parts[1].trim()
        } else {
            ""
        }
    }
}

fn activity(
    item: &Item,
    cover: &str,
    headline: Headline,
    name: &str,
    started: u64,
    duration_ms: u64,
    fmt: DiscordFormat,
    playing: bool,
    party: Option<(&str, usize)>,
) -> Value {
    let app_name = if !fmt.app_name.is_empty() {
        fmt.app_name.as_str()
    } else if !name.is_empty() {
        name
    } else {
        "AURI"
    };

    if fmt.hide_details {
        let mut act = json!({
            "type": LISTENING,
            "name": app_name,
            "details": "Playing music",
        });
        act["assets"] = json!({
            "large_text": app_name,
        });
        apply_party(&mut act, party);
        return act;
    }

    let field = match fmt.status_text {
        crate::app::DiscordStatusField::Line2Artist => 1,
        crate::app::DiscordStatusField::Line1Title => 2,
        crate::app::DiscordStatusField::Line3Album => 1,
        crate::app::DiscordStatusField::ListeningToApp | crate::app::DiscordStatusField::Custom | crate::app::DiscordStatusField::None => match headline {
            Headline::App | Headline::Custom => 0,
            Headline::Artist => 1,
            Headline::Song => 2,
        },
    };

    let art_name = artist(item);
    let alb_name = album_of(item);
    let details_str = format_content(fmt.line1, &item.title, art_name, alb_name);
    let state_str = format_content(fmt.line2, &item.title, art_name, alb_name);
    let large_txt = format_content(fmt.line3, &item.title, art_name, alb_name).unwrap_or_else(|| app_name.to_string());

    let mut activity = json!({
        "type": LISTENING,
        "status_display_type": field,
        "name": app_name,
        "details": details_str,
        "state": state_str,
    });

    if fmt.progress_bar && playing {
        let start_secs = started / 1000;
        if duration_ms > 0 {
            let end_secs = (started + duration_ms) / 1000;
            activity["timestamps"] = json!({ "start": start_secs, "end": end_secs });
        } else {
            activity["timestamps"] = json!({ "start": start_secs });
        }
    }

    let mut assets = json!({
        "large_text": large_txt
    });
    if fmt.album_art && cover.starts_with("https://") {
        assets["large_image"] = json!(ytm::sized(cover, COVER_SIDE));
    }
    if fmt.provider_badge {
        assets["small_text"] = json!(app_name);
    }
    activity["assets"] = assets;

    let mut buttons = Vec::new();
    let add_button = |b: crate::app::DiscordButtonOption, btns: &mut Vec<Value>| {
        match b {
            crate::app::DiscordButtonOption::ListenOnYtm => {
                if !item.video_id.is_empty() {
                    btns.push(json!({
                        "label": "Listen on YouTube Music",
                        "url": format!("https://music.youtube.com/watch?v={}", item.video_id)
                    }));
                }
            }
            crate::app::DiscordButtonOption::GetApp => {
                btns.push(json!({
                    "label": format!("Get {}", app_name),
                    "url": "https://github.com/SuryakiranMK/AURI"
                }));
            }
            crate::app::DiscordButtonOption::None => {}
        }
    };
    add_button(fmt.button1, &mut buttons);
    if buttons.len() < 2 {
        add_button(fmt.button2, &mut buttons);
    }
    if !buttons.is_empty() {
        activity["buttons"] = json!(buttons);
    }

    apply_party(&mut activity, party);
    activity
}

/// The Listen Together invite: friends see a Join button on the user's Discord
/// profile, and clicking it hands the room code back via ACTIVITY_JOIN.
fn apply_party(activity: &mut Value, party: Option<(&str, usize)>) {
    if let Some((room, size)) = party {
        activity["party"] = json!({ "id": room, "size": [size.max(1), 20] });
        activity["secrets"] = json!({ "join": room });
    }
}

fn artist(item: &Item) -> &str {
    let mut parts = item.subtitle.split(" • ");
    let named = item
        .artists
        .first()
        .and_then(|(name, _)| parts.clone().find(|part| part.contains(name.as_str())));
    named.or_else(|| parts.next()).unwrap_or_default()
}

fn line(text: &str) -> Option<String> {
    let mut units = 0;
    let line: String = text
        .trim()
        .chars()
        .take_while(|letter| {
            units += letter.len_utf16();
            units <= LONGEST_LINE
        })
        .collect();
    (!line.is_empty()).then(|| format!("{line:<SHORTEST_LINE$}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakePipe {
        answer: io::Cursor<Vec<u8>>,
        written: Vec<u8>,
    }

    impl Read for FakePipe {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            self.answer.read(buffer)
        }
    }

    impl Write for FakePipe {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.written.write(bytes)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn frames_are_an_opcode_a_length_and_json() {
        let talk = |answer: Vec<u8>| {
            let mut pipe = FakePipe {
                answer: io::Cursor::new(answer),
                written: Vec::new(),
            };
            let accepted = exchange(&mut pipe, HANDSHAKE, &json!({ "v": 1 })).is_ok();
            (accepted, pipe.written, pipe.answer.position())
        };
        let ready = frame(FRAME, &json!({ "evt": "READY" }));
        let (accepted, written, read) = talk(ready.clone());
        assert!(accepted);
        assert_eq!(written, b"\0\0\0\0\x07\0\0\0{\"v\":1}");
        assert_eq!(read, ready.len() as u64);
        assert!(!talk(frame(CLOSE, &json!({ "code": 4000 }))).0);
        assert!(!talk(ready[..ready.len() - 1].to_vec()).0);
        assert!(!talk(Vec::new()).0);
    }

    #[test]
    fn activity_names_the_song_its_artist_and_its_time_bar() {
        let song = |title: &str, subtitle: &str, artists: &[&str]| Item {
            title: title.to_owned(),
            subtitle: subtitle.to_owned(),
            artists: artists.iter().map(|name| (name.to_string(), String::new())).collect(),
            ..Item::default()
        };
        let cover = "https://lh3.googleusercontent.com/abc=w120-h120-l90-rj";
        let lucky = song("Get Lucky", "Daft Punk • Random Access Memories", &[]);
        let mut fmt = DiscordFormat::default();
        fmt.status_text = crate::app::DiscordStatusField::ListeningToApp;
        let shown = activity(&lucky, cover, Headline::App, "", 1000, 248_000, fmt.clone(), true, None);
        assert_eq!(shown["type"], 2);
        assert_eq!(shown["status_display_type"], 0);
        assert_eq!(shown["name"], "AURI");
        assert_eq!(activity(&lucky, cover, Headline::Artist, "", 0, 1, fmt.clone(), true, None)["status_display_type"], 1);
        assert_eq!(activity(&lucky, cover, Headline::Song, "", 0, 1, fmt.clone(), true, None)["status_display_type"], 2);
        assert_eq!(shown["details"], "Get Lucky");
        assert_eq!(shown["state"], "Daft Punk");
        assert_eq!(shown["timestamps"], json!({ "start": 1, "end": 249 }));
        assert_eq!(
            shown["assets"]["large_image"],
            "https://lh3.googleusercontent.com/abc=w512-h512-l90-rj"
        );
        assert_eq!(shown["assets"]["large_text"], "Random Access Memories");

        let video = activity(&song("7", "Video • Prince • 12M views", &["Prince"]), "", Headline::App, "", 0, 1, fmt.clone(), true, None);
        assert_eq!(video["details"], "7 ");
        assert_eq!(video["state"], "Prince");
        assert_eq!(video["assets"]["large_text"], "Prince");

        let mut fmt_off = fmt.clone();
        fmt_off.line3 = crate::app::DiscordLineContent::Off;
        let video_off = activity(&song("7", "Video • Prince • 12M views", &["Prince"]), "", Headline::App, "", 0, 1, fmt_off, true, None);
        assert_eq!(video_off["assets"]["large_text"], "AURI");

        let unnamed = activity(&song(&"é😀".repeat(100), "", &[]), "", Headline::App, "", 0, 1, fmt.clone(), true, None);
        assert_eq!(unnamed["details"].as_str().unwrap().encode_utf16().count(), 127);
        assert!(unnamed["state"].is_null());

        let (updates, inbox) = mpsc::channel();
        let mut discord = Discord { updates, shown: None };
        discord.show(Some((&lucky, cover)), Headline::App, "", 0, 248_000, fmt.clone(), true, None);
        discord.show(Some((&lucky, cover)), Headline::App, "", 0, 248_000, fmt.clone(), true, None);
        discord.show(Some((&lucky, cover)), Headline::Artist, "mine", 0, 248_000, fmt.clone(), true, None);
        discord.show(Some((&lucky, cover)), Headline::Custom, "mine", 0, 248_000, fmt.clone(), true, None);
        discord.show(Some((&lucky, cover)), Headline::Custom, "m", 0, 248_000, fmt.clone(), true, None);
        let sent: Vec<_> = inbox
            .try_iter()
            .flatten()
            .map(|shown| (shown["status_display_type"].clone(), shown["name"].clone()))
            .collect();
        let expected = [
            (json!(0), json!("AURI")),
            (json!(1), json!("AURI")),
            (json!(0), json!("mine")),
            (json!(0), json!("m")),
        ];
        assert_eq!(sent, expected);
    }

    #[test]
    fn listen_together_party_adds_a_join_invite() {
        let fmt = DiscordFormat::default();
        let lucky = Item {
            title: "Get Lucky".to_owned(),
            subtitle: "Daft Punk • Random Access Memories".to_owned(),
            ..Item::default()
        };
        let act = activity(&lucky, "", Headline::App, "", 0, 248_000, fmt, true, Some(("K7QX-9M2T-B4", 3)));
        assert_eq!(act["party"]["id"], "K7QX-9M2T-B4");
        assert_eq!(act["party"]["size"][0], 3);
        assert_eq!(act["secrets"]["join"], "K7QX-9M2T-B4");
        let without = activity(&lucky, "", Headline::App, "", 0, 248_000, DiscordFormat::default(), true, None);
        assert!(without["party"].is_null());
        assert!(without["secrets"].is_null());
    }
}
