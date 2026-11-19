#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod art;
mod auth;
mod cache;
mod discord;
mod dsp;
mod hotkeys;
mod media;
mod opus;
mod player;
mod sync;
mod tray;
mod ui;
mod update;
mod ytm;

use eframe::egui;
use std::io::Write;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const APP_NAME: &str = "AURI";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_ID: &str = "auri";
const CRASH_LOG: &str = "crash.log";
const CRASH_LOG_BYTES: u64 = 64 * 1024;
const PROFILE_FLAG: &str = "--profile";
const PREVIOUS_VERSION_EXIT: Duration = Duration::from_millis(800);
static PROFILE: OnceLock<String> = OnceLock::new();

pub fn app_id() -> &'static str {
    PROFILE.get().map_or(APP_ID, String::as_str)
}

fn take_profile(arguments: &mut Vec<String>) {
    let Some(at) = arguments.iter().position(|argument| argument == PROFILE_FLAG) else {
        return;
    };
    let name = arguments.drain(at..(at + 2).min(arguments.len())).nth(1).unwrap_or_default();
    if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        let _ = PROFILE.set(format!("{APP_ID}-{name}"));
    }
}

pub fn profile_arguments() -> Vec<String> {
    let name = PROFILE.get().and_then(|id| id.strip_prefix(APP_ID)?.strip_prefix('-'));
    name.map(|name| vec![PROFILE_FLAG.to_owned(), name.to_owned()]).unwrap_or_default()
}

fn take_updated(arguments: &mut Vec<String>) -> bool {
    let before = arguments.len();
    arguments.retain(|argument| argument != update::UPDATED_FLAG);
    arguments.len() < before
}

#[cfg(windows)]
fn attach_console() {
    use windows_sys::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};
    unsafe { AttachConsole(ATTACH_PARENT_PROCESS) };
}

#[cfg(not(windows))]
fn attach_console() {}

fn parse_join_code(arg: &str) -> Option<String> {
    let clean = arg.trim().trim_matches('"').trim_matches('\'');
    let code_candidate = if let Some(rest) = clean.strip_prefix("auri://join/") {
        rest.trim_matches('/')
    } else if let Some(rest) = clean.strip_prefix("auri://join?room=") {
        rest.split('&').next().unwrap_or(rest)
    } else if let Some(rest) = clean.strip_prefix("auri://") {
        rest.strip_prefix("join/").or_else(|| rest.strip_prefix("join?room=")).unwrap_or(rest).trim_matches('/')
    } else {
        return None;
    };
    let code = code_candidate.trim().to_uppercase();
    if crate::sync::is_valid_room_code(&code) {
        Some(code)
    } else {
        None
    }
}

#[cfg(windows)]
pub fn register_protocol() {
    use std::os::windows::process::CommandExt;
    if let Ok(exe) = std::env::current_exe() {
        let exe_str = exe.to_string_lossy();
        let cmd = format!("\"{}\" \"%1\"", exe_str);
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let _ = std::process::Command::new("reg")
            .args(&["add", r"HKCU\Software\Classes\auri", "/ve", "/d", "URL:AURI Protocol", "/f"])
            .creation_flags(CREATE_NO_WINDOW)
            .status();
        let _ = std::process::Command::new("reg")
            .args(&["add", r"HKCU\Software\Classes\auri", "/v", "URL Protocol", "/d", "", "/f"])
            .creation_flags(CREATE_NO_WINDOW)
            .status();
        let _ = std::process::Command::new("reg")
            .args(&["add", r"HKCU\Software\Classes\auri\shell\open\command", "/ve", "/d", &cmd, "/f"])
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    }
}

#[cfg(not(windows))]
pub fn register_protocol() {}

#[cfg(windows)]
pub fn start_ipc_server(sender: std::sync::mpsc::Sender<app::Event>, ctx: egui::Context) {
    let profile = app_id();
    let folder = match eframe::storage_dir(profile) {
        Some(f) => f,
        None => return,
    };
    let _ = std::fs::create_dir_all(&folder);
    let lock_file = folder.join("ipc.lock");

    let listener = match std::net::TcpListener::bind("127.0.0.1:0") {
        Ok(l) => l,
        Err(_) => return,
    };
    let port = match listener.local_addr() {
        Ok(addr) => addr.port(),
        Err(_) => return,
    };
    let token = crate::sync::new_room_code();
    let _ = std::fs::write(&lock_file, format!("{}:{}", port, token));

    std::thread::spawn(move || {
        use std::io::{BufRead, BufReader, Write};
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
                let mut reader = BufReader::new(&stream);
                let mut line = String::new();
                if reader.read_line(&mut line).is_ok() {
                    let parts: Vec<&str> = line.trim().split_whitespace().collect();
                    if parts.len() >= 2 && parts[0] == token {
                        match parts[1] {
                            "JOIN" => {
                                if let Some(code) = parts.get(2) {
                                    if crate::sync::is_valid_room_code(code) {
                                        let _ = sender.send(app::Event::Pressed(app::Action::PromptSyncJoin(code.to_string())));
                                        ctx.request_repaint();
                                    }
                                }
                            }
                            "FOCUS" => {
                                let _ = sender.send(app::Event::Pressed(app::Action::RestoreFromTray));
                                ctx.request_repaint();
                            }
                            _ => {}
                        }
                        let _ = stream.write_all(b"OK\n");
                    }
                }
            }
        }
    });
}

#[cfg(not(windows))]
pub fn start_ipc_server(_sender: std::sync::mpsc::Sender<app::Event>, _ctx: egui::Context) {}

