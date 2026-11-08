use crate::app::{
    Action, App, BarLayout, Covers, CustomPlaylist, Load, MiniBackground, MiniPalette, MiniVariant, PerfProfile, Repeat, Route, SettingsTab, ThemeBase, Words,
};
use crate::art::Art;
use crate::discord::Headline;
use crate::hotkeys::{self, Command};
use crate::ytm::{Item, Layout};
use eframe::egui::ecolor::Hsva;
use eframe::egui::epaint::{RectShape, Shadow};
use eframe::egui::style::ScrollStyle;
use eframe::egui::text::{CCursor, LayoutJob, TextFormat, TextWrapping};
use eframe::egui::{
    self, Align, Align2, Color32, FontData, FontDefinitions, FontFamily, FontId, Frame, Galley, Id, Key, Margin, Pos2, Rect, Response,
    RichText, Sense, Shape, Stroke, StrokeKind, TextStyle, Ui, UiBuilder, Vec2, WidgetInfo, WidgetType, pos2, vec2,
};
use egui_phosphor::regular as icon;
use std::ops::Range;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

const BASE: Color32 = rgb(0x0A0B0E);
const PANEL: Color32 = rgb(0x07080A);
const RAISED: Color32 = rgb(0x13151D);
const TEXT: Color32 = rgb(0xECEDEF);
const MUTED: Color32 = Color32::from_rgba_premultiplied(166, 166, 166, 166);
const FAINT: Color32 = Color32::from_rgba_premultiplied(122, 122, 122, 122);
const ACCENT: Color32 = rgb(0x8B5CF6);
const ON_ACCENT: Color32 = rgb(0xFFFFFF);
const FLOATING: Color32 = rgb(0x181A24);

const SIDEBAR_WIDTH: f32 = 244.0;
const QUEUE_WIDTH: f32 = 336.0;
const QUEUE_MIN_WINDOW: f32 = 1140.0;
const BAR_HEIGHT: f32 = 84.0;
const ACCOUNT_HEIGHT: f32 = 68.0;
const UPDATE_HEIGHT: f32 = 50.0;
const TOPBAR_HEIGHT: f32 = 68.0;
const AMBIENT_HEIGHT: f32 = 380.0;
const HERO_ART: f32 = 216.0;
const BAR_ART: f32 = 52.0;
const ROW_HEIGHT: f32 = 56.0;
const ROW_RADIUS: f32 = 8.0;
const ART_RADIUS: f32 = 8.0;
const CARD_MIN: f32 = 164.0;
const CARD_GAP: f32 = 20.0;
const CARD_TEXT: f32 = 62.0;
const COMPACT_MIN: f32 = 320.0;
const COMPACT_ROWS: usize = 3;
const PAGE_MARGIN: i8 = 32;
const HOVER_TIME: f32 = 0.1;
const MENU_WIDTH: f32 = 248.0;
const QUEUE_ROW: f32 = 52.0;
const QUEUE_ART: f32 = 76.0;
const STICKY_HEIGHT: f32 = 60.0;
const STAGE_ART: f32 = 512.0;
const STAGE_LUMINANCE: f32 = 0.14;
const STAGE_BLUR: u8 = 84;
const PAGE_FADE: f64 = 0.12;
const SETTINGS_WIDTH: f32 = 640.0;
const SETTING_ROW: f32 = 48.0;
const HOTKEY_ROW: f32 = 56.0;
const STATUS_ROW: f32 = 26.0;
const SECTION_GAP: f32 = 32.0;
const CARD_PAD: f32 = 6.0;
const CARD_RADIUS: f32 = 12.0;

const fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

fn veil(opacity: f32) -> Color32 {
    Color32::from_white_alpha((opacity.clamp(0.0, 1.0) * 255.0) as u8)
}

fn shade(opacity: f32) -> Color32 {
    Color32::from_black_alpha((opacity.clamp(0.0, 1.0) * 255.0) as u8)
}

static FAMILIES: LazyLock<[FontFamily; 5]> =
    LazyLock::new(|| ["medium", "bold", "display", "icons", "icons-fill"].map(|name| FontFamily::Name(name.into())));

fn sans(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

fn medium(size: f32) -> FontId {
    FontId::new(size, FAMILIES[0].clone())
}

fn bold(size: f32) -> FontId {
    FontId::new(size, FAMILIES[1].clone())
}

fn display(size: f32) -> FontId {
    FontId::new(size, FAMILIES[2].clone())
}

fn glyph(size: f32) -> FontId {
    FontId::new(size, FAMILIES[3].clone())
}

fn solid(size: f32) -> FontId {
    FontId::new(size, FAMILIES[4].clone())
}

fn system_font(file: &str) -> Option<&'static [u8]> {
    let fonts = if cfg!(target_os = "macos") {
        std::path::PathBuf::from("/System/Library/Fonts")
    } else {
        std::path::Path::new(&std::env::var_os("WINDIR")?).join("Fonts")
    };
    mapped(&fonts.join(file))
}

fn mapped(path: &std::path::Path) -> Option<&'static [u8]> {
    let file = std::fs::File::open(path).ok()?;
    let map = unsafe { memmap2::Mmap::map(&file) }.ok()?;
    Some(&Box::leak(Box::new(map))[..])
}

/// Fonts each system ships for the scripts the fonts above lack, in fallback order. Every Noto font follows them, the sans ones first.
const FALLBACK_FONTS: [&str; 43] = [
    // Linux
    "NotoSansCJK-Regular.ttc",
    "NotoSansCJK-VF.ttc",
    "DroidSansFallbackFull.ttf",
    "wqy-microhei.ttc",
    "NanumGothic.ttf",
    "Loma.ttf",
    "Lohit-Devanagari.ttf",
    "DejaVuSans.ttf",
    "FreeSans.ttf",
    // macOS
    "SFArabic.ttf",
    "GeezaPro.ttc",
    "SFHebrew.ttf",
    "SFArmenian.ttf",
    "SFGeorgian.ttf",
    "KohinoorBangla.ttc",
    "KohinoorGujarati.ttc",
    "KohinoorTelugu.ttc",
    "MuktaMahee.ttc",
    "Tamil Sangam MN.ttc",
    "Kannada Sangam MN.ttc",
    "Malayalam Sangam MN.ttc",
    "Oriya Sangam MN.ttc",
    "Sinhala Sangam MN.ttc",
    "Khmer Sangam MN.ttf",
    "Lao Sangam MN.ttf",
    "Myanmar Sangam MN.ttc",
    "KefaIII.ttf",
    "Kailasa.ttc",
    "Arial Unicode.ttf",
    // Windows
    "msjh.ttc",
    "YuGothR.ttc",
    "ebrima.ttf",
    "mmrtext.ttf",
    "himalaya.ttf",
    "monbaiti.ttf",
    "gadugi.ttf",
    "javatext.ttf",
    "msyi.ttf",
    "taile.ttf",
    "ntailu.ttf",
    "phagspa.ttf",
    "seguihis.ttf",
    "simsunb.ttf",
];

fn fallback_fonts() -> Vec<(String, &'static [u8])> {
    let mut dirs: Vec<std::path::PathBuf> = [
        "/usr/share/fonts",
        "/usr/local/share/fonts",
        "/System/Library/Fonts",
        "/Library/Fonts",
    ]
    .map(Into::into)
    .into();
    for (var, below) in [
        ("HOME", ".local/share/fonts"),
        ("HOME", ".fonts"),
        ("HOME", "Library/Fonts"),
        ("WINDIR", "Fonts"),
    ] {
        dirs.extend(std::env::var_os(var).map(|root| std::path::Path::new(&root).join(below)));
    }
    let mut found = Vec::new();
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let noto = name.starts_with("Noto")
                && !name.contains("Italic")
                && ["-Regular.ttf", "-Regular.otf", "[wght].ttf"].iter().any(|end| name.ends_with(end));
            let listed = FALLBACK_FONTS.iter().position(|font| name.eq_ignore_ascii_case(font));
            if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                dirs.push(entry.path());
            } else if listed.is_some() || noto {
                let rank = listed.unwrap_or(FALLBACK_FONTS.len() + usize::from(!name.starts_with("NotoSans")));
                found.push((rank, name.to_lowercase(), entry.path()));
            }
        }
    }
    found.sort();
    found.dedup_by(|a, b| a.1 == b.1);
    found
        .into_iter()
        .filter_map(|(_, name, path)| Some((name, mapped(&path)?)))
        .collect()
}

pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let embedded: [(&str, &'static [u8]); 8] = [
        ("onest-400", include_bytes!("../assets/fonts/onest-latin-400.ttf")),
        ("onest-400-ext", include_bytes!("../assets/fonts/onest-latin-ext-400.ttf")),
        ("onest-500", include_bytes!("../assets/fonts/onest-latin-500.ttf")),
        ("onest-500-ext", include_bytes!("../assets/fonts/onest-latin-ext-500.ttf")),
        ("onest-600", include_bytes!("../assets/fonts/onest-latin-600.ttf")),
        ("onest-600-ext", include_bytes!("../assets/fonts/onest-latin-ext-600.ttf")),
        (
            "shoulders-800",
            include_bytes!("../assets/fonts/big-shoulders-display-latin-800.ttf"),
        ),
        (
            "shoulders-800-ext",
            include_bytes!("../assets/fonts/big-shoulders-display-latin-ext-800.ttf"),
        ),
    ];
    for (name, bytes) in embedded {
        fonts.font_data.insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
    }
    fonts
        .font_data
        .insert("phosphor".to_owned(), Arc::new(egui_phosphor::Variant::Regular.font_data()));
    fonts
        .font_data
        .insert("phosphor-fill".to_owned(), Arc::new(egui_phosphor::Variant::Fill.font_data()));
    let system: &[_] = if cfg!(target_os = "macos") {
        &[
            ("segoe", "SFNS.ttf"),
            ("segoe-bold", "SFNS.ttf"),
            ("yahei", "Hiragino Sans GB.ttc"),
            ("malgun", "AppleSDGothicNeo.ttc"),
            ("thai", "ThonburiUI.ttc"),
            ("indic", "Kohinoor.ttc"),
            ("symbols", "Apple Symbols.ttf"),
        ]
    } else {
        &[
            ("segoe", "segoeui.ttf"),
            ("segoe-bold", "segoeuib.ttf"),
            ("yahei", "msyh.ttc"),
            ("malgun", "malgun.ttf"),
            ("thai", "leelawui.ttf"),
            ("indic", "Nirmala.ttc"),
            ("indic-legacy", "Nirmala.ttf"),
            ("symbols", "seguisym.ttf"),
        ]
    };
    for &(name, file) in system {
        if let Some(bytes) = system_font(file) {
            fonts.font_data.insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
        }
    }
    let fallback = fallback_fonts();
    for (name, bytes) in &fallback {
        fonts.font_data.insert(name.clone(), Arc::new(FontData::from_static(bytes)));
    }
    let known = fonts.font_data.clone();
    let stack = |own: [&str; 2], system: &str| -> Vec<String> {
        let fallbacks = [
            system,
            "NotoEmoji-Regular",
            "emoji-icon-font",
            "yahei",
            "malgun",
            "thai",
            "indic",
            "indic-legacy",
            "symbols",
            "Ubuntu-Light",
        ];
        own.into_iter()
            .chain(fallbacks)
            .chain(fallback.iter().map(|(name, _)| name.as_str()))
            .filter(|name| known.contains_key(*name))
            .map(str::to_owned)
            .collect()
    };
    fonts
        .families
        .insert(FontFamily::Proportional, stack(["onest-400", "onest-400-ext"], "segoe"));
    fonts
        .families
        .insert(FAMILIES[0].clone(), stack(["onest-500", "onest-500-ext"], "segoe"));
    fonts
        .families
        .insert(FAMILIES[1].clone(), stack(["onest-600", "onest-600-ext"], "segoe-bold"));
    fonts
        .families
        .insert(FAMILIES[2].clone(), stack(["shoulders-800", "shoulders-800-ext"], "segoe-bold"));
    fonts.families.insert(FAMILIES[3].clone(), vec!["phosphor".to_owned()]);
    fonts.families.insert(FAMILIES[4].clone(), vec!["phosphor-fill".to_owned()]);
    ctx.set_fonts(fonts);

    ctx.style_mut(|style| {
        let visuals = &mut style.visuals;
        *visuals = egui::Visuals::dark();
        visuals.panel_fill = BASE;
        visuals.window_fill = FLOATING;
        visuals.extreme_bg_color = PANEL;
        visuals.override_text_color = Some(TEXT);
        visuals.window_stroke = Stroke::new(1.0_f32, veil(0.08));
        visuals.window_corner_radius = 10.into();
        visuals.menu_corner_radius = 13.into();
        visuals.popup_shadow = Shadow {
            offset: [0, 10],
            blur: 28,
            spread: 0,
            color: shade(0.5),
        };
        visuals.selection.bg_fill = ACCENT.gamma_multiply(0.45);
        visuals.selection.stroke = Stroke::new(1.0_f32, TEXT);
        visuals.text_cursor.stroke = Stroke::new(1.5_f32, ACCENT);
        visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);
        for (widget, fill) in [
            (&mut visuals.widgets.inactive, Color32::TRANSPARENT),
            (&mut visuals.widgets.hovered, veil(0.08)),
            (&mut visuals.widgets.active, veil(0.12)),
            (&mut visuals.widgets.open, veil(0.08)),
        ] {
            widget.bg_fill = fill;
            widget.weak_bg_fill = fill;
            widget.bg_stroke = Stroke::NONE;
            widget.fg_stroke = Stroke::new(1.0_f32, TEXT);
            widget.corner_radius = 6.into();
            widget.expansion = 0.0;
        }
        style.spacing.item_spacing = Vec2::ZERO;
        style.spacing.button_padding = vec2(12.0, 8.0);
        style.spacing.menu_margin = Margin::same(6);
        style.spacing.scroll = ScrollStyle {
            bar_width: 8.0,
            floating_width: 4.0,
            dormant_handle_opacity: 0.0,
            ..ScrollStyle::floating()
        };
        style.interaction.selectable_labels = false;
        style.animation_time = HOVER_TIME;
        style.text_styles = [
            (TextStyle::Body, sans(14.0)),
            (TextStyle::Button, medium(13.5)),
            (TextStyle::Small, sans(12.0)),
            (TextStyle::Heading, bold(20.0)),
            (TextStyle::Monospace, FontId::monospace(13.0)),
        ]
        .into();
    });
}

pub fn app_icon() -> egui::IconData {
    if let Ok(img) = image::load_from_memory(include_bytes!("../assets/brand_64.png")) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        return egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        };
    }
    const SIZE: usize = 128;
    const SAMPLES: usize = 3;
    let mut rgba = Vec::with_capacity(SIZE * SIZE * 4);
    for pixel in 0..SIZE * SIZE {
        let (mut covered, mut sum) = (0.0, [0.0; 3]);
        for sample in 0..SAMPLES * SAMPLES {
            let x = (pixel % SIZE) as f32 + ((sample % SAMPLES) as f32 + 0.5) / SAMPLES as f32;
            let y = (pixel / SIZE) as f32 + ((sample / SAMPLES) as f32 + 0.5) / SAMPLES as f32;
            let radius = (x / SIZE as f32 * 2.0 - 1.0).hypot(y / SIZE as f32 * 2.0 - 1.0);
            if radius < 0.96 {
                let color = if radius < 0.17 || (0.53..0.57).contains(&radius) {
                    PANEL
                } else {
                    ACCENT
                };
                covered += 1.0;
                sum = [sum[0] + color.r() as f32, sum[1] + color.g() as f32, sum[2] + color.b() as f32];
            }
        }
        let alpha = covered / (SAMPLES * SAMPLES) as f32 * 255.0;
        rgba.extend(sum.map(|channel| (channel / f32::max(covered, 1.0)) as u8));
        rgba.push(alpha as u8);
    }
    egui::IconData {
        rgba,
        width: SIZE as u32,
        height: SIZE as u32,
    }
}

struct Cx<'a> {
    art: &'a Art,
    covers: &'a Covers,
    playing: &'a str,
    paused: bool,
    liked: &'a [Item],
    single: bool,
    playlists: &'a [CustomPlaylist],
    out: &'a mut Vec<Action>,
}

impl Cx<'_> {
    fn is_liked(&self, item: &Item) -> bool {
        item.is_song() && self.liked.iter().any(|liked| liked.video_id == item.video_id)
    }

    fn is_playing(&self, item: &Item) -> bool {
        item.is_song() && item.video_id == self.playing
    }

    fn activate(&mut self, items: &[Item], index: usize) {
        let item = &items[index];
        if self.is_playing(item) {
            self.out.push(Action::Toggle);
        } else if item.is_song() {
            let (queue, at) = if self.single {
                (vec![item.clone()], 0)
            } else {
                (items.to_vec(), index)
            };
            self.out.push(Action::Play(queue, at));
        } else if !item.browse_id.is_empty() {
            self.out.push(Action::Go(Route::Browse(item.browse_id.clone())));
        }
    }
}

struct Scrub {
    bar: Rect,
    played: f32,
    buffered: f32,
    hover: f32,
    aim: Option<(f32, u64)>,
}

#[cfg(windows)]
pub struct TitleBar {
    window: isize,
    painted: Option<Color32>,
}

#[cfg(windows)]
#[repr(C)]
struct Margins {
    cx_left_width: i32,
    cx_right_width: i32,
    cy_top_height: i32,
    cy_bottom_height: i32,
}

#[cfg(windows)]
impl TitleBar {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let window = match cc.window_handle().map(|handle| handle.as_raw()) {
            Ok(RawWindowHandle::Win32(window)) => window.hwnd.get(),
            _ => 0,
        };
        Self { window, painted: None }
    }

    fn paint(&mut self, color: Color32) {
        use windows_sys::Win32::Graphics::Dwm::{
            DWMWA_CAPTION_COLOR, DWMWA_TEXT_COLOR, DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute,
        };
        if self.window == 0 || self.painted == Some(color) {
            return;
        }
        self.painted = Some(color);
        let packed = |color: Color32| color.r() as u32 | (color.g() as u32) << 8 | (color.b() as u32) << 16;
        let attributes = [
            (DWMWA_USE_IMMERSIVE_DARK_MODE, 1),
            (DWMWA_CAPTION_COLOR, packed(color)),
            (DWMWA_TEXT_COLOR, packed(TEXT)),
        ];
        for (attribute, value) in attributes {
            unsafe { DwmSetWindowAttribute(self.window as _, attribute as u32, (&raw const value).cast(), 4) };
        }
    }

    pub fn enable_transparency(&self) {
        #[link(name = "dwmapi")]
        unsafe extern "system" {
            fn DwmExtendFrameIntoClientArea(hwnd: isize, pMarInset: *const Margins) -> i32;
            fn DwmSetWindowAttribute(hwnd: isize, dwAttribute: u32, pvAttribute: *const std::ffi::c_void, cbAttribute: u32) -> i32;
        }

        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_STYLE, SWP_FRAMECHANGED, SWP_NOMOVE,
            SWP_NOSIZE, SWP_NOZORDER, WS_CAPTION, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_SYSMENU, WS_THICKFRAME,
        };

        if self.window != 0 {
            unsafe {
                // Strip all window caption and button styles so Windows DWM never renders minimize/maximize/close buttons
                let style = GetWindowLongPtrW(self.window as _, GWL_STYLE) as u32;
                let new_style = style & !(WS_CAPTION | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU | WS_THICKFRAME);
                SetWindowLongPtrW(self.window as _, GWL_STYLE, new_style as isize);
                SetWindowPos(self.window as _, std::ptr::null_mut(), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED);

                // Disable Windows 11 system outer border (DWMWA_BORDER_COLOR = 34, 0xFFFFFFFE is DWMWA_COLOR_NONE)
                let border_color: u32 = 0xFFFFFFFE;
                DwmSetWindowAttribute(self.window as _, 34, (&raw const border_color).cast(), 4);

                let margins = Margins {
                    cx_left_width: -1,
                    cx_right_width: -1,
                    cy_top_height: -1,
                    cy_bottom_height: -1,
                };
                DwmExtendFrameIntoClientArea(self.window, &margins);
            }
        }
    }

    pub fn restore_decorations(&self) {
        #[link(name = "dwmapi")]
        unsafe extern "system" {
            fn DwmExtendFrameIntoClientArea(hwnd: isize, pMarInset: *const Margins) -> i32;
            fn DwmSetWindowAttribute(hwnd: isize, dwAttribute: u32, pvAttribute: *const std::ffi::c_void, cbAttribute: u32) -> i32;
        }

        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_STYLE, SWP_FRAMECHANGED, SWP_NOMOVE,
            SWP_NOSIZE, SWP_NOZORDER, WS_CAPTION, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_SYSMENU, WS_THICKFRAME,
        };

        if self.window != 0 {
            unsafe {
                let style = GetWindowLongPtrW(self.window as _, GWL_STYLE) as u32;
                let new_style = style | WS_CAPTION | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU | WS_THICKFRAME;
                SetWindowLongPtrW(self.window as _, GWL_STYLE, new_style as isize);
                SetWindowPos(self.window as _, std::ptr::null_mut(), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED);

                let border_color: u32 = 0xFFFFFFFF;
                DwmSetWindowAttribute(self.window as _, 34, (&raw const border_color).cast(), 4);

                let margins = Margins {
                    cx_left_width: 0,
                    cx_right_width: 0,
                    cy_top_height: 0,
                    cy_bottom_height: 0,
                };
                DwmExtendFrameIntoClientArea(self.window, &margins);
            }
        }
    }
}

fn paint_glass_grain(painter: &egui::Painter, rect: Rect, strength: f32) {
    if strength <= 0.01 {
        return;
    }
    let step = 6.0;
    let mut y = rect.top() + 3.0;
    let max_y = rect.bottom() - 3.0;
    let max_x = rect.right() - 3.0;
    let base_a = (strength * 75.0).clamp(3.0, 50.0) as u8;

    while y < max_y {
        let mut x = rect.left() + 3.0;
        while x < max_x {
            // Fast pseudo-random hash
            let h = ((x * 12.9898 + y * 78.233).sin() * 43758.5453).fract().abs();
            if h > 0.42 {
                let dot_alpha = ((h - 0.42) * 1.7 * base_a as f32) as u8;
                let col = if h > 0.70 {
                    Color32::from_rgba_unmultiplied(255, 255, 255, dot_alpha)
                } else {
                    Color32::from_rgba_unmultiplied(0, 0, 0, dot_alpha)
                };
                painter.rect_filled(Rect::from_min_size(pos2(x, y), vec2(1.5, 1.5)), 0.5, col);
            }
            x += step;
        }
        y += step;
    }
}

#[cfg(not(windows))]
pub struct TitleBar;

#[cfg(not(windows))]
impl TitleBar {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self
    }

    fn paint(&mut self, _color: Color32) {}

    pub fn enable_transparency(&self) {}

    pub fn restore_decorations(&self) {}
}

pub fn draw(app: &mut App, ctx: &egui::Context) -> Vec<Action> {
    let mut out = Vec::new();
    let plain = |fill: Color32| Frame::new().fill(fill);
    let bar_frame = if app.saved.bar_glass || app.saved.bar_layout != BarLayout::Docked {
        Frame::new().fill(Color32::TRANSPARENT)
    } else {
        plain(PANEL)
    };
    let scrub = egui::TopBottomPanel::bottom("player")
        .exact_height(BAR_HEIGHT)
        .show_separator_line(false)
        .frame(bar_frame)
        .show(ctx, |ui| player_bar(app, ui, &mut out))
        .inner;
    let mut frame = PANEL;
    if app.expanded && app.now_playing().is_some() {
        frame = egui::CentralPanel::default()
            .frame(plain(PANEL))
            .show(ctx, |ui| now_playing(app, ui, &mut out))
            .inner;
    } else {
        if app.saved.sidebar_open {
            egui::SidePanel::left("sidebar")
                .exact_width(SIDEBAR_WIDTH)
                .resizable(false)
                .show_separator_line(false)
                .frame(plain(PANEL))
                .show(ctx, |ui| sidebar(app, ui, &mut out));
        }
        let queue_fits = app.saved.queue_open && ctx.content_rect().width() >= QUEUE_MIN_WINDOW;
        if app.saved.queue_open {
            egui::SidePanel::right("queue")
                .exact_width(QUEUE_WIDTH)
                .resizable(false)
                .show_separator_line(false)
                .frame(plain(PANEL))
                .show_animated(ctx, queue_fits, |ui| queue_panel(app, ui, &mut out));
        }
        egui::CentralPanel::default()
            .frame(plain(BASE))
            .show(ctx, |ui| content(app, ui, &mut out));

        if !app.saved.sidebar_open {
            let screen = ctx.content_rect();
            let tab_y = screen.center().y - 28.0;
            egui::Area::new(Id::new("sidebar-expand-tab"))
                .order(egui::Order::Foreground)
                .fixed_pos(pos2(0.0, tab_y))
                .show(ctx, |ui| {
                    let (tab_rect, resp) = ui.allocate_exact_size(vec2(22.0, 56.0), Sense::click());
                    let hover = hover_of(ui, &resp);
                    let pal_color = app.saved.theme_palette.color();
                    ui.painter().rect_filled(tab_rect, 8.0, veil(0.12 + 0.10 * hover));
                    ui.painter().rect_stroke(tab_rect, 8.0, Stroke::new(1.0_f32, veil(0.18 + 0.12 * hover)), StrokeKind::Inside);
                    ui.painter().text(tab_rect.center(), Align2::CENTER_CENTER, icon::CARET_RIGHT, solid(15.0), if hover > 0.1 { pal_color } else { MUTED });
                    describe(&resp, "Expand navigation sidebar");
                    if resp.clicked() {
                        out.push(Action::ToggleSidebar);
                    }
                });
        }

        if !app.saved.queue_open {
            let screen = ctx.content_rect();
            let tab_y = screen.center().y - 28.0;
            egui::Area::new(Id::new("queue-expand-tab"))
                .order(egui::Order::Foreground)
                .fixed_pos(pos2(screen.right() - 22.0, tab_y))
                .show(ctx, |ui| {
                    let (tab_rect, resp) = ui.allocate_exact_size(vec2(22.0, 56.0), Sense::click());
                    let hover = hover_of(ui, &resp);
                    let pal_color = app.saved.theme_palette.color();
                    ui.painter().rect_filled(tab_rect, 8.0, veil(0.12 + 0.10 * hover));
                    ui.painter().rect_stroke(tab_rect, 8.0, Stroke::new(1.0_f32, veil(0.18 + 0.12 * hover)), StrokeKind::Inside);
                    ui.painter().text(tab_rect.center(), Align2::CENTER_CENTER, icon::CARET_LEFT, solid(15.0), if hover > 0.1 { pal_color } else { MUTED });
                    describe(&resp, "Expand song queue");
                    if resp.clicked() {
                        out.push(Action::ToggleQueue);
                    }
                });
        }
    }
    app.title_bar.paint(frame);
    paint_scrubber(ctx, &scrub);
    sign_in_dialog(app, ctx, &mut out);
    notice(app, ctx, &mut out);
    out
}

fn fit(ui: &Ui, text: &str, font: FontId, color: Color32, width: f32, rows: usize) -> Arc<Galley> {
    let mut job = LayoutJob::simple_singleline(text.to_owned(), font, color);
    job.wrap = TextWrapping {
        max_width: width.max(1.0),
        max_rows: rows,
        break_anywhere: rows == 1,
        overflow_character: Some('…'),
    };
    ui.painter().layout_job(job)
}

fn label(ui: &Ui, at: Pos2, anchor: Align2, text: &str, font: FontId, color: Color32, width: f32) -> Rect {
    let galley = fit(ui, text, font, color, width, 1);
    let rect = anchor.anchor_size(at, galley.size());
    ui.painter().galley(rect.min, galley, color);
    rect
}

fn artist_spans(item: &Item) -> Vec<(Range<usize>, &str)> {
    let subtitle = item.subtitle.as_str();
    if item.artists.is_empty() && !item.artist_id.is_empty() {
        return vec![(0..subtitle.chars().count(), &item.artist_id)];
    }
    let mut searched = 0;
    let mut spans = Vec::new();
    for (name, browse_id) in &item.artists {
        let later = subtitle[searched..].find(name.as_str()).map(|at| searched + at);
        let Some(start) = later.or_else(|| subtitle.find(name.as_str())) else {
            continue;
        };
        searched = start + name.len();
        let first = subtitle[..start].chars().count();
        spans.push((first..first + name.chars().count(), browse_id.as_str()));
    }
    spans
}

fn artist_links(ui: &Ui, out: &mut Vec<Action>, item: &Item, id: Id, origin: Pos2, galley: &Arc<Galley>) {
    if item.is_artist {
        return;
    }
    for (index, (chars, browse_id)) in artist_spans(item).into_iter().enumerate() {
        let edge = |at: usize| galley.pos_from_cursor(CCursor::new(at));
        let rect = Rect::from_min_max(edge(chars.start).min, edge(chars.end).max).translate(origin.to_vec2() + galley.rect.min.to_vec2());
        if rect.width() < 1.0 {
            continue;
        }
        let response = ui.interact(rect, id.with(index), Sense::click());
        let hover = hover_of(ui, &response);
        if hover > 0.0 {
            ui.painter()
                .with_clip_rect(rect)
                .galley_with_override_text_color(origin, galley.clone(), MUTED.lerp_to_gamma(TEXT, hover));
            let line = Stroke::new(1.0_f32, TEXT.gamma_multiply(hover));
            ui.painter().hline(rect.x_range(), rect.bottom() - 1.0, line);
        }
        describe(&response, "Go to artist");
        if response.clicked() {
            out.push(Action::Go(Route::Browse(browse_id.to_owned())));
        }
    }
}

fn byline(ui: &Ui, out: &mut Vec<Action>, item: &Item, id: Id, at: Pos2, font: FontId, width: f32) -> Rect {
    let galley = fit(ui, &item.subtitle, font, MUTED, width, 1);
    let rect = Align2::LEFT_CENTER.anchor_size(at, galley.size());
    ui.painter().galley(rect.min, galley.clone(), MUTED);
    artist_links(ui, out, item, id, rect.min, &galley);
    rect
}

fn paragraph(ui: &mut Ui, text: &str, font: FontId, color: Color32, rows: usize) {
    let galley = fit(ui, text, font, color, ui.available_width(), rows);
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), galley.size().y), Sense::hover());
    ui.painter().galley(rect.min, galley, color);
}

fn eyebrow(ui: &mut Ui, text: &str, inset: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 30.0), Sense::hover());
    let format = TextFormat {
        font_id: bold(11.0),
        color: FAINT,
        extra_letter_spacing: 1.3,
        ..TextFormat::default()
    };
    let mut job = LayoutJob::default();
    job.append(&text.to_uppercase(), 0.0, format);
    let galley = ui.painter().layout_job(job);
    ui.painter()
        .galley(pos2(rect.left() + inset, rect.center().y - galley.size().y / 2.0), galley, FAINT);
}

