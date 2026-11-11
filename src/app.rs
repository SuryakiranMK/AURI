use crate::art::Art;
use crate::auth;
use crate::discord::{Discord, Headline};
use crate::hotkeys::{self, Combo, Command, Hotkeys};
use crate::media::Media;
use crate::player::Player;
use crate::ui;
use crate::update::{self, Plan};
use crate::ytm::{Account, Client, Item, Lyrics, Page, QUICK_PICKS, Section, VIDEO_FRAME_HOST};
use crate::{APP_NAME, VERSION};
use eframe::egui::{self, Key, Modifiers, PointerButton};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const RECENT_LIMIT: usize = 100;
const CACHED_PAGES: usize = 6;
const PAGES_KEPT_ON_TRIM: usize = 3;
const SEARCH_DEBOUNCE: Duration = Duration::from_millis(350);
const RESTART_AFTER_MS: u64 = 3000;
const NOTICE_TIME: Duration = Duration::from_secs(6);
const UPDATE_NOTICE_TIME: Duration = Duration::from_secs(20);
const LATEST_RELEASE: &str = concat!(env!("CARGO_PKG_REPOSITORY"), "/releases/latest");
const PLAYED_KEPT: usize = 50;
const PROGRESS_TICK: Duration = Duration::from_millis(250);
const RADIO_MARGIN: usize = 2;
const HOME_SEEDS: usize = 3;
const COVER_LOOKUPS: usize = 2;
const QUEUED_COVERS: usize = 30;
const REMEMBERED_COVERS: usize = 1500;
const WINDOW_KEY: &str = "window";
pub const WINDOW_SIZE: [f32; 2] = [1360.0, 860.0];
pub const WINDOW_MIN: [f32; 2] = [940.0, 600.0];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CustomPlaylist {
    pub id: String,
    pub name: String,
    pub songs: Vec<Item>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SettingsTab {
    #[default]
    General,
    Appearance,
    Playback,
    Equalizer,
    Performance,
    GamingMode,
    MiniMode,
    GlobalHotkeys,
    Discord,
    DataStorage,
    About,
}

#[allow(dead_code)]
impl SettingsTab {
    pub const ALL: [SettingsTab; 11] = [
        SettingsTab::General,
        SettingsTab::Appearance,
        SettingsTab::Playback,
        SettingsTab::Equalizer,
        SettingsTab::Performance,
        SettingsTab::GamingMode,
        SettingsTab::MiniMode,
        SettingsTab::GlobalHotkeys,
        SettingsTab::Discord,
        SettingsTab::DataStorage,
        SettingsTab::About,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            SettingsTab::General => "General",
            SettingsTab::Appearance => "Appearance",
            SettingsTab::Playback => "Playback",
            SettingsTab::Equalizer => "Equalizer",
            SettingsTab::Performance => "Performance",
            SettingsTab::GamingMode => "Gaming Mode",
            SettingsTab::MiniMode => "Mini Mode",
            SettingsTab::GlobalHotkeys => "Global Hotkeys",
            SettingsTab::Discord => "Discord",
            SettingsTab::DataStorage => "Data & storage",
            SettingsTab::About => "About",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
    System,
}

#[allow(dead_code)]
impl ThemeMode {
    pub const ALL: [ThemeMode; 3] = [
        ThemeMode::Dark,
        ThemeMode::Light,
        ThemeMode::System,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            ThemeMode::Dark => "Dark",
            ThemeMode::Light => "Light",
            ThemeMode::System => "System",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ThemePreset {
    #[default]
    Default,
    Catppuccin,
    TokyoNight,
    Caffeine,
    Neon,
    Breeze,
    Amoled,
}

#[allow(dead_code)]
impl ThemePreset {
    pub const ALL: [ThemePreset; 7] = [
        ThemePreset::Default,
        ThemePreset::Catppuccin,
        ThemePreset::TokyoNight,
        ThemePreset::Caffeine,
        ThemePreset::Neon,
        ThemePreset::Breeze,
        ThemePreset::Amoled,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            ThemePreset::Default => "Default",
            ThemePreset::Catppuccin => "Catppuccin",
            ThemePreset::TokyoNight => "Tokyo Night",
            ThemePreset::Caffeine => "Caffeine",
            ThemePreset::Neon => "Neon",
            ThemePreset::Breeze => "Breeze",
            ThemePreset::Amoled => "AMOLED",
        }
    }

    pub fn base_color(&self) -> eframe::egui::Color32 {
        match self {
            ThemePreset::Default => eframe::egui::Color32::from_rgb(10, 11, 14),
            ThemePreset::Catppuccin => eframe::egui::Color32::from_rgb(30, 30, 46),
            ThemePreset::TokyoNight => eframe::egui::Color32::from_rgb(26, 27, 38),
            ThemePreset::Caffeine => eframe::egui::Color32::from_rgb(30, 24, 21),
            ThemePreset::Neon => eframe::egui::Color32::from_rgb(8, 8, 15),
            ThemePreset::Breeze => eframe::egui::Color32::from_rgb(15, 23, 29),
            ThemePreset::Amoled => eframe::egui::Color32::from_rgb(0, 0, 0),
        }
    }

    pub fn panel_color(&self) -> eframe::egui::Color32 {
        match self {
            ThemePreset::Default => eframe::egui::Color32::from_rgb(7, 8, 10),
            ThemePreset::Catppuccin => eframe::egui::Color32::from_rgb(24, 24, 37),
            ThemePreset::TokyoNight => eframe::egui::Color32::from_rgb(22, 22, 30),
            ThemePreset::Caffeine => eframe::egui::Color32::from_rgb(23, 18, 16),
            ThemePreset::Neon => eframe::egui::Color32::from_rgb(12, 12, 22),
            ThemePreset::Breeze => eframe::egui::Color32::from_rgb(11, 18, 23),
            ThemePreset::Amoled => eframe::egui::Color32::from_rgb(8, 8, 8),
        }
    }

    pub fn default_accent(&self) -> MiniPalette {
        match self {
            ThemePreset::Default => MiniPalette::CyberViolet,
            ThemePreset::Catppuccin => MiniPalette::CyberViolet,
            ThemePreset::TokyoNight => MiniPalette::CobaltBlue,
            ThemePreset::Caffeine => MiniPalette::AmberGold,
            ThemePreset::Neon => MiniPalette::ElectricCyan,
            ThemePreset::Breeze => MiniPalette::ElectricCyan,
            ThemePreset::Amoled => MiniPalette::SunsetCrimson,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum StreamQuality {
    #[default]
    High,
    Low,
}

#[allow(dead_code)]
impl StreamQuality {
    pub const ALL: [StreamQuality; 2] = [
        StreamQuality::High,
        StreamQuality::Low,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            StreamQuality::High => "High Quality (160 kbps Opus)",
            StreamQuality::Low => "Data Saver (70 kbps Opus)",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Default)]
pub enum MiniPalette {
    #[default]
    CyberViolet,
    NeonEmerald,
    ElectricCyan,
    SunsetCrimson,
    AmberGold,
    CobaltBlue,
    SakuraPink,
    Monochrome,
}

impl MiniPalette {
    pub const ALL: [MiniPalette; 8] = [
        MiniPalette::CyberViolet,
        MiniPalette::NeonEmerald,
        MiniPalette::ElectricCyan,
        MiniPalette::SunsetCrimson,
        MiniPalette::AmberGold,
        MiniPalette::CobaltBlue,
        MiniPalette::SakuraPink,
        MiniPalette::Monochrome,
    ];

    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            MiniPalette::CyberViolet => "Cyber Violet",
            MiniPalette::NeonEmerald => "Neon Emerald",
            MiniPalette::ElectricCyan => "Electric Cyan",
            MiniPalette::SunsetCrimson => "Sunset Crimson",
            MiniPalette::AmberGold => "Amber Gold",
            MiniPalette::CobaltBlue => "Cobalt Blue",
            MiniPalette::SakuraPink => "Sakura Pink",
            MiniPalette::Monochrome => "Monochrome Glass",
        }
    }

    pub fn color(&self) -> eframe::egui::Color32 {
        match self {
            MiniPalette::CyberViolet => eframe::egui::Color32::from_rgb(139, 92, 246),
            MiniPalette::NeonEmerald => eframe::egui::Color32::from_rgb(16, 185, 129),
            MiniPalette::ElectricCyan => eframe::egui::Color32::from_rgb(6, 182, 212),
            MiniPalette::SunsetCrimson => eframe::egui::Color32::from_rgb(244, 63, 94),
            MiniPalette::AmberGold => eframe::egui::Color32::from_rgb(245, 158, 11),
            MiniPalette::CobaltBlue => eframe::egui::Color32::from_rgb(59, 130, 246),
            MiniPalette::SakuraPink => eframe::egui::Color32::from_rgb(236, 72, 153),
            MiniPalette::Monochrome => eframe::egui::Color32::from_rgb(226, 232, 240),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Default)]
pub enum MiniBackground {
    #[default]
    Obsidian,
    MidnightNavy,
    SlateCarbon,
    FrostedSlate,
    PureOled,
    MatchAccent,
}

impl MiniBackground {
    pub const ALL: [MiniBackground; 6] = [
        MiniBackground::Obsidian,
        MiniBackground::MidnightNavy,
        MiniBackground::SlateCarbon,
        MiniBackground::FrostedSlate,
        MiniBackground::PureOled,
        MiniBackground::MatchAccent,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            MiniBackground::Obsidian => "Obsidian",
            MiniBackground::MidnightNavy => "Midnight",
            MiniBackground::SlateCarbon => "Carbon",
            MiniBackground::FrostedSlate => "Frosted",
            MiniBackground::PureOled => "Pure OLED",
            MiniBackground::MatchAccent => "Accent Tint",
        }
    }

    pub fn color(&self, accent: eframe::egui::Color32) -> eframe::egui::Color32 {
        match self {
            MiniBackground::Obsidian => eframe::egui::Color32::from_rgb(12, 13, 20),
            MiniBackground::MidnightNavy => eframe::egui::Color32::from_rgb(10, 17, 36),
            MiniBackground::SlateCarbon => eframe::egui::Color32::from_rgb(24, 26, 33),
            MiniBackground::FrostedSlate => eframe::egui::Color32::from_rgb(36, 40, 52),
            MiniBackground::PureOled => eframe::egui::Color32::from_rgb(0, 0, 0),
            MiniBackground::MatchAccent => eframe::egui::Color32::from_rgb(
                (accent.r() / 8).max(8),
                (accent.g() / 8).max(8),
                (accent.b() / 8).max(12),
            ),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Default)]
pub enum ThemeBase {
    #[default]
    DarkObsidian,
    PureOled,
    MidnightNavy,
    CharcoalSlate,
}

#[allow(dead_code)]
impl ThemeBase {
    pub const ALL: [ThemeBase; 4] = [
        ThemeBase::DarkObsidian,
        ThemeBase::PureOled,
        ThemeBase::MidnightNavy,
        ThemeBase::CharcoalSlate,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            ThemeBase::DarkObsidian => "Dark Obsidian",
            ThemeBase::PureOled => "Pure OLED Black",
            ThemeBase::MidnightNavy => "Midnight Navy",
            ThemeBase::CharcoalSlate => "Charcoal Slate",
        }
    }

    pub fn color(&self) -> eframe::egui::Color32 {
        match self {
            ThemeBase::DarkObsidian => eframe::egui::Color32::from_rgb(10, 11, 14),
            ThemeBase::PureOled => eframe::egui::Color32::from_rgb(0, 0, 0),
            ThemeBase::MidnightNavy => eframe::egui::Color32::from_rgb(7, 10, 18),
            ThemeBase::CharcoalSlate => eframe::egui::Color32::from_rgb(18, 20, 28),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Default)]
pub enum PerfProfile {
    #[default]
    CompetitiveGaming,
    Balanced,
    Audiophile,
    Custom,
}

impl PerfProfile {
    pub const ALL: [PerfProfile; 4] = [
        PerfProfile::CompetitiveGaming,
        PerfProfile::Balanced,
        PerfProfile::Audiophile,
        PerfProfile::Custom,
    ];

    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            PerfProfile::CompetitiveGaming => "Competitive Gaming",
            PerfProfile::Balanced => "Balanced Desktop",
            PerfProfile::Audiophile => "Audiophile Max",
            PerfProfile::Custom => "Custom Profile",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Default)]
pub enum ArtworkBgVariant {
    #[default]
    CanvasTint,
    BlurredBackdrop,
    AmbientRadialGlow,
    MinimalEdgeGlow,
}

impl ArtworkBgVariant {
    pub const ALL: [ArtworkBgVariant; 4] = [
        ArtworkBgVariant::CanvasTint,
        ArtworkBgVariant::BlurredBackdrop,
        ArtworkBgVariant::AmbientRadialGlow,
        ArtworkBgVariant::MinimalEdgeGlow,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            ArtworkBgVariant::CanvasTint => "Canvas Tint",
            ArtworkBgVariant::BlurredBackdrop => "Frosted Blur",
            ArtworkBgVariant::AmbientRadialGlow => "Radial Glow",
            ArtworkBgVariant::MinimalEdgeGlow => "Edge Glow",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MiniVariant {
    #[default]
    ClassicHud,
    CapsulePill,
    StudioCard,
}

impl MiniVariant {
    pub const ALL: [MiniVariant; 3] = [
        MiniVariant::ClassicHud,
        MiniVariant::CapsulePill,
        MiniVariant::StudioCard,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            MiniVariant::ClassicHud => "Classic HUD",
            MiniVariant::CapsulePill => "Capsule Pill",
            MiniVariant::StudioCard => "Studio Card",
        }
    }

    pub fn size(&self, queue_open: bool) -> (f32, f32) {
        match self {
            MiniVariant::ClassicHud => (384.0, if queue_open { 260.0 } else { 108.0 }),
            MiniVariant::CapsulePill => (330.0, 68.0),
            MiniVariant::StudioCard => (280.0, if queue_open { 530.0 } else { 390.0 }),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BarLayout {
    #[default]
    Docked,
    FloatingPill,
    MiniCapsule,
}

impl BarLayout {
    pub const ALL: [BarLayout; 3] = [BarLayout::Docked, BarLayout::FloatingPill, BarLayout::MiniCapsule];

    pub fn name(&self) -> &'static str {
        match self {
            BarLayout::Docked => "Docked Full-Width",
            BarLayout::FloatingPill => "Floating Deck",
            BarLayout::MiniCapsule => "Trim Mini Capsule",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Route {
    Home,
    Search(String),
    Browse(String),
    Liked,
    #[allow(dead_code)]
    Recent,
    Stats,
    Settings(SettingsTab),
    CustomPlaylist(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum StatsPeriod {
    SevenDays,
    #[default]
    ThirtyDays,
    TwelveMonths,
    AllTime,
}

#[allow(dead_code)]
impl StatsPeriod {
    pub const ALL: [StatsPeriod; 4] = [
        StatsPeriod::SevenDays,
        StatsPeriod::ThirtyDays,
        StatsPeriod::TwelveMonths,
        StatsPeriod::AllTime,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            StatsPeriod::SevenDays => "7 days",
            StatsPeriod::ThirtyDays => "30 days",
            StatsPeriod::TwelveMonths => "12 months",
            StatsPeriod::AllTime => "All time",
        }
    }

    pub fn subtitle_period(&self) -> &'static str {
        match self {
            StatsPeriod::SevenDays => "in the last 7 days",
            StatsPeriod::ThirtyDays => "in the last 30 days",
            StatsPeriod::TwelveMonths => "in the last 12 months",
            StatsPeriod::AllTime => "all time",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DailyActivity {
    pub day_key: u32,
    pub listen_ms: u64,
    pub plays: u32,
    pub hourly_ms: [u32; 24],
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AlbumStat {
    pub title: String,
    pub artist: String,
    #[serde(default)]
    pub thumb: String,
    pub plays: u32,
    #[serde(default)]
    pub listen_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackStat {
    pub video_id: String,
    pub title: String,
    pub artist: String,
    #[serde(default)]
    pub thumb: String,
    #[serde(default)]
    pub duration: String,
    pub plays: u32,
    #[serde(default)]
    pub listen_ms: u64,
    pub last_played: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtistStat {
    pub artist: String,
    #[serde(default)]
    pub thumb: String,
    pub plays: u32,
    #[serde(default)]
    pub listen_ms: u64,
}

pub enum Load {
    Loading,
    Ready(Box<Page>),
    Failed(String),
}

#[derive(Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub enum Repeat {
    #[default]
    Off,
    All,
    One,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DiscordLineContent {
    #[default]
    Title,
    Artist,
    Album,
    ArtistAlbum,
    Off,
}

#[allow(dead_code)]
impl DiscordLineContent {
    pub const ALL: [DiscordLineContent; 5] = [
        DiscordLineContent::Title,
        DiscordLineContent::Artist,
        DiscordLineContent::Album,
        DiscordLineContent::ArtistAlbum,
        DiscordLineContent::Off,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            DiscordLineContent::Title => "Song Title",
            DiscordLineContent::Artist => "Artist",
            DiscordLineContent::Album => "Album",
            DiscordLineContent::ArtistAlbum => "Artist · Album",
            DiscordLineContent::Off => "Off",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DiscordStatusField {
    #[default]
    Line2Artist,
    Line1Title,
    Line3Album,
    ListeningToApp,
    Custom,
    None,
}

#[allow(dead_code)]
impl DiscordStatusField {
    pub const ALL: [DiscordStatusField; 6] = [
        DiscordStatusField::Line2Artist,
        DiscordStatusField::Line1Title,
        DiscordStatusField::Line3Album,
        DiscordStatusField::ListeningToApp,
        DiscordStatusField::Custom,
        DiscordStatusField::None,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            DiscordStatusField::Line2Artist => "Line 2 · Artist",
            DiscordStatusField::Line1Title => "Line 1 · Song title",
            DiscordStatusField::Line3Album => "Line 3 · Album",
            DiscordStatusField::ListeningToApp => "Listening to App",
            DiscordStatusField::Custom => "Custom status text",
            DiscordStatusField::None => "None",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DiscordButtonOption {
    #[default]
    ListenOnYtm,
    GetApp,
    None,
}

#[allow(dead_code)]
impl DiscordButtonOption {
    pub const ALL: [DiscordButtonOption; 3] = [
        DiscordButtonOption::ListenOnYtm,
        DiscordButtonOption::GetApp,
        DiscordButtonOption::None,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            DiscordButtonOption::ListenOnYtm => "Listen on YouTube Music",
            DiscordButtonOption::GetApp => "Get AURI",
            DiscordButtonOption::None => "None",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GameOverlayHotkey {
    #[default]
    CtrlShiftG,
    ShiftTab,
    F9,
    F12,
    CtrlF1,
}

#[allow(dead_code)]
impl GameOverlayHotkey {
    pub const ALL: [GameOverlayHotkey; 5] = [
        GameOverlayHotkey::CtrlShiftG,
        GameOverlayHotkey::ShiftTab,
        GameOverlayHotkey::F9,
        GameOverlayHotkey::F12,
        GameOverlayHotkey::CtrlF1,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            GameOverlayHotkey::CtrlShiftG => "Ctrl + Shift + G",
            GameOverlayHotkey::ShiftTab => "Shift + Tab",
            GameOverlayHotkey::F9 => "F9",
            GameOverlayHotkey::F12 => "F12",
            GameOverlayHotkey::CtrlF1 => "Ctrl + F1",
        }
    }

    pub fn matches(&self, input: &egui::InputState) -> bool {
        match self {
            GameOverlayHotkey::CtrlShiftG => input.modifiers.ctrl && input.modifiers.shift && input.key_pressed(egui::Key::G),
            GameOverlayHotkey::ShiftTab => input.modifiers.shift && input.key_pressed(egui::Key::Tab),
            GameOverlayHotkey::F9 => input.key_pressed(egui::Key::F9),
            GameOverlayHotkey::F12 => input.key_pressed(egui::Key::F12),
            GameOverlayHotkey::CtrlF1 => input.modifiers.ctrl && input.key_pressed(egui::Key::F1),
        }
    }
}

fn default_discord_line1() -> DiscordLineContent { DiscordLineContent::Title }
fn default_discord_line2() -> DiscordLineContent { DiscordLineContent::Artist }
fn default_discord_line3() -> DiscordLineContent { DiscordLineContent::Album }
fn default_discord_button2() -> DiscordButtonOption { DiscordButtonOption::GetApp }
fn default_game_overlay_opacity() -> f32 { 0.55 }
fn default_corner_roundness() -> f32 { 12.0 }
fn default_background_tint_shade() -> f32 { 0.0 }
fn default_game_whitelist() -> Vec<String> {
    vec![
        "cs2.exe".into(),
        "valorant.exe".into(),
        "dota2.exe".into(),
        "r5apex.exe".into(),
        "fortniteclient-win64-shipping.exe".into(),
        "overwatch.exe".into(),
        "gta5.exe".into(),
    ]
}
fn default_game_blacklist() -> Vec<String> {
    vec![
        "chrome.exe".into(),
        "msedge.exe".into(),
        "firefox.exe".into(),
        "discord.exe".into(),
        "code.exe".into(),
        "vlc.exe".into(),
        "explorer.exe".into(),
        "spotify.exe".into(),
        "auri.exe".into(),
    ]
}
fn default_game_music_pos() -> [f32; 2] { [60.0, 105.0] }
fn default_game_audio_pos() -> [f32; 2] { [60.0, 335.0] }
fn default_game_queue_pos() -> [f32; 2] { [760.0, 105.0] }

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Saved {
    pub volume: f32,
    pub liked: Vec<Item>,
    pub signed_out_liked: Option<Vec<Item>>,
    pub recent: Vec<Item>,
    pub pinned: Vec<Item>,
    pub visitor: String,
    pub queue_open: bool,
    pub shuffle: bool,
    pub repeat: Repeat,
    pub account: Option<Account>,
    pub covers: Covers,
    pub queue: Vec<Item>,
    pub current: Option<usize>,
    pub position_ms: u64,
    pub lyrics_open: bool,
    pub discord: bool,
    pub discord_headline: Headline,
    pub discord_name: String,
    pub mini_mode: bool,
    pub loudness_normalizer: bool,
    pub hotkeys: bool,
    pub hotkey_combos: [Option<Combo>; 5],
    pub auto_purge_ram: bool,
    #[serde(default = "default_false")]
    pub auto_game_mode: bool,
    #[serde(default = "default_false")]
    pub performance_mode: bool,
    #[serde(default = "default_true")]
    pub perf_aggressive_ram: bool,
    #[serde(default = "default_true")]
    pub perf_suspend_animations: bool,
    #[serde(default = "default_true")]
    pub perf_disable_blurs: bool,
    #[serde(default = "default_true")]
    pub perf_compact_buffers: bool,
    #[serde(default)]
    pub perf_placeholder_mood_art: bool,
    #[serde(default = "default_false")]
    pub eq_enabled: bool,
    #[serde(default = "default_eq_preset")]
    pub eq_preset: crate::dsp::EqPreset,
    #[serde(default = "default_eq_gains")]
    pub eq_gains: [f32; 10],
    #[serde(default)]
    pub eq_preamp_db: f32,
    #[serde(default)]
    pub autoeq_headphone: crate::dsp::AutoEqHeadphone,
    #[serde(default = "default_discord_line1")]
    pub discord_line1: DiscordLineContent,
    #[serde(default = "default_discord_line2")]
    pub discord_line2: DiscordLineContent,
    #[serde(default = "default_discord_line3")]
    pub discord_line3: DiscordLineContent,
    #[serde(default = "default_true")]
    pub discord_button_listen: bool,
    #[serde(default = "default_true")]
    pub discord_button_app: bool,
    #[serde(default = "default_false")]
    pub discord_show_when_paused: bool,
    #[serde(default = "default_false")]
    pub discord_hide_details: bool,
    #[serde(default)]
    pub discord_status_text: DiscordStatusField,
    #[serde(default)]
    pub discord_app_name: String,
    #[serde(default = "default_true")]
    pub discord_album_art: bool,
    #[serde(default = "default_true")]
    pub discord_provider_badge: bool,
    #[serde(default = "default_true")]
    pub discord_progress_bar: bool,
    #[serde(default = "default_false")]
    pub discord_line1_clickable: bool,
    #[serde(default = "default_false")]
    pub discord_line2_clickable: bool,
    #[serde(default = "default_false")]
    pub discord_art_clickable: bool,
    #[serde(default)]
    pub discord_button1: DiscordButtonOption,
    #[serde(default = "default_discord_button2")]
    pub discord_button2: DiscordButtonOption,

    #[serde(default)]
    pub game_overlay_hotkey: GameOverlayHotkey,
    #[serde(default = "default_game_overlay_opacity")]
    pub game_overlay_opacity: f32,
    #[serde(default = "default_true")]
    pub game_show_music_widget: bool,
    #[serde(default = "default_true")]
    pub game_show_audio_widget: bool,
    #[serde(default = "default_true")]
    pub game_show_queue_widget: bool,
    #[serde(default = "default_game_whitelist")]
    pub game_whitelist: Vec<String>,
    #[serde(default = "default_game_blacklist")]
    pub game_blacklist: Vec<String>,
    #[serde(default = "default_game_music_pos")]
    pub game_music_pos: [f32; 2],
    #[serde(default = "default_game_audio_pos")]
    pub game_audio_pos: [f32; 2],
    #[serde(default = "default_game_queue_pos")]
    pub game_queue_pos: [f32; 2],

    #[serde(default)]
    pub mini_palette: MiniPalette,
    #[serde(default = "default_mini_opacity")]
    pub mini_opacity: f32,
    #[serde(default = "default_true")]
    pub mini_show_shuffle_repeat: bool,
    #[serde(default = "default_true")]
    pub mini_show_queue: bool,
    #[serde(default = "default_true")]
    pub mini_show_volume: bool,
    #[serde(default = "default_true")]
    pub mini_show_clock: bool,
    #[serde(default = "default_true")]
    pub mini_always_on_top: bool,
    #[serde(default)]
    pub mini_bg: MiniBackground,
    #[serde(default = "default_true")]
    pub mini_grain: bool,
    #[serde(default = "default_mini_grain_strength")]
    pub mini_grain_strength: f32,

    #[serde(default)]
    pub theme_palette: MiniPalette,
    #[serde(default)]
    pub theme_base: ThemeBase,
    #[serde(default = "default_true")]
    pub ambient_glow: bool,
    #[serde(default)]
    pub theme_mode: ThemeMode,
    #[serde(default)]
    pub theme_preset: ThemePreset,
    #[serde(default = "default_corner_roundness")]
    pub corner_roundness: f32,
    #[serde(default = "default_background_tint_shade")]
    pub background_tint_shade: f32,
    #[serde(default = "default_false")]
    pub close_to_tray: bool,
    #[serde(default = "default_false")]
    pub song_notifications: bool,
    #[serde(default = "default_false")]
    pub launch_on_startup: bool,
    #[serde(default = "default_true")]
    pub history_enabled: bool,
    #[serde(default = "default_home_filter")]
    pub home_filter: String,

    #[serde(default = "default_storage_dir")]
    pub storage_dir: String,
    #[serde(default)]
    pub playlists: Vec<CustomPlaylist>,

    #[serde(default = "default_true")]
    pub selection_glow: bool,
    #[serde(default)]
    pub mini_variant: MiniVariant,
    #[serde(default = "default_false")]
    pub mini_locked: bool,

    #[serde(default)]
    pub bar_layout: BarLayout,
    #[serde(default = "default_true")]
    pub bar_glass: bool,
    #[serde(default = "default_bar_opacity")]
    pub bar_opacity: f32,
    #[serde(default = "default_false")]
    pub bar_grain: bool,
    #[serde(default = "default_bar_grain_strength")]
    pub bar_grain_strength: f32,

    #[serde(default = "default_true")]
    pub sidebar_open: bool,
    #[serde(default = "default_false")]
    pub queue_retractable: bool,
    #[serde(default = "default_false")]
    pub mini_capsule_retractable: bool,

    #[serde(default)]
    pub perf_profile: PerfProfile,
    #[serde(default = "default_perf_audio_buffer")]
    pub perf_audio_buffer_mb: f32,
    #[serde(default = "default_perf_art_cache")]
    pub perf_art_cache_mb: usize,
    #[serde(default = "default_perf_tick_fps")]
    pub perf_tick_fps: u32,
    #[serde(default)]
    pub sync_github_token: String,
    #[serde(default)]
    pub sync_gist_id: String,
    #[serde(default)]
    pub sync_room_code: String,

    #[serde(default)]
    pub stats_filter: StatsPeriod,
    #[serde(default)]
    pub stats_daily: Vec<DailyActivity>,
    #[serde(default)]
    pub stats_total_listen_ms: u64,
    #[serde(default)]
    pub stats_total_plays: u32,
    #[serde(default)]
    pub stats_top_tracks: Vec<TrackStat>,
    #[serde(default)]
    pub stats_top_artists: Vec<ArtistStat>,
    #[serde(default)]
    pub stats_top_albums: Vec<AlbumStat>,

    #[serde(default = "default_true")]
    pub lyrics_synced_mode: bool,
    #[serde(default = "default_lyrics_font_size")]
    pub lyrics_font_size: f32,
    #[serde(default = "default_true")]
    pub lyrics_align_center: bool,
    #[serde(default = "default_lyrics_dim_unsung")]
    pub lyrics_dim_unsung: f32,

    #[serde(default)]
    pub stream_quality: StreamQuality,
    #[serde(default = "default_false")]
    pub crossfade_enabled: bool,
    #[serde(default = "default_crossfade_secs")]
    pub crossfade_secs: u8,
    #[serde(default)]
    pub blocked_artists: Vec<String>,
    #[serde(default)]
    pub last_playlist_id: Option<String>,
    #[serde(default = "default_true")]
    pub ambient_artwork_blur: bool,
    #[serde(default)]
    pub artwork_bg_variant: ArtworkBgVariant,
    #[serde(default = "default_artwork_bg_opacity")]
    pub artwork_bg_opacity: f32,
    #[serde(default = "default_artwork_bg_blur")]
    pub artwork_bg_blur: f32,
}

fn default_artwork_bg_opacity() -> f32 {
    0.50
}

fn default_artwork_bg_blur() -> f32 {
    0.60
}

fn default_crossfade_secs() -> u8 {
    3
}

fn default_lyrics_font_size() -> f32 {
    24.0
}

fn default_lyrics_dim_unsung() -> f32 {
    0.35
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_eq_preset() -> crate::dsp::EqPreset {
    crate::dsp::EqPreset::Flat
}

fn default_eq_gains() -> [f32; 10] {
    [0.0; 10]
}

fn default_mini_opacity() -> f32 {
    0.92
}

fn default_mini_grain_strength() -> f32 {
    0.15
}

fn default_bar_opacity() -> f32 {
    0.95
}

fn default_bar_grain_strength() -> f32 {
    0.10
}

fn default_storage_dir() -> String {
    if let Some(user_profile) = std::env::var_os("USERPROFILE") {
        let p = std::path::PathBuf::from(user_profile).join("Music").join("AURI");
        return p.to_string_lossy().to_string();
    }
    "Music/AURI".to_string()
}

fn default_home_filter() -> String {
    "All".to_string()
}

fn default_perf_audio_buffer() -> f32 {
    1.5
}

fn default_perf_art_cache() -> usize {
    4
}

fn default_perf_tick_fps() -> u32 {
    1
}

pub enum Words {
    Loading,
    Ready(Lyrics),
    Missing,
    Failed,
}

pub struct Notice {
    pub text: String,
    pub action: Option<(&'static str, Action)>,
    until: Instant,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Covers {
    known: HashMap<String, String>,
    #[serde(skip)]
    wanted: RefCell<Vec<String>>,
}

impl Covers {
    pub fn of<'a>(&'a self, item: &'a Item) -> &'a str {
        if item.video_id.is_empty() || !item.thumb.contains(VIDEO_FRAME_HOST) {
            return &item.thumb;
        }
        match self.known.get(&item.video_id) {
            Some(cover) if !cover.is_empty() => cover,
            Some(_) => &item.thumb,
            None => {
                self.wanted.borrow_mut().push(item.video_id.clone());
                &item.thumb
            }
        }
    }
}

#[derive(Default)]
pub struct SignIn {
    pub cookie: String,
    pub busy: bool,
    pub status: String,
}

impl Default for Saved {
    fn default() -> Self {
        Self {
            volume: 0.8,
            liked: Vec::new(),
            signed_out_liked: None,
            recent: Vec::new(),
            pinned: Vec::new(),
            visitor: String::new(),
            queue_open: true,
            shuffle: false,
            repeat: Repeat::Off,
            account: None,
            covers: Covers::default(),
            queue: Vec::new(),
            current: None,
            position_ms: 0,
            lyrics_open: false,
            discord: false,
            discord_headline: Headline::App,
            discord_name: String::new(),
            mini_mode: false,
            loudness_normalizer: true,
            hotkeys: false,
            hotkey_combos: hotkeys::standard_combos(),
            auto_purge_ram: true,
            auto_game_mode: false,
            performance_mode: false,
            perf_aggressive_ram: true,
            perf_suspend_animations: true,
            perf_disable_blurs: true,
            perf_compact_buffers: true,
            perf_placeholder_mood_art: false,
            eq_enabled: false,
            eq_preset: crate::dsp::EqPreset::Flat,
            eq_gains: [0.0; 10],
            eq_preamp_db: 0.0,
            autoeq_headphone: crate::dsp::AutoEqHeadphone::None,
            discord_line1: DiscordLineContent::Title,
            discord_line2: DiscordLineContent::Artist,
            discord_line3: DiscordLineContent::Album,
            discord_button_listen: true,
            discord_button_app: true,
            discord_show_when_paused: false,
            discord_hide_details: false,
            discord_status_text: DiscordStatusField::Line2Artist,
            discord_app_name: String::new(),
            discord_album_art: true,
            discord_provider_badge: true,
            discord_progress_bar: true,
            discord_line1_clickable: false,
            discord_line2_clickable: false,
            discord_art_clickable: false,
            discord_button1: DiscordButtonOption::ListenOnYtm,
            discord_button2: DiscordButtonOption::GetApp,
            game_overlay_hotkey: GameOverlayHotkey::CtrlShiftG,
            game_overlay_opacity: 0.55,
            game_show_music_widget: true,
            game_show_audio_widget: true,
            game_show_queue_widget: true,
            game_whitelist: default_game_whitelist(),
            game_blacklist: default_game_blacklist(),
            game_music_pos: default_game_music_pos(),
            game_audio_pos: default_game_audio_pos(),
            game_queue_pos: default_game_queue_pos(),
            mini_palette: MiniPalette::CyberViolet,
            mini_opacity: 0.92,
            mini_show_shuffle_repeat: true,
            mini_show_queue: true,
            mini_show_volume: true,
            mini_show_clock: true,
            mini_always_on_top: true,
            mini_bg: MiniBackground::Obsidian,
            mini_grain: true,
            mini_grain_strength: 0.15,
            theme_palette: MiniPalette::CyberViolet,
            theme_base: ThemeBase::DarkObsidian,
            ambient_glow: true,
            theme_mode: ThemeMode::Dark,
            theme_preset: ThemePreset::Default,
            corner_roundness: 12.0,
            background_tint_shade: 0.0,
            close_to_tray: false,
            song_notifications: false,
            launch_on_startup: false,
            history_enabled: true,
            home_filter: default_home_filter(),
            storage_dir: default_storage_dir(),
            playlists: Vec::new(),
            selection_glow: true,
            mini_variant: MiniVariant::ClassicHud,
            mini_locked: false,
            bar_layout: BarLayout::Docked,
            bar_glass: true,
            bar_opacity: 0.95,
            bar_grain: false,
            bar_grain_strength: 0.10,
            sidebar_open: true,
            queue_retractable: false,
            mini_capsule_retractable: false,
            perf_profile: PerfProfile::CompetitiveGaming,
            perf_audio_buffer_mb: 1.5,
            perf_art_cache_mb: 4,
            perf_tick_fps: 1,
            sync_github_token: String::new(),
            sync_gist_id: String::new(),
            sync_room_code: String::new(),
            stats_filter: StatsPeriod::ThirtyDays,
            stats_daily: Vec::new(),
            stats_total_listen_ms: 0,
            stats_total_plays: 0,
            stats_top_tracks: Vec::new(),
            stats_top_artists: Vec::new(),
            stats_top_albums: Vec::new(),
            lyrics_synced_mode: true,
            lyrics_font_size: 24.0,
            lyrics_align_center: true,
            lyrics_dim_unsung: 0.35,
            stream_quality: StreamQuality::High,
            crossfade_enabled: false,
            crossfade_secs: 3,
            blocked_artists: Vec::new(),
            last_playlist_id: None,
            ambient_artwork_blur: true,
            artwork_bg_variant: ArtworkBgVariant::CanvasTint,
            artwork_bg_opacity: 0.50,
            artwork_bg_blur: 0.60,
        }
    }
}

impl Saved {
    fn leave_account(&mut self) {
        self.account = None;
        if let Some(liked) = self.signed_out_liked.take() {
            self.liked = liked;
        }
    }
}

fn share_link(item: &Item) -> String {
    if item.is_song() {
        return format!("https://music.youtube.com/watch?v={}", item.video_id);
    }
    if !item.browse_id.is_empty() {
        return format!("https://music.youtube.com/browse/{}", item.browse_id);
    }
    if !item.artist_id.is_empty() {
        return format!("https://music.youtube.com/channel/{}", item.artist_id);
    }
    if !item.album_id.is_empty() {
        return format!("https://music.youtube.com/browse/{}", item.album_id);
    }
    String::new()
}

#[derive(Clone)]
pub enum Action {
    Go(Route),
    Back,
    Forward,
    Reload,
    Play(Vec<Item>, usize),
    Jump(usize),
    PlayNext(Item),
    Enqueue(Item),
    Remove(usize),
    Toggle,
    Next,
    Previous,
    Seek(u64),
    Volume(f32),
    Like(Item),
    Share(Item),
    Pin(Item),
    ToggleQueue,
    ToggleSidebar,
    #[allow(dead_code)]
    ToggleQueueRetractable,
    ToggleMiniCapsuleRetractable,
    ToggleShuffle,
    CycleRepeat,
    OpenSignIn,
    CloseSignIn,
    BrowserSignIn,
    CookieSignIn,
    SignOut,
    SwitchAccount(Account),
    Undo,
    GetUpdate,
    OpenRelease,
    #[cfg_attr(not(any(windows, unix)), allow(dead_code))]
    Pause(bool),
    Move(usize, usize),
    Expand(bool),
    ShowLyrics(bool),
    ClearHistory,
    RestoreHistory(Vec<Item>),
    ShowOnDiscord(bool),
    #[allow(dead_code)]
    DiscordHeadline(Headline),
    #[allow(dead_code)]
    DiscordName(String),
    ToggleMiniMode,
    ToggleLoudness,
    PurgeMemory,
    VolumeBy(f32),
    GlobalHotkeys(bool),
    Record(Option<Command>),
    ResetHotkeys,
    #[allow(dead_code)]
    AutoPurgeRam(bool),
    TogglePerformanceMode,
    SetPerfAggressiveRam(bool),
    #[allow(dead_code)]
    SetPerfSuspendAnimations(bool),
    SetPerfDisableBlurs(bool),
    SetPerfCompactBuffers(bool),
    SetPerfPlaceholderMoodArt(bool),
    ToggleEq(bool),
    SetEqPreset(crate::dsp::EqPreset),
    SetEqGain(usize, f32),
    SetEqPreamp(f32),
    SetAutoEq(crate::dsp::AutoEqHeadphone),
    ResetEq,
    SetDiscordLine1(DiscordLineContent),
    SetDiscordLine2(DiscordLineContent),
    SetDiscordLine3(DiscordLineContent),
    #[allow(dead_code)]
    SetDiscordButtonListen(bool),
    #[allow(dead_code)]
    SetDiscordButtonApp(bool),
    #[allow(dead_code)]
    SetSettingsTab(SettingsTab),
    SetMiniPalette(MiniPalette),
    SetMiniOpacity(f32),
    SetMiniShowShuffleRepeat(bool),
    SetMiniShowQueue(bool),
    SetMiniShowVolume(bool),
    SetMiniShowClock(bool),
    SetMiniAlwaysOnTop(bool),
    ToggleMiniQueue,
    SetPerfProfile(PerfProfile),
    SetPerfAudioBuffer(f32),
    SetPerfArtCache(usize),
    SetPerfTickFps(u32),
    CreatePlaylist(String),
    DeletePlaylist(String),
    AddToPlaylist(String, Item),
    RemoveFromPlaylist(String, String),
    PromptCreatePlaylist,
    CancelCreatePlaylist,
    DownloadSong(Item),
    SetMiniBackground(MiniBackground),
    SetMiniGrain(bool),
    SetMiniGrainStrength(f32),
    SetThemePalette(MiniPalette),
    #[allow(dead_code)]
    SetThemeBase(ThemeBase),
    SetAmbientGlow(bool),
    SetAmbientArtworkBlur(bool),
    SetSelectionGlow(bool),
    SetMiniVariant(MiniVariant),
    ToggleMiniLock,
    CheckUpdate,
    SetBarLayout(BarLayout),
    SetBarGlass(bool),
    SetBarOpacity(f32),
    SetBarGrain(bool),
    SetBarGrainStrength(f32),
    #[allow(dead_code)]
    SetStorageDir(String),
    BrowseStorageDir,
    OpenStorageDir,
    ResetStorageDir,
    PurgeAudioCache,
    OpenCacheDir,
    StartSyncHost(String),
    StartSyncJoin(String),
    StopSync,
    SetSyncGithubToken(String),
    SetSyncGistId(String),
    SetLyricsSyncedMode(bool),
    SetLyricsFontSize(f32),
    SetLyricsAlignCenter(bool),
    SetLyricsDimUnsung(f32),
    ClearStats,
    SetStatsPeriod(StatsPeriod),
    #[allow(dead_code)]
    SetThemeMode(ThemeMode),
    #[allow(dead_code)]
    SetThemePreset(ThemePreset),
    #[allow(dead_code)]
    SetCloseToTray(bool),
    #[allow(dead_code)]
    SetSongNotifications(bool),
    #[allow(dead_code)]
    SetLaunchOnStartup(bool),
    #[allow(dead_code)]
    SetHistoryEnabled(bool),
    SetHomeFilter(String),
    SetStreamQuality(StreamQuality),
    SetCrossfadeEnabled(bool),
    SetCrossfadeSecs(u8),
    BlockArtist(String),
    UnblockArtist(String),
    OpenUpdateDialog,
    CloseUpdateDialog,
    StartRadio(Item),
    Dislike(Item),
    ToggleGameMode,
    ExitGameMode,
    ToggleGameOverlay,
    #[allow(dead_code)]
    ToggleGameHud,
    SetAutoGameMode(bool),
    SetGameOverlayHotkey(GameOverlayHotkey),
    SetGameOverlayOpacity(f32),
    ToggleGameMusicWidget,
    ToggleGameAudioWidget,
    ToggleGameQueueWidget,
    #[allow(dead_code)]
    AddGameWhitelist(String),
    #[allow(dead_code)]
    RemoveGameWhitelist(usize),
    #[allow(dead_code)]
    AddGameBlacklist(String),
    #[allow(dead_code)]
    RemoveGameBlacklist(usize),
    #[allow(dead_code)]
    SetGameMusicPos(f32, f32),
    #[allow(dead_code)]
    SetGameAudioPos(f32, f32),
    #[allow(dead_code)]
    SetGameQueuePos(f32, f32),
    #[allow(dead_code)]
    ResetGameWidgetPositions,
    ResetDiscordDefaults,
    SetDiscordShowWhenPaused(bool),
    SetDiscordHideDetails(bool),
    SetDiscordStatusText(DiscordStatusField),
    SetDiscordAppName(String),
    SetDiscordAlbumArt(bool),
    SetDiscordProviderBadge(bool),
    SetDiscordProgressBar(bool),
    SetDiscordLine1Clickable(bool),
    SetDiscordLine2Clickable(bool),
    SetDiscordArtClickable(bool),
    SetDiscordButton1(DiscordButtonOption),
    SetDiscordButton2(DiscordButtonOption),
    SetCornerRoundness(f32),
    SetBackgroundTintShade(f32),
    ResetThemeDefaults,
    ToggleSettingsPreview,
    SetArtworkBgVariant(ArtworkBgVariant),
    SetArtworkBgOpacity(f32),
    SetArtworkBgBlur(f32),
    ExitGameModeToMini,
}

enum Event {
    Page(Route, Result<Box<Page>, String>),
    Part(u64, usize, Vec<Section>),
    Radio(String, Vec<Item>),
    SignedIn(Result<(Account, String), String>),
    Library(Vec<Item>),
    Accounts(Vec<Account>),
    Liked(Vec<Item>),
    LikeRejected(Item, String),
    Cover(String, Option<String>),
    Release(String),
    Updated(Result<PathBuf, String>),
    Pressed(Action),
    Lyrics(String, Result<Option<Lyrics>, String>),
    Discord(bool),
    Toast(String),
    StorageDir(String),
    DownloadProgress { video_id: String, title: String, percent: f32 },
    DownloadDone { title: String, error: Option<String> },
    SyncPacket(crate::sync::SyncPacket),
}

#[derive(Clone, Debug)]
pub struct DownloadState {
    pub video_id: String,
    pub title: String,
    pub percent: f32,
}

pub struct App {
    ctx: egui::Context,
    yt: Arc<Client>,
    events: Receiver<Event>,
    sender: Sender<Event>,
    radio_seed: String,
    autoplay: Option<String>,
    home_parts: Vec<(usize, Vec<Section>)>,
    home_load: u64,
    geometry: Option<String>,
    cancel_sign_in: Arc<AtomicBool>,
    cover_queue: Vec<String>,
    cover_tried: HashSet<String>,
    cover_lookups: usize,
    media: Option<Media>,
    discord: Option<Discord>,
    hotkeys: Option<Hotkeys>,
    replaced: Option<(Vec<Item>, Option<usize>, u64)>,
    curated: bool,
    growing: Option<(Route, usize, usize)>,
    pub resume_at: Option<u64>,
    pub sign_in: Option<SignIn>,
    pub library: Vec<Item>,
    pub accounts: Vec<Account>,
    pub player: Player,
    pub art: Art,
    pub history: Vec<Route>,
    pub at: usize,
    pub pages: HashMap<Route, Load>,
    pub query: String,
    pub typed: Option<Instant>,
    pub focus_search: bool,
    pub expanded: bool,
    pub lyrics: Option<(String, Words)>,
    pub title_bar: ui::TitleBar,
    pub saved: Saved,
    pub notice: Option<Notice>,
    pub update: Option<String>,
    pub updating: bool,
    pub update_dialog_open: bool,
    pub discord_linked: Option<bool>,
    pub hotkey_taken: Vec<Command>,
    pub recording: Option<Command>,
    pub last_purge: Instant,
    pub mini_queue_open: bool,
    pub new_playlist_name: String,
    pub creating_playlist: bool,
    pub download_queue: std::collections::VecDeque<Item>,
    pub download_active: Option<DownloadState>,
    pub sync: Option<crate::sync::SyncSession>,
    pub sync_mode: crate::sync::SyncMode,
    pub sync_room: String,
    pub sync_last_publish: Option<Instant>,
    pub last_listen_tick: Option<Instant>,
    pub in_game_mode: bool,
    #[allow(dead_code)]
    pub game_hud_visible: bool,
    pub game_overlay_open: bool,
    pub game_mode_start_time: Option<Instant>,
    pub hotkey_was_down: bool,
    pub last_game_check: Instant,
    pub auto_game_detected: bool,
    pub original_queue: Option<Vec<Item>>,
    pub settings_preview_open: bool,
    restart: Option<PathBuf>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, query: Option<String>, autoplay: bool) -> Self {
        ui::install(&cc.egui_ctx);
        let mut saved: Saved = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, eframe::APP_KEY))
            .unwrap_or_default();
        saved.theme_mode = ThemeMode::Dark;
        saved.queue_open = true;
        // Readability floor: a nearly transparent glass HUD hides its controls
        // over light wallpapers, so lift older low-opacity configs once.
        saved.mini_opacity = saved.mini_opacity.clamp(0.55, 1.0);
        let yt = Arc::new(Client::new(saved.visitor.clone()));
        yt.set_session(auth::load());
        if !yt.signed_in() {
            saved.leave_account();
        }
        yt.act_as(saved.account.as_ref().map_or("", |account| &account.page));
        saved.current = saved.current.filter(|current| *current < saved.queue.len());
        let ctx = cc.egui_ctx.clone();
        let mut player = Player::new(yt.clone(), {
            let ctx = ctx.clone();
            move || ctx.request_repaint()
        });
        player.set_volume(saved.volume);
        player.set_loudness_normalizer(saved.loudness_normalizer);
        player.set_eq_enabled(saved.eq_enabled);
        player.set_eq_gains(saved.eq_gains);
        player.set_eq_preamp(saved.eq_preamp_db);
        let (sender, events) = mpsc::channel();
        let media = Media::new(cc, {
            let (sender, ctx) = (sender.clone(), ctx.clone());
            move |action| {
                let _ = sender.send(Event::Pressed(action));
                ctx.request_repaint();
            }
        });
        let discord = saved.discord.then(|| link_discord(&sender, &ctx));
        let mut app = Self {
            art: Art::new(ctx.clone(), yt.agent.clone()),
            ctx: ctx.clone(),
            yt,
            events,
            sender: sender.clone(),
            radio_seed: String::new(),
            autoplay: query.clone().filter(|_| autoplay),
            home_parts: Vec::new(),
            home_load: 0,
            geometry: cc.storage.and_then(|storage| storage.get_string(WINDOW_KEY)),
            cancel_sign_in: Arc::default(),
            cover_queue: Vec::new(),
            cover_tried: HashSet::new(),
            cover_lookups: 0,
            media,
            discord,
            hotkeys: None,
            replaced: None,
            curated: !saved.queue.is_empty(),
            growing: None,
            resume_at: saved.current.map(|_| saved.position_ms),
            sign_in: None,
            library: Vec::new(),
            accounts: Vec::new(),
            player,
            history: vec![Route::Home],
            at: 0,
            pages: HashMap::new(),
            query: String::new(),
            typed: None,
            focus_search: false,
            expanded: false,
            lyrics: None,
            title_bar: ui::TitleBar::new(cc),
            saved,
            notice: None,
            update: None,
            updating: false,
            update_dialog_open: false,
            discord_linked: None,
            hotkey_taken: Vec::new(),
            recording: None,
            last_purge: Instant::now(),
            mini_queue_open: false,
            new_playlist_name: String::new(),
            creating_playlist: false,
            download_queue: std::collections::VecDeque::new(),
            download_active: None,
            sync: {
                let (sync_sender, sync_ctx) = (sender.clone(), ctx.clone());
                Some(crate::sync::SyncSession::new(move |pkt| {
                    let _ = sync_sender.send(Event::SyncPacket(pkt));
                    sync_ctx.request_repaint();
                }))
            },
            sync_mode: crate::sync::SyncMode::Disconnected,
            sync_room: String::new(),
            sync_last_publish: None,
            last_listen_tick: None,
            in_game_mode: false,
            game_hud_visible: false,
            game_overlay_open: false,
            game_mode_start_time: None,
            hotkey_was_down: false,
            last_game_check: Instant::now(),
            auto_game_detected: false,
            original_queue: None,
            settings_preview_open: false,
            restart: None,
        };
        app.art.set_budget_bytes(app.saved.perf_art_cache_mb * 1024 * 1024);
        app.yt.set_low_quality(app.saved.stream_quality == StreamQuality::Low);
        if app.saved.mini_mode {
            app.title_bar.enable_transparency();
            app.ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
            app.ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(false));
            app.ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
            let (w, h) = app.saved.mini_variant.size(app.mini_queue_open);
            app.ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(w, h)));
            let level = if app.saved.mini_always_on_top { egui::WindowLevel::AlwaysOnTop } else { egui::WindowLevel::Normal };
            app.ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
        }
        match query {
            Some(query) => {
                app.query = query.clone();
                app.go(Route::Search(query));
            }
            None => {
                let init_route = match app.saved.last_playlist_id.as_deref() {
                    Some("liked") => Route::Liked,
                    Some(id) if app.saved.playlists.iter().any(|p| p.id == id) => Route::CustomPlaylist(id.to_string()),
                    _ => Route::Home,
                };
                app.history = vec![init_route.clone()];
                app.ensure(&init_route);
            }
        }
        app.restart_hotkeys();
        app.load_library();
        app.background(|yt| {
            yt.refresh_clients();
            let latest = yt.agent.head(LATEST_RELEASE).call().ok()?;
            newer_version(latest.get_url(), VERSION).map(Event::Release)
        });
        app
    }

    fn notify(&mut self, text: impl Into<String>) {
        self.offer(text, None, NOTICE_TIME);
    }

    fn offer(&mut self, text: impl Into<String>, action: Option<(&'static str, Action)>, lasts: Duration) {
        self.replaced = None;
        self.notice = Some(Notice {
            text: text.into(),
            action,
            until: Instant::now() + lasts,
        });
    }

    fn background(&self, work: impl FnOnce(&Client) -> Option<Event> + Send + 'static) {
        let (yt, sender, ctx) = (self.yt.clone(), self.sender.clone(), self.ctx.clone());
        std::thread::spawn(move || {
            if let Some(event) = work(&yt) {
                let _ = sender.send(event);
                ctx.request_repaint();
            }
        });
    }

    fn start_next_download(&mut self) {
        if self.download_active.is_some() {
            return;
        }
        let Some(item) = self.download_queue.pop_front() else {
            return;
        };
        self.download_active = Some(DownloadState {
            video_id: item.video_id.clone(),
            title: item.title.clone(),
            percent: 0.0,
        });
        let yt = self.yt.clone();
        let dir = PathBuf::from(&self.saved.storage_dir).join("downloads");
        let sender = self.sender.clone();
        let ctx = self.ctx.clone();
        let title = item.title.clone();
        let artist = item.subtitle.clone();
        let video_id = item.video_id.clone();

        std::thread::spawn(move || {
            let _ = std::fs::create_dir_all(&dir);
            let run_dl = || -> Result<(), String> {
                let stream = yt.stream(&video_id).map_err(|e| format!("Stream error: {e}"))?;
                let ext = if stream.opus { "opus" } else { "m4a" };
                let safe_name: String = format!("{} - {}", title, artist)
                    .chars()
                    .map(|c| match c {
                        '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                        _ => c,
                    })
                    .collect();
                let safe_name = safe_name.trim();
                let file_path = dir.join(format!("{}.{}", safe_name, ext));
                let response = yt.agent.get(&stream.url).call().map_err(|e| format!("Network error: {e}"))?;
                let total_bytes: u64 = response
                    .header("Content-Length")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let mut reader = response.into_reader();
                let mut file = std::fs::File::create(&file_path).map_err(|e| format!("File error: {e}"))?;

                let mut buffer = [0u8; 32768];
                let mut downloaded = 0u64;
                let mut last_pct = 0.0f32;
                loop {
                    use std::io::Read;
                    let n = reader.read(&mut buffer).map_err(|e| format!("Read error: {e}"))?;
                    if n == 0 {
                        break;
                    }
                    use std::io::Write;
                    file.write_all(&buffer[..n]).map_err(|e| format!("Write error: {e}"))?;
                    downloaded += n as u64;
                    let pct = if total_bytes > 0 {
                        (downloaded as f32 / total_bytes as f32).clamp(0.0, 1.0)
                    } else {
                        0.5
                    };
                    if (pct - last_pct).abs() >= 0.02 || pct >= 0.99 {
                        last_pct = pct;
                        let _ = sender.send(Event::DownloadProgress {
                            video_id: video_id.clone(),
                            title: title.clone(),
                            percent: pct,
                        });
                        ctx.request_repaint();
                    }
                }
                Ok(())
            };

            let err = run_dl().err();
            let _ = sender.send(Event::DownloadDone {
                title,
                error: err,
            });
            ctx.request_repaint();
        });
    }

    fn look_up_lyrics(&mut self) {
        if !self.expanded || !self.saved.lyrics_open {
            return;
        }
        let Some(song) = self.now_playing() else {
            return;
        };
        if self.lyrics.as_ref().is_some_and(|(known, _)| *known == song.video_id) {
            return;
        }
        if self.player.loading() {
            self.lyrics = None;
            return;
        }
        let (song, seconds) = (song.clone(), self.player.duration_ms() / 1000);
        self.lyrics = Some((song.video_id.clone(), Words::Loading));
        self.background(move |yt| {
            let found = yt.lyrics(&song, seconds);
            Some(Event::Lyrics(song.video_id, found))
        });
    }

    fn look_up_covers(&mut self) {
        for video_id in self.saved.covers.wanted.take() {
            if self.cover_tried.insert(video_id.clone()) {
                self.cover_queue.push(video_id);
            }
        }
        if self.cover_queue.len() > QUEUED_COVERS {
            let skipped = self.cover_queue.remove(0);
            self.cover_tried.remove(&skipped);
        }
        while self.cover_lookups < COVER_LOOKUPS {
            let Some(video_id) = self.cover_queue.pop() else { break };
            self.cover_lookups += 1;
            self.background(move |yt| {
                let cover = yt.cover(&video_id).ok();
                Some(Event::Cover(video_id, cover))
            });
        }
    }

    fn load_library(&self) {
        if self.yt.signed_in() {
            self.background(|yt| yt.library().ok().map(Event::Library));
            self.background(|yt| yt.liked().ok().map(Event::Liked));
            self.background(|yt| yt.accounts().ok().map(Event::Accounts));
        }
    }

    fn refresh(&mut self) {
        self.pages.clear();
        self.library.clear();
        self.ensure(&self.route().clone());
        self.load_library();
    }

    fn start_sign_in(&mut self, status: &str, cookie: impl FnOnce() -> Result<String, String> + Send + 'static) {
        if let Some(sign_in) = &mut self.sign_in {
            sign_in.busy = true;
            sign_in.status = status.to_owned();
        }
        self.background(move |yt| {
            let signed = cookie().and_then(|cookie| Ok((yt.sign_in(cookie.clone())?, cookie)));
            Some(Event::SignedIn(signed))
        });
    }

    fn sign_out(&mut self) {
        self.yt.set_session(None);
        self.yt.act_as("");
        self.accounts.clear();
        auth::forget();
        self.saved.leave_account();
        self.refresh();
    }

    fn home_page(&self, partial: bool) -> Page {
        let mut sections: Vec<Section> = self.home_parts.iter().flat_map(|(_, sections)| sections.iter().cloned()).collect();
        if let Some(at) = sections.iter().position(|section| section.title.eq_ignore_ascii_case("quick picks") || section.title == QUICK_PICKS) {
            sections[..=at].rotate_right(1);
        } else if let Some(at) = sections.iter().position(|section| section.items.iter().any(Item::is_song)) {
            sections[..=at].rotate_right(1);
            sections[0].title = QUICK_PICKS.to_owned();
        }
        let filter = self.saved.home_filter.trim();
        if !filter.is_empty() && !filter.eq_ignore_ascii_case("all") {
            let filter_lower = filter.to_lowercase();
            sections.retain(|section| {
                section.title.to_lowercase().contains(&filter_lower)
                    || section.items.iter().any(|item| item.title.to_lowercase().contains(&filter_lower) || item.subtitle.to_lowercase().contains(&filter_lower))
            });
        }
        Page {
            header: None,
            sections,
            partial,
        }
    }

    pub fn route(&self) -> &Route {
        &self.history[self.at]
    }

    pub fn now_playing(&self) -> Option<&Item> {
        self.saved.queue.get(self.saved.current?)
    }

    pub fn playing(&self) -> bool {
        self.saved.current.is_some() && !self.player.paused()
    }

    fn ensure(&mut self, route: &Route) {
        if self.pages.contains_key(route) || matches!(route, Route::Liked | Route::Recent | Route::Stats | Route::Settings(_) | Route::CustomPlaylist(_)) {
            return;
        }
        if self.pages.len() >= CACHED_PAGES {
            let kept = &self.history[self.history.len().saturating_sub(PAGES_KEPT_ON_TRIM)..];
            self.pages.retain(|cached, _| kept.contains(cached));
        }
        self.pages.insert(route.clone(), Load::Loading);
        let mut seeds: Vec<Item> = Vec::new();
        let wanted = if self.yt.signed_in() { 0 } else { HOME_SEEDS };
        for item in self.saved.recent.iter().chain(&self.saved.liked) {
            if seeds.len() < wanted && !seeds.iter().any(|seed| seed.subtitle == item.subtitle) {
                seeds.push(item.clone());
            }
        }
        if *route == Route::Home {
            self.home_load += 1;
            self.home_parts.clear();
        }
        let load = self.home_load;
        let (yt, sender, ctx, route) = (self.yt.clone(), self.sender.clone(), self.ctx.clone(), route.clone());
        std::thread::spawn(move || {
            let emit = |order: usize, sections: Vec<Section>| {
                let _ = sender.send(Event::Part(load, order, sections));
                ctx.request_repaint();
            };
            let result = match &route {
                Route::Home => yt.home(&seeds, &emit).map(|()| Page::default()),
                Route::Search(query) => yt.search(query),
                Route::Browse(id) => yt.browse(id, &|page| {
                    let _ = sender.send(Event::Page(route.clone(), Ok(Box::new(page.clone()))));
                    ctx.request_repaint();
                }),
                Route::Liked | Route::Recent | Route::Stats | Route::Settings(_) | Route::CustomPlaylist(_) => return,
            };
            let _ = sender.send(Event::Page(route, result.map(Box::new)));
            ctx.request_repaint();
        });
    }

    fn go(&mut self, route: Route) {
        if *self.route() == route {
            return;
        }
        if matches!(route, Route::Settings(SettingsTab::Discord | SettingsTab::Appearance | SettingsTab::GamingMode)) && self.saved.queue_open {
            self.saved.queue_open = false;
            self.notify("Queue docked to expand view");
        }
        if matches!((self.route(), &route), (Route::Search(_), Route::Search(_))) {
            self.history[self.at] = route;
        } else {
            self.history.truncate(self.at + 1);
            self.history.push(route);
            self.at += 1;
        }
        self.ensure(&self.route().clone());
    }

    fn step(&mut self, at: usize) {
        self.at = at;
        let route = self.route().clone();
        if let Route::Search(query) = &route {
            self.query = query.clone();
        }
        self.ensure(&route);
    }

    fn growing_list(&self, items: &[Item]) -> Option<(Route, usize, usize)> {
        let Some(Load::Ready(page)) = self.pages.get(self.route()) else {
            return None;
        };
        let first = items.first().map(|item| &item.video_id);
        let at = page
            .sections
            .iter()
            .position(|section| section.items.first().map(|item| &item.video_id) == first)?;
        page.partial.then(|| (self.route().clone(), at, page.sections[at].items.len()))
    }

    fn grow_queue(&mut self, route: &Route, page: &Page) {
        let Some((from, at, offered)) = &mut self.growing else { return };
        if from != route {
            return;
        }
        let Some(section) = page.sections.get(*at) else { return };
        let fresh = section.items.iter().skip(*offered).filter(|item| item.is_song()).cloned();
        *offered = section.items.len();
        self.saved.queue.extend(fresh);
        if self.saved.shuffle {
            let upcoming = self.saved.current.map_or(0, |current| current + 1).min(self.saved.queue.len());
            shuffle(&mut self.saved.queue[upcoming..]);
        }
        if !page.partial {
            self.growing = None;
        }
    }

    fn play(&mut self, items: Vec<Item>, index: usize) {
        let chosen = items.get(index).map(|item| item.video_id.clone()).unwrap_or_default();
        let songs: Vec<Item> = items.into_iter().filter(Item::is_song).collect();
        self.growing = None;
        let before = std::mem::replace(&mut self.saved.queue, songs);
        let upcoming = self.saved.current.map_or(0, |current| current + 1);
        let same_list = before.len() == self.saved.queue.len()
            && before
                .iter()
                .all(|old| self.saved.queue.iter().any(|new| new.video_id == old.video_id));
        if self.curated && upcoming < before.len() && !same_list {
            let position = self.resume_at.unwrap_or_else(|| self.player.position_ms());
            self.offer("Queue replaced", Some(("Undo", Action::Undo)), NOTICE_TIME);
            self.replaced = Some((before, self.saved.current, position));
        }
        self.curated = self.saved.queue.len() > 1;
        let mut at = self.saved.queue.iter().position(|item| item.video_id == chosen).unwrap_or(0);
        let original = self.saved.queue.clone();
        if self.saved.shuffle && !self.saved.queue.is_empty() {
            self.original_queue = Some(original);
            self.saved.queue.swap(0, at);
            at = 0;
            shuffle(&mut self.saved.queue[1..]);
        } else {
            self.original_queue = None;
        }
        self.radio_seed.clear();
        self.play_index(at);
    }

    fn play_index(&mut self, index: usize) {
        if index >= self.saved.queue.len() {
            return;
        }
        let played = if self.saved.repeat == Repeat::All {
            0
        } else {
            index.saturating_sub(PLAYED_KEPT)
        };
        self.saved.queue.drain(..played);
        let index = index - played;
        let item = self.saved.queue[index].clone();
        self.resume_at = None;
        self.saved.current = Some(index);
        self.player.load(&item.video_id);
        if self.saved.performance_mode && self.saved.perf_aggressive_ram {
            self.apply(Action::PurgeMemory);
        }
        if let Some(next) = self.saved.queue.get(index + 1) {
            self.player.preload(&next.video_id);
        }
        if index + RADIO_MARGIN >= self.saved.queue.len() && self.radio_seed != item.video_id {
            self.radio_seed = item.video_id.clone();
            let (yt, sender, ctx, seed) = (self.yt.clone(), self.sender.clone(), self.ctx.clone(), item.video_id.clone());
            std::thread::spawn(move || {
                if let Ok(items) = yt.radio(&seed) {
                    let _ = sender.send(Event::Radio(seed, items));
                    ctx.request_repaint();
                }
            });
        }
        self.record_stat_play(&item);
        self.saved.recent.retain(|recent| recent.video_id != item.video_id);
        self.saved.recent.insert(0, item);
        self.saved.recent.truncate(RECENT_LIMIT);
    }

    pub fn record_stat_play(&mut self, item: &Item) {
        self.saved.stats_total_plays += 1;
        let now = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d.as_secs(),
            Err(_) => 0,
        };
        let day_key = (now / 86400) as u32;
        let hour = ((now / 3600) % 24) as usize;
        if let Some(day) = self.saved.stats_daily.iter_mut().find(|d| d.day_key == day_key) {
            day.plays += 1;
            day.hourly_ms[hour] = day.hourly_ms[hour].saturating_add(1000);
        } else {
            let mut hourly_ms = [0u32; 24];
            hourly_ms[hour] = 1000;
            self.saved.stats_daily.push(DailyActivity {
                day_key,
                listen_ms: 0,
                plays: 1,
                hourly_ms,
            });
        }
        let cover_str = self.saved.covers.of(item).to_owned();

        if let Some(pos) = self.saved.stats_top_tracks.iter().position(|t| t.video_id == item.video_id) {
            self.saved.stats_top_tracks[pos].plays += 1;
            self.saved.stats_top_tracks[pos].last_played = now;
            if self.saved.stats_top_tracks[pos].thumb.is_empty() {
                self.saved.stats_top_tracks[pos].thumb = cover_str.clone();
            }
            if self.saved.stats_top_tracks[pos].duration.is_empty() {
                self.saved.stats_top_tracks[pos].duration = item.duration.clone();
            }
        } else {
            self.saved.stats_top_tracks.push(TrackStat {
                video_id: item.video_id.clone(),
                title: item.title.clone(),
                artist: item.subtitle.clone(),
                thumb: cover_str.clone(),
                duration: item.duration.clone(),
                plays: 1,
                listen_ms: 0,
                last_played: now,
            });
        }
        self.saved.stats_top_tracks.sort_by(|a, b| b.plays.cmp(&a.plays));
        self.saved.stats_top_tracks.truncate(50);

        if !item.subtitle.is_empty() {
            if let Some(pos) = self.saved.stats_top_artists.iter().position(|a| a.artist == item.subtitle) {
                self.saved.stats_top_artists[pos].plays += 1;
                if self.saved.stats_top_artists[pos].thumb.is_empty() {
                    self.saved.stats_top_artists[pos].thumb = cover_str.clone();
                }
            } else {
                self.saved.stats_top_artists.push(ArtistStat {
                    artist: item.subtitle.clone(),
                    thumb: cover_str.clone(),
                    plays: 1,
                    listen_ms: 0,
                });
            }
            self.saved.stats_top_artists.sort_by(|a, b| b.plays.cmp(&a.plays));
            self.saved.stats_top_artists.truncate(30);
        }

        if !item.extra.is_empty() {
            let album_name = item.extra.clone();
            if let Some(pos) = self.saved.stats_top_albums.iter().position(|a| a.title == album_name) {
                self.saved.stats_top_albums[pos].plays += 1;
                if self.saved.stats_top_albums[pos].thumb.is_empty() {
                    self.saved.stats_top_albums[pos].thumb = cover_str;
                }
            } else {
                self.saved.stats_top_albums.push(AlbumStat {
                    title: album_name,
                    artist: item.subtitle.clone(),
                    thumb: cover_str,
                    plays: 1,
                    listen_ms: 0,
                });
            }
            self.saved.stats_top_albums.sort_by(|a, b| b.plays.cmp(&a.plays));
            self.saved.stats_top_albums.truncate(20);
        }
    }

    fn next(&mut self, automatic: bool) {
        let Some(current) = self.saved.current else { return };
        let mut idx = current + 1;
        while idx < self.saved.queue.len() {
            let item = &self.saved.queue[idx];
            let is_blocked = self.saved.blocked_artists.iter().any(|b| {
                item.subtitle.to_lowercase().contains(&b.to_lowercase())
            });
            if !is_blocked {
                self.play_index(idx);
                return;
            }
            idx += 1;
        }
        if self.saved.repeat == Repeat::All {
            for i in 0..self.saved.queue.len() {
                let item = &self.saved.queue[i];
                let is_blocked = self.saved.blocked_artists.iter().any(|b| {
                    item.subtitle.to_lowercase().contains(&b.to_lowercase())
                });
                if !is_blocked {
                    self.play_index(i);
                    return;
                }
            }
        } else if automatic {
            self.player.seek(0);
            self.player.set_paused(true);
        }
    }

    fn previous(&mut self) {
        match self.saved.current {
            Some(current) if current > 0 && self.player.position_ms() < RESTART_AFTER_MS => self.play_index(current - 1),
            Some(_) => self.player.seek(0),
            None => {}
        }
    }

    fn drain(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.events.try_recv() {
            match event {
                Event::Part(load, order, sections) if load == self.home_load => {
                    self.home_parts.push((order, sections));
                    self.home_parts.sort_by_key(|(order, _)| *order);
                    self.pages.insert(Route::Home, Load::Ready(Box::new(self.home_page(true))));
                }
                Event::Part(..) => {}
                Event::Page(Route::Home, result) => {
                    let page = self.home_page(false);
                    let load = match result {
                        Err(message) if page.sections.is_empty() => Load::Failed(message),
                        _ => Load::Ready(Box::new(page)),
                    };
                    self.pages.insert(Route::Home, load);
                }
                Event::Page(route, Ok(page)) => {
                    let wanted = matches!(&route, Route::Search(query) if self.autoplay.as_ref() == Some(query));
                    let first = page.songs().into_iter().next().filter(|_| wanted);
                    self.grow_queue(&route, &page);
                    self.pages.insert(route, Load::Ready(page));
                    if let Some(song) = first {
                        self.autoplay = None;
                        self.play(vec![song], 0);
                    }
                }
                Event::Page(route, Err(message)) => drop(self.pages.insert(route, Load::Failed(message))),
                Event::Radio(seed, items) if seed == self.radio_seed => {
                    let upcoming = self.saved.current.map_or(0, |current| current + 1);
                    let had_next = upcoming < self.saved.queue.len();
                    for item in items {
                        let is_blocked = self.saved.blocked_artists.iter().any(|b| {
                            item.subtitle.to_lowercase().contains(&b.to_lowercase())
                        });
                        if !is_blocked && !self.saved.queue.iter().any(|queued| queued.video_id == item.video_id) {
                            self.saved.queue.push(item);
                        }
                    }
                    if let (false, Some(next)) = (had_next, self.saved.queue.get(upcoming)) {
                        self.player.preload(&next.video_id);
                    }
                }
                Event::Radio(..) => {}
                Event::Library(items) => self.library = items,
                Event::Accounts(accounts) if self.yt.signed_in() => self.accounts = accounts,
                Event::Accounts(_) => {}
                Event::Liked(items) if self.yt.signed_in() => {
                    let shown = std::mem::replace(&mut self.saved.liked, items);
                    self.saved.signed_out_liked.get_or_insert(shown);
                }
                Event::Liked(_) => {}
                Event::LikeRejected(item, message) => {
                    if self.yt.signed_in() {
                        toggle(&mut self.saved.liked, item);
                    }
                    self.notify(message);
                }
                Event::Cover(video_id, cover) => {
                    self.cover_lookups -= 1;
                    if self.saved.covers.known.len() >= REMEMBERED_COVERS {
                        self.saved.covers.known.clear();
                    }
                    if let Some(cover) = cover {
                        self.saved.covers.known.insert(video_id, cover);
                    }
                }
                Event::SignedIn(Ok((account, cookie))) if self.sign_in.is_some() => {
                    if let Err(message) = auth::store(&cookie) {
                        self.notify(message);
                    }
                    self.saved.account = Some(account);
                    self.sign_in = None;
                    self.refresh();
                }
                Event::SignedIn(Ok(_)) => self.yt.set_session(None),
                Event::SignedIn(Err(message)) => {
                    if let Some(sign_in) = &mut self.sign_in {
                        sign_in.busy = false;
                        sign_in.status = message;
                    }
                }
                Event::Pressed(action) => self.apply(action),
                Event::Discord(linked) => self.discord_linked = Some(linked),
                Event::Lyrics(video_id, found) => {
                    if let Some((_, words)) = self.lyrics.as_mut().filter(|(wanted, _)| *wanted == video_id) {
                        *words = match found {
                            Ok(Some(lyrics)) => Words::Ready(lyrics),
                            Ok(None) => Words::Missing,
                            Err(_) => Words::Failed,
                        };
                    }
                }
                Event::Release(version) => {
                    self.offer(
                        format!("{APP_NAME} {version} is out"),
                        Some(("Update", Action::OpenUpdateDialog)),
                        UPDATE_NOTICE_TIME,
                    );
                    self.update = Some(version);
                    self.update_dialog_open = true;
                }
                Event::Updated(Ok(program)) => {
                    self.restart = Some(program);
                    self.ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                Event::Updated(Err(reason)) => {
                    self.updating = false;
                    let page = Some(("Download page", Action::OpenRelease));
                    self.offer(format!("The update did not install: {reason}"), page, UPDATE_NOTICE_TIME);
                }
                Event::Toast(msg) => self.notify(msg),
                Event::StorageDir(dir) => {
                    self.saved.storage_dir = dir.clone();
                    self.notify(format!("Storage directory set to: {}", dir));
                }
                Event::DownloadProgress { video_id, title, percent } => {
                    if let Some(active) = &mut self.download_active {
                        if active.video_id == video_id {
                            active.percent = percent;
                        }
                    } else {
                        self.download_active = Some(DownloadState { video_id, title, percent });
                    }
                    self.ctx.request_repaint();
                }
                Event::DownloadDone { title, error } => {
                    if let Some(err) = error {
                        self.notify(format!("Download failed for '{}': {}", title, err));
                    } else {
                        self.notify(format!("Downloaded '{}'", title));
                    }
                    self.download_active = None;
                    self.start_next_download();
                    self.ctx.request_repaint();
                }
                Event::SyncPacket(pkt) => {
                    if self.sync_mode == crate::sync::SyncMode::Guest {
                        let current_id = self.saved.current.and_then(|c| self.saved.queue.get(c)).map(|it| it.video_id.clone());
                        if current_id.as_deref() != Some(&pkt.video_id) && !pkt.video_id.is_empty() {
                            let item = Item {
                                video_id: pkt.video_id.clone(),
                                title: pkt.title.clone(),
                                subtitle: pkt.subtitle.clone(),
                                ..Default::default()
                            };
                            self.apply(Action::Play(vec![item], 0));
                        }
                        if pkt.playing != self.playing() {
                            self.apply(Action::Pause(!pkt.playing));
                        }
                        if pkt.playing {
                            let now_ms = crate::sync::now_timestamp_ms();
                            let drift = now_ms.saturating_sub(pkt.timestamp_ms);
                            let target_pos = pkt.position_ms + drift;
                            let cur_pos = self.player.position_ms();
                            if (target_pos as i64 - cur_pos as i64).abs() > 2500 {
                                self.player.seek(target_pos);
                            }
                        }
                    }
                }
            }
        }
        if self.yt.take_expired() {
            self.sign_out();
            self.notify("Your YouTube session ended. Sign in again.");
        }
        self.player.follow_default_device();
        if let Some(message) = self.player.take_error() {
            self.player.set_paused(true);
            self.notify(message);
        }
        if self.player.take_ended() {
            if self.saved.repeat == Repeat::One {
                self.player.seek(0);
            } else {
                self.next(true);
            }
        }
        if let Some(typed) = self.typed {
            if typed.elapsed() >= SEARCH_DEBOUNCE {
                self.typed = None;
                let query = self.query.trim().to_owned();
                if !query.is_empty() {
                    self.go(Route::Search(query));
                }
            } else {
                ctx.request_repaint_after(SEARCH_DEBOUNCE - typed.elapsed());
            }
        }
        if let Some(notice) = &self.notice {
            let left = notice.until.saturating_duration_since(Instant::now());
            if left.is_zero() {
                self.notice = None;
                self.replaced = None;
            } else {
                ctx.request_repaint_after(left);
            }
        }
    }

    fn restart_hotkeys(&mut self) {
        self.hotkeys = None;
        self.hotkey_taken.clear();
        if !self.saved.hotkeys || self.recording.is_some() {
            return;
        }
        let bindings = Command::ALL
            .into_iter()
            .filter_map(|command| Some((command, self.saved.hotkey_combos[command as usize]?)))
            .collect();
        let (sender, ctx) = (self.sender.clone(), self.ctx.clone());
        let press = move |command: Command| {
            let _ = sender.send(Event::Pressed(command.action()));
            ctx.request_repaint();
        };
        if let Some((hotkeys, taken)) = Hotkeys::start(bindings, press) {
            self.hotkeys = Some(hotkeys);
            self.hotkey_taken = taken;
        }
    }

    fn set_recording(&mut self, command: Option<Command>) {
        if self.recording != command {
            self.recording = command;
            self.restart_hotkeys();
        }
    }

    fn bind(&mut self, command: Command, combo: Option<Combo>) {
        for other in &mut self.saved.hotkey_combos {
            if combo.is_some() && *other == combo {
                *other = None;
            }
        }
        self.saved.hotkey_combos[command as usize] = combo;
        self.restart_hotkeys();
    }

    fn record(&mut self, ctx: &egui::Context) -> bool {
        if !matches!(*self.route(), Route::Settings(_)) {
            self.set_recording(None);
        }
        let Some(command) = self.recording else {
            return false;
        };
        let pressed = ctx.input_mut(|input| {
            let pressed = input.events.iter().find_map(|event| match event {
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => Some((*key, *modifiers)),
                _ => None,
            });
            input
                .events
                .retain(|event| !matches!(event, egui::Event::Key { .. } | egui::Event::Text(_)));
            pressed
        });
        match pressed {
            Some((Key::Escape, _)) => self.set_recording(None),
            Some((Key::Backspace | Key::Delete, _)) => {
                self.set_recording(None);
                self.bind(command, None);
            }
            Some((key, modifiers)) => {
                if let Some(combo) = Combo::from_input(key, modifiers) {
                    self.set_recording(None);
                    self.bind(command, Some(combo));
                }
            }
            None => {}
        }
        true
    }

    fn shortcuts(&mut self, ctx: &egui::Context) -> Vec<Action> {
        let mut out = Vec::new();
        let free = !ctx.wants_keyboard_input() && ctx.memory(|memory| memory.focused().is_none());
        ctx.input_mut(|input| {
            if input.consume_key(Modifiers::COMMAND, Key::K) || (free && input.consume_key(Modifiers::NONE, Key::Slash)) {
                self.focus_search = true;
            }
            if input.consume_key(Modifiers::ALT, Key::ArrowLeft) || input.pointer.button_pressed(PointerButton::Extra1) {
                out.push(Action::Back);
            }
            if input.consume_key(Modifiers::ALT, Key::ArrowRight) || input.pointer.button_pressed(PointerButton::Extra2) {
                out.push(Action::Forward);
            }
            if input.consume_key(Modifiers::COMMAND, Key::R) || input.consume_key(Modifiers::NONE, Key::F5) {
                out.push(Action::Reload);
            }
            if input.consume_key(Modifiers::COMMAND, Key::ArrowRight) {
                out.push(Action::Next);
            }
            if input.consume_key(Modifiers::COMMAND, Key::ArrowLeft) {
                out.push(Action::Previous);
            }
            if free && input.consume_key(Modifiers::NONE, Key::Space) {
                out.push(Action::Toggle);
            }
            if free && input.consume_key(Modifiers::COMMAND, Key::Z) {
                out.push(Action::Undo);
            }
            if self.expanded && free && input.consume_key(Modifiers::NONE, Key::Escape) {
                out.push(Action::Expand(false));
            }
        });
        out
    }

    fn apply(&mut self, action: Action) {
        match action {
            Action::Go(route) => {
                self.expanded = false;
                self.go(route);
            }
            Action::Back if self.expanded => self.expanded = false,
            Action::Back if self.at > 0 => self.step(self.at - 1),
            Action::Forward if self.at + 1 < self.history.len() => self.step(self.at + 1),
            Action::Back | Action::Forward => {}
            Action::Reload => {
                let route = self.route().clone();
                self.pages.remove(&route);
                self.ensure(&route);
            }
            Action::Play(items, index) => {
                let growing = self.growing_list(&items);
                self.play(items, index);
                self.growing = growing;
            }
            Action::Jump(index) => self.play_index(index),
            Action::PlayNext(item) => {
                let at = self.saved.current.map_or(0, |current| current + 1).min(self.saved.queue.len());
                self.curated = true;
                self.saved.queue.insert(at, item);
                if self.saved.current.is_none() {
                    self.play_index(0);
                }
            }
            Action::Enqueue(item) => {
                self.curated = true;
                self.saved.queue.push(item);
                if self.saved.current.is_none() {
                    self.play_index(0);
                }
            }
            Action::Remove(index) => {
                if index < self.saved.queue.len() && self.saved.current != Some(index) {
                    self.saved.queue.remove(index);
                    if let Some(current) = self.saved.current.as_mut().filter(|current| index < **current) {
                        *current -= 1;
                    }
                }
            }
            Action::Toggle => match (self.saved.current, self.resume_at) {
                (Some(current), Some(position)) => {
                    self.play_index(current);
                    self.player.seek(position);
                }
                (Some(_), None) => self.player.set_paused(!self.player.paused()),
                (None, _) => {}
            },
            Action::Next => self.next(false),
            Action::Previous => self.previous(),
            Action::Seek(ms) => self.player.seek(ms),
            Action::Volume(volume) => {
                self.saved.volume = volume.clamp(0.0, 1.0);
                self.player.set_volume(self.saved.volume);
            }
            Action::Like(item) => {
                let liked = !self.saved.liked.iter().any(|known| known.key() == item.key());
                toggle(&mut self.saved.liked, item.clone());
                if self.yt.signed_in() && item.is_song() {
                    self.background(move |yt| {
                        let problem = yt.rate(&item.video_id, liked).err()?;
                        Some(Event::LikeRejected(item, format!("Like not saved. {problem}")))
                    });
                }
            }
            Action::Share(item) => {
                let link = share_link(&item);
                if link.is_empty() {
                    return;
                }
                self.ctx.copy_text(link);
                self.notify("Copied share link");
            }
            Action::Pin(item) => toggle(&mut self.saved.pinned, item),
            Action::ToggleQueue => {
                self.saved.queue_open ^= true;
                if self.saved.queue_open {
                    self.settings_preview_open = false;
                }
            }
            Action::ToggleSidebar => self.saved.sidebar_open ^= true,
            Action::ToggleQueueRetractable => self.saved.queue_retractable ^= true,
            Action::ToggleMiniCapsuleRetractable => self.saved.mini_capsule_retractable ^= true,
            Action::ToggleShuffle => {
                self.saved.shuffle ^= true;
                if self.saved.shuffle {
                    self.original_queue = Some(self.saved.queue.clone());
                    let upcoming = self.saved.current.map_or(0, |current| current + 1).min(self.saved.queue.len());
                    shuffle(&mut self.saved.queue[upcoming..]);
                } else if let Some(orig) = self.original_queue.take() {
                    let cur_id = self.saved.current.and_then(|c| self.saved.queue.get(c)).map(|s| s.video_id.clone());
                    self.saved.queue = orig;
                    if let Some(id) = cur_id {
                        if let Some(pos) = self.saved.queue.iter().position(|s| s.video_id == id) {
                            self.saved.current = Some(pos);
                        }
                    }
                }
            }
            Action::CycleRepeat => {
                self.saved.repeat = match self.saved.repeat {
                    Repeat::Off => Repeat::All,
                    Repeat::All => Repeat::One,
                    Repeat::One => Repeat::Off,
                }
            }
            Action::OpenSignIn => self.sign_in = Some(SignIn::default()),
            Action::CloseSignIn => {
                self.cancel_sign_in.store(true, Relaxed);
                self.sign_in = None;
            }
            Action::BrowserSignIn => {
                self.cancel_sign_in.store(true, Relaxed);
                self.cancel_sign_in = Arc::default();
                let cancel = self.cancel_sign_in.clone();
                self.start_sign_in("Finish signing in in the browser window. It closes by itself.", move || {
                    auth::browser_sign_in(&cancel)
                });
            }
            Action::CookieSignIn => {
                let pasted = self.sign_in.as_ref().map(|sign_in| sign_in.cookie.clone()).unwrap_or_default();
                self.start_sign_in("Checking with YouTube.", move || auth::clean_cookie(&pasted));
            }
            Action::SignOut => self.sign_out(),
            Action::SwitchAccount(account) => {
                self.yt.act_as(&account.page);
                self.saved.account = Some(account);
                self.refresh();
            }
            Action::Undo => {
                if let Some((queue, current, position)) = self.replaced.take() {
                    self.saved.queue = queue;
                    self.curated = true;
                    self.notice = None;
                    if let Some(current) = current {
                        self.play_index(current);
                        self.player.seek(position);
                    }
                }
            }
            Action::GetUpdate => {
                let Some(version) = self.update.clone().filter(|_| !self.updating) else {
                    return;
                };
                match update::plan() {
                    Plan::InPlace => {
                        self.updating = true;
                        self.offer(
                            format!("Downloading {APP_NAME} {version}. It restarts by itself."),
                            None,
                            UPDATE_NOTICE_TIME,
                        );
                        self.background(move |yt| Some(Event::Updated(update::install(&yt.agent, &version))));
                    }
                    Plan::PackageManager(command) => {
                        self.offer(
                            format!("{APP_NAME} {version} is out. Update it with: {command}"),
                            None,
                            UPDATE_NOTICE_TIME,
                        );
                    }
                    Plan::DownloadPage => self.apply(Action::OpenRelease),
                }
            }
            Action::OpenRelease => {
                self.notice = None;
                self.ctx.open_url(egui::OpenUrl::new_tab(LATEST_RELEASE));
            }
            Action::Pause(paused) => {
                if paused == self.playing() {
                    self.apply(Action::Toggle);
                }
            }
            Action::Move(from, gap) => {
                let upcoming = self.saved.current.map_or(0, |current| current + 1);
                if reorder(&mut self.saved.queue, upcoming, from, gap) {
                    self.curated = true;
                    if from == upcoming || gap == upcoming {
                        self.player.preload(&self.saved.queue[upcoming].video_id);
                    }
                }
            }
            Action::Expand(open) => self.expanded = open && self.saved.current.is_some(),
            Action::ShowLyrics(open) => {
                self.saved.lyrics_open = open;
                if matches!(self.lyrics, Some((_, Words::Failed))) {
                    self.lyrics = None;
                }
            }
            Action::ClearHistory => {
                let undo = Action::RestoreHistory(std::mem::take(&mut self.saved.recent));
                self.offer("History cleared", Some(("Undo", undo)), NOTICE_TIME);
            }
            Action::RestoreHistory(recent) => {
                self.saved.recent = recent;
                self.notice = None;
            }
            Action::ShowOnDiscord(shown) => {
                self.saved.discord = shown;
                self.discord_linked = None;
                self.discord = shown.then(|| link_discord(&self.sender, &self.ctx));
            }
            Action::DiscordHeadline(headline) => self.saved.discord_headline = headline,
            Action::DiscordName(name) => {
                self.saved.discord_name = name;
                self.saved.discord_headline = Headline::Custom;
            }
            Action::ToggleMiniMode => {
                self.saved.mini_mode = !self.saved.mini_mode;
                if self.saved.mini_mode {
                    self.title_bar.enable_transparency();
                    self.ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
                    self.ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(false));
                    self.ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
                    let (w, h) = self.saved.mini_variant.size(self.mini_queue_open);
                    self.ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(w, h)));
                    let level = if self.saved.mini_always_on_top { egui::WindowLevel::AlwaysOnTop } else { egui::WindowLevel::Normal };
                    self.ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
                    if self.saved.auto_purge_ram || (self.saved.performance_mode && self.saved.perf_aggressive_ram) {
                        self.apply(Action::PurgeMemory);
                    }
                } else {
                    self.title_bar.restore_decorations();
                    self.ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
                    self.ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(true));
                    self.ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(WINDOW_SIZE.into()));
                    self.ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::Normal));
                    self.ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
                }
            }
            Action::ToggleLoudness => {
                self.saved.loudness_normalizer = !self.saved.loudness_normalizer;
                self.player.set_loudness_normalizer(self.saved.loudness_normalizer);
                self.notify(if self.saved.loudness_normalizer {
                    "Loudness Normalizer (-14 LUFS) Active"
                } else {
                    "Loudness Normalizer Bypassed (Raw Volume)"
                });
            }
            Action::PurgeMemory => {
                #[cfg(windows)]
                {
                    #[link(name = "psapi")]
                    unsafe extern "system" {
                        fn EmptyWorkingSet(h_process: isize) -> i32;
                        fn GetCurrentProcess() -> isize;
                    }
                    unsafe {
                        EmptyWorkingSet(GetCurrentProcess());
                    }
                }
            }
            Action::VolumeBy(step) => self.apply(Action::Volume(self.saved.volume + step)),
            Action::GlobalHotkeys(enabled) => {
                self.saved.hotkeys = enabled;
                self.recording = None;
                self.restart_hotkeys();
            }
            Action::Record(command) => self.set_recording(command),
            Action::ResetHotkeys => {
                self.saved.hotkey_combos = hotkeys::standard_combos();
                self.recording = None;
                self.restart_hotkeys();
            }
            Action::AutoPurgeRam(enabled) => self.saved.auto_purge_ram = enabled,
            Action::TogglePerformanceMode => {
                self.saved.performance_mode = !self.saved.performance_mode;
                if self.saved.performance_mode && self.saved.perf_aggressive_ram {
                    self.apply(Action::PurgeMemory);
                }
            }
            Action::SetPerfAggressiveRam(enabled) => self.saved.perf_aggressive_ram = enabled,
            Action::SetPerfSuspendAnimations(enabled) => self.saved.perf_suspend_animations = enabled,
            Action::SetPerfDisableBlurs(enabled) => self.saved.perf_disable_blurs = enabled,
            Action::SetPerfCompactBuffers(enabled) => self.saved.perf_compact_buffers = enabled,
            Action::SetPerfPlaceholderMoodArt(enabled) => self.saved.perf_placeholder_mood_art = enabled,
            Action::ToggleEq(enabled) => {
                self.saved.eq_enabled = enabled;
                self.player.set_eq_enabled(enabled);
            }
            Action::SetEqPreset(preset) => {
                self.saved.eq_preset = preset;
                if preset != crate::dsp::EqPreset::Custom {
                    self.saved.eq_gains = preset.gains();
                    self.player.set_eq_gains(self.saved.eq_gains);
                }
            }
            Action::SetEqGain(band, gain) => {
                if band < 10 {
                    self.saved.eq_preset = crate::dsp::EqPreset::Custom;
                    self.saved.eq_gains[band] = gain;
                    self.player.set_eq_gains(self.saved.eq_gains);
                }
            }
            Action::SetEqPreamp(preamp) => {
                self.saved.eq_preamp_db = preamp.clamp(-12.0, 12.0);
                self.player.set_eq_preamp(self.saved.eq_preamp_db);
            }
            Action::SetAutoEq(hp) => {
                self.saved.autoeq_headphone = hp;
                if let Some(gains) = hp.gains() {
                    self.saved.eq_preset = crate::dsp::EqPreset::Custom;
                    self.saved.eq_gains = gains;
                    self.player.set_eq_gains(self.saved.eq_gains);
                }
            }
            Action::ResetEq => {
                self.saved.eq_preset = crate::dsp::EqPreset::Flat;
                self.saved.autoeq_headphone = crate::dsp::AutoEqHeadphone::None;
                self.saved.eq_gains = [0.0; 10];
                self.saved.eq_preamp_db = 0.0;
                self.player.set_eq_gains([0.0; 10]);
                self.player.set_eq_preamp(0.0);
            }
            Action::SetDiscordLine1(c) => self.saved.discord_line1 = c,
            Action::SetDiscordLine2(c) => self.saved.discord_line2 = c,
            Action::SetDiscordLine3(c) => self.saved.discord_line3 = c,
            Action::SetDiscordButtonListen(b) => self.saved.discord_button_listen = b,
            Action::SetDiscordButtonApp(b) => self.saved.discord_button_app = b,
            Action::SetSettingsTab(tab) => {
                self.expanded = false;
                self.go(Route::Settings(tab));
            }
            Action::SetMiniPalette(p) => self.saved.mini_palette = p,
            Action::SetMiniOpacity(o) => self.saved.mini_opacity = o.clamp(0.55, 1.0),
            Action::SetMiniShowShuffleRepeat(v) => self.saved.mini_show_shuffle_repeat = v,
            Action::SetMiniShowQueue(v) => self.saved.mini_show_queue = v,
            Action::SetMiniShowVolume(v) => self.saved.mini_show_volume = v,
            Action::SetMiniShowClock(v) => self.saved.mini_show_clock = v,
            Action::SetMiniAlwaysOnTop(v) => {
                self.saved.mini_always_on_top = v;
                if self.saved.mini_mode {
                    let level = if v { egui::WindowLevel::AlwaysOnTop } else { egui::WindowLevel::Normal };
                    self.ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
                }
            }
            Action::ToggleMiniQueue => {
                self.mini_queue_open = !self.mini_queue_open;
                if self.saved.mini_mode {
                    let (w, h) = self.saved.mini_variant.size(self.mini_queue_open);
                    self.ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(w, h)));
                }
            }
            Action::SetPerfProfile(profile) => {
                self.saved.perf_profile = profile;
                match profile {
                    PerfProfile::CompetitiveGaming => {
                        self.saved.perf_audio_buffer_mb = 1.5;
                        self.saved.perf_art_cache_mb = 4;
                        self.saved.perf_tick_fps = 1;
                        self.saved.perf_aggressive_ram = true;
                        self.saved.perf_disable_blurs = true;
                        self.saved.perf_compact_buffers = true;
                        self.apply(Action::PurgeMemory);
                    }
                    PerfProfile::Balanced => {
                        self.saved.perf_audio_buffer_mb = 4.0;
                        self.saved.perf_art_cache_mb = 12;
                        self.saved.perf_tick_fps = 30;
                        self.saved.perf_aggressive_ram = false;
                        self.saved.perf_disable_blurs = false;
                        self.saved.perf_compact_buffers = false;
                    }
                    PerfProfile::Audiophile => {
                        self.saved.perf_audio_buffer_mb = 8.0;
                        self.saved.perf_art_cache_mb = 32;
                        self.saved.perf_tick_fps = 60;
                        self.saved.perf_aggressive_ram = false;
                        self.saved.perf_disable_blurs = false;
                        self.saved.perf_compact_buffers = false;
                    }
                    PerfProfile::Custom => {}
                }
                self.art.set_budget_bytes(self.saved.perf_art_cache_mb * 1024 * 1024);
            }
            Action::SetPerfAudioBuffer(mb) => {
                self.saved.perf_profile = PerfProfile::Custom;
                self.saved.perf_audio_buffer_mb = mb.clamp(1.0, 32.0);
            }
            Action::SetPerfArtCache(mb) => {
                self.saved.perf_profile = PerfProfile::Custom;
                self.saved.perf_art_cache_mb = mb.clamp(2, 64);
                self.art.set_budget_bytes(self.saved.perf_art_cache_mb * 1024 * 1024);
            }
            Action::SetPerfTickFps(fps) => {
                self.saved.perf_profile = PerfProfile::Custom;
                self.saved.perf_tick_fps = fps.clamp(1, 144);
            }
            Action::CreatePlaylist(name) => {
                let name = name.trim();
                if !name.is_empty() {
                    let id = format!("pl-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis());
                    self.saved.playlists.push(CustomPlaylist {
                        id: id.clone(),
                        name: name.to_string(),
                        songs: Vec::new(),
                    });
                    self.creating_playlist = false;
                    self.new_playlist_name.clear();
                    self.notify(format!("Created playlist '{}'", name));
                    self.go(Route::CustomPlaylist(id));
                }
            }
            Action::DeletePlaylist(id) => {
                if let Some(pos) = self.saved.playlists.iter().position(|p| p.id == id) {
                    let removed = self.saved.playlists.remove(pos);
                    self.notify(format!("Deleted playlist '{}'", removed.name));
                    if *self.route() == Route::CustomPlaylist(id) {
                        self.go(Route::Home);
                    }
                }
            }
            Action::AddToPlaylist(id, item) => {
                if let Some(pl) = self.saved.playlists.iter_mut().find(|p| p.id == id) {
                    if !pl.songs.iter().any(|s| s.video_id == item.video_id) {
                        let pl_name = pl.name.clone();
                        pl.songs.push(item);
                        self.notify(format!("Added to '{}'", pl_name));
                    } else {
                        self.notify("Already in playlist");
                    }
                }
            }
            Action::RemoveFromPlaylist(id, video_id) => {
                let mut removed = None;
                if let Some(pl) = self.saved.playlists.iter_mut().find(|p| p.id == id) {
                    if let Some(pos) = pl.songs.iter().position(|s| s.video_id == video_id) {
                        let song = pl.songs.remove(pos);
                        removed = Some((song.title, pl.name.clone()));
                    }
                }
                if let Some((title, pl_name)) = removed {
                    self.notify(format!("Removed '{}' from {}", title, pl_name));
                }
            }
            Action::PromptCreatePlaylist => {
                self.creating_playlist = true;
                self.new_playlist_name.clear();
            }
            Action::CancelCreatePlaylist => {
                self.creating_playlist = false;
                self.new_playlist_name.clear();
            }
            Action::DownloadSong(item) => {
                let already_queued = self.download_active.as_ref().map_or(false, |a| a.video_id == item.video_id)
                    || self.download_queue.iter().any(|it| it.video_id == item.video_id);
                if !already_queued {
                    let title = item.title.clone();
                    self.download_queue.push_back(item);
                    if self.download_active.is_none() {
                        self.start_next_download();
                    } else {
                        self.notify(format!("Added to download queue: '{}'", title));
                    }
                }
            }
            Action::SetMiniBackground(bg) => {
                self.saved.mini_bg = bg;
            }
            Action::SetMiniGrain(v) => {
                self.saved.mini_grain = v;
            }
            Action::SetMiniGrainStrength(v) => {
                self.saved.mini_grain_strength = v.clamp(0.01, 0.50);
            }
            Action::SetThemePalette(palette) => {
                self.saved.theme_palette = palette;
            }
            Action::SetThemeBase(base) => {
                self.saved.theme_base = base;
            }
            Action::SetAmbientGlow(v) => {
                self.saved.ambient_glow = v;
            }
            Action::SetAmbientArtworkBlur(v) => {
                self.saved.ambient_artwork_blur = v;
            }
            Action::SetSelectionGlow(v) => {
                self.saved.selection_glow = v;
            }
            Action::SetMiniVariant(v) => {
                self.saved.mini_variant = v;
                if self.saved.mini_mode {
                    let (w, h) = v.size(self.mini_queue_open);
                    self.ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(w, h)));
                }
            }
            Action::ToggleMiniLock => {
                self.saved.mini_locked = !self.saved.mini_locked;
                self.notify(if self.saved.mini_locked {
                    "Mini Mode Locked (Movement Restricted)"
                } else {
                    "Mini Mode Unlocked (Freely Draggable)"
                });
            }
            Action::SetBarLayout(layout) => {
                self.saved.bar_layout = layout;
            }
            Action::SetBarGlass(glass) => {
                self.saved.bar_glass = glass;
            }
            Action::SetBarOpacity(opacity) => {
                self.saved.bar_opacity = opacity.clamp(0.20, 1.0);
            }
            Action::SetBarGrain(grain) => {
                self.saved.bar_grain = grain;
            }
            Action::SetBarGrainStrength(strength) => {
                self.saved.bar_grain_strength = strength.clamp(0.02, 0.50);
            }
            Action::CheckUpdate => {
                self.notify("Checking for updates...");
                let sender = self.sender.clone();
                let ctx = self.ctx.clone();
                std::thread::spawn(move || {
                    let agent = ureq::agent();
                    match agent.head(LATEST_RELEASE).call() {
                        Ok(latest) => {
                            if let Some(version) = newer_version(latest.get_url(), VERSION) {
                                let _ = sender.send(Event::Release(version));
                            } else {
                                let _ = sender.send(Event::Toast(format!("AURI v{} is up to date", VERSION)));
                            }
                        }
                        Err(e) => {
                            let _ = sender.send(Event::Toast(format!("Update check failed: {}", e)));
                        }
                    }
                    ctx.request_repaint();
                });
            }
            Action::SetStorageDir(dir) => {
                self.saved.storage_dir = dir;
            }
            Action::BrowseStorageDir => {
                let sender = self.sender.clone();
                let ctx = self.ctx.clone();
                std::thread::spawn(move || {
                    #[cfg(windows)]
                    {
                        let script = "Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.FolderBrowserDialog; $f.Description = 'Select AURI Storage & Downloads Directory'; if ($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) { Write-Output $f.SelectedPath }";
                        let out = std::process::Command::new("powershell")
                            .args(["-NoProfile", "-Command", script])
                            .output();
                        if let Ok(res) = out {
                            let path = String::from_utf8_lossy(&res.stdout).trim().to_string();
                            if !path.is_empty() {
                                let _ = sender.send(Event::StorageDir(path));
                                ctx.request_repaint();
                            }
                        }
                    }
                });
            }
            Action::OpenStorageDir => {
                let dir = PathBuf::from(&self.saved.storage_dir).join("downloads");
                let _ = std::fs::create_dir_all(&dir);
                let _ = std::process::Command::new("explorer").arg(&dir).spawn();
            }
            Action::ResetStorageDir => {
                self.saved.storage_dir = default_storage_dir();
                self.notify("Storage directory reset to default");
            }
            Action::PurgeAudioCache => {
                crate::cache::purge_cache();
                self.notify("Offline audio cache purged");
            }
            Action::OpenCacheDir => {
                let dir = crate::cache::get_cache_dir();
                let _ = std::process::Command::new("explorer").arg(&dir).spawn();
            }
            Action::StartSyncHost(room) => {
                self.sync_mode = crate::sync::SyncMode::Host;
                self.sync_room = room.clone();
                self.saved.sync_room_code = room.clone();
                let token = if self.saved.sync_github_token.is_empty() { None } else { Some(self.saved.sync_github_token.clone()) };
                let gist = if self.saved.sync_gist_id.is_empty() { None } else { Some(self.saved.sync_gist_id.clone()) };
                if let Some(s) = &self.sync {
                    s.host(room, token, gist);
                }
                self.notify("🎧 Started Listen Together Session (Hosting)");
            }
            Action::StartSyncJoin(room) => {
                self.sync_mode = crate::sync::SyncMode::Guest;
                self.sync_room = room.clone();
                self.saved.sync_room_code = room.clone();
                let gist = if self.saved.sync_gist_id.is_empty() { None } else { Some(self.saved.sync_gist_id.clone()) };
                if let Some(s) = &self.sync {
                    s.join(room, gist);
                }
                self.notify("🎧 Joined Listen Together Session (Syncing)");
            }
            Action::StopSync => {
                self.sync_mode = crate::sync::SyncMode::Disconnected;
                self.sync_room.clear();
                if let Some(s) = &self.sync {
                    s.disconnect();
                }
                self.notify("Disconnected from Listen Together Session");
            }
            Action::SetSyncGithubToken(tok) => self.saved.sync_github_token = tok,
            Action::SetSyncGistId(gist) => self.saved.sync_gist_id = gist,
            Action::SetLyricsSyncedMode(v) => self.saved.lyrics_synced_mode = v,
            Action::SetLyricsFontSize(sz) => self.saved.lyrics_font_size = sz,
            Action::SetLyricsAlignCenter(v) => self.saved.lyrics_align_center = v,
            Action::SetLyricsDimUnsung(dim) => self.saved.lyrics_dim_unsung = dim,
            Action::ClearStats => {
                self.saved.stats_total_listen_ms = 0;
                self.saved.stats_total_plays = 0;
                self.saved.stats_top_tracks.clear();
                self.saved.stats_top_artists.clear();
                self.saved.stats_top_albums.clear();
                self.saved.stats_daily.clear();
                self.notify("Listening statistics reset");
            }
            Action::SetStatsPeriod(period) => {
                self.saved.stats_filter = period;
            }
            Action::SetThemeMode(_mode) => {
                // Light mode is frozen/locked to Dark mode
                self.saved.theme_mode = ThemeMode::Dark;
            }
            Action::SetThemePreset(preset) => {
                self.saved.theme_preset = preset;
                self.saved.theme_palette = preset.default_accent();
            }
            Action::SetCloseToTray(val) => {
                self.saved.close_to_tray = val;
            }
            Action::SetSongNotifications(val) => {
                self.saved.song_notifications = val;
            }
            Action::SetLaunchOnStartup(val) => {
                self.saved.launch_on_startup = val;
            }
            Action::SetHistoryEnabled(val) => {
                self.saved.history_enabled = val;
            }
            Action::SetHomeFilter(filter) => {
                self.saved.home_filter = filter;
                self.refresh();
            }
            Action::SetStreamQuality(quality) => {
                self.saved.stream_quality = quality;
                self.yt.set_low_quality(quality == StreamQuality::Low);
                self.notify(format!("Stream Quality: {}", quality.name()));
            }
            Action::SetCrossfadeEnabled(enabled) => {
                self.saved.crossfade_enabled = enabled;
            }
            Action::SetCrossfadeSecs(secs) => {
                self.saved.crossfade_secs = secs.clamp(1, 10);
            }
            Action::BlockArtist(artist) => {
                let trimmed = artist.trim().to_string();
                if !trimmed.is_empty() && !self.saved.blocked_artists.iter().any(|a| a.eq_ignore_ascii_case(&trimmed)) {
                    self.saved.blocked_artists.push(trimmed.clone());
                    self.notify(format!("Blocked artist: {trimmed}"));
                }
            }
            Action::UnblockArtist(artist) => {
                self.saved.blocked_artists.retain(|a| !a.eq_ignore_ascii_case(&artist));
                self.notify(format!("Unblocked artist: {artist}"));
            }
            Action::OpenUpdateDialog => {
                self.update_dialog_open = true;
            }
            Action::CloseUpdateDialog => {
                self.update_dialog_open = false;
            }
            Action::StartRadio(item) => {
                let seed = item.video_id.clone();
                self.curated = false;
                self.saved.queue = vec![item.clone()];
                self.saved.current = Some(0);
                self.play_index(0);
                self.radio_seed = seed.clone();
                let (yt, sender, ctx) = (self.yt.clone(), self.sender.clone(), self.ctx.clone());
                std::thread::spawn(move || {
                    if let Ok(items) = yt.radio(&seed) {
                        let _ = sender.send(Event::Radio(seed, items));
                        ctx.request_repaint();
                    }
                });
                self.notify(format!("Starting radio for {}", item.title));
            }
            Action::Dislike(item) => {
                if self.saved.current.and_then(|c| self.saved.queue.get(c)).is_some_and(|cur| cur.video_id == item.video_id) {
                    self.next(false);
                }
                let mut i = 0;
                while i < self.saved.queue.len() {
                    if self.saved.queue[i].video_id == item.video_id {
                        self.saved.queue.remove(i);
                        if let Some(current) = self.saved.current.as_mut().filter(|current| i < **current) {
                            *current -= 1;
                        }
                    } else {
                        i += 1;
                    }
                }
                self.notify(format!("Disliked and removed: {}", item.title));
            }
            Action::ToggleGameMode => {
                if self.in_game_mode {
                    self.apply(Action::ExitGameMode);
                } else {
                    self.in_game_mode = true;
                    self.game_overlay_open = false;
                    self.game_mode_start_time = Some(Instant::now());
                    self.apply(Action::PurgeMemory);
                    self.title_bar.enable_transparency();
                    self.ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(-32000.0, -32000.0)));
                    self.ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1.0, 1.0)));
                    self.ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::Normal));
                    self.notify(format!("⚡ Gaming Mode Active ({})", self.saved.game_overlay_hotkey.label()));
                }
            }
            Action::ExitGameMode => {
                self.in_game_mode = false;
                self.game_overlay_open = false;
                self.game_mode_start_time = None;
                self.title_bar.restore_decorations();
                self.ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(100.0, 100.0)));
                self.ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(WINDOW_SIZE.into()));
                self.ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::Normal));
                self.ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                self.notify("Restored Normal AURI Mode");
            }
            Action::ExitGameModeToMini => {
                self.in_game_mode = false;
                self.game_overlay_open = false;
                self.game_mode_start_time = None;
                self.title_bar.restore_decorations();
                self.saved.mini_mode = true;
                let (w, h) = self.saved.mini_variant.size(self.mini_queue_open);
                self.ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(120.0, 120.0)));
                self.ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(w, h)));
                let level = if self.saved.mini_always_on_top { egui::WindowLevel::AlwaysOnTop } else { egui::WindowLevel::Normal };
                self.ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
                self.ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                self.notify("Switched to Mini Player Mode");
            }
            Action::ToggleGameOverlay => {
                if self.in_game_mode {
                    self.game_overlay_open = !self.game_overlay_open;
                    if self.game_overlay_open {
                        #[cfg(windows)]
                        let (sw, sh) = {
                            use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
                            unsafe { (GetSystemMetrics(SM_CXSCREEN) as f32, GetSystemMetrics(SM_CYSCREEN) as f32) }
                        };
                        #[cfg(not(windows))]
                        let (sw, sh) = (1920.0, 1080.0);
                        let ppp = self.ctx.pixels_per_point().max(0.5);
                        self.ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(0.0, 0.0)));
                        self.ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(sw / ppp, sh / ppp)));
                        self.ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop));
                        self.ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                        self.apply(Action::PurgeMemory);
                    } else {
                        self.ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(-32000.0, -32000.0)));
                        self.ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1.0, 1.0)));
                        self.ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::Normal));
                        self.apply(Action::PurgeMemory);
                    }
                }
            }
            Action::ToggleGameHud => {
                self.apply(Action::ToggleGameOverlay);
            }
            Action::SetAutoGameMode(on) => {
                self.saved.auto_game_mode = on;
            }
            Action::SetGameOverlayHotkey(hk) => {
                self.saved.game_overlay_hotkey = hk;
            }
            Action::SetGameOverlayOpacity(op) => {
                self.saved.game_overlay_opacity = op.clamp(0.15, 0.95);
            }
            Action::ToggleGameMusicWidget => {
                self.saved.game_show_music_widget = !self.saved.game_show_music_widget;
            }
            Action::ToggleGameAudioWidget => {
                self.saved.game_show_audio_widget = !self.saved.game_show_audio_widget;
            }
            Action::ToggleGameQueueWidget => {
                self.saved.game_show_queue_widget = !self.saved.game_show_queue_widget;
            }
            Action::AddGameWhitelist(exe) => {
                let trimmed = exe.trim().to_string();
                if !trimmed.is_empty() && !self.saved.game_whitelist.iter().any(|g| g.eq_ignore_ascii_case(&trimmed)) {
                    self.saved.game_whitelist.push(trimmed.clone());
                    self.notify(format!("Added \"{}\" to game whitelist", trimmed));
                }
            }
            Action::RemoveGameWhitelist(idx) => {
                if idx < self.saved.game_whitelist.len() {
                    let removed = self.saved.game_whitelist.remove(idx);
                    self.notify(format!("Removed \"{}\" from whitelist", removed));
                }
            }
            Action::AddGameBlacklist(exe) => {
                let trimmed = exe.trim().to_string();
                if !trimmed.is_empty() && !self.saved.game_blacklist.iter().any(|b| b.eq_ignore_ascii_case(&trimmed)) {
                    self.saved.game_blacklist.push(trimmed.clone());
                    self.notify(format!("Added \"{}\" to app blacklist", trimmed));
                }
            }
            Action::RemoveGameBlacklist(idx) => {
                if idx < self.saved.game_blacklist.len() {
                    let removed = self.saved.game_blacklist.remove(idx);
                    self.notify(format!("Removed \"{}\" from blacklist", removed));
                }
            }
            Action::SetGameMusicPos(x, y) => {
                self.saved.game_music_pos = [x.max(0.0), y.max(0.0)];
            }
            Action::SetGameAudioPos(x, y) => {
                self.saved.game_audio_pos = [x.max(0.0), y.max(0.0)];
            }
            Action::SetGameQueuePos(x, y) => {
                self.saved.game_queue_pos = [x.max(0.0), y.max(0.0)];
            }
            Action::ResetGameWidgetPositions => {
                self.saved.game_music_pos = default_game_music_pos();
                self.saved.game_audio_pos = default_game_audio_pos();
                self.saved.game_queue_pos = default_game_queue_pos();
                self.notify("Overlay widget positions reset to default");
            }
            Action::ResetDiscordDefaults => {
                self.saved.discord_show_when_paused = false;
                self.saved.discord_hide_details = false;
                self.saved.discord_status_text = DiscordStatusField::Line2Artist;
                self.saved.discord_app_name = String::new();
                self.saved.discord_line1 = DiscordLineContent::Title;
                self.saved.discord_line2 = DiscordLineContent::Artist;
                self.saved.discord_line3 = DiscordLineContent::Album;
                self.saved.discord_album_art = true;
                self.saved.discord_provider_badge = true;
                self.saved.discord_progress_bar = true;
                self.saved.discord_line1_clickable = false;
                self.saved.discord_line2_clickable = false;
                self.saved.discord_art_clickable = false;
                self.saved.discord_button1 = DiscordButtonOption::ListenOnYtm;
                self.saved.discord_button2 = DiscordButtonOption::GetApp;
                self.notify("Discord settings reset to defaults");
            }
            Action::SetDiscordShowWhenPaused(val) => self.saved.discord_show_when_paused = val,
            Action::SetDiscordHideDetails(val) => self.saved.discord_hide_details = val,
            Action::SetDiscordStatusText(st) => self.saved.discord_status_text = st,
            Action::SetDiscordAppName(name) => self.saved.discord_app_name = name,
            Action::SetDiscordAlbumArt(val) => self.saved.discord_album_art = val,
            Action::SetDiscordProviderBadge(val) => self.saved.discord_provider_badge = val,
            Action::SetDiscordProgressBar(val) => self.saved.discord_progress_bar = val,
            Action::SetDiscordLine1Clickable(val) => self.saved.discord_line1_clickable = val,
            Action::SetDiscordLine2Clickable(val) => self.saved.discord_line2_clickable = val,
            Action::SetDiscordArtClickable(val) => self.saved.discord_art_clickable = val,
            Action::SetDiscordButton1(b) => self.saved.discord_button1 = b,
            Action::SetDiscordButton2(b) => self.saved.discord_button2 = b,
            Action::SetCornerRoundness(r) => {
                self.saved.corner_roundness = r.clamp(0.0, 24.0);
            }
            Action::SetBackgroundTintShade(s) => {
                self.saved.background_tint_shade = s.clamp(0.0, 1.0);
            }
            Action::ResetThemeDefaults => {
                self.saved.theme_palette = self.saved.theme_preset.default_accent();
                self.saved.theme_base = ThemeBase::DarkObsidian;
                self.saved.corner_roundness = 12.0;
                self.saved.background_tint_shade = 0.0;
                self.saved.artwork_bg_variant = ArtworkBgVariant::CanvasTint;
                self.saved.artwork_bg_opacity = 0.50;
                self.saved.artwork_bg_blur = 0.60;
                self.notify("Appearance reset to preset defaults");
            }
            Action::ToggleSettingsPreview => {
                self.settings_preview_open = !self.settings_preview_open;
                if self.settings_preview_open {
                    self.saved.queue_open = false;
                }
            }
            Action::SetArtworkBgVariant(v) => self.saved.artwork_bg_variant = v,
            Action::SetArtworkBgOpacity(v) => self.saved.artwork_bg_opacity = v.clamp(0.05, 1.0),
            Action::SetArtworkBgBlur(v) => self.saved.artwork_bg_blur = v.clamp(0.1, 1.0),
        }
    }
}

