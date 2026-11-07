// Project AURI: Minimalist, Ultra-Lightweight, Gaming-First Desktop Music Player
// Sleek Apple Music / Spotify Glassmorphic UI in Slint with Tubefast InnerTube Streaming Engine

pub mod audio;
pub mod library;
pub mod provider;

use std::sync::Arc;
use std::time::Duration;

use audio::{AudioEngine, AudioEngineKind, PlaybackState};
use library::LibraryManager;
use provider::ytm::Client as YtmClient;
use provider::TrackItem;
use slint::{ComponentHandle, ModelRc, VecModel};

slint::include_modules!();

fn format_duration(seconds: f64) -> String {
    let total_secs = seconds.max(0.0) as u64;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{}:{:02}", mins, secs)
}

fn get_working_set_size_bytes() -> Option<usize> {
    #[cfg(windows)]
    {
        #[repr(C)]
        struct PROCESS_MEMORY_COUNTERS {
            cb: u32,
            page_fault_count: u32,
            peak_working_set_size: usize,
            working_set_size: usize,
            quota_peak_paged_pool_usage: usize,
            quota_paged_pool_usage: usize,
            quota_peak_non_paged_pool_usage: usize,
            quota_non_paged_pool_usage: usize,
            pagefile_usage: usize,
            peak_pagefile_usage: usize,
        }

        #[link(name = "psapi")]
        extern "system" {
            fn GetCurrentProcess() -> *mut std::ffi::c_void;
            fn GetProcessMemoryInfo(
                process: *mut std::ffi::c_void,
                counters: *mut PROCESS_MEMORY_COUNTERS,
                cb: u32,
            ) -> i32;
        }

        let mut counters: PROCESS_MEMORY_COUNTERS = unsafe { std::mem::zeroed() };
        counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;

        let ret = unsafe {
            GetProcessMemoryInfo(
                GetCurrentProcess(),
                &mut counters,
                std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            )
        };

        if ret != 0 {
            Some(counters.working_set_size)
        } else {
            None
        }
    }
    #[cfg(not(windows))]
    {
        None
    }
}

fn trim_working_set() {
    #[cfg(windows)]
    {
        #[link(name = "psapi")]
        extern "system" {
            fn GetCurrentProcess() -> *mut std::ffi::c_void;
            fn EmptyWorkingSet(hProcess: *mut std::ffi::c_void) -> i32;
        }
        unsafe {
            EmptyWorkingSet(GetCurrentProcess());
        }
    }
}

fn scan_directory_tracks(dir: &str, lib: &LibraryManager) -> Vec<TrackData> {
    let path = std::path::Path::new(dir);
    let items = lib.scan_local_files(&[path]);
    if items.is_empty() {
        vec![
            TrackData {
                id: "curated-1".into(),
                title: "Horizon Wave (Demo)".into(),
                artist: "AURI Native".into(),
                album: "Lossless Demos".into(),
                duration: "3:34".into(),
                format_badge: "FLAC 24b".into(),
                is_playing: false,
                is_favorite: true,
                color_hue: slint::Color::from_rgb_u8(99, 102, 241),
            },
            TrackData {
                id: "curated-2".into(),
                title: "Pure Sine Dynamics".into(),
                artist: "Studio WASAPI".into(),
                album: "Audio Calibration".into(),
                duration: "2:45".into(),
                format_badge: "WAV 32b".into(),
                is_playing: false,
                is_favorite: false,
                color_hue: slint::Color::from_rgb_u8(16, 185, 129),
            },
        ]
    } else {
        items
            .into_iter()
            .map(|t| TrackData {
                id: t.id.into(),
                title: t.title.into(),
                artist: t.artist.into(),
                album: t.album.into(),
                duration: format_duration(t.duration_secs as f64).into(),
                format_badge: t.format_badge.into(),
                is_playing: false,
                is_favorite: false,
                color_hue: slint::Color::from_rgb_u8(139, 92, 246),
            })
            .collect()
    }
}

fn play_ytm_stream(
    audio: &Arc<AudioEngine>,
    ytm: &Arc<YtmClient>,
    video_id: &str,
) -> Result<(), String> {
    let mut stream = ytm.stream(video_id)?;
    let (url, len, ext) = if stream.opus {
        (stream.url, stream.len, "webm")
    } else if let Some(aac) = stream.fallback.take() {
        (aac.url, aac.len, "m4a")
    } else {
        (stream.url, stream.len, "m4a")
    };
    let stats = Arc::new(provider::buffer::StreamStats::default());
    let seekable = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let fetch = provider::buffer::Fetch::start(
        ytm.agent.clone(),
        url,
        len,
        ytm.player_agent(),
        stats,
    );
    let reader = provider::buffer::StreamReader::new(fetch, seekable);
    audio.play_stream(Box::new(reader), ext, Some(-14.0));
    Ok(())
}