fn hover_of(ui: &Ui, response: &Response) -> f32 {
    ui.ctx()
        .animate_bool_with_time(response.id, response.hovered() || response.has_focus(), HOVER_TIME)
}

fn describe(response: &Response, name: &str) {
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, name));
    point_at(response);
}

fn point_at(response: &Response) {
    if response.hovered() {
        response.ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
    }
}

fn icon_button(ui: &Ui, rect: Rect, id: Id, symbol: &str, font: FontId, color: Color32, hint: &str) -> Response {
    let response = ui.interact(rect, id, Sense::click());
    let hover = hover_of(ui, &response);
    let pressed = if response.is_pointer_button_down_on() { 1.0 } else { 0.0 };
    ui.painter()
        .circle_filled(rect.center(), rect.width() / 2.0 - pressed, veil(0.09 * hover));
    let tone = if color == MUTED || color == FAINT {
        color.lerp_to_gamma(TEXT, hover)
    } else {
        color
    };
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, symbol, font, tone);
    describe(&response, hint);
    response.on_hover_text(hint)
}

fn text_button(ui: &Ui, right_center: Pos2, id: Id, text: &str) -> Response {
    let galley = fit(ui, text, medium(13.0), MUTED, 120.0, 1);
    let slot = Align2::RIGHT_CENTER
        .anchor_size(right_center, galley.size())
        .expand2(vec2(10.0, 6.0));
    let response = ui.interact(slot, id, Sense::click());
    let hover = hover_of(ui, &response);
    ui.painter().rect_filled(slot, 14.0, veil(0.07 * hover));
    ui.painter()
        .galley_with_override_text_color(slot.shrink2(vec2(10.0, 6.0)).min, galley, MUTED.lerp_to_gamma(TEXT, hover));
    describe(&response, text);
    response
}

fn pill(ui: &Ui, at: Pos2, id: Id, symbol: &str, text: &str, primary: bool) -> Response {
    let galley = fit(ui, text, bold(14.0), if primary { ON_ACCENT } else { TEXT }, 240.0, 1);
    let rect = Rect::from_min_size(at, vec2(galley.size().x + 62.0, 42.0));
    let response = ui.interact(rect, id, Sense::click());
    let hover = hover_of(ui, &response);
    let body = if response.is_pointer_button_down_on() {
        rect.shrink(1.0)
    } else {
        rect
    };
    let fill = if primary {
        ACCENT.lerp_to_gamma(Color32::WHITE, 0.12 * hover)
    } else {
        veil(0.09 + 0.05 * hover)
    };
    let ink = if primary { ON_ACCENT } else { TEXT };
    ui.painter().rect_filled(body, 21.0, fill);
    ui.painter().text(
        pos2(rect.left() + 28.0, rect.center().y),
        Align2::CENTER_CENTER,
        symbol,
        solid(16.0),
        ink,
    );
    ui.painter()
        .galley(pos2(rect.left() + 42.0, rect.center().y - galley.size().y / 2.0), galley, ink);
    describe(&response, text);
    response
}

fn artwork(ui: &Ui, art: &Art, url: &str, rect: Rect, radius: f32) {
    match art.get(ui.ctx(), url, rect.width()) {
        Some(picture) => egui::Image::from_texture((picture.id, rect.size()))
            .corner_radius(radius)
            .paint_at(ui, rect),
        None => paint_shimmer_wave(ui.painter(), ui.ctx(), rect, radius),
    }
}

fn paint_shimmer_wave(painter: &egui::Painter, ctx: &egui::Context, rect: Rect, radius: f32) {
    ctx.request_repaint_after(Duration::from_millis(33));
    let time = ctx.input(|i| i.time);
    painter.rect_filled(rect, radius, RAISED);

    // Traveling wave shimmer pulse across the card
    let phase = (time * 1.2).fract() as f32;
    let wave_cx = rect.left() + (rect.width() + 100.0) * phase - 50.0;
    let wave_w = (rect.width() * 0.45).max(35.0);
    let wave_rect = Rect::from_min_max(
        pos2((wave_cx - wave_w).max(rect.left()), rect.top()),
        pos2((wave_cx + wave_w).min(rect.right()), rect.bottom()),
    );
    if wave_rect.is_positive() {
        let alpha = ((time * 3.0).sin().abs() as f32 * 0.08 + 0.08).clamp(0.04, 0.20);
        painter.rect_filled(wave_rect, radius, Color32::from_white_alpha((alpha * 255.0) as u8));
    }

    // Subtle pulsing wave equalizer bars in the center
    let pulse = (time * 4.0).sin().abs() as f32;
    let bar_color = ACCENT.gamma_multiply(0.25 + 0.20 * pulse);
    let cy = rect.center().y;
    let cx = rect.center().x;
    for (i, h_factor) in [0.6, 1.0, 0.75, 0.4].iter().enumerate() {
        let x = cx + (i as f32 - 1.5) * 6.0;
        let h = (rect.height() * 0.25).min(16.0) * h_factor * (0.6 + 0.4 * pulse);
        painter.rect_filled(Rect::from_center_size(pos2(x, cy), vec2(2.5, h)), 1.25, bar_color);
    }
}

fn rotating_circular_artwork(ui: &Ui, art: &Art, url: &str, center: Pos2, radius: f32, angle: f32, pal_color: Color32) {
    if let Some(picture) = art.get(ui.ctx(), url, radius * 2.0) {
        let segments = 32;
        let mut mesh = egui::Mesh::with_texture(picture.id);
        mesh.vertices.push(egui::epaint::Vertex {
            pos: center,
            uv: pos2(0.5, 0.5),
            color: Color32::WHITE,
        });

        for i in 0..segments {
            let theta = i as f32 * std::f32::consts::TAU / segments as f32;
            let pos = pos2(center.x + radius * theta.cos(), center.y + radius * theta.sin());
            let uv_theta = theta + angle;
            let uv = pos2(0.5 + 0.5 * uv_theta.cos(), 0.5 + 0.5 * uv_theta.sin());
            mesh.vertices.push(egui::epaint::Vertex {
                pos,
                uv,
                color: Color32::WHITE,
            });
        }

        for i in 1..segments {
            mesh.indices.extend_from_slice(&[0, i as u32, (i + 1) as u32]);
        }
        mesh.indices.extend_from_slice(&[0, segments as u32, 1]);

        ui.painter().add(Shape::mesh(mesh));

        // Vinyl center spindle hole & groove rings
        ui.painter().circle_filled(center, 3.5, Color32::BLACK);
        ui.painter().circle_stroke(center, 3.5, Stroke::new(1.0_f32, veil(0.35)));
        ui.painter().circle_stroke(center, radius * 0.55, Stroke::new(1.0_f32, veil(0.08)));
        ui.painter().circle_stroke(center, radius, Stroke::new(1.0_f32, veil(0.18)));
    } else {
        ui.painter().circle_filled(center, radius, veil(0.06));
        ui.painter().text(center, Align2::CENTER_CENTER, icon::VINYL_RECORD, glyph(20.0), pal_color);
        ui.painter().circle_stroke(center, radius, Stroke::new(1.0_f32, pal_color.gamma_multiply(0.40)));
    }
}

fn bars(ui: &Ui, center: Pos2, moving: bool) {
    let time = if moving { ui.input(|input| input.time) } else { 0.9 };
    for bar in 0..3 {
        let wave = (time * (5.2 + bar as f64 * 1.9) + bar as f64 * 2.1).sin() as f32;
        let x = center.x + (bar as f32 - 1.0) * 5.0;
        let top = center.y + 7.0 - (4.0 + 5.0 * (1.0 + wave));
        ui.painter()
            .rect_filled(Rect::from_min_max(pos2(x - 1.5, top), pos2(x + 1.5, center.y + 7.0)), 1.0, ACCENT);
    }
    if moving && ui.input(|input| input.focused) {
        ui.ctx().request_repaint_after(Duration::from_millis(66));
    }
}

fn clock(ms: u64) -> String {
    format!("{}:{:02}", ms / 60_000, ms / 1000 % 60)
}

fn menu_row(ui: &mut Ui, symbol: &str, font: FontId, tint: Color32, text: &str) -> Response {
    wide_row(ui, vec2(MENU_WIDTH, 38.0), symbol, font, tint, text)
}

fn wide_row(ui: &mut Ui, size: Vec2, symbol: &str, font: FontId, tint: Color32, text: &str) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let hover = hover_of(ui, &response);
    ui.painter().rect_filled(rect, 7.0, veil(0.08 * hover));
    let tone = if tint == ACCENT { tint } else { tint.lerp_to_gamma(TEXT, hover) };
    ui.painter()
        .text(pos2(rect.left() + 22.0, rect.center().y), Align2::CENTER_CENTER, symbol, font, tone);
    label(
        ui,
        pos2(rect.left() + 46.0, rect.center().y),
        Align2::LEFT_CENTER,
        text,
        medium(13.5),
        TEXT,
        rect.width() - 58.0,
    );
    describe(&response, text);
    response
}

fn switch_row(ui: &mut Ui, size: Vec2, symbol: &str, text: &str, on: bool) -> bool {
    let response = wide_row(ui, size, symbol, glyph(17.0), MUTED, text);
    let lit = ui.ctx().animate_bool_with_time(response.id.with("switch"), on, HOVER_TIME);
    let track = Rect::from_center_size(pos2(response.rect.right() - 27.0, response.rect.center().y), vec2(30.0, 18.0));
    ui.painter().rect_filled(track, 9.0, veil(0.16).lerp_to_gamma(ACCENT, lit));
    let knob = pos2(egui::lerp(track.left() + 9.0..=track.right() - 9.0, lit), track.center().y);
    ui.painter().circle_filled(knob, 6.0, TEXT.lerp_to_gamma(ON_ACCENT, lit));
    response.widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, true, on, text));
    response.clicked()
}

fn menu_divider(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(MENU_WIDTH, 11.0), Sense::hover());
    ui.painter().hline(
        rect.shrink2(vec2(8.0, 0.0)).x_range(),
        rect.center().y,
        Stroke::new(1.0_f32, veil(0.08)),
    );
}

fn song_actions(ui: &mut Ui, cx: &mut Cx, item: &Item) {
    ui.set_width(MENU_WIDTH);
    let (head, _) = ui.allocate_exact_size(vec2(MENU_WIDTH, 56.0), Sense::hover());
    let art = Rect::from_min_size(pos2(head.left() + 8.0, head.center().y - 20.0), Vec2::splat(40.0));
    artwork(ui, cx.art, cx.covers.of(item), art, 5.0);
    let (left, width) = (art.right() + 12.0, head.right() - art.right() - 20.0);
    label(
        ui,
        pos2(left, head.center().y - 9.0),
        Align2::LEFT_CENTER,
        &item.title,
        medium(13.5),
        TEXT,
        width,
    );
    label(
        ui,
        pos2(left, head.center().y + 9.0),
        Align2::LEFT_CENTER,
        &item.subtitle,
        sans(12.5),
        MUTED,
        width,
    );
    menu_divider(ui);

    let mut choice = None;
    if menu_row(ui, icon::ARROW_BEND_DOWN_RIGHT, glyph(17.0), MUTED, "Play next").clicked() {
        choice = Some(Action::PlayNext(item.clone()));
    }
    if menu_row(ui, icon::LIST_PLUS, glyph(17.0), MUTED, "Add to queue").clicked() {
        choice = Some(Action::Enqueue(item.clone()));
    }
    let (font, tint, text) = if cx.is_liked(item) {
        (solid(17.0), ACCENT, "Remove from Liked songs")
    } else {
        (glyph(17.0), MUTED, "Add to Liked songs")
    };
    if menu_row(ui, icon::HEART, font, tint, text).clicked() {
        choice = Some(Action::Like(item.clone()));
    }
    if menu_row(ui, icon::DOWNLOAD_SIMPLE, glyph(17.0), MUTED, "Download song (Opus/160k)").clicked() {
        choice = Some(Action::DownloadSong(item.clone()));
    }
    if menu_row(ui, icon::SHARE_NETWORK, glyph(17.0), MUTED, "Share").clicked() {
        choice = Some(Action::Share(item.clone()));
    }
    if !cx.playlists.is_empty() {
        menu_divider(ui);
        for pl in cx.playlists {
            let already_in = pl.songs.iter().any(|s| s.video_id == item.video_id);
            let (p_icon, p_label, p_color) = if already_in {
                (icon::CHECK, format!("In: {}", pl.name), ACCENT)
            } else {
                (icon::PLAYLIST, format!("Add to: {}", pl.name), MUTED)
            };
            if menu_row(ui, p_icon, glyph(17.0), p_color, &p_label).clicked() {
                if already_in {
                    choice = Some(Action::RemoveFromPlaylist(pl.id.clone(), item.video_id.clone()));
                } else {
                    choice = Some(Action::AddToPlaylist(pl.id.clone(), item.clone()));
                }
            }
        }
    }
    if !item.artist_id.is_empty() || !item.album_id.is_empty() {
        menu_divider(ui);
    }
    if !item.artist_id.is_empty() && menu_row(ui, icon::MICROPHONE_STAGE, glyph(17.0), MUTED, "Go to artist").clicked() {
        choice = Some(Action::Go(Route::Browse(item.artist_id.clone())));
    }
    if !item.album_id.is_empty() && menu_row(ui, icon::VINYL_RECORD, glyph(17.0), MUTED, "Go to album").clicked() {
        choice = Some(Action::Go(Route::Browse(item.album_id.clone())));
    }
    if let Some(action) = choice {
        cx.out.push(action);
        ui.close();
    }
}

fn song_menu(response: &Response, cx: &mut Cx, item: &Item) {
    if item.is_song() {
        response.context_menu(|ui| song_actions(ui, cx, item));
    }
}

fn track_row(ui: &mut Ui, cx: &mut Cx, items: &[Item], index: usize, number: Option<usize>) {
    let item = &items[index];
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_HEIGHT), Sense::click());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let more_slot = Rect::from_center_size(pos2(rect.right() - 24.0, rect.center().y), Vec2::splat(30.0));
    let more = item
        .is_song()
        .then(|| ui.interact(more_slot, response.id.with("more"), Sense::click()));
    let more_open = more.as_ref().is_some_and(|more| egui::Popup::menu(more).is_open());
    let engaged = response.hovered() || response.context_menu_opened() || more_open;
    let hover = ui.ctx().animate_bool_with_time(response.id, engaged, HOVER_TIME);
    let playing = cx.is_playing(item);
    let mid = rect.center().y;
    ui.painter().rect_filled(rect, ROW_RADIUS, veil(0.06 * hover));
    if response.has_focus() {
        ui.painter()
            .rect_stroke(rect, ROW_RADIUS, Stroke::new(1.0_f32, MUTED), StrokeKind::Inside);
    }

    let mut left = rect.left() + 12.0;
    match number {
        Some(number) => {
            let slot = pos2(left + 12.0, mid);
            if playing {
                bars(ui, slot, !cx.paused);
            } else {
                ui.painter()
                    .text(slot, Align2::CENTER_CENTER, number.to_string(), sans(13.5), FAINT);
            }
            left += 40.0;
        }
        None => {
            let art = Rect::from_min_size(pos2(left, mid - 20.0), Vec2::splat(40.0));
            artwork(ui, cx.art, cx.covers.of(item), art, if item.is_artist { 20.0 } else { 5.0 });
            if playing {
                ui.painter().rect_filled(art, 5.0, shade(0.6));
                bars(ui, art.center(), !cx.paused);
            }
            left += 54.0;
        }
    }

    let mut right = rect.right() - 14.0;
    if let Some(more) = &more {
        if hover > 0.0 {
            let lift = hover_of(ui, more);
            ui.painter().circle_filled(more_slot.center(), 15.0, veil(0.09 * lift));
            ui.painter().text(
                more_slot.center(),
                Align2::CENTER_CENTER,
                icon::DOTS_THREE,
                glyph(19.0),
                MUTED.gamma_multiply(hover).lerp_to_gamma(TEXT, lift),
            );
        }
        describe(more, "More");
        egui::Popup::menu(more).gap(4.0).show(|ui| song_actions(ui, cx, item));
        right -= 36.0;
        label(ui, pos2(right, mid), Align2::RIGHT_CENTER, &item.duration, sans(13.0), MUTED, 60.0);
        right -= 52.0;
        let liked = cx.is_liked(item);
        if liked || hover > 0.0 {
            let slot = Rect::from_center_size(pos2(right - 14.0, mid), Vec2::splat(30.0));
            let (font, color) = if liked {
                (solid(16.0), ACCENT)
            } else {
                (glyph(16.0), MUTED.gamma_multiply(hover))
            };
            let hint = if liked { "Remove from Liked songs" } else { "Add to Liked songs" };
            if icon_button(ui, slot, response.id.with("like"), icon::HEART, font, color, hint).clicked() {
                cx.out.push(Action::Like(item.clone()));
            }
        }
        right -= 40.0;
    } else {
        ui.painter()
            .text(pos2(right - 6.0, mid), Align2::CENTER_CENTER, icon::CARET_RIGHT, glyph(14.0), FAINT);
        right -= 28.0;
    }
    if rect.width() > 700.0 && !item.extra.is_empty() {
        let column = rect.width() * 0.26;
        label(
            ui,
            pos2(right - column, mid),
            Align2::LEFT_CENTER,
            &item.extra,
            sans(13.0),
            MUTED,
            column - 12.0,
        );
        right -= column + 20.0;
    }

    let title = if playing { ACCENT } else { TEXT };
    label(
        ui,
        pos2(left, mid - 10.0),
        Align2::LEFT_CENTER,
        &item.title,
        medium(14.0),
        title,
        right - left,
    );
    let artist = response.id.with("artist");
    byline(ui, cx.out, item, artist, pos2(left, mid + 10.0), sans(13.0), right - left);

    describe(&response, &item.title);
    if response.clicked() {
        cx.activate(items, index);
    }
    song_menu(&response, cx, item);
}

fn card(ui: &Ui, cx: &mut Cx, items: &[Item], index: usize, cell: Rect, id: Id) {
    let item = &items[index];
    let response = ui.interact(cell, id, Sense::click());
    let hover = ui.ctx().animate_bool_with_time(
        id,
        response.hovered() || response.has_focus() || response.context_menu_opened(),
        0.14,
    );
    let playing = cx.is_playing(item);
    let art = Rect::from_min_size(cell.min, Vec2::splat(cell.width()));
    let radius = if item.is_artist { art.width() / 2.0 } else { ART_RADIUS };
    artwork(ui, cx.art, cx.covers.of(item), art, radius);
    if hover > 0.0 || playing {
        let lit = if playing { 1.0 } else { hover };
        ui.painter().rect_filled(art, radius, shade(0.28 * lit));
        if item.is_song() {
            let badge = pos2(art.right() - 32.0, art.bottom() - 26.0 - 6.0 * lit);
            ui.painter().circle_filled(badge, 22.0, ACCENT.gamma_multiply(lit));
            let symbol = if playing && !cx.paused { icon::PAUSE } else { icon::PLAY };
            ui.painter()
                .text(badge, Align2::CENTER_CENTER, symbol, solid(19.0), ON_ACCENT.gamma_multiply(lit));
        }
    }
    if response.has_focus() {
        ui.painter().rect_stroke(art, radius, Stroke::new(1.5_f32, TEXT), StrokeKind::Inside);
    }
    let title = if playing { ACCENT } else { TEXT };
    label(
        ui,
        pos2(cell.left(), art.bottom() + 22.0),
        Align2::LEFT_CENTER,
        &item.title,
        medium(14.0),
        title,
        cell.width(),
    );
    let below = pos2(cell.left(), art.bottom() + 42.0);
    byline(ui, cx.out, item, id.with("artist"), below, sans(13.0), cell.width());
    describe(&response, &item.title);
    if response.clicked() {
        cx.activate(items, index);
    }
    song_menu(&response, cx, item);
}

fn compact(ui: &Ui, cx: &mut Cx, items: &[Item], index: usize, cell: Rect, id: Id) {
    let item = &items[index];
    let response = ui.interact(cell, id, Sense::click());
    let hover = ui.ctx().animate_bool_with_time(
        id,
        response.hovered() || response.has_focus() || response.context_menu_opened(),
        HOVER_TIME,
    );
    let playing = cx.is_playing(item);
    ui.painter().rect_filled(cell, ROW_RADIUS, veil(0.06 * hover));
    let art = Rect::from_min_size(pos2(cell.left() + 8.0, cell.center().y - 24.0), Vec2::splat(48.0));
    artwork(ui, cx.art, cx.covers.of(item), art, if item.is_artist { 24.0 } else { 6.0 });
    if playing {
        ui.painter().rect_filled(art, 6.0, shade(0.6));
        bars(ui, art.center(), !cx.paused);
    }
    let (left, width) = (art.right() + 14.0, cell.right() - art.right() - 26.0);
    let title = if playing { ACCENT } else { TEXT };
    label(
        ui,
        pos2(left, cell.center().y - 10.0),
        Align2::LEFT_CENTER,
        &item.title,
        medium(14.0),
        title,
        width,
    );
    let below = pos2(left, cell.center().y + 10.0);
    byline(ui, cx.out, item, id.with("artist"), below, sans(13.0), width);
    describe(&response, &item.title);
    if response.clicked() {
        cx.activate(items, index);
    }
    song_menu(&response, cx, item);
}

fn columns(width: f32, minimum: f32, gap: f32) -> (usize, f32) {
    let count = (((width + gap) / (minimum + gap)).floor() as usize).max(1);
    (count, ((width - gap * (count - 1) as f32) / count as f32).floor())
}

fn grid(ui: &mut Ui, count: usize, columns: usize, cell: Vec2, gap: Vec2, mut paint: impl FnMut(&Ui, usize, Rect)) {
    let rows = count.div_ceil(columns);
    let height = rows as f32 * cell.y + rows.saturating_sub(1) as f32 * gap.y;
    let (area, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    for index in 0..count {
        let offset = vec2(
            (index % columns) as f32 * (cell.x + gap.x),
            (index / columns) as f32 * (cell.y + gap.y),
        );
        let rect = Rect::from_min_size(area.min + offset, cell);
        if ui.is_rect_visible(rect) {
            paint(ui, index, rect);
        }
    }
}

fn section_view(ui: &mut Ui, cx: &mut Cx, title: &str, layout: Layout, items: &[Item], id: Id, numbered: bool) {
    let width = ui.available_width();
    let (cards, card_width) = columns(width, CARD_MIN, CARD_GAP);
    let (cells, _cell_width) = columns(width, COMPACT_MIN, 12.0);
    let visible = match layout {
        Layout::Cards => cards,
        Layout::Compact => cells * COMPACT_ROWS,
        Layout::Tracks | Layout::Top => usize::MAX,
    };
    let foldable = items.len() > visible;
    let mut open = ui.data(|data| data.get_temp::<bool>(id)).unwrap_or(false);
    if !title.is_empty() || foldable {
        let (row, _) = ui.allocate_exact_size(vec2(width, 32.0), Sense::hover());
        label(ui, row.left_center(), Align2::LEFT_CENTER, title, bold(20.0), TEXT, width - 120.0);
        if foldable {
            let text = if open { "Show less" } else { "Show all" };
            if text_button(ui, row.right_center(), id.with("fold"), text).clicked() {
                open = !open;
                ui.data_mut(|data| data.insert_temp(id, open));
            }
        }
        ui.add_space(12.0);
    }
    let shown = if open { items.len() } else { items.len().min(visible) };
    match layout {
        Layout::Cards => {
            let cell = vec2(card_width, card_width + CARD_TEXT);
            grid(ui, shown, cards, cell, vec2(CARD_GAP, 14.0), |ui, index, rect| {
                card(ui, cx, items, index, rect, id.with(index))
            });
        }
        Layout::Compact => {
            let pad_x = 10.0;
            let pad_y = 10.0;
            let inner_w = width - pad_x * 2.0;
            let (inner_cells, inner_cell_width) = columns(inner_w, COMPACT_MIN, 12.0);
            let rows = (shown + inner_cells.max(1) - 1) / inner_cells.max(1);
            let total_h = rows as f32 * 64.0 + (rows.saturating_sub(1)) as f32 * 2.0 + pad_y * 2.0;
            let (box_rect, _) = ui.allocate_exact_size(vec2(width, total_h), Sense::hover());
            ui.painter().rect_filled(box_rect, 14.0, veil(0.04));
            ui.painter().rect_stroke(box_rect, 14.0, Stroke::new(1.0_f32, veil(0.08)), StrokeKind::Inside);
            let cell = vec2(inner_cell_width, 64.0);
            let start = box_rect.min + vec2(pad_x, pad_y);
            for index in 0..shown {
                let col = index % inner_cells.max(1);
                let row = index / inner_cells.max(1);
                let x = start.x + col as f32 * (inner_cell_width + 12.0);
                let y = start.y + row as f32 * (64.0 + 2.0);
                let rect = Rect::from_min_size(pos2(x, y), cell);
                compact(ui, cx, items, index, rect, id.with(index));
            }
        }
        Layout::Tracks => {
            for index in 0..shown {
                track_row(ui, cx, items, index, numbered.then_some(index + 1));
            }
        }
        Layout::Top => {
            top_result(ui, cx, items, id);
            for index in 1..shown {
                track_row(ui, cx, items, index, None);
            }
        }
    }
}

fn top_result(ui: &mut Ui, cx: &mut Cx, items: &[Item], id: Id) {
    let item = &items[0];
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 128.0), Sense::click());
    let hover = hover_of(ui, &response);
    ui.painter().rect_filled(rect, 12.0, veil(0.045 + 0.035 * hover));
    let art = Rect::from_min_size(rect.min + vec2(16.0, 16.0), Vec2::splat(96.0));
    artwork(ui, cx.art, cx.covers.of(item), art, if item.is_artist { 48.0 } else { ART_RADIUS });
    let left = art.right() + 22.0;
    let width = rect.right() - left - 150.0;
    label(
        ui,
        pos2(left, rect.center().y - 16.0),
        Align2::LEFT_CENTER,
        &item.title,
        display(38.0),
        TEXT,
        width,
    );
    let below = pos2(left, rect.center().y + 20.0);
    byline(ui, cx.out, item, id.with("artist"), below, sans(14.0), width);
    let (symbol, text) = if item.is_song() {
        (icon::PLAY, "Play")
    } else {
        (icon::CARET_RIGHT, "Open")
    };
    let button = pill(
        ui,
        pos2(rect.right() - 128.0, rect.center().y - 21.0),
        id.with("top"),
        symbol,
        text,
        item.is_song(),
    );
    describe(&response, &item.title);
    if response.clicked() || button.clicked() {
        cx.activate(items, 0);
    }
    song_menu(&response, cx, item);
    ui.add_space(8.0);
}

fn hero(ui: &mut Ui, cx: &mut Cx, item: &Item, symbol: Option<&str>, playable: bool, pinned: Option<bool>) -> bool {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), HERO_ART), Sense::hover());
    let art = Rect::from_min_size(rect.min, Vec2::splat(HERO_ART));
    match symbol {
        Some(symbol) => {
            ui.painter().rect_filled(art, 12.0, veil(0.06));
            ui.painter().text(art.center(), Align2::CENTER_CENTER, symbol, solid(84.0), ACCENT);
        }
        None => artwork(ui, cx.art, &item.thumb, art, if item.is_artist { HERO_ART / 2.0 } else { 12.0 }),
    }
    let left = art.right() + 30.0;
    let width = (rect.right() - left).max(120.0);
    let mut bottom = rect.bottom();

    let mut at = pos2(left, bottom - 42.0);
    let mut play_clicked = false;
    if playable {
        let play = pill(ui, at, ui.id().with("hero-play"), icon::PLAY, "Play", true);
        at.x = play.rect.right() + 10.0;
        play_clicked = play.clicked();
    }
    if let Some(pinned) = pinned {
        let slot = Rect::from_min_size(at, Vec2::splat(42.0));
        let (font, color, hint) = if pinned {
            (solid(19.0), ACCENT, "Remove from library")
        } else {
            (glyph(19.0), MUTED, "Save to library")
        };
        if icon_button(ui, slot, ui.id().with("hero-pin"), icon::BOOKMARK_SIMPLE, font, color, hint).clicked() {
            cx.out.push(Action::Pin(item.clone()));
        }
        at.x = slot.right() + 10.0;
    }
    let share = Rect::from_min_size(at, Vec2::splat(42.0));
    if symbol.is_none()
        && icon_button(
            ui,
            share,
            ui.id().with("hero-share"),
            icon::SHARE_NETWORK,
            glyph(19.0),
            MUTED,
            "Copy link",
        )
        .clicked()
    {
        cx.out.push(Action::Share(item.clone()));
    }
    bottom -= 60.0;

    for (text, font, color) in [(&item.subtitle, sans(14.0), MUTED), (&item.extra, medium(15.0), TEXT)] {
        if !text.is_empty() {
            let line = label(ui, pos2(left, bottom), Align2::LEFT_BOTTOM, text, font, color, width);
            bottom = line.top() - 5.0;
        }
    }
    let size = match item.title.chars().count() {
        0..=16 => 68.0,
        17..=34 => 50.0,
        _ => 38.0,
    };
    let title = fit(ui, &item.title, display(size), TEXT, width, 2);
    ui.painter().galley(pos2(left, bottom - title.size().y), title, TEXT);
    play_clicked
}

fn skeleton(ui: &mut Ui) {
    let pulse = veil(0.05 + 0.025 * (ui.input(|input| input.time) * 3.2).sin() as f32);
    ui.ctx().request_repaint_after(Duration::from_millis(60));
    let (count, side) = columns(ui.available_width(), CARD_MIN, CARD_GAP);
    for _ in 0..2 {
        let (heading, _) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::hover());
        ui.painter()
            .rect_filled(Rect::from_min_size(heading.min + vec2(0.0, 5.0), vec2(190.0, 22.0)), 6.0, pulse);
        ui.add_space(12.0);
        grid(
            ui,
            count,
            count,
            vec2(side, side + CARD_TEXT),
            vec2(CARD_GAP, 0.0),
            |ui, _, cell| {
                ui.painter()
                    .rect_filled(Rect::from_min_size(cell.min, Vec2::splat(side)), ART_RADIUS, pulse);
                ui.painter().rect_filled(
                    Rect::from_min_size(cell.min + vec2(0.0, side + 14.0), vec2(side * 0.7, 13.0)),
                    4.0,
                    pulse,
                );
                ui.painter().rect_filled(
                    Rect::from_min_size(cell.min + vec2(0.0, side + 35.0), vec2(side * 0.45, 12.0)),
                    4.0,
                    pulse,
                );
            },
        );
        ui.add_space(32.0);
    }
}

