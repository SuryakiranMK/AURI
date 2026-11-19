//! System tray integration (Windows): a tray icon with a Show/Quit menu,
//! close-to-tray support and song-change balloon notifications. The icon
//! lives on a dedicated thread with a hidden message-only window, so the UI
//! thread never pumps Win32 messages.

#[cfg(windows)]
pub use implementation::*;

#[cfg(not(windows))]
pub use stubs::*;

#[cfg(windows)]
mod implementation {
    use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
    use std::sync::{Arc, Mutex, OnceLock};
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::Shell::{
        ExtractIconExW, Shell_NotifyIconW, NOTIFYICONDATAW, NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_TIP,
        NIIF_INFO, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW,
        FindWindowW, GetCursorPos, GetMessageW, MF_STRING, MSG, PostMessageW, RegisterClassW,
        SetForegroundWindow, ShowWindow, TrackPopupMenu, TranslateMessage, HICON, WNDCLASSW,
        SW_RESTORE, SW_SHOWMAXIMIZED, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_NONOTIFY, TPM_RETURNCMD,
        TPM_RIGHTBUTTON, WM_APP, WM_COMMAND, WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_NULL, WM_RBUTTONUP,
        WS_POPUP,
    };

    const WM_TRAY: u32 = WM_APP + 1;
    const WM_APPLY: u32 = WM_APP + 2;
    const MENU_RESTORE_ID: usize = 1;
    const MENU_QUIT_ID: usize = 2;