fn main() -> Result<(), slint::PlatformError> {
    println!("========================================================");
    println!("  AURI Desktop Music Player                            ");
    println!("  Apple Glassmorphic Dark UI • Tubefast Streaming Core  ");
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

    // Initialize YouTube Music Client (Tubefast VisionOS engine)
    let ytm = Arc::new(YtmClient::new(String::new()));
    println!("[AURI Streaming] YouTube Music Client online (VisionOS player)");

    // Initialize Local Music Library & Queue Manager
    let library = Arc::new(LibraryManager::new());
    println!("[AURI Library] Playback queue initialized");

    // Create Slint Window
    let window = MainWindow::new()?;

    // Populate Initial Curated Home Tracks (Matching Reference 1)
    let home_tracks = vec![
        TrackData {
            id: "curated-1".into(),
            title: "Tamagotchi".into(),
            artist: "Taco Hemingway ft. Quebonafide".into(),
            album: "Soma 0,5".into(),
            duration: "3:18".into(),
            format_badge: "FLAC 24b".into(),
            is_playing: false,
            is_favorite: true,
            color_hue: slint::Color::from_rgb_u8(236, 72, 153), // Pink
        },
        TrackData {
            id: "curated-2".into(),
            title: "Wszystko na niby".into(),
            artist: "Taco Hemingway".into(),
            album: "Café Belga".into(),
            duration: "3:32".into(),
            format_badge: "Opus 160k".into(),
            is_playing: false,
            is_favorite: false,
            color_hue: slint::Color::from_rgb_u8(99, 102, 241), // Indigo
        },
        TrackData {
            id: "curated-3".into(),
            title: "W piątki leżę w wannie".into(),
            artist: "Taco Hemingway".into(),
            album: "Pocztówka z WWA".into(),
            duration: "3:24".into(),
            format_badge: "Opus 160k".into(),
            is_playing: false,
            is_favorite: true,
            color_hue: slint::Color::from_rgb_u8(139, 92, 246), // Purple
        },
        TrackData {
            id: "curated-4".into(),
            title: "Nostalgia".into(),
            artist: "Taco Hemingway".into(),
            album: "Szprycer".into(),
            duration: "3:48".into(),
            format_badge: "FLAC 24b".into(),
            is_playing: false,
            is_favorite: false,
            color_hue: slint::Color::from_rgb_u8(16, 185, 129), // Emerald
        },
        TrackData {
            id: "curated-5".into(),
            title: "Deszcz na betonie".into(),
            artist: "Taco Hemingway".into(),
            album: "Marmur".into(),
            duration: "4:06".into(),
            format_badge: "FLAC".into(),
            is_playing: false,
            is_favorite: true,
            color_hue: slint::Color::from_rgb_u8(245, 158, 11), // Amber
        },
    ];

    let home_albums = vec![
        AlbumData {
            id: "alb-1".into(),
            title: "Pocztówka z WWA, lato '19".into(),
            artist: "Taco Hemingway".into(),
            year: "2019".into(),
            track_count: 11,
            color_hue: slint::Color::from_rgb_u8(251, 191, 36),
        },
        AlbumData {
            id: "alb-2".into(),
            title: "Café Belga".into(),
            artist: "Taco Hemingway".into(),
            year: "2018".into(),
            track_count: 11,
            color_hue: slint::Color::from_rgb_u8(20, 184, 166),
        },
        AlbumData {
            id: "alb-3".into(),
            title: "Flagey EP".into(),
            artist: "Taco Hemingway".into(),
            year: "2018".into(),
            track_count: 5,
            color_hue: slint::Color::from_rgb_u8(147, 51, 234),
        },
        AlbumData {
            id: "alb-4".into(),
            title: "Szprycer".into(),
            artist: "Taco Hemingway".into(),
            year: "2017".into(),
            track_count: 9,
            color_hue: slint::Color::from_rgb_u8(239, 68, 68),
        },
        AlbumData {
            id: "alb-5".into(),
            title: "Marmur".into(),
            artist: "Taco Hemingway".into(),
            year: "2016".into(),
            track_count: 17,
            color_hue: slint::Color::from_rgb_u8(59, 130, 246),
        },
        AlbumData {
            id: "alb-6".into(),
            title: "Wosk".into(),
            artist: "Taco Hemingway".into(),
            year: "2016".into(),
            track_count: 6,
            color_hue: slint::Color::from_rgb_u8(107, 114, 128),
        },
    ];

    let similar_artists = vec![
        ArtistData {
            name: "Dawid Podsiadło".into(),
            role: "Pop / Indie".into(),
            color_hue: slint::Color::from_rgb_u8(245, 158, 11),
        },
        ArtistData {
            name: "Billie Eilish".into(),
            role: "Alt Pop".into(),
            color_hue: slint::Color::from_rgb_u8(16, 185, 129),
        },
        ArtistData {
            name: "Quebonafide".into(),
            role: "Hip-Hop".into(),
            color_hue: slint::Color::from_rgb_u8(99, 102, 241),
        },
        ArtistData {
            name: "Daria Zawiałow".into(),
            role: "Electro-Pop".into(),
            color_hue: slint::Color::from_rgb_u8(236, 72, 153),
        },
    ];

    // Seed Queue Manager with initial tracks
    let queue_items: Vec<TrackItem> = home_tracks
        .iter()
        .map(|t| TrackItem {
            id: t.id.to_string(),
            title: t.title.to_string(),
            artist: t.artist.to_string(),
            album: t.album.to_string(),
            duration_secs: 214,
            thumbnail_url: String::new(),
            is_local: false,
            file_path: None,
            loudness_db: Some(-14.0),
            format_badge: t.format_badge.to_string(),
        })
        .collect();
    library.queue().set_items(queue_items, Some(0));

    // Populate initial models
    window.set_home_tracks(ModelRc::new(VecModel::from(home_tracks.clone())));
    window.set_home_albums(ModelRc::new(VecModel::from(home_albums.clone())));
    window.set_all_albums(ModelRc::new(VecModel::from(home_albums)));
    window.set_queue_tracks(ModelRc::new(VecModel::from(home_tracks.clone())));
    window.set_similar_artists(ModelRc::new(VecModel::from(similar_artists)));

    // Populate favorites from home_tracks
    let favorites: Vec<TrackData> = home_tracks.iter().filter(|t| t.is_favorite).cloned().collect();
    window.set_favorite_tracks(ModelRc::new(VecModel::from(favorites)));

    // Scan initial local tracks
    let default_local = scan_directory_tracks("C:\\Users\\user\\Music", &library);
    window.set_local_tracks(ModelRc::new(VecModel::from(default_local)));

    let win_weak = window.as_weak();

    // 1. Navigation
    let win_nav = win_weak.clone();
    window.on_navigate(move |page| {
        if let Some(w) = win_nav.upgrade() {
            w.set_current_nav(page);
        }
    });

    // 2. Play / Pause Toggle
    let audio_toggle = Arc::clone(&audio);
    let win_play = win_weak.clone();
    window.on_toggle_play(move || {
        let _ = audio_toggle.toggle_playback();
        let status = audio_toggle.get_status();
        if let Some(w) = win_play.upgrade() {
            w.set_is_playing(status.state == PlaybackState::Playing);
        }
    });

    // 3. Previous Track
    let queue_prev = Arc::clone(&library);
    let audio_prev = Arc::clone(&audio);
    let ytm_prev = Arc::clone(&ytm);
    let win_prev = win_weak.clone();
    window.on_prev_track(move || {
        if let Some(track) = queue_prev.queue().previous() {
            if let Some(w) = win_prev.upgrade() {
                w.set_track_title(track.title.clone().into());
                w.set_track_artist(track.artist.clone().into());
                w.set_track_album(track.album.clone().into());
                w.set_track_duration(format_duration(track.duration_secs as f64).into());
                w.set_is_playing(true);
            }
            let audio_c = Arc::clone(&audio_prev);
            let ytm_c = Arc::clone(&ytm_prev);
            std::thread::spawn(move || {
                if track.id.starts_with("local:") {
                    let path = &track.id["local:".len()..];
                    audio_c.play_file(path, None);
                } else if track.id.starts_with("curated-") {
                    let query = format!("{} {}", track.title, track.artist);
                    if let Ok(page) = ytm_c.search(&query) {
                        if let Some(first) = page.songs().first() {
                            let _ = play_ytm_stream(&audio_c, &ytm_c, &first.video_id);
                        }
                    }
                } else {
                    let _ = play_ytm_stream(&audio_c, &ytm_c, &track.id);
                }
            });
        }
    });

    // 4. Next Track
    let queue_next = Arc::clone(&library);
    let audio_next = Arc::clone(&audio);
    let ytm_next = Arc::clone(&ytm);
    let win_next = win_weak.clone();
    window.on_next_track(move || {
        if let Some(track) = queue_next.queue().next() {
            if let Some(w) = win_next.upgrade() {
                w.set_track_title(track.title.clone().into());
                w.set_track_artist(track.artist.clone().into());
                w.set_track_album(track.album.clone().into());
                w.set_track_duration(format_duration(track.duration_secs as f64).into());
                w.set_is_playing(true);
            }
            let audio_c = Arc::clone(&audio_next);
            let ytm_c = Arc::clone(&ytm_next);
            std::thread::spawn(move || {
                if track.id.starts_with("local:") {
                    let path = &track.id["local:".len()..];
                    audio_c.play_file(path, None);
                } else if track.id.starts_with("curated-") {
                    let query = format!("{} {}", track.title, track.artist);
                    if let Ok(page) = ytm_c.search(&query) {
                        if let Some(first) = page.songs().first() {
                            let _ = play_ytm_stream(&audio_c, &ytm_c, &first.video_id);
                        }
                    }
                } else {
                    let _ = play_ytm_stream(&audio_c, &ytm_c, &track.id);
                }
            });
        }
    });

    // 5. Seek
    let audio_seek = Arc::clone(&audio);
    window.on_seek(move |progress| {
        let status = audio_seek.get_status();
        let target_secs = progress as f64 * status.duration.as_secs_f64();
        let _ = audio_seek.seek(target_secs);
    });

    // 6. Volume
    let audio_vol = Arc::clone(&audio);
    let win_vol = win_weak.clone();
    window.on_volume_changed(move |val| {
        audio_vol.set_volume(val);
        if let Some(w) = win_vol.upgrade() {
            w.set_volume(val);
        }
    });

    // 7. Toggle Favorite (Now-Playing Card)
    let win_fav = win_weak.clone();
    window.on_toggle_favorite(move || {
        if let Some(w) = win_fav.upgrade() {
            let next_fav = !w.get_is_favorite();
            w.set_is_favorite(next_fav);
        }
    });

    // 8. Toggle Favorite Track by ID
    let win_fav_track = win_weak.clone();
    window.on_toggle_track_favorite(move |_id| {
        if let Some(w) = win_fav_track.upgrade() {
            let cur = w.get_is_favorite();
            w.set_is_favorite(!cur);
        }
    });

    // 9. Shuffle
    let queue_shuf = Arc::clone(&library);
    let win_shuf = win_weak.clone();
    window.on_toggle_shuffle(move || {
        let active = queue_shuf.queue().toggle_shuffle();
        if let Some(w) = win_shuf.upgrade() {
            w.set_is_shuffle(active);
        }
    });

    // 10. Repeat
    let queue_rep = Arc::clone(&library);
    let win_rep = win_weak.clone();
    window.on_toggle_repeat(move || {
        let mode = queue_rep.queue().toggle_repeat();
        if let Some(w) = win_rep.upgrade() {
            w.set_is_repeat(mode != library::RepeatMode::Off);
        }
    });

    // 11. Purge RAM Working Set
    let win_trim = win_weak.clone();
    window.on_trim_ram(move || {
        trim_working_set();
        if let Some(ws) = get_working_set_size_bytes() {
            let mb = ws as f64 / (1024.0 * 1024.0);
            if let Some(w) = win_trim.upgrade() {
                w.set_ram_usage(format!("{:.1} MB", mb).into());
            }
        }
    });

    // 12. Search (Tubefast YTM Search)
    let ytm_search = Arc::clone(&ytm);
    let win_search = win_weak.clone();
    window.on_perform_search(move |query| {
        let q = query.to_string();
        let yt = Arc::clone(&ytm_search);
        let win = win_search.clone();

        std::thread::spawn(move || {
            if let Ok(page) = yt.search(&q) {
                let songs = page.songs();
                let slint_tracks: Vec<TrackData> = songs
                    .into_iter()
                    .map(|item| TrackData {
                        id: item.video_id.into(),
                        title: item.title.into(),
                        artist: item.subtitle.into(),
                        album: "YouTube Music".into(),
                        duration: if item.duration.is_empty() {
                            "3:30".into()
                        } else {
                            item.duration.into()
                        },
                        format_badge: "Opus 160k".into(),
                        is_playing: false,
                        is_favorite: false,
                        color_hue: slint::Color::from_rgb_u8(99, 102, 241),
                    })
                    .collect();

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(w) = win.upgrade() {
                        w.set_search_results(ModelRc::new(VecModel::from(slint_tracks)));
                    }
                });
            }
        });
    });

    // 13. Play Track (Instant Real Music Playback)
    let audio_play = Arc::clone(&audio);
    let ytm_play = Arc::clone(&ytm);
    let win_track = win_weak.clone();
    window.on_play_track(move |id, title, artist, dur_secs| {
        let video_id = id.to_string();
        let tit = title.to_string();
        let art = artist.to_string();

        if let Some(w) = win_track.upgrade() {
            w.set_track_title(tit.clone().into());
            w.set_track_artist(art.clone().into());
            w.set_track_elapsed("0:00".into());
            w.set_track_progress(0.0);
            if dur_secs > 0 {
                w.set_track_duration(format_duration(dur_secs as f64).into());
            }
            w.set_is_playing(true);
        }

        let audio = Arc::clone(&audio_play);
        let yt = Arc::clone(&ytm_play);

        std::thread::spawn(move || {
            if video_id.starts_with("local:") {
                let file_path = &video_id["local:".len()..];
                audio.play_file(file_path, None);
            } else if video_id.starts_with("curated-") {
                // Search and stream the actual song from YouTube Music!
                let query = format!("{} {}", tit, art);
                if let Ok(page) = yt.search(&query) {
                    if let Some(first) = page.songs().first() {
                        if play_ytm_stream(&audio, &yt, &first.video_id).is_ok() {
                            return;
                        }
                    }
                }
                audio.play_test_tone(440.0, 5.0);
            } else {
                if let Err(e) = play_ytm_stream(&audio, &yt, &video_id) {
                    eprintln!("[AURI Stream Error] Failed to stream {}: {}", video_id, e);
                }
            }
        });
    });

    // 14. Rescan Local Folder
    let lib_rescan = Arc::clone(&library);
    let win_rescan = win_weak.clone();
    window.on_rescan_local(move |path_str| {
        let path = path_str.to_string();
        let scanned = scan_directory_tracks(&path, &lib_rescan);
        if let Some(w) = win_rescan.upgrade() {
            w.set_local_tracks(ModelRc::new(VecModel::from(scanned)));
        }
    });

    // 15. Play Album
    let win_alb = win_weak.clone();
    let audio_alb = Arc::clone(&audio);
    let ytm_alb = Arc::clone(&ytm);
    window.on_play_album(move |alb_id| {
        let query = format!("album {}", alb_id);
        if let Some(w) = win_alb.upgrade() {
            w.set_is_playing(true);
            w.set_track_album(format!("Album {}", alb_id).into());
        }
        let audio = Arc::clone(&audio_alb);
        let yt = Arc::clone(&ytm_alb);
        std::thread::spawn(move || {
            if let Ok(page) = yt.search(&query) {
                if let Some(first) = page.songs().first() {
                    let _ = play_ytm_stream(&audio, &yt, &first.video_id);
                }
            }
        });
    });

    // 16. Toggle Loudness Normalization
    let win_loudness = win_weak.clone();
    window.on_toggle_loudness(move |enabled| {
        if let Some(w) = win_loudness.upgrade() {
            w.set_loudness_norm_enabled(enabled);
        }
    });

    // 17. Switch Audio Engine
    let audio_eng = Arc::clone(&audio);
    let win_eng = win_weak.clone();
    window.on_switch_audio_engine(move |eng_idx| {
        let engine = if eng_idx == 0 {
            AudioEngineKind::NativeWasapi
        } else {
            AudioEngineKind::StudioMpv
        };
        audio_eng.set_engine(engine);
        if let Some(w) = win_eng.upgrade() {
            w.set_audio_engine_name(engine.display_name().into());
        }
    });

    // 18. Timer: 10 FPS UI Progress & RAM synchronizer
    let win_timer = win_weak.clone();
    let audio_timer = Arc::clone(&audio);
    let timer = slint::Timer::default();
    let mut tick = 0u64;

    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(100),
        move || {
            if let Some(w) = win_timer.upgrade() {
                let status = audio_timer.get_status();
                let dur_secs = status.duration.as_secs_f64();
                let pos_secs = status.position.as_secs_f64();

                if dur_secs > 0.0 {
                    let progress = (pos_secs / dur_secs).clamp(0.0, 1.0) as f32;
                    w.set_track_progress(progress);
                    w.set_track_elapsed(format_duration(pos_secs).into());
                    w.set_track_duration(format_duration(dur_secs).into());
                }

                w.set_is_playing(status.state == PlaybackState::Playing);

                // Periodic RAM update (every 1 second / 10 ticks)
                tick = tick.wrapping_add(1);
                if tick % 10 == 0 {
                    if let Some(ws) = get_working_set_size_bytes() {
                        let mb = ws as f64 / (1024.0 * 1024.0);
                        w.set_ram_usage(format!("{:.1} MB", mb).into());
                    }
                }
            }
        },
    );

    println!("[AURI Startup] Online. Launching Glassmorphic UI...");
    window.run()
}
