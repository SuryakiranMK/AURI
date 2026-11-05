// Project AURI: Anti-Cheat Safe Game Detector
// Passive foreground listener via SetWinEventHook(EVENT_SYSTEM_FOREGROUND)
// Zero code injection, zero memory scanning, zero game process hooking.
// Fully compliant with BattlEye, Easy Anti-Cheat, Riot Vanguard, Ricochet, and VAC.

use std::collections::HashSet;
use std::ffi::{c_void, OsString};
use std::os::windows::ffi::OsStringExt;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

const EVENT_SYSTEM_FOREGROUND: u32 = 0x0003;
const WINEVENT_OUTOFCONTEXT: u32 = 0x0000;
const WINEVENT_SKIPOWNPROCESS: u32 = 0x0002;
const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
const SM_CXSCREEN: i32 = 0;
const SM_CYSCREEN: i32 = 1;
const WM_QUIT: u32 = 0x0012;
const PM_REMOVE: u32 = 0x0001;
const QS_ALLINPUT: u32 = 0x04FF;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct Msg {
    hwnd: *mut c_void,
    message: u32,
    wparam: usize,
    lparam: isize,
    time: u32,
    pt: Point,
    lprivate: u32,
}

type WINEVENTPROC = unsafe extern "system" fn(
    h_win_event_hook: *mut c_void,
    event: u32,
    hwnd: *mut c_void,
    id_object: i32,
    id_child: i32,
    id_event_thread: u32,
    dwms_event_time: u32,
);

#[link(name = "user32")]
extern "system" {
    fn SetWinEventHook(
        event_min: u32,
        event_max: u32,
        hmod_win_event_proc: *mut c_void,
        lpfn_win_event_proc: Option<WINEVENTPROC>,
        id_process: u32,
        id_thread: u32,
        dw_flags: u32,
    ) -> *mut c_void;
    fn UnhookWinEvent(h_win_event_hook: *mut c_void) -> i32;
    fn GetWindowThreadProcessId(hwnd: *mut c_void, lpdw_process_id: *mut u32) -> u32;
    fn GetWindowRect(hwnd: *mut c_void, lp_rect: *mut RECT) -> i32;
    fn GetSystemMetrics(n_index: i32) -> i32;
    fn MsgWaitForMultipleObjectsEx(
        n_count: u32,
        p_handles: *const *mut c_void,
        dw_milliseconds: u32,
        dw_wake_mask: u32,
        dw_flags: u32,
    ) -> u32;
    fn PeekMessageW(
        lp_msg: *mut Msg,
        hwnd: *mut c_void,
        w_msg_filter_min: u32,
        w_msg_filter_max: u32,
        w_remove_msg: u32,
    ) -> i32;
    fn TranslateMessage(lp_msg: *const Msg) -> i32;
    fn DispatchMessageW(lp_msg: *const Msg) -> isize;
    fn PostThreadMessageW(id_thread: u32, msg: u32, w_param: usize, l_param: isize) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcessId() -> u32;
    fn GetCurrentThreadId() -> u32;
    fn OpenProcess(dw_desired_access: u32, b_inherit_handle: i32, dw_process_id: u32) -> *mut c_void;
    fn QueryFullProcessImageNameW(
        h_process: *mut c_void,
        dw_flags: u32,
        lp_exe_name: *mut u16,
        lpdw_size: *mut u32,
    ) -> i32;
    fn CloseHandle(h_object: *mut c_void) -> i32;
}

/// Events emitted by the Anti-Cheat Safe Game Detector
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEvent {
    /// A recognized game entered the foreground
    GameFocused { process_name: String },
    /// Foreground window is no longer a recognized game
    GameUnfocused,
}

struct DetectorSharedState {
    known_games: RwLock<HashSet<String>>,
    excluded_apps: RwLock<HashSet<String>>,
    auto_detect_enabled: AtomicBool,
    fullscreen_heuristic: AtomicBool,
    last_detected_game: Mutex<Option<String>>,
    event_tx: Mutex<Option<Sender<GameEvent>>>,
}

static DETECTOR_STATE: OnceLock<Arc<DetectorSharedState>> = OnceLock::new();