    /// What the tray can ask the app to do.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum TrayAction {
        Restore,
        Quit,
    }

    static CALLBACK: OnceLock<Arc<dyn Fn(TrayAction) + Send + Sync>> = OnceLock::new();
    static VISIBLE: AtomicBool = AtomicBool::new(false);
    static ADDED: AtomicBool = AtomicBool::new(false);
    static FORCE_QUIT: AtomicBool = AtomicBool::new(false);
    static TRAY_WINDOW: AtomicIsize = AtomicIsize::new(0);
    static APP_ICON: AtomicIsize = AtomicIsize::new(0);
    static BALLOON: OnceLock<Mutex<Option<(String, String)>>> = OnceLock::new();

    pub fn is_force_quit() -> bool {
        FORCE_QUIT.load(Ordering::SeqCst)
    }

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(Some(0)).collect()
    }

    /// Spawns the tray thread. Call once at startup; the callback receives
    /// menu/interaction requests from the tray.
    pub fn start(callback: Arc<dyn Fn(TrayAction) + Send + Sync>) {
        let _ = CALLBACK.set(callback);
        std::thread::spawn(tray_thread);
    }

    /// Shows or hides the tray icon (used by close-to-tray and notifications).
    pub fn set_visible(visible: bool) {
        VISIBLE.store(visible, Ordering::Relaxed);
        post_apply();
    }

    /// Queues a song-change balloon (title, artist). Shown only while the
    /// tray icon is visible.
    pub fn notify_song(title: &str, subtitle: &str) {
        let cell = BALLOON.get_or_init(Mutex::default);
        if let Ok(mut pending) = cell.lock() {
            *pending = Some((title.to_owned(), subtitle.to_owned()));
        }
        post_apply();
    }

    /// Removes the tray icon (call on app exit).
    pub fn shutdown() {
        VISIBLE.store(false, Ordering::Relaxed);
        post_apply();
    }

    fn post_apply() {
        let window = TRAY_WINDOW.load(Ordering::Relaxed);
        if window != 0 {
            unsafe { PostMessageW(window as HWND, WM_APPLY, 0, 0) };
        }
    }

    fn tray_thread() {
        unsafe {
            let class_name = wide("AURI_TRAY");
            let instance = GetModuleHandleW(std::ptr::null());
            let class = WNDCLASSW {
                lpfnWndProc: Some(tray_wnd_proc),
                hInstance: instance,
                lpszClassName: class_name.as_ptr(),
                ..std::mem::zeroed()
            };
            RegisterClassW(&class);
            let window = CreateWindowExW(
                0,
                class_name.as_ptr(),
                class_name.as_ptr(),
                WS_POPUP,
                0,
                0,
                0,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance,
                std::ptr::null(),
            );
            TRAY_WINDOW.store(window as isize, Ordering::Relaxed);
            // Pick up any visibility requested before the window existed.
            apply();
            let mut message: MSG = std::mem::zeroed();
            while GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0 {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }

    unsafe extern "system" fn tray_wnd_proc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            match message {
                WM_TRAY => {
                    match lparam as u32 & 0xFFFF {
                        WM_LBUTTONUP | WM_LBUTTONDBLCLK => fire(TrayAction::Restore),
                        WM_RBUTTONUP => open_menu(window),
                        _ => {}
                    }
                    0
                }
                WM_APPLY => {
                    apply();
                    0
                }
                WM_COMMAND => {
                    match wparam & 0xFFFF {
                        MENU_RESTORE_ID => fire(TrayAction::Restore),
                        MENU_QUIT_ID => fire(TrayAction::Quit),
                        _ => {}
                    }
                    0
                }
                _ => DefWindowProcW(window, message, wparam, lparam),
            }
        }
    }

    fn fire(action: TrayAction) {
        if action == TrayAction::Restore {
            restore_main_window();
        } else if action == TrayAction::Quit {
            FORCE_QUIT.store(true, Ordering::SeqCst);
            shutdown();
            if let Some(callback) = CALLBACK.get() {
                callback(TrayAction::Quit);
            }
            unsafe {
                let title = wide(crate::APP_NAME);
                let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
                if !hwnd.is_null() {
                    windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                        hwnd,
                        windows_sys::Win32::UI::WindowsAndMessaging::WM_CLOSE,
                        0,
                        0,
                    );
                }
            }
            // Watchdog: If the window event loop is paused or hidden in tray and
            // does not exit promptly, ensure the process exits cleanly.
            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(350));
                std::process::exit(0);
            });
            return;
        }
        if let Some(callback) = CALLBACK.get() {
            callback(action);
        }
    }

    /// Directly un-hides and raises the main AURI window at the OS level.
    pub fn restore_main_window() {
        unsafe {
            let title = wide(crate::APP_NAME);
            let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
            if !hwnd.is_null() {
                ShowWindow(hwnd, SW_RESTORE);
                ShowWindow(hwnd, SW_SHOWMAXIMIZED);
                SetForegroundWindow(hwnd);
            }
        }
    }

    unsafe fn open_menu(window: HWND) {
        unsafe {
            let menu = CreatePopupMenu();
            let restore = wide("Show AURI");
            let quit = wide("Quit AURI");
            AppendMenuW(menu, MF_STRING, MENU_RESTORE_ID, restore.as_ptr());
            AppendMenuW(menu, MF_STRING, MENU_QUIT_ID, quit.as_ptr());
            let mut cursor = POINT { x: 0, y: 0 };
            GetCursorPos(&mut cursor);
            // Per MS KB131845: must set foreground window before TrackPopupMenu
            SetForegroundWindow(window);
            let cmd = TrackPopupMenu(
                menu,
                TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON,
                cursor.x,
                cursor.y,
                0,
                window,
                std::ptr::null(),
            ) as usize;
            // Per MS KB131845: must post WM_NULL to force task switch / release Explorer capture
            PostMessageW(window, WM_NULL, 0, 0);
            DestroyMenu(menu);
            match cmd {
                MENU_RESTORE_ID => fire(TrayAction::Restore),
                MENU_QUIT_ID => fire(TrayAction::Quit),
                _ => {}
            }
        }
    }

    unsafe fn app_icon() -> HICON {
        unsafe {
            let cached = APP_ICON.load(Ordering::Relaxed);
            if cached != 0 {
                return cached as HICON;
            }
            let mut icon: HICON = std::ptr::null_mut();
            if let Ok(exe) = std::env::current_exe() {
                let path = wide(&exe.to_string_lossy());
                ExtractIconExW(path.as_ptr(), 0, &mut icon, std::ptr::null_mut(), 1);
            }
            APP_ICON.store(icon as isize, Ordering::Relaxed);
            icon
        }
    }

    unsafe fn icon_data() -> NOTIFYICONDATAW {
        unsafe {
            let window = TRAY_WINDOW.load(Ordering::Relaxed) as HWND;
            let mut data: NOTIFYICONDATAW = std::mem::zeroed();
            data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            data.hWnd = window;
            data.uID = 1;
            data.uCallbackMessage = WM_TRAY;
            data
        }
    }

    /// Applies visibility changes and flushes any queued balloon.
    unsafe fn apply() {
        unsafe {
            let visible = VISIBLE.load(Ordering::Relaxed);
            let added = ADDED.load(Ordering::Relaxed);
            if visible && !added {
                let mut data = icon_data();
                data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
                data.hIcon = app_icon();
                let tip = wide("AURI");
                let tip = &tip[..tip.len().min(data.szTip.len())];
                data.szTip[..tip.len()].copy_from_slice(tip);
                if Shell_NotifyIconW(NIM_ADD, &data) != 0 {
                    ADDED.store(true, Ordering::Relaxed);
                }
            } else if !visible && added {
                let data = icon_data();
                Shell_NotifyIconW(NIM_DELETE, &data);
                ADDED.store(false, Ordering::Relaxed);
            }
            if !ADDED.load(Ordering::Relaxed) {
                return;
            }
            let Some(cell) = BALLOON.get() else { return };
            let Ok(mut pending) = cell.lock() else { return };
            if let Some((title, subtitle)) = pending.take() {
                let mut data = icon_data();
                data.uFlags = NIF_INFO;
                data.dwInfoFlags = NIIF_INFO;
                let headline: Vec<u16> = format!("AURI — {title}").encode_utf16().take(63).collect();
                data.szInfoTitle[..headline.len()].copy_from_slice(&headline);
                let body: Vec<u16> = format!("{title}\n{subtitle}").encode_utf16().take(255).collect();
                data.szInfo[..body.len()].copy_from_slice(&body);
                Shell_NotifyIconW(NIM_MODIFY, &data);
            }
        }
    }
}

#[cfg(not(windows))]
mod stubs {
    /// What the tray can ask the app to do (stub on non-Windows platforms).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum TrayAction {
        Restore,
        Quit,
    }

    pub fn start(_callback: std::sync::Arc<dyn Fn(TrayAction) + Send + Sync>) {}
    pub fn set_visible(_visible: bool) {}
    pub fn notify_song(_title: &str, _subtitle: &str) {}
    pub fn shutdown() {}
    pub fn restore_main_window() {}
    pub fn is_force_quit() -> bool { false }
}
