// Project AURI: Minimalist, Ultra-Lightweight, Gaming-First Desktop Music Player
// Phase 1: High-Fidelity Audio Core & Streaming Engine (Milestone v0.1.0)

pub mod audio;
pub mod library;
pub mod provider;

use std::sync::Arc;

use audio::AudioEngine;
use library::LibraryManager;
use provider::innertube::InnertubeClient;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    println!("========================================================");
    println!("  AURI Desktop Music Player - Milestone v0.1.0         ");
    println!("  High-Fidelity Audio Core & Streaming Engine           ");
    println!("========================================================");

    // Initialize native WASAPI audio engine
    let audio = match AudioEngine::new() {
        Ok(engine) => {
            let status = engine.get_status();
            println!("[AURI Audio Engine] Online via Windows WASAPI");
            println!("  Endpoint Device: {}", status.device_name);
            println!("  Hardware Mixing: {} Hz, {} channels", status.sample_rate, status.channels);
            println!("  Loudness Target: -7.0 LUFS with Soft-Knee Peak Limiter");
            Arc::new(engine)
        }
        Err(err) => {
            eprintln!("[AURI Audio Engine Error] Initialization failed: {}", err);
            panic!("Cannot run AURI without audio sink: {}", err);
        }
    };

    // Initialize YouTube Music Innertube Client
    let _innertube = Arc::new(InnertubeClient::new());
    println!("[AURI Streaming] Innertube Client initialized (synchronous, zero Tokio bloat)");

    // Initialize Local Music Library & Queue Manager
    let _library = Arc::new(LibraryManager::new());
    println!("[AURI Library] Playback queue initialized");

    // Play initial subtle startup chime (A4 440 Hz) to verify WASAPI audio output is functional
    println!("[AURI Audio] Playing startup chime (440 Hz A4)...");
    audio.play_test_tone(440.0, 0.4);

    // Create Slint Window
    let window = MainWindow::new()?;

    println!("[AURI Startup] Phase 1 Audio Core verified. Launching window...");
    window.run()
}