fn get_shared_state() -> &'static Arc<DetectorSharedState> {
    DETECTOR_STATE.get_or_init(|| {
        let mut games = HashSet::new();
        // High-profile esports, multiplayer and demanding games
        let default_games = [
            "cs2.exe",
            "csgo.exe",
            "valorant.exe",
            "valorant-win64-shipping.exe",
            "dota2.exe",
            "league of legends.exe",
            "leagueclientuxrender.exe",
            "overwatch.exe",
            "r5apex.exe",
            "fortniteclient-win64-shipping.exe",
            "destiny2.exe",
            "rainbowsix.exe",
            "rainbowsix_vulkan.exe",
            "cod.exe",
            "gta5.exe",
            "cyberpunk2077.exe",
            "eldenring.exe",
            "starfield.exe",
            "bg3.exe",
            "bg3_dx11.exe",
            "helldivers2.exe",
            "rocketleague.exe",
            "minecraft.exe",
            "javaw.exe",
            "genshinimpact.exe",
            "honkaistarrail.exe",
            "pubg.exe",
            "tslgame.exe",
            "warframe.x64.exe",
            "osu!.exe",
            "rustclient.exe",
            "monsterhunterwilds.exe",
            "witcher3.exe",
        ];
        for g in default_games {
            games.insert(g.to_lowercase());
        }

        let mut exclusions = HashSet::new();
        let default_exclusions = [
            "explorer.exe",
            "chrome.exe",
            "firefox.exe",
            "msedge.exe",
            "code.exe",
            "taskmgr.exe",
            "auri.exe",
            "devenv.exe",
            "powershell.exe",
            "cmd.exe",
            "windowsterminal.exe",
            "discord.exe",
            "slack.exe",
            "spotify.exe",
        ];
        for ex in default_exclusions {
            exclusions.insert(ex.to_lowercase());
        }

        Arc::new(DetectorSharedState {
            known_games: RwLock::new(games),
            excluded_apps: RwLock::new(exclusions),
            auto_detect_enabled: AtomicBool::new(true),
            fullscreen_heuristic: AtomicBool::new(true),
            last_detected_game: Mutex::new(None),
            event_tx: Mutex::new(None),
        })
    })
}

/// Retrieves the base executable name for a given process ID
pub fn get_process_name_by_pid(pid: u32) -> Option<String> {
    if pid == 0 || pid == 4 {
        return None;
    }
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return None;
        }

        let mut buffer = [0u16; 1024];
        let mut size = buffer.len() as u32;

        let ok = QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut size);
        CloseHandle(handle);

        if ok != 0 && size > 0 {
            let full_path = OsString::from_wide(&buffer[..size as usize]);
            let path_str = full_path.to_string_lossy().to_string();
            // Extract file name
            if let Some(pos) = path_str.rfind('\\') {
                Some(path_str[pos + 1..].to_lowercase())
            } else {
                Some(path_str.to_lowercase())
            }
        } else {
            None
        }
    }
}

/// Checks if a window covers the full primary screen (fullscreen heuristic)
pub fn is_window_fullscreen(hwnd: *mut c_void) -> bool {
    if hwnd.is_null() {
        return false;
    }
    unsafe {
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect) == 0 {
            return false;
        }

        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);

        rect.left <= 0 && rect.top <= 0 && rect.right >= screen_w && rect.bottom >= screen_h
    }
}

/// WinEvent callback invoked strictly out-of-context in AURI's background thread
unsafe extern "system" fn winevent_proc(
    _hook: *mut c_void,
    event: u32,
    hwnd: *mut c_void,
    _id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    if event != EVENT_SYSTEM_FOREGROUND || hwnd.is_null() {
        return;
    }

    let state = get_shared_state();
    if !state.auto_detect_enabled.load(Ordering::Relaxed) {
        return;
    }

    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);

    if pid == 0 || pid == GetCurrentProcessId() {
        return;
    }

    let process_name = match get_process_name_by_pid(pid) {
        Some(name) => name,
        None => return,
    };

    let is_known = {
        let games = state.known_games.read().unwrap();
        games.contains(&process_name)
    };

    let is_fullscreen_game = if !is_known && state.fullscreen_heuristic.load(Ordering::Relaxed) {
        let exclusions = state.excluded_apps.read().unwrap();
        if !exclusions.contains(&process_name) {
            is_window_fullscreen(hwnd)
        } else {
            false
        }
    } else {
        false
    };

    let mut last_game = state.last_detected_game.lock().unwrap();
    let tx_opt = state.event_tx.lock().unwrap();

    if is_known || is_fullscreen_game {
        let game_id = process_name.clone();
        if last_game.as_deref() != Some(&game_id) {
            *last_game = Some(game_id.clone());
            if let Some(tx) = tx_opt.as_ref() {
                let _ = tx.send(GameEvent::GameFocused {
                    process_name: game_id,
                });
            }
        }
    } else if last_game.is_some() {
        *last_game = None;
        if let Some(tx) = tx_opt.as_ref() {
            let _ = tx.send(GameEvent::GameUnfocused);
        }
    }
}