#[cfg(windows)]
pub fn detect_fullscreen_game(whitelist: &[String], blacklist: &[String]) -> bool {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetForegroundWindow, GetSystemMetrics, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId,
        SM_CXSCREEN, SM_CYSCREEN,
    };
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return false;
        }
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 || pid == std::process::id() {
            return false;
        }

        // 1. Query process executable name
        let mut proc_name = String::new();
        let h_proc = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if !h_proc.is_null() {
            let mut buf = [0u16; 512];
            let mut size = 512u32;
            if QueryFullProcessImageNameW(h_proc, 0, buf.as_mut_ptr(), &mut size) != 0 && size > 0 {
                let full = String::from_utf16_lossy(&buf[..size as usize]);
                if let Some(exe) = full.split('\\').last() {
                    proc_name = exe.to_lowercase();
                }
            }
            CloseHandle(h_proc);
        }

        // 2. Query window title
        let mut title_buf = [0u16; 512];
        let tlen = GetWindowTextW(hwnd, title_buf.as_mut_ptr(), 512);
        let win_title = if tlen > 0 {
            String::from_utf16_lossy(&title_buf[..tlen as usize]).to_lowercase()
        } else {
            String::new()
        };

        // 3. Blacklist check: if foreground app matches blacklist, never trigger gaming mode
        for b in blacklist {
            let b_lower = b.trim().to_lowercase();
            if !b_lower.is_empty() && (proc_name.contains(&b_lower) || win_title.contains(&b_lower)) {
                return false;
            }
        }

        // 4. Whitelist check: if foreground app matches whitelist, trigger gaming mode immediately
        for w in whitelist {
            let w_lower = w.trim().to_lowercase();
            if !w_lower.is_empty() && (proc_name.contains(&w_lower) || win_title.contains(&w_lower)) {
                return true;
            }
        }

        // 5. Fallback: check if window is full screen resolution and not Windows Shell
        let mut rect = std::mem::zeroed();
        if GetWindowRect(hwnd, &mut rect) != 0 {
            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);
            let w = rect.right - rect.left;
            let h = rect.bottom - rect.top;
            if w >= screen_w && h >= screen_h {
                let mut class_name = [0u16; 256];
                let len = GetClassNameW(hwnd, class_name.as_mut_ptr(), 256);
                if len > 0 {
                    let s = String::from_utf16_lossy(&class_name[..len as usize]);
                    if s != "Progman" && s != "WorkerW" && s != "Shell_TrayWnd" && s != "Windows.UI.Core.CoreWindow" {
                        return true;
                    }
                }
            }
        }
        false
    }
}

