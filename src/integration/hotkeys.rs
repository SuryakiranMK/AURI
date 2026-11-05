// Windows Global Hotkey Listener
// Supports hardware media keys and custom gaming shortcuts (Win+Alt+Space, Win+Alt+Right, Win+Alt+M).

use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HotkeyAction {
    PlayPause,
    NextTrack,
    PrevTrack,
    VolumeUp,
    VolumeDown,
    CycleMode,
    ToggleWindow,
}

// Windows Virtual Key codes
const VK_SPACE: u32 = 0x20;
const VK_LEFT: u32 = 0x25;
const VK_UP: u32 = 0x26;
const VK_RIGHT: u32 = 0x27;
const VK_DOWN: u32 = 0x28;
const VK_KEY_M: u32 = 0x4D;

// Windows Multimedia keys
const VK_MEDIA_NEXT_TRACK: u32 = 0xB0;
const VK_MEDIA_PREV_TRACK: u32 = 0xB1;
const VK_MEDIA_STOP: u32 = 0xB2;
const VK_MEDIA_PLAY_PAUSE: u32 = 0xB3;

// Modifiers
const MOD_ALT: u32 = 0x0001;
const MOD_WIN: u32 = 0x0008;
const MOD_NOREPEAT: u32 = 0x4000;

const WM_HOTKEY: u32 = 0x0312;
const WM_QUIT: u32 = 0x0012;

#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
struct Msg {
    hwnd: *mut std::ffi::c_void,
    message: u32,
    wparam: usize,
    lparam: isize,
    time: u32,
    pt: Point,
    lprivate: u32,
}

#[cfg(windows)]
#[link(name = "user32")]
extern "system" {
    fn RegisterHotKey(
        hwnd: *mut std::ffi::c_void,
        id: i32,
        fs_modifiers: u32,
        vk: u32,
    ) -> i32;
    fn UnregisterHotKey(hwnd: *mut std::ffi::c_void, id: i32) -> i32;
    fn GetMessageW(
        lp_msg: *mut Msg,
        hwnd: *mut std::ffi::c_void,
        w_msg_filter_min: u32,
        w_msg_filter_max: u32,
    ) -> i32;
    fn PostThreadMessageW(id_thread: u32, msg: u32, wparam: usize, lparam: isize) -> i32;
    fn GetCurrentThreadId() -> u32;
}

pub struct GlobalHotkeys {
    action_rx: Mutex<Receiver<HotkeyAction>>,
    is_running: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
    thread_id: Arc<std::sync::atomic::AtomicU32>,
}

impl GlobalHotkeys {
    pub fn new() -> Self {
        let (action_tx, action_rx) = channel::<HotkeyAction>();
        let is_running = Arc::new(AtomicBool::new(true));
        let thread_id = Arc::new(std::sync::atomic::AtomicU32::new(0));

        let is_running_clone = is_running.clone();
        let thread_id_clone = thread_id.clone();

        let handle = thread::Builder::new()
            .name("auri-hotkey-listener".to_string())
            .spawn(move || {
                Self::run_listener(action_tx, is_running_clone, thread_id_clone);
            })
            .ok();

        Self {
            action_rx: Mutex::new(action_rx),
            is_running,
            thread_handle: handle,
            thread_id,
        }
    }

    pub fn try_recv(&self) -> Option<HotkeyAction> {
        self.action_rx.lock().ok()?.try_recv().ok()
    }

    fn run_listener(
        tx: Sender<HotkeyAction>,
        is_running: Arc<AtomicBool>,
        thread_id: Arc<std::sync::atomic::AtomicU32>,
    ) {
        #[cfg(windows)]
        unsafe {
            let tid = GetCurrentThreadId();
            thread_id.store(tid, std::sync::atomic::Ordering::SeqCst);

            let null_hwnd = std::ptr::null_mut();

            // Register Hotkeys: (ID, Modifiers, VK, Action)
            let hotkeys = [
                (1, MOD_WIN | MOD_ALT | MOD_NOREPEAT, VK_SPACE, HotkeyAction::PlayPause),
                (2, MOD_WIN | MOD_ALT | MOD_NOREPEAT, VK_RIGHT, HotkeyAction::NextTrack),
                (3, MOD_WIN | MOD_ALT | MOD_NOREPEAT, VK_LEFT, HotkeyAction::PrevTrack),
                (4, MOD_WIN | MOD_ALT | MOD_NOREPEAT, VK_UP, HotkeyAction::VolumeUp),
                (5, MOD_WIN | MOD_ALT | MOD_NOREPEAT, VK_DOWN, HotkeyAction::VolumeDown),
                (6, MOD_WIN | MOD_ALT | MOD_NOREPEAT, VK_KEY_M, HotkeyAction::CycleMode),
                // Hardware Media Keys
                (10, MOD_NOREPEAT, VK_MEDIA_PLAY_PAUSE, HotkeyAction::PlayPause),
                (11, MOD_NOREPEAT, VK_MEDIA_NEXT_TRACK, HotkeyAction::NextTrack),
                (12, MOD_NOREPEAT, VK_MEDIA_PREV_TRACK, HotkeyAction::PrevTrack),
                (13, MOD_NOREPEAT, VK_MEDIA_STOP, HotkeyAction::PlayPause),
            ];

            let mut registered_count = 0;
            for &(id, mods, vk, _) in &hotkeys {
                if RegisterHotKey(null_hwnd, id, mods, vk) != 0 {
                    registered_count += 1;
                }
            }

            println!("[AURI Hotkeys] Registered {} global hotkeys & media key shortcuts", registered_count);

            let mut msg: Msg = std::mem::zeroed();
            while is_running.load(std::sync::atomic::Ordering::Relaxed) {
                let res = GetMessageW(&mut msg, null_hwnd, 0, 0);
                if res <= 0 || msg.message == WM_QUIT {
                    break;
                }

                if msg.message == WM_HOTKEY {
                    let id = msg.wparam as i32;
                    for &(hk_id, _, _, action) in &hotkeys {
                        if hk_id == id {
                            let _ = tx.send(action);
                            break;
                        }
                    }
                }
            }

            // Unregister hotkeys on shutdown
            for &(id, _, _, _) in &hotkeys {
                UnregisterHotKey(null_hwnd, id);
            }
        }
    }
}

impl Drop for GlobalHotkeys {
    fn drop(&mut self) {
        self.is_running.store(false, std::sync::atomic::Ordering::SeqCst);
        #[cfg(windows)]
        unsafe {
            let tid = self.thread_id.load(std::sync::atomic::Ordering::SeqCst);
            if tid != 0 {
                PostThreadMessageW(tid, WM_QUIT, 0, 0);
            }
        }
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_hotkeys_init_and_drop() {
        let hotkeys = GlobalHotkeys::new();
        // Verifies the listener thread starts and drops cleanly
        drop(hotkeys);
    }
}