fn message(ui: &mut Ui, symbol: &str, title: &str, hint: &str) -> Rect {
    ui.add_space(72.0);
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 132.0), Sense::hover());
    let center = rect.center_top();
    ui.painter().circle_filled(center + vec2(0.0, 34.0), 34.0, veil(0.06));
    ui.painter()
        .text(center + vec2(0.0, 34.0), Align2::CENTER_CENTER, symbol, glyph(30.0), MUTED);
    label(
        ui,
        center + vec2(0.0, 92.0),
        Align2::CENTER_CENTER,
        title,
        bold(18.0),
        TEXT,
        rect.width(),
    );
    label(
        ui,
        center + vec2(0.0, 118.0),
        Align2::CENTER_CENTER,
        hint,
        sans(14.0),
        MUTED,
        rect.width(),
    );
    rect
}

fn local(ui: &mut Ui, cx: &mut Cx, title: &str, symbol: &str, items: &[Item], empty: [&str; 2], clearable: bool) {
    let count = if items.len() == 1 {
        "1 song".to_owned()
    } else {
        format!("{} songs", items.len())
    };
    let header = Item {
        title: title.to_owned(),
        subtitle: count,
        ..Item::default()
    };
    let play_row_end = pos2(ui.max_rect().right(), ui.cursor().top() + HERO_ART - 21.0);
    if hero(ui, cx, &header, Some(symbol), !items.is_empty(), None) {
        cx.out.push(Action::Play(items.to_vec(), 0));
    }
    if clearable && !items.is_empty() && text_button(ui, play_row_end, ui.id().with("clear"), "Clear history").clicked() {
        cx.out.push(Action::ClearHistory);
    }
    ui.add_space(28.0);
    if items.is_empty() {
        message(ui, symbol, empty[0], empty[1]);
    }
    for index in 0..items.len() {
        track_row(ui, cx, items, index, None);
    }
}

fn custom_playlist_page(ui: &mut Ui, cx: &mut Cx, pl: &CustomPlaylist) {
    let count = if pl.songs.len() == 1 {
        "1 song".to_owned()
    } else {
        format!("{} songs", pl.songs.len())
    };
    let header = Item {
        title: pl.name.clone(),
        subtitle: count,
        ..Item::default()
    };
    let play_row_end = pos2(ui.max_rect().right(), ui.cursor().top() + HERO_ART - 21.0);
    if hero(ui, cx, &header, Some(icon::PLAYLIST), !pl.songs.is_empty(), None) {
        cx.out.push(Action::Play(pl.songs.clone(), 0));
    }
    if text_button(ui, play_row_end, ui.id().with("delete_pl"), "Delete playlist").clicked() {
        cx.out.push(Action::DeletePlaylist(pl.id.clone()));
    }
    ui.add_space(28.0);
    if pl.songs.is_empty() {
        message(
            ui,
            icon::PLAYLIST,
            "Playlist is empty",
            "Right-click any track and choose 'Add to: <Playlist>' to build your mix.",
        );
    }
    for index in 0..pl.songs.len() {
        track_row(ui, cx, &pl.songs, index, None);
    }
}

fn page(ui: &mut Ui, cx: &mut Cx, load: Option<&Load>, pinned: &[Item], query: Option<&str>) {
    match load {
        Some(Load::Ready(page)) => {
            if let Some(header) = &page.header {
                let saved = pinned.iter().any(|item| item.browse_id == header.browse_id);
                let playable = page.sections.iter().flat_map(|section| &section.items).any(Item::is_song);
                if hero(ui, cx, header, None, playable, Some(saved)) {
                    cx.out.push(Action::Play(page.songs(), 0));
                }
                ui.add_space(8.0);
            }
            if let (true, Some(query)) = (page.sections.is_empty(), query) {
                message(
                    ui,
                    icon::MAGNIFYING_GLASS,
                    &format!("Nothing found for \"{query}\""),
                    "Check the spelling or try fewer words.",
                );
            }
            for (index, section) in page.sections.iter().enumerate() {
                ui.add_space(if index == 0 && page.header.is_none() { 4.0 } else { 30.0 });
                let numbered = section.layout == Layout::Tracks
                    && page
                        .header
                        .as_ref()
                        .is_some_and(|header| !header.is_artist && section.items.iter().all(|item| item.thumb == header.thumb));
                let id = ui.id().with(index);
                section_view(ui, cx, &section.title, section.layout, &section.items, id, numbered);
            }
            if page.partial {
                ui.add_space(30.0);
                skeleton(ui);
            }
        }
        Some(Load::Failed(error)) => {
            let rect = message(ui, icon::WARNING_CIRCLE, "This page did not load", error);
            ui.add_space(12.0);
            let (row, _) = ui.allocate_exact_size(vec2(rect.width(), 42.0), Sense::hover());
            if pill(
                ui,
                pos2(row.center().x - 70.0, row.top()),
                ui.id().with("retry"),
                icon::ARROW_CLOCKWISE,
                "Try again",
                false,
            )
            .clicked()
            {
                cx.out.push(Action::Reload);
            }
        }
        _ => skeleton(ui),
    }
}

fn mood(ui: &Ui, name: &str, tint: Option<Color32>, floor: Color32, brightness: f32) -> Color32 {
    let target = tint.map_or(floor, |tint| {
        let mut color = Hsva::from(tint);
        color.s = (color.s * 1.5).min(0.8);
        color.v = brightness;
        floor.lerp_to_gamma(Color32::from(color), 0.8)
    });
    let channel = |index: usize, value: u8| ui.ctx().animate_value_with_time(Id::new((name, index)), value as f32, 0.6) as u8;
    Color32::from_rgb(channel(0, target.r()), channel(1, target.g()), channel(2, target.b()))
}

fn luminance(color: Color32) -> f32 {
    let linear = egui::Rgba::from(color);
    0.2126 * linear.r() + 0.7152 * linear.g() + 0.0722 * linear.b()
}

fn dimmed(color: Color32, limit: f32) -> Color32 {
    let scale = limit / luminance(color);
    if scale >= 1.0 {
        return color;
    }
    let linear = egui::Rgba::from(color);
    Color32::from(egui::Rgba::from_rgb(linear.r() * scale, linear.g() * scale, linear.b() * scale))
}

fn wash(painter: &egui::Painter, rect: Rect, top: Color32, bottom: Color32) {
    if top == bottom {
        return;
    }
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), top);
    mesh.colored_vertex(rect.right_top(), top);
    mesh.colored_vertex(rect.left_bottom(), bottom);
    mesh.colored_vertex(rect.right_bottom(), bottom);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(1, 2, 3);
    painter.add(Shape::mesh(mesh));
}

fn entrance(ui: &Ui, scene: Id) -> f32 {
    let now = ui.input(|input| input.time);
    let started = ui.data_mut(|data| {
        let shown = data.get_temp_mut_or_insert_with(Id::new("entrance"), || (scene, f64::NEG_INFINITY));
        if shown.0 != scene {
            *shown = (scene, now);
        }
        shown.1
    });
    let progress = ((now - started) / PAGE_FADE).min(1.0) as f32;
    if progress < 1.0 {
        ui.ctx().request_repaint();
    }
    progress * (2.0 - progress)
}

fn sticky(ui: &Ui, strip: Rect, title: &str, playable: bool) -> bool {
    ui.interact(strip, Id::new("sticky"), Sense::click());
    let mid = strip.bottom() - STICKY_HEIGHT / 2.0;
    let mut left = strip.left() + PAGE_MARGIN as f32;
    let mut clicked = false;
    if playable {
        let slot = Rect::from_center_size(pos2(left + 20.0, mid), Vec2::splat(40.0));
        let play = ui.interact(slot, Id::new("sticky-play"), Sense::click());
        let hover = hover_of(ui, &play);
        let pressed = if play.is_pointer_button_down_on() { 1.0 } else { 0.0 };
        ui.painter().circle_filled(
            slot.center(),
            19.0 + hover - pressed,
            ACCENT.lerp_to_gamma(Color32::WHITE, 0.12 * hover),
        );
        ui.painter()
            .text(slot.center(), Align2::CENTER_CENTER, icon::PLAY, solid(17.0), ON_ACCENT);
        describe(&play, "Play");
        clicked = play.clicked();
        left = slot.right() + 16.0;
    }
    let width = strip.right() - left - PAGE_MARGIN as f32;
    label(ui, pos2(left, mid), Align2::LEFT_CENTER, title, bold(18.0), TEXT, width);
    ui.painter()
        .hline(strip.x_range(), strip.bottom() - 0.5, Stroke::new(1.0_f32, veil(0.07)));
    clicked
}

fn topbar(app: &mut App, ui: &mut Ui, out: &mut Vec<Action>) {
    let (bar, _) = ui.allocate_exact_size(vec2(ui.available_width(), TOPBAR_HEIGHT), Sense::hover());
    let mid = bar.center().y;
    let mut left = bar.left() + 24.0;
    let steps = [
        (icon::CARET_LEFT, app.at > 0, Action::Back, "Back"),
        (icon::CARET_RIGHT, app.at + 1 < app.history.len(), Action::Forward, "Forward"),
    ];
    for (symbol, enabled, action, hint) in steps {
        let slot = Rect::from_center_size(pos2(left + 18.0, mid), Vec2::splat(36.0));
        if !enabled {
            ui.painter()
                .text(slot.center(), Align2::CENTER_CENTER, symbol, glyph(17.0), FAINT.gamma_multiply(0.5));
        } else if icon_button(ui, slot, Id::new(hint), symbol, glyph(17.0), MUTED, hint).clicked() {
            out.push(action);
        }
        left += 40.0;
    }
    left += 10.0;

    let field = Rect::from_min_size(
        pos2(left, mid - 21.0),
        vec2((bar.right() - PAGE_MARGIN as f32 - left).min(440.0), 42.0),
    );
    let background = ui.painter().add(Shape::Noop);
    ui.painter().text(
        pos2(field.left() + 22.0, mid),
        Align2::CENTER_CENTER,
        icon::MAGNIFYING_GLASS,
        glyph(17.0),
        MUTED,
    );
    let input = Rect::from_min_max(pos2(field.left() + 44.0, field.top()), pos2(field.right() - 64.0, field.bottom()));
    let layout = egui::Layout::left_to_right(Align::Center);
    let response = ui
        .scope_builder(UiBuilder::new().max_rect(input).layout(layout), |ui| {
            let edit = egui::TextEdit::singleline(&mut app.query)
                .id(Id::new("search"))
                .frame(false)
                .margin(Margin::ZERO)
                .desired_width(input.width())
                .font(sans(14.5))
                .text_color(TEXT)
                .hint_text(RichText::new("Search songs, albums, artists").color(FAINT));
            ui.add(edit)
        })
        .inner;
    if std::mem::take(&mut app.focus_search) {
        response.request_focus();
    }
    if response.changed() {
        app.typed = Some(Instant::now());
    }
    let query = app.query.trim();
    if response.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter)) && !query.is_empty() {
        app.typed = None;
        out.push(Action::Go(Route::Search(query.to_owned())));
    }
    let focus = ui.ctx().animate_bool_with_time(Id::new("search-focus"), response.has_focus(), 0.12);
    let stroke_color = veil(0.08).lerp_to_gamma(ACCENT, focus);
    let stroke = Stroke::new(1.0 + 0.5 * focus, stroke_color);
    ui.painter().set(
        background,
        RectShape::new(field, 21.0, veil(0.07 + 0.02 * focus), stroke, StrokeKind::Inside),
    );
    let trailing = pos2(field.right() - 20.0, mid);
    if app.query.is_empty() {
        ui.painter()
            .text(pos2(field.right() - 16.0, mid), Align2::RIGHT_CENTER, "Ctrl K", sans(12.0), FAINT);
    } else if icon_button(
        ui,
        Rect::from_center_size(trailing, Vec2::splat(26.0)),
        Id::new("search-clear"),
        icon::X,
        glyph(13.0),
        MUTED,
        "Clear",
    )
    .clicked()
    {
        app.query.clear();
        app.typed = None;
        response.request_focus();
    }

    let right_edge = bar.right() - PAGE_MARGIN as f32;
    let discord_w = 34.0;
    let perf_w = 116.0;

    // Discord Presence quick icon toggle (to the right of perf button)
    let discord_rect = Rect::from_min_size(
        pos2(right_edge - discord_w, mid - 16.0),
        vec2(discord_w, 32.0),
    );
    let discord_resp = ui.interact(discord_rect, Id::new("topbar-discord-toggle"), Sense::click());
    let discord_hover = hover_of(ui, &discord_resp);
    let discord_on = app.saved.discord;
    let (d_bg, d_stroke, d_color) = if discord_on {
        (
            Color32::from_rgb(88, 101, 242),
            Color32::from_rgb(121, 133, 246),
            Color32::WHITE,
        )
    } else {
        (
            veil(0.06 + 0.05 * discord_hover),
            veil(0.18 + 0.12 * discord_hover),
            TEXT,
        )
    };
    ui.painter().rect_filled(discord_rect, 16.0, d_bg);
    ui.painter().rect_stroke(discord_rect, 16.0, Stroke::new(1.2_f32, d_stroke), StrokeKind::Inside);
    ui.painter().text(discord_rect.center(), Align2::CENTER_CENTER, icon::DISCORD_LOGO, solid(18.0), d_color);
    describe(&discord_resp, if discord_on { "Discord Presence Active (Click to disable)" } else { "Discord Presence Disabled (Click to enable)" });
    if discord_resp.clicked() {
        out.push(Action::ShowOnDiscord(!discord_on));
    }

    // Extreme Performance Mode quick toggle
    let perf_rect = Rect::from_min_size(
        pos2(right_edge - discord_w - 8.0 - perf_w, mid - 16.0),
        vec2(perf_w, 32.0),
    );
    let perf_resp = ui.interact(perf_rect, Id::new("topbar-perf-mode"), Sense::click());
    let perf_hover = hover_of(ui, &perf_resp);
    let perf_on = app.saved.performance_mode;
    let (bg_color, stroke_color, text_color, label_text) = if perf_on {
        (
            Color32::from_rgba_unmultiplied(139, 92, 246, 36),
            ACCENT,
            ACCENT,
            "⚡ Perf ON",
        )
    } else {
        (
            veil(0.04 + 0.03 * perf_hover),
            veil(0.08 + 0.06 * perf_hover),
            MUTED,
            "⚡ Perf OFF",
        )
    };
    ui.painter().rect_filled(perf_rect, 16.0, bg_color);
    ui.painter().rect_stroke(perf_rect, 16.0, Stroke::new(1.0_f32, stroke_color), StrokeKind::Inside);
    ui.painter().text(perf_rect.center(), Align2::CENTER_CENTER, label_text, medium(12.0), text_color);
    if perf_resp.clicked() {
        out.push(Action::TogglePerformanceMode);
    }

    ui.allocate_rect(bar, Sense::hover());
}

fn content(app: &mut App, ui: &mut Ui, out: &mut Vec<Action>) {
    let area = ui.max_rect();
    let base_color = app.saved.theme_base.color();
    let route = app.route().clone();
    let header = match app.pages.get(&route) {
        Some(Load::Ready(page)) => page.header.as_ref().map(|header| (header.thumb.as_str(), HERO_ART)),
        _ => None,
    };
    let playing = app.now_playing().map(|item| (app.saved.covers.of(item), BAR_ART));
    let picture = |source: Option<(&str, f32)>| source.and_then(|(thumb, size)| app.art.get(ui.ctx(), thumb, size));
    let tint = picture(header).or_else(|| picture(playing)).map(|picture| picture.tint);
    let glow = mood(ui, "ambient", tint, base_color, 0.36);
    let haze = Rect::from_min_size(area.min, vec2(area.width(), AMBIENT_HEIGHT.min(area.height())));
    let disable_blurs = app.saved.performance_mode && app.saved.perf_disable_blurs;
    if !disable_blurs && app.saved.ambient_glow {
        wash(ui.painter(), haze, glow, base_color);
    }
    topbar(app, ui, out);

    let loading = matches!(app.pages.get(&route), Some(Load::Loading));
    let shown = entrance(ui, Id::new((&route, loading)));
    let headline = match (&route, app.pages.get(&route)) {
        (Route::Liked, _) => Some(("Liked songs", !app.saved.liked.is_empty())),
        (Route::Recent, _) => Some(("Recently played", !app.saved.recent.is_empty())),
        (Route::CustomPlaylist(id), _) => {
            app.saved.playlists.iter().find(|p| &p.id == id).map(|p| (p.name.as_str(), !p.songs.is_empty()))
        }
        (Route::Browse(_), Some(Load::Ready(page))) => page.header.as_ref().map(|header| {
            let playable = page.sections.iter().flat_map(|section| &section.items).any(Item::is_song);
            (header.title.as_str(), playable)
        }),
        _ => None,
    };
    let mut cx = Cx {
        art: &app.art,
        covers: &app.saved.covers,
        playing: app.now_playing().map_or("", |item| item.video_id.as_str()),
        paused: app.player.paused(),
        liked: &app.saved.liked,
        single: matches!(route, Route::Search(_) | Route::Home),
        playlists: &app.saved.playlists,
        out: &mut *out,
    };
    let margin = Margin {
        left: PAGE_MARGIN,
        right: PAGE_MARGIN,
        top: 8,
        bottom: 56,
    };
    let scrolled = egui::ScrollArea::vertical().id_salt(&route).auto_shrink(false).show(ui, |ui| {
        Frame::new().inner_margin(margin).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_opacity(shown);
            match &route {
                Route::Liked => {
                    let empty = ["No liked songs yet", "Tap the heart on any track to keep it here."];
                    local(ui, &mut cx, "Liked songs", icon::HEART, &app.saved.liked, empty, false);
                }
                Route::Recent => {
                    let empty = ["Nothing played yet", "Tracks you listen to are remembered here."];
                    local(
                        ui,
                        &mut cx,
                        "Recently played",
                        icon::CLOCK_COUNTER_CLOCKWISE,
                        &app.saved.recent,
                        empty,
                        true,
                    );
                }
                Route::CustomPlaylist(id) => {
                    if let Some(pl) = app.saved.playlists.iter().find(|p| &p.id == id) {
                        custom_playlist_page(ui, &mut cx, pl);
                    }
                }
                Route::Settings(tab) => settings(ui, app, *tab, cx.out),
                Route::Search(query) => page(ui, &mut cx, app.pages.get(&route), &app.saved.pinned, Some(query)),
                Route::Home | Route::Browse(_) => page(ui, &mut cx, app.pages.get(&route), &app.saved.pinned, None),
            }
        });
    });

    let past_hero = scrolled.state.offset.y > HERO_ART - 36.0;
    let stuck = ui
        .ctx()
        .animate_bool_with_time(Id::new("sticky"), headline.is_some() && past_hero, 0.16);
    let Some((title, playable)) = headline.filter(|_| stuck > 0.0) else {
        return;
    };
    let overlap = ui.visuals().clip_rect_margin;
    let strip = Rect::from_min_size(
        pos2(area.left(), area.top() + TOPBAR_HEIGHT - overlap),
        vec2(area.width(), STICKY_HEIGHT + overlap),
    );
    let mut pane = ui.new_child(UiBuilder::new().id_salt("sticky").max_rect(strip));
    pane.set_clip_rect(strip);
    pane.set_opacity(stuck);
    pane.painter().rect_filled(strip, 0.0, base_color);
    if !disable_blurs && app.saved.ambient_glow {
        wash(pane.painter(), haze, glow, base_color);
    }
    if sticky(&pane, strip, title, playable) {
        let songs = match (&route, app.pages.get(&route)) {
            (Route::Liked, _) => app.saved.liked.clone(),
            (Route::Recent, _) => app.saved.recent.clone(),
            (_, Some(Load::Ready(page))) => page.songs(),
            _ => Vec::new(),
        };
        out.push(Action::Play(songs, 0));
    }
}

fn section(ui: &mut Ui, title: &str, height: f32, body: impl FnOnce(&mut Ui)) -> Rect {
    let full = ui.available_width();
    let width = full.min(SETTINGS_WIDTH);
    eyebrow(ui, title, (full - width) / 2.0 + 4.0);
    let (slot, _) = ui.allocate_exact_size(vec2(full, height), Sense::hover());
    let card = Rect::from_center_size(slot.center(), vec2(width, height));
    ui.painter().rect_filled(card, CARD_RADIUS, veil(0.05));
    let mut inner = ui.new_child(UiBuilder::new().id_salt(title).max_rect(card.shrink(CARD_PAD)));
    inner.set_clip_rect(card.intersect(ui.clip_rect()));
    body(&mut inner);
    ui.add_space(SECTION_GAP);
    card
}

fn keycaps(ui: &Ui, right_center: Pos2, parts: &[String], tint: Color32) -> f32 {
    let mut right = right_center.x;
    for part in parts.iter().rev() {
        let galley = ui.painter().layout_no_wrap(part.clone(), bold(12.0), tint);
        let size = vec2((galley.size().x + 18.0).max(30.0), 28.0);
        let cap = Rect::from_min_size(pos2(right - size.x, right_center.y - size.y / 2.0), size);
        let (fill, edge) = if tint == ACCENT {
            (ACCENT.gamma_multiply(0.14), ACCENT.gamma_multiply(0.6))
        } else {
            (veil(0.09), veil(0.13))
        };
        ui.painter().rect(cap, 7.0, fill, Stroke::new(1.0_f32, edge), StrokeKind::Inside);
        ui.painter().galley(cap.center() - galley.size() / 2.0, galley, tint);
        right = cap.left() - 6.0;
    }
    right + 6.0
}

fn hotkey_row(ui: &mut Ui, app: &App, command: Command, enabled: bool, out: &mut Vec<Action>) {
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), HOTKEY_ROW), sense);
    let recording = app.recording == Some(command);
    let taken = app.hotkey_taken.contains(&command);
    let hover = if enabled { hover_of(ui, &response) } else { 0.0 };
    let lit = ui
        .ctx()
        .animate_bool_with_time(response.id.with("recording"), recording, HOVER_TIME);
    ui.painter().rect_filled(rect, ROW_RADIUS, veil(0.06 * hover.max(lit)));
    let middle = rect.center().y;
    let symbol = match command {
        Command::PlayPause => icon::PLAY_PAUSE,
        Command::Next => icon::SKIP_FORWARD,
        Command::Previous => icon::SKIP_BACK,
        Command::VolumeUp => icon::SPEAKER_HIGH,
        Command::VolumeDown => icon::SPEAKER_LOW,
    };
    let tone = MUTED.lerp_to_gamma(TEXT, hover);
    ui.painter()
        .text(pos2(rect.left() + 22.0, middle), Align2::CENTER_CENTER, symbol, glyph(17.0), tone);
    let edge = pos2(rect.right() - 14.0, middle);
    let caps_left = match (recording, app.saved.hotkey_combos[command as usize]) {
        (true, _) => keycaps(ui, edge, &["Press keys\u{2026}".to_owned()], ACCENT),
        (false, Some(combo)) => keycaps(ui, edge, &combo.parts(), TEXT),
        (false, None) => label(ui, edge, Align2::RIGHT_CENTER, "Not set", sans(13.0), FAINT, 120.0).left(),
    };
    let note = match (recording, taken) {
        (true, _) => "Hold Ctrl, Alt or Shift with a key. Esc cancels, Backspace clears.",
        (false, true) => "In use by another app",
        (false, false) => "",
    };
    let shown = ui
        .ctx()
        .animate_bool_with_time(response.id.with("note"), !note.is_empty(), HOVER_TIME);
    let (left, width) = (rect.left() + 46.0, caps_left - rect.left() - 58.0);
    let name = pos2(left, middle - 9.0 * shown);
    label(ui, name, Align2::LEFT_CENTER, command.name(), medium(13.5), TEXT, width);
    if shown > 0.0 {
        let color = if taken && !recording { ACCENT } else { MUTED };
        let at = pos2(left, middle + 11.0);
        label(ui, at, Align2::LEFT_CENTER, note, sans(12.0), color.gamma_multiply(shown), width);
    }
    if enabled {
        describe(&response, command.name());
        if response.clicked() {
            out.push(Action::Record((!recording).then_some(command)));
        }
    }
}

fn discord_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    let open = ui.ctx().animate_bool_with_time(Id::new("discord-open"), app.saved.discord, 0.16);
    let extra = STATUS_ROW + 4.0 * SETTING_ROW;
    let height = 2.0 * CARD_PAD + SETTING_ROW + extra * open;
    section(ui, "Discord", height, |ui| {
        let size = vec2(ui.available_width(), SETTING_ROW);
        if switch_row(ui, size, icon::DISCORD_LOGO, "Show on Discord", app.saved.discord) {
            out.push(Action::ShowOnDiscord(!app.saved.discord));
        }
        if open == 0.0 {
            return;
        }
        ui.multiply_opacity(open);
        let status = match app.discord_linked {
            Some(true) if app.playing() => "Connected to Discord",
            Some(true) => "Shows while a song is playing",
            Some(false) => "Discord is not running",
            None => "",
        };
        let (line, response) = ui.allocate_exact_size(vec2(size.x, STATUS_ROW), Sense::hover());
        let at = pos2(line.left() + 46.0, line.center().y);
        label(ui, at, Align2::LEFT_CENTER, status, sans(12.0), MUTED, size.x - 58.0);
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, status));
        for (headline, text) in [
            (Headline::App, "Listening to AURI"),
            (Headline::Artist, "Listening to the artist"),
            (Headline::Song, "Listening to the song"),
        ] {
            let chosen = app.saved.discord_headline == headline;
            let symbol = if chosen { icon::CHECK } else { "" };
            let row = wide_row(ui, size, symbol, glyph(15.0), ACCENT, text);
            row.widget_info(|| WidgetInfo::selected(WidgetType::RadioButton, true, chosen, text));
            if row.clicked() {
                out.push(Action::DiscordHeadline(headline));
            }
        }
        let (row, _) = ui.allocate_exact_size(size, Sense::hover());
        if app.saved.discord_headline == Headline::Custom {
            let at = pos2(row.left() + 22.0, row.center().y);
            ui.painter().text(at, Align2::CENTER_CENTER, icon::CHECK, glyph(15.0), ACCENT);
        }
        let field = Rect::from_min_max(
            pos2(row.left() + 38.0, row.top() + 5.0),
            pos2(row.right() - 8.0, row.bottom() - 5.0),
        );
        ui.painter().rect_filled(field, 7.0, veil(0.07));
        let mut name = app.saved.discord_name.clone();
        let edit = egui::TextEdit::singleline(&mut name)
            .id(Id::new("discord-name"))
            .frame(false)
            .margin(Margin::ZERO)
            .font(medium(13.5))
            .text_color(TEXT)
            .vertical_align(Align::Center)
            .hint_text(RichText::new("Your own text").color(FAINT));
        let typed = ui.put(field.shrink2(vec2(10.0, 0.0)), edit);
        typed.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, "Your own text"));
        if typed.changed() || typed.gained_focus() {
            out.push(Action::DiscordName(name));
        }
    });
}

fn hotkey_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    let rows = if hotkeys::SUPPORTED {
        Command::ALL.len() as f32 * HOTKEY_ROW
    } else {
        0.0
    };
    let height = 2.0 * CARD_PAD + SETTING_ROW + rows;
    let card = section(ui, "Global hotkeys", height, |ui| {
        let size = vec2(ui.available_width(), SETTING_ROW);
        if !hotkeys::SUPPORTED {
            let (row, _) = ui.allocate_exact_size(size, Sense::hover());
            let at = pos2(row.left() + 22.0, row.center().y);
            ui.painter().text(at, Align2::CENTER_CENTER, icon::KEYBOARD, glyph(17.0), MUTED);
            let text = "Global hotkeys are only available on Windows for now";
            let at = pos2(row.left() + 46.0, row.center().y);
            label(ui, at, Align2::LEFT_CENTER, text, medium(13.5), MUTED, size.x - 58.0);
            return;
        }
        if switch_row(ui, size, icon::KEYBOARD, "Enable global hotkeys", app.saved.hotkeys) {
            out.push(Action::GlobalHotkeys(!app.saved.hotkeys));
        }
        let on = ui.ctx().animate_bool_with_time(Id::new("hotkeys-on"), app.saved.hotkeys, 0.16);
        ui.multiply_opacity(egui::lerp(0.4..=1.0, on));
        for command in Command::ALL {
            hotkey_row(ui, app, command, app.saved.hotkeys, out);
        }
    });
    if !hotkeys::SUPPORTED {
        return;
    }
    let reset_at = pos2(card.right() - 10.0, card.top() - 15.0);
    if app.saved.hotkey_combos != hotkeys::standard_combos()
        && text_button(ui, reset_at, Id::new("hotkeys-reset"), "Reset to defaults").clicked()
    {
        out.push(Action::ResetHotkeys);
    }
    let text = "Hotkeys keep working while AURI is minimized. Each one needs Ctrl, Alt or Shift.";
    let at = pos2(card.left() + 4.0, card.bottom() + SECTION_GAP / 2.0);
    label(ui, at, Align2::LEFT_CENTER, text, sans(12.0), FAINT, card.width());
}