#[cfg(not(windows))]
pub fn detect_fullscreen_game(_whitelist: &[String], _blacklist: &[String]) -> bool {
    false
}

#[cfg(windows)]
pub fn is_overlay_hotkey_pressed(hotkey: GameOverlayHotkey) -> bool {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, VK_CONTROL, VK_F1, VK_F12, VK_F9, VK_SHIFT, VK_TAB,
    };
    unsafe {
        let is_down = |vk: i32| (GetAsyncKeyState(vk) as u16 & 0x8000) != 0;
        match hotkey {
            GameOverlayHotkey::CtrlShiftG => is_down(VK_CONTROL as i32) && is_down(VK_SHIFT as i32) && is_down(0x47 /* 'G' */),
            GameOverlayHotkey::ShiftTab => is_down(VK_SHIFT as i32) && is_down(VK_TAB as i32),
            GameOverlayHotkey::F9 => is_down(VK_F9 as i32),
            GameOverlayHotkey::F12 => is_down(VK_F12 as i32),
            GameOverlayHotkey::CtrlF1 => is_down(VK_CONTROL as i32) && is_down(VK_F1 as i32),
        }
    }
}

#[cfg(not(windows))]
pub fn is_overlay_hotkey_pressed(_hotkey: GameOverlayHotkey) -> bool {
    false
}

