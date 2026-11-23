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
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::Shell::{
        ExtractIconExW, Shell_NotifyIconW, NOTIFYICONDATAW, NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_TIP,
        NIIF_INFO, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW,
        FindWindowW, GetCursorPos, GetMessageW, GetWindowLongPtrW, GetWindowRect, IsZoomed, MF_DISABLED,
        MF_GRAYED, MF_SEPARATOR, MF_STRING, MSG, PostMessageW, RegisterClassW, SetForegroundWindow,
        SetWindowLongPtrW, SetWindowPos, ShowWindow, TrackPopupMenu, TranslateMessage, GWL_EXSTYLE,
        HICON, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SW_RESTORE, SW_SHOWMAXIMIZED,
        TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_NONOTIFY, TPM_RETURNCMD, TPM_RIGHTBUTTON, WM_APP, WM_COMMAND,
        WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_NULL, WM_RBUTTONUP, WNDCLASSW, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
        WS_POPUP,
    };

    const WM_TRAY: u32 = WM_APP + 1;
    const WM_APPLY: u32 = WM_APP + 2;
    const MENU_RESTORE_ID: usize = 1;
    const MENU_QUIT_ID: usize = 2;
    const MENU_PLAY_PAUSE_ID: usize = 3;
    const MENU_NEXT_ID: usize = 4;
    const MENU_PREV_ID: usize = 5;

    /// What the tray can ask the app to do.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum TrayAction {
        Restore,
        Quit,
        PlayPause,
        Next,
        Prev,
    }

    struct SavedPlacement {
        rect: (i32, i32, i32, i32),
        was_zoomed: bool,
    }

    static CALLBACK: OnceLock<Arc<dyn Fn(TrayAction) + Send + Sync>> = OnceLock::new();
    static VISIBLE: AtomicBool = AtomicBool::new(false);
    static ADDED: AtomicBool = AtomicBool::new(false);
    static FORCE_QUIT: AtomicBool = AtomicBool::new(false);
    static IS_PLAYING: AtomicBool = AtomicBool::new(false);
    static TRAY_WINDOW: AtomicIsize = AtomicIsize::new(0);
    static MAIN_HWND: AtomicIsize = AtomicIsize::new(0);
    static APP_ICON: AtomicIsize = AtomicIsize::new(0);
    static BALLOON: OnceLock<Mutex<Option<(String, String)>>> = OnceLock::new();
    static CURRENT_SONG: OnceLock<Mutex<Option<(String, String)>>> = OnceLock::new();
    static SAVED_PLACEMENT: OnceLock<Mutex<Option<SavedPlacement>>> = OnceLock::new();

    pub fn is_force_quit() -> bool {
        FORCE_QUIT.load(Ordering::SeqCst)
    }

    pub fn set_playing(playing: bool) {
        IS_PLAYING.store(playing, Ordering::Relaxed);
    }

    #[allow(dead_code)]
    pub fn is_playing() -> bool {
        IS_PLAYING.load(Ordering::Relaxed)
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
        let song_cell = CURRENT_SONG.get_or_init(Mutex::default);
        if let Ok(mut cur) = song_cell.lock() {
            *cur = Some((title.to_owned(), subtitle.to_owned()));
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
                        MENU_PLAY_PAUSE_ID => fire(TrayAction::PlayPause),
                        MENU_NEXT_ID => fire(TrayAction::Next),
                        MENU_PREV_ID => fire(TrayAction::Prev),
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

    pub fn main_hwnd() -> HWND {
        let cached = MAIN_HWND.load(Ordering::Relaxed);
        if cached != 0 {
            return cached as HWND;
        }
        unsafe {
            let title = wide(crate::APP_NAME);
            let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
            if !hwnd.is_null() {
                MAIN_HWND.store(hwnd as isize, Ordering::Relaxed);
                return hwnd;
            }
            std::ptr::null_mut()
        }
    }

    /// Hides the main AURI window off-screen and removes it from the taskbar,
    /// but keeps WS_VISIBLE active so the Win32 message pump (and eframe/winit
    /// event loop) continues to run background ticks at 0% CPU.
    pub fn hide_main_window_to_tray() {
        let hwnd = main_hwnd();
        if hwnd.is_null() {
            return;
        }
        unsafe {
            let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            GetWindowRect(hwnd, &mut rect);
            let was_zoomed = IsZoomed(hwnd) != 0;
            if rect.left > -10000 && rect.top > -10000 {
                let cell = SAVED_PLACEMENT.get_or_init(Mutex::default);
                if let Ok(mut store) = cell.lock() {
                    *store = Some(SavedPlacement {
                        rect: (rect.left, rect.top, (rect.right - rect.left).max(200), (rect.bottom - rect.top).max(200)),
                        was_zoomed,
                    });
                }
            }

            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
            let new_ex = (ex_style & !WS_EX_APPWINDOW) | WS_EX_TOOLWINDOW;
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex as isize);

            SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                -32000,
                -32000,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOSIZE | SWP_FRAMECHANGED,
            );
            set_visible(true);
        }
    }

    /// Directly un-hides and raises the main AURI window at the OS level.
    pub fn restore_main_window() {
        let hwnd = main_hwnd();
        if hwnd.is_null() {
            return;
        }
        unsafe {
            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
            let new_ex = (ex_style & !WS_EX_TOOLWINDOW) | WS_EX_APPWINDOW;
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex as isize);

            let cell = SAVED_PLACEMENT.get_or_init(Mutex::default);
            let placement = cell.lock().ok().and_then(|mut s| s.take());

            if let Some(p) = placement {
                if p.was_zoomed {
                    ShowWindow(hwnd, SW_RESTORE);
                    ShowWindow(hwnd, SW_SHOWMAXIMIZED);
                } else {
                    SetWindowPos(
                        hwnd,
                        std::ptr::null_mut(),
                        p.rect.0,
                        p.rect.1,
                        p.rect.2,
                        p.rect.3,
                        SWP_NOZORDER | SWP_FRAMECHANGED,
                    );
                    ShowWindow(hwnd, SW_RESTORE);
                }
            } else {
                ShowWindow(hwnd, SW_RESTORE);
                ShowWindow(hwnd, SW_SHOWMAXIMIZED);
            }
            SetForegroundWindow(hwnd);
        }
    }

    #[repr(i32)]
    #[allow(dead_code)]
    enum PreferredAppMode {
        Default = 0,
        AllowDark = 1,
        ForceDark = 2,
        ForceLight = 3,
        Max = 4,
    }

    unsafe fn enable_dark_menu() {
        unsafe {
            use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
            let uxtheme = LoadLibraryA(b"uxtheme.dll\0".as_ptr());
            if !uxtheme.is_null() {
                type FnSetPreferredAppMode = unsafe extern "system" fn(PreferredAppMode) -> PreferredAppMode;
                type FnFlushMenuThemes = unsafe extern "system" fn();

                // Ordinal 135: SetPreferredAppMode (Windows 10 1903+)
                let set_mode: Option<FnSetPreferredAppMode> =
                    std::mem::transmute(GetProcAddress(uxtheme, 135 as _));
                if let Some(f) = set_mode {
                    f(PreferredAppMode::ForceDark);
                }

                // Ordinal 136: FlushMenuThemes
                let flush: Option<FnFlushMenuThemes> =
                    std::mem::transmute(GetProcAddress(uxtheme, 136 as _));
                if let Some(f) = flush {
                    f();
                }
            }
        }
    }

    unsafe fn open_menu(window: HWND) {
        unsafe {
            enable_dark_menu();
            let menu = CreatePopupMenu();

            // Optional Live Now-Playing Header (dimmed non-clickable status row)
            if let Some(song_cell) = CURRENT_SONG.get() {
                if let Ok(guard) = song_cell.lock() {
                    if let Some((title, artist)) = guard.as_ref() {
                        let short_title = if title.chars().count() > 28 {
                            format!("{}...", title.chars().take(26).collect::<String>())
                        } else {
                            title.clone()
                        };
                        let header_text = if artist.is_empty() {
                            format!("Now Playing: {short_title}")
                        } else {
                            let short_artist = if artist.chars().count() > 18 {
                                format!("{}...", artist.chars().take(16).collect::<String>())
                            } else {
                                artist.clone()
                            };
                            format!("{short_title} • {short_artist}")
                        };
                        let header = wide(&header_text);
                        AppendMenuW(menu, MF_STRING | MF_DISABLED | MF_GRAYED, 0, header.as_ptr());
                        AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
                    }
                }
            }

            let is_playing = IS_PLAYING.load(Ordering::Relaxed);
            let play_pause_label = if is_playing { "Pause" } else { "Play" };
            let play_pause = wide(play_pause_label);
            let next = wide("Next Track");
            let prev = wide("Previous Track");
            let restore = wide("Open AURI");
            let quit = wide("Quit AURI");

            AppendMenuW(menu, MF_STRING, MENU_PLAY_PAUSE_ID, play_pause.as_ptr());
            AppendMenuW(menu, MF_STRING, MENU_NEXT_ID, next.as_ptr());
            AppendMenuW(menu, MF_STRING, MENU_PREV_ID, prev.as_ptr());
            AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
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
                MENU_PLAY_PAUSE_ID => fire(TrayAction::PlayPause),
                MENU_NEXT_ID => fire(TrayAction::Next),
                MENU_PREV_ID => fire(TrayAction::Prev),
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
        PlayPause,
        Next,
        Prev,
    }

    pub fn start(_callback: std::sync::Arc<dyn Fn(TrayAction) + Send + Sync>) {}
    pub fn set_visible(_visible: bool) {}
    pub fn notify_song(_title: &str, _subtitle: &str) {}
    pub fn shutdown() {}
    pub fn hide_main_window_to_tray() {}
    pub fn restore_main_window() {}
    pub fn set_playing(_playing: bool) {}
    pub fn is_playing() -> bool { false }
    pub fn is_force_quit() -> bool { false }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_action_variants() {
        assert_eq!(TrayAction::Restore, TrayAction::Restore);
        assert_ne!(TrayAction::PlayPause, TrayAction::Next);
        assert_ne!(TrayAction::Prev, TrayAction::Quit);
    }

    #[test]
    fn test_tray_playback_state() {
        set_playing(true);
        #[cfg(windows)]
        assert!(is_playing());
        set_playing(false);
        #[cfg(windows)]
        assert!(!is_playing());
    }

    #[test]
    fn test_notify_song_no_panic() {
        notify_song("Starboy", "The Weeknd, Daft Punk");
        notify_song("A very long title that exceeds the limits of standard notification buffers to ensure truncation handles wide characters correctly", "Artist");
    }
}