fn audio_dsp_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    let eq_height = if app.saved.eq_enabled { 260.0 } else { 0.0 };
    let height = 2.0 * CARD_PAD + 3.0 * SETTING_ROW + 44.0 + eq_height;
    section(ui, "Audio & DSP Engine", height, |ui| {
        let size = vec2(ui.available_width(), SETTING_ROW);
        if switch_row(ui, size, icon::WAVEFORM, "Studio Loudness Normalizer (-14.0 LUFS)", app.saved.loudness_normalizer) {
            out.push(Action::ToggleLoudness);
        }
        let desc = "Normalizes tracks based on YouTube Music broadcast loudness metadata for uniform listening.";
        let (row, _) = ui.allocate_exact_size(vec2(size.x, 22.0), Sense::hover());
        label(ui, pos2(row.left() + 46.0, row.center().y), Align2::LEFT_CENTER, desc, sans(12.0), MUTED, size.x - 58.0);

        let (limiter_row, _) = ui.allocate_exact_size(size, Sense::hover());
        let middle = limiter_row.center().y;
        ui.painter().text(pos2(limiter_row.left() + 22.0, middle), Align2::CENTER_CENTER, icon::FADERS, glyph(17.0), ACCENT);
        label(ui, pos2(limiter_row.left() + 46.0, middle - 8.0), Align2::LEFT_CENTER, "Soft-Knee Peak Limiter (Active)", medium(13.5), TEXT, size.x - 58.0);
        let sub = "Prevents clipping and harsh distortion on loud transients with studio-grade brickwall limiting.";
        label(ui, pos2(limiter_row.left() + 46.0, middle + 10.0), Align2::LEFT_CENTER, sub, sans(12.0), MUTED, size.x - 58.0);

        if switch_row(ui, size, icon::EQUALIZER, "10-Band Graphic Equalizer", app.saved.eq_enabled) {
            out.push(Action::ToggleEq(!app.saved.eq_enabled));
        }
        let eq_desc = "Precision biquad peaking filter bank (31Hz–16kHz) for studio mastering & gaming audio tuning.";
        let (row2, _) = ui.allocate_exact_size(vec2(size.x, 22.0), Sense::hover());
        label(ui, pos2(row2.left() + 46.0, row2.center().y), Align2::LEFT_CENTER, eq_desc, sans(12.0), MUTED, size.x - 58.0);

        if app.saved.eq_enabled {
            ui.add_space(8.0);
            // EQ Presets Row
            let presets = crate::dsp::EqPreset::ALL;
            let (preset_row, _) = ui.allocate_exact_size(vec2(size.x, 34.0), Sense::hover());
            let pill_width = ((preset_row.width() - 5.0 * 8.0) / presets.len() as f32).max(70.0);
            for (idx, preset) in presets.iter().enumerate() {
                let pill_rect = Rect::from_min_size(
                    pos2(preset_row.left() + idx as f32 * (pill_width + 8.0), preset_row.top()),
                    vec2(pill_width, 30.0),
                );
                let selected = app.saved.eq_preset == *preset;
                let resp = ui.interact(pill_rect, Id::new(("eq-preset", idx)), Sense::click());
                let hover = hover_of(ui, &resp);
                let fill = if selected { ACCENT } else { veil(0.06 + 0.04 * hover) };
                let text_color = if selected { Color32::WHITE } else { TEXT };
                ui.painter().rect_filled(pill_rect, 15.0, fill);
                if selected {
                    ui.painter().rect_stroke(pill_rect, 15.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 80)), StrokeKind::Inside);
                }
                let short_name = match preset {
                    crate::dsp::EqPreset::Flat => "Flat",
                    crate::dsp::EqPreset::FpsFootsteps => "FPS Footsteps",
                    crate::dsp::EqPreset::BassPunch => "Bass Punch",
                    crate::dsp::EqPreset::VocalClarity => "Vocal",
                    crate::dsp::EqPreset::Cinematic => "Cinematic",
                    crate::dsp::EqPreset::Custom => "Custom",
                };
                ui.painter().text(pill_rect.center(), Align2::CENTER_CENTER, short_name, medium(11.5), text_color);
                if resp.clicked() {
                    out.push(Action::SetEqPreset(*preset));
                }
            }

            ui.add_space(12.0);
            // 10-Band Slider Rack
            let (rack_rect, _) = ui.allocate_exact_size(vec2(size.x, 150.0), Sense::hover());
            ui.painter().rect_filled(rack_rect, 10.0, veil(0.03));
            ui.painter().rect_stroke(rack_rect, 10.0, Stroke::new(1.0_f32, veil(0.06)), StrokeKind::Inside);

            let band_width = (rack_rect.width() - 20.0) / 10.0;
            let slider_height = 84.0;
            let slider_top = rack_rect.top() + 32.0;

            for b in 0..10 {
                let bx = rack_rect.left() + 10.0 + b as f32 * band_width + band_width / 2.0;
                let gain = app.saved.eq_gains[b];

                // Value dB readout on top
                let sign = if gain > 0.0 { "+" } else { "" };
                let val_str = format!("{}{:.1}", sign, gain);
                let val_color = if gain.abs() > 0.05 { ACCENT } else { FAINT };
                ui.painter().text(pos2(bx, rack_rect.top() + 16.0), Align2::CENTER_CENTER, val_str, sans(10.5), val_color);

                // Slider track
                let track_rect = Rect::from_center_size(pos2(bx, slider_top + slider_height / 2.0), vec2(4.0, slider_height));
                ui.painter().rect_filled(track_rect, 2.0, veil(0.12));

                // Center 0 dB notch mark
                let center_y = slider_top + slider_height / 2.0;
                ui.painter().line_segment(
                    [pos2(bx - 6.0, center_y), pos2(bx + 6.0, center_y)],
                    Stroke::new(1.0_f32, veil(0.2)),
                );

                // Thumb position: gain ranges from -12.0 (bottom) to +12.0 (top)
                let frac = ((gain + 12.0) / 24.0).clamp(0.0, 1.0);
                let thumb_y = slider_top + slider_height * (1.0 - frac);

                // Active fill from center 0dB to thumb
                let fill_top = thumb_y.min(center_y);
                let fill_bottom = thumb_y.max(center_y);
                let active_track = Rect::from_min_max(pos2(bx - 2.0, fill_top), pos2(bx + 2.0, fill_bottom));
                ui.painter().rect_filled(active_track, 2.0, ACCENT);

                // Draggable interaction area
                let hit_rect = Rect::from_center_size(pos2(bx, slider_top + slider_height / 2.0), vec2(band_width.min(32.0), slider_height + 12.0));
                let resp = ui.interact(hit_rect, Id::new(("eq-slider", b)), Sense::click_and_drag());
                if let (true, Some(pos)) = (resp.clicked() || resp.dragged(), resp.interact_pointer_pos()) {
                    let new_frac = (1.0 - ((pos.y - slider_top) / slider_height)).clamp(0.0, 1.0);
                    let mut new_gain = new_frac * 24.0 - 12.0;
                    // Snap to 0.5 dB
                    new_gain = (new_gain * 2.0).round() / 2.0;
                    out.push(Action::SetEqGain(b, new_gain));
                }

                // Thumb circle
                let thumb_hover = hover_of(ui, &resp);
                let thumb_rad = 6.0 + 1.0 * thumb_hover;
                ui.painter().circle_filled(pos2(bx, thumb_y), thumb_rad, TEXT);
                ui.painter().circle_filled(pos2(bx, thumb_y), thumb_rad - 2.0, ACCENT);

                // Frequency label at bottom
                let label_text = crate::dsp::EQ_BAND_LABELS[b];
                ui.painter().text(pos2(bx, rack_rect.bottom() - 14.0), Align2::CENTER_CENTER, label_text, sans(10.5), MUTED);
            }
        }
    });
}

fn performance_settings_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    let perf_mode = app.saved.performance_mode;
    let height = 2.0 * CARD_PAD + 6.0 * SETTING_ROW + 170.0;
    let card = section(ui, "Extreme Performance Engine & Resource Tuning", height, |ui| {
        let size = vec2(ui.available_width(), SETTING_ROW);

        // 1. Master Mode Switch
        if switch_row(ui, size, icon::LIGHTNING, "⚡ Master Performance Mode", perf_mode) {
            out.push(Action::TogglePerformanceMode);
        }
        let desc = "When active, AURI enforces hardware resource limits for maximum in-game FPS and zero background lag.";
        let (row, _) = ui.allocate_exact_size(vec2(size.x, 20.0), Sense::hover());
        label(ui, pos2(row.left() + 46.0, row.center().y), Align2::LEFT_CENTER, desc, sans(12.0), MUTED, size.x - 58.0);
        ui.add_space(8.0);

        // 2. Profile Selection Row
        let (prof_row, _) = ui.allocate_exact_size(vec2(size.x, 34.0), Sense::hover());
        let prof_width = (size.x - 18.0) / 4.0;
        for (i, &prof) in PerfProfile::ALL.iter().enumerate() {
            let active = app.saved.perf_profile == prof;
            let pill_rect = Rect::from_min_size(
                pos2(prof_row.left() + i as f32 * (prof_width + 6.0), prof_row.top()),
                vec2(prof_width, 32.0),
            );
            let resp = ui.interact(pill_rect, Id::new(("perf-prof-btn", i)), Sense::click());
            let hover = hover_of(ui, &resp);
            let fill = if active { ACCENT } else { veil(0.06 + 0.04 * hover) };
            let text_color = if active { Color32::WHITE } else { TEXT };
            ui.painter().rect_filled(pill_rect, 8.0, fill);
            if active {
                ui.painter().rect_stroke(pill_rect, 8.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 80)), StrokeKind::Inside);
            }
            let label_str = match prof {
                PerfProfile::CompetitiveGaming => "⚡ Gaming Max",
                PerfProfile::Balanced => "Balanced",
                PerfProfile::Audiophile => "Audiophile",
                PerfProfile::Custom => "Custom",
            };
            ui.painter().text(pill_rect.center(), Align2::CENTER_CENTER, label_str, medium(12.0), text_color);
            if resp.clicked() {
                out.push(Action::SetPerfProfile(prof));
            }
        }
        ui.add_space(10.0);

        // 3. Audio Stream Window Buffer Size
        let (buf_row, _) = ui.allocate_exact_size(vec2(size.x, 32.0), Sense::hover());
        ui.painter().text(pos2(buf_row.left() + 22.0, buf_row.center().y), Align2::CENTER_CENTER, icon::DATABASE, glyph(16.0), MUTED);
        label(ui, pos2(buf_row.left() + 46.0, buf_row.center().y), Align2::LEFT_CENTER, "Audio Stream Buffer", medium(13.5), TEXT, 160.0);
        let buf_opts: [(f32, &str); 4] = [(1.5, "1.5 MB"), (4.0, "4 MB"), (8.0, "8 MB"), (16.0, "16 MB")];
        let mut cur_x = buf_row.right() - 8.0;
        for (val, name) in buf_opts.iter().rev() {
            let active = (app.saved.perf_audio_buffer_mb - *val).abs() < 0.1;
            let pill_width = 62.0;
            let p_rect = Rect::from_min_size(pos2(cur_x - pill_width, buf_row.center().y - 13.0), vec2(pill_width, 26.0));
            cur_x -= pill_width + 6.0;
            let resp = ui.interact(p_rect, Id::new(("perf-buf", *name)), Sense::click());
            let hover = hover_of(ui, &resp);
            let fill = if active { ACCENT } else { veil(0.06 + 0.04 * hover) };
            let color = if active { Color32::WHITE } else { MUTED };
            ui.painter().rect_filled(p_rect, 6.0, fill);
            ui.painter().text(p_rect.center(), Align2::CENTER_CENTER, *name, sans(11.5), color);
            if resp.clicked() {
                out.push(Action::SetPerfAudioBuffer(*val));
            }
        }

        // 4. Artwork Texture Cache Budget
        let (art_row, _) = ui.allocate_exact_size(vec2(size.x, 32.0), Sense::hover());
        ui.painter().text(pos2(art_row.left() + 22.0, art_row.center().y), Align2::CENTER_CENTER, icon::IMAGE, glyph(16.0), MUTED);
        label(ui, pos2(art_row.left() + 46.0, art_row.center().y), Align2::LEFT_CENTER, "Cover Art Cache Budget", medium(13.5), TEXT, 160.0);
        let art_opts: [(usize, &str); 4] = [(2, "2 MB"), (4, "4 MB"), (12, "12 MB"), (32, "32 MB")];
        let mut cur_x = art_row.right() - 8.0;
        for (val, name) in art_opts.iter().rev() {
            let active = app.saved.perf_art_cache_mb == *val;
            let pill_width = 62.0;
            let p_rect = Rect::from_min_size(pos2(cur_x - pill_width, art_row.center().y - 13.0), vec2(pill_width, 26.0));
            cur_x -= pill_width + 6.0;
            let resp = ui.interact(p_rect, Id::new(("perf-art", *name)), Sense::click());
            let hover = hover_of(ui, &resp);
            let fill = if active { ACCENT } else { veil(0.06 + 0.04 * hover) };
            let color = if active { Color32::WHITE } else { MUTED };
            ui.painter().rect_filled(p_rect, 6.0, fill);
            ui.painter().text(p_rect.center(), Align2::CENTER_CENTER, *name, sans(11.5), color);
            if resp.clicked() {
                out.push(Action::SetPerfArtCache(*val));
            }
        }

        // 5. UI Animation Tick Rate
        let (fps_row, _) = ui.allocate_exact_size(vec2(size.x, 32.0), Sense::hover());
        ui.painter().text(pos2(fps_row.left() + 22.0, fps_row.center().y), Align2::CENTER_CENTER, icon::SPEEDOMETER, glyph(16.0), MUTED);
        label(ui, pos2(fps_row.left() + 46.0, fps_row.center().y), Align2::LEFT_CENTER, "HUD Refresh Rate", medium(13.5), TEXT, 160.0);
        let fps_opts: [(u32, &str); 3] = [(1, "1 FPS (Static)"), (30, "30 FPS"), (60, "60 FPS (Fluid)")];
        let mut cur_x = fps_row.right() - 8.0;
        for (val, name) in fps_opts.iter().rev() {
            let active = app.saved.perf_tick_fps == *val;
            let pill_width = 100.0;
            let p_rect = Rect::from_min_size(pos2(cur_x - pill_width, fps_row.center().y - 13.0), vec2(pill_width, 26.0));
            cur_x -= pill_width + 6.0;
            let resp = ui.interact(p_rect, Id::new(("perf-fps", *name)), Sense::click());
            let hover = hover_of(ui, &resp);
            let fill = if active { ACCENT } else { veil(0.06 + 0.04 * hover) };
            let color = if active { Color32::WHITE } else { MUTED };
            ui.painter().rect_filled(p_rect, 6.0, fill);
            ui.painter().text(p_rect.center(), Align2::CENTER_CENTER, *name, sans(11.0), color);
            if resp.clicked() {
                out.push(Action::SetPerfTickFps(*val));
            }
        }
        ui.add_space(4.0);

        // 6. Switches
        if switch_row(ui, size, icon::BROOM, "Aggressive RAM Purge (Every 30s & Track Switch)", app.saved.perf_aggressive_ram) {
            out.push(Action::SetPerfAggressiveRam(!app.saved.perf_aggressive_ram));
        }
        if switch_row(ui, size, icon::EYE_SLASH, "Disable Ambient Blurs & Background Shader Glows", app.saved.perf_disable_blurs) {
            out.push(Action::SetPerfDisableBlurs(!app.saved.perf_disable_blurs));
        }
        if switch_row(ui, size, icon::DATABASE, "Compact Audio & Stream Buffers (Sub-2MB Window)", app.saved.perf_compact_buffers) {
            out.push(Action::SetPerfCompactBuffers(!app.saved.perf_compact_buffers));
        }

        // 7. Manual Purge Button
        let (purge_row, resp) = ui.allocate_exact_size(size, Sense::click());
        let hover = hover_of(ui, &resp);
        ui.painter().rect_filled(purge_row, 7.0, veil(0.08 * hover));
        ui.painter().text(pos2(purge_row.left() + 22.0, purge_row.center().y), Align2::CENTER_CENTER, icon::CPU, glyph(17.0), ACCENT);
        label(ui, pos2(purge_row.left() + 46.0, purge_row.center().y), Align2::LEFT_CENTER, "Purge Working Set Memory Now", medium(13.5), TEXT, size.x - 180.0);
        let right_center = pos2(purge_row.right() - 14.0, purge_row.center().y);
        let _ = keycaps(ui, right_center, &["Clean RAM".to_owned()], ACCENT);
        if resp.clicked() {
            out.push(Action::PurgeMemory);
        }
    });
    let text = "Customizable profiles allow you to connect performance settings directly to your gaming setup or audio preference.";
    let at = pos2(card.left() + 4.0, card.bottom() + SECTION_GAP / 2.0);
    label(ui, at, Align2::LEFT_CENTER, text, sans(12.0), FAINT, card.width());
}

fn mini_mode_settings_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    let pal_color = app.saved.mini_palette.color();

    // 1. Live Preview Card
    let prev_height = if app.saved.mini_variant == MiniVariant::StudioCard { 230.0 } else { 160.0 };
    let card1 = section(ui, "Live Mini Mode Preview", prev_height, |ui| {
        let size = vec2(ui.available_width(), prev_height - 2.0 * CARD_PAD);
        let (slot, _) = ui.allocate_exact_size(size, Sense::hover());

        match app.saved.mini_variant {
            MiniVariant::ClassicHud => {
                let mini_w = 340.0_f32.min(slot.width());
                let mini_h = 96.0;
                let mini_rect = Rect::from_center_size(slot.center(), vec2(mini_w, mini_h));
                let bg_color = app.saved.mini_bg.color(pal_color);
                let alpha = (app.saved.mini_opacity.clamp(0.05, 1.0) * 255.0) as u8;
                ui.painter().rect_filled(mini_rect, 14.0, Color32::from_rgba_unmultiplied(bg_color.r(), bg_color.g(), bg_color.b(), alpha));
                ui.painter().rect_stroke(mini_rect, 14.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(pal_color.r(), pal_color.g(), pal_color.b(), 180)), StrokeKind::Inside);
                if app.saved.mini_grain {
                    paint_glass_grain(ui.painter(), mini_rect, app.saved.mini_grain_strength);
                }

                // Art
                let art_rect = Rect::from_min_size(pos2(mini_rect.left() + 12.0, mini_rect.center().y - 32.0), vec2(60.0, 60.0));
                if let Some(item) = app.now_playing() {
                    artwork(ui, &app.art, app.saved.covers.of(item), art_rect, 8.0);
                } else {
                    ui.painter().rect_filled(art_rect, 8.0, veil(0.08));
                    ui.painter().text(art_rect.center(), Align2::CENTER_CENTER, icon::VINYL_RECORD, glyph(24.0), pal_color);
                }

                // Title & Subtitle
                let p_left = art_rect.right() + 10.0;
                let title_str = app.now_playing().map_or("AURI Gaming Mode", |it| it.title.as_str());
                let sub_str = app.now_playing().map_or("Ready to play", |it| it.subtitle.as_str());
                label(ui, pos2(p_left, mini_rect.top() + 15.0), Align2::LEFT_CENTER, title_str, medium(12.5), TEXT, mini_rect.width() - 170.0);
                label(ui, pos2(p_left, mini_rect.top() + 29.0), Align2::LEFT_CENTER, sub_str, sans(10.5), MUTED, mini_rect.width() - 170.0);

                // Transport Controls (Centered in card preview)
                let play_center = pos2(mini_rect.center().x, mini_rect.top() + 48.0);
                ui.painter().circle_filled(play_center, 12.0, TEXT);
                ui.painter().text(play_center, Align2::CENTER_CENTER, icon::PAUSE, solid(12.0), PANEL);
                ui.painter().text(pos2(play_center.x - 24.0, play_center.y), Align2::CENTER_CENTER, icon::SKIP_BACK, solid(12.0), TEXT);
                ui.painter().text(pos2(play_center.x + 24.0, play_center.y), Align2::CENTER_CENTER, icon::SKIP_FORWARD, solid(12.0), TEXT);
                if app.saved.mini_show_shuffle_repeat {
                    ui.painter().text(pos2(play_center.x - 48.0, play_center.y), Align2::CENTER_CENTER, icon::SHUFFLE, glyph(11.0), if app.saved.shuffle { pal_color } else { MUTED });
                    ui.painter().text(pos2(play_center.x + 48.0, play_center.y), Align2::CENTER_CENTER, icon::ARROWS_CLOCKWISE, glyph(11.0), if app.saved.repeat != Repeat::Off { pal_color } else { MUTED });
                }

                // Top-right window controls (Restore, Pin Lock, Queue)
                ui.painter().text(pos2(mini_rect.right() - 16.0, mini_rect.top() + 14.0), Align2::CENTER_CENTER, icon::ARROWS_OUT_SIMPLE, glyph(13.0), MUTED);
                let pin_tone = if app.saved.mini_locked { pal_color } else { MUTED };
                ui.painter().text(pos2(mini_rect.right() - 34.0, mini_rect.top() + 14.0), Align2::CENTER_CENTER, icon::PUSH_PIN, glyph(13.0), pin_tone);
                if app.saved.mini_show_queue {
                    ui.painter().text(pos2(mini_rect.right() - 52.0, mini_rect.top() + 14.0), Align2::CENTER_CENTER, icon::LIST, glyph(13.0), MUTED);
                }

                // Far right vertical volume slider (Moved down and left)
                if app.saved.mini_show_volume {
                    let v_track = Rect::from_min_max(pos2(mini_rect.right() - 20.0, mini_rect.top() + 32.0), pos2(mini_rect.right() - 17.0, mini_rect.top() + 78.0));
                    ui.painter().rect_filled(v_track, 1.5, veil(0.10));
                    let v_fill = Rect::from_min_max(pos2(v_track.left(), v_track.bottom() - v_track.height() * app.saved.volume), v_track.max);
                    ui.painter().rect_filled(v_fill, 1.5, pal_color);
                }

                // Bottom Edge Progression Line (Moved down, with thumb knob & timestamps)
                let p_scrub = Rect::from_min_size(pos2(mini_rect.left() + 12.0, mini_rect.top() + 80.0), vec2(mini_rect.width() - 44.0, 3.5));
                ui.painter().rect_filled(p_scrub, 1.75, veil(0.12));
                let fill_w = p_scrub.width() * 0.45;
                ui.painter().rect_filled(Rect::from_min_size(p_scrub.min, vec2(fill_w, 3.5)), 1.75, pal_color);
                let thumb_x = p_scrub.left() + fill_w;
                ui.painter().circle_filled(pos2(thumb_x, p_scrub.center().y), 4.0, pal_color);
                ui.painter().circle_filled(pos2(thumb_x, p_scrub.center().y), 2.0, Color32::WHITE);

                if app.saved.mini_show_clock {
                    ui.painter().text(pos2(mini_rect.left() + 12.0, mini_rect.top() + 85.0), Align2::LEFT_CENTER, "1:24 / 3:45", sans(9.5), FAINT);
                }
            }
            MiniVariant::CapsulePill => {
                let mini_w = 310.0_f32.min(slot.width());
                let mini_h = 54.0;
                let mini_rect = Rect::from_center_size(slot.center(), vec2(mini_w, mini_h));
                let bg_color = app.saved.mini_bg.color(pal_color);
                let alpha = (app.saved.mini_opacity.clamp(0.05, 1.0) * 255.0) as u8;
                ui.painter().rect_filled(mini_rect, 27.0, Color32::from_rgba_unmultiplied(bg_color.r(), bg_color.g(), bg_color.b(), alpha));
                ui.painter().rect_stroke(mini_rect, 27.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(pal_color.r(), pal_color.g(), pal_color.b(), 180)), StrokeKind::Inside);
                if app.saved.mini_grain {
                    paint_glass_grain(ui.painter(), mini_rect, app.saved.mini_grain_strength);
                }

                // Circular Rotating Vinyl Art Badge
                let art_center = pos2(mini_rect.left() + 27.0, mini_rect.center().y);
                let angle = (ui.input(|i| i.time) * 1.5) as f32;
                let cover = app.now_playing().map_or("", |it| app.saved.covers.of(it));
                rotating_circular_artwork(ui, &app.art, cover, art_center, 18.0, angle, pal_color);

                // Compact Title / Subtitle
                let p_left = mini_rect.left() + 52.0;
                let title_str = app.now_playing().map_or("AURI Capsule", |it| it.title.as_str());
                let sub_str = app.now_playing().map_or("Ready", |it| it.subtitle.as_str());
                label(ui, pos2(p_left, mini_rect.center().y - 8.0), Align2::LEFT_CENTER, title_str, medium(11.5), TEXT, 75.0);
                label(ui, pos2(p_left, mini_rect.center().y + 8.0), Align2::LEFT_CENTER, sub_str, sans(9.5), MUTED, 75.0);

                // Centered Transport Controls
                let cy = mini_rect.center().y;
                let play_x = mini_rect.center().x + 10.0;
                ui.painter().circle_filled(pos2(play_x, cy), 12.0, TEXT);
                ui.painter().text(pos2(play_x, cy), Align2::CENTER_CENTER, icon::PAUSE, solid(12.0), PANEL);
                ui.painter().text(pos2(play_x - 22.0, cy), Align2::CENTER_CENTER, icon::SKIP_BACK, solid(11.0), TEXT);
                ui.painter().text(pos2(play_x + 22.0, cy), Align2::CENTER_CENTER, icon::SKIP_FORWARD, solid(11.0), TEXT);

                // Volume Icon Button
                ui.painter().text(pos2(play_x + 46.0, cy), Align2::CENTER_CENTER, icon::SPEAKER_HIGH, glyph(12.0), pal_color);

                // Utilities: Lock Pin & Restore
                let pin_tone = if app.saved.mini_locked { pal_color } else { MUTED };
                ui.painter().text(pos2(mini_rect.right() - 32.0, cy), Align2::CENTER_CENTER, icon::PUSH_PIN, glyph(12.0), pin_tone);
                ui.painter().text(pos2(mini_rect.right() - 15.0, cy), Align2::CENTER_CENTER, icon::ARROWS_OUT_SIMPLE, glyph(12.0), MUTED);
                // No bottom scrubber line
            }
            MiniVariant::StudioCard => {
                let mini_w = 210.0_f32.min(slot.width());
                let mini_h = 205.0;
                let mini_rect = Rect::from_center_size(slot.center(), vec2(mini_w, mini_h));
                let bg_color = app.saved.mini_bg.color(pal_color);
                let alpha = (app.saved.mini_opacity.clamp(0.05, 1.0) * 255.0) as u8;
                ui.painter().rect_filled(mini_rect, 14.0, Color32::from_rgba_unmultiplied(bg_color.r(), bg_color.g(), bg_color.b(), alpha));
                ui.painter().rect_stroke(mini_rect, 14.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(pal_color.r(), pal_color.g(), pal_color.b(), 180)), StrokeKind::Inside);
                if app.saved.mini_grain {
                    paint_glass_grain(ui.painter(), mini_rect, app.saved.mini_grain_strength);
                }

                // Top Utilities
                ui.painter().text(pos2(mini_rect.right() - 14.0, mini_rect.top() + 14.0), Align2::CENTER_CENTER, icon::ARROWS_OUT_SIMPLE, glyph(11.0), MUTED);
                let pin_tone = if app.saved.mini_locked { pal_color } else { MUTED };
                ui.painter().text(pos2(mini_rect.right() - 30.0, mini_rect.top() + 14.0), Align2::CENTER_CENTER, icon::PUSH_PIN, glyph(11.0), pin_tone);
                if app.saved.mini_show_queue {
                    ui.painter().text(pos2(mini_rect.right() - 46.0, mini_rect.top() + 14.0), Align2::CENTER_CENTER, icon::LIST, glyph(11.0), MUTED);
                }

                // Typography Above Image
                let title_str = app.now_playing().map_or("AURI Studio", |it| it.title.as_str());
                let sub_str = app.now_playing().map_or("Ready to play", |it| it.subtitle.as_str());
                label(ui, pos2(mini_rect.left() + 12.0, mini_rect.top() + 11.0), Align2::LEFT_CENTER, title_str, bold(11.5), TEXT, mini_rect.width() - 65.0);
                label(ui, pos2(mini_rect.left() + 12.0, mini_rect.top() + 23.0), Align2::LEFT_CENTER, sub_str, sans(9.5), MUTED, mini_rect.width() - 65.0);

                // Center Square Artwork
                let art_rect = Rect::from_center_size(pos2(mini_rect.center().x, mini_rect.top() + 84.0), vec2(100.0, 100.0));
                if let Some(item) = app.now_playing() {
                    artwork(ui, &app.art, app.saved.covers.of(item), art_rect, 8.0);
                } else {
                    ui.painter().rect_filled(art_rect, 8.0, veil(0.08));
                    ui.painter().text(art_rect.center(), Align2::CENTER_CENTER, icon::VINYL_RECORD, glyph(28.0), pal_color);
                }

                // Transport Beneath Image
                let cy = mini_rect.top() + 146.0;
                let play_x = mini_rect.center().x;
                ui.painter().circle_filled(pos2(play_x, cy), 12.0, TEXT);
                ui.painter().text(pos2(play_x, cy), Align2::CENTER_CENTER, icon::PAUSE, solid(12.0), PANEL);
                ui.painter().text(pos2(play_x - 22.0, cy), Align2::CENTER_CENTER, icon::SKIP_BACK, solid(11.0), TEXT);
                ui.painter().text(pos2(play_x + 22.0, cy), Align2::CENTER_CENTER, icon::SKIP_FORWARD, solid(11.0), TEXT);
                ui.painter().text(pos2(play_x - 44.0, cy), Align2::CENTER_CENTER, icon::SHUFFLE, glyph(10.0), MUTED);
                ui.painter().text(pos2(play_x + 44.0, cy), Align2::CENTER_CENTER, icon::ARROWS_CLOCKWISE, glyph(10.0), MUTED);

                // Scrubber Line
                let p_scrub = Rect::from_min_size(pos2(mini_rect.left() + 14.0, mini_rect.top() + 168.0), vec2(mini_rect.width() - 28.0, 3.0));
                ui.painter().rect_filled(p_scrub, 1.5, veil(0.12));
                let fill_w = p_scrub.width() * 0.45;
                ui.painter().rect_filled(Rect::from_min_size(p_scrub.min, vec2(fill_w, 3.0)), 1.5, pal_color);
                let dot_center = pos2(p_scrub.left() + fill_w, p_scrub.center().y);
                ui.painter().circle_filled(dot_center, 3.5, TEXT);
                ui.painter().circle_stroke(dot_center, 3.5, Stroke::new(1.0_f32, pal_color));

                // Bottom: Time on Left, Speaker + Volume on Right
                let bot_y = mini_rect.top() + 185.0;
                ui.painter().text(pos2(mini_rect.left() + 14.0, bot_y), Align2::LEFT_CENTER, "1:24 / 3:45", sans(9.0), FAINT);
                ui.painter().text(pos2(mini_rect.right() - 56.0, bot_y), Align2::CENTER_CENTER, icon::SPEAKER_HIGH, glyph(10.0), MUTED);
                let v_track = Rect::from_min_max(pos2(mini_rect.right() - 48.0, bot_y - 1.5), pos2(mini_rect.right() - 14.0, bot_y + 1.5));
                ui.painter().rect_filled(v_track, 1.5, veil(0.10));
                let v_fill = Rect::from_min_max(v_track.min, pos2(v_track.left() + v_track.width() * app.saved.volume, v_track.bottom()));
                ui.painter().rect_filled(v_fill, 1.5, pal_color);
            }
        }
    });
    let text = "Live interactive preview dynamically reflects your active opacity, background tint, accent palette, grain, and tools.";
    label(ui, pos2(card1.left() + 4.0, card1.bottom() + SECTION_GAP / 2.0), Align2::LEFT_CENTER, text, sans(12.0), FAINT, card1.width());

    // 2. Customization & Theme Section
    let cust_height = 2.0 * CARD_PAD + 9.0 * SETTING_ROW + 220.0;
    let card2 = section(ui, "Appearance, Color Themes & Features", cust_height, |ui| {
        let size = vec2(ui.available_width(), SETTING_ROW);

        // Mini Variant Layout Selector
        eyebrow(ui, "Mini Player Variant & Layout", 0.0);
        let (var_row, _) = ui.allocate_exact_size(vec2(size.x, 34.0), Sense::hover());
        let var_w = (size.x - 10.0) / 3.0;
        for (i, &variant) in MiniVariant::ALL.iter().enumerate() {
            let active = app.saved.mini_variant == variant;
            let slot = Rect::from_min_size(
                pos2(var_row.left() + i as f32 * (var_w + 5.0), var_row.top()),
                vec2(var_w, 32.0),
            );
            let resp = ui.interact(slot, Id::new(("mini-var-btn", i)), Sense::click());
            let hover = hover_of(ui, &resp);
            ui.painter().rect_filled(slot, 8.0, veil(0.06 + 0.05 * hover));
            if active {
                ui.painter().rect_stroke(slot, 8.0, Stroke::new(1.5_f32, pal_color), StrokeKind::Inside);
            }
            ui.painter().text(slot.center(), Align2::CENTER_CENTER, variant.name(), medium(12.0), if active { TEXT } else { MUTED });
            if resp.clicked() {
                out.push(Action::SetMiniVariant(variant));
            }
        }
        ui.add_space(8.0);

        // Theme Palette Swatches
        eyebrow(ui, "Accent Color Theme", 0.0);
        let (pal_row, _) = ui.allocate_exact_size(vec2(size.x, 34.0), Sense::hover());
        let pal_w = (size.x - 35.0) / 8.0;
        for (i, &palette) in MiniPalette::ALL.iter().enumerate() {
            let active = app.saved.mini_palette == palette;
            let swatch_rect = Rect::from_min_size(
                pos2(pal_row.left() + i as f32 * (pal_w + 5.0), pal_row.top()),
                vec2(pal_w, 32.0),
            );
            let resp = ui.interact(swatch_rect, Id::new(("mini-pal", i)), Sense::click());
            let hover = hover_of(ui, &resp);
            let p_col = palette.color();
            ui.painter().rect_filled(swatch_rect, 8.0, veil(0.06 + 0.05 * hover));
            if active {
                ui.painter().rect_stroke(swatch_rect, 8.0, Stroke::new(1.5_f32, p_col), StrokeKind::Inside);
            }
            ui.painter().circle_filled(pos2(swatch_rect.left() + 12.0, swatch_rect.center().y), 5.0, p_col);
            let short_name = match palette {
                MiniPalette::CyberViolet => "Violet",
                MiniPalette::NeonEmerald => "Emerald",
                MiniPalette::ElectricCyan => "Cyan",
                MiniPalette::SunsetCrimson => "Crimson",
                MiniPalette::AmberGold => "Amber",
                MiniPalette::CobaltBlue => "Cobalt",
                MiniPalette::SakuraPink => "Sakura",
                MiniPalette::Monochrome => "Mono",
            };
            ui.painter().text(pos2(swatch_rect.left() + 22.0, swatch_rect.center().y), Align2::LEFT_CENTER, short_name, medium(10.5), if active { TEXT } else { MUTED });
            if resp.clicked() {
                out.push(Action::SetMiniPalette(palette));
            }
        }
        ui.add_space(8.0);

        // Mini Mode Background Tint Swatches
        eyebrow(ui, "Mini Background Canvas Tint", 0.0);
        let (bg_row, _) = ui.allocate_exact_size(vec2(size.x, 34.0), Sense::hover());
        let bg_w = (size.x - 25.0) / 6.0;
        for (i, &bg) in MiniBackground::ALL.iter().enumerate() {
            let active = app.saved.mini_bg == bg;
            let swatch_rect = Rect::from_min_size(
                pos2(bg_row.left() + i as f32 * (bg_w + 5.0), bg_row.top()),
                vec2(bg_w, 32.0),
            );
            let resp = ui.interact(swatch_rect, Id::new(("mini-bg", i)), Sense::click());
            let hover = hover_of(ui, &resp);
            let col = bg.color(pal_color);
            ui.painter().rect_filled(swatch_rect, 8.0, col);
            let stroke_col = if active { pal_color } else { veil(0.14 + 0.08 * hover) };
            ui.painter().rect_stroke(swatch_rect, 8.0, Stroke::new(if active { 1.5_f32 } else { 1.0 }, stroke_col), StrokeKind::Inside);
            ui.painter().text(swatch_rect.center(), Align2::CENTER_CENTER, bg.name(), medium(11.0), if active { TEXT } else { MUTED });
            if resp.clicked() {
                out.push(Action::SetMiniBackground(bg));
            }
        }
        ui.add_space(8.0);

        // Glass Opacity Slider
        let (op_row, _) = ui.allocate_exact_size(vec2(size.x, 32.0), Sense::hover());
        ui.painter().text(pos2(op_row.left() + 22.0, op_row.center().y), Align2::CENTER_CENTER, icon::EYEDROPPER, glyph(16.0), MUTED);
        label(ui, pos2(op_row.left() + 46.0, op_row.center().y), Align2::LEFT_CENTER, "Glass Card Opacity", medium(13.5), TEXT, 160.0);
        let slider_w = 160.0;
        let slider_rect = Rect::from_min_size(pos2(op_row.right() - slider_w - 50.0, op_row.center().y - 3.0), vec2(slider_w, 6.0));
        let op_resp = ui.interact(slider_rect.expand(4.0), Id::new("mini-opacity-slider"), Sense::click_and_drag());
        if let (true, Some(pos)) = (op_resp.clicked() || op_resp.dragged(), op_resp.interact_pointer_pos()) {
            let frac = ((pos.x - slider_rect.left()) / slider_rect.width()).clamp(0.0, 1.0);
            let val = 0.05 + frac * 0.95;
            out.push(Action::SetMiniOpacity(val));
        }
        ui.painter().rect_filled(slider_rect, 3.0, veil(0.12));
        let frac = ((app.saved.mini_opacity - 0.05) / 0.95).clamp(0.0, 1.0);
        ui.painter().rect_filled(Rect::from_min_size(slider_rect.min, vec2(slider_rect.width() * frac, 6.0)), 3.0, pal_color);
        ui.painter().circle_filled(pos2(slider_rect.left() + slider_rect.width() * frac, slider_rect.center().y), 6.0, TEXT);
        let op_text = format!("{:.0}%", app.saved.mini_opacity * 100.0);
        ui.painter().text(pos2(op_row.right() - 10.0, op_row.center().y), Align2::RIGHT_CENTER, op_text, sans(12.0), pal_color);
        ui.add_space(8.0);

        // Frosted Grain Effect Toggle & Slider
        if switch_row(ui, size, icon::SPARKLE, "Frosted Acrylic Grain Effect", app.saved.mini_grain) {
            out.push(Action::SetMiniGrain(!app.saved.mini_grain));
        }
        if app.saved.mini_grain {
            let (grain_row, _) = ui.allocate_exact_size(vec2(size.x, 32.0), Sense::hover());
            label(ui, pos2(grain_row.left() + 46.0, grain_row.center().y), Align2::LEFT_CENTER, "Grain Intensity", sans(12.5), MUTED, 160.0);
            let slider_w = 160.0;
            let slider_rect = Rect::from_min_size(pos2(grain_row.right() - slider_w - 50.0, grain_row.center().y - 3.0), vec2(slider_w, 6.0));
            let g_resp = ui.interact(slider_rect.expand(4.0), Id::new("mini-grain-slider"), Sense::click_and_drag());
            if let (true, Some(pos)) = (g_resp.clicked() || g_resp.dragged(), g_resp.interact_pointer_pos()) {
                let frac = ((pos.x - slider_rect.left()) / slider_rect.width()).clamp(0.0, 1.0);
                let val = 0.05 + frac * 0.35;
                out.push(Action::SetMiniGrainStrength(val));
            }
            ui.painter().rect_filled(slider_rect, 3.0, veil(0.12));
            let g_frac = ((app.saved.mini_grain_strength - 0.05) / 0.35).clamp(0.0, 1.0);
            ui.painter().rect_filled(Rect::from_min_size(slider_rect.min, vec2(slider_rect.width() * g_frac, 6.0)), 3.0, pal_color);
            ui.painter().circle_filled(pos2(slider_rect.left() + slider_rect.width() * g_frac, slider_rect.center().y), 6.0, TEXT);
            let g_text = format!("{:.0}%", app.saved.mini_grain_strength * 100.0);
            ui.painter().text(pos2(grain_row.right() - 10.0, grain_row.center().y), Align2::RIGHT_CENTER, g_text, sans(12.0), pal_color);
            ui.add_space(6.0);
        }

        // Feature Toggles
        if switch_row(ui, size, icon::SHUFFLE, "Show Shuffle & Repeat Controls", app.saved.mini_show_shuffle_repeat) {
            out.push(Action::SetMiniShowShuffleRepeat(!app.saved.mini_show_shuffle_repeat));
        }
        if switch_row(ui, size, icon::LIST, "Show Upcoming Queue Drawer Button", app.saved.mini_show_queue) {
            out.push(Action::SetMiniShowQueue(!app.saved.mini_show_queue));
        }
        if switch_row(ui, size, icon::SPEAKER_HIGH, "Show Mini Volume Slider", app.saved.mini_show_volume) {
            out.push(Action::SetMiniShowVolume(!app.saved.mini_show_volume));
        }
        if switch_row(ui, size, icon::CLOCK, "Show Track Clock & Elapsed Time", app.saved.mini_show_clock) {
            out.push(Action::SetMiniShowClock(!app.saved.mini_show_clock));
        }
        if switch_row(ui, size, icon::PUSH_PIN, "Always On Top Window Overlay", app.saved.mini_always_on_top) {
            out.push(Action::SetMiniAlwaysOnTop(!app.saved.mini_always_on_top));
        }
        if switch_row(ui, size, icon::PUSH_PIN, "Lock Mini Player Position (Disable Dragging)", app.saved.mini_locked) {
            out.push(Action::ToggleMiniLock);
        }
        ui.add_space(8.0);

        // Launch Mini Mode Button
        let (launch_row, resp) = ui.allocate_exact_size(size, Sense::click());
        let hover = hover_of(ui, &resp);
        ui.painter().rect_filled(launch_row, 7.0, pal_color.gamma_multiply(0.18 + 0.12 * hover));
        ui.painter().rect_stroke(launch_row, 7.0, Stroke::new(1.0_f32, pal_color.gamma_multiply(0.5)), StrokeKind::Inside);
        ui.painter().text(pos2(launch_row.left() + 22.0, launch_row.center().y), Align2::CENTER_CENTER, icon::PICTURE_IN_PICTURE, glyph(17.0), pal_color);
        label(ui, pos2(launch_row.left() + 46.0, launch_row.center().y), Align2::LEFT_CENTER, "Switch To Floating Glass Mini Player", medium(13.5), TEXT, size.x - 180.0);
        let right_center = pos2(launch_row.right() - 14.0, launch_row.center().y);
        let _ = keycaps(ui, right_center, &["Open Mini".to_owned()], pal_color);
        if resp.clicked() {
            out.push(Action::ToggleMiniMode);
        }
    });
    let text = "Mini Mode floats seamlessly over games and applications with zero window borders and full transparency.";
    let at = pos2(card2.left() + 4.0, card2.bottom() + SECTION_GAP / 2.0);
    label(ui, at, Align2::LEFT_CENTER, text, sans(12.0), FAINT, card2.width());
}

