// AURI Storage & Persistence Manager
// Manages local state, persistent recently played history, and player layout preferences.

use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistentTrack {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: String,
    pub duration_secs: u64,
    pub format_badge: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub player_layout: String, // "bottom" (default) or "sidebar"
    pub volume: f32,
    pub loudness_enabled: bool,
    pub is_shuffle: bool,
    pub is_repeat: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            player_layout: "bottom".to_string(), // Tubefast bottom bar by default
            volume: 0.80,
            loudness_enabled: true,
            is_shuffle: false,
            is_repeat: false,
        }
    }
}

pub fn get_storage_dir() -> PathBuf {
    if let Some(mut dir) = dirs_or_env_appdata() {
        dir.push("AURI");
        let _ = fs::create_dir_all(&dir);
        return dir;
    }
    let fallback = PathBuf::from("data");
    let _ = fs::create_dir_all(&fallback);
    fallback
}

fn dirs_or_env_appdata() -> Option<PathBuf> {
    if let Ok(appdata) = std::env::var("APPDATA") {
        return Some(PathBuf::from(appdata));
    }
    if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
        return Some(PathBuf::from(localappdata));
    }
    if let Ok(home) = std::env::var("USERPROFILE") {
        return Some(PathBuf::from(home).join("AppData").join("Roaming"));
    }
    None
}

pub fn load_recent_history() -> Vec<PersistentTrack> {
    let path = get_storage_dir().join("recent.json");
    if let Ok(file) = File::open(&path) {
        let reader = BufReader::new(file);
        if let Ok(tracks) = serde_json::from_reader(reader) {
            return tracks;
        }
    }
    Vec::new()
}

pub fn save_recent_history(tracks: &[PersistentTrack]) {
    let path = get_storage_dir().join("recent.json");
    if let Ok(file) = File::create(&path) {
        let writer = BufWriter::new(file);
        let _ = serde_json::to_writer_pretty(writer, tracks);
    }
}

pub fn add_recent_track(track: PersistentTrack) -> Vec<PersistentTrack> {
    let mut history = load_recent_history();
    // Deduplicate by track ID
    history.retain(|t| t.id != track.id);
    // Prepend to top of history
    history.insert(0, track);
    // Keep max 100 recent items (matching Tubefast RECENT_LIMIT)
    history.truncate(100);
    save_recent_history(&history);
    history
}

pub fn clear_recent_history() {
    save_recent_history(&[]);
}

pub fn load_settings() -> AppSettings {
    let path = get_storage_dir().join("settings.json");
    if let Ok(file) = File::open(&path) {
        let reader = BufReader::new(file);
        if let Ok(settings) = serde_json::from_reader(reader) {
            return settings;
        }
    }
    AppSettings::default()
}

pub fn save_settings(settings: &AppSettings) {
    let path = get_storage_dir().join("settings.json");
    if let Ok(file) = File::create(&path) {
        let writer = BufWriter::new(file);
        let _ = serde_json::to_writer_pretty(writer, settings);
    }
}

