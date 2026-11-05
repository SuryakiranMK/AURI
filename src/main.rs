// Project AURI: Minimalist, Ultra-Lightweight, Gaming-First Music Player
// Official Release v1.0.0

pub mod audio;
pub mod gaming;
pub mod integration;
pub mod library;
pub mod provider;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use audio::{AudioEngine, PlaybackState};
use gaming::{get_working_set_size_bytes, trim_working_set, GameDetector, GameEvent};
use integration::{HotkeyAction, TrayAction, WindowsIntegration};
use library::{LibraryManager, RepeatMode};
use provider::innertube::InnertubeProvider;
use provider::MusicProvider;
use slint::Timer;

slint::include_modules!();

fn format_duration(seconds: f64) -> String {
    let total_secs = seconds.max(0.0) as u64;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{}:{:02}", mins, secs)
}

#[cfg(windows)]
fn enforce_window_geometry(mode: i32) {
    #[link(name = "user32")]
    extern "system" {
        fn FindWindowW(lp_class_name: *const u16, lp_window_name: *const u16) -> *mut std::ffi::c_void;
        fn SetWindowPos(
            hwnd: *mut std::ffi::c_void,
            hwnd_insert_after: *mut std::ffi::c_void,
            x: i32,
            y: i32,
            cx: i32,
            cy: i32,
            u_flags: u32,
        ) -> i32;
        fn ShowWindow(hwnd: *mut std::ffi::c_void, n_cmd_show: i32) -> i32;
    }

    const SW_RESTORE: i32 = 9;
    const HWND_TOPMOST: *mut std::ffi::c_void = -1isize as *mut std::ffi::c_void;
    const HWND_NOTOPMOST: *mut std::ffi::c_void = -2isize as *mut std::ffi::c_void;
    const SWP_NOMOVE: u32 = 0x0002;
    const SWP_SHOWWINDOW: u32 = 0x0040;

    unsafe {
        let title_wide: Vec<u16> = "AURI\0".encode_utf16().collect();
        let hwnd = FindWindowW(std::ptr::null(), title_wide.as_ptr());
        if !hwnd.is_null() {
            if mode == 1 {
                // Mini Mode: Restore from maximized if needed, pin top-most, fixed compact size
                ShowWindow(hwnd, SW_RESTORE);
                SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 326, 116, SWP_NOMOVE | SWP_SHOWWINDOW);
            } else if mode == 0 {
                // Normal Mode: Un-pin top-most, standard window size
                SetWindowPos(hwnd, HWND_NOTOPMOST, 0, 0, 1120, 720, SWP_NOMOVE | SWP_SHOWWINDOW);
            } else if mode == 2 {
                // Gaming HUD: Restore, pin top-most, compact HUD size
                ShowWindow(hwnd, SW_RESTORE);
                SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 420, 380, SWP_NOMOVE | SWP_SHOWWINDOW);
            }
        }
    }
}

#[cfg(not(windows))]
fn enforce_window_geometry(_mode: i32) {}