fn appearance_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    let height = 2.0 * CARD_PAD + 6.0 * SETTING_ROW + 180.0;
    section(ui, "Appearance & Theming (Normal Mode)", height, |ui| {
        let size = vec2(ui.available_width(), SETTING_ROW);

        // Accent Theme Swatches
        eyebrow(ui, "Accent Color Theme", 0.0);
        let (pal_row, _) = ui.allocate_exact_size(vec2(size.x, 34.0), Sense::hover());
        let pal_w = (size.x - 35.0) / 8.0;
        for (i, &palette) in MiniPalette::ALL.iter().enumerate() {
            let active = app.saved.theme_palette == palette;
            let swatch_rect = Rect::from_min_size(
                pos2(pal_row.left() + i as f32 * (pal_w + 5.0), pal_row.top()),
                vec2(pal_w, 32.0),
            );
            let resp = ui.interact(swatch_rect, Id::new(("theme-pal", i)), Sense::click());
            let hover = hover_of(ui, &resp);
            let p_col = palette.color();
            ui.painter().rect_filled(swatch_rect, 8.0, veil(0.06 + 0.05 * hover));
            if active {
                ui.painter().rect_stroke(swatch_rect, 8.0, Stroke::new(1.5_f32, p_col), StrokeKind::Inside);
            }
            ui.painter().circle_filled(pos2(swatch_rect.left() + 12.0, swatch_rect.center().y), 5.0, p_col);
            let short_name = match palette {
                MiniPalette::CyberViolet => "Violet",
                MiniPalette::NeonEmerald => "Emerald",
                MiniPalette::ElectricCyan => "Cyan",
                MiniPalette::SunsetCrimson => "Crimson",
                MiniPalette::AmberGold => "Amber",
                MiniPalette::CobaltBlue => "Cobalt",
                MiniPalette::SakuraPink => "Sakura",
                MiniPalette::Monochrome => "Mono",
            };
            ui.painter().text(pos2(swatch_rect.left() + 22.0, swatch_rect.center().y), Align2::LEFT_CENTER, short_name, medium(10.5), if active { TEXT } else { MUTED });
            if resp.clicked() {
                out.push(Action::SetThemePalette(palette));
            }
        }
        ui.add_space(8.0);

        // Base Canvas Swatches
        eyebrow(ui, "Base Canvas Background", 0.0);
        let (base_row, _) = ui.allocate_exact_size(vec2(size.x, 34.0), Sense::hover());
        let base_w = (size.x - 15.0) / 4.0;
        for (i, &base) in ThemeBase::ALL.iter().enumerate() {
            let active = app.saved.theme_base == base;
            let swatch_rect = Rect::from_min_size(
                pos2(base_row.left() + i as f32 * (base_w + 5.0), base_row.top()),
                vec2(base_w, 32.0),
            );
            let resp = ui.interact(swatch_rect, Id::new(("theme-base", i)), Sense::click());
            let hover = hover_of(ui, &resp);
            let col = base.color();
            ui.painter().rect_filled(swatch_rect, 8.0, col);
            let stroke_col = if active { app.saved.theme_palette.color() } else { veil(0.12 + 0.08 * hover) };
            ui.painter().rect_stroke(swatch_rect, 8.0, Stroke::new(if active { 1.5_f32 } else { 1.0 }, stroke_col), StrokeKind::Inside);
            ui.painter().text(swatch_rect.center(), Align2::CENTER_CENTER, base.name(), medium(11.0), if active { TEXT } else { MUTED });
            if resp.clicked() {
                out.push(Action::SetThemeBase(base));
            }
        }
        ui.add_space(8.0);

        // Ambient Glow Switch
        if switch_row(ui, size, icon::SUN, "Dynamic Ambient Cover Glow", app.saved.ambient_glow) {
            out.push(Action::SetAmbientGlow(!app.saved.ambient_glow));
        }
        let glow_desc = "Projects subtle, diffused ambient backlights matching active song artwork on the canvas.";
        let (glow_desc_row, _) = ui.allocate_exact_size(vec2(size.x, 20.0), Sense::hover());
        label(ui, pos2(glow_desc_row.left() + 46.0, glow_desc_row.center().y), Align2::LEFT_CENTER, glow_desc, sans(11.5), MUTED, size.x - 58.0);
        ui.add_space(6.0);

        // Selection Glow Switch
        if switch_row(ui, size, icon::SPARKLE, "High-Visibility Selection Outline & Glow", app.saved.selection_glow) {
            out.push(Action::SetSelectionGlow(!app.saved.selection_glow));
        }
        let sel_desc = "Adds a luminous accent border and soft aura around active navigation tabs and selections.";
        let (sel_desc_row, _) = ui.allocate_exact_size(vec2(size.x, 20.0), Sense::hover());
        label(ui, pos2(sel_desc_row.left() + 46.0, sel_desc_row.center().y), Align2::LEFT_CENTER, sel_desc, sans(11.5), MUTED, size.x - 58.0);
        ui.add_space(6.0);

        // Sidebar Switch
        if switch_row(ui, size, icon::SIDEBAR, "Navigation Sidebar", app.saved.sidebar_open) {
            out.push(Action::ToggleSidebar);
        }
        let sb_desc = "Shows or hides the left navigation panel with your playlists, home, and library.";
        let (sb_desc_row, _) = ui.allocate_exact_size(vec2(size.x, 20.0), Sense::hover());
        label(ui, pos2(sb_desc_row.left() + 46.0, sb_desc_row.center().y), Align2::LEFT_CENTER, sb_desc, sans(11.5), MUTED, size.x - 58.0);
        ui.add_space(6.0);

        // Queue Switch
        if switch_row(ui, size, icon::LIST, "Song Queue Sidebar", app.saved.queue_open) {
            out.push(Action::ToggleQueue);
        }
        let q_desc = "Shows or hides the right side panel with upcoming tracks and current track artwork.";
        let (q_desc_row, _) = ui.allocate_exact_size(vec2(size.x, 20.0), Sense::hover());
        label(ui, pos2(q_desc_row.left() + 46.0, q_desc_row.center().y), Align2::LEFT_CENTER, q_desc, sans(11.5), MUTED, size.x - 58.0);
    });
}

fn player_bar_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    let height = 2.0 * CARD_PAD + 4.0 * SETTING_ROW + 130.0;
    section(ui, "Bottom Playback Bar & Deck (Normal Mode)", height, |ui| {
        let size = vec2(ui.available_width(), SETTING_ROW);

        // Bar Layout Selector: Docked vs Floating Capsule Pill
        eyebrow(ui, "Deck Structure & Layout", 0.0);
        let (layout_row, _) = ui.allocate_exact_size(vec2(size.x, 34.0), Sense::hover());
        let count = BarLayout::ALL.len() as f32;
        let pill_w = (size.x - 5.0 * (count - 1.0)) / count;
        for (i, &layout) in BarLayout::ALL.iter().enumerate() {
            let active = app.saved.bar_layout == layout;
            let slot = Rect::from_min_size(
                pos2(layout_row.left() + i as f32 * (pill_w + 5.0), layout_row.top()),
                vec2(pill_w, 32.0),
            );
            let resp = ui.interact(slot, Id::new(("bar-layout-btn", i)), Sense::click());
            let hover = hover_of(ui, &resp);
            ui.painter().rect_filled(slot, 8.0, veil(0.06 + 0.05 * hover));
            if active {
                ui.painter().rect_stroke(slot, 8.0, Stroke::new(1.5_f32, app.saved.theme_palette.color()), StrokeKind::Inside);
            }
            ui.painter().text(slot.center(), Align2::CENTER_CENTER, layout.name(), medium(12.0), if active { TEXT } else { MUTED });
            if resp.clicked() {
                out.push(Action::SetBarLayout(layout));
            }
        }
        ui.add_space(8.0);

        // Glassmorphism Switch
        if switch_row(ui, size, icon::DROP, "Glassmorphism & Translucency", app.saved.bar_glass) {
            out.push(Action::SetBarGlass(!app.saved.bar_glass));
        }

        // Bar Opacity Slider
        if app.saved.bar_glass {
            let (op_row, _) = ui.allocate_exact_size(size, Sense::hover());
            ui.painter().text(pos2(op_row.left() + 22.0, op_row.center().y), Align2::CENTER_CENTER, icon::SLIDERS, glyph(17.0), MUTED);
            label(ui, pos2(op_row.left() + 46.0, op_row.center().y), Align2::LEFT_CENTER, "Deck Opacity", medium(13.0), TEXT, 140.0);
            let val_str = format!("{:.0}%", app.saved.bar_opacity * 100.0);
            label(ui, pos2(op_row.right() - 40.0, op_row.center().y), Align2::RIGHT_CENTER, &val_str, sans(12.0), ACCENT, 50.0);
            let track = Rect::from_min_max(pos2(op_row.left() + 190.0, op_row.center().y - 3.0), pos2(op_row.right() - 55.0, op_row.center().y + 3.0));
            let resp = ui.interact(track.expand(4.0), Id::new("bar-opacity-slider"), Sense::click_and_drag());
            if let (true, Some(pointer)) = (resp.dragged() || resp.clicked(), resp.interact_pointer_pos()) {
                let frac = ((pointer.x - track.left()) / track.width()).clamp(0.0, 1.0);
                out.push(Action::SetBarOpacity(0.20 + frac * 0.80));
            }
            let cur_frac = ((app.saved.bar_opacity - 0.20) / 0.80).clamp(0.0, 1.0);
            ui.painter().rect_filled(track, 2.0, veil(0.12));
            ui.painter().rect_filled(Rect::from_min_max(track.min, pos2(track.left() + track.width() * cur_frac, track.max.y)), 2.0, app.saved.theme_palette.color());
            ui.painter().circle_filled(pos2(track.left() + track.width() * cur_frac, track.center().y), 6.0, TEXT);
        }

        // Frosted Grain Switch
        if switch_row(ui, size, icon::SPARKLE, "Frosted Acrylic Grain Filter", app.saved.bar_grain) {
            out.push(Action::SetBarGrain(!app.saved.bar_grain));
        }

        // Grain Strength Slider
        if app.saved.bar_grain {
            let (gr_row, _) = ui.allocate_exact_size(size, Sense::hover());
            ui.painter().text(pos2(gr_row.left() + 22.0, gr_row.center().y), Align2::CENTER_CENTER, icon::WAVEFORM, glyph(17.0), MUTED);
            label(ui, pos2(gr_row.left() + 46.0, gr_row.center().y), Align2::LEFT_CENTER, "Grain Intensity", medium(13.0), TEXT, 140.0);
            let val_str = format!("{:.0}%", app.saved.bar_grain_strength * 100.0);
            label(ui, pos2(gr_row.right() - 40.0, gr_row.center().y), Align2::RIGHT_CENTER, &val_str, sans(12.0), ACCENT, 50.0);
            let track = Rect::from_min_max(pos2(gr_row.left() + 190.0, gr_row.center().y - 3.0), pos2(gr_row.right() - 55.0, gr_row.center().y + 3.0));
            let resp = ui.interact(track.expand(4.0), Id::new("bar-grain-slider"), Sense::click_and_drag());
            if let (true, Some(pointer)) = (resp.dragged() || resp.clicked(), resp.interact_pointer_pos()) {
                let frac = ((pointer.x - track.left()) / track.width()).clamp(0.0, 1.0);
                out.push(Action::SetBarGrainStrength(0.05 + frac * 0.30));
            }
            let cur_frac = ((app.saved.bar_grain_strength - 0.05) / 0.30).clamp(0.0, 1.0);
            ui.painter().rect_filled(track, 2.0, veil(0.12));
            ui.painter().rect_filled(Rect::from_min_max(track.min, pos2(track.left() + track.width() * cur_frac, track.max.y)), 2.0, app.saved.theme_palette.color());
            ui.painter().circle_filled(pos2(track.left() + track.width() * cur_frac, track.center().y), 6.0, TEXT);
        }
    });
}

fn storage_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    let height = 2.0 * CARD_PAD + 2.0 * SETTING_ROW + 36.0;
    section(ui, "Storage & Local Downloads", height, |ui| {
        let size = vec2(ui.available_width(), SETTING_ROW);
        let (row, _) = ui.allocate_exact_size(size, Sense::hover());
        ui.painter().text(pos2(row.left() + 22.0, row.center().y), Align2::CENTER_CENTER, icon::FOLDER_SIMPLE, glyph(17.0), MUTED);
        label(ui, pos2(row.left() + 46.0, row.center().y - 9.0), Align2::LEFT_CENTER, "Main Storage & Downloads Directory", medium(13.5), TEXT, size.x - 220.0);
        let display_path = format!("{}\\downloads", app.saved.storage_dir);
        label(ui, pos2(row.left() + 46.0, row.center().y + 9.0), Align2::LEFT_CENTER, &display_path, sans(12.0), ACCENT, size.x - 220.0);

        let btn_w = 80.0;
        let browse_btn_rect = Rect::from_min_size(pos2(row.right() - 3.0 * btn_w - 12.0, row.center().y - 14.0), vec2(btn_w, 28.0));
        let browse_resp = ui.interact(browse_btn_rect, Id::new("storage-browse-btn"), Sense::click());
        let browse_hover = hover_of(ui, &browse_resp);
        ui.painter().rect_filled(browse_btn_rect, 6.0, veil(0.06 + 0.05 * browse_hover));
        ui.painter().rect_stroke(browse_btn_rect, 6.0, Stroke::new(1.0_f32, veil(0.12)), StrokeKind::Inside);
        ui.painter().text(browse_btn_rect.center(), Align2::CENTER_CENTER, "Browse...", medium(11.5), TEXT);
        if browse_resp.clicked() {
            out.push(Action::BrowseStorageDir);
        }

        let open_btn_rect = Rect::from_min_size(pos2(row.right() - 2.0 * btn_w - 6.0, row.center().y - 14.0), vec2(btn_w, 28.0));
        let open_resp = ui.interact(open_btn_rect, Id::new("storage-open-btn"), Sense::click());
        let open_hover = hover_of(ui, &open_resp);
        ui.painter().rect_filled(open_btn_rect, 6.0, veil(0.06 + 0.05 * open_hover));
        ui.painter().rect_stroke(open_btn_rect, 6.0, Stroke::new(1.0_f32, veil(0.12)), StrokeKind::Inside);
        ui.painter().text(open_btn_rect.center(), Align2::CENTER_CENTER, "Open", medium(11.5), TEXT);
        if open_resp.clicked() {
            out.push(Action::OpenStorageDir);
        }

        let reset_btn_rect = Rect::from_min_size(pos2(row.right() - btn_w, row.center().y - 14.0), vec2(btn_w, 28.0));
        let reset_resp = ui.interact(reset_btn_rect, Id::new("storage-reset-btn"), Sense::click());
        let reset_hover = hover_of(ui, &reset_resp);
        ui.painter().rect_filled(reset_btn_rect, 6.0, veil(0.06 + 0.05 * reset_hover));
        ui.painter().rect_stroke(reset_btn_rect, 6.0, Stroke::new(1.0_f32, veil(0.12)), StrokeKind::Inside);
        ui.painter().text(reset_btn_rect.center(), Align2::CENTER_CENTER, "Default", sans(11.5), MUTED);
        if reset_resp.clicked() {
            out.push(Action::ResetStorageDir);
        }

        let desc = "Right-click any song in Home, Liked, or Search and select 'Download song' to save locally in high bitrate (Opus/160k or M4A). User preferences & playlists are saved in %APPDATA%\\auri\\app.ron.";
        let (_desc_row, _) = ui.allocate_exact_size(vec2(size.x, 32.0), Sense::hover());
        paragraph(ui, desc, sans(11.5), FAINT, 2);
    });
}

fn about_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    let height = 2.0 * CARD_PAD + SETTING_ROW;
    let card = section(ui, "About & Updates", height, |ui| {
        let size = vec2(ui.available_width(), SETTING_ROW);
        let (row, _) = ui.allocate_exact_size(size, Sense::hover());
        ui.painter().text(pos2(row.left() + 22.0, row.center().y), Align2::CENTER_CENTER, icon::INFO, glyph(17.0), MUTED);
        let text = format!("AURI v{} (Gaming Edition)", crate::VERSION);
        label(ui, pos2(row.left() + 46.0, row.center().y), Align2::LEFT_CENTER, &text, medium(13.5), TEXT, size.x - 220.0);
        let right_center = pos2(row.right() - 14.0, row.center().y);
        if app.update.is_some() {
            if text_button(ui, right_center, Id::new("update-btn"), "Update Available").clicked() {
                out.push(Action::GetUpdate);
            }
        } else {
            label(ui, right_center, Align2::RIGHT_CENTER, "Up to date", sans(12.5), FAINT, 120.0);
        }
    });
    let text = "AURI is open source and updates directly from github.com/SuryakiranMK/AURI.";
    let at = pos2(card.left() + 4.0, card.bottom() + SECTION_GAP / 2.0);
    label(ui, at, Align2::LEFT_CENTER, text, sans(12.0), FAINT, card.width());
}

fn settings(ui: &mut Ui, app: &App, active_tab: SettingsTab, out: &mut Vec<Action>) {
    let full = ui.available_width();
    let width = full.min(SETTINGS_WIDTH);
    ui.add_space(8.0);

    // Title Row
    let (title, _) = ui.allocate_exact_size(vec2(full, 60.0), Sense::hover());
    let at = pos2(title.left() + (full - width) / 2.0, title.bottom() - 10.0);
    label(ui, at, Align2::LEFT_BOTTOM, "Settings", display(38.0), TEXT, width);

    // Sub-Page / Tab Switcher
    let (tabs_bar, _) = ui.allocate_exact_size(vec2(full, 44.0), Sense::hover());
    let tabs_left = tabs_bar.left() + (full - width) / 2.0;
    let tab_items: [(SettingsTab, &str, &str); 4] = [
        (SettingsTab::General, "General", icon::GEAR_SIX),
        (SettingsTab::Equalizer, "Equalizer", icon::SLIDERS),
        (SettingsTab::Performance, "Performance", icon::LIGHTNING),
        (SettingsTab::MiniMode, "Mini Mode", icon::PICTURE_IN_PICTURE),
    ];
    let tab_w = (width - 18.0) / 4.0;
    for (i, &(tab, name, sym)) in tab_items.iter().enumerate() {
        let is_cur = active_tab == tab;
        let tab_rect = Rect::from_min_size(
            pos2(tabs_left + i as f32 * (tab_w + 6.0), tabs_bar.top() + 4.0),
            vec2(tab_w, 34.0),
        );
        let resp = ui.interact(tab_rect, Id::new(("settings-tab-btn", i)), Sense::click());
        let hover = hover_of(ui, &resp);
        let fill = if is_cur { veil(0.12) } else { veil(0.04 * hover) };
        ui.painter().rect_filled(tab_rect, 8.0, fill);
        if is_cur {
            if app.saved.selection_glow {
                ui.painter().rect_stroke(tab_rect.expand(1.0), 9.0, Stroke::new(1.0_f32, ACCENT.gamma_multiply(0.35)), StrokeKind::Outside);
                ui.painter().rect_stroke(
                    tab_rect,
                    8.0,
                    Stroke::new(1.6_f32, ACCENT.lerp_to_gamma(Color32::WHITE, 0.40)),
                    StrokeKind::Inside,
                );
            } else {
                ui.painter().rect_stroke(
                    tab_rect,
                    8.0,
                    Stroke::new(1.2_f32, ACCENT),
                    StrokeKind::Inside,
                );
            }
        }
        let tone = if is_cur { TEXT } else { MUTED.lerp_to_gamma(TEXT, hover) };
        let icon_color = if is_cur { ACCENT } else { tone };
        let center = tab_rect.center();
        ui.painter().text(pos2(center.x - 36.0, center.y), Align2::CENTER_CENTER, sym, glyph(16.0), icon_color);
        ui.painter().text(pos2(center.x - 22.0, center.y), Align2::LEFT_CENTER, name, medium(13.0), tone);
        if resp.clicked() {
            out.push(Action::Go(Route::Settings(tab)));
        }
    }
    ui.add_space(20.0);

    // Tab Content Routing
    match active_tab {
        SettingsTab::General => {
            appearance_section(ui, app, out);
            player_bar_section(ui, app, out);
            storage_section(ui, app, out);
            discord_section(ui, app, out);
            hotkey_section(ui, app, out);
            about_section(ui, app, out);
        }
        SettingsTab::Equalizer => {
            audio_dsp_section(ui, app, out);
        }
        SettingsTab::Performance => {
            performance_settings_section(ui, app, out);
        }
        SettingsTab::MiniMode => {
            mini_mode_settings_section(ui, app, out);
        }
    }
    ui.add_space(32.0);
}