/// Controller handle for the Game Detector
pub struct GameDetector {
    is_running: Arc<AtomicBool>,
    thread_id: Arc<AtomicU32>,
    join_handle: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl GameDetector {
    /// Starts the anti-cheat safe background game detector
    pub fn start() -> (Self, Receiver<GameEvent>) {
        let (tx, rx) = channel();
        let state = get_shared_state();
        {
            let mut event_tx = state.event_tx.lock().unwrap();
            *event_tx = Some(tx);
        }

        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_clone = is_running.clone();
        let thread_id_atomic = Arc::new(AtomicU32::new(0));
        let thread_id_clone = thread_id_atomic.clone();

        let handle = std::thread::Builder::new()
            .name("auri-game-detector".to_string())
            .spawn(move || {
                let tid = unsafe { GetCurrentThreadId() };
                thread_id_clone.store(tid, Ordering::SeqCst);

                let hook = unsafe {
                    SetWinEventHook(
                        EVENT_SYSTEM_FOREGROUND,
                        EVENT_SYSTEM_FOREGROUND,
                        std::ptr::null_mut(),
                        Some(winevent_proc),
                        0,
                        0,
                        WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
                    )
                };

                if hook.is_null() {
                    eprintln!("[AURI] Failed to install SetWinEventHook for game detector");
                    return;
                }

                while is_running_clone.load(Ordering::Relaxed) {
                    unsafe {
                        MsgWaitForMultipleObjectsEx(0, std::ptr::null(), 50, QS_ALLINPUT, 0);
                        let mut msg = Msg::default();
                        while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                            if msg.message == WM_QUIT {
                                is_running_clone.store(false, Ordering::Relaxed);
                                break;
                            }
                            TranslateMessage(&msg);
                            DispatchMessageW(&msg);
                        }
                    }
                }

                unsafe {
                    UnhookWinEvent(hook);
                }
            })
            .expect("Failed to spawn game detector thread");

        (
            Self {
                is_running,
                thread_id: thread_id_atomic,
                join_handle: Mutex::new(Some(handle)),
            },
            rx,
        )
    }

    /// Enable or disable automatic game detection
    pub fn set_enabled(&self, enabled: bool) {
        get_shared_state().auto_detect_enabled.store(enabled, Ordering::SeqCst);
    }

    /// Returns whether auto game detection is currently enabled
    pub fn is_enabled(&self) -> bool {
        get_shared_state().auto_detect_enabled.load(Ordering::Relaxed)
    }

    /// Add a custom game process name (e.g. "mygame.exe")
    pub fn add_custom_game(&self, exe_name: &str) {
        let mut games = get_shared_state().known_games.write().unwrap();
        games.insert(exe_name.to_lowercase());
    }

    /// Returns the currently active focused game if any
    pub fn current_game(&self) -> Option<String> {
        let last = get_shared_state().last_detected_game.lock().unwrap();
        last.clone()
    }
}

impl Drop for GameDetector {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);
        let tid = self.thread_id.load(Ordering::SeqCst);
        if tid != 0 {
            unsafe {
                PostThreadMessageW(tid, WM_QUIT, 0, 0);
            }
        }
        // Disconnect channel
        if let Ok(mut tx) = get_shared_state().event_tx.lock() {
            *tx = None;
        }
        if let Ok(mut lock) = self.join_handle.lock() {
            if let Some(handle) = lock.take() {
                let _ = handle.join();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_game_lookup() {
        let state = get_shared_state();
        let games = state.known_games.read().unwrap();
        assert!(games.contains("cs2.exe"));
        assert!(games.contains("valorant.exe"));
        assert!(games.contains("dota2.exe"));
        assert!(games.contains("eldenring.exe"));
        assert!(!games.contains("notepad.exe"));
    }

    #[test]
    fn test_exclusion_list() {
        let state = get_shared_state();
        let exclusions = state.excluded_apps.read().unwrap();
        assert!(exclusions.contains("chrome.exe"));
        assert!(exclusions.contains("explorer.exe"));
        assert!(exclusions.contains("auri.exe"));
    }

    #[test]
    fn test_custom_game_registration() {
        let (detector, _rx) = GameDetector::start();
        detector.add_custom_game("custom_rpg.exe");
        let state = get_shared_state();
        let games = state.known_games.read().unwrap();
        assert!(games.contains("custom_rpg.exe"));
    }
}
