// Project AURI: Minimalist, Ultra-Lightweight, Gaming-First Music Player
// Milestone v0.1.0: Multi-Mode UI Prototype & Settings Switcher

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    println!("========================================================");
    println!("  AURI Desktop Music Player - v0.1.0 Prototype");
    println!("  Ultra-Lightweight • Gaming-First • Native Windows 11");
    println!("========================================================");

    let ui = AppWindow::new()?;

    // Register callbacks
    let ui_handle = ui.as_weak();
    ui.on_toggle_play(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let playing = ui.get_is_playing();
            println!("[AURI Playback] State toggled -> playing = {}", playing);
        }
    });

    let ui_handle = ui.as_weak();
    ui.on_prev_track(move || {
        if let Some(ui) = ui_handle.upgrade() {
            println!("[AURI Queue] Previous track triggered");
            ui.set_track_title("Tamagotchi".into());
            ui.set_track_artist("Taco Hemingway, Quebonafide".into());
            ui.set_track_duration("3:22".into());
            ui.set_track_elapsed("0:00".into());
            ui.set_track_progress(0.0);
        }
    });

    let ui_handle = ui.as_weak();
    ui.on_next_track(move || {
        if let Some(ui) = ui_handle.upgrade() {
            println!("[AURI Queue] Next track triggered");
            ui.set_track_title("Midnight Drive".into());
            ui.set_track_artist("The Cassettes".into());
            ui.set_track_duration("3:34".into());
            ui.set_track_elapsed("0:00".into());
            ui.set_track_progress(0.0);
        }
    });

    ui.on_seek(move |progress| {
        println!("[AURI Audio] Seek: {:.1}%", progress * 100.0);
    });

    ui.on_volume_changed(move |vol| {
        println!("[AURI Audio] Volume: {:.0}%", vol * 100.0);
    });

    ui.on_toggle_favorite(move || {
        println!("[AURI Library] Favorite state toggled");
    });

    ui.on_toggle_shuffle(move || {
        println!("[AURI Queue] Shuffle toggled");
    });

    ui.on_toggle_repeat(move || {
        println!("[AURI Queue] Repeat toggled");
    });

    ui.on_switch_mode(move |mode| {
        let name = match mode {
            0 => "Normal Mode (Full Desktop Player)",
            1 => "Mini Mode (Floating Controller)",
            2 => "Gaming Mode (Companion HUD)",
            3 => "Pure Headless (0.0% GPU / Suspended UI)",
            _ => "Unknown",
        };
        println!("[AURI Mode Switcher] Active Mode changed to: {}", name);
    });

    ui.on_set_normal_layout(move |layout| {
        let name = if layout == 0 {
            "Variant A1: Right-Panel Embedded Player Card (Reference 1 Style)"
        } else {
            "Variant A2: Classic Persistent Docked Bottom Bar"
        };
        println!("[AURI Settings] Normal Mode Layout set to: {}", name);
    });

    ui.on_set_mini_layout(move |layout| {
        let name = if layout == 0 {
            "Variant B1: Compact Card with Album Art (Reference 2A)"
        } else {
            "Variant B2: Ultra-thin Micro-Pill (Reference 2B)"
        };
        println!("[AURI Settings] Mini Mode Layout set to: {}", name);
    });

    ui.on_navigate(move |dest| {
        println!("[AURI Navigation] Navigated to section: {}", dest);
    });

    ui.on_enter_headless(move || {
        println!("[AURI Gaming Mode] Pure Headless Mode engaged: Frame rendering suspended (0.0% GPU)");
    });

    println!("[AURI Startup] Launching interface...");
    ui.run()
}