fn nav_item(ui: &mut Ui, symbol: &str, text: &str, active: bool, trailing: &str, glow: bool) -> Response {
    let (slot, response) = ui.allocate_exact_size(vec2(ui.available_width(), 42.0), Sense::click());
    let rect = slot.shrink2(vec2(12.0, 1.0));
    let hover = hover_of(ui, &response);
    ui.painter()
        .rect_filled(rect, ROW_RADIUS, veil(if active { 0.08 } else { 0.05 * hover }));
    if active && glow {
        ui.painter().rect_stroke(rect.expand(1.0), ROW_RADIUS + 1.0, Stroke::new(1.0_f32, ACCENT.gamma_multiply(0.35)), StrokeKind::Outside);
        ui.painter().rect_stroke(rect, ROW_RADIUS, Stroke::new(1.5_f32, ACCENT.lerp_to_gamma(Color32::WHITE, 0.40)), StrokeKind::Inside);
    }
    let tone = if active { TEXT } else { MUTED.lerp_to_gamma(TEXT, hover) };
    let icon_color = if active {
        if glow { ACCENT.lerp_to_gamma(Color32::WHITE, 0.35) } else { ACCENT }
    } else {
        tone
    };
    let font = if active { solid(19.0) } else { glyph(19.0) };
    ui.painter().text(
        pos2(rect.left() + 24.0, rect.center().y),
        Align2::CENTER_CENTER,
        symbol,
        font,
        icon_color,
    );
    label(
        ui,
        pos2(rect.left() + 48.0, rect.center().y),
        Align2::LEFT_CENTER,
        text,
        medium(14.0),
        tone,
        rect.width() - 96.0,
    );
    label(
        ui,
        pos2(rect.right() - 14.0, rect.center().y),
        Align2::RIGHT_CENTER,
        trailing,
        sans(12.5),
        FAINT,
        40.0,
    );
    describe(&response, text);
    response
}

#[allow(dead_code)]
fn mark(ui: &Ui, center: Pos2, radius: f32) {
    ui.painter().circle_filled(center, radius, ACCENT);
    ui.painter().circle_stroke(center, radius * 0.55, Stroke::new(radius * 0.09, PANEL));
    ui.painter().circle_filled(center, radius * 0.18, PANEL);
}

static BRAND_BANNER: LazyLock<eframe::egui::ColorImage> = LazyLock::new(|| {
    let bytes = include_bytes!("../assets/banner_left.jpg");
    let img = image::load_from_memory(bytes).expect("Failed to load brand banner").to_rgba8();
    let (w, h) = img.dimensions();
    let mut raw = img.into_raw();
    for chunk in raw.chunks_exact_mut(4) {
        let v = chunk[0].max(chunk[1]).max(chunk[2]);
        if v <= 16 {
            chunk[3] = 0;
        } else if v < 36 {
            let factor = (v - 16) as f32 / 20.0;
            chunk[3] = (chunk[3] as f32 * factor) as u8;
        }
    }
    eframe::egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &raw)
});

fn sidebar(app: &mut App, ui: &mut Ui, out: &mut Vec<Action>) {
    let area = ui.max_rect();
    ui.painter().vline(area.right() - 0.5, area.y_range(), Stroke::new(1.0_f32, veil(0.06)));
    let (brand, response) = ui.allocate_exact_size(vec2(area.width(), 84.0), Sense::click());
    if response.clicked() {
        out.push(Action::Go(Route::Home));
    }
    let texture = ui.ctx().load_texture("auri-brand", BRAND_BANNER.clone(), eframe::egui::TextureOptions::LINEAR);
    let banner_rect = Rect::from_min_size(pos2(brand.left() + 2.0, brand.center().y - 40.0), vec2(240.0, 80.0));
    ui.painter().image(texture.id(), banner_rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);

    let col_btn_rect = Rect::from_center_size(pos2(brand.right() - 22.0, brand.center().y), vec2(26.0, 26.0));
    let col_resp = ui.interact(col_btn_rect, Id::new("sidebar-collapse-btn"), Sense::click());
    let col_hover = hover_of(ui, &col_resp);
    ui.painter().rect_filled(col_btn_rect, 6.0, veil(0.04 + 0.06 * col_hover));
    ui.painter().text(col_btn_rect.center(), Align2::CENTER_CENTER, icon::CARET_LEFT, solid(16.0), if col_hover > 0.1 { TEXT } else { MUTED });
    describe(&col_resp, "Collapse sidebar");
    if col_resp.clicked() {
        out.push(Action::ToggleSidebar);
    }

    let count = |items: &[Item]| if items.is_empty() { String::new() } else { items.len().to_string() };
    let links = [
        (icon::HOUSE, "Home", Route::Home, String::new()),
        (icon::HEART, "Liked songs", Route::Liked, count(&app.saved.liked)),
        (
            icon::CLOCK_COUNTER_CLOCKWISE,
            "Recently played",
            Route::Recent,
            count(&app.saved.recent),
        ),
    ];
    for (symbol, text, route, trailing) in links {
        if nav_item(ui, symbol, text, *app.route() == route, &trailing, app.saved.selection_glow).clicked() {
            out.push(Action::Go(route));
        }
    }
    ui.add_space(18.0);
    let (header_row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());
    label(ui, pos2(header_row.left() + 26.0, header_row.center().y), Align2::LEFT_CENTER, "LIBRARY", bold(11.0), FAINT, 120.0);
    let add_btn_rect = Rect::from_center_size(pos2(header_row.right() - 24.0, header_row.center().y), vec2(22.0, 22.0));
    let add_resp = ui.interact(add_btn_rect, Id::new("add-playlist-btn"), Sense::click());
    let add_hover = hover_of(ui, &add_resp);
    ui.painter().text(add_btn_rect.center(), Align2::CENTER_CENTER, icon::PLUS, glyph(14.0), if add_hover > 0.1 { TEXT } else { ACCENT });
    describe(&add_resp, "New playlist");
    if add_resp.clicked() {
        out.push(Action::PromptCreatePlaylist);
    }

    if app.creating_playlist {
        ui.add_space(4.0);
        let mut create_name = None;
        let mut cancel = false;
        Frame::new()
            .inner_margin(Margin::symmetric(14, 6))
            .fill(RAISED)
            .corner_radius(6.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let re = ui.add(
                        egui::TextEdit::singleline(&mut app.new_playlist_name)
                            .hint_text("Playlist name...")
                            .desired_width(120.0),
                    );
                    if re.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                        create_name = Some(app.new_playlist_name.trim().to_string());
                    }
                    if ui.button(RichText::new("✓").color(ACCENT)).clicked() {
                        create_name = Some(app.new_playlist_name.trim().to_string());
                    }
                    if ui.button(RichText::new("✕").color(MUTED)).clicked() {
                        cancel = true;
                    }
                });
            });
        if let Some(name) = create_name {
            if !name.is_empty() {
                out.push(Action::CreatePlaylist(name));
            } else {
                out.push(Action::CancelCreatePlaylist);
            }
        } else if cancel {
            out.push(Action::CancelCreatePlaylist);
        }
        ui.add_space(4.0);
    }

    let foot = Rect::from_min_max(pos2(area.left(), area.bottom() - ACCOUNT_HEIGHT), area.max);
    account(app, ui, foot, out);

    let dl_box_h = 46.0;
    let dl_lift = if app.download_active.is_some() { dl_box_h + 8.0 } else { 0.0 };
    if let Some(active) = &app.download_active {
        let dl_rect = Rect::from_min_size(
            pos2(area.left() + 10.0, foot.top() - dl_box_h - 4.0),
            vec2(area.width() - 20.0, dl_box_h),
        );
        let pal_color = app.saved.theme_palette.color();
        ui.painter().rect_filled(dl_rect, 8.0, veil(0.06));
        ui.painter().rect_stroke(dl_rect, 8.0, Stroke::new(1.0_f32, pal_color.gamma_multiply(0.35)), StrokeKind::Inside);

        let icon_pos = pos2(dl_rect.left() + 18.0, dl_rect.top() + 16.0);
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(30));
        ui.painter().text(icon_pos, Align2::CENTER_CENTER, icon::ARROW_DOWN, solid(14.0), pal_color);

        let pct_str = format!("{:.0}%", active.percent * 100.0);
        label(ui, pos2(dl_rect.left() + 34.0, dl_rect.top() + 16.0), Align2::LEFT_CENTER, &active.title, medium(12.0), TEXT, dl_rect.width() - 86.0);
        label(ui, pos2(dl_rect.right() - 10.0, dl_rect.top() + 16.0), Align2::RIGHT_CENTER, &pct_str, sans(11.5), pal_color, 45.0);

        let p_track = Rect::from_min_size(pos2(dl_rect.left() + 12.0, dl_rect.bottom() - 10.0), vec2(dl_rect.width() - 24.0, 3.5));
        ui.painter().rect_filled(p_track, 1.75, veil(0.12));
        let fill_w = p_track.width() * active.percent.clamp(0.0, 1.0);
        if fill_w > 0.0 {
            ui.painter().rect_filled(Rect::from_min_size(p_track.min, vec2(fill_w, 3.5)), 1.75, pal_color);
            let knob_x = p_track.left() + fill_w;
            ui.painter().circle_filled(pos2(knob_x, p_track.center().y), 3.5, pal_color);
            ui.painter().circle_filled(pos2(knob_x, p_track.center().y), 1.5, Color32::WHITE);
        }
    }

    let lift = dl_lift + if app.update.is_some() { UPDATE_HEIGHT } else { 0.0 };
    let offer = Rect::from_min_max(pos2(area.left(), foot.top() - lift), foot.right_top());
    update_offer(app, ui, offer, out);

    let height = (offer.top() - ui.cursor().top()).max(0.0);
    egui::ScrollArea::vertical().max_height(height).auto_shrink(false).show(ui, |ui| {
        // Render custom playlists first
        for pl in &app.saved.playlists {
            let (slot, response) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_HEIGHT), Sense::click());
            let rect = slot.shrink2(vec2(12.0, 1.0));
            let active = matches!(app.route(), Route::CustomPlaylist(id) if id == &pl.id);
            let hover = hover_of(ui, &response);
            ui.painter()
                .rect_filled(rect, ROW_RADIUS, veil(if active { 0.08 } else { 0.05 * hover }));
            if active && app.saved.selection_glow {
                ui.painter().rect_stroke(rect.expand(1.0), ROW_RADIUS + 1.0, Stroke::new(1.0_f32, ACCENT.gamma_multiply(0.35)), StrokeKind::Outside);
                ui.painter().rect_stroke(rect, ROW_RADIUS, Stroke::new(1.4_f32, ACCENT.lerp_to_gamma(Color32::WHITE, 0.40)), StrokeKind::Inside);
            }
            let icon_rect = Rect::from_min_size(pos2(rect.left() + 8.0, rect.center().y - 20.0), Vec2::splat(40.0));
            ui.painter().rect_filled(icon_rect, 6.0, ACCENT.gamma_multiply(0.14));
            ui.painter().text(icon_rect.center(), Align2::CENTER_CENTER, icon::PLAYLIST, glyph(20.0), ACCENT);
            let (left, width) = (icon_rect.right() + 12.0, rect.right() - icon_rect.right() - 22.0);
            label(
                ui,
                pos2(left, rect.center().y - 9.0),
                Align2::LEFT_CENTER,
                &pl.name,
                medium(13.5),
                TEXT,
                width,
            );
            let song_count_str = if pl.songs.len() == 1 { "1 song".to_string() } else { format!("{} songs", pl.songs.len()) };
            label(
                ui,
                pos2(left, rect.center().y + 9.0),
                Align2::LEFT_CENTER,
                &song_count_str,
                sans(12.5),
                MUTED,
                width,
            );
            describe(&response, &pl.name);
            if response.clicked() {
                out.push(Action::Go(Route::CustomPlaylist(pl.id.clone())));
            }
        }

        // Pinned and library items
        let pinned = &app.saved.pinned;
        let own = app
            .library
            .iter()
            .filter(|item| !pinned.iter().any(|saved| saved.browse_id == item.browse_id));
        let entries: Vec<&Item> = pinned.iter().chain(own).collect();
        for item in entries {
            let (slot, response) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_HEIGHT), Sense::click());
            let rect = slot.shrink2(vec2(12.0, 1.0));
            let active = matches!(app.route(), Route::Browse(id) if *id == item.browse_id);
            let hover = hover_of(ui, &response);
            ui.painter()
                .rect_filled(rect, ROW_RADIUS, veil(if active { 0.08 } else { 0.05 * hover }));
            if active && app.saved.selection_glow {
                ui.painter().rect_stroke(rect.expand(1.0), ROW_RADIUS + 1.0, Stroke::new(1.0_f32, ACCENT.gamma_multiply(0.35)), StrokeKind::Outside);
                ui.painter().rect_stroke(rect, ROW_RADIUS, Stroke::new(1.4_f32, ACCENT.lerp_to_gamma(Color32::WHITE, 0.40)), StrokeKind::Inside);
            }
            let art = Rect::from_min_size(pos2(rect.left() + 8.0, rect.center().y - 20.0), Vec2::splat(40.0));
            artwork(ui, &app.art, &item.thumb, art, if item.is_artist { 20.0 } else { 5.0 });
            let (left, width) = (art.right() + 12.0, rect.right() - art.right() - 22.0);
            let kind = if item.is_artist { "Artist" } else { item.subtitle.as_str() };
            label(
                ui,
                pos2(left, rect.center().y - 9.0),
                Align2::LEFT_CENTER,
                &item.title,
                medium(13.5),
                TEXT,
                width,
            );
            label(
                ui,
                pos2(left, rect.center().y + 9.0),
                Align2::LEFT_CENTER,
                kind,
                sans(12.5),
                MUTED,
                width,
            );
            describe(&response, &item.title);
            if response.clicked() {
                out.push(Action::Go(Route::Browse(item.browse_id.clone())));
            }
        }
        ui.add_space(12.0);
    });
}

fn update_offer(app: &App, ui: &Ui, slot: Rect, out: &mut Vec<Action>) {
    let Some(version) = &app.update else { return };
    let row = slot.shrink2(vec2(12.0, 4.0));
    let response = ui.interact(row, Id::new("update-offer"), Sense::click());
    let hover = hover_of(ui, &response);
    ui.painter()
        .rect_filled(row, ROW_RADIUS, ACCENT.gamma_multiply(0.14 + 0.08 * hover));
    let mark = pos2(row.left() + 24.0, row.center().y);
    if app.updating {
        egui::Spinner::new()
            .color(ACCENT)
            .paint_at(ui, Rect::from_center_size(mark, Vec2::splat(16.0)));
    } else {
        ui.painter()
            .text(mark, Align2::CENTER_CENTER, icon::ARROW_CIRCLE_UP, solid(19.0), ACCENT);
    }
    let text = if app.updating {
        "Updating".to_owned()
    } else {
        format!("Update to {version}")
    };
    label(
        ui,
        pos2(row.left() + 48.0, row.center().y),
        Align2::LEFT_CENTER,
        &text,
        medium(13.5),
        TEXT,
        row.width() - 60.0,
    );
    describe(&response, &text);
    if response.clicked() {
        out.push(Action::GetUpdate);
    }
}

fn account(app: &App, ui: &Ui, foot: Rect, out: &mut Vec<Action>) {
    ui.painter().hline(foot.x_range(), foot.top(), Stroke::new(1.0_f32, veil(0.06)));
    let mid_y = foot.center().y;
    let btn_size = Vec2::splat(34.0);

    let step = foot.width() / 4.0;
    let b1_x = foot.left() + step * 0.5;
    let b2_x = foot.left() + step * 1.5;
    let b3_x = foot.left() + step * 2.5;
    let b4_x = foot.left() + step * 3.5;

    // 1. Account Button
    let acc_slot = Rect::from_center_size(pos2(b1_x, mid_y), btn_size);
    match &app.saved.account {
        Some(account) => {
            let avatar = Rect::from_center_size(acc_slot.center(), Vec2::splat(28.0));
            artwork(ui, &app.art, &account.photo, avatar, 14.0);
            let resp = ui.interact(acc_slot, Id::new("account-menu-btn"), Sense::click());
            describe(&resp, &format!("Account: {}", account.name));
            egui::Popup::menu(&resp).gap(4.0).align(egui::RectAlign::TOP_START).show(|ui| {
                ui.set_width(MENU_WIDTH);
                ui.label(RichText::new(&account.name).strong());
                if app.accounts.len() > 1 {
                    for channel in &app.accounts {
                        let chosen = channel.page == account.page;
                        let symbol = if chosen { icon::CHECK } else { "" };
                        let row = menu_row(ui, symbol, glyph(15.0), ACCENT, &channel.name);
                        if row.clicked() && !chosen {
                            out.push(Action::SwitchAccount(channel.clone()));
                        }
                    }
                }
                if menu_row(ui, icon::SIGN_OUT, glyph(15.0), MUTED, "Sign out").clicked() {
                    out.push(Action::SignOut);
                }
            });
        }
        None => {
            if icon_button(ui, acc_slot, Id::new("sign-in"), icon::USER_CIRCLE, glyph(19.0), MUTED, "Sign in").clicked() {
                out.push(Action::OpenSignIn);
            }
        }
    }

    // 2. Equalizer Button
    let is_eq = matches!(app.route(), Route::Settings(SettingsTab::Equalizer));
    let eq_slot = Rect::from_center_size(pos2(b2_x, mid_y), btn_size);
    let eq_tone = if is_eq { ACCENT } else { MUTED };
    if icon_button(ui, eq_slot, Id::new("sound-eq-btn"), icon::SLIDERS, glyph(18.0), eq_tone, "Equalizer").clicked() {
        out.push(Action::Go(Route::Settings(SettingsTab::Equalizer)));
    }

    // 3. Update Check Button (between Equalizer and Settings)
    let upd_slot = Rect::from_center_size(pos2(b3_x, mid_y), btn_size);
    let has_upd = app.update.is_some();
    let upd_icon = if has_upd { icon::ARROW_CIRCLE_UP } else { icon::ARROW_CLOCKWISE };
    let upd_tone = if has_upd { ACCENT } else { MUTED };
    let upd_hint = if has_upd { "Update Available (Click to Install)" } else { "Check for Updates" };
    if icon_button(ui, upd_slot, Id::new("dock-update-btn"), upd_icon, if has_upd { solid(18.0) } else { glyph(18.0) }, upd_tone, upd_hint).clicked() {
        if has_upd {
            out.push(Action::GetUpdate);
        } else {
            out.push(Action::CheckUpdate);
        }
    }

    // 4. Settings Button
    let is_settings = matches!(app.route(), Route::Settings(tab) if *tab != SettingsTab::Equalizer);
    let set_slot = Rect::from_center_size(pos2(b4_x, mid_y), btn_size);
    let set_tone = if is_settings { ACCENT } else { MUTED };
    if icon_button(ui, set_slot, Id::new("settings-btn"), icon::GEAR_SIX, glyph(18.0), set_tone, "Settings").clicked() {
        out.push(Action::Go(Route::Settings(SettingsTab::General)));
    }
}

fn sign_in_dialog(app: &mut App, ctx: &egui::Context, out: &mut Vec<Action>) {
    let Some(sign_in) = &mut app.sign_in else { return };
    let frame = Frame::new()
        .fill(rgb(0x15161A))
        .stroke(Stroke::new(1.0_f32, veil(0.08)))
        .corner_radius(18)
        .inner_margin(Margin::same(30));
    let modal = egui::Modal::new(Id::new("sign-in-dialog"))
        .frame(frame)
        .backdrop_color(shade(0.6))
        .show(ctx, |ui| {
            ui.set_width(430.0);
            let width = ui.available_width();
            let (head, _) = ui.allocate_exact_size(vec2(width, 44.0), Sense::hover());
            ui.painter()
                .text(head.left_center(), Align2::LEFT_CENTER, "Sign in", display(38.0), TEXT);
            let slot = Rect::from_center_size(pos2(head.right() - 16.0, head.center().y), Vec2::splat(34.0));
            let closed = icon_button(ui, slot, Id::new("sign-in-close"), icon::X, glyph(16.0), MUTED, "Close").clicked();
            ui.add_space(8.0);
            paragraph(ui, "Your own home feed and playlists from YouTube Music.", sans(14.0), MUTED, 2);
            ui.add_space(24.0);

            let (row, _) = ui.allocate_exact_size(vec2(width, 42.0), Sense::hover());
            if pill(ui, row.min, Id::new("sign-in-browser"), icon::GLOBE, "Continue in browser", true).clicked() && !sign_in.busy {
                out.push(Action::BrowserSignIn);
            }
            ui.add_space(12.0);
            paragraph(
                ui,
                "Opens a separate Edge or Chrome window. Sign in to Google there and it closes by itself.",
                sans(13.0),
                MUTED,
                3,
            );
            ui.add_space(22.0);

            eyebrow(ui, "Or paste your cookie", 0.0);
            let steps = [
                "1.  Open music.youtube.com in your browser and sign in.",
                "2.  Press F12, open Network and click a request to music.youtube.com.",
                "3.  Copy the whole cookie value under Request Headers and paste it here.",
            ];
            for step in steps {
                paragraph(ui, step, sans(13.0), MUTED, 2);
                ui.add_space(5.0);
            }
            ui.add_space(7.0);
            let background = ui.painter().add(Shape::Noop);
            let field = egui::ScrollArea::vertical().max_height(74.0).auto_shrink(false).show(ui, |ui| {
                let edit = egui::TextEdit::multiline(&mut sign_in.cookie)
                    .frame(false)
                    .margin(Margin::same(10))
                    .desired_width(f32::INFINITY)
                    .desired_rows(3)
                    .font(FontId::monospace(11.5))
                    .text_color(TEXT)
                    .hint_text(RichText::new("SID=...; SAPISID=...; LOGIN_INFO=...").color(FAINT));
                ui.add(edit);
            });
            ui.painter().set(
                background,
                RectShape::new(field.inner_rect, 10.0, veil(0.06), Stroke::new(1.0_f32, veil(0.08)), StrokeKind::Inside),
            );
            ui.add_space(14.0);
            let (row, _) = ui.allocate_exact_size(vec2(width, 42.0), Sense::hover());
            if pill(
                ui,
                row.min,
                Id::new("sign-in-cookie"),
                icon::CLIPBOARD_TEXT,
                "Use this cookie",
                false,
            )
            .clicked()
                && !sign_in.busy
            {
                out.push(Action::CookieSignIn);
            }

            if !sign_in.status.is_empty() {
                ui.add_space(18.0);
                let indent = if sign_in.busy { 26.0 } else { 0.0 };
                let status = fit(ui, &sign_in.status, medium(13.0), TEXT, width - indent, 3);
                let (row, _) = ui.allocate_exact_size(vec2(width, status.size().y.max(18.0)), Sense::hover());
                if sign_in.busy {
                    egui::Spinner::new()
                        .color(ACCENT)
                        .paint_at(ui, Rect::from_min_size(row.min + vec2(0.0, 1.0), Vec2::splat(16.0)));
                }
                ui.painter().galley(row.min + vec2(indent, 0.0), status, TEXT);
            }
            ui.add_space(20.0);
            paragraph(ui, crate::auth::SESSION_KEPT, sans(12.0), FAINT, 2);
            closed
        });
    if modal.inner || modal.should_close() {
        out.push(Action::CloseSignIn);
    }
}

fn queue_panel(app: &App, ui: &mut Ui, out: &mut Vec<Action>) {
    let area = ui.max_rect();
    let cover = app.now_playing().map(|item| app.saved.covers.of(item));
    let tint = cover
        .and_then(|cover| app.art.get(ui.ctx(), cover, BAR_ART))
        .map(|picture| picture.tint);
    let glow = mood(ui, "queue-glow", tint, PANEL, 0.3);
    let haze = Rect::from_min_size(area.min, vec2(area.width(), AMBIENT_HEIGHT.min(area.height())));
    wash(ui.painter(), haze, glow, PANEL);
    ui.painter().vline(area.left() + 0.5, area.y_range(), Stroke::new(1.0_f32, veil(0.06)));
    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        Frame::new().inner_margin(Margin::same(20)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            let (q_head, _) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());
            label(ui, pos2(q_head.left(), q_head.center().y), Align2::LEFT_CENTER, "NOW PLAYING", bold(11.0), FAINT, 120.0);
            let col_resp = icon_button(ui, Rect::from_center_size(pos2(q_head.right() - 8.0, q_head.center().y), vec2(20.0, 20.0)), Id::new("queue-collapse-btn"), icon::CARET_RIGHT, solid(15.0), MUTED, "Collapse queue");
            if col_resp.clicked() {
                out.push(Action::ToggleQueue);
            }
            ui.add_space(4.0);

            let Some(item) = app.now_playing() else {
                message(
                    ui,
                    icon::VINYL_RECORD,
                    "Nothing playing",
                    "Pick a song and the queue lines up here.",
                );
                return;
            };
            let (head, cover) = ui.allocate_exact_size(vec2(ui.available_width(), QUEUE_ART), Sense::click());
            let art = Rect::from_min_size(head.min, Vec2::splat(QUEUE_ART));
            artwork(ui, &app.art, app.saved.covers.of(item), art, ART_RADIUS);
            let lift = hover_of(ui, &cover);
            ui.painter().rect_filled(art, ART_RADIUS, shade(0.4 * lift));
            ui.painter().text(
                art.center(),
                Align2::CENTER_CENTER,
                icon::ARROWS_OUT_SIMPLE,
                glyph(22.0),
                TEXT.gamma_multiply(lift),
            );
            describe(&cover, "Now playing");
            if cover.clicked() {
                out.push(Action::Expand(true));
            }
            let left = art.right() + 14.0;
            let title = fit(ui, &item.title, bold(15.5), TEXT, head.right() - left, 2);
            let top = head.center().y - (title.size().y + 22.0) / 2.0;
            let below = pos2(left, top + title.size().y + 12.0);
            ui.painter().galley(pos2(left, top), title, TEXT);
            byline(ui, out, item, Id::new("queue-artist"), below, sans(13.0), head.right() - left);
            ui.add_space(26.0);
            queue_rows(app, ui, true, out);
        });
    });
}

fn tab(ui: &Ui, at: Pos2, text: &str, active: bool) -> Response {
    let galley = fit(ui, text, medium(13.5), TEXT, 160.0, 1);
    let rect = Rect::from_min_size(at, vec2(galley.size().x + 28.0, 32.0));
    let response = ui.interact(rect, Id::new(("tab", text)), Sense::click());
    let hover = hover_of(ui, &response);
    ui.painter().rect_filled(rect, 16.0, veil(if active { 0.12 } else { 0.06 * hover }));
    let ink = if active { TEXT } else { MUTED.lerp_to_gamma(TEXT, hover) };
    ui.painter()
        .galley_with_override_text_color(rect.center() - galley.size() / 2.0, galley, ink);
    describe(&response, text);
    response
}

fn lyrics_view(app: &App, ui: &mut Ui, out: &mut Vec<Action>) {
    let lyrics = match app.lyrics.as_ref().map(|(_, words)| words) {
        Some(Words::Ready(lyrics)) => lyrics,
        Some(Words::Missing) => {
            message(
                ui,
                icon::SUBTITLES,
                "No lyrics for this song",
                "YouTube Music and lrclib do not have them yet.",
            );
            return;
        }
        Some(Words::Failed) => {
            message(ui, icon::WARNING_CIRCLE, "Lyrics did not load", "Open this tab again to retry.");
            return;
        }
        Some(Words::Loading) | None => {
            let pulse = veil(0.05 + 0.025 * (ui.input(|input| input.time) * 3.2).sin() as f32);
            ui.ctx().request_repaint_after(Duration::from_millis(60));
            for share in [0.7, 0.9, 0.55, 0.8, 0.65, 0.4] {
                let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::hover());
                let bar = Rect::from_min_size(row.min + vec2(0.0, 10.0), vec2(row.width() * share, 20.0));
                ui.painter().rect_filled(bar, 6.0, pulse);
            }
            return;
        }
    };
    let position = app.player.position_ms();
    let sung = lyrics
        .synced
        .then(|| lyrics.lines.iter().rposition(|(start, _)| *start <= position))
        .flatten();
    let follow = Id::new("lyrics-follow");
    let moved = ui.data(|data| data.get_temp::<Option<usize>>(follow)) != Some(sung);
    ui.data_mut(|data| data.insert_temp(follow, sung));
    let width = ui.available_width();
    let sense = if lyrics.synced { Sense::click() } else { Sense::hover() };
    for (index, (start, line)) in lyrics.lines.iter().enumerate() {
        if line.is_empty() {
            ui.add_space(18.0);
            continue;
        }
        let galley = fit(ui, line, bold(20.0), TEXT, width, 4);
        let (rect, response) = ui.allocate_exact_size(vec2(width, galley.size().y + 14.0), sense);
        let current = sung == Some(index);
        if current && moved {
            ui.scroll_to_rect(rect, Some(Align::Center));
        }
        if !ui.is_rect_visible(rect) {
            continue;
        }
        let lit = ui
            .ctx()
            .animate_bool_with_time(response.id.with("lit"), current || !lyrics.synced, 0.25);
        let hover = hover_of(ui, &response);
        let ink = TEXT.gamma_multiply(0.34 + 0.66 * lit.max(0.5 * hover));
        ui.painter().galley_with_override_text_color(rect.min + vec2(0.0, 7.0), galley, ink);
        if lyrics.synced {
            point_at(&response);
        }
        if response.clicked() {
            out.push(Action::Seek(*start));
        }
    }
    ui.add_space(12.0);
    paragraph(ui, &lyrics.source, sans(12.0), FAINT, 2);
    let next = sung.map_or(0, |index| index + 1);
    if let (true, Some((start, _))) = (lyrics.synced && app.playing(), lyrics.lines.get(next)) {
        let wait = start.saturating_sub(position).max(30);
        ui.ctx().request_repaint_after(Duration::from_millis(wait));
    }
}

fn queue_row(ui: &Ui, app: &App, item: &Item, slot: Rect, hover: f32) -> (Pos2, f32) {
    ui.painter()
        .rect_filled(slot.expand2(vec2(8.0, 0.0)), ROW_RADIUS, veil(0.06 * hover));
    let art = Rect::from_min_size(pos2(slot.left(), slot.center().y - 20.0), Vec2::splat(40.0));
    artwork(ui, &app.art, app.saved.covers.of(item), art, 5.0);
    let (left, width) = (art.right() + 12.0, slot.right() - art.right() - 44.0);
    label(
        ui,
        pos2(left, slot.center().y - 9.0),
        Align2::LEFT_CENTER,
        &item.title,
        medium(13.5),
        TEXT,
        width,
    );
    (pos2(left, slot.center().y + 9.0), width)
}