impl eframe::App for App {
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        if self.in_game_mode || self.saved.mini_mode {
            [0.0, 0.0, 0.0, 0.0]
        } else {
            let fill = visuals.panel_fill;
            [
                fill.r() as f32 / 255.0,
                fill.g() as f32 / 255.0,
                fill.b() as f32 / 255.0,
                1.0,
            ]
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let (size, minimized) = ctx.input(|input| (input.content_rect().size(), input.viewport().minimized == Some(true)));
        if minimized && (self.saved.auto_purge_ram || (self.saved.performance_mode && self.saved.perf_aggressive_ram)) {
            self.apply(Action::PurgeMemory);
        }
        if self.saved.performance_mode && self.saved.perf_aggressive_ram && self.last_purge.elapsed() >= Duration::from_secs(30) {
            self.apply(Action::PurgeMemory);
            self.last_purge = Instant::now();
        }
        #[cfg(windows)]
        if self.saved.auto_game_mode && self.last_game_check.elapsed() >= Duration::from_millis(2500) {
            self.last_game_check = Instant::now();
            let is_gaming = detect_fullscreen_game(&self.saved.game_whitelist, &self.saved.game_blacklist);
            if is_gaming && !self.in_game_mode {
                self.auto_game_detected = true;
                self.apply(Action::ToggleGameMode);
            } else if !is_gaming && self.in_game_mode && self.auto_game_detected {
                self.auto_game_detected = false;
                self.apply(Action::ExitGameMode);
            }
        }
        if self.sync_mode == crate::sync::SyncMode::Host {
            let should_pub = match self.sync_last_publish {
                Some(last) => last.elapsed() > Duration::from_millis(1500),
                None => true,
            };
            if should_pub {
                self.sync_last_publish = Some(Instant::now());
                if let Some(cur) = self.saved.current.and_then(|c| self.saved.queue.get(c)) {
                    let pkt = crate::sync::SyncPacket {
                        room: self.sync_room.clone(),
                        video_id: cur.video_id.clone(),
                        title: cur.title.clone(),
                        subtitle: cur.subtitle.clone(),
                        position_ms: self.player.position_ms(),
                        playing: self.playing(),
                        timestamp_ms: crate::sync::now_timestamp_ms(),
                    };
                    if let Some(s) = &self.sync {
                        s.publish(pkt);
                    }
                }
            }
        }
        if !minimized && !self.saved.mini_mode && (size.x < WINDOW_MIN[0] - 1.0 || size.y < WINDOW_MIN[1] - 1.0) {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(WINDOW_SIZE.into()));
        }
        self.art.begin_frame(ctx);
        self.drain(ctx);
        let mut actions = if self.record(ctx) { Vec::new() } else { self.shortcuts(ctx) };
        #[cfg(windows)]
        if self.in_game_mode {
            let down = is_overlay_hotkey_pressed(self.saved.game_overlay_hotkey);
            if down && !self.hotkey_was_down {
                actions.push(Action::ToggleGameOverlay);
            }
            self.hotkey_was_down = down;
        }

