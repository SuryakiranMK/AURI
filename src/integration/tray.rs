// Windows System Tray Integration (Shell_NotifyIconW)
// Provides background playback persistence, system tray icon, and context menu.

use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, OnceLock};
use std::thread::{self, JoinHandle};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TrayAction {
    TogglePlay,
    NextTrack,
    PrevTrack,
    SetMode(i32),
    ToggleWindow,
    Exit,
}

// Win32 Menu Item IDs
const ID_MENU_PLAY: usize = 101;
const ID_MENU_NEXT: usize = 102;
const ID_MENU_PREV: usize = 103;
const ID_MODE_NORMAL: usize = 201;
const ID_MODE_MINI: usize = 202;
const ID_MODE_GAMING: usize = 203;
const ID_MODE_HEADLESS: usize = 204;
const ID_MENU_TOGGLE_WIN: usize = 301;
const ID_MENU_EXIT: usize = 999;

// Shell_NotifyIcon message constants
const NIM_ADD: u32 = 0x00000000;
const NIM_MODIFY: u32 = 0x00000001;
const NIM_DELETE: u32 = 0x00000002;

const NIF_MESSAGE: u32 = 0x00000001;
const NIF_ICON: u32 = 0x00000002;
const NIF_TIP: u32 = 0x00000004;

const WM_APP: u32 = 0x8000;
const WM_TRAY_CALLBACK: u32 = WM_APP + 42;

const WM_LBUTTONUP: isize = 0x0202;
const WM_LBUTTONDBLCLK: isize = 0x0203;
const WM_RBUTTONUP: isize = 0x0205;

const MF_STRING: u32 = 0x00000000;
const MF_GRAYED: u32 = 0x00000001;
const MF_DISABLED: u32 = 0x00000002;
const MF_SEPARATOR: u32 = 0x00000800;

const TPM_RIGHTBUTTON: u32 = 0x0002;
const TPM_RETURNCMD: u32 = 0x0100;

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

#[repr(C)]
struct NotifyIconDataW {
    cb_size: u32,
    hwnd: *mut std::ffi::c_void,
    u_id: u32,
    u_flags: u32,
    u_callback_message: u32,
    h_icon: *mut std::ffi::c_void,
    sz_tip: [u16; 128],
    dw_state: u32,
    dw_state_mask: u32,
    sz_info: [u16; 256],
    u_timeout_or_version: u32,
    sz_info_title: [u16; 64],
    dw_info_flags: u32,
    guid_item: [u8; 16],
    h_balloon_icon: *mut std::ffi::c_void,
}

#[repr(C)]
struct WndClassExW {
    cb_size: u32,
    style: u32,
    lpfn_wnd_proc: Option<unsafe extern "system" fn(*mut std::ffi::c_void, u32, usize, isize) -> isize>,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: *mut std::ffi::c_void,
    h_icon: *mut std::ffi::c_void,
    h_cursor: *mut std::ffi::c_void,
    hbr_background: *mut std::ffi::c_void,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
    h_icon_sm: *mut std::ffi::c_void,
}

#[cfg(windows)]
#[link(name = "user32")]
#[link(name = "shell32")]
extern "system" {
    fn Shell_NotifyIconW(dw_message: u32, lp_data: *const NotifyIconDataW) -> i32;
    fn RegisterClassExW(lp_wnd_class: *const WndClassExW) -> u16;
    fn CreateWindowExW(
        dw_ex_style: u32,
        lp_class_name: *const u16,
        lp_window_name: *const u16,
        dw_style: u32,
        x: i32,
        y: i32,
        n_width: i32,
        n_height: i32,
        h_wnd_parent: *mut std::ffi::c_void,
        h_menu: *mut std::ffi::c_void,
        h_instance: *mut std::ffi::c_void,
        lp_param: *mut std::ffi::c_void,
    ) -> *mut std::ffi::c_void;
    fn DefWindowProcW(hwnd: *mut std::ffi::c_void, msg: u32, wparam: usize, lparam: isize) -> isize;
    fn DestroyWindow(hwnd: *mut std::ffi::c_void) -> i32;
    fn GetMessageW(lp_msg: *mut Msg, hwnd: *mut std::ffi::c_void, min: u32, max: u32) -> i32;
    fn TranslateMessage(lp_msg: *const Msg) -> i32;
    fn DispatchMessageW(lp_msg: *const Msg) -> isize;
    fn PostQuitMessage(exit_code: i32) -> ();
    fn PostMessageW(hwnd: *mut std::ffi::c_void, msg: u32, wparam: usize, lparam: isize) -> i32;
    fn CreatePopupMenu() -> *mut std::ffi::c_void;
    fn AppendMenuW(hmenu: *mut std::ffi::c_void, flags: u32, id_new: usize, lp_new: *const u16) -> i32;
    fn DestroyMenu(hmenu: *mut std::ffi::c_void) -> i32;
    fn GetCursorPos(lp_point: *mut Point) -> i32;
    fn SetForegroundWindow(hwnd: *mut std::ffi::c_void) -> i32;
    fn TrackPopupMenuEx(
        hmenu: *mut std::ffi::c_void,
        flags: u32,
        x: i32,
        y: i32,
        hwnd: *mut std::ffi::c_void,
        lptpm: *mut std::ffi::c_void,
    ) -> i32;
    fn LoadIconW(h_instance: *mut std::ffi::c_void, lp_icon_name: *const u16) -> *mut std::ffi::c_void;
    fn GetModuleHandleW(lp_module_name: *const u16) -> *mut std::ffi::c_void;
}

