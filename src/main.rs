// Project AURI: Minimalist, Ultra-Lightweight, Gaming-First Music Player
// Milestone v0.2.0: Minimal Audio Engine (Symphonia Codecs & WASAPI Sink)

pub mod audio;
pub mod library;

use std::sync::Arc;
use std::time::Duration;
use audio::{AudioEngine, PlaybackState};
use library::{LibraryManager, RepeatMode};
use slint::Timer;

slint::include_modules!();

fn format_duration(seconds: f64) -> String {
    let total_secs = seconds.max(0.0) as u64;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{}:{:02}", mins, secs)
}

fn main() -> Result<(), slint::PlatformError> {
    println!("========================================================");
    println!("  AURI Desktop Music Player - v0.3.0 Library & Queue Core");
    println!("  Native Decoders • WASAPI Real-Time • 16MB LRU Cache");
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

    let ui = AppWindow::new()?;

    // Sync initial state from discovered library
    let initial_status = audio.get_status();
    ui.set_volume(initial_status.volume);

    if let Some(first_track) = library.queue().lock().unwrap().current_track().cloned() {
        ui.set_track_title(first_track.title.as_str().into());
        ui.set_track_artist(first_track.artist.as_str().into());
        ui.set_track_duration(first_track.formatted_duration().into());
        println!("[AURI Queue] Initial track staged: {} - {}", first_track.artist, first_track.title);
    }

    // Wire: Play / Pause toggle
    let audio_clone = audio.clone();
    let library_clone = library.clone();
    let ui_handle = ui.as_weak();
    ui.on_toggle_play(move || {
        let status = audio_clone.get_status();
        if status.state == PlaybackState::Stopped || status.duration_secs <= 0.0 {
            let cur_opt = library_clone.queue().lock().unwrap().current_track().cloned();
            if let Some(track) = cur_opt {
                println!("[AURI Playback] Playing track from queue: {} - {}", track.artist, track.title);
                audio_clone.play_file(&track.path);
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_track_title(track.title.as_str().into());
                    ui.set_track_artist(track.artist.as_str().into());
                    ui.set_track_duration(track.formatted_duration().into());
                }
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
    let ui_handle = ui.as_weak();
    ui.on_prev_track(move || {
        let prev_opt = library_clone.queue().lock().unwrap().previous().cloned();
        if let Some(track) = prev_opt {
            println!("[AURI Queue] Playing Previous: {} - {}", track.artist, track.title);
            audio_clone.play_file(&track.path);
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_track_title(track.title.as_str().into());
                ui.set_track_artist(track.artist.as_str().into());
                ui.set_track_duration(track.formatted_duration().into());
            }
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
    let ui_handle = ui.as_weak();
    ui.on_next_track(move || {
        let next_opt = library_clone.queue().lock().unwrap().next().cloned();
        if let Some(track) = next_opt {
            println!("[AURI Queue] Playing Next: {} - {}", track.artist, track.title);
            audio_clone.play_file(&track.path);
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_track_title(track.title.as_str().into());
                ui.set_track_artist(track.artist.as_str().into());
                ui.set_track_duration(track.formatted_duration().into());
            }
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

    ui.on_enter_headless(move || {
        println!("[AURI Gaming Mode] Headless Mode: UI Rendering paused (0.0% GPU)");
    });

    // Smooth UI synchronization timer (updates seekbar and timers at 10 FPS without loading CPU)
    let ui_weak = ui.as_weak();
    let audio_sync = audio.clone();
    let library_sync = library.clone();
    let sync_timer = Timer::default();
    sync_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(100),
        move || {
            // Check EOF and auto-advance queue
            if audio_sync.take_eof() {
                println!("[AURI Audio] Track EOF reached. Auto-advancing to next track in queue...");
                let next_opt = library_sync.queue().lock().unwrap().next().cloned();
                if let Some(track) = next_opt {
                    audio_sync.play_file(&track.path);
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_track_title(track.title.as_str().into());
                        ui.set_track_artist(track.artist.as_str().into());
                        ui.set_track_duration(track.formatted_duration().into());
                    }
                }
            }

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
            }
        },
    );

    println!("[AURI Startup] Launching interface with active audio engine and local library...");
    ui.run()
}
