// Project AURI: Minimalist, Ultra-Lightweight, Gaming-First Music Player
// Milestone v0.2.0: Minimal Audio Engine (Symphonia Codecs & WASAPI Sink)

pub mod audio;

use std::sync::Arc;
use std::time::Duration;
use audio::{AudioEngine, PlaybackState};
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
    println!("  AURI Desktop Music Player - v0.2.0 Audio Core");
    println!("  Pure Rust Decoders • WASAPI Output • MMCSS Real-Time");
    println!("========================================================");

    // Initialize the native audio engine
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

    let ui = AppWindow::new()?;

    // Sync initial state
    let initial_status = audio.get_status();
    ui.set_volume(initial_status.volume);

    // Wire: Play / Pause toggle
    let audio_clone = audio.clone();
    ui.on_toggle_play(move || {
        audio_clone.toggle_playback();
        let status = audio_clone.get_status();
        println!("[AURI Playback] Toggle -> State: {:?}", status.state);

        // If stopped with no track, launch melodic AURI sample chime
        if status.state == PlaybackState::Stopped || status.duration_secs <= 0.0 {
            println!("[AURI Audio] No track active: Generating high-fidelity test tone chime (440 Hz)");
            audio_clone.play_test_tone(440.0, 15.0);
        }
    });

    // Wire: Previous track
    let audio_clone = audio.clone();
    let ui_handle = ui.as_weak();
    ui.on_prev_track(move || {
        println!("[AURI Queue] Playing Previous Track Tone (330 Hz E4)");
        audio_clone.play_test_tone(329.63, 12.0);
        if let Some(ui) = ui_handle.upgrade() {
            ui.set_track_title("AURI Chime (E4 - 330 Hz)".into());
            ui.set_track_artist("Audio Engine Test".into());
        }
    });

    // Wire: Next track
    let audio_clone = audio.clone();
    let ui_handle = ui.as_weak();
    ui.on_next_track(move || {
        println!("[AURI Queue] Playing Next Track Tone (523 Hz C5)");
        audio_clone.play_test_tone(523.25, 12.0);
        if let Some(ui) = ui_handle.upgrade() {
            ui.set_track_title("AURI Chime (C5 - 523 Hz)".into());
            ui.set_track_artist("Audio Engine Test".into());
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
    ui.on_toggle_shuffle(move || {
        println!("[AURI Queue] Shuffle toggled");
    });
    ui.on_toggle_repeat(move || {
        println!("[AURI Queue] Repeat toggled");
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
    let sync_timer = Timer::default();
    sync_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(100),
        move || {
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

    println!("[AURI Startup] Launching interface with active audio engine...");
    ui.run()
}