/// One AURI per profile. A named mutex keyed by the profile id makes a plain
/// double-click focus the running instance instead of stacking a second
/// process — a leftover process left in Mini Mode would otherwise keep its
/// always-on-top capsule floating over the window. Explicit `--profile`
/// launches get their own mutex, so multi-instance testing still works.
#[cfg(windows)]
fn enforce_single_instance(join_code: Option<&str>) {
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    use windows_sys::Win32::System::Threading::CreateMutexW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE};

    let profile = app_id();
    let name: Vec<u16> = format!("Local\\AURI.{profile}\0").encode_utf16().collect();
    // The handle is intentionally never closed: the OS releases it on exit,
    // keeping the mutex alive for this process's whole lifetime.
    let _mutex = unsafe { CreateMutexW(std::ptr::null(), 1, name.as_ptr()) };
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        // Forward code or focus to running instance via authenticated loopback IPC
        if let Some(folder) = eframe::storage_dir(profile) {
            let lock_file = folder.join("ipc.lock");
            if let Ok(content) = std::fs::read_to_string(&lock_file) {
                let mut parts = content.trim().split(':');
                if let (Some(port_str), Some(token)) = (parts.next(), parts.next()) {
                    if let Ok(port) = port_str.parse::<u16>() {
                        use std::io::Write;
                        use std::net::TcpStream;
                        if let Ok(mut stream) = TcpStream::connect_timeout(
                            &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
                            Duration::from_millis(400),
                        ) {
                            let msg = if let Some(code) = join_code {
                                format!("{token} JOIN {code}\n")
                            } else {
                                format!("{token} FOCUS\n")
                            };
                            let _ = stream.write_all(msg.as_bytes());
                            let _ = stream.flush();
                        }
                    }
                }
            }
        }

        let title: Vec<u16> = format!("{APP_NAME}\0").encode_utf16().collect();
        let window = unsafe { FindWindowW(std::ptr::null(), title.as_ptr()) } as isize;
        if window != 0 {
            unsafe {
                ShowWindow(window as _, SW_RESTORE);
                SetForegroundWindow(window as _);
            }
        }
        std::process::exit(0);
    }
}

#[cfg(not(windows))]
fn enforce_single_instance(_join_code: Option<&str>) {}

fn log_crashes() {
    let Some(folder) = eframe::storage_dir(app_id()) else { return };
    std::panic::set_hook(Box::new(move |panic| {
        let file = folder.join(CRASH_LOG);
        let seconds = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_secs());
        let grown = std::fs::metadata(&file).is_ok_and(|log| log.len() > CRASH_LOG_BYTES);
        let _ = std::fs::create_dir_all(&folder);
        let log = std::fs::OpenOptions::new()
            .create(true)
            .append(!grown)
            .write(true)
            .truncate(grown)
            .open(file);
        if let Ok(mut log) = log {
            let _ = writeln!(log, "{APP_NAME} {VERSION} at unix {seconds}: {panic}");
        }
    }));
}

fn main() -> eframe::Result {
    let mut arguments: Vec<String> = std::env::args().skip(1).collect();
    take_profile(&mut arguments);
    log_crashes();
    if take_updated(&mut arguments) {
        std::thread::sleep(PREVIOUS_VERSION_EXIT);
    }
    update::clear_leftovers();

    // Check for auri:// deep-link argument
    let mut pending_join = None;
    arguments.retain(|arg| {
        if let Some(code) = parse_join_code(arg) {
            pending_join = Some(code);
            false
        } else {
            true
        }
    });

    let mut arguments = arguments.into_iter();
    let (flag, value) = (arguments.next(), arguments.next());
    if flag.as_deref() == Some("--selftest") {
        attach_console();
        match player::selftest(value.as_deref().unwrap_or("daft punk get lucky")) {
            Ok(report) => println!("{report}"),
            Err(error) => {
                eprintln!("selftest failed: {error}");
                std::process::exit(1);
            }
        }
        return Ok(());
    }
    let (query, autoplay) = match (flag, value) {
        (Some(flag), Some(query)) if flag == "--play" => (Some(query), true),
        (Some(query), None) if !query.starts_with("--") => (Some(query), false),
        _ => (None, false),
    };
    enforce_single_instance(pending_join.as_deref());
    let viewport = egui::ViewportBuilder::default()
        .with_title(APP_NAME)
        .with_app_id(app_id())
        .with_inner_size(app::WINDOW_SIZE)
        .with_min_inner_size(app::WINDOW_MIN)
        .with_maximized(true)
        .with_transparent(true)
        .with_decorations(false)
        .with_close_button(true)
        .with_icon(ui::app_icon());
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        APP_NAME,
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, query, autoplay, pending_join)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_join_code() {
        assert_eq!(parse_join_code("auri://join/K7QX-9M2T-B4"), Some("K7QX-9M2T-B4".to_string()));
        assert_eq!(parse_join_code("auri://join/K7QX-9M2T-B4/"), Some("K7QX-9M2T-B4".to_string()));
        assert_eq!(parse_join_code("\"auri://join/K7QX-9M2T-B4\""), Some("K7QX-9M2T-B4".to_string()));
        assert_eq!(parse_join_code("auri://join?room=K7QX-9M2T-B4"), Some("K7QX-9M2T-B4".to_string()));
        assert_eq!(parse_join_code("auri://join/INVALID"), None);
        assert_eq!(parse_join_code("--play"), None);
        assert_eq!(parse_join_code("daft punk"), None);
    }

    #[cfg(windows)]
    #[test]
    fn test_register_protocol() {
        register_protocol();
        let out = std::process::Command::new("reg")
            .args(&["query", r"HKCU\Software\Classes\auri"])
            .output()
            .expect("reg query should succeed");
        assert!(out.status.success());
    }
}
