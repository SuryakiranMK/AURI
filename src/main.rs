// Project AURI: Minimalist, Ultra-Lightweight, Gaming-First Desktop Music Player
// Sleek Apple Music / Spotify Glassmorphic UI in Slint with Tubefast InnerTube Streaming Engine

pub mod artwork;
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

#[cfg(windows)]
pub fn set_window_topmost(pinned: bool) {
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
            flags: u32,
        ) -> i32;
    }

    let title_utf16: Vec<u16> = "AURI\0".encode_utf16().collect();
    let hwnd = unsafe { FindWindowW(std::ptr::null(), title_utf16.as_ptr()) };
    if !hwnd.is_null() {
        const HWND_TOPMOST: isize = -1;
        const HWND_NOTOPMOST: isize = -2;
        const SWP_NOMOVE: u32 = 0x0002;
        const SWP_NOSIZE: u32 = 0x0001;
        const SWP_NOACTIVATE: u32 = 0x0010;

        let insert_after = if pinned {
            HWND_TOPMOST as *mut std::ffi::c_void
        } else {
            HWND_NOTOPMOST as *mut std::ffi::c_void
        };
        unsafe {
            SetWindowPos(
                hwnd,
                insert_after,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
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
    duration_secs: Option<f64>,
) -> Result<(), String> {
    let mut stream = ytm.stream(video_id)?;
    // Prefer AAC (fallback) because Symphonia's native AAC decoder is rock-solid,
    // bit-perfect, and never panics with arithmetic overflow.
    let (url, len, ext) = if let Some(aac) = stream.fallback.take() {
        (aac.url, aac.len, "m4a")
    } else if !stream.opus {
        (stream.url, stream.len, "m4a")
    } else {
        (stream.url, stream.len, "webm")
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
    audio.play_stream(Box::new(reader), ext, Some(-14.0), duration_secs);
    Ok(())
}

fn load_artwork_async(win: slint::Weak<MainWindow>, video_id: String) {
    std::thread::spawn(move || {
        if let Some(path) = artwork::fetch_artwork(&video_id, None) {
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(w) = win.upgrade() {
                    if let Ok(img) = slint::Image::load_from_path(&path) {
                        w.set_current_artwork(img);
                        w.set_has_artwork(true);
                    }
                }
            });
        }
    });
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

    // Populate Initial Curated Home Tracks (Popular Hits with Real Streaming IDs)
    let home_tracks = vec![
        TrackData {
            id: "dhurandhar".into(),
            title: "Dhurandhar - Title Track".into(),
            artist: "Shashwat Sachdev, Hanumankind".into(),
            album: "Dhurandhar".into(),
            duration: "3:30".into(),
            format_badge: "Opus 160k".into(),
            is_playing: false,
            is_favorite: true,
            color_hue: slint::Color::from_rgb_u8(236, 72, 153), // Pink
        },
        TrackData {
            id: "34Na4j8AVgA".into(),
            title: "Starboy".into(),
            artist: "The Weeknd ft. Daft Punk".into(),
            album: "Starboy".into(),
            duration: "3:50".into(),
            format_badge: "Opus 160k".into(),
            is_playing: false,
            is_favorite: false,
            color_hue: slint::Color::from_rgb_u8(99, 102, 241), // Indigo
        },
        TrackData {
            id: "4NRXx6U8ABQ".into(),
            title: "Blinding Lights".into(),
            artist: "The Weeknd".into(),
            album: "After Hours".into(),
            duration: "3:20".into(),
            format_badge: "Opus 160k".into(),
            is_playing: false,
            is_favorite: true,
            color_hue: slint::Color::from_rgb_u8(139, 92, 246), // Purple
        },
        TrackData {
            id: "pHnm_2n52xM".into(),
            title: "Espresso".into(),
            artist: "Sabrina Carpenter".into(),
            album: "Short n' Sweet".into(),
            duration: "2:55".into(),
            format_badge: "FLAC 24b".into(),
            is_playing: false,
            is_favorite: false,
            color_hue: slint::Color::from_rgb_u8(16, 185, 129), // Emerald
        },
        TrackData {
            id: "JGwWNGJdvx8".into(),
            title: "Shape of You".into(),
            artist: "Ed Sheeran".into(),
            album: "Divide".into(),
            duration: "3:53".into(),
            format_badge: "FLAC".into(),
            is_playing: false,
            is_favorite: true,
            color_hue: slint::Color::from_rgb_u8(245, 158, 11), // Amber
        },
    ];

    let home_albums = vec![
        AlbumData {
            id: "alb-1".into(),
            title: "Starboy".into(),
            artist: "The Weeknd".into(),
            year: "2016".into(),
            track_count: 18,
            color_hue: slint::Color::from_rgb_u8(251, 191, 36),
        },
        AlbumData {
            id: "alb-2".into(),
            title: "After Hours".into(),
            artist: "The Weeknd".into(),
            year: "2020".into(),
            track_count: 14,
            color_hue: slint::Color::from_rgb_u8(20, 184, 166),
        },
        AlbumData {
            id: "alb-3".into(),
            title: "Short n' Sweet".into(),
            artist: "Sabrina Carpenter".into(),
            year: "2024".into(),
            track_count: 12,
            color_hue: slint::Color::from_rgb_u8(147, 51, 234),
        },
        AlbumData {
            id: "alb-4".into(),
            title: "Divide".into(),
            artist: "Ed Sheeran".into(),
            year: "2017".into(),
            track_count: 16,
            color_hue: slint::Color::from_rgb_u8(239, 68, 68),
        },
        AlbumData {
            id: "alb-5".into(),
            title: "Dhurandhar".into(),
            artist: "Shashwat Sachdev".into(),
            year: "2024".into(),
            track_count: 8,
            color_hue: slint::Color::from_rgb_u8(59, 130, 246),
        },
        AlbumData {
            id: "alb-6".into(),
            title: "D-Day".into(),
            artist: "Agust D".into(),
            year: "2023".into(),
            track_count: 10,
            color_hue: slint::Color::from_rgb_u8(107, 114, 128),
        },
    ];

    let similar_artists = vec![
        ArtistData {
            name: "Hanumankind".into(),
            role: "Hip-Hop / Rap".into(),
            color_hue: slint::Color::from_rgb_u8(245, 158, 11),
        },
        ArtistData {
            name: "The Weeknd".into(),
            role: "R&B / Synthpop".into(),
            color_hue: slint::Color::from_rgb_u8(16, 185, 129),
        },
        ArtistData {
            name: "Sabrina Carpenter".into(),
            role: "Pop".into(),
            color_hue: slint::Color::from_rgb_u8(99, 102, 241),
        },
        ArtistData {
            name: "Ed Sheeran".into(),
            role: "Acoustic Pop".into(),
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

    // Initial artwork load
    load_artwork_async(win_weak.clone(), "dhurandhar".to_string());

    // Asynchronously fetch live YouTube Music Home Feed & Recommendations (Tubefast architecture)
    let ytm_home = Arc::clone(&ytm);
    let win_home = win_weak.clone();
    std::thread::spawn(move || {
        // Prefer explore (Top Trending Charts) or home feed
        let page = ytm_home.browse("FEmusic_explore", &|_| {})
            .or_else(|_| ytm_home.browse("FEmusic_home", &|_| {}));

        if let Ok(page) = page {
            let mut dynamic_tracks = Vec::new();
            let mut dynamic_albums = Vec::new();
            let mut dynamic_artists = Vec::new();

            for sec in page.sections {
                for (i, item) in sec.items.into_iter().enumerate() {
                    if item.is_song() && dynamic_tracks.len() < 15 {
                        let hue = match i % 5 {
                            0 => slint::Color::from_rgb_u8(236, 72, 153),
                            1 => slint::Color::from_rgb_u8(99, 102, 241),
                            2 => slint::Color::from_rgb_u8(139, 92, 246),
                            3 => slint::Color::from_rgb_u8(16, 185, 129),
                            _ => slint::Color::from_rgb_u8(245, 158, 11),
                        };
                        dynamic_tracks.push(TrackData {
                            id: item.video_id.into(),
                            title: item.title.into(),
                            artist: item.subtitle.into(),
                            album: item.extra.into(),
                            duration: if item.duration.is_empty() { "3:30".into() } else { item.duration.into() },
                            format_badge: "Opus 160k".into(),
                            is_playing: false,
                            is_favorite: false,
                            color_hue: hue,
                        });
                    } else if !item.album_id.is_empty() && dynamic_albums.len() < 8 {
                        dynamic_albums.push(AlbumData {
                            id: item.album_id.into(),
                            title: item.title.into(),
                            artist: item.subtitle.into(),
                            year: "2024".into(),
                            track_count: 10,
                            color_hue: slint::Color::from_rgb_u8(99, 102, 241),
                        });
                    } else if item.is_artist && dynamic_artists.len() < 6 {
                        dynamic_artists.push(ArtistData {
                            name: item.title.into(),
                            role: if item.subtitle.is_empty() { "Artist".into() } else { item.subtitle.into() },
                            color_hue: slint::Color::from_rgb_u8(139, 92, 246),
                        });
                    }
                }
            }

            // If regional explore returned no artist channels, extract trending artists from tracks
            if dynamic_artists.is_empty() {
                let mut seen_artists = std::collections::HashSet::new();
                for t in &dynamic_tracks {
                    let artist_str = t.artist.as_str();
                    let clean = artist_str.split(&[',', '•', '&'][..]).next().unwrap_or(artist_str).trim();
                    if !clean.is_empty() && clean != "Video" && clean != "Song" && seen_artists.insert(clean.to_string()) {
                        dynamic_artists.push(ArtistData {
                            name: clean.into(),
                            role: "Trending Artist".into(),
                            color_hue: t.color_hue,
                        });
                        if dynamic_artists.len() >= 5 {
                            break;
                        }
                    }
                }
            }

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(w) = win_home.upgrade() {
                    if !dynamic_tracks.is_empty() {
                        w.set_home_tracks(ModelRc::new(VecModel::from(dynamic_tracks.clone())));
                        w.set_queue_tracks(ModelRc::new(VecModel::from(dynamic_tracks)));
                    }
                    if !dynamic_albums.is_empty() {
                        w.set_home_albums(ModelRc::new(VecModel::from(dynamic_albums)));
                    }
                    if !dynamic_artists.is_empty() {
                        w.set_similar_artists(ModelRc::new(VecModel::from(dynamic_artists)));
                    }
                }
            });
        }
    });

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
            load_artwork_async(win_prev.clone(), track.id.clone());
            let audio_c = Arc::clone(&audio_prev);
            let ytm_c = Arc::clone(&ytm_prev);
            let win_art = win_prev.clone();
            std::thread::spawn(move || {
                if track.id.starts_with("local:") {
                    let path = &track.id["local:".len()..];
                    audio_c.play_file(path, None);
                } else if track.id.starts_with("curated-") {
                    let query = format!("{} {}", track.title, track.artist);
                    if let Ok(page) = ytm_c.search(&query) {
                        if let Some(first) = page.songs().first() {
                            load_artwork_async(win_art, first.video_id.clone());
                            let _ = play_ytm_stream(&audio_c, &ytm_c, &first.video_id, Some(track.duration_secs as f64));
                        }
                    }
                } else {
                    let _ = play_ytm_stream(&audio_c, &ytm_c, &track.id, Some(track.duration_secs as f64));
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
            load_artwork_async(win_next.clone(), track.id.clone());
            let audio_c = Arc::clone(&audio_next);
            let ytm_c = Arc::clone(&ytm_next);
            let win_art = win_next.clone();
            std::thread::spawn(move || {
                if track.id.starts_with("local:") {
                    let path = &track.id["local:".len()..];
                    audio_c.play_file(path, None);
                } else if track.id.starts_with("curated-") {
                    let query = format!("{} {}", track.title, track.artist);
                    if let Ok(page) = ytm_c.search(&query) {
                        if let Some(first) = page.songs().first() {
                            load_artwork_async(win_art, first.video_id.clone());
                            let _ = play_ytm_stream(&audio_c, &ytm_c, &first.video_id, Some(track.duration_secs as f64));
                        }
                    }
                } else {
                    let _ = play_ytm_stream(&audio_c, &ytm_c, &track.id, Some(track.duration_secs as f64));
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

        load_artwork_async(win_track.clone(), video_id.clone());

        let audio = Arc::clone(&audio_play);
        let yt = Arc::clone(&ytm_play);
        let win_art = win_track.clone();

        std::thread::spawn(move || {
            if video_id.starts_with("local:") {
                let file_path = &video_id["local:".len()..];
                audio.play_file(file_path, None);
            } else if video_id.starts_with("curated-") {
                // Search and stream the actual song from YouTube Music!
                let query = format!("{} {}", tit, art);
                if let Ok(page) = yt.search(&query) {
                    if let Some(first) = page.songs().first() {
                        load_artwork_async(win_art, first.video_id.clone());
                        if play_ytm_stream(&audio, &yt, &first.video_id, Some(dur_secs as f64)).is_ok() {
                            return;
                        }
                    }
                }
                audio.play_test_tone(440.0, 5.0);
            } else {
                if let Err(e) = play_ytm_stream(&audio, &yt, &video_id, Some(dur_secs as f64)) {
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
                    let _ = play_ytm_stream(&audio, &yt, &first.video_id, Some(200.0));
                }
            }
        });
    });

    // 16. Toggle Loudness Normalization & Peak Limiter
    let audio_loudness = Arc::clone(&audio);
    let win_loudness = win_weak.clone();
    window.on_toggle_loudness(move |enabled| {
        audio_loudness.set_loudness_enabled(enabled);
        audio_loudness.set_limiter_enabled(enabled);
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

    // 18. Switch to Mini Mode (Curved Capsule)
    let win_to_mini = win_weak.clone();
    window.on_switch_to_mini(move || {
        if let Some(w) = win_to_mini.upgrade() {
            w.set_app_mode("mini".into());
            w.window().set_size(slint::LogicalSize::new(420.0, 108.0));
            #[cfg(windows)]
            set_window_topmost(w.get_is_pinned());
        }
    });

    // 19. Switch to Normal Mode
    let win_to_normal = win_weak.clone();
    window.on_switch_to_normal(move || {
        if let Some(w) = win_to_normal.upgrade() {
            w.set_app_mode("normal".into());
            w.window().set_size(slint::LogicalSize::new(1320.0, 820.0));
            #[cfg(windows)]
            set_window_topmost(false);
        }
    });

    // 20. Toggle Pin (Always-on-top)
    let win_pin = win_weak.clone();
    window.on_toggle_pin(move || {
        if let Some(w) = win_pin.upgrade() {
            let next_pin = !w.get_is_pinned();
            w.set_is_pinned(next_pin);
            #[cfg(windows)]
            set_window_topmost(next_pin);
        }
    });

    // 21. Switch to Gaming Mode
    let win_gaming = win_weak.clone();
    window.on_switch_to_gaming(move || {
        if let Some(w) = win_gaming.upgrade() {
            w.set_app_mode("mini".into());
            w.window().set_size(slint::LogicalSize::new(420.0, 108.0));
            #[cfg(windows)]
            {
                set_window_topmost(true);
                trim_working_set();
            }
        }
    });

    // 22. Native Window Dragging for Mini Mode
    window.on_drag_window(move || {
        #[cfg(windows)]
        unsafe {
            #[link(name = "user32")]
            extern "system" {
                fn ReleaseCapture() -> i32;
                fn SendMessageW(hwnd: *mut std::ffi::c_void, msg: u32, wparam: usize, lparam: isize) -> isize;
                fn FindWindowW(lp_class_name: *const u16, lp_window_name: *const u16) -> *mut std::ffi::c_void;
            }
            let title_utf16: Vec<u16> = "AURI\0".encode_utf16().collect();
            let hwnd = FindWindowW(std::ptr::null(), title_utf16.as_ptr());
            if !hwnd.is_null() {
                ReleaseCapture();
                SendMessageW(hwnd, 0x00A1 /* WM_NCLBUTTONDOWN */, 2 /* HTCAPTION */, 0);
            }
        }
    });

    // 22. Timer: 10 FPS UI Progress & RAM synchronizer
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

#[cfg(test)]
mod playback_tests {
    use super::*;

    #[test]
    fn test_live_stream_playback_to_sink() {
        let engine = Arc::new(AudioEngine::new().expect("engine init"));
        let ytm = Arc::new(YtmClient::new(String::new()));
        let search = ytm.search("dhurandhar").expect("search");
        let song = search.songs().first().expect("song").clone();
        println!("Testing playback for: {} ({})", song.title, song.video_id);
        play_ytm_stream(&engine, &ytm, &song.video_id, Some(210.0)).expect("play_ytm_stream");

        let mut observed_progress = false;
        for i in 0..15 {
            std::thread::sleep(Duration::from_millis(300));
            let status = engine.get_status();
            println!("[tick {}] state={:?}, pos={:.2}s, is_decoding={}",
                i, status.state, status.position.as_secs_f64(), engine.is_decoding());
            if status.position.as_secs_f64() > 0.1 {
                observed_progress = true;
            }
        }
        assert!(observed_progress, "Hardware position should advance as audio plays");
    }

    #[test]
    fn test_toggle_loudness_and_limiter_during_playback() {
        let engine = Arc::new(AudioEngine::new().expect("engine init"));
        // Play tone
        engine.play_test_tone(440.0, 3.0);
        std::thread::sleep(Duration::from_millis(100));

        for _ in 0..5 {
            engine.set_loudness_enabled(false);
            engine.set_limiter_enabled(false);
            std::thread::sleep(Duration::from_millis(50));
            assert_eq!(engine.get_status().state, PlaybackState::Playing);

            engine.set_loudness_enabled(true);
            engine.set_limiter_enabled(true);
            std::thread::sleep(Duration::from_millis(50));
            assert_eq!(engine.get_status().state, PlaybackState::Playing);
        }
    }
}