// Global action sender accessible from Win32 WindowProc
static GLOBAL_TRAY_TX: OnceLock<Sender<TrayAction>> = OnceLock::new();

unsafe extern "system" fn tray_wnd_proc(
    hwnd: *mut std::ffi::c_void,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    #[cfg(windows)]
    if msg == 0x0010 { // WM_CLOSE
        PostQuitMessage(0);
        return 0;
    }

    #[cfg(windows)]
    if msg == WM_TRAY_CALLBACK {
        match lparam {
            WM_LBUTTONUP | WM_LBUTTONDBLCLK => {
                if let Some(tx) = GLOBAL_TRAY_TX.get() {
                    let _ = tx.send(TrayAction::ToggleWindow);
                }
            }
            WM_RBUTTONUP => {
                let hmenu = CreatePopupMenu();
                if !hmenu.is_null() {
                    let title = to_wide_null("AURI - Minimalist Music Player");
                    let sep = to_wide_null("-");
                    let play = to_wide_null("Play / Pause");
                    let next = to_wide_null("Next Track");
                    let prev = to_wide_null("Previous Track");
                    let m_normal = to_wide_null("Mode: Normal Desktop");
                    let m_mini = to_wide_null("Mode: Mini Floating Player");
                    let m_gaming = to_wide_null("Mode: Gaming HUD");
                    let m_headless = to_wide_null("Mode: Pure Headless (0.0% GPU)");
                    let toggle = to_wide_null("Show / Hide AURI");
                    let exit = to_wide_null("Exit AURI");

                    AppendMenuW(hmenu, MF_STRING | MF_DISABLED | MF_GRAYED, 0, title.as_ptr());
                    AppendMenuW(hmenu, MF_SEPARATOR, 0, sep.as_ptr());
                    AppendMenuW(hmenu, MF_STRING, ID_MENU_PLAY, play.as_ptr());
                    AppendMenuW(hmenu, MF_STRING, ID_MENU_NEXT, next.as_ptr());
                    AppendMenuW(hmenu, MF_STRING, ID_MENU_PREV, prev.as_ptr());
                    AppendMenuW(hmenu, MF_SEPARATOR, 0, sep.as_ptr());
                    AppendMenuW(hmenu, MF_STRING, ID_MODE_NORMAL, m_normal.as_ptr());
                    AppendMenuW(hmenu, MF_STRING, ID_MODE_MINI, m_mini.as_ptr());
                    AppendMenuW(hmenu, MF_STRING, ID_MODE_GAMING, m_gaming.as_ptr());
                    AppendMenuW(hmenu, MF_STRING, ID_MODE_HEADLESS, m_headless.as_ptr());
                    AppendMenuW(hmenu, MF_SEPARATOR, 0, sep.as_ptr());
                    AppendMenuW(hmenu, MF_STRING, ID_MENU_TOGGLE_WIN, toggle.as_ptr());
                    AppendMenuW(hmenu, MF_STRING, ID_MENU_EXIT, exit.as_ptr());

                    let mut pt: Point = std::mem::zeroed();
                    GetCursorPos(&mut pt);
                    SetForegroundWindow(hwnd);

                    let cmd = TrackPopupMenuEx(
                        hmenu,
                        TPM_RIGHTBUTTON | TPM_RETURNCMD,
                        pt.x,
                        pt.y,
                        hwnd,
                        std::ptr::null_mut(),
                    ) as usize;

                    DestroyMenu(hmenu);

                    if let Some(tx) = GLOBAL_TRAY_TX.get() {
                        match cmd {
                            ID_MENU_PLAY => { let _ = tx.send(TrayAction::TogglePlay); }
                            ID_MENU_NEXT => { let _ = tx.send(TrayAction::NextTrack); }
                            ID_MENU_PREV => { let _ = tx.send(TrayAction::PrevTrack); }
                            ID_MODE_NORMAL => { let _ = tx.send(TrayAction::SetMode(0)); }
                            ID_MODE_MINI => { let _ = tx.send(TrayAction::SetMode(1)); }
                            ID_MODE_GAMING => { let _ = tx.send(TrayAction::SetMode(2)); }
                            ID_MODE_HEADLESS => { let _ = tx.send(TrayAction::SetMode(3)); }
                            ID_MENU_TOGGLE_WIN => { let _ = tx.send(TrayAction::ToggleWindow); }
                            ID_MENU_EXIT => { let _ = tx.send(TrayAction::Exit); }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
        return 0;
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn to_wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub struct SystemTray {
    action_rx: Receiver<TrayAction>,
    hwnd: Arc<AtomicIsize>,
    _thread_handle: Option<JoinHandle<()>>,
}

unsafe impl Send for SystemTray {}
unsafe impl Sync for SystemTray {}

impl SystemTray {
    pub fn new() -> Self {
        let (action_tx, action_rx) = channel::<TrayAction>();
        let hwnd_arc = Arc::new(AtomicIsize::new(0));
        let hwnd_clone = hwnd_arc.clone();

        let handle = thread::Builder::new()
            .name("auri-tray-thread".to_string())
            .spawn(move || {
                Self::run_tray_loop(action_tx, hwnd_clone);
            })
            .ok();

        Self {
            action_rx,
            hwnd: hwnd_arc,
            _thread_handle: handle,
        }
    }

    pub fn try_recv(&self) -> Option<TrayAction> {
        self.action_rx.try_recv().ok()
    }

    pub fn update_tooltip(&self, title: &str, artist: &str) {
        #[cfg(windows)]
        unsafe {
            let raw = self.hwnd.load(Ordering::SeqCst);
            if raw != 0 {
                let hwnd = raw as *mut std::ffi::c_void;
                let tip_text = format!("AURI - {}\n{}", title, artist);
                let mut nid: NotifyIconDataW = std::mem::zeroed();
                nid.cb_size = std::mem::size_of::<NotifyIconDataW>() as u32;
                nid.hwnd = hwnd;
                nid.u_id = 1;
                nid.u_flags = NIF_TIP;

                let wide_tip = to_wide_null(&tip_text);
                let copy_len = wide_tip.len().min(127);
                nid.sz_tip[..copy_len].copy_from_slice(&wide_tip[..copy_len]);

                Shell_NotifyIconW(NIM_MODIFY, &nid);
            }
        }
    }

    fn run_tray_loop(tx: Sender<TrayAction>, hwnd_holder: Arc<AtomicIsize>) {
        #[cfg(windows)]
        unsafe {
            let _ = GLOBAL_TRAY_TX.set(tx);

            let class_name = to_wide_null("AURI_Tray_WindowClass");
            let win_name = to_wide_null("AURI_Tray_MessageWindow");
            let h_inst = GetModuleHandleW(std::ptr::null());

            let mut wnd_class: WndClassExW = std::mem::zeroed();
            wnd_class.cb_size = std::mem::size_of::<WndClassExW>() as u32;
            wnd_class.lpfn_wnd_proc = Some(tray_wnd_proc);
            wnd_class.h_instance = h_inst;
            wnd_class.lpsz_class_name = class_name.as_ptr();

            RegisterClassExW(&wnd_class);

            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                win_name.as_ptr(),
                0,
                0, 0, 0, 0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                h_inst,
                std::ptr::null_mut(),
            );

            if hwnd.is_null() {
                eprintln!("[AURI Tray] Failed to create tray message window");
                return;
            }

            hwnd_holder.store(hwnd as isize, Ordering::SeqCst);

            // Load AURI application icon (resource ID 1 from winres) or default
            let mut h_icon = LoadIconW(h_inst, 1 as *const u16);
            if h_icon.is_null() {
                h_icon = LoadIconW(std::ptr::null_mut(), 32512 as *const u16); // IDI_APPLICATION
            }

            let tip_text = to_wide_null("AURI - Minimalist Music Player");
            let mut nid: NotifyIconDataW = std::mem::zeroed();
            nid.cb_size = std::mem::size_of::<NotifyIconDataW>() as u32;
            nid.hwnd = hwnd;
            nid.u_id = 1;
            nid.u_flags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
            nid.u_callback_message = WM_TRAY_CALLBACK;
            nid.h_icon = h_icon;

            let copy_len = tip_text.len().min(127);
            nid.sz_tip[..copy_len].copy_from_slice(&tip_text[..copy_len]);

            if Shell_NotifyIconW(NIM_ADD, &nid) != 0 {
                println!("[AURI Tray] System tray icon registered successfully");
            } else {
                eprintln!("[AURI Tray] Note: Shell_NotifyIconW returned 0");
            }

            let mut msg: Msg = std::mem::zeroed();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            // Clean up icon
            Shell_NotifyIconW(NIM_DELETE, &nid);
            DestroyWindow(hwnd);
        }
    }
}

impl Drop for SystemTray {
    fn drop(&mut self) {
        #[cfg(windows)]
        unsafe {
            let raw = self.hwnd.load(Ordering::SeqCst);
            if raw != 0 {
                let hwnd = raw as *mut std::ffi::c_void;
                let mut nid: NotifyIconDataW = std::mem::zeroed();
                nid.cb_size = std::mem::size_of::<NotifyIconDataW>() as u32;
                nid.hwnd = hwnd;
                nid.u_id = 1;
                Shell_NotifyIconW(NIM_DELETE, &nid);
                PostMessageW(hwnd, 0x0010, 0, 0); // WM_CLOSE
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_initialization_and_drop() {
        let tray = SystemTray::new();
        tray.update_tooltip("Test Track", "Test Artist");
        drop(tray);
    }
}