fn main() -> Result<(), slint::PlatformError> {
    println!("========================================================");
    println!("  AURI Desktop Music Player - v1.0.0 OFFICIAL RELEASE   ");
    println!("  Ultra-Lightweight • Gaming-First • WASAPI & Innertube ");
    println!("========================================================");
    println!("========================================================");

    // Initialize native audio engine
    let audio = match AudioEngine::new() {
        Ok(engine) => {
            let status = engine.get_status();
            println!("[AURI Audio Engine] Online via Windows WASAPI");
            println!("  Endpoint Device: {}", status.device_name);
            println!("  Hardware Mixing: {} Hz, {} channels", status.sample_rate, status.channels);
            Arc::new(engine)
        }
        Err(err) => {
            eprintln!("[AURI Audio Engine Error] Initialization failed: {}", err);
            panic!("Cannot run AURI without audio sink: {}", err);
        }
    };

    // Initialize local music library and playback queue
    let library = Arc::new(LibraryManager::new());
    let music_dirs = library::LibraryScanner::default_music_dirs();
    println!("[AURI Library] Scanning default music paths: {:?}", music_dirs);
    let discovered_count = library.scan_directories(&music_dirs);
    println!("[AURI Library] Discovered {} tracks in local library", discovered_count);

    // Initialize Windows System Integration (System Tray & Global Hotkeys)
    let integration = Arc::new(WindowsIntegration::new());
    let is_window_visible = Arc::new(AtomicBool::new(true));

    // Initialize YouTube Music Innertube Streaming Provider
    let provider = Arc::new(InnertubeProvider::new());
    println!("[AURI Streaming] Innertube Streaming Engine online (Tier 1 Provider)");

    // Initialize Anti-Cheat Safe Game Detector
    let (game_detector, game_rx) = GameDetector::start();
    let game_detector = Arc::new(game_detector);
    let game_rx_sync = Arc::new(std::sync::Mutex::new(game_rx));
    println!("[AURI Gaming] Anti-Cheat Safe Game Detector active (Win32 EVENT_SYSTEM_FOREGROUND)");

    let ui = AppWindow::new()?;
    let _ = trim_working_set(); // Purge GPU driver initialization overhead from working set

    // Sync initial state from discovered library
    let initial_status = audio.get_status();
    ui.set_volume(initial_status.volume);

    if let Some(ws) = get_working_set_size_bytes() {
        let mb = ws as f64 / (1024.0 * 1024.0);
        println!("[AURI Memory] Initial Working Set: {:.1} MB", mb);
        ui.set_ram_usage(format!("{:.1} MB", mb).into());
    }

    if let Some(first_track) = library.queue().lock().unwrap().current_track().cloned() {
        ui.set_track_title(first_track.title.as_str().into());
        ui.set_track_artist(first_track.artist.as_str().into());
        ui.set_track_duration(first_track.formatted_duration().into());
        integration.update_tray_tooltip(&first_track.title, &first_track.artist);
        println!("[AURI Queue] Initial track staged: {} - {}", first_track.artist, first_track.title);
    }

    // Helper closure to play a track and sync UI + Tray
    let play_track_helper = {
        let audio = audio.clone();
        let integration = integration.clone();
        let ui_handle = ui.as_weak();
        move |track: &library::TrackMetadata| {
            audio.play_file(&track.path);
            integration.update_tray_tooltip(&track.title, &track.artist);
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_track_title(track.title.as_str().into());
                ui.set_track_artist(track.artist.as_str().into());
                ui.set_track_duration(track.formatted_duration().as_str().into());
            }
        }
    };

    // Wire: Play / Pause toggle
    let audio_clone = audio.clone();
    let library_clone = library.clone();
    let play_helper = play_track_helper.clone();
    ui.on_toggle_play(move || {
        let status = audio_clone.get_status();
        if status.state == PlaybackState::Stopped || status.duration_secs <= 0.0 {
            let cur_opt = library_clone.queue().lock().unwrap().current_track().cloned();
            if let Some(track) = cur_opt {
                println!("[AURI Playback] Playing track from queue: {} - {}", track.artist, track.title);
                play_helper(&track);
            } else {
                println!("[AURI Audio] No track in queue: Playing AURI test chime (440 Hz A4)");
                audio_clone.play_test_tone(440.0, 15.0);
            }
        } else {
            audio_clone.toggle_playback();
        }
        let current_state = audio_clone.get_status().state;
        println!("[AURI Playback] Toggle -> State: {:?}", current_state);
    });

    // Wire: Previous track
    let audio_clone = audio.clone();
    let library_clone = library.clone();
    let play_helper = play_track_helper.clone();
    let ui_handle = ui.as_weak();
    ui.on_prev_track(move || {
        let prev_opt = library_clone.queue().lock().unwrap().previous().cloned();
        if let Some(track) = prev_opt {
            println!("[AURI Queue] Playing Previous: {} - {}", track.artist, track.title);
            play_helper(&track);
        } else {
            println!("[AURI Queue] Playing Previous Chime (330 Hz E4)");
            audio_clone.play_test_tone(329.63, 12.0);
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_track_title("AURI Chime (E4 - 330 Hz)".into());
                ui.set_track_artist("Audio Engine Test".into());
            }
        }
    });

    // Wire: Next track
    let audio_clone = audio.clone();
    let library_clone = library.clone();
    let play_helper = play_track_helper.clone();
    let ui_handle = ui.as_weak();
    ui.on_next_track(move || {
        let next_opt = library_clone.queue().lock().unwrap().next().cloned();
        if let Some(track) = next_opt {
            println!("[AURI Queue] Playing Next: {} - {}", track.artist, track.title);
            play_helper(&track);
        } else {
            println!("[AURI Queue] Playing Next Chime (523 Hz C5)");
            audio_clone.play_test_tone(523.25, 12.0);
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_track_title("AURI Chime (C5 - 523 Hz)".into());
                ui.set_track_artist("Audio Engine Test".into());
            }
        }
    });

    // Wire: Seeking
    let audio_clone = audio.clone();
    ui.on_seek(move |progress| {
        let status = audio_clone.get_status();
        let target_secs = (progress as f64) * status.duration_secs;
        audio_clone.seek(target_secs);
        println!("[AURI Audio] Seek -> {:.2}s ({:.1}%)", target_secs, progress * 100.0);
    });

    // Wire: Volume
    let audio_clone = audio.clone();
    ui.on_volume_changed(move |vol| {
        audio_clone.set_volume(vol);
        println!("[AURI Audio] Volume -> {:.0}%", vol * 100.0);
    });

    // Wire: Favorites & Queue toggles
    ui.on_toggle_favorite(move || {
        println!("[AURI Library] Favorite toggled");
    });

    let library_clone = library.clone();
    let ui_handle = ui.as_weak();
    ui.on_toggle_shuffle(move || {
        let is_shuffled = library_clone.queue().lock().unwrap().toggle_shuffle();
        println!("[AURI Queue] Shuffle toggled -> {}", is_shuffled);
        if let Some(ui) = ui_handle.upgrade() {
            ui.set_is_shuffle(is_shuffled);
        }
    });

    let library_clone = library.clone();
    let ui_handle = ui.as_weak();
    ui.on_toggle_repeat(move || {
        let repeat_mode = library_clone.queue().lock().unwrap().toggle_repeat();
        println!("[AURI Queue] Repeat mode toggled -> {:?}", repeat_mode);
        if let Some(ui) = ui_handle.upgrade() {
            ui.set_is_repeat(repeat_mode != RepeatMode::Off);
        }
    });

    // Wire: Modes & Settings
    ui.on_switch_mode(move |mode| {
        let name = match mode {
            0 => "Normal Mode",
            1 => "Mini Mode (Floating Controller)",
            2 => "Gaming Mode (Companion HUD)",
            3 => "Pure Headless (0.0% GPU)",
            _ => "Unknown",
        };
        println!("[AURI Mode] Switched to {}", name);
        enforce_window_geometry(mode);

        if mode >= 2 {
            if let Ok(report) = trim_working_set() {
                println!("[AURI Memory] Working set trimmed on mode switch: {:.1} MB", report.after_mb());
            }
        }
    });

    ui.on_set_normal_layout(move |layout| {
        let name = if layout == 0 { "Variant A1 (Ref 1 Style)" } else { "Variant A2 (Docked Bar)" };
        println!("[AURI Settings] Normal Layout set to: {}", name);
    });

    ui.on_set_mini_layout(move |layout| {
        let name = if layout == 0 { "Variant B1 (Card with Art)" } else { "Variant B2 (Micro-Pill)" };
        println!("[AURI Settings] Mini Layout set to: {}", name);
    });

    ui.on_navigate(move |dest| {
        println!("[AURI Navigation] Destination: {}", dest);
    });

    let detector_toggle = game_detector.clone();
    ui.on_toggle_auto_game(move || {
        let current = detector_toggle.is_enabled();
        detector_toggle.set_enabled(!current);
        println!("[AURI Gaming] Auto Game Detection toggled -> {}", !current);
    });

    let ui_ram_weak = ui.as_weak();
    ui.on_trim_ram(move || {
        match trim_working_set() {
            Ok(report) => {
                println!(
                    "[AURI Memory Trimmer] Working Set purged: {:.1} MB -> {:.1} MB (Freed: {:.1} MB)",
                    report.before_bytes as f64 / (1024.0 * 1024.0),
                    report.after_mb(),
                    report.freed_mb()
                );
                if let Some(ui) = ui_ram_weak.upgrade() {
                    ui.set_ram_usage(format!("{:.1} MB", report.after_mb()).into());
                }
            }
            Err(e) => eprintln!("[AURI Memory Error] {}", e),
        }
    });

    // Wire: Search (Unified Local Library + YouTube Music Streaming)
    let provider_search = provider.clone();
    let library_search = library.clone();
    let ui_search_weak = ui.as_weak();
    ui.on_perform_search(move |query| {
        let q = query.to_string();
        let provider = provider_search.clone();
        let library = library_search.clone();
        let ui_weak = ui_search_weak.clone();

        std::thread::Builder::new()
            .name("auri-unified-search".to_string())
            .spawn(move || {
                let trimmed = q.trim();
                if trimmed.is_empty() {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_search_results(std::rc::Rc::new(slint::VecModel::default()).into());
                        }
                    });
                    return;
                }

                println!("[AURI Search] Executing unified search for: \"{}\"", trimmed);
                let mut combined_results: Vec<TrackData> = Vec::new();

                // 1. Search Local Scanned Library
                let q_lower = trimmed.to_lowercase();
                let local_tracks = library.library_tracks().lock().unwrap().clone();
                for track in local_tracks {
                    if track.title.to_lowercase().contains(&q_lower)
                        || track.artist.to_lowercase().contains(&q_lower)
                        || track.album.to_lowercase().contains(&q_lower)
                    {
                        let dur_str = track.formatted_duration();
                        combined_results.push(TrackData {
                            id: format!("local:{}", track.path.to_string_lossy()).into(),
                            title: track.title.into(),
                            artist: track.artist.into(),
                            album: track.album.into(),
                            duration: dur_str.into(),
                            duration_secs: track.duration_secs as i32,
                            format_badge: "Local Library".into(),
                            is_favorite: false,
                            cover_hue: slint::Color::from_rgb_u8(16, 185, 129),
                        });
                    }
                }

                // 2. Search Online YouTube Music via Innertube
                match provider.search(trimmed) {
                    Ok(items) => {
                        println!("[AURI Streaming] Retrieved {} results from Innertube", items.len());
                        for item in items {
                            combined_results.push(TrackData {
                                id: item.id.into(),
                                title: item.title.into(),
                                artist: item.artist.into(),
                                album: item.album.into(),
                                duration: item.formatted_duration.into(),
                                duration_secs: item.duration_secs as i32,
                                format_badge: "YouTube Music".into(),
                                is_favorite: false,
                                cover_hue: slint::Color::from_rgb_u8(139, 92, 246),
                            });
                        }
                    }
                    Err(e) => {
                        eprintln!("[AURI Streaming Search Error] {}", e);
                    }
                }

                println!("[AURI Search] Dispatching {} total search results to UI", combined_results.len());
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        let model = std::rc::Rc::new(slint::VecModel::from(combined_results));
                        ui.set_search_results(model.into());
                    }
                });
            })
            .expect("Failed to spawn search thread");
    });

    // Wire: Play Stream Track (Unified Local + Online Streaming)
    let provider_resolve = provider.clone();
    let audio_stream = audio.clone();
    let integration_stream = integration.clone();
    let ui_stream_weak = ui.as_weak();
    ui.on_play_stream_track(move |id, title, artist, duration_secs| {
        let id_str = id.to_string();
        let title_str = title.to_string();
        let artist_str = artist.to_string();
        let dur = duration_secs as f64;
        let provider = provider_resolve.clone();
        let audio = audio_stream.clone();
        let integration = integration_stream.clone();
        let ui_weak = ui_stream_weak.clone();

        // Check if this is a local track
        if id_str.starts_with("local:") {
            let file_path = id_str.trim_start_matches("local:").to_string();
            println!("[AURI Local] Playing selected search track from disk: {}", file_path);
            let path = std::path::PathBuf::from(file_path);
            if path.exists() {
                audio.play_file(&path);
                integration.update_tray_tooltip(&title_str, &artist_str);
                let title_disp = title_str.clone();
                let artist_disp = artist_str.clone();
                let dur_disp = format_duration(dur);
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_track_title(title_disp.into());
                        ui.set_track_artist(artist_disp.into());
                        ui.set_track_duration(dur_disp.into());
                        ui.set_is_playing(true);
                    }
                });
            }
            return;
        }

        // Online YouTube Music streaming
        std::thread::Builder::new()
            .name("auri-stream-resolver".to_string())
            .spawn(move || {
                println!("[AURI Streaming] Resolving stream for \"{}\" (id: {})...", title_str, id_str);
                match provider.resolve_stream(&id_str) {
                    Ok(stream) => {
                        println!(
                            "[AURI Streaming] Resolved audio stream: {:?} (bitrate: {} bps)",
                            stream.format, stream.bitrate_bps
                        );
                        let hint = match stream.format {
                            provider::AudioStreamFormat::WebmOpus => Some("webm".to_string()),
                            provider::AudioStreamFormat::Mp4Aac => Some("m4a".to_string()),
                            _ => None,
                        };

                        let stream_dur = if dur > 0.0 { dur } else { stream.duration_secs };
                        audio.play_stream(
                            stream.stream_url,
                            title_str.clone(),
                            artist_str.clone(),
                            stream_dur,
                            hint,
                        );

                        integration.update_tray_tooltip(&title_str, &artist_str);

                        let title_disp = title_str.clone();
                        let artist_disp = artist_str.clone();
                        let dur_disp = format_duration(stream_dur);
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_track_title(title_disp.into());
                                ui.set_track_artist(artist_disp.into());
                                ui.set_track_duration(dur_disp.into());
                                ui.set_is_playing(true);
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("[AURI Streaming Error] Failed to resolve stream for {}: {}", title_str, e);
                    }
                }
            })
            .expect("Failed to spawn stream resolver thread");
    });

    let ui_handle_headless = ui.as_weak();
    let vis_headless = is_window_visible.clone();
    ui.on_enter_headless(move || {
        println!("[AURI Gaming Mode] Headless Mode: UI Rendering paused (0.0% GPU)");
        if let Some(ui) = ui_handle_headless.upgrade() {
            let _ = ui.hide();
            vis_headless.store(false, Ordering::SeqCst);
            if let Ok(report) = trim_working_set() {
                println!("[AURI Memory] Purged working set in headless mode: {:.1} MB", report.after_mb());
                ui.set_ram_usage(format!("{:.1} MB", report.after_mb()).into());
            }
        }
    });

    // Smooth UI synchronization and Windows events timer (10 FPS)
    let ui_weak = ui.as_weak();
    let audio_sync = audio.clone();
    let library_sync = library.clone();
    let integration_sync = integration.clone();
    let vis_sync = is_window_visible.clone();
    let play_helper_sync = play_track_helper.clone();
    let game_detector_sync = game_detector.clone();
    let game_rx_poll = game_rx_sync.clone();
    let mut tick_counter: u64 = 0;

    let sync_timer = Timer::default();
    sync_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(100),
        move || {
            // 0. Process Anti-Cheat Safe Game Detection Events
            if let Ok(rx_guard) = game_rx_poll.try_lock() {
                while let Ok(event) = rx_guard.try_recv() {
                    match event {
                        GameEvent::GameFocused { process_name } => {
                            println!("[AURI Gaming] Foreground game detected: {}", process_name);
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_active_game(process_name.as_str().into());
                                if game_detector_sync.is_enabled() {
                                    let cur_mode = ui.get_current_mode();
                                    if cur_mode == 0 {
                                        // Auto-transition to Gaming HUD Mode (Mode 2)
                                        ui.set_current_mode(2);
                                        println!("[AURI Gaming] Auto-transitioned to Gaming HUD for {}", process_name);
                                        if let Ok(report) = trim_working_set() {
                                            println!("[AURI Memory] Working set trimmed: {:.1} MB", report.after_mb());
                                            ui.set_ram_usage(format!("{:.1} MB", report.after_mb()).into());
                                        }
                                    }
                                }
                            }
                        }
                        GameEvent::GameUnfocused => {
                            println!("[AURI Gaming] Game unfocused.");
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_active_game("".into());
                            }
                        }
                    }
                }
            }
            // 1. Process Global Hotkeys
            while let Some(action) = integration_sync.poll_hotkey() {
                match action {
                    HotkeyAction::PlayPause => {
                        println!("[AURI Hotkey] Play/Pause triggered");
                        let status = audio_sync.get_status();
                        if status.state == PlaybackState::Stopped || status.duration_secs <= 0.0 {
                            if let Some(track) = library_sync.queue().lock().unwrap().current_track().cloned() {
                                play_helper_sync(&track);
                            } else {
                                audio_sync.play_test_tone(440.0, 15.0);
                            }
                        } else {
                            audio_sync.toggle_playback();
                        }
                    }
                    HotkeyAction::NextTrack => {
                        println!("[AURI Hotkey] Next Track triggered");
                        if let Some(track) = library_sync.queue().lock().unwrap().next().cloned() {
                            play_helper_sync(&track);
                        } else {
                            audio_sync.play_test_tone(523.25, 12.0);
                        }
                    }
                    HotkeyAction::PrevTrack => {
                        println!("[AURI Hotkey] Previous Track triggered");
                        if let Some(track) = library_sync.queue().lock().unwrap().previous().cloned() {
                            play_helper_sync(&track);
                        } else {
                            audio_sync.play_test_tone(329.63, 12.0);
                        }
                    }
                    HotkeyAction::VolumeUp => {
                        let cur_vol = audio_sync.get_status().volume;
                        let new_vol = (cur_vol + 0.05).min(1.0);
                        audio_sync.set_volume(new_vol);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_volume(new_vol);
                        }
                        println!("[AURI Hotkey] Volume Up -> {:.0}%", new_vol * 100.0);
                    }
                    HotkeyAction::VolumeDown => {
                        let cur_vol = audio_sync.get_status().volume;
                        let new_vol = (cur_vol - 0.05).max(0.0);
                        audio_sync.set_volume(new_vol);
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_volume(new_vol);
                        }
                        println!("[AURI Hotkey] Volume Down -> {:.0}%", new_vol * 100.0);
                    }
                    HotkeyAction::CycleMode => {
                        if let Some(ui) = ui_weak.upgrade() {
                            let cur = ui.get_current_mode();
                            let next = (cur + 1) % 4;
                            ui.set_current_mode(next);
                            if next == 3 {
                                let _ = ui.hide();
                                vis_sync.store(false, Ordering::SeqCst);
                                println!("[AURI Mode] Cycled to Pure Headless (Window Hidden, 0.0% GPU)");
                            } else {
                                let _ = ui.show();
                                vis_sync.store(true, Ordering::SeqCst);
                                println!("[AURI Mode] Cycled to Mode {}", next);
                            }
                        }
                    }
                    HotkeyAction::ToggleWindow => {
                        let vis = vis_sync.load(Ordering::SeqCst);
                        if let Some(ui) = ui_weak.upgrade() {
                            if vis {
                                let _ = ui.hide();
                                vis_sync.store(false, Ordering::SeqCst);
                                println!("[AURI Hotkey] Window Hidden");
                            } else {
                                let _ = ui.show();
                                vis_sync.store(true, Ordering::SeqCst);
                                println!("[AURI Hotkey] Window Restored");
                            }
                        }
                    }
                }
            }

            // 2. Process System Tray Actions
            while let Some(action) = integration_sync.poll_tray() {
                match action {
                    TrayAction::TogglePlay => {
                        let status = audio_sync.get_status();
                        if status.state == PlaybackState::Stopped || status.duration_secs <= 0.0 {
                            if let Some(track) = library_sync.queue().lock().unwrap().current_track().cloned() {
                                play_helper_sync(&track);
                            } else {
                                audio_sync.play_test_tone(440.0, 15.0);
                            }
                        } else {
                            audio_sync.toggle_playback();
                        }
                    }
                    TrayAction::NextTrack => {
                        if let Some(track) = library_sync.queue().lock().unwrap().next().cloned() {
                            play_helper_sync(&track);
                        } else {
                            audio_sync.play_test_tone(523.25, 12.0);
                        }
                    }
                    TrayAction::PrevTrack => {
                        if let Some(track) = library_sync.queue().lock().unwrap().previous().cloned() {
                            play_helper_sync(&track);
                        } else {
                            audio_sync.play_test_tone(329.63, 12.0);
                        }
                    }
                    TrayAction::SetMode(mode) => {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_current_mode(mode);
                            if mode == 3 {
                                let _ = ui.hide();
                                vis_sync.store(false, Ordering::SeqCst);
                                println!("[AURI Tray] Switched to Pure Headless Mode (0.0% GPU)");
                            } else {
                                let _ = ui.show();
                                vis_sync.store(true, Ordering::SeqCst);
                                println!("[AURI Tray] Switched to Mode {}", mode);
                            }
                        }
                    }
                    TrayAction::ToggleWindow => {
                        let vis = vis_sync.load(Ordering::SeqCst);
                        if let Some(ui) = ui_weak.upgrade() {
                            if vis {
                                let _ = ui.hide();
                                vis_sync.store(false, Ordering::SeqCst);
                                println!("[AURI Tray] Window Hidden to Tray");
                            } else {
                                let _ = ui.show();
                                vis_sync.store(true, Ordering::SeqCst);
                                println!("[AURI Tray] Window Restored from Tray");
                            }
                        }
                    }
                    TrayAction::Exit => {
                        println!("[AURI] Clean exit requested from System Tray");
                        std::process::exit(0);
                    }
                }
            }

            // 3. Check EOF and auto-advance queue
            if audio_sync.take_eof() {
                println!("[AURI Audio] Track EOF reached. Auto-advancing to next track in queue...");
                let next_opt = library_sync.queue().lock().unwrap().next().cloned();
                if let Some(track) = next_opt {
                    play_helper_sync(&track);
                }
            }

            // 4. Update UI Seekbar and Timers
            if let Some(ui) = ui_weak.upgrade() {
                let status = audio_sync.get_status();
                let is_playing = status.state == PlaybackState::Playing;

                // Only update properties when playing or if state changed
                if ui.get_is_playing() != is_playing {
                    ui.set_is_playing(is_playing);
                }

                if is_playing && status.duration_secs > 0.0 {
                    let progress = (status.position_secs / status.duration_secs).clamp(0.0, 1.0) as f32;
                    ui.set_track_progress(progress);
                    ui.set_track_elapsed(format_duration(status.position_secs).into());
                    ui.set_track_duration(format_duration(status.duration_secs).into());
                }

                // Periodic RAM display update (every 1 second / 10 ticks)
                tick_counter = tick_counter.wrapping_add(1);
                if tick_counter % 10 == 0 {
                    if let Some(ws) = get_working_set_size_bytes() {
                        let mb = ws as f64 / (1024.0 * 1024.0);
                        ui.set_ram_usage(format!("{:.1} MB", mb).into());
                    }
                }
            }
        },
    );

    println!("[AURI Startup] Launching interface with active audio engine, local library, and system tray...");
    ui.run()
}