        if self.in_game_mode {
            if self.game_overlay_open {
                if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                    actions.push(Action::ToggleGameOverlay);
                }
                actions.extend(ui::draw_game_overlay(self, ctx));
            } else {
                ctx.request_repaint_after(Duration::from_millis(50));
            }
        } else if self.saved.mini_mode {
            let is_maximized = ctx.input(|input| input.viewport().maximized == Some(true));
            if is_maximized {
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(false));
                let (w, h) = self.saved.mini_variant.size(self.mini_queue_open);
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(w, h)));
            }
            actions.extend(ui::draw_mini(self, ctx));
        } else {
            actions.extend(ui::draw(self, ctx));
        }
        for action in actions {
            self.apply(action);
        }
        self.look_up_covers();
        self.look_up_lyrics();
        let minimized = self.ctx.input(|input| input.viewport().minimized == Some(true));
        if self.playing() || self.player.loading() {
            let tick = if minimized || self.in_game_mode {
                Duration::from_millis(500)
            } else if self.saved.performance_mode {
                Duration::from_millis(1000 / self.saved.perf_tick_fps.max(1) as u64)
            } else {
                PROGRESS_TICK
            };
            ctx.request_repaint_after(tick);
        }

        // Seamless crossfade transition (bypassed / disabled in Performance & Gaming Mode)
        if self.saved.crossfade_enabled && !self.saved.performance_mode && self.playing() && self.saved.repeat != Repeat::One {
            let pos_ms = self.player.position_ms();
            let dur_ms = self.player.duration_ms();
            let fade_ms = (self.saved.crossfade_secs as u64) * 1000;
            if dur_ms > fade_ms + 2000 && pos_ms >= dur_ms.saturating_sub(fade_ms) {
                let remaining = dur_ms.saturating_sub(pos_ms);
                if remaining <= 250 {
                    self.next(true);
                }
            }
        }

        let item = self.saved.current.and_then(|current| self.saved.queue.get(current));
        let playing = item.is_some() && !self.player.paused();
        if playing {
            let now = std::time::Instant::now();
            if let Some(prev) = self.last_listen_tick {
                let delta = now.duration_since(prev).as_millis() as u64;
                if delta < 2000 {
                    self.saved.stats_total_listen_ms += delta;

                    let secs = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                        Ok(d) => d.as_secs(),
                        Err(_) => 0,
                    };
                    let day_key = (secs / 86400) as u32;
                    let hour = ((secs / 3600) % 24) as usize;
                    if let Some(day) = self.saved.stats_daily.iter_mut().find(|d| d.day_key == day_key) {
                        day.listen_ms += delta;
                        day.hourly_ms[hour] = day.hourly_ms[hour].saturating_add(delta as u32);
                    } else {
                        let mut hourly_ms = [0u32; 24];
                        hourly_ms[hour] = delta as u32;
                        self.saved.stats_daily.push(DailyActivity {
                            day_key,
                            listen_ms: delta,
                            plays: 0,
                            hourly_ms,
                        });
                    }

                    if let Some(it) = item {
                        if let Some(track) = self.saved.stats_top_tracks.iter_mut().find(|t| t.video_id == it.video_id) {
                            track.listen_ms += delta;
                        }
                        if let Some(art) = self.saved.stats_top_artists.iter_mut().find(|a| a.artist == it.subtitle) {
                            art.listen_ms += delta;
                        }
                        if !it.extra.is_empty() {
                            if let Some(alb) = self.saved.stats_top_albums.iter_mut().find(|a| a.title == it.extra) {
                                alb.listen_ms += delta;
                            }
                        }
                    }
                }
            }
            self.last_listen_tick = Some(now);
        } else {
            self.last_listen_tick = None;
        }
        let (position_ms, duration_ms) = (self.player.position_ms(), self.player.duration_ms());
        if let Some(media) = &mut self.media {
            media.show(item, playing, position_ms, duration_ms);
        }
        if let Some(discord) = self.discord.as_mut() {
            let track = item.filter(|_| playing || self.saved.discord_show_when_paused).map(|item| (item, self.saved.covers.of(item)));
            let (headline, name) = (self.saved.discord_headline, &self.saved.discord_name);
            let fmt = crate::discord::DiscordFormat {
                show_when_paused: self.saved.discord_show_when_paused,
                hide_details: self.saved.discord_hide_details,
                status_text: self.saved.discord_status_text,
                app_name: self.saved.discord_app_name.clone(),
                line1: self.saved.discord_line1,
                line2: self.saved.discord_line2,
                line3: self.saved.discord_line3,
                album_art: self.saved.discord_album_art,
                provider_badge: self.saved.discord_provider_badge,
                progress_bar: self.saved.discord_progress_bar,
                line1_clickable: self.saved.discord_line1_clickable,
                line2_clickable: self.saved.discord_line2_clickable,
                art_clickable: self.saved.discord_art_clickable,
                button1: self.saved.discord_button1,
                button2: self.saved.discord_button2,
            };
            discord.show(track, headline, name, position_ms, duration_ms, fmt, playing);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(program) = &self.restart {
            update::restart(program, &crate::profile_arguments());
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let minimized = self.ctx.input(|input| input.viewport().minimized == Some(true));
        match (&self.geometry, minimized) {
            (Some(geometry), true) => storage.set_string(WINDOW_KEY, geometry.clone()),
            (_, true) => {}
            (_, false) => self.geometry = storage.get_string(WINDOW_KEY),
        }
        self.saved.visitor = self.yt.visitor();
        self.saved.position_ms = if self.playing() {
            self.player.position_ms()
        } else {
            self.resume_at.unwrap_or_else(|| self.player.position_ms())
        };
        match self.route() {
            Route::CustomPlaylist(id) => self.saved.last_playlist_id = Some(id.clone()),
            Route::Liked => self.saved.last_playlist_id = Some("liked".to_string()),
            Route::Home => self.saved.last_playlist_id = None,
            _ => {}
        }
        eframe::set_value(storage, eframe::APP_KEY, &self.saved);
    }
}

