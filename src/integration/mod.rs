// Windows Deep Integration Subsystem
// Milestone v0.4.0: System Tray, Global Hotkeys, Hardware Media Keys, Decoupled Window Lifecycle.

pub mod hotkeys;
pub mod tray;

pub use hotkeys::{GlobalHotkeys, HotkeyAction};
pub use tray::{SystemTray, TrayAction};

pub struct WindowsIntegration {
    hotkeys: GlobalHotkeys,
    tray: SystemTray,
}

impl WindowsIntegration {
    pub fn new() -> Self {
        println!("[AURI Integration] Initializing Windows System Integration...");
        let hotkeys = GlobalHotkeys::new();
        let tray = SystemTray::new();

        Self { hotkeys, tray }
    }

    pub fn poll_hotkey(&self) -> Option<HotkeyAction> {
        self.hotkeys.try_recv()
    }

    pub fn poll_tray(&self) -> Option<TrayAction> {
        self.tray.try_recv()
    }

    pub fn update_tray_tooltip(&self, title: &str, artist: &str) {
        self.tray.update_tooltip(title, artist);
    }
}