fn queue_rows(app: &App, ui: &mut Ui, heading: bool, out: &mut Vec<Action>) -> bool {
    let queue = &app.saved.queue;
    let upcoming = app.saved.current.map_or(0, |current| current + 1);
    if upcoming >= queue.len() {
        return false;
    }
    if heading {
        eyebrow(ui, "Next up", 0.0);
    }
    let top = ui.cursor().top();
    let pointer = ui.ctx().pointer_interact_pos();
    let grab = Id::new("queue-grab");
    let rows: Vec<(Rect, Response)> = (upcoming..queue.len())
        .map(|_| ui.allocate_exact_size(vec2(ui.available_width(), QUEUE_ROW), Sense::click_and_drag()))
        .collect();
    let carried = rows.iter().position(|(_, row)| row.dragged() || row.drag_stopped());
    if let (Some((slot, _)), Some(pointer)) = (rows.iter().find(|(_, row)| row.drag_started()), pointer) {
        ui.data_mut(|data| data.insert_temp(grab, pointer.y - slot.top()));
    }
    let held = ui.data(|data| data.get_temp::<f32>(grab)).unwrap_or(QUEUE_ROW / 2.0);
    let moving = carried.zip(pointer).map(|(from, pointer)| {
        let lifted = pointer.y - held;
        let over = ((lifted + QUEUE_ROW / 2.0 - top) / QUEUE_ROW).floor().max(0.0) as usize;
        (from, over.min(rows.len() - 1), lifted)
    });
    let glide = if moving.is_some() { 0.12 } else { 0.0 };
    for (at, (slot, response)) in rows.iter().enumerate() {
        let (index, item) = (upcoming + at, &queue[upcoming + at]);
        let aside = match moving {
            Some((from, to, _)) if from < at && at <= to => -QUEUE_ROW,
            Some((from, to, _)) if to <= at && at < from => QUEUE_ROW,
            _ => 0.0,
        };
        let aside = ui.ctx().animate_value_with_time(response.id.with("aside"), aside, glide);
        let slot = slot.translate(vec2(0.0, aside));
        if moving.is_some_and(|(from, ..)| from == at) || !ui.is_rect_visible(slot) {
            continue;
        }
        let hover = hover_of(ui, response);
        let (below, width) = queue_row(ui, app, item, slot, hover);
        byline(ui, out, item, response.id.with("artist"), below, sans(12.5), width);
        describe(response, &item.title);
        if hover > 0.0 {
            let close = Rect::from_center_size(pos2(slot.right() - 12.0, slot.center().y), Vec2::splat(28.0));
            let tone = MUTED.gamma_multiply(hover);
            if icon_button(
                ui,
                close,
                response.id.with("remove"),
                icon::X,
                glyph(13.0),
                tone,
                "Remove from queue",
            )
            .clicked()
            {
                out.push(Action::Remove(index));
            }
        }
        if response.clicked() {
            out.push(Action::Jump(index));
        }
    }

    let Some((from, to, lifted)) = moving else { return true };
    let (slot, response) = &rows[from];
    if response.drag_stopped() {
        let gap = if to > from { to + 1 } else { to };
        out.push(Action::Move(upcoming + from, upcoming + gap));
        return true;
    }
    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    let lifted = slot.translate(vec2(0.0, lifted - slot.top()));
    let plate = lifted.expand2(vec2(8.0, 0.0));
    let layer = egui::LayerId::new(egui::Order::Tooltip, grab);
    let ghost = ui.new_child(UiBuilder::new().layer_id(layer).max_rect(lifted));
    let shadow = Shadow {
        offset: [0, 8],
        blur: 24,
        spread: 0,
        color: shade(0.5),
    };
    ghost.painter().add(shadow.as_shape(plate, ROW_RADIUS));
    ghost.painter().rect_filled(plate, ROW_RADIUS, FLOATING);
    let item = &queue[upcoming + from];
    let (below, width) = queue_row(&ghost, app, item, lifted, 0.0);
    label(&ghost, below, Align2::LEFT_CENTER, &item.subtitle, sans(12.5), MUTED, width);
    true
}

fn now_playing(app: &App, ui: &mut Ui, out: &mut Vec<Action>) -> Color32 {
    let area = ui.max_rect();
    let Some(item) = app.now_playing() else { return PANEL };
    let shown = entrance(ui, Id::new("now-playing"));
    ui.set_opacity(shown);
    let cover = app.saved.covers.of(item);
    let column = (area.width() * 0.32).clamp(320.0, 420.0);
    let stage = Rect::from_min_max(area.min, pos2(area.right() - column - 24.0, area.bottom()));
    let side = (stage.width() - 96.0).min(stage.height() - 230.0).clamp(120.0, STAGE_ART);
    let tint = |size: f32| app.art.get(ui.ctx(), cover, size).map(|picture| picture.tint);
    let glow = mood(ui, "stage", tint(side).or_else(|| tint(BAR_ART)), PANEL, 0.46);
    let mut glow = dimmed(glow, STAGE_LUMINANCE);
    match app.art.get(ui.ctx(), cover, 1.0) {
        Some(blur) => {
            let fill = Rect::from_center_size(area.center(), Vec2::splat(area.width().max(area.height())));
            let dark = Color32::from_gray(STAGE_BLUR);
            egui::Image::from_texture((blur.id, fill.size())).tint(dark).paint_at(ui, fill);
            wash(ui.painter(), area, Color32::TRANSPARENT, PANEL);
            let scale = |channel: u8| (channel as u16 * STAGE_BLUR as u16 / 255) as u8;
            glow = Color32::from_rgb(scale(blur.tint.r()), scale(blur.tint.g()), scale(blur.tint.b()));
        }
        None => wash(ui.painter(), area, glow, PANEL),
    }

    let close = Rect::from_center_size(area.min + vec2(44.0, 42.0), Vec2::splat(38.0));
    if icon_button(ui, close, Id::new("stage-close"), icon::CARET_DOWN, glyph(19.0), MUTED, "Collapse").clicked() {
        out.push(Action::Expand(false));
    }

    let centered = |text: &str, font: FontId, color: Color32, rows: usize| {
        let mut job = LayoutJob::simple_singleline(text.to_owned(), font, color);
        job.halign = Align::Center;
        job.wrap = TextWrapping {
            max_width: (stage.width() - 64.0).min(side.max(440.0)),
            max_rows: rows,
            break_anywhere: false,
            overflow_character: Some('…'),
        };
        ui.painter().layout_job(job)
    };
    let size = match item.title.chars().count() {
        0..=18 => 54.0,
        19..=40 => 42.0,
        _ => 34.0,
    };
    let title = centered(&item.title, display(size), TEXT, 2);
    let artist = centered(&item.subtitle, sans(16.0), MUTED, 1);
    let block = side + 30.0 + title.size().y + 8.0 + artist.size().y;
    let top = (stage.center().y - block / 2.0).max(stage.top() + 56.0);
    let art = Rect::from_min_size(pos2(stage.center().x - side / 2.0, top), Vec2::splat(side));
    let shadow = Shadow {
        offset: [0, 22],
        blur: 56,
        spread: 0,
        color: shade(0.45),
    };
    ui.painter().add(shadow.as_shape(art, 18.0));
    artwork(ui, &app.art, cover, art, 18.0);
    let below = art.bottom() + 30.0;
    let lower = below + title.size().y + 8.0;
    ui.painter().galley(pos2(stage.center().x, below), title, TEXT);
    let origin = pos2(stage.center().x, lower);
    ui.painter().galley(origin, artist.clone(), MUTED);
    artist_links(ui, out, item, Id::new("stage-artist"), origin, &artist);

    let card = Rect::from_min_max(
        pos2(stage.right(), area.top() + 24.0),
        pos2(area.right() - 24.0, area.bottom() - 12.0),
    );
    ui.painter().rect_filled(card, 18.0, shade(0.24));
    let lyrics = app.saved.lyrics_open;
    let mut at = card.min + vec2(14.0, 14.0);
    for (text, shows_lyrics) in [("Next up", false), ("Lyrics", true)] {
        let tab = tab(ui, at, text, lyrics == shows_lyrics);
        if tab.clicked() {
            out.push(Action::ShowLyrics(shows_lyrics));
        }
        at.x = tab.rect.right() + 6.0;
    }
    let body = Rect::from_min_max(pos2(card.left(), card.top() + 56.0), card.max);
    let margin = Margin {
        left: 20,
        right: 20,
        top: 6,
        bottom: 14,
    };
    let mut list = ui.new_child(UiBuilder::new().id_salt("stage-card").max_rect(body));
    egui::ScrollArea::vertical()
        .id_salt(lyrics)
        .auto_shrink(false)
        .show(&mut list, |ui| {
            Frame::new().inner_margin(margin).show(ui, |ui| {
                ui.set_width(ui.available_width());
                if lyrics {
                    lyrics_view(app, ui, out);
                } else if !queue_rows(app, ui, false, out) {
                    message(ui, icon::QUEUE, "Nothing queued", "Songs you add line up here.");
                }
            });
        });
    PANEL.lerp_to_gamma(glow, shown)
}

fn player_bar(app: &App, ui: &mut Ui, out: &mut Vec<Action>) -> Scrub {
    let bar = ui.max_rect();
    let (deck, rounding) = match app.saved.bar_layout {
        BarLayout::Docked => (bar, 0.0),
        BarLayout::FloatingPill => {
            let pill_w = (bar.width() - 32.0).clamp(600.0, 1300.0).min(bar.width() - 16.0);
            let pill_h = (bar.height() - 12.0).max(64.0);
            let pill_rect = Rect::from_center_size(pos2(bar.center().x, bar.center().y - 2.0), vec2(pill_w, pill_h));
            (pill_rect, 26.0)
        }
        BarLayout::MiniCapsule => {
            let pill_w = 460.0_f32.min(bar.width() - 24.0);
            let pill_h = 56.0_f32;
            let pill_rect = Rect::from_center_size(pos2(bar.center().x, bar.center().y - 2.0), vec2(pill_w, pill_h));
            (pill_rect, 28.0)
        }
    };
    let mid = deck.center().y + 2.0;
    let duration = app.player.duration_ms();
    let item = app.now_playing();
    let cover = item.map(|item| app.saved.covers.of(item));
    let tint = cover
        .and_then(|cover| app.art.get(ui.ctx(), cover, BAR_ART))
        .map(|picture| picture.tint);

    // Deck Background: Glassmorphism / Opacity / Grain
    let opacity = if app.saved.bar_glass {
        app.saved.bar_opacity.clamp(0.15, 1.0)
    } else {
        1.0
    };
    let bg_color = Color32::from_rgba_unmultiplied(PANEL.r(), PANEL.g(), PANEL.b(), (opacity * 255.0) as u8);
    ui.painter().rect_filled(deck, rounding, bg_color);
    if let Some(tint) = tint {
        let glow = mood(ui, "bar-glow", Some(tint), PANEL, 0.16 * opacity);
        ui.painter().rect_filled(deck, rounding, glow);
    }
    if app.saved.bar_layout == BarLayout::FloatingPill || app.saved.bar_layout == BarLayout::MiniCapsule {
        ui.painter().rect_stroke(
            deck,
            rounding,
            Stroke::new(1.0_f32, Color32::from_white_alpha(32)),
            StrokeKind::Inside,
        );
    }
    if app.saved.bar_grain {
        paint_glass_grain(ui.painter(), deck, app.saved.bar_grain_strength);
    }

    if app.saved.bar_layout == BarLayout::MiniCapsule {
        let pal_color = app.saved.theme_palette.color();
        let art_center = pos2(deck.left() + 28.0, mid);
        let angle = if app.playing() {
            ui.ctx().request_repaint_after(Duration::from_millis(16));
            (ui.ctx().input(|i| i.time) * 1.5) as f32
        } else {
            0.0
        };
        let cover_str = item.map_or("", |it| app.saved.covers.of(it));
        rotating_circular_artwork(ui, &app.art, cover_str, art_center, 18.0, angle, pal_color);

        let title_x = deck.left() + 54.0;
        let title_w = (deck.center().x - 70.0 - title_x).max(60.0);
        let title_str = item.map_or("AURI", |it| it.title.as_str());
        let sub_str = item.map_or("Ready", |it| it.subtitle.as_str());
        label(ui, pos2(title_x, mid - 8.0), Align2::LEFT_CENTER, title_str, medium(12.0), TEXT, title_w);
        label(ui, pos2(title_x, mid + 8.0), Align2::LEFT_CENTER, sub_str, sans(10.0), MUTED, title_w);

        let play_center = pos2(deck.center().x + 20.0, mid);
        let play_rect = Rect::from_center_size(play_center, vec2(30.0, 30.0));
        let play_resp = ui.interact(play_rect, Id::new("capsule-bar-play"), Sense::click());
        let active = item.is_some();
        ui.painter().circle_filled(play_center, 14.0, if active { TEXT } else { veil(0.14) });
        let symbol = if app.playing() { icon::PAUSE } else { icon::PLAY };
        ui.painter().text(play_center, Align2::CENTER_CENTER, symbol, solid(14.0), if active { PANEL } else { FAINT });
        if play_resp.clicked() {
            out.push(Action::Toggle);
        }

        let prev_rect = Rect::from_center_size(pos2(play_center.x - 30.0, mid), vec2(24.0, 24.0));
        if icon_button(ui, prev_rect, Id::new("capsule-bar-prev"), icon::SKIP_BACK, solid(13.0), if active { TEXT } else { FAINT }, "Previous").clicked() {
            out.push(Action::Previous);
        }

        let next_rect = Rect::from_center_size(pos2(play_center.x + 30.0, mid), vec2(24.0, 24.0));
        if icon_button(ui, next_rect, Id::new("capsule-bar-next"), icon::SKIP_FORWARD, solid(13.0), if active { TEXT } else { FAINT }, "Next").clicked() {
            out.push(Action::Next);
        }

        let vol_rect = Rect::from_center_size(pos2(deck.right() - 56.0, mid), vec2(24.0, 24.0));
        let vol_resp = icon_button(
            ui,
            vol_rect,
            Id::new("capsule-bar-vol"),
            if app.saved.volume <= 0.001 { icon::SPEAKER_SIMPLE_SLASH } else { icon::SPEAKER_HIGH },
            glyph(14.0),
            MUTED,
            &format!("Volume: {:.0}% (Scroll to adjust)", app.saved.volume * 100.0),
        );
        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
        if vol_resp.hovered() && scroll_delta.abs() > 0.1 {
            let next_vol = (app.saved.volume + scroll_delta.signum() * 0.05).clamp(0.0, 1.0);
            out.push(Action::Volume(next_vol));
        }
        if vol_resp.clicked() {
            let next_vol = if app.saved.volume > 0.05 { 0.0 } else { 0.70 };
            out.push(Action::Volume(next_vol));
        }

        let expand_rect = Rect::from_center_size(pos2(deck.right() - 26.0, mid), vec2(24.0, 24.0));
        let exp_symbol = if app.expanded { icon::ARROWS_IN_SIMPLE } else { icon::ARROWS_OUT_SIMPLE };
        if icon_button(ui, expand_rect, Id::new("capsule-bar-exp"), exp_symbol, glyph(13.0), MUTED, "Toggle Now Playing").clicked() {
            out.push(Action::Expand(!app.expanded));
        }

        let scrub_track = Rect::from_min_size(pos2(deck.left() + 24.0, deck.bottom() - 4.5), vec2(deck.width() - 48.0, 3.0));
        let sense = if duration > 0 { Sense::click_and_drag() } else { Sense::hover() };
        let scrub_resp = ui.interact(scrub_track.expand(4.0), Id::new("capsule-bar-scrub"), sense);
        if let (true, Some(pointer)) = (scrub_resp.dragged() || scrub_resp.clicked(), scrub_resp.interact_pointer_pos()) {
            if duration > 0 {
                let fraction = ((pointer.x - scrub_track.left()) / scrub_track.width()).clamp(0.0, 1.0);
                out.push(Action::Seek((fraction * duration as f32) as u64));
            }
        }
        let pos_ms = app.player.position_ms().min(duration);
        let frac = if duration > 0 { pos_ms as f32 / duration as f32 } else { 0.0 };
        ui.painter().rect_filled(scrub_track, 1.5, veil(0.12));
        let fill_rect = Rect::from_min_size(scrub_track.min, vec2(scrub_track.width() * frac, 3.0));
        ui.painter().rect_filled(fill_rect, 1.5, pal_color);
        let thumb_x = scrub_track.left() + scrub_track.width() * frac;
        ui.painter().circle_filled(pos2(thumb_x, scrub_track.center().y), 3.5, pal_color);
        ui.painter().circle_filled(pos2(thumb_x, scrub_track.center().y), 1.5, Color32::WHITE);

        return Scrub {
            bar: scrub_track,
            played: frac,
            buffered: if item.is_some() { app.player.buffered() } else { 0.0 },
            hover: 0.0,
            aim: None,
        };
    }

    let backdrop = ui.interact(deck, Id::new("bar-backdrop"), Sense::click());
    if backdrop.clicked() && item.is_some() {
        out.push(Action::Expand(!app.expanded));
    }

    let zone = if app.saved.bar_layout == BarLayout::FloatingPill {
        Rect::from_min_size(pos2(deck.left() + 26.0, deck.top()), vec2(deck.width() - 52.0, 10.0))
    } else {
        Rect::from_min_size(bar.min, vec2(bar.width(), 12.0))
    };
    let sense = if duration > 0 { Sense::click_and_drag() } else { Sense::hover() };
    let scrubber = ui.interact(zone, Id::new("scrubber"), sense);
    if duration > 0 {
        point_at(&scrubber);
    }
    let fraction_at = |pointer: Pos2| ((pointer.x - zone.left()) / zone.width()).clamp(0.0, 1.0);
    let pointed = scrubber.interact_pointer_pos().map(fraction_at);
    let aimed = pointed.or(scrubber.hover_pos().map(fraction_at)).filter(|_| duration > 0);
    let mut position = app.player.position_ms().min(duration);
    if let (true, Some(fraction)) = (scrubber.dragged() || scrubber.is_pointer_button_down_on(), pointed) {
        position = (fraction * duration as f32) as u64;
    }
    if let (true, Some(fraction)) = (scrubber.clicked() || scrubber.drag_stopped(), pointed) {
        out.push(Action::Seek((fraction * duration as f32) as u64));
    }
    let scrub = Scrub {
        bar: zone,
        played: if duration > 0 { position as f32 / duration as f32 } else { 0.0 },
        buffered: if item.is_some() { app.player.buffered() } else { 0.0 },
        hover: ui
            .ctx()
            .animate_bool_with_time(scrubber.id, scrubber.hovered() || scrubber.dragged(), HOVER_TIME),
        aim: aimed.map(|fraction| (fraction, (fraction * duration as f32) as u64)),
    };

    match item {
        Some(item) => {
            let art = Rect::from_min_size(pos2(deck.left() + 18.0, mid - BAR_ART / 2.0), Vec2::splat(BAR_ART));
            artwork(ui, &app.art, app.saved.covers.of(item), art, 6.0);
            let left = art.right() + 14.0;
            let width = (deck.center().x - 170.0 - left - 78.0).max(60.0);
            let title = label(
                ui,
                pos2(left, mid - 10.0),
                Align2::LEFT_CENTER,
                &item.title,
                medium(14.0),
                TEXT,
                width,
            );
            let reach = left + fit(ui, &item.subtitle, sans(12.5), MUTED, width, 1).size().x;
            let zone = Rect::from_min_max(art.min, pos2(title.right().max(reach), art.bottom()));
            let open = ui.interact(zone, Id::new("bar-now-playing"), Sense::click());
            let lift = hover_of(ui, &open);
            let symbol = if app.expanded { icon::CARET_DOWN } else { icon::CARET_UP };
            ui.painter().rect_filled(art, 6.0, shade(0.55 * lift));
            ui.painter()
                .text(art.center(), Align2::CENTER_CENTER, symbol, glyph(20.0), TEXT.gamma_multiply(lift));
            describe(&open, "Now playing");
            if open.clicked() {
                out.push(Action::Expand(!app.expanded));
            }
            let below = pos2(left, mid + 10.0);
            let artist = byline(ui, out, item, Id::new("bar-artist"), below, sans(12.5), width);
            let liked = app.saved.liked.iter().any(|liked| liked.video_id == item.video_id);
            let slot = Rect::from_center_size(pos2(title.right().max(artist.right()) + 26.0, mid), Vec2::splat(34.0));
            let (font, color, hint) = if liked {
                (solid(17.0), ACCENT, "Remove from Liked songs")
            } else {
                (glyph(17.0), MUTED, "Add to Liked songs")
            };
            if icon_button(ui, slot, Id::new("bar-like"), icon::HEART, font, color, hint).clicked() {
                out.push(Action::Like(item.clone()));
            }
            let share = Rect::from_center_size(pos2(slot.center().x + 34.0, mid), Vec2::splat(34.0));
            if icon_button(
                ui,
                share,
                Id::new("bar-share"),
                icon::SHARE_NETWORK,
                glyph(16.0),
                MUTED,
                "Copy playback URL",
            )
            .clicked()
            {
                out.push(Action::Share(item.clone()));
            }
        }
        None => {
            let art = Rect::from_min_size(pos2(deck.left() + 18.0, mid - BAR_ART / 2.0), Vec2::splat(BAR_ART));
            ui.painter().rect_filled(art, 6.0, veil(0.05));
            ui.painter()
                .text(art.center(), Align2::CENTER_CENTER, icon::VINYL_RECORD, glyph(22.0), FAINT);
            label(
                ui,
                pos2(art.right() + 14.0, mid),
                Align2::LEFT_CENTER,
                "Nothing playing",
                sans(13.5),
                FAINT,
                200.0,
            );
        }
    }

    let center = pos2(deck.center().x, mid);
    let active = item.is_some();
    let play = Rect::from_center_size(center, Vec2::splat(42.0));
    let toggle = ui.interact(play, Id::new("bar-play"), Sense::click());
    let lift = hover_of(ui, &toggle);
    let pressed = if toggle.is_pointer_button_down_on() { 1.0 } else { 0.0 };
    ui.painter()
        .circle_filled(center, 20.0 + lift - pressed, if active { TEXT } else { veil(0.14) });
    if app.player.loading() {
        egui::Spinner::new()
            .color(PANEL)
            .paint_at(ui, Rect::from_center_size(center, Vec2::splat(18.0)));
    } else {
        let symbol = if app.playing() { icon::PAUSE } else { icon::PLAY };
        ui.painter().text(
            center,
            Align2::CENTER_CENTER,
            symbol,
            solid(19.0),
            if active { PANEL } else { FAINT },
        );
    }
    let name = if app.playing() { "Pause" } else { "Play" };
    describe(&toggle, name);
    if toggle.on_hover_text(name).clicked() {
        out.push(Action::Toggle);
    }
    let around = |offset: f32, size: f32| Rect::from_center_size(center + vec2(offset, 0.0), Vec2::splat(size));
    let tone = if active { TEXT } else { FAINT };
    if icon_button(
        ui,
        around(-56.0, 36.0),
        Id::new("bar-previous"),
        icon::SKIP_BACK,
        solid(18.0),
        tone,
        "Previous",
    )
    .clicked()
    {
        out.push(Action::Previous);
    }
    if icon_button(
        ui,
        around(56.0, 36.0),
        Id::new("bar-next"),
        icon::SKIP_FORWARD,
        solid(18.0),
        tone,
        "Next",
    )
    .clicked()
    {
        out.push(Action::Next);
    }
    let shuffle = if app.saved.shuffle { ACCENT } else { MUTED };
    if icon_button(
        ui,
        around(-104.0, 34.0),
        Id::new("bar-shuffle"),
        icon::SHUFFLE,
        glyph(17.0),
        shuffle,
        "Shuffle",
    )
    .clicked()
    {
        out.push(Action::ToggleShuffle);
    }
    let (symbol, repeat, hint) = match app.saved.repeat {
        Repeat::Off => (icon::REPEAT, MUTED, "Repeat"),
        Repeat::All => (icon::REPEAT, ACCENT, "Repeat one"),
        Repeat::One => (icon::REPEAT_ONCE, ACCENT, "Stop repeating"),
    };
    if icon_button(ui, around(104.0, 34.0), Id::new("bar-repeat"), symbol, glyph(17.0), repeat, hint).clicked() {
        out.push(Action::CycleRepeat);
    }

    let mut right = deck.right() - 22.0;
    let volume = app.saved.volume;
    let track = Rect::from_min_max(pos2(right - 96.0, mid - 8.0), pos2(right, mid + 8.0));
    let slider = ui.interact(track, Id::new("bar-volume"), Sense::click_and_drag());
    point_at(&slider);
    let grip = ui
        .ctx()
        .animate_bool_with_time(slider.id, slider.hovered() || slider.dragged(), HOVER_TIME);
    if let (true, Some(pointer)) = (
        slider.dragged() || slider.is_pointer_button_down_on(),
        slider.interact_pointer_pos(),
    ) {
        out.push(Action::Volume((pointer.x - track.left()) / track.width()));
    }
    let wheel = ui.input(|input| input.smooth_scroll_delta.y);
    if slider.hovered() && wheel != 0.0 {
        out.push(Action::Volume(volume + wheel * 0.002));
    }
    let line = Rect::from_center_size(track.center(), vec2(track.width(), 4.0));
    let level = Rect::from_min_size(line.min, vec2(line.width() * volume, 4.0));
    ui.painter().rect_filled(line, 2.0, veil(0.14));
    ui.painter().rect_filled(level, 2.0, TEXT.lerp_to_gamma(ACCENT, grip));
    ui.painter().circle_filled(pos2(level.right(), mid), 6.0 * grip, TEXT);
    slider.widget_info(|| WidgetInfo::slider(true, volume as f64, "Volume"));
    right -= 96.0 + 22.0;

    let symbol = if volume == 0.0 {
        icon::SPEAKER_X
    } else if volume < 0.5 {
        icon::SPEAKER_LOW
    } else {
        icon::SPEAKER_HIGH
    };
    let memory = Id::new("volume-before-mute");
    if icon_button(
        ui,
        Rect::from_center_size(pos2(right, mid), Vec2::splat(34.0)),
        Id::new("bar-mute"),
        symbol,
        glyph(18.0),
        MUTED,
        "Mute",
    )
    .clicked()
    {
        let restored = ui.data(|data| data.get_temp::<f32>(memory)).unwrap_or(0.8);
        ui.data_mut(|data| data.insert_temp(memory, if volume > 0.0 { volume } else { 0.8 }));
        out.push(Action::Volume(if volume > 0.0 { 0.0 } else { restored }));
    }
    right -= 38.0;
    let queue = if app.saved.queue_open { ACCENT } else { MUTED };
    if icon_button(
        ui,
        Rect::from_center_size(pos2(right, mid), Vec2::splat(34.0)),
        Id::new("bar-queue"),
        icon::QUEUE,
        glyph(18.0),
        queue,
        "Queue",
    )
    .clicked()
    {
        out.push(Action::ToggleQueue);
    }
    right -= 38.0;
    let singing = app.expanded && app.saved.lyrics_open;
    if icon_button(
        ui,
        Rect::from_center_size(pos2(right, mid), Vec2::splat(34.0)),
        Id::new("bar-lyrics"),
        icon::SUBTITLES,
        glyph(18.0),
        if singing { ACCENT } else { MUTED },
        "Lyrics",
    )
    .clicked()
    {
        out.push(Action::ShowLyrics(true));
        out.push(Action::Expand(!singing));
    }
    right -= 38.0;
    let mini_color = if app.saved.mini_mode { ACCENT } else { MUTED };
    if icon_button(
        ui,
        Rect::from_center_size(pos2(right, mid), Vec2::splat(34.0)),
        Id::new("bar-mini-mode"),
        icon::PICTURE_IN_PICTURE,
        glyph(18.0),
        mini_color,
        "Gaming Mini Mode",
    )
    .clicked()
    {
        out.push(Action::ToggleMiniMode);
    }
    right -= 38.0;
    let norm_color = if app.saved.loudness_normalizer { ACCENT } else { MUTED };
    if icon_button(
        ui,
        Rect::from_center_size(pos2(right, mid), Vec2::splat(34.0)),
        Id::new("bar-loudness"),
        icon::WAVEFORM,
        glyph(18.0),
        norm_color,
        "Loudness Normalizer (-14.0 LUFS) & Soft Limiter",
    )
    .clicked()
    {
        out.push(Action::ToggleLoudness);
    }
    right -= 38.0;
    if icon_button(
        ui,
        Rect::from_center_size(pos2(right, mid), Vec2::splat(34.0)),
        Id::new("bar-purge"),
        icon::BROOM,
        glyph(18.0),
        MUTED,
        "Purge Working Set RAM (Gaming Optimizer)",
    )
    .clicked()
    {
        out.push(Action::PurgeMemory);
    }
    right -= 34.0;
    if duration > 0 {
        let time = format!("{}  /  {}", clock(position), clock(duration));
        label(ui, pos2(right, mid), Align2::RIGHT_CENTER, &time, sans(12.5), MUTED, 140.0);
    }
    scrub
}

fn paint_scrubber(ctx: &egui::Context, scrub: &Scrub) {
    let painter = ctx.layer_painter(egui::LayerId::background());
    let height = 2.0 + 2.0 * scrub.hover;
    let span = |fraction: f32| Rect::from_min_size(scrub.bar.min - vec2(0.0, height / 2.0), vec2(scrub.bar.width() * fraction, height));
    painter.rect_filled(span(1.0), 0.0, rgb(0x26272C));
    painter.rect_filled(span(scrub.buffered), 0.0, rgb(0x45474E));
    painter.rect_filled(span(scrub.played), 0.0, ACCENT);
    painter.circle_filled(pos2(span(scrub.played).right(), scrub.bar.top()), 6.0 * scrub.hover, TEXT);

    let Some((fraction, ms)) = scrub.aim else { return };
    let mut painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, Id::new("scrub-time")));
    painter.set_opacity(scrub.hover);
    let text = painter.layout_no_wrap(clock(ms), medium(12.5), TEXT);
    let size = text.size() + vec2(20.0, 12.0);
    let reach = (scrub.bar.width() - size.x) / 2.0 - 8.0;
    let x = scrub.bar.center().x + (scrub.bar.width() * (fraction - 0.5)).clamp(-reach, reach);
    let bubble = Rect::from_center_size(pos2(x, scrub.bar.top() - 14.0 - size.y / 2.0), size);
    painter.rect(bubble, 9.0, FLOATING, Stroke::new(1.0_f32, veil(0.1)), StrokeKind::Inside);
    painter.galley(bubble.center() - text.size() / 2.0, text, TEXT);
}