fn link_discord(sender: &Sender<Event>, ctx: &egui::Context) -> Discord {
    let (sender, ctx) = (sender.clone(), ctx.clone());
    Discord::start(move |linked| {
        let _ = sender.send(Event::Discord(linked));
        ctx.request_repaint();
    })
}

fn newer_version(release_url: &str, current: &str) -> Option<String> {
    let numbers = |version: &str| version.split('.').map(|part| part.parse().ok()).collect::<Option<Vec<u32>>>();
    let latest = release_url.rsplit_once("/tag/v")?.1;
    (numbers(latest)? > numbers(current)?).then(|| latest.to_owned())
}

fn reorder(queue: &mut Vec<Item>, upcoming: usize, from: usize, gap: usize) -> bool {
    let to = if gap > from { gap - 1 } else { gap };
    if from == to || from.min(to) < upcoming || from.max(to) >= queue.len() {
        return false;
    }
    let item = queue.remove(from);
    queue.insert(to, item);
    true
}

fn toggle(list: &mut Vec<Item>, item: Item) {
    match list.iter().position(|existing| existing.key() == item.key()) {
        Some(at) => drop(list.remove(at)),
        None => list.insert(0, item),
    }
}

fn shuffle(items: &mut [Item]) {
    let mut state = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(1, |elapsed| elapsed.as_nanos() as u64)
        | 1;
    for index in (1..items.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        items.swap(index, (state % (index as u64 + 1)) as usize);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offers_only_releases_newer_than_this_build() {
        let release = |tag: &str| format!("https://github.com/SuryakiranMK/AURI/releases/tag/{tag}");
        assert_eq!(newer_version(&release("v0.2.0"), "0.1.9").as_deref(), Some("0.2.0"));
        assert_eq!(newer_version(&release("v0.10.0"), "0.9.1").as_deref(), Some("0.10.0"));
        assert_eq!(newer_version(&release("v0.1.0"), "0.1.0"), None);
        assert_eq!(newer_version(&release("v0.1.0"), "0.2.0"), None);
        assert_eq!(newer_version(&release("nightly"), "0.1.0"), None);
        assert_eq!(newer_version("https://github.com/SuryakiranMK/AURI/releases", "0.1.0"), None);
    }

    #[test]
    fn reorders_only_the_upcoming_part_of_the_queue() {
        let order = |queue: &[Item]| queue.iter().map(|item| item.video_id.as_str()).collect::<String>();
        let mut queue: Vec<Item> = "abcde"
            .chars()
            .map(|id| Item {
                video_id: id.to_string(),
                ..Item::default()
            })
            .collect();
        assert!(reorder(&mut queue, 1, 1, 4));
        assert_eq!(order(&queue), "acdbe");
        assert!(reorder(&mut queue, 1, 4, 1));
        assert_eq!(order(&queue), "aecdb");
        assert!(reorder(&mut queue, 1, 1, 5));
        assert_eq!(order(&queue), "acdbe");
        for (from, gap) in [(2, 2), (2, 3), (0, 3), (3, 0), (9, 2), (2, 9)] {
            assert!(!reorder(&mut queue, 1, from, gap));
        }
        assert_eq!(order(&queue), "acdbe");
    }

    #[test]
    fn test_custom_playlist_lifecycle() {
        let mut pl = CustomPlaylist {
            id: "pl_123".into(),
            name: "Gaming Mix".into(),
            songs: Vec::new(),
        };
        let song = Item {
            video_id: "vid_1".into(),
            title: "Test Track".into(),
            subtitle: "Artist".into(),
            ..Item::default()
        };
        pl.songs.push(song.clone());
        assert_eq!(pl.songs.len(), 1);
        assert_eq!(pl.songs[0].title, "Test Track");

        // Serialize and deserialize test
        let json = serde_json::to_string(&pl).unwrap();
        let deserialized: CustomPlaylist = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, pl);

        // Remove song
        let pos = pl.songs.iter().position(|s| s.video_id == "vid_1");
        assert_eq!(pos, Some(0));
        pl.songs.remove(pos.unwrap());
        assert!(pl.songs.is_empty());
    }

    #[test]
    fn test_theme_base_and_mini_background_colors() {
        assert_eq!(ThemeBase::PureOled.color(), eframe::egui::Color32::from_rgb(0, 0, 0));
        assert_eq!(MiniBackground::PureOled.color(eframe::egui::Color32::RED), eframe::egui::Color32::from_rgb(0, 0, 0));
        let accent = eframe::egui::Color32::from_rgb(139, 92, 246);
        let match_col = MiniBackground::MatchAccent.color(accent);
        assert!(match_col.r() >= 8 && match_col.b() >= 12);
    }

    #[test]
    fn test_storage_dir_default() {
        let dir = default_storage_dir();
        assert!(!dir.is_empty());
        assert!(dir.contains("Music") || dir.contains("AURI"));
    }

    #[test]
    fn test_bar_layout_and_mini_sizes() {
        assert_eq!(BarLayout::ALL.len(), 3);
        assert_eq!(BarLayout::Docked.name(), "Docked Full-Width");
        assert_eq!(BarLayout::FloatingPill.name(), "Floating Deck");
        assert_eq!(BarLayout::MiniCapsule.name(), "Trim Mini Capsule");

        let saved = Saved::default();
        assert!(saved.sidebar_open);
        assert!(saved.queue_open);

        let (cw, ch) = MiniVariant::ClassicHud.size(false);
        assert_eq!((cw, ch), (384.0, 108.0));
        let (pw, ph) = MiniVariant::CapsulePill.size(false);
        assert_eq!((pw, ph), (330.0, 68.0));
        let (sw, sh) = MiniVariant::StudioCard.size(false);
        assert_eq!((sw, sh), (280.0, 390.0));
        let (sw_q, sh_q) = MiniVariant::StudioCard.size(true);
        assert_eq!((sw_q, sh_q), (280.0, 530.0));
    }

    #[test]
    fn test_stats_and_lyrics_customization() {
        let mut saved = Saved::default();
        assert!(saved.lyrics_synced_mode);
        assert_eq!(saved.lyrics_font_size, 24.0);
        assert!(saved.lyrics_align_center);
        assert_eq!(saved.lyrics_dim_unsung, 0.35);

        saved.stats_total_plays += 5;
        saved.stats_total_listen_ms += 120000;
        assert_eq!(saved.stats_total_plays, 5);
        assert_eq!(saved.stats_total_listen_ms, 120000);

        saved.stats_top_tracks.push(TrackStat {
            video_id: "test123".into(),
            title: "Test Song".into(),
            artist: "Test Artist".into(),
            thumb: String::new(),
            duration: "3:30".into(),
            plays: 5,
            listen_ms: 60000,
            last_played: 1000,
        });
        assert_eq!(saved.stats_top_tracks.len(), 1);
        assert_eq!(saved.stats_top_tracks[0].plays, 5);
    }

    #[test]
    fn test_stream_quality_crossfade_and_blocked_artists() {
        let mut saved = Saved::default();
        assert_eq!(saved.stream_quality, StreamQuality::High);
        assert!(!saved.crossfade_enabled);
        assert_eq!(saved.crossfade_secs, 3);
        assert!(saved.blocked_artists.is_empty());

        // Test stream quality names
        assert_eq!(StreamQuality::High.name(), "High Quality (160 kbps Opus)");
        assert_eq!(StreamQuality::Low.name(), "Data Saver (70 kbps Opus)");

        // Test blocked artists filter
        saved.blocked_artists.push("Disliked Band".into());
        let track1 = Item {
            video_id: "t1".into(),
            title: "Great Song".into(),
            subtitle: "Favorite Artist".into(),
            ..Item::default()
        };
        let track2 = Item {
            video_id: "t2".into(),
            title: "Bad Song".into(),
            subtitle: "Disliked Band ft. Someone".into(),
            ..Item::default()
        };

        let is_t1_blocked = saved.blocked_artists.iter().any(|b| track1.subtitle.to_lowercase().contains(&b.to_lowercase()));
        let is_t2_blocked = saved.blocked_artists.iter().any(|b| track2.subtitle.to_lowercase().contains(&b.to_lowercase()));
        assert!(!is_t1_blocked);
        assert!(is_t2_blocked);

        // Test unblock
        saved.blocked_artists.retain(|b| !b.eq_ignore_ascii_case("disliked band"));
        assert!(saved.blocked_artists.is_empty());
    }

    #[test]
    fn test_artwork_bg_variant_and_defaults() {
        assert_eq!(ArtworkBgVariant::ALL.len(), 4);
        assert_eq!(ArtworkBgVariant::CanvasTint.name(), "Canvas Tint");
        assert_eq!(ArtworkBgVariant::BlurredBackdrop.name(), "Frosted Blur");
        assert_eq!(ArtworkBgVariant::AmbientRadialGlow.name(), "Radial Glow");
        assert_eq!(ArtworkBgVariant::MinimalEdgeGlow.name(), "Edge Glow");

        let saved = Saved::default();
        assert_eq!(saved.artwork_bg_variant, ArtworkBgVariant::CanvasTint);
        assert!((saved.artwork_bg_opacity - 0.50).abs() < 0.01);
        assert!((saved.artwork_bg_blur - 0.60).abs() < 0.01);
    }
}