fn notice(app: &App, ctx: &egui::Context, out: &mut Vec<Action>) {
    let Some(notice) = &app.notice else { return };
    egui::Area::new(Id::new("notice"))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -(BAR_HEIGHT + 22.0)))
        .interactable(notice.action.is_some())
        .show(ctx, |ui| {
            let text = fit(ui, &notice.text, medium(13.5), PANEL, 420.0, 2);
            let button = notice.action.as_ref().map(|(name, _)| fit(ui, name, bold(13.0), TEXT, 160.0, 1));
            let button_width = button.as_ref().map_or(0.0, |name| name.size().x + 28.0 + 14.0);
            let height = (text.size().y + 24.0).max(44.0);
            let (toast, _) = ui.allocate_exact_size(vec2(20.0 + text.size().x + button_width + 20.0, height), Sense::hover());
            ui.painter().rect_filled(toast, 22.0, TEXT);
            ui.painter()
                .galley(pos2(toast.left() + 20.0, toast.center().y - text.size().y / 2.0), text, PANEL);
            let (Some(name), Some((hint, action))) = (button, &notice.action) else {
                return;
            };
            let slot = Rect::from_min_max(
                pos2(toast.right() - 8.0 - name.size().x - 28.0, toast.center().y - 15.0),
                pos2(toast.right() - 8.0, toast.center().y + 15.0),
            );
            let response = ui.interact(slot, Id::new("notice-action"), Sense::click());
            let hover = hover_of(ui, &response);
            ui.painter().rect_filled(slot, 15.0, PANEL.lerp_to_gamma(ACCENT, hover));
            let ink = TEXT.lerp_to_gamma(ON_ACCENT, hover);
            ui.painter()
                .galley_with_override_text_color(slot.center() - name.size() / 2.0, name, ink);
            describe(&response, hint);
            if response.clicked() {
                out.push(action.clone());
            }
        });
}

pub fn draw_mini(app: &mut App, ctx: &egui::Context) -> Vec<Action> {
    let mut out = Vec::new();
    let duration = app.player.duration_ms();
    let item = app.now_playing();
    let active = item.is_some();
    let pal_color = app.saved.mini_palette.color();
    let is_locked = app.saved.mini_locked;

    egui::CentralPanel::default()
        .frame(Frame::new().fill(Color32::TRANSPARENT))
        .show(ctx, |ui| {
            let (w, h) = app.saved.mini_variant.size(app.mini_queue_open);
            let card_rect = Rect::from_min_size(ui.max_rect().min, vec2(w, h));
            let rounding = match app.saved.mini_variant {
                MiniVariant::ClassicHud => 16.0,
                MiniVariant::CapsulePill => 28.0,
                MiniVariant::StudioCard => 18.0,
            };

            // Glassmorphic background with custom background tint, opacity and single palette border
            let bg_base = app.saved.mini_bg.color(pal_color);
            let alpha = (app.saved.mini_opacity.clamp(0.05, 1.0) * 255.0) as u8;
            ui.painter().rect_filled(card_rect, rounding, Color32::from_rgba_unmultiplied(bg_base.r(), bg_base.g(), bg_base.b(), alpha));
            ui.painter().rect_stroke(
                card_rect,
                rounding,
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(pal_color.r(), pal_color.g(), pal_color.b(), 160)),
                StrokeKind::Inside,
            );

            if app.saved.mini_grain {
                paint_glass_grain(ui.painter(), card_rect, app.saved.mini_grain_strength);
            }

            // Drag area: user can click and drag anywhere on non-interactive top area to move window
            // Movement is restricted if locked
            let drag_rect = Rect::from_min_max(
                card_rect.min,
                pos2(card_rect.right() - 60.0, card_rect.top() + (if app.saved.mini_variant == MiniVariant::CapsulePill { 30.0 } else { 45.0 })),
            );
            let drag_resp = ui.interact(drag_rect, Id::new("mini-drag"), Sense::click_and_drag());
            if !is_locked && drag_resp.dragged() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            match app.saved.mini_variant {
                MiniVariant::ClassicHud => {
                    // 1. Top-Right Utilities
                    let restore_rect = Rect::from_center_size(pos2(card_rect.right() - 18.0, card_rect.top() + 18.0), vec2(24.0, 24.0));
                    if icon_button(ui, restore_rect, Id::new("mini-restore"), icon::ARROWS_OUT_SIMPLE, glyph(15.0), MUTED, "Restore Normal Mode").clicked() {
                        out.push(Action::ToggleMiniMode);
                    }

                    let pin_rect = Rect::from_center_size(pos2(card_rect.right() - 42.0, card_rect.top() + 18.0), vec2(24.0, 24.0));
                    let pin_tone = if is_locked { pal_color } else { MUTED };
                    let pin_hint = if is_locked { "Mini Mode Position Locked (Click to unlock)" } else { "Lock Mini Mode Position (Restrict movement)" };
                    if icon_button(ui, pin_rect, Id::new("mini-pin"), icon::PUSH_PIN, glyph(15.0), pin_tone, pin_hint).clicked() {
                        out.push(Action::ToggleMiniLock);
                    }

                    if app.saved.mini_show_queue {
                        let q_rect = Rect::from_center_size(pos2(card_rect.right() - 66.0, card_rect.top() + 18.0), vec2(24.0, 24.0));
                        let q_tone = if app.mini_queue_open { pal_color } else { MUTED };
                        if icon_button(ui, q_rect, Id::new("mini-queue-toggle"), icon::LIST, glyph(15.0), q_tone, "Upcoming Queue").clicked() {
                            out.push(Action::ToggleMiniQueue);
                        }
                    }

                    // 2. Left: Album Art
                    let art_rect = Rect::from_min_size(pos2(card_rect.left() + 14.0, card_rect.top() + 12.0), vec2(64.0, 64.0));
                    if let Some(item) = item {
                        artwork(ui, &app.art, app.saved.covers.of(item), art_rect, 10.0);
                    } else {
                        ui.painter().rect_filled(art_rect, 10.0, veil(0.06));
                        ui.painter().text(art_rect.center(), Align2::CENTER_CENTER, icon::VINYL_RECORD, glyph(28.0), pal_color);
                    }

                    // 3. Middle Top: Title, Subtitle
                    let mid_left = art_rect.right() + 12.0;
                    let title_w = card_rect.width() - 170.0;
                    let title_str = item.map_or("AURI Gaming Mode", |it| it.title.as_str());
                    let sub_str = item.map_or("Ready to play", |it| it.subtitle.as_str());
                    label(ui, pos2(mid_left, card_rect.top() + 16.0), Align2::LEFT_CENTER, title_str, medium(13.0), TEXT, title_w);
                    label(ui, pos2(mid_left, card_rect.top() + 30.0), Align2::LEFT_CENTER, sub_str, sans(11.0), MUTED, title_w);

                    // 4. Center Transport Controls (Centered horizontally in card)
                    let play_center = pos2(card_rect.center().x, card_rect.top() + 52.0);
                    let play_rect = Rect::from_center_size(play_center, vec2(30.0, 30.0));
                    let play_resp = ui.interact(play_rect, Id::new("mini-play"), Sense::click());
                    ui.painter().circle_filled(play_center, 14.0, if active { TEXT } else { veil(0.14) });
                    let symbol = if app.playing() { icon::PAUSE } else { icon::PLAY };
                    ui.painter().text(play_center, Align2::CENTER_CENTER, symbol, solid(15.0), if active { PANEL } else { FAINT });
                    if play_resp.clicked() {
                        out.push(Action::Toggle);
                    }

                    let prev_rect = Rect::from_center_size(pos2(play_center.x - 28.0, play_center.y), vec2(24.0, 24.0));
                    if icon_button(ui, prev_rect, Id::new("mini-prev"), icon::SKIP_BACK, solid(14.0), if active { TEXT } else { FAINT }, "Previous").clicked() {
                        out.push(Action::Previous);
                    }

                    let next_rect = Rect::from_center_size(pos2(play_center.x + 28.0, play_center.y), vec2(24.0, 24.0));
                    if icon_button(ui, next_rect, Id::new("mini-next"), icon::SKIP_FORWARD, solid(14.0), if active { TEXT } else { FAINT }, "Next").clicked() {
                        out.push(Action::Next);
                    }

                    if app.saved.mini_show_shuffle_repeat {
                        let shuf_rect = Rect::from_center_size(pos2(play_center.x - 56.0, play_center.y), vec2(22.0, 22.0));
                        let shuf_tone = if app.saved.shuffle { pal_color } else { MUTED };
                        if icon_button(ui, shuf_rect, Id::new("mini-shuffle"), icon::SHUFFLE, glyph(13.0), shuf_tone, "Shuffle").clicked() {
                            out.push(Action::ToggleShuffle);
                        }

                        let rep_rect = Rect::from_center_size(pos2(play_center.x + 56.0, play_center.y), vec2(22.0, 22.0));
                        let rep_tone = if app.saved.repeat != Repeat::Off { pal_color } else { MUTED };
                        let rep_sym = if app.saved.repeat == Repeat::One { icon::REPEAT_ONCE } else { icon::ARROWS_CLOCKWISE };
                        if icon_button(ui, rep_rect, Id::new("mini-repeat"), rep_sym, glyph(13.0), rep_tone, "Repeat").clicked() {
                            out.push(Action::CycleRepeat);
                        }
                    }

                    // 5. Vertical Volume Bar (Moved down and left into card)
                    if app.saved.mini_show_volume {
                        let vol_rect = Rect::from_min_max(
                            pos2(card_rect.right() - 24.0, card_rect.top() + 36.0),
                            pos2(card_rect.right() - 20.0, card_rect.top() + 88.0),
                        );
                        let vol_resp = ui.interact(vol_rect.expand(6.0), Id::new("mini-vert-vol"), Sense::click_and_drag());
                        if let (true, Some(pointer)) = (vol_resp.dragged() || vol_resp.clicked(), vol_resp.interact_pointer_pos()) {
                            let fraction = ((vol_rect.bottom() - pointer.y) / vol_rect.height()).clamp(0.0, 1.0);
                            out.push(Action::Volume(fraction));
                        }
                        ui.painter().rect_filled(vol_rect, 2.0, veil(0.12));
                        let fill_h = vol_rect.height() * app.saved.volume;
                        let vol_fill = Rect::from_min_max(pos2(vol_rect.left(), vol_rect.bottom() - fill_h), vol_rect.max);
                        ui.painter().rect_filled(vol_fill, 2.0, pal_color);
                        describe(&vol_resp, &format!("Volume: {:.0}%", app.saved.volume * 100.0));
                    }

                    // 6. Bottom Song Control Bar (Moved down with thumb knob & timestamps)
                    let scrub_y = card_rect.top() + 92.0;
                    let scrub_rect = Rect::from_min_max(
                        pos2(card_rect.left() + 14.0, scrub_y),
                        pos2(card_rect.right() - 36.0, scrub_y + 3.5),
                    );
                    let scrub_resp = ui.interact(scrub_rect.expand(4.0), Id::new("mini-bottom-scrubber"), Sense::click_and_drag());
                    if let (true, Some(pointer)) = (scrub_resp.dragged() || scrub_resp.clicked(), scrub_resp.interact_pointer_pos()) {
                        if duration > 0 {
                            let fraction = ((pointer.x - scrub_rect.left()) / scrub_rect.width()).clamp(0.0, 1.0);
                            out.push(Action::Seek((fraction * duration as f32) as u64));
                        }
                    }
                    let pos_ms = app.player.position_ms().min(duration);
                    let frac = if duration > 0 { pos_ms as f32 / duration as f32 } else { 0.0 };
                    ui.painter().rect_filled(scrub_rect, 1.75, veil(0.14));
                    let fill_rect = Rect::from_min_size(scrub_rect.min, vec2(scrub_rect.width() * frac, 3.5));
                    ui.painter().rect_filled(fill_rect, 1.75, pal_color);
                    let thumb_x = scrub_rect.left() + scrub_rect.width() * frac;
                    ui.painter().circle_filled(pos2(thumb_x, scrub_rect.center().y), 4.5, pal_color);
                    ui.painter().circle_filled(pos2(thumb_x, scrub_rect.center().y), 2.0, Color32::WHITE);

                    if app.saved.mini_show_clock && duration > 0 {
                        let dur_text = format!("{} / {}", clock(pos_ms), clock(duration));
                        ui.painter().text(pos2(card_rect.left() + 14.0, card_rect.top() + 96.0), Align2::LEFT_CENTER, dur_text, sans(10.0), FAINT);
                    }
                }
                MiniVariant::CapsulePill => {
                    // 1. Left: Spinning vinyl circular album art
                    let art_center = pos2(card_rect.left() + 28.0, card_rect.center().y);
                    let angle = if app.playing() {
                        ctx.request_repaint_after(Duration::from_millis(16));
                        (ctx.input(|i| i.time) * 1.5) as f32
                    } else {
                        0.0
                    };
                    let cover = item.map_or("", |it| app.saved.covers.of(it));
                    rotating_circular_artwork(ui, &app.art, cover, art_center, 19.0, angle, pal_color);

                    // 2. Compact Title & Artist
                    let title_x = card_rect.left() + 54.0;
                    let title_str = item.map_or("AURI", |it| it.title.as_str());
                    let sub_str = item.map_or("Ready", |it| it.subtitle.as_str());
                    label(ui, pos2(title_x, card_rect.center().y - 8.0), Align2::LEFT_CENTER, title_str, medium(11.5), TEXT, 75.0);
                    label(ui, pos2(title_x, card_rect.center().y + 8.0), Align2::LEFT_CENTER, sub_str, sans(9.5), MUTED, 75.0);

                    // 3. Centered 3 Transport Buttons
                    let play_center = pos2(card_rect.center().x + 10.0, card_rect.center().y);
                    let play_rect = Rect::from_center_size(play_center, vec2(26.0, 26.0));
                    let play_resp = ui.interact(play_rect, Id::new("pill-play"), Sense::click());
                    ui.painter().circle_filled(play_center, 12.0, if active { TEXT } else { veil(0.14) });
                    let symbol = if app.playing() { icon::PAUSE } else { icon::PLAY };
                    ui.painter().text(play_center, Align2::CENTER_CENTER, symbol, solid(13.0), if active { PANEL } else { FAINT });
                    if play_resp.clicked() {
                        out.push(Action::Toggle);
                    }

                    let prev_rect = Rect::from_center_size(pos2(play_center.x - 24.0, play_center.y), vec2(20.0, 20.0));
                    if icon_button(ui, prev_rect, Id::new("pill-prev"), icon::SKIP_BACK, solid(12.0), if active { TEXT } else { FAINT }, "Previous").clicked() {
                        out.push(Action::Previous);
                    }

                    let next_rect = Rect::from_center_size(pos2(play_center.x + 24.0, play_center.y), vec2(20.0, 20.0));
                    if icon_button(ui, next_rect, Id::new("pill-next"), icon::SKIP_FORWARD, solid(12.0), if active { TEXT } else { FAINT }, "Next").clicked() {
                        out.push(Action::Next);
                    }

                    // 4. Volume Icon Button (Click to cycle, scroll to adjust)
                    let vol_btn_rect = Rect::from_center_size(pos2(play_center.x + 50.0, play_center.y), vec2(20.0, 20.0));
                    let vol_resp = icon_button(
                        ui,
                        vol_btn_rect,
                        Id::new("pill-vol-btn"),
                        icon::SPEAKER_HIGH,
                        glyph(13.0),
                        if app.saved.volume > 0.0 { pal_color } else { MUTED },
                        &format!("Volume: {:.0}% (Click to cycle, scroll to adjust)", app.saved.volume * 100.0),
                    );
                    if vol_resp.clicked() {
                        let next_vol = if app.saved.volume >= 0.99 { 0.35 } else if app.saved.volume >= 0.65 { 1.0 } else { 0.70 };
                        out.push(Action::Volume(next_vol));
                    }
                    let scroll = ui.input(|i| i.raw_scroll_delta.y);
                    if vol_resp.hovered() && scroll.abs() > 0.1 {
                        let delta = if scroll > 0.0 { 0.05 } else { -0.05 };
                        out.push(Action::Volume((app.saved.volume + delta).clamp(0.0, 1.0)));
                    }

                    // 5. Far Right Utilities: Lock Pin & Restore
                    let pin_rect = Rect::from_center_size(pos2(card_rect.right() - 36.0, card_rect.center().y), vec2(20.0, 20.0));
                    let pin_tone = if is_locked { pal_color } else { MUTED };
                    let pin_hint = if is_locked { "Mini Mode Position Locked (Click to unlock)" } else { "Lock Mini Mode Position (Restrict movement)" };
                    if icon_button(ui, pin_rect, Id::new("pill-pin"), icon::PUSH_PIN, glyph(13.0), pin_tone, pin_hint).clicked() {
                        out.push(Action::ToggleMiniLock);
                    }

                    let restore_rect = Rect::from_center_size(pos2(card_rect.right() - 16.0, card_rect.center().y), vec2(20.0, 20.0));
                    if icon_button(ui, restore_rect, Id::new("pill-restore"), icon::ARROWS_OUT_SIMPLE, glyph(13.0), MUTED, "Restore Normal Mode").clicked() {
                        out.push(Action::ToggleMiniMode);
                    }
                    // No shuffle, repeat, queue, or bottom scrubber line
                }
                MiniVariant::StudioCard => {
                    // 1. Top Right: Utilities (Queue, Pin, Restore)
                    let restore_rect = Rect::from_center_size(pos2(card_rect.right() - 18.0, card_rect.top() + 20.0), vec2(22.0, 22.0));
                    if icon_button(ui, restore_rect, Id::new("studio-restore"), icon::ARROWS_OUT_SIMPLE, glyph(14.0), MUTED, "Restore Normal Mode").clicked() {
                        out.push(Action::ToggleMiniMode);
                    }

                    let pin_rect = Rect::from_center_size(pos2(card_rect.right() - 42.0, card_rect.top() + 20.0), vec2(22.0, 22.0));
                    let pin_tone = if is_locked { pal_color } else { MUTED };
                    let pin_hint = if is_locked { "Mini Mode Position Locked (Click to unlock)" } else { "Lock Mini Mode Position (Restrict movement)" };
                    if icon_button(ui, pin_rect, Id::new("studio-pin"), icon::PUSH_PIN, glyph(14.0), pin_tone, pin_hint).clicked() {
                        out.push(Action::ToggleMiniLock);
                    }

                    if app.saved.mini_show_queue {
                        let q_rect = Rect::from_center_size(pos2(card_rect.right() - 66.0, card_rect.top() + 20.0), vec2(22.0, 22.0));
                        let q_tone = if app.mini_queue_open { pal_color } else { MUTED };
                        if icon_button(ui, q_rect, Id::new("studio-queue"), icon::LIST, glyph(14.0), q_tone, "Upcoming Queue").clicked() {
                            out.push(Action::ToggleMiniQueue);
                        }
                    }

                    // 2. Top: Song Title and Artist above the image
                    let title_w = card_rect.width() - 100.0;
                    let title_str = item.map_or("AURI Studio", |it| it.title.as_str());
                    let sub_str = item.map_or("Ready to play", |it| it.subtitle.as_str());
                    label(ui, pos2(card_rect.left() + 18.0, card_rect.top() + 14.0), Align2::LEFT_CENTER, title_str, bold(13.5), TEXT, title_w);
                    label(ui, pos2(card_rect.left() + 18.0, card_rect.top() + 30.0), Align2::LEFT_CENTER, sub_str, sans(11.0), MUTED, title_w);

                    // 3. Large Square Artwork in upper-middle of vertical card
                    let art_rect = Rect::from_center_size(pos2(card_rect.center().x, card_rect.top() + 165.0), vec2(230.0, 230.0));
                    if let Some(item) = item {
                        artwork(ui, &app.art, app.saved.covers.of(item), art_rect, 14.0);
                    } else {
                        ui.painter().rect_filled(art_rect, 14.0, veil(0.06));
                        ui.painter().text(art_rect.center(), Align2::CENTER_CENTER, icon::VINYL_RECORD, glyph(48.0), pal_color);
                    }

                    // 4. Beneath Image: 3 Controls with Shuffle on Left & Repeat on Right
                    let play_y = card_rect.top() + 308.0;
                    let play_center = pos2(card_rect.center().x, play_y);
                    let play_rect = Rect::from_center_size(play_center, vec2(32.0, 32.0));
                    let play_resp = ui.interact(play_rect, Id::new("studio-play"), Sense::click());
                    ui.painter().circle_filled(play_center, 15.0, if active { TEXT } else { veil(0.14) });
                    let symbol = if app.playing() { icon::PAUSE } else { icon::PLAY };
                    ui.painter().text(play_center, Align2::CENTER_CENTER, symbol, solid(16.0), if active { PANEL } else { FAINT });
                    if play_resp.clicked() {
                        out.push(Action::Toggle);
                    }

                    let prev_rect = Rect::from_center_size(pos2(play_center.x - 36.0, play_y), vec2(24.0, 24.0));
                    if icon_button(ui, prev_rect, Id::new("studio-prev"), icon::SKIP_BACK, solid(14.0), if active { TEXT } else { FAINT }, "Previous").clicked() {
                        out.push(Action::Previous);
                    }

                    let next_rect = Rect::from_center_size(pos2(play_center.x + 36.0, play_y), vec2(24.0, 24.0));
                    if icon_button(ui, next_rect, Id::new("studio-next"), icon::SKIP_FORWARD, solid(14.0), if active { TEXT } else { FAINT }, "Next").clicked() {
                        out.push(Action::Next);
                    }

                    let shuf_rect = Rect::from_center_size(pos2(play_center.x - 72.0, play_y), vec2(22.0, 22.0));
                    let shuf_tone = if app.saved.shuffle { pal_color } else { MUTED };
                    if icon_button(ui, shuf_rect, Id::new("studio-shuf"), icon::SHUFFLE, glyph(13.0), shuf_tone, "Shuffle").clicked() {
                        out.push(Action::ToggleShuffle);
                    }

                    let rep_rect = Rect::from_center_size(pos2(play_center.x + 72.0, play_y), vec2(22.0, 22.0));
                    let rep_tone = if app.saved.repeat != Repeat::Off { pal_color } else { MUTED };
                    let rep_sym = if app.saved.repeat == Repeat::One { icon::REPEAT_ONCE } else { icon::ARROWS_CLOCKWISE };
                    if icon_button(ui, rep_rect, Id::new("studio-rep"), rep_sym, glyph(13.0), rep_tone, "Repeat").clicked() {
                        out.push(Action::CycleRepeat);
                    }

                    // 5. Scrubber Line under controls
                    let scrub_y = card_rect.top() + 342.0;
                    let scrub_rect = Rect::from_min_max(
                        pos2(card_rect.left() + 20.0, scrub_y),
                        pos2(card_rect.right() - 20.0, scrub_y + 3.5),
                    );
                    let scrub_resp = ui.interact(scrub_rect.expand(4.0), Id::new("studio-scrub"), Sense::click_and_drag());
                    if let (true, Some(pointer)) = (scrub_resp.dragged() || scrub_resp.clicked(), scrub_resp.interact_pointer_pos()) {
                        if duration > 0 {
                            let fraction = ((pointer.x - scrub_rect.left()) / scrub_rect.width()).clamp(0.0, 1.0);
                            out.push(Action::Seek((fraction * duration as f32) as u64));
                        }
                    }
                    let pos_ms = app.player.position_ms().min(duration);
                    let frac = if duration > 0 { pos_ms as f32 / duration as f32 } else { 0.0 };
                    ui.painter().rect_filled(scrub_rect, 1.75, veil(0.14));
                    let fill_rect = Rect::from_min_size(scrub_rect.min, vec2(scrub_rect.width() * frac, 3.5));
                    ui.painter().rect_filled(fill_rect, 1.75, pal_color);
                    let thumb_x = scrub_rect.left() + scrub_rect.width() * frac;
                    ui.painter().circle_filled(pos2(thumb_x, scrub_rect.center().y), 4.5, pal_color);
                    ui.painter().circle_filled(pos2(thumb_x, scrub_rect.center().y), 2.0, Color32::WHITE);

                    // 6. Bottom Row: Timestamps on left, Volume with Speaker Icon on bottom right
                    let bot_y = card_rect.top() + 366.0;
                    if duration > 0 {
                        let dur_text = format!("{} / {}", clock(pos_ms), clock(duration));
                        ui.painter().text(pos2(card_rect.left() + 20.0, bot_y), Align2::LEFT_CENTER, dur_text, sans(10.5), FAINT);
                    }

                    // Speaker icon
                    ui.painter().text(pos2(card_rect.right() - 88.0, bot_y), Align2::CENTER_CENTER, icon::SPEAKER_HIGH, glyph(13.0), MUTED);
                    let v_line = Rect::from_min_max(pos2(card_rect.right() - 76.0, bot_y - 2.0), pos2(card_rect.right() - 20.0, bot_y + 2.0));
                    let v_resp = ui.interact(v_line.expand(5.0), Id::new("studio-vol"), Sense::click_and_drag());
                    if let (true, Some(pointer)) = (v_resp.dragged() || v_resp.clicked(), v_resp.interact_pointer_pos()) {
                        let fraction = ((pointer.x - v_line.left()) / v_line.width()).clamp(0.0, 1.0);
                        out.push(Action::Volume(fraction));
                    }
                    ui.painter().rect_filled(v_line, 2.0, veil(0.14));
                    let v_fill = Rect::from_min_size(v_line.min, vec2(v_line.width() * app.saved.volume, 4.0));
                    ui.painter().rect_filled(v_fill, 2.0, pal_color);
                    let v_thumb_x = v_line.left() + v_line.width() * app.saved.volume;
                    ui.painter().circle_filled(pos2(v_thumb_x, v_line.center().y), 4.0, pal_color);
                    describe(&v_resp, &format!("Volume: {:.0}%", app.saved.volume * 100.0));
                }
            }

            // Expandable Mini-Queue Drawer (for ClassicHud)
            if app.saved.mini_variant == MiniVariant::ClassicHud && app.mini_queue_open {
                ui.painter().hline(card_rect.x_range(), card_rect.top() + 104.0, Stroke::new(1.0_f32, veil(0.08)));
                let q_top = card_rect.top() + 108.0;
                ui.painter().text(pos2(card_rect.left() + 16.0, q_top + 10.0), Align2::LEFT_CENTER, "UPCOMING QUEUE", bold(10.0), pal_color);
                let start = app.saved.current.map_or(0, |c| c + 1);
                let upcoming: Vec<(usize, &Item)> = app.saved.queue.iter().enumerate().skip(start).take(3).collect();
                if upcoming.is_empty() {
                    label(ui, pos2(card_rect.center().x, q_top + 60.0), Align2::CENTER_CENTER, "No more songs in queue", sans(11.5), FAINT, card_rect.width() - 40.0);
                } else {
                    for (rel_i, &(idx, song)) in upcoming.iter().enumerate() {
                        let row_y = q_top + 24.0 + rel_i as f32 * 40.0;
                        let row_rect = Rect::from_min_size(pos2(card_rect.left() + 10.0, row_y), vec2(card_rect.width() - 20.0, 36.0));
                        let resp = ui.interact(row_rect, Id::new(("mini-q-row", idx)), Sense::click());
                        let hover = hover_of(ui, &resp);
                        if hover > 0.0 {
                            ui.painter().rect_filled(row_rect, 6.0, veil(0.06 * hover));
                        }
                        // Thumbnail
                        let thumb_rect = Rect::from_min_size(pos2(row_rect.left() + 6.0, row_rect.center().y - 14.0), vec2(28.0, 28.0));
                        artwork(ui, &app.art, app.saved.covers.of(song), thumb_rect, 4.0);
                        // Song Title & Artist
                        label(ui, pos2(row_rect.left() + 40.0, row_rect.center().y - 7.0), Align2::LEFT_CENTER, &song.title, medium(12.0), TEXT, row_rect.width() - 50.0);
                        label(ui, pos2(row_rect.left() + 40.0, row_rect.center().y + 8.0), Align2::LEFT_CENTER, &song.subtitle, sans(10.0), MUTED, row_rect.width() - 50.0);
                        if resp.clicked() {
                            out.push(Action::Jump(idx));
                        }
                    }
                }
            }
        });

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_banner_alpha_blend() {
        let bytes = include_bytes!("../assets/banner_left.jpg");
        let img = image::load_from_memory(bytes).unwrap().to_rgba8();
        let (w, h) = img.dimensions();
        let mut transparent_count = 0;
        let mut opaque_count = 0;
        for y in 0..h {
            for x in 0..w {
                let p = img.get_pixel(x, y);
                let v = p[0].max(p[1]).max(p[2]);
                if v <= 16 {
                    transparent_count += 1;
                } else if v >= 32 {
                    opaque_count += 1;
                }
            }
        }
        println!("Transparent pixels: {}, Opaque pixels: {}, Total: {}", transparent_count, opaque_count, w * h);
        assert!(transparent_count > (w * h) / 2);
    }

    #[test]
    fn links_each_artist_name_to_its_own_page() {
        let credit = |name: &str, id: &str| (name.to_owned(), id.to_owned());
        let mut song = Item {
            subtitle: "Çağrı Sinci, Şehinşah & Çağrı Sinci Jr • 4.9M plays".to_owned(),
            artists: vec![
                credit("Çağrı Sinci", "UC1"),
                credit("Şehinşah", "UC2"),
                credit("Çağrı Sinci Jr", "UC3"),
            ],
            ..Item::default()
        };
        assert_eq!(artist_spans(&song), [(0..11, "UC1"), (13..21, "UC2"), (24..38, "UC3")]);

        song.artists = vec![credit("Çağrı Sinci Jr", "UC3"), credit("Şehinşah", "UC2"), credit("Nobody", "UC9")];
        assert_eq!(artist_spans(&song), [(24..38, "UC3"), (13..21, "UC2")]);

        song.artists.clear();
        assert!(artist_spans(&song).is_empty());
        song.artist_id = "UC1".to_owned();
        assert_eq!(artist_spans(&song), [(0..51, "UC1")]);
    }

    #[test]
    fn keeps_the_now_playing_tint_dark_enough_for_white_text() {
        let contrast = |ink: Color32, paper: Color32| (luminance(ink) + 0.05) / (luminance(paper) + 0.05);
        let pale = rgb(0x86917F);
        assert!(contrast(TEXT, pale) < 3.0);
        let tint = dimmed(pale, STAGE_LUMINANCE);
        assert!(contrast(TEXT, tint) > 4.5);
        assert!(tint.g() > tint.r() && tint.r() > tint.b());
        assert_eq!(dimmed(PANEL, STAGE_LUMINANCE), PANEL);
    }

    #[test]
    fn draws_text_with_every_system_font_it_finds() {
        for (name, bytes) in fallback_fonts() {
            let mut fonts = FontDefinitions::empty();
            fonts.font_data.insert(name.clone(), Arc::new(FontData::from_static(bytes)));
            fonts.families.insert(FontFamily::Proportional, vec![name.clone()]);
            fonts.families.insert(FontFamily::Monospace, vec![name.clone()]);
            let ctx = egui::Context::default();
            ctx.set_fonts(fonts);
            let _ = ctx.run(Default::default(), |_| {});
            let text: String = "A1中한กकبאঅத"
                .chars()
                .filter(|c| ctx.fonts_mut(|fonts| fonts.has_glyph(&sans(16.0), *c)))
                .collect();
            let galley = ctx.fonts_mut(|fonts| fonts.layout_no_wrap(text.clone(), sans(16.0), TEXT));
            let blank = galley
                .rows
                .iter()
                .flat_map(|row| &row.glyphs)
                .any(|glyph| glyph.uv_rect.size == Vec2::ZERO);
            assert!(!blank, "{name} cannot draw {text:?}");
        }
    }
}
