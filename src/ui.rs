use crate::app::{
    Action, App, BarLayout, Covers, CustomPlaylist, Load, MiniBackground, MiniPalette, MiniVariant, PerfProfile, Repeat, Route, SettingsTab, StreamQuality, ThemeMode, ThemePreset, Words,
};
use crate::art::Art;
#[allow(unused_imports)]
use crate::discord::Headline;
use crate::hotkeys::{self, Command};
use crate::ytm::{Item, Layout, QUICK_PICKS};
use eframe::egui::ecolor::Hsva;
use eframe::egui::epaint::{RectShape, Shadow};
use eframe::egui::style::ScrollStyle;
use eframe::egui::text::{CCursor, LayoutJob, TextFormat, TextWrapping};
use eframe::egui::{
    self, Align, Align2, Color32, ColorImage, FontData, FontDefinitions, FontFamily, FontId, Frame, Galley, Id, Key, Margin, Painter, Pos2, Rect, Response,
    RichText, Sense, Shape, Stroke, StrokeKind, TextStyle, TextureHandle, TextureId, TextureOptions, Ui, UiBuilder, Vec2, WidgetInfo, WidgetType, pos2, vec2,
};
use egui_phosphor::regular as icon;
use std::ops::Range;
use std::sync::{Arc, LazyLock};
use std::time::Duration;

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
const BAR_HEIGHT: f32 = 84.0;
#[allow(dead_code)]
const ACCOUNT_HEIGHT: f32 = 68.0;
const UPDATE_HEIGHT: f32 = 50.0;
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
const STAGE_ART: f32 = 512.0;
const STAGE_LUMINANCE: f32 = 0.14;
const STAGE_BLUR: u8 = 84;
const PAGE_FADE: f64 = 0.12;
const SETTINGS_WIDTH: f32 = 740.0;
const SETTING_ROW: f32 = 52.0;
const HOTKEY_ROW: f32 = 56.0;
#[allow(dead_code)]
const STATUS_ROW: f32 = 26.0;
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

fn contrast_text_for(bg: Color32) -> Color32 {
    let lum = 0.299 * bg.r() as f32 + 0.587 * bg.g() as f32 + 0.114 * bg.b() as f32;
    if lum > 155.0 {
        Color32::from_rgb(18, 18, 22)
    } else {
        Color32::WHITE
    }
}

fn is_light_color(color: Color32) -> bool {
    0.299 * color.r() as f32 + 0.587 * color.g() as f32 + 0.114 * color.b() as f32 > 155.0
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
    blocked_artists: &'a [String],
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
            GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, ShowWindow, GWL_STYLE, GWL_EXSTYLE, SWP_FRAMECHANGED, SWP_NOMOVE,
            SWP_NOSIZE, WS_CAPTION, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_SYSMENU, WS_THICKFRAME, WS_POPUP,
            WS_EX_TOOLWINDOW, WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOPMOST, SW_HIDE, SW_SHOWNOACTIVATE, HWND_TOPMOST,
        };
        use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
        use windows::Win32::UI::Shell::{ITaskbarList3, TaskbarList};
        use windows::Win32::Foundation::HWND;

        if self.window != 0 {
            unsafe {
                // 1. Hide the window first so Explorer can detach the taskbar button
                ShowWindow(self.window as _, SW_HIDE);

                // 2. Remove tab from Windows taskbar via COM interface
                if let Ok(taskbar) = CoCreateInstance::<_, ITaskbarList3>(&TaskbarList, None, CLSCTX_INPROC_SERVER) {
                    let _ = taskbar.HrInit();
                    let _ = taskbar.DeleteTab(HWND(self.window as _));
                }

                // 3. Strip all window caption and button styles so Windows DWM never renders minimize/maximize/close buttons
                let style = GetWindowLongPtrW(self.window as _, GWL_STYLE) as u32;
                let new_style = (style | WS_POPUP) & !(WS_CAPTION | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU | WS_THICKFRAME);
                SetWindowLongPtrW(self.window as _, GWL_STYLE, new_style as isize);

                // 4. Set WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_NOACTIVATE, and clear WS_EX_APPWINDOW (Metrix-style overlay)
                let ex_style = GetWindowLongPtrW(self.window as _, GWL_EXSTYLE) as u32;
                let new_ex_style = (ex_style | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE) & !WS_EX_APPWINDOW;
                SetWindowLongPtrW(self.window as _, GWL_EXSTYLE, new_ex_style as isize);

                SetWindowPos(self.window as _, HWND_TOPMOST as _, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED);

                // Disable Windows 11 system outer border (DWMWA_BORDER_COLOR = 34, 0xFFFFFFFE is DWMWA_COLOR_NONE)
                let border_color: u32 = 0xFFFFFFFE;
                DwmSetWindowAttribute(self.window as _, 34, (&raw const border_color).cast(), 4);

                // Disable non-client rendering and drop shadow (DWMWA_NCRENDERING_POLICY = 2, DWMNCRP_DISABLED = 1)
                let nc_policy: u32 = 1;
                DwmSetWindowAttribute(self.window as _, 2, (&raw const nc_policy).cast(), 4);

                // Disable Windows 11 rounded window frame (DWMWA_WINDOW_CORNER_PREFERENCE = 33, DWMWCP_DONOTROUND = 1)
                let corner_pref: u32 = 1;
                DwmSetWindowAttribute(self.window as _, 33, (&raw const corner_pref).cast(), 4);

                // Disable Windows 11 compositor backdrop (DWMWA_SYSTEMBACKDROP_TYPE = 38, DWMSBT_NONE = 1)
                let backdrop_type: u32 = 1;
                DwmSetWindowAttribute(self.window as _, 38, (&raw const backdrop_type).cast(), 4);

                let margins = Margins {
                    cx_left_width: -1,
                    cx_right_width: -1,
                    cy_top_height: -1,
                    cy_bottom_height: -1,
                };
                DwmExtendFrameIntoClientArea(self.window, &margins);

                // 5. Show without stealing focus from active fullscreen game or window
                ShowWindow(self.window as _, SW_SHOWNOACTIVATE);
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
            GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, ShowWindow, GWL_STYLE, GWL_EXSTYLE, SWP_FRAMECHANGED, SWP_NOMOVE,
            SWP_NOSIZE, WS_CAPTION, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_SYSMENU, WS_THICKFRAME, WS_POPUP,
            WS_EX_TOOLWINDOW, WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOPMOST, SW_HIDE, SW_SHOW, HWND_NOTOPMOST,
        };
        use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
        use windows::Win32::UI::Shell::{ITaskbarList3, TaskbarList};
        use windows::Win32::Foundation::HWND;

        if self.window != 0 {
            unsafe {
                // 1. Hide briefly to reset window styles and re-register in taskbar
                ShowWindow(self.window as _, SW_HIDE);

                let style = GetWindowLongPtrW(self.window as _, GWL_STYLE) as u32;
                let new_style = (style | WS_THICKFRAME) & !(WS_CAPTION | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU | WS_POPUP);
                SetWindowLongPtrW(self.window as _, GWL_STYLE, new_style as isize);

                // Restore to standard application window on the Windows taskbar, remove TOOLWINDOW and NOACTIVATE
                let ex_style = GetWindowLongPtrW(self.window as _, GWL_EXSTYLE) as u32;
                let new_ex_style = (ex_style & !(WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE)) | WS_EX_APPWINDOW;
                SetWindowLongPtrW(self.window as _, GWL_EXSTYLE, new_ex_style as isize);

                // Restore tab on Windows taskbar via COM interface
                if let Ok(taskbar) = CoCreateInstance::<_, ITaskbarList3>(&TaskbarList, None, CLSCTX_INPROC_SERVER) {
                    let _ = taskbar.HrInit();
                    let _ = taskbar.AddTab(HWND(self.window as _));
                }

                SetWindowPos(self.window as _, HWND_NOTOPMOST as _, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED);

                let border_color: u32 = 0xFFFFFFFF;
                DwmSetWindowAttribute(self.window as _, 34, (&raw const border_color).cast(), 4);

                let nc_policy: u32 = 0;
                DwmSetWindowAttribute(self.window as _, 2, (&raw const nc_policy).cast(), 4);

                let corner_pref: u32 = 0;
                DwmSetWindowAttribute(self.window as _, 33, (&raw const corner_pref).cast(), 4);

                let backdrop_type: u32 = 0;
                DwmSetWindowAttribute(self.window as _, 38, (&raw const backdrop_type).cast(), 4);

                let margins = Margins {
                    cx_left_width: 0,
                    cx_right_width: 0,
                    cy_top_height: 0,
                    cy_bottom_height: 0,
                };
                DwmExtendFrameIntoClientArea(self.window, &margins);

                ShowWindow(self.window as _, SW_SHOW);
            }
        }
    }
}

fn paint_glass_grain(painter: &egui::Painter, rect: Rect, rounding: f32, strength: f32) {
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
            if rounding > 0.0 && !inside_rounded_rect(pos2(x, y), rect, rounding) {
                x += step;
                continue;
            }
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

fn paint_rounded_gradient(painter: &Painter, rect: Rect, rounding: f32, col_left: Color32, col_right: Color32) {
    let r = rounding.min(rect.width() * 0.5).min(rect.height() * 0.5);
    let mut mesh = egui::Mesh::default();
    let center = rect.center();
    let col_center = col_left.lerp_to_gamma(col_right, 0.5);
    mesh.vertices.push(egui::epaint::Vertex { pos: center, uv: Pos2::ZERO, color: col_center });

    let mut perimeter = Vec::with_capacity(36);
    let steps = 6;
    let add_arc = |pts: &mut Vec<Pos2>, arc_center: Pos2, start_ang: f32, end_ang: f32| {
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            let ang = start_ang + t * (end_ang - start_ang);
            pts.push(pos2(arc_center.x + r * ang.cos(), arc_center.y + r * ang.sin()));
        }
    };

    // Top-right arc (-PI/2 to 0)
    add_arc(&mut perimeter, pos2(rect.right() - r, rect.top() + r), -std::f32::consts::FRAC_PI_2, 0.0);
    // Bottom-right arc (0 to PI/2)
    add_arc(&mut perimeter, pos2(rect.right() - r, rect.bottom() - r), 0.0, std::f32::consts::FRAC_PI_2);
    // Bottom-left arc (PI/2 to PI)
    add_arc(&mut perimeter, pos2(rect.left() + r, rect.bottom() - r), std::f32::consts::FRAC_PI_2, std::f32::consts::PI);
    // Top-left arc (PI to 3*PI/2)
    add_arc(&mut perimeter, pos2(rect.left() + r, rect.top() + r), std::f32::consts::PI, std::f32::consts::PI * 1.5);

    for pt in &perimeter {
        let frac = ((pt.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        let color = col_left.lerp_to_gamma(col_right, frac);
        mesh.vertices.push(egui::epaint::Vertex { pos: *pt, uv: Pos2::ZERO, color });
    }

    let n = perimeter.len() as u32;
    for i in 1..=n {
        let next = if i == n { 1 } else { i + 1 };
        mesh.indices.extend_from_slice(&[0, i, next]);
    }

    painter.add(Shape::mesh(mesh));
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

fn panel_bg_color(saved: &crate::app::Saved) -> Color32 {
    if saved.theme_mode == ThemeMode::Light {
        Color32::from_rgb(255, 255, 255)
    } else {
        saved.theme_preset.panel_color()
    }
}

fn base_bg_color(saved: &crate::app::Saved) -> Color32 {
    if saved.theme_mode == ThemeMode::Light {
        Color32::from_rgb(245, 246, 250)
    } else {
        match saved.theme_preset {
            ThemePreset::Default => saved.theme_base.color(),
            preset => preset.base_color(),
        }
    }
}

fn top_window_bar(app: &mut App, ui: &mut Ui, out: &mut Vec<Action>) {
    let bar = ui.max_rect();
    let mid = bar.center().y;
    let pal_color = app.saved.theme_palette.color();
    let is_light = app.saved.theme_mode == ThemeMode::Light;
    let t_color = if is_light { Color32::from_rgb(24, 28, 36) } else { TEXT };
    let m_color = if is_light { Color32::from_rgb(100, 110, 125) } else { MUTED };

    // Bottom subtle line separator
    ui.painter().hline(bar.x_range(), bar.bottom() - 0.5, Stroke::new(1.0_f32, veil(if is_light { 0.12 } else { 0.06 })));

    // Left: Navigation (< > ⟳)
    let mut left = bar.left() + 12.0;
    let can_back = app.at > 0;
    let can_fwd = app.at + 1 < app.history.len();

    let back_rect = Rect::from_center_size(pos2(left + 14.0, mid), Vec2::splat(28.0));
    let back_resp = ui.interact(back_rect, Id::new("win-back"), Sense::click());
    let back_hov = hover_of(ui, &back_resp);
    if can_back {
        ui.painter().rect_filled(back_rect, 6.0, veil(0.04 * back_hov));
        ui.painter().text(back_rect.center(), Align2::CENTER_CENTER, icon::CARET_LEFT, solid(16.0), if back_hov > 0.1 { t_color } else { m_color });
        describe(&back_resp, "Go back (Alt+Left)");
        if back_resp.clicked() {
            out.push(Action::Back);
        }
    } else {
        ui.painter().text(back_rect.center(), Align2::CENTER_CENTER, icon::CARET_LEFT, solid(16.0), FAINT.gamma_multiply(0.4));
    }
    left += 32.0;

    let fwd_rect = Rect::from_center_size(pos2(left + 14.0, mid), Vec2::splat(28.0));
    let fwd_resp = ui.interact(fwd_rect, Id::new("win-fwd"), Sense::click());
    let fwd_hov = hover_of(ui, &fwd_resp);
    if can_fwd {
        ui.painter().rect_filled(fwd_rect, 6.0, veil(0.04 * fwd_hov));
        ui.painter().text(fwd_rect.center(), Align2::CENTER_CENTER, icon::CARET_RIGHT, solid(16.0), if fwd_hov > 0.1 { t_color } else { m_color });
        describe(&fwd_resp, "Go forward (Alt+Right)");
        if fwd_resp.clicked() {
            out.push(Action::Forward);
        }
    } else {
        ui.painter().text(fwd_rect.center(), Align2::CENTER_CENTER, icon::CARET_RIGHT, solid(16.0), FAINT.gamma_multiply(0.4));
    }
    left += 32.0;

    let reload_rect = Rect::from_center_size(pos2(left + 14.0, mid), Vec2::splat(28.0));
    let r_resp = ui.interact(reload_rect, Id::new("win-reload"), Sense::click());
    let r_hov = hover_of(ui, &r_resp);
    ui.painter().rect_filled(reload_rect, 6.0, veil(0.04 * r_hov));
    ui.painter().text(reload_rect.center(), Align2::CENTER_CENTER, icon::ARROW_CLOCKWISE, glyph(15.0), if r_hov > 0.1 { t_color } else { m_color });
    describe(&r_resp, "Reload (Ctrl+R / F5)");
    if r_resp.clicked() {
        out.push(Action::Reload);
    }
    left += 38.0;

    // Right Controls
    let right_edge = bar.right();
    let win_btn_w = 42.0;
    let win_controls_w = win_btn_w * 3.0; // min, max, close

    // Window controls on far right:
    // Close button
    let close_rect = Rect::from_min_size(pos2(right_edge - win_btn_w, bar.top()), vec2(win_btn_w, bar.height()));
    let close_resp = ui.interact(close_rect, Id::new("win-cmd-close"), Sense::click());
    let close_hov = hover_of(ui, &close_resp);
    if close_hov > 0.0 {
        ui.painter().rect_filled(close_rect, 0.0, Color32::from_rgb(232, 17, 35));
        ui.painter().text(close_rect.center(), Align2::CENTER_CENTER, icon::X, glyph(15.0), Color32::WHITE);
    } else {
        ui.painter().text(close_rect.center(), Align2::CENTER_CENTER, icon::X, glyph(15.0), m_color);
    }
    describe(&close_resp, "Close");
    if close_resp.clicked() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
    }

    // Maximize / Restore button
    let is_max = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
    let max_rect = Rect::from_min_size(pos2(right_edge - win_btn_w * 2.0, bar.top()), vec2(win_btn_w, bar.height()));
    let max_resp = ui.interact(max_rect, Id::new("win-cmd-max"), Sense::click());
    let max_hov = hover_of(ui, &max_resp);
    ui.painter().rect_filled(max_rect, 0.0, veil(0.06 * max_hov));
    let max_sym = if is_max { icon::SQUARES_FOUR } else { icon::SQUARE };
    ui.painter().text(max_rect.center(), Align2::CENTER_CENTER, max_sym, glyph(14.0), if max_hov > 0.1 { t_color } else { m_color });
    describe(&max_resp, if is_max { "Restore" } else { "Maximize" });
    if max_resp.clicked() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!is_max));
    }

    // Minimize button
    let min_rect = Rect::from_min_size(pos2(right_edge - win_btn_w * 3.0, bar.top()), vec2(win_btn_w, bar.height()));
    let min_resp = ui.interact(min_rect, Id::new("win-cmd-min"), Sense::click());
    let min_hov = hover_of(ui, &min_resp);
    ui.painter().rect_filled(min_rect, 0.0, veil(0.06 * min_hov));
    ui.painter().text(min_rect.center(), Align2::CENTER_CENTER, icon::MINUS, glyph(14.0), if min_hov > 0.1 { t_color } else { m_color });
    describe(&min_resp, "Minimize");
    if min_resp.clicked() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
    }

    // Divider between window controls and action tools
    let sep_x = right_edge - win_controls_w - 6.0;
    ui.painter().vline(sep_x, bar.y_range().shrink(8.0), Stroke::new(1.0_f32, veil(0.08)));

    let mut tool_right = sep_x - 8.0;

    // Game Mode Icon Button
    let game_rect = Rect::from_center_size(pos2(tool_right - 14.0, mid), Vec2::splat(28.0));
    let game_resp = ui.interact(game_rect, Id::new("win-tool-gamemode"), Sense::click());
    let game_hov = hover_of(ui, &game_resp);
    ui.painter().rect_filled(game_rect, 6.0, if app.in_game_mode { pal_color.gamma_multiply(0.25) } else { veil(0.04 * game_hov) });
    ui.painter().text(game_rect.center(), Align2::CENTER_CENTER, icon::GAME_CONTROLLER, solid(16.0), if app.in_game_mode || game_hov > 0.1 { pal_color } else { m_color });
    describe(&game_resp, "Gaming Mode (Zero-Overhead Player & HUD)");
    if game_resp.clicked() {
        out.push(Action::ToggleGameMode);
    }
    tool_right -= 34.0;

    // Mini Mode Icon Button
    let mini_rect = Rect::from_center_size(pos2(tool_right - 14.0, mid), Vec2::splat(28.0));
    let mini_resp = ui.interact(mini_rect, Id::new("win-tool-mini"), Sense::click());
    let mini_hov = hover_of(ui, &mini_resp);
    ui.painter().rect_filled(mini_rect, 6.0, veil(0.04 * mini_hov));
    ui.painter().text(mini_rect.center(), Align2::CENTER_CENTER, icon::PICTURE_IN_PICTURE, solid(16.0), if mini_hov > 0.1 { pal_color } else { m_color });
    describe(&mini_resp, "Enter Mini Mode (Floating HUD)");
    if mini_resp.clicked() {
        out.push(Action::ToggleMiniMode);
    }
    tool_right -= 34.0;

    // Settings Gear Icon Button
    let is_settings = matches!(app.route(), Route::Settings(_));
    let set_rect = Rect::from_center_size(pos2(tool_right - 14.0, mid), Vec2::splat(28.0));
    let set_resp = ui.interact(set_rect, Id::new("win-tool-settings"), Sense::click());
    let set_hov = hover_of(ui, &set_resp);
    ui.painter().rect_filled(set_rect, 6.0, if is_settings { pal_color.gamma_multiply(0.18) } else { veil(0.04 * set_hov) });
    ui.painter().text(set_rect.center(), Align2::CENTER_CENTER, icon::GEAR_SIX, solid(16.0), if is_settings || set_hov > 0.1 { pal_color } else { m_color });
    describe(&set_resp, "Settings (Ctrl+,)");
    if set_resp.clicked() {
        out.push(Action::SetSettingsTab(SettingsTab::General));
    }
    tool_right -= 34.0;

    // Update Check Icon Button
    let has_upd = app.update.is_some();
    let upd_rect = Rect::from_center_size(pos2(tool_right - 14.0, mid), Vec2::splat(28.0));
    let upd_resp = ui.interact(upd_rect, Id::new("win-tool-update"), Sense::click());
    let upd_hov = hover_of(ui, &upd_resp);
    ui.painter().rect_filled(upd_rect, 6.0, if has_upd { pal_color.gamma_multiply(0.20 + 0.10 * upd_hov) } else { veil(0.04 * upd_hov) });
    if has_upd {
        ui.painter().rect_stroke(upd_rect, 6.0, Stroke::new(1.0_f32, pal_color), StrokeKind::Inside);
    }
    let upd_sym = if has_upd { icon::CLOUD_ARROW_DOWN } else { icon::ARROW_FAT_LINES_DOWN };
    let upd_color = if has_upd { pal_color } else if upd_hov > 0.1 { pal_color } else { m_color };
    ui.painter().text(upd_rect.center(), Align2::CENTER_CENTER, upd_sym, glyph(16.0), upd_color);
    describe(&upd_resp, if has_upd { "Update Available (Click to view details)" } else { "Check for Updates" });
    if upd_resp.clicked() {
        if has_upd {
            out.push(Action::OpenUpdateDialog);
        } else {
            out.push(Action::CheckUpdate);
        }
    }
    tool_right -= 34.0;

    // Equalizer Shortcut Button
    let is_eq = matches!(app.route(), Route::Settings(SettingsTab::Equalizer));
    let eq_rect = Rect::from_center_size(pos2(tool_right - 14.0, mid), Vec2::splat(28.0));
    let eq_resp = ui.interact(eq_rect, Id::new("win-tool-eq"), Sense::click());
    let eq_hov = hover_of(ui, &eq_resp);
    ui.painter().rect_filled(eq_rect, 6.0, if is_eq { pal_color.gamma_multiply(0.18) } else { veil(0.04 * eq_hov) });
    ui.painter().text(eq_rect.center(), Align2::CENTER_CENTER, icon::SLIDERS, solid(16.0), if is_eq || eq_hov > 0.1 { pal_color } else { m_color });
    describe(&eq_resp, "Equalizer & Sound DSP");
    if eq_resp.clicked() {
        out.push(Action::SetSettingsTab(SettingsTab::Equalizer));
    }
    tool_right -= 34.0;

    // Extreme Performance Mode Pill (⚡ Perf)
    let perf_on = app.saved.performance_mode;
    let perf_w = 76.0;
    let perf_rect = Rect::from_min_size(pos2(tool_right - perf_w, mid - 13.0), vec2(perf_w, 26.0));
    let perf_resp = ui.interact(perf_rect, Id::new("win-tool-perf"), Sense::click());
    let perf_hov = hover_of(ui, &perf_resp);
    if perf_on {
        ui.painter().rect_filled(perf_rect, 13.0, pal_color.gamma_multiply(0.20));
        ui.painter().rect_stroke(perf_rect, 13.0, Stroke::new(1.0_f32, pal_color), StrokeKind::Inside);
        ui.painter().text(perf_rect.center(), Align2::CENTER_CENTER, "⚡ Perf", bold(11.5), pal_color);
    } else {
        ui.painter().rect_filled(perf_rect, 13.0, veil(0.04 + 0.04 * perf_hov));
        ui.painter().rect_stroke(perf_rect, 13.0, Stroke::new(1.0_f32, veil(0.08)), StrokeKind::Inside);
        ui.painter().text(perf_rect.center(), Align2::CENTER_CENTER, "⚡ Perf", medium(11.5), m_color);
    }
    describe(&perf_resp, "Toggle Extreme Performance & Gaming Mode");
    if perf_resp.clicked() {
        out.push(Action::TogglePerformanceMode);
    }
    tool_right -= perf_w + 8.0;

    // Discord Presence Toggle (clean glyph + green/red status dot)
    let disc_on = app.saved.discord;
    let disc_rect = Rect::from_center_size(pos2(tool_right - 14.0, mid), Vec2::splat(28.0));
    let disc_resp = ui.interact(disc_rect, Id::new("win-tool-discord"), Sense::click());
    let disc_hov = hover_of(ui, &disc_resp);
    if disc_hov > 0.05 {
        ui.painter().rect_filled(disc_rect, 6.0, veil(0.06 * disc_hov));
    }
    let disc_icon_color = if disc_hov > 0.1 || disc_on { TEXT } else { m_color };
    ui.painter().text(disc_rect.center(), Align2::CENTER_CENTER, icon::DISCORD_LOGO, solid(16.0), disc_icon_color);

    // Top-right status dot
    let dot_center = pos2(disc_rect.center().x + 7.0, disc_rect.center().y - 6.5);
    let dot_color = if disc_on {
        Color32::from_rgb(16, 185, 129) // emerald green
    } else {
        Color32::from_rgb(239, 68, 68)  // red
    };
    ui.painter().circle_filled(dot_center, 3.5, dot_color);
    ui.painter().circle_stroke(dot_center, 3.5, Stroke::new(1.0_f32, Color32::from_rgb(20, 21, 26)));

    describe(&disc_resp, if disc_on { "Discord Presence Active (Click to disable)" } else { "Discord Presence Disconnected (Click to enable)" });
    if disc_resp.clicked() {
        out.push(Action::ShowOnDiscord(!disc_on));
    }
    tool_right -= 34.0;

    // Listen Together / Sync Button
    let sync_on = app.sync_mode != crate::sync::SyncMode::Disconnected;
    let sync_rect = Rect::from_center_size(pos2(tool_right - 14.0, mid), Vec2::splat(28.0));
    let sync_resp = ui.interact(sync_rect, Id::new("win-tool-sync"), Sense::click());
    let sync_hov = hover_of(ui, &sync_resp);
    if sync_on {
        ui.painter().rect_filled(sync_rect, 6.0, Color32::from_rgba_unmultiplied(16, 185, 129, 45));
        ui.painter().rect_stroke(sync_rect, 6.0, Stroke::new(1.0_f32, Color32::from_rgb(16, 185, 129)), StrokeKind::Inside);
        ui.painter().text(sync_rect.center(), Align2::CENTER_CENTER, icon::HEADPHONES, solid(16.0), Color32::from_rgb(52, 211, 153));
    } else {
        ui.painter().rect_filled(sync_rect, 6.0, veil(0.04 * sync_hov));
        ui.painter().text(sync_rect.center(), Align2::CENTER_CENTER, icon::HEADPHONES, glyph(16.0), m_color);
    }
    describe(&sync_resp, "Listen Together / GitHub Sync");
    if sync_resp.clicked() {
        if sync_on {
            out.push(Action::StopSync);
        } else {
            out.push(Action::SetSettingsTab(SettingsTab::DataStorage));
        }
    }
    tool_right -= 34.0;

    // Account Avatar Pill (e.g. avatar circle with user photo + user name + caret down + popup menu)
    let acc_name = app.saved.account.as_ref().map(|a| a.name.as_str()).unwrap_or("Sign in");
    let acc_pill_w = (20.0 + 8.0 + (acc_name.len() as f32 * 6.5) + 20.0).clamp(96.0, 160.0);
    let acc_rect = Rect::from_min_size(pos2(tool_right - acc_pill_w, mid - 13.0), vec2(acc_pill_w, 26.0));
    let acc_resp = ui.interact(acc_rect, Id::new("win-tool-account"), Sense::click());
    let acc_hov = hover_of(ui, &acc_resp);
    ui.painter().rect_filled(acc_rect, 13.0, veil(0.04 + 0.05 * acc_hov));
    ui.painter().rect_stroke(acc_rect, 13.0, Stroke::new(1.0_f32, veil(0.08)), StrokeKind::Inside);
    let av_center = pos2(acc_rect.left() + 13.0, mid);
    if let Some(ref account) = app.saved.account {
        if !account.photo.is_empty() {
            let av_rect = Rect::from_center_size(av_center, Vec2::splat(18.0));
            artwork(ui, &app.art, &account.photo, av_rect, 9.0);
        } else {
            ui.painter().circle_filled(av_center, 9.0, pal_color.gamma_multiply(0.25));
            ui.painter().text(av_center, Align2::CENTER_CENTER, icon::USER, glyph(11.0), pal_color);
        }
    } else {
        ui.painter().circle_filled(av_center, 9.0, pal_color.gamma_multiply(0.25));
        ui.painter().text(av_center, Align2::CENTER_CENTER, icon::USER, glyph(11.0), pal_color);
    }
    label(ui, pos2(acc_rect.left() + 27.0, mid), Align2::LEFT_CENTER, acc_name, medium(11.5), t_color, acc_pill_w - 42.0);
    ui.painter().text(pos2(acc_rect.right() - 10.0, mid), Align2::CENTER_CENTER, icon::CARET_DOWN, glyph(10.0), m_color);
    describe(&acc_resp, "YouTube Music Account & Profiles");

    egui::Popup::menu(&acc_resp).gap(6.0).show(|ui| {
        let popup_frame = Frame::new()
            .fill(Color32::from_rgb(24, 25, 30))
            .stroke(Stroke::new(1.0_f32, veil(0.18)))
            .corner_radius(12.0)
            .shadow(Shadow {
                offset: [0, 8],
                blur: 24,
                spread: 0,
                color: shade(0.70),
            })
            .inner_margin(Margin::same(14));

        popup_frame.show(ui, |ui| {
            ui.set_width(MENU_WIDTH);
            if let Some(ref account) = app.saved.account {
                // Profile card header
                ui.horizontal(|ui| {
                    let (av_rect, _) = ui.allocate_exact_size(Vec2::splat(34.0), Sense::hover());
                    if !account.photo.is_empty() {
                        artwork(ui, &app.art, &account.photo, av_rect, 17.0);
                    } else {
                        ui.painter().circle_filled(av_rect.center(), 17.0, pal_color.gamma_multiply(0.25));
                        ui.painter().text(av_rect.center(), Align2::CENTER_CENTER, icon::USER, glyph(16.0), pal_color);
                    }
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        label(ui, pos2(ui.cursor().left(), ui.cursor().top() + 6.0), Align2::LEFT_CENTER, &account.name, bold(13.5), TEXT, MENU_WIDTH - 54.0);
                        label(ui, pos2(ui.cursor().left(), ui.cursor().top() + 20.0), Align2::LEFT_CENTER, "YouTube Music Connected", sans(11.0), MUTED, MENU_WIDTH - 54.0);
                    });
                });
                ui.add_space(8.0);
                ui.painter().hline(ui.available_rect_before_wrap().x_range(), ui.cursor().top(), Stroke::new(1.0_f32, veil(0.12)));
                ui.add_space(8.0);

                if app.accounts.len() > 1 {
                    ui.label(RichText::new("Channels / Profiles").size(11.0).color(MUTED));
                    for channel in &app.accounts {
                        let chosen = channel.page == account.page;
                        let symbol = if chosen { icon::CHECK } else { "" };
                        let row = menu_row(ui, symbol, glyph(13.0), pal_color, &channel.name);
                        if row.clicked() && !chosen {
                            out.push(Action::SwitchAccount(channel.clone()));
                        }
                    }
                    ui.add_space(6.0);
                    ui.painter().hline(ui.available_rect_before_wrap().x_range(), ui.cursor().top(), Stroke::new(1.0_f32, veil(0.12)));
                    ui.add_space(6.0);
                }

                if menu_row(ui, icon::USER_SWITCH, glyph(14.0), TEXT, "Switch Account").clicked() {
                    out.push(Action::OpenSignIn);
                }

                ui.add_space(4.0);
                if menu_row(ui, icon::SIGN_OUT, glyph(14.0), Color32::from_rgb(239, 68, 68), "Log Out").clicked() {
                    out.push(Action::SignOut);
                }
            } else {
                ui.label(RichText::new("Not Signed In").strong().color(TEXT));
                ui.label(RichText::new("Connect your Google/YouTube Music account to access your playlists, liked tracks, and personalized recommendations.").size(11.0).color(MUTED));
                ui.add_space(8.0);
                if menu_row(ui, icon::SIGN_IN, glyph(14.0), pal_color, "Sign In with YouTube Music").clicked() {
                    out.push(Action::OpenSignIn);
                }
            }
        });
    });
    tool_right -= acc_pill_w + 12.0;

    // Center draggable title area: spans from `left` to `tool_right`
    let center_rect = Rect::from_min_max(pos2(left, bar.top()), pos2(tool_right, bar.bottom()));
    let drag_resp = ui.interact(center_rect, Id::new("win-title-drag-zone"), Sense::click_and_drag());
    if drag_resp.drag_started() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
    if drag_resp.double_clicked() {
        let is_max = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!is_max));
    }
    ui.painter().text(center_rect.center(), Align2::CENTER_CENTER, "AURI", bold(13.0), t_color.gamma_multiply(0.85));
}

pub fn draw(app: &mut App, ctx: &egui::Context) -> Vec<Action> {
    let mut out = Vec::new();
    let plain = |fill: Color32| Frame::new().fill(fill);
    let p_col = panel_bg_color(&app.saved);
    let b_col = base_bg_color(&app.saved);
    let pal_color = app.saved.theme_palette.color();

    ctx.data_mut(|d| d.insert_temp(Id::new("app-theme-accent"), pal_color));
    ctx.style_mut(|s| {
        s.visuals.selection.bg_fill = pal_color.gamma_multiply(0.45);
        s.visuals.text_cursor.stroke = Stroke::new(1.5_f32, pal_color);
    });

    let top_bar_h = 38.0;
    egui::TopBottomPanel::top("top_window_bar")
        .exact_height(top_bar_h)
        .show_separator_line(false)
        .frame(Frame::new().fill(if app.saved.theme_mode == ThemeMode::Light {
            Color32::from_rgb(240, 241, 246)
        } else {
            p_col
        }))
        .show(ctx, |ui| top_window_bar(app, ui, &mut out));

    if app.saved.bar_layout == BarLayout::Docked {
        let bar_frame = if app.saved.bar_glass {
            Frame::new().fill(Color32::TRANSPARENT)
        } else {
            plain(p_col)
        };
        egui::TopBottomPanel::bottom("player")
            .exact_height(BAR_HEIGHT)
            .show_separator_line(false)
            .frame(bar_frame)
            .show(ctx, |ui| { player_bar(app, ui, &mut out); });
    }
    let mut frame = p_col;
    if app.expanded && app.now_playing().is_some() {
        frame = egui::CentralPanel::default()
            .frame(plain(p_col))
            .show(ctx, |ui| now_playing(app, ui, &mut out))
            .inner;
    } else {
        let target_sidebar_w = if app.saved.sidebar_open { SIDEBAR_WIDTH } else { 60.0 };
        let anim_sidebar_w = ctx.animate_value_with_time(Id::new("anim-sidebar-w"), target_sidebar_w, 0.22);
        egui::SidePanel::left("sidebar")
            .exact_width(anim_sidebar_w)
            .resizable(false)
            .show_separator_line(false)
            .frame(plain(p_col))
            .show(ctx, |ui| sidebar(app, ui, anim_sidebar_w, &mut out));
        if app.saved.queue_open {
            egui::SidePanel::right("queue")
                .exact_width(QUEUE_WIDTH)
                .resizable(false)
                .show_separator_line(false)
                .frame(plain(p_col))
                .show(ctx, |ui| queue_panel(app, ui, &mut out));
        } else if app.settings_preview_open
            && matches!(
                app.route(),
                Route::Settings(SettingsTab::Discord | SettingsTab::Appearance | SettingsTab::GamingMode)
            )
        {
            // The settings live preview borrows the queue dock's rail slot, so
            // the settings column itself never shrinks.
            let preview_w = QUEUE_WIDTH - 24.0;
            egui::SidePanel::right("settings-preview")
                .exact_width(QUEUE_WIDTH)
                .resizable(false)
                .show_separator_line(false)
                .frame(plain(p_col))
                .show(ctx, |ui| {
                    ui.add_space(12.0);
                    match app.route() {
                        Route::Settings(SettingsTab::Discord) => render_discord_preview(ui, app, &mut out, preview_w),
                        Route::Settings(SettingsTab::Appearance) => render_appearance_preview(ui, app, &mut out, preview_w),
                        Route::Settings(SettingsTab::GamingMode) => render_gaming_preview(ui, app, &mut out, preview_w),
                        _ => unreachable!(),
                    }
                });
        }
        egui::CentralPanel::default()
            .frame(plain(b_col))
            .show(ctx, |ui| content(app, ui, &mut out));

        if app.saved.queue_retractable && !app.saved.queue_open && !app.settings_preview_open {
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

        if app.saved.bar_layout != BarLayout::Docked {
            let screen = ctx.content_rect();
            let queue_w = if app.saved.queue_open { QUEUE_WIDTH } else { 0.0 };
            let center_x = (anim_sidebar_w + (screen.right() - queue_w)) / 2.0;

            let (deck_w, deck_h) = if app.saved.bar_layout == BarLayout::MiniCapsule {
                (520.0_f32.min(screen.width() - anim_sidebar_w - queue_w - 32.0), 56.0_f32)
            } else {
                let max_w = (screen.width() - anim_sidebar_w - queue_w - 48.0).clamp(640.0, 1080.0);
                (max_w.min(screen.width() - 32.0), 80.0_f32)
            };
            let deck_bottom = screen.bottom() - 14.0;
            let deck_rect = Rect::from_center_size(pos2(center_x, deck_bottom - deck_h / 2.0), vec2(deck_w, deck_h));

            egui::Area::new(Id::new("floating-player-bar"))
                .order(egui::Order::Foreground)
                .fixed_pos(deck_rect.min)
                .show(ctx, |ui| {
                    ui.scope_builder(egui::UiBuilder::new().max_rect(deck_rect), |ui| {
                        player_bar(app, ui, &mut out);
                    });
                });
        }
    }
    app.title_bar.paint(frame);
    sign_in_dialog(app, ctx, &mut out);
    update_dialog(app, ctx, &mut out);
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
    let accent = ui.ctx().data(|d| d.get_temp(Id::new("app-theme-accent"))).unwrap_or(ACCENT);
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
        accent.lerp_to_gamma(Color32::WHITE, 0.12 * hover)
    } else {
        veil(0.09 + 0.05 * hover)
    };
    let ink = if primary { contrast_text_for(accent) } else { TEXT };
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
    let minimized = ctx.input(|i| i.viewport().minimized == Some(true));
    if minimized {
        painter.rect_filled(rect, radius, RAISED);
        return;
    }
    ctx.request_repaint_after(Duration::from_millis(50));
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
    let accent = ctx.data(|d| d.get_temp(Id::new("app-theme-accent"))).unwrap_or(ACCENT);
    let bar_color = accent.gamma_multiply(0.25 + 0.20 * pulse);
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

#[allow(dead_code)]
fn rotating_circular_artwork_clipped(painter: &Painter, ctx: &egui::Context, art: &Art, url: &str, center: Pos2, radius: f32, angle: f32, pal_color: Color32) {
    if let Some(picture) = art.get(ctx, url, radius * 2.0) {
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

        painter.add(Shape::mesh(mesh));

        // Vinyl center spindle hole with silver border & groove rings
        painter.circle_filled(center, 5.0, Color32::BLACK);
        painter.circle_stroke(center, 5.0, Stroke::new(1.0_f32, Color32::from_rgb(180, 180, 190)));
        painter.circle_stroke(center, radius * 0.55, Stroke::new(0.8_f32, veil(0.18)));
        painter.circle_stroke(center, radius - 1.0, Stroke::new(1.2_f32, pal_color.gamma_multiply(0.70)));
    } else {
        painter.circle_filled(center, radius, veil(0.06));
        painter.text(center, Align2::CENTER_CENTER, icon::VINYL_RECORD, glyph(24.0), pal_color);
        painter.circle_stroke(center, radius, Stroke::new(1.0_f32, pal_color.gamma_multiply(0.40)));
    }
}

fn inside_rounded_rect(p: Pos2, rect: Rect, r: f32) -> bool {
    if p.x < rect.left() || p.x > rect.right() || p.y < rect.top() || p.y > rect.bottom() {
        return false;
    }
    let rx = if p.x < rect.left() + r {
        rect.left() + r - p.x
    } else if p.x > rect.right() - r {
        p.x - (rect.right() - r)
    } else {
        0.0
    };
    let ry = if p.y < rect.top() + r {
        rect.top() + r - p.y
    } else if p.y > rect.bottom() - r {
        p.y - (rect.bottom() - r)
    } else {
        0.0
    };
    rx * rx + ry * ry <= r * r
}

fn get_embedded_mood_texture(ctx: &egui::Context, name: &str) -> Option<TextureHandle> {
    let bytes: &'static [u8] = match name {
        "Chill" => include_bytes!("../assets/moods/chill.jpg"),
        "Commute" => include_bytes!("../assets/moods/commute.jpg"),
        "Diwali" => include_bytes!("../assets/moods/diwali.jpg"),
        "Energize" => include_bytes!("../assets/moods/energize.jpg"),
        "Feel good" => include_bytes!("../assets/moods/feel_good.jpg"),
        "Focus" => include_bytes!("../assets/moods/focus.jpg"),
        "Gaming" => include_bytes!("../assets/moods/gaming.jpg"),
        "Party" => include_bytes!("../assets/moods/party.jpg"),
        "Romance" => include_bytes!("../assets/moods/romance.jpg"),
        "Sad" => include_bytes!("../assets/moods/sad.jpg"),
        "Sleep" => include_bytes!("../assets/moods/sleep.jpg"),
        "Workout" => include_bytes!("../assets/moods/workout.jpg"),
        _ => return None,
    };
    let id = Id::new(("embedded-mood-tex", name));
    ctx.data(|d| d.get_temp::<TextureHandle>(id)).or_else(|| {
        let img = image::load_from_memory(bytes).ok()?.to_rgba8();
        let size = [img.width() as usize, img.height() as usize];
        let color_image = ColorImage::from_rgba_unmultiplied(size, &img.into_raw());
        let handle = ctx.load_texture(format!("mood-{name}"), color_image, TextureOptions::LINEAR);
        ctx.data_mut(|d| d.insert_temp(id, handle.clone()));
        Some(handle)
    })
}

fn get_embedded_genre_texture(ctx: &egui::Context, name: &str) -> Option<TextureHandle> {
    let bytes: &'static [u8] = match name {
        "African" => include_bytes!("../assets/moods/african.jpg"),
        "Arabic" => include_bytes!("../assets/moods/arabic.jpg"),
        "Bengali" | "Gujarati" | "Hindi" | "Marathi" => include_bytes!("../assets/moods/bengali.jpg"),
        "Bhojpuri" => include_bytes!("../assets/moods/diwali.jpg"),
        "Carnatic classical" | "Classical" | "Hindustani classical" => include_bytes!("../assets/moods/focus.jpg"),
        "Country & Americana" => include_bytes!("../assets/moods/party.jpg"),
        "Dance & electronic" | "Indian pop" => include_bytes!("../assets/moods/energize.jpg"),
        "Decades" | "Kannada" | "Malayalam" => include_bytes!("../assets/moods/decades.jpg"),
        "Desi hip-hop" | "Haryanvi" | "Hip-hop" => include_bytes!("../assets/moods/commute.jpg"),
        "Devotional" => include_bytes!("../assets/moods/devotional.jpg"),
        "Family" => include_bytes!("../assets/moods/family.jpg"),
        "Folk & acoustic" => include_bytes!("../assets/moods/folk.jpg"),
        "Ghazal/sufi" => include_bytes!("../assets/moods/chill.jpg"),
        "Indian indie" | "Indie & alternative" => include_bytes!("../assets/moods/indian_indie.jpg"),
        "J-Pop" => include_bytes!("../assets/moods/jpop.jpg"),
        "Jazz" => include_bytes!("../assets/moods/jazz.jpg"),
        "K-Pop" => include_bytes!("../assets/moods/kpop.jpg"),
        "Latin" => include_bytes!("../assets/moods/latin.jpg"),
        "Metal" => include_bytes!("../assets/moods/metal.jpg"),
        _ => return None,
    };
    let id = Id::new(("embedded-genre-tex", name));
    ctx.data(|d| d.get_temp::<TextureHandle>(id)).or_else(|| {
        let img = image::load_from_memory(bytes).ok()?.to_rgba8();
        let size = [img.width() as usize, img.height() as usize];
        let color_image = ColorImage::from_rgba_unmultiplied(size, &img.into_raw());
        let handle = ctx.load_texture(format!("genre-{name}"), color_image, TextureOptions::LINEAR);
        ctx.data_mut(|d| d.insert_temp(id, handle.clone()));
        Some(handle)
    })
}

fn draw_mood_peeking_vinyl(
    ui: &Ui,
    art: &Art,
    url: &str,
    embedded_tex_id: Option<TextureId>,
    disc_center: Pos2,
    radius: f32,
    angle: f32,
    c_rect: Rect,
    rounding: f32,
    mood_color: Color32,
) {
    let bbox_min = pos2(
        (disc_center.x - radius).max(c_rect.left()),
        (disc_center.y - radius).max(c_rect.top()),
    );
    let bbox_max = pos2(
        (disc_center.x + radius).min(c_rect.right()),
        (disc_center.y + radius).min(c_rect.bottom()),
    );
    if bbox_min.x >= bbox_max.x || bbox_min.y >= bbox_max.y {
        return;
    }

    let cols = 22;
    let rows = 22;
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    let fade_start = disc_center.x - radius * 0.95;
    let fade_end = disc_center.x + radius * 0.15;

    let pic_id = if let Some(tid) = embedded_tex_id {
        Some(tid)
    } else if !url.is_empty() {
        art.get(ui.ctx(), url, radius * 2.0).map(|p| p.id)
    } else {
        None
    };

    let mut mesh = if let Some(tid) = pic_id {
        egui::Mesh::with_texture(tid)
    } else {
        egui::Mesh::default()
    };

    let mut verts = Vec::with_capacity((cols + 1) * (rows + 1));
    for iy in 0..=rows {
        let ty = iy as f32 / rows as f32;
        let y = bbox_min.y + (bbox_max.y - bbox_min.y) * ty;
        for ix in 0..=cols {
            let tx = ix as f32 / cols as f32;
            let x = bbox_min.x + (bbox_max.x - bbox_min.x) * tx;
            let p = pos2(x, y);

            let d = (p - disc_center).length();
            let inside = inside_rounded_rect(p, c_rect, rounding) && d <= radius;

            let alpha = if inside {
                let fade = ((p.x - fade_start) / (fade_end - fade_start)).clamp(0.0, 1.0).powf(1.3);
                let edge_aa = ((radius - d) / 1.5).clamp(0.0, 1.0);
                fade * edge_aa
            } else {
                0.0
            };

            let rx = (p.x - disc_center.x) * cos_a - (p.y - disc_center.y) * sin_a;
            let ry = (p.x - disc_center.x) * sin_a + (p.y - disc_center.y) * cos_a;
            let uv = pos2(
                (0.5 + 0.5 * (rx / radius)).clamp(0.0, 1.0),
                (0.5 + 0.5 * (ry / radius)).clamp(0.0, 1.0),
            );

            let col = if pic_id.is_some() {
                Color32::from_white_alpha((alpha * 255.0) as u8)
            } else {
                mood_color.gamma_multiply(alpha * 0.55)
            };

            verts.push((p, uv, col, alpha > 0.001));
        }
    }

    let base_idx = mesh.vertices.len() as u32;
    for (p, uv, col, _) in &verts {
        mesh.vertices.push(egui::epaint::Vertex {
            pos: *p,
            uv: *uv,
            color: *col,
        });
    }

    let stride = (cols + 1) as u32;
    for iy in 0..rows {
        for ix in 0..cols {
            let i0 = base_idx + (iy as u32) * stride + (ix as u32);
            let i1 = i0 + 1;
            let i2 = i0 + stride;
            let i3 = i2 + 1;

            let v_idx0 = (iy * (cols + 1) + ix) as usize;
            let v_idx1 = v_idx0 + 1;
            let v_idx2 = v_idx0 + (cols + 1);
            let v_idx3 = v_idx2 + 1;

            if verts[v_idx0].3 || verts[v_idx1].3 || verts[v_idx2].3 || verts[v_idx3].3 {
                mesh.indices.extend_from_slice(&[i0, i1, i3, i0, i3, i2]);
            }
        }
    }

    let painter = ui.painter().with_clip_rect(c_rect);
    painter.add(Shape::mesh(mesh));

    // Vinyl center spindle hole (only if inside c_rect)
    if inside_rounded_rect(disc_center, c_rect, rounding) {
        painter.circle_filled(disc_center, 4.0, Color32::from_rgb(18, 19, 24));
        painter.circle_stroke(disc_center, 4.0, Stroke::new(1.0_f32, veil(0.35)));
    }
}

fn draw_tilted_artwork(
    ui: &Ui,
    art: &Art,
    url: &str,
    embedded_tex_id: Option<TextureId>,
    center: Pos2,
    size: f32,
    angle: f32,
    border_color: Option<Color32>,
) {
    let half = size * 0.5;
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    let rot = |dx: f32, dy: f32| -> Pos2 {
        pos2(center.x + dx * cos_a - dy * sin_a, center.y + dx * sin_a + dy * cos_a)
    };

    let p0 = rot(-half, -half);
    let p1 = rot(half, -half);
    let p2 = rot(half, half);
    let p3 = rot(-half, half);

    let pic_id = if let Some(tid) = embedded_tex_id {
        Some(tid)
    } else if !url.is_empty() {
        art.get(ui.ctx(), url, size).map(|p| p.id)
    } else {
        None
    };

    if let Some(tid) = pic_id {
        let mut mesh = egui::Mesh::with_texture(tid);
        mesh.vertices.push(egui::epaint::Vertex { pos: p0, uv: pos2(0.0, 0.0), color: Color32::WHITE });
        mesh.vertices.push(egui::epaint::Vertex { pos: p1, uv: pos2(1.0, 0.0), color: Color32::WHITE });
        mesh.vertices.push(egui::epaint::Vertex { pos: p2, uv: pos2(1.0, 1.0), color: Color32::WHITE });
        mesh.vertices.push(egui::epaint::Vertex { pos: p3, uv: pos2(0.0, 1.0), color: Color32::WHITE });
        mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
        ui.painter().add(Shape::mesh(mesh));
    } else {
        let mut mesh = egui::Mesh::default();
        let bg = border_color.unwrap_or(Color32::from_rgb(60, 60, 75)).gamma_multiply(0.3);
        mesh.vertices.push(egui::epaint::Vertex { pos: p0, uv: Pos2::ZERO, color: bg });
        mesh.vertices.push(egui::epaint::Vertex { pos: p1, uv: Pos2::ZERO, color: bg });
        mesh.vertices.push(egui::epaint::Vertex { pos: p2, uv: Pos2::ZERO, color: bg });
        mesh.vertices.push(egui::epaint::Vertex { pos: p3, uv: Pos2::ZERO, color: bg });
        mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
        ui.painter().add(Shape::mesh(mesh));
    }

    if let Some(stroke_col) = border_color {
        let stroke = Stroke::new(1.2_f32, stroke_col);
        ui.painter().line_segment([p0, p1], stroke);
        ui.painter().line_segment([p1, p2], stroke);
        ui.painter().line_segment([p2, p3], stroke);
        ui.painter().line_segment([p3, p0], stroke);
    }
}

fn bars(ui: &Ui, center: Pos2, moving: bool) {
    let accent = ui.ctx().data(|d| d.get_temp(Id::new("app-theme-accent"))).unwrap_or(ACCENT);
    let time = if moving { ui.input(|input| input.time) } else { 0.9 };
    for bar in 0..3 {
        let wave = (time * (5.2 + bar as f64 * 1.9) + bar as f64 * 2.1).sin() as f32;
        let x = center.x + (bar as f32 - 1.0) * 5.0;
        let top = center.y + 7.0 - (4.0 + 5.0 * (1.0 + wave));
        ui.painter()
            .rect_filled(Rect::from_min_max(pos2(x - 1.5, top), pos2(x + 1.5, center.y + 7.0)), 1.0, accent);
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
    let tone = tint.lerp_to_gamma(TEXT, hover * 0.5);
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
    let accent = ui.ctx().data(|d| d.get_temp(Id::new("app-theme-accent"))).unwrap_or(ACCENT);
    if menu_row(ui, icon::BROADCAST, glyph(17.0), accent, "Start radio").clicked() {
        choice = Some(Action::StartRadio(item.clone()));
    }
    if menu_row(ui, icon::ARROW_BEND_DOWN_RIGHT, glyph(17.0), MUTED, "Play next").clicked() {
        choice = Some(Action::PlayNext(item.clone()));
    }
    if menu_row(ui, icon::LIST_PLUS, glyph(17.0), MUTED, "Add to queue").clicked() {
        choice = Some(Action::Enqueue(item.clone()));
    }
    let (font, tint, text) = if cx.is_liked(item) {
        (solid(17.0), accent, "Remove from Liked songs")
    } else {
        (glyph(17.0), MUTED, "Add to Liked songs")
    };
    if menu_row(ui, icon::HEART, font, tint, text).clicked() {
        choice = Some(Action::Like(item.clone()));
    }
    if menu_row(ui, icon::THUMBS_DOWN, glyph(17.0), MUTED, "Dislike song").clicked() {
        choice = Some(Action::Dislike(item.clone()));
    }
    if menu_row(ui, icon::DOWNLOAD_SIMPLE, glyph(17.0), MUTED, "Download song (Opus/160k)").clicked() {
        choice = Some(Action::DownloadSong(item.clone()));
    }
    if menu_row(ui, icon::SHARE_NETWORK, glyph(17.0), MUTED, "Share").clicked() {
        choice = Some(Action::Share(item.clone()));
    }
    if !item.subtitle.is_empty() {
        let is_blocked = cx.blocked_artists.iter().any(|b| b.eq_ignore_ascii_case(&item.subtitle));
        let (icon_sym, tint, lbl) = if is_blocked {
            (icon::PROHIBIT, Color32::from_rgb(52, 211, 153), format!("Unblock artist ({})", item.subtitle))
        } else {
            (icon::PROHIBIT, Color32::from_rgb(239, 68, 68), format!("Block artist ({})", item.subtitle))
        };
        if menu_row(ui, icon_sym, glyph(17.0), tint, &lbl).clicked() {
            choice = Some(if is_blocked {
                Action::UnblockArtist(item.subtitle.clone())
            } else {
                Action::BlockArtist(item.subtitle.clone())
            });
        }
    }
    if !cx.playlists.is_empty() {
        menu_divider(ui);
        for pl in cx.playlists {
            let already_in = pl.songs.iter().any(|s| s.video_id == item.video_id);
            let (p_icon, p_label, p_color) = if already_in {
                (icon::CHECK, format!("In: {}", pl.name), accent)
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
    if !item.artist_id.is_empty() || !item.subtitle.is_empty() || !item.album_id.is_empty() {
        menu_divider(ui);
    }
    if !item.artist_id.is_empty() {
        if menu_row(ui, icon::MICROPHONE_STAGE, glyph(17.0), MUTED, "Go to artist").clicked() {
            choice = Some(Action::Go(Route::Browse(item.artist_id.clone())));
        }
    } else if !item.subtitle.is_empty() {
        if menu_row(ui, icon::MICROPHONE_STAGE, glyph(17.0), MUTED, "Go to artist").clicked() {
            choice = Some(Action::Go(Route::Search(item.subtitle.clone())));
        }
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
    let accent = ui.ctx().data(|d| d.get_temp(Id::new("app-theme-accent"))).unwrap_or(ACCENT);
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
                (solid(16.0), accent)
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

    let title = if playing { accent } else { TEXT };
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
    let accent = ui.ctx().data(|d| d.get_temp(Id::new("app-theme-accent"))).unwrap_or(ACCENT);
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
            ui.painter().circle_filled(badge, 22.0, accent.gamma_multiply(lit));
            let symbol = if playing && !cx.paused { icon::PAUSE } else { icon::PLAY };
            let icon_color = contrast_text_for(accent);
            ui.painter()
                .text(badge, Align2::CENTER_CENTER, symbol, solid(19.0), icon_color.gamma_multiply(lit));
        }
    }
    if response.has_focus() {
        ui.painter().rect_stroke(art, radius, Stroke::new(1.5_f32, TEXT), StrokeKind::Inside);
    }
    let title = if playing { accent } else { TEXT };
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
    let accent = ui.ctx().data(|d| d.get_temp(Id::new("app-theme-accent"))).unwrap_or(ACCENT);
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
    let title = if playing { accent } else { TEXT };
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
    let (cells, cell_width) = columns(width, COMPACT_MIN, 12.0);
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
            let is_quick_picks = title.eq_ignore_ascii_case("quick picks") || title == QUICK_PICKS;
            if is_quick_picks {
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
            } else {
                let cell = vec2(cell_width, 64.0);
                grid(ui, shown, cells, cell, vec2(12.0, 2.0), |ui, index, rect| {
                    compact(ui, cx, items, index, rect, id.with(index))
                });
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

/// Paints a 2x2 cover mosaic from real track thumbnails, replacing the baked
/// composite YouTube serves for playlists (which contains placeholder tiles
/// for tracks whose artwork is unavailable).
fn paint_hero_mosaic(ui: &Ui, art: &Art, cells: &[String], rect: Rect) {
    fn tile(ui: &Ui, art: &Art, url: &str, cell: Rect, nw: u8, ne: u8, sw: u8, se: u8) {
        match art.get(ui.ctx(), url, cell.width()) {
            Some(picture) => egui::Image::from_texture((picture.id, cell.size()))
                .corner_radius(egui::CornerRadius { nw, ne, sw, se })
                .paint_at(ui, cell),
            None => paint_shimmer_wave(ui.painter(), ui.ctx(), cell, 0.0),
        }
    }
    let r: u8 = 12;
    let half_w = rect.width() / 2.0;
    let half_h = rect.height() / 2.0;
    match cells.len() {
        2 => {
            let left = Rect::from_min_max(rect.min, pos2(rect.left() + half_w, rect.bottom()));
            let right = Rect::from_min_max(pos2(rect.left() + half_w, rect.top()), rect.max);
            tile(ui, art, &cells[0], left, r, 0, r, 0);
            tile(ui, art, &cells[1], right, 0, r, 0, r);
        }
        3 => {
            let tl = Rect::from_min_max(rect.min, pos2(rect.left() + half_w, rect.top() + half_h));
            let tr = Rect::from_min_max(pos2(rect.left() + half_w, rect.top()), pos2(rect.right(), rect.top() + half_h));
            let bottom = Rect::from_min_max(pos2(rect.left(), rect.top() + half_h), rect.max);
            tile(ui, art, &cells[0], tl, r, 0, 0, 0);
            tile(ui, art, &cells[1], tr, 0, r, 0, 0);
            tile(ui, art, &cells[2], bottom, 0, 0, r, r);
        }
        _ => {
            let tl = Rect::from_min_max(rect.min, pos2(rect.left() + half_w, rect.top() + half_h));
            let tr = Rect::from_min_max(pos2(rect.left() + half_w, rect.top()), pos2(rect.right(), rect.top() + half_h));
            let bl = Rect::from_min_max(pos2(rect.left(), rect.top() + half_h), pos2(rect.left() + half_w, rect.bottom()));
            let br = Rect::from_min_max(pos2(rect.left() + half_w, rect.top() + half_h), rect.max);
            tile(ui, art, &cells[0], tl, r, 0, 0, 0);
            tile(ui, art, &cells[1], tr, 0, r, 0, 0);
            tile(ui, art, &cells[2], bl, 0, 0, r, 0);
            tile(ui, art, &cells[3], br, 0, 0, 0, r);
        }
    }
}

fn hero(
    ui: &mut Ui,
    cx: &mut Cx,
    item: &Item,
    symbol: Option<&str>,
    playable: bool,
    pinned: Option<bool>,
    mosaic: &[String],
) -> bool {
    let accent = ui.ctx().data(|d| d.get_temp(Id::new("app-theme-accent"))).unwrap_or(ACCENT);
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), HERO_ART), Sense::hover());
    let art = Rect::from_min_size(rect.min, Vec2::splat(HERO_ART));
    match symbol {
        Some(symbol) => {
            ui.painter().rect_filled(art, 12.0, veil(0.06));
            ui.painter().text(art.center(), Align2::CENTER_CENTER, symbol, solid(84.0), accent);
        }
        None => {
            if !item.is_artist && mosaic.len() >= 2 {
                paint_hero_mosaic(ui, cx.art, mosaic, art);
            } else {
                artwork(ui, cx.art, &item.thumb, art, if item.is_artist { HERO_ART / 2.0 } else { 12.0 });
            }
        }
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
            (solid(19.0), accent, "Remove from library")
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
    if hero(ui, cx, &header, Some(symbol), !items.is_empty(), None, &[]) {
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
    if hero(ui, cx, &header, Some(icon::PLAYLIST), !pl.songs.is_empty(), None, &[]) {
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
                // Rebuild playlist covers locally from real track thumbnails;
                // YouTube's served composite bakes in placeholder tiles for
                // tracks whose artwork is unavailable. Cells that simply reuse
                // the header art (albums, singles) are skipped so those pages
                // keep their single cover.
                let mut mosaic: Vec<String> = Vec::new();
                if !header.is_artist {
                    for item in page.sections.iter().flat_map(|section| &section.items) {
                        let thumb = &item.thumb;
                        if !thumb.is_empty() && thumb != &header.thumb && !mosaic.contains(thumb) {
                            mosaic.push(thumb.clone());
                            if mosaic.len() == 4 {
                                break;
                            }
                        }
                    }
                    if mosaic.len() < 2 {
                        mosaic.clear();
                    }
                }
                if hero(ui, cx, header, None, playable, Some(saved), &mosaic) {
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

#[allow(dead_code)]
fn horizontal_wash(painter: &egui::Painter, rect: Rect, left: Color32, right: Color32) {
    if left == right {
        return;
    }
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), left);
    mesh.colored_vertex(rect.right_top(), right);
    mesh.colored_vertex(rect.left_bottom(), left);
    mesh.colored_vertex(rect.right_bottom(), right);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(1, 2, 3);
    painter.add(Shape::mesh(mesh));
}

fn horizontal_wash_3(painter: &egui::Painter, rect: Rect, left: Color32, mid: Color32, right: Color32) {
    let mut mesh = egui::Mesh::default();
    let mid_x = rect.left() + rect.width() * 0.38;
    mesh.colored_vertex(rect.left_top(), left);
    mesh.colored_vertex(pos2(mid_x, rect.top()), mid);
    mesh.colored_vertex(rect.left_bottom(), left);
    mesh.colored_vertex(pos2(mid_x, rect.bottom()), mid);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(1, 2, 3);

    mesh.colored_vertex(rect.right_top(), right);
    mesh.colored_vertex(rect.right_bottom(), right);
    mesh.add_triangle(1, 4, 3);
    mesh.add_triangle(4, 3, 5);
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

#[allow(dead_code)]
fn draw_rotating_vinyl(painter: &Painter, center: Pos2, radius: f32, angle: f32, mood_color: Color32) {
    painter.circle_filled(center, radius, Color32::from_rgb(14, 15, 18));
    painter.circle_stroke(center, radius, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 30)));

    painter.circle_stroke(center, radius * 0.85, Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 16)));
    painter.circle_stroke(center, radius * 0.70, Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 14)));
    painter.circle_stroke(center, radius * 0.55, Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 12)));

    let label_radius = radius * 0.42;
    painter.circle_filled(center, label_radius, mood_color.gamma_multiply(0.80));

    let p_line = center + vec2(angle.cos(), angle.sin()) * (label_radius - 2.0);
    painter.line_segment([center, p_line], Stroke::new(1.5_f32, Color32::WHITE));

    painter.circle_filled(center, 5.0, Color32::BLACK);
    painter.circle_stroke(center, 5.0, Stroke::new(1.0_f32, Color32::from_rgb(180, 180, 190)));
}

fn home_page_view(ui: &mut Ui, cx: &mut Cx, saved: &crate::app::Saved, home_load: Option<&Load>) {
    let width = ui.available_width();
    let is_light = saved.theme_mode == ThemeMode::Light;
    let t_color = if is_light { Color32::from_rgb(24, 28, 36) } else { TEXT };
    let m_color = if is_light { Color32::from_rgb(100, 110, 125) } else { MUTED };
    let pal_color = saved.theme_palette.color();

    #[cfg(windows)]
    let hour = {
        let mut st = std::mem::MaybeUninit::<windows_sys::Win32::Foundation::SYSTEMTIME>::zeroed();
        unsafe {
            windows_sys::Win32::System::SystemInformation::GetLocalTime(st.as_mut_ptr());
            st.assume_init().wHour
        }
    };
    #[cfg(not(windows))]
    let hour = {
        let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        ((secs / 3600) % 24) as u16
    };
    let greeting = if hour >= 5 && hour < 12 {
        "Good morning"
    } else if hour >= 12 && hour < 17 {
        "Good afternoon"
    } else {
        "Good evening"
    };

    ui.add_space(4.0);
    let (greet_row, _) = ui.allocate_exact_size(vec2(width, 58.0), Sense::hover());
    label(ui, pos2(greet_row.left(), greet_row.center().y), Align2::LEFT_CENTER, greeting, bold(44.0), t_color, width);
    ui.add_space(16.0);

    const CATEGORIES: [&str; 11] = [
        "All", "Podcasts", "Workout", "Feel good", "Energize", "Relax", "Romance", "Sad", "Party", "Commute", "Sleep",
    ];
    let (chip_row, _) = ui.allocate_exact_size(vec2(width, 34.0), Sense::hover());
    let mut chip_x = chip_row.left();
    let active_filter = saved.home_filter.as_str();

    for &cat in &CATEGORIES {
        let is_active = active_filter.eq_ignore_ascii_case(cat) || (active_filter.is_empty() && cat == "All");
        let text_w = 16.0 + cat.len() as f32 * 7.5;
        let chip_rect = Rect::from_min_size(pos2(chip_x, chip_row.top()), vec2(text_w, 30.0));
        let resp = ui.interact(chip_rect, Id::new(("home-cat-chip", cat)), Sense::click());
        let hover = hover_of(ui, &resp);

        if is_active {
            let active_text_col = contrast_text_for(pal_color);
            ui.painter().rect_filled(chip_rect, 15.0, pal_color);
            if active_text_col != Color32::WHITE {
                ui.painter().rect_stroke(chip_rect, 15.0, Stroke::new(1.0_f32, Color32::from_rgb(160, 160, 170)), StrokeKind::Inside);
            }
            ui.painter().text(chip_rect.center(), Align2::CENTER_CENTER, cat, bold(12.5), active_text_col);
        } else {
            ui.painter().rect_filled(chip_rect, 15.0, veil(if is_light { 0.08 + 0.05 * hover } else { 0.05 + 0.05 * hover }));
            ui.painter().rect_stroke(chip_rect, 15.0, Stroke::new(1.0_f32, veil(0.08)), StrokeKind::Inside);
            ui.painter().text(chip_rect.center(), Align2::CENTER_CENTER, cat, medium(12.5), if hover > 0.1 { t_color } else { m_color });
        }

        if resp.clicked() {
            cx.out.push(Action::SetHomeFilter(cat.to_owned()));
        }
        chip_x += text_w + 8.0;
        if chip_x > chip_row.right() - 50.0 {
            break;
        }
    }
    ui.add_space(22.0);

    page(ui, cx, home_load, &saved.pinned, None);
}

fn search_discovery_view(query: &mut String, focus_search: &mut bool, saved: &crate::app::Saved, ui: &mut Ui, cx: &mut Cx) {
    let width = ui.available_width();
    let is_light = saved.theme_mode == ThemeMode::Light;
    let t_color = if is_light { Color32::from_rgb(24, 28, 36) } else { TEXT };
    let m_color = if is_light { Color32::from_rgb(100, 110, 125) } else { MUTED };
    let pal_color = saved.theme_palette.color();

    ui.add_space(4.0);
    let search_bar_h = 46.0;
    let (bar_rect, _) = ui.allocate_exact_size(vec2(width, search_bar_h), Sense::hover());
    let field = bar_rect.shrink2(vec2(2.0, 1.0));
    let mid = field.center().y;

    let background = ui.painter().add(Shape::Noop);
    ui.painter().text(pos2(field.left() + 22.0, mid), Align2::CENTER_CENTER, icon::MAGNIFYING_GLASS, glyph(18.0), m_color);

    let input_rect = Rect::from_min_max(pos2(field.left() + 46.0, field.top()), pos2(field.right() - 80.0, field.bottom()));
    let response = ui.scope_builder(UiBuilder::new().max_rect(input_rect).layout(egui::Layout::left_to_right(Align::Center)), |ui| {
        let edit = egui::TextEdit::singleline(query)
            .id(Id::new("discovery-search-edit"))
            .frame(false)
            .margin(Margin::ZERO)
            .desired_width(input_rect.width())
            .font(sans(15.0))
            .text_color(t_color)
            .hint_text("Search songs, albums, artists, playlists...");
        ui.add(edit)
    }).inner;

    if std::mem::take(focus_search) {
        response.request_focus();
    }
    let query_trimmed = query.trim();
    if response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) && !query_trimmed.is_empty() {
        cx.out.push(Action::Go(Route::Search(query_trimmed.to_owned())));
    }
    let focus = ui.ctx().animate_bool_with_time(Id::new("search-disc-focus"), response.has_focus(), 0.12);
    let stroke_col = veil(if is_light { 0.14 } else { 0.08 }).lerp_to_gamma(pal_color, focus);
    ui.painter().set(
        background,
        RectShape::new(field, 23.0, veil(if is_light { 0.08 + 0.03 * focus } else { 0.06 + 0.03 * focus }), Stroke::new(1.0 + 0.5 * focus, stroke_col), StrokeKind::Inside),
    );
    if query.is_empty() {
        ui.painter().text(pos2(field.right() - 20.0, mid), Align2::RIGHT_CENTER, "Ctrl K", sans(12.0), FAINT);
    } else if icon_button(ui, Rect::from_center_size(pos2(field.right() - 24.0, mid), Vec2::splat(26.0)), Id::new("clear-disc-search"), icon::X, glyph(13.0), m_color, "Clear").clicked() {
        query.clear();
        response.request_focus();
    }
    ui.add_space(24.0);

    let (m_head, _) = ui.allocate_exact_size(vec2(width, 32.0), Sense::hover());
    ui.painter().text(pos2(m_head.left() + 10.0, m_head.center().y), Align2::CENTER_CENTER, icon::HEART, solid(18.0), pal_color);
    label(ui, pos2(m_head.left() + 26.0, m_head.center().y), Align2::LEFT_CENTER, "Moods & moments", bold(19.0), t_color, width - 40.0);
    ui.add_space(10.0);

    struct MoodDef {
        name: &'static str,
        color: Color32,
        thumb: &'static str,
    }
    const MOODS: [MoodDef; 12] = [
        MoodDef { name: "Chill", color: Color32::from_rgb(56, 189, 248), thumb: "rAuzuLfViDJUshG4WU2sHbQv6hFX6ZCnGpI-_ucalfi9RGDoZisfPx9iESN1--HSyeCmTc2YKfJO4wM" },
        MoodDef { name: "Commute", color: Color32::from_rgb(217, 119, 6), thumb: "SMpT_3hW0yjMrAoiZ3EhiGP45ettuyjdSxJ2dHShO1N8hg1EOkBtv0sBRApF4bsSmIZw-w4BGilxMc8" },
        MoodDef { name: "Diwali", color: Color32::from_rgb(168, 85, 247), thumb: "Ey-p72hOgNHfU0NxW6Ai1CD67kaxKeOll7agJg7G25BMcPlRNrkVpUn1rmFK_psiNdIAjueXype4o2WG" },
        MoodDef { name: "Energize", color: Color32::from_rgb(249, 115, 22), thumb: "eHXeI8ZwBw_VIoPjnmKuCs2TBuWMfLe2x3zrLQcnGsC1tfO5uzHBFD2dCi7ZhpcUlW8tS0NbE04hgUsU" },
        MoodDef { name: "Feel good", color: Color32::from_rgb(16, 185, 129), thumb: "5F28fEHuZr_xIDxhhrku9zYos278p9jDrxGs_OGHqRMVxmqmRRnqtOZkl9cuymSCOwx_ymD7pky-BQ" },
        MoodDef { name: "Focus", color: Color32::from_rgb(234, 179, 8), thumb: "Ql0IVQFj5EjQdzn4FJpbjnXlPMlmSWYtudKxhl_E_ZeCNDLPX7gHzBTk0tQQlDFGoxjrcVN53uWWf_8" },
        MoodDef { name: "Gaming", color: Color32::from_rgb(236, 72, 153), thumb: "falgLCbx514wdlBjaNsulnyLw0pPPP3tqoceRUKoI4RLcezgilNjc1L294fNRO0R1TvldgGWv20ggA" },
        MoodDef { name: "Party", color: Color32::from_rgb(139, 92, 246), thumb: "WhhEXIUv8zXrDDLaZmT1tOMmlls8JPOvu_mzwC5a1QNBEwOrBI-sFambyKy5lF4SpInsRDXbrYLB9Q" },
        MoodDef { name: "Romance", color: Color32::from_rgb(244, 63, 94), thumb: "NTW2SQcwpsRpnY5S6pyymw82zlXebkeTsfly0h_RHEZG6DO1FPWT7buULrkPNtAtwxJMkyLWIRw-p_4" },
        MoodDef { name: "Sad", color: Color32::from_rgb(99, 102, 241), thumb: "0K0J7aA-vQaLqc71ZqSoDsZR5x0dIvjzJYLQfCoYSiCH18dvLXnwFJ9UYmQ1nLyUSXGPglupcHBhBqid" },
        MoodDef { name: "Sleep", color: Color32::from_rgb(129, 140, 248), thumb: "uABch78iXzNqPW5ot6KpF6cqrVqhx1c3L7VGRGqZDoV-0ZkjkodzE2nZH5sk7hRK59dHNWvdv7eSkhbW" },
        MoodDef { name: "Workout", color: Color32::from_rgb(239, 68, 68), thumb: "vccnCaZw3ie3ITUhRsRxOUwikeMmlL_xWIA5ilyx3A1ydiLt54tCc2hCJ_h0EkHgwSbImiX_XhMC-g" },
    ];

    let cols = if width > 1150.0 { 5 } else if width > 880.0 { 4 } else if width > 620.0 { 3 } else { 2 };
    let gap = 16.0;
    let card_w = (width - gap * (cols as f32 - 1.0)) / cols as f32;
    let card_h = 96.0;
    let rounding = card_h * 0.5;

    let rows = (MOODS.len() + cols - 1) / cols;
    let (grid_rect, _) = ui.allocate_exact_size(vec2(width, rows as f32 * (card_h + gap)), Sense::hover());

    for (i, mood) in MOODS.iter().enumerate() {
        let col = i % cols;
        let row = i / cols;
        let c_rect = Rect::from_min_size(
            pos2(grid_rect.left() + col as f32 * (card_w + gap), grid_rect.top() + row as f32 * (card_h + gap)),
            vec2(card_w, card_h),
        );
        let resp = ui.interact(c_rect, Id::new(("mood-card", i)), Sense::click());
        let hover = hover_of(ui, &resp);

        // Pre-bundled offline embedded texture for zero latency and offline support
        let embedded_tex = get_embedded_mood_texture(ui.ctx(), mood.name);

        // Radiant gradient background tinted by mood cover's dominant color
        let col_left = mood.color.gamma_multiply(if is_light { 0.16 + 0.10 * hover } else { 0.15 + 0.12 * hover });
        let col_right = mood.color.gamma_multiply(if is_light { 0.35 + 0.16 * hover } else { 0.42 + 0.20 * hover });

        // Draw rounded background filled with smoothly blended horizontal gradient
        paint_rounded_gradient(ui.painter(), c_rect, rounding, col_left, col_right);

        label(ui, pos2(c_rect.left() + 24.0, c_rect.top() + 30.0), Align2::LEFT_CENTER, mood.name, bold(19.0), t_color, card_w - 110.0);
        let dash_rect = Rect::from_min_size(pos2(c_rect.left() + 24.0, c_rect.bottom() - 26.0), vec2(32.0, 4.0));
        ui.painter().rect_filled(dash_rect, 2.0, mood.color);

        // Enlarged vinyl disk positioned cleanly inside the right hemisphere of the capsule
        let vinyl_center = pos2(c_rect.right() - rounding, c_rect.center().y);
        let vinyl_radius = rounding - 2.0;

        // Smooth rotation state starting at 0.0 (upright) and only advancing when hovered
        let rot_id = Id::new(("mood-card-rot", i));
        let mut angle: f32 = ui.ctx().data(|d| d.get_temp(rot_id).unwrap_or(0.0));
        if hover > 0.0 {
            ui.ctx().request_repaint_after(Duration::from_millis(16));
            let dt = ui.input(|inp| inp.stable_dt).min(0.05);
            angle += dt * 0.70;
            if angle >= std::f32::consts::TAU {
                angle -= std::f32::consts::TAU;
            }
            ui.ctx().data_mut(|d| d.insert_temp(rot_id, angle));
        }

        if saved.perf_placeholder_mood_art {
            draw_mood_peeking_vinyl(ui, cx.art, "", None, vinyl_center, vinyl_radius, angle, c_rect, rounding, mood.color);
        } else {
            let img_url = format!("https://lh3.googleusercontent.com/{}={}", mood.thumb, "w120-h120-l75-rj");
            let tex_id = embedded_tex.as_ref().map(|t| t.id());
            draw_mood_peeking_vinyl(ui, cx.art, &img_url, tex_id, vinyl_center, vinyl_radius, angle, c_rect, rounding, mood.color);
        }

        // Draw capsule stroke with matching corner radius for a clean, unbroken capsule
        let stroke_col = if hover > 0.0 { mood.color } else { mood.color.gamma_multiply(0.28) };
        ui.painter().rect_stroke(c_rect, rounding, Stroke::new(if hover > 0.0 { 1.5_f32 } else { 1.0_f32 }, stroke_col), StrokeKind::Inside);

        if resp.clicked() {
            cx.out.push(Action::Go(Route::Search(format!("{} music", mood.name))));
        }
    }
    ui.add_space(28.0);

    let (g_head, _) = ui.allocate_exact_size(vec2(width, 32.0), Sense::hover());
    ui.painter().text(pos2(g_head.left() + 10.0, g_head.center().y), Align2::CENTER_CENTER, icon::GUITAR, solid(18.0), pal_color);
    label(ui, pos2(g_head.left() + 26.0, g_head.center().y), Align2::LEFT_CENTER, "Genres", bold(19.0), t_color, width - 40.0);
    ui.add_space(10.0);

    struct GenreDef {
        name: &'static str,
        color: Color32,
        thumb: &'static str,
    }
    const GENRES: [GenreDef; 30] = [
        GenreDef { name: "African", color: Color32::from_rgb(16, 185, 129), thumb: "5iEV4BXK_LQvlOnb4fiS_6mn9ux0nmS3MSkYoG7kePZjPXNxDQOGFTZBU707BIK78rk4TyPWt2k1swo" },
        GenreDef { name: "Arabic", color: Color32::from_rgb(249, 115, 22), thumb: "Ox_xJNYqzLkHYdPcmtdCAhnsmAXkgKTNuhM2HGcfaXEq3hnkQAVK7rhV7ZerPCcIQ4tpVIzBIL2YFmo" },
        GenreDef { name: "Bengali", color: Color32::from_rgb(234, 179, 8), thumb: "twtZoAVWxJUgUk62bPEPDuxqXPEkReKeIc_31xi2jNRagPLYzBG356LUbBMB_xHJfbWWphbLWaIcsRA" },
        GenreDef { name: "Bhojpuri", color: Color32::from_rgb(236, 72, 153), thumb: "Ey-p72hOgNHfU0NxW6Ai1CD67kaxKeOll7agJg7G25BMcPlRNrkVpUn1rmFK_psiNdIAjueXype4o2WG" },
        GenreDef { name: "Carnatic classical", color: Color32::from_rgb(217, 119, 6), thumb: "Ql0IVQFj5EjQdzn4FJpbjnXlPMlmSWYtudKxhl_E_ZeCNDLPX7gHzBTk0tQQlDFGoxjrcVN53uWWf_8" },
        GenreDef { name: "Classical", color: Color32::from_rgb(148, 163, 184), thumb: "Ql0IVQFj5EjQdzn4FJpbjnXlPMlmSWYtudKxhl_E_ZeCNDLPX7gHzBTk0tQQlDFGoxjrcVN53uWWf_8" },
        GenreDef { name: "Country & Americana", color: Color32::from_rgb(59, 130, 246), thumb: "WhhEXIUv8zXrDDLaZmT1tOMmlls8JPOvu_mzwC5a1QNBEwOrBI-sFambyKy5lF4SpInsRDXbrYLB9Q" },
        GenreDef { name: "Dance & electronic", color: Color32::from_rgb(6, 182, 212), thumb: "eHXeI8ZwBw_VIoPjnmKuCs2TBuWMfLe2x3zrLQcnGsC1tfO5uzHBFD2dCi7ZhpcUlW8tS0NbE04hgUsU" },
        GenreDef { name: "Decades", color: Color32::from_rgb(16, 185, 129), thumb: "NPNY9n7Klaje5alEv_toJoRrjG-vaD2T9hdsiZKIA23kDU0jKQHZBI66-geguNEGtVp-ZwXTYdV7gLw" },
        GenreDef { name: "Desi hip-hop", color: Color32::from_rgb(168, 85, 247), thumb: "SMpT_3hW0yjMrAoiZ3EhiGP45ettuyjdSxJ2dHShO1N8hg1EOkBtv0sBRApF4bsSmIZw-w4BGilxMc8" },
        GenreDef { name: "Devotional", color: Color32::from_rgb(245, 158, 11), thumb: "WlWKWCVEhM2Tn6oZuY69QLCtU6J6rC0r5gWLuRLXeMNqWha9yYJXV9frC3-SSVvBvHr9TdtYIb7HJQ" },
        GenreDef { name: "Family", color: Color32::from_rgb(56, 189, 248), thumb: "R3LjYWeO4nKV-3BvqP4ESUFndRuyEVD68DLOKX2KpY-VW-Ran80TCeo10TEjUESONK0MbbhgrroE7v1J" },
        GenreDef { name: "Folk & acoustic", color: Color32::from_rgb(34, 197, 94), thumb: "J0m3mhwQYktptEg8IouqUm8EuO3u_sm0drLpNQCdA1PHmLkdfVsZY64Qi15_uqyIAbh1Fp6ixgLZAOz8" },
        GenreDef { name: "Ghazal/sufi", color: Color32::from_rgb(14, 165, 233), thumb: "rAuzuLfViDJUshG4WU2sHbQv6hFX6ZCnGpI-_ucalfi9RGDoZisfPx9iESN1--HSyeCmTc2YKfJO4wM" },
        GenreDef { name: "Gujarati", color: Color32::from_rgb(249, 115, 22), thumb: "twtZoAVWxJUgUk62bPEPDuxqXPEkReKeIc_31xi2jNRagPLYzBG356LUbBMB_xHJfbWWphbLWaIcsRA" },
        GenreDef { name: "Haryanvi", color: Color32::from_rgb(59, 130, 246), thumb: "SMpT_3hW0yjMrAoiZ3EhiGP45ettuyjdSxJ2dHShO1N8hg1EOkBtv0sBRApF4bsSmIZw-w4BGilxMc8" },
        GenreDef { name: "Hindi", color: Color32::from_rgb(245, 158, 11), thumb: "twtZoAVWxJUgUk62bPEPDuxqXPEkReKeIc_31xi2jNRagPLYzBG356LUbBMB_xHJfbWWphbLWaIcsRA" },
        GenreDef { name: "Hindustani classical", color: Color32::from_rgb(236, 72, 153), thumb: "Ql0IVQFj5EjQdzn4FJpbjnXlPMlmSWYtudKxhl_E_ZeCNDLPX7gHzBTk0tQQlDFGoxjrcVN53uWWf_8" },
        GenreDef { name: "Hip-hop", color: Color32::from_rgb(239, 68, 68), thumb: "SMpT_3hW0yjMrAoiZ3EhiGP45ettuyjdSxJ2dHShO1N8hg1EOkBtv0sBRApF4bsSmIZw-w4BGilxMc8" },
        GenreDef { name: "Indian indie", color: Color32::from_rgb(139, 92, 246), thumb: "Kz_AUVlrDSAEQ405NwMs94RMNe_65iiEpuw1P5xJe1-cuOirWXMw_LLLeCRUlKH5xaFbSfwVp7r6-A" },
        GenreDef { name: "Indian pop", color: Color32::from_rgb(16, 185, 129), thumb: "eHXeI8ZwBw_VIoPjnmKuCs2TBuWMfLe2x3zrLQcnGsC1tfO5uzHBFD2dCi7ZhpcUlW8tS0NbE04hgUsU" },
        GenreDef { name: "Indie & alternative", color: Color32::from_rgb(20, 184, 166), thumb: "Kz_AUVlrDSAEQ405NwMs94RMNe_65iiEpuw1P5xJe1-cuOirWXMw_LLLeCRUlKH5xaFbSfwVp7r6-A" },
        GenreDef { name: "J-Pop", color: Color32::from_rgb(244, 63, 94), thumb: "Vry7iLCt6cTHbTEEKH4t3-_WgRqkSFvqHjRn0j3vY2VbFMUPfpzvoEWBK3XdlBB_aBm3OmKOdzUi-8A" },
        GenreDef { name: "Jazz", color: Color32::from_rgb(59, 130, 246), thumb: "vtO-yvOzP9gdAIxfReYHTP-cICOzdEidVbt34vYpAmLGwZJTJJTzrn64YYoO2Ll33MLP--e1ZSwfEOV2" },
        GenreDef { name: "K-Pop", color: Color32::from_rgb(168, 85, 247), thumb: "Bse0tDvaNbxhbcxhprmh-0KEs04SgHG4rDwqqIeaWVSL9bDcvTKi6RVMJvUUDSzSBQ_5DZN76NwkX3I" },
        GenreDef { name: "Kannada", color: Color32::from_rgb(245, 158, 11), thumb: "NPNY9n7Klaje5alEv_toJoRrjG-vaD2T9hdsiZKIA23kDU0jKQHZBI66-geguNEGtVp-ZwXTYdV7gLw" },
        GenreDef { name: "Latin", color: Color32::from_rgb(234, 179, 8), thumb: "gxO4oCQCYLwC2jMivZ9nLqEvWOoc73U8hPYt4ejSOk5gXmBgevuyYFCoODBOa754Yt65BeRE0oiqK5Q" },
        GenreDef { name: "Malayalam", color: Color32::from_rgb(16, 185, 129), thumb: "NPNY9n7Klaje5alEv_toJoRrjG-vaD2T9hdsiZKIA23kDU0jKQHZBI66-geguNEGtVp-ZwXTYdV7gLw" },
        GenreDef { name: "Marathi", color: Color32::from_rgb(236, 72, 153), thumb: "twtZoAVWxJUgUk62bPEPDuxqXPEkReKeIc_31xi2jNRagPLYzBG356LUbBMB_xHJfbWWphbLWaIcsRA" },
        GenreDef { name: "Metal", color: Color32::from_rgb(148, 163, 184), thumb: "9Q3Bg10TTEpckT8loX0eSPzkxAGiD8mIAw6MgKHI92MNHvSQk2KtbGjwIrNOMKpp5Qw1QHoen8WvO5ct" },
    ];

    let g_cols = if width > 980.0 { 5 } else if width > 760.0 { 4 } else if width > 520.0 { 3 } else { 2 };
    let g_gap = 12.0;
    let g_card_w = (width - g_gap * (g_cols as f32 - 1.0)) / g_cols as f32;
    let g_card_h = 74.0;

    let g_rows = (GENRES.len() + g_cols - 1) / g_cols;
    let (g_grid, _) = ui.allocate_exact_size(vec2(width, g_rows as f32 * (g_card_h + g_gap)), Sense::hover());

    for (i, genre) in GENRES.iter().enumerate() {
        let col = i % g_cols;
        let row = i / g_cols;
        let c_rect = Rect::from_min_size(
            pos2(g_grid.left() + col as f32 * (g_card_w + g_gap), g_grid.top() + row as f32 * (g_card_h + g_gap)),
            vec2(g_card_w, g_card_h),
        );
        let resp = ui.interact(c_rect, Id::new(("genre-capsule", i)), Sense::click());
        let hover = hover_of(ui, &resp);

        let hover_anim = ui.ctx().animate_value_with_time(Id::new(("genre-hover-tilt", i)), if hover > 0.0 { 1.0 } else { 0.0 }, 0.20);
        if hover > 0.0 || hover_anim > 0.01 {
            ui.ctx().request_repaint();
        }

        // Pre-bundled offline embedded texture for genre
        let embedded_g_tex = get_embedded_genre_texture(ui.ctx(), genre.name);

        let bg_tint = genre.color.gamma_multiply(if is_light { 0.08 + 0.06 * hover_anim } else { 0.07 + 0.07 * hover_anim });
        ui.painter().rect_filled(c_rect, 14.0, bg_tint);
        let stroke_col = if hover_anim > 0.05 { genre.color.gamma_multiply(0.40 + 0.60 * hover_anim) } else { veil(0.10) };
        ui.painter().rect_stroke(c_rect, 14.0, Stroke::new(if hover > 0.0 { 1.5_f32 } else { 1.0_f32 }, stroke_col), StrokeKind::Inside);

        ui.painter().circle_filled(pos2(c_rect.left() + 18.0, c_rect.center().y), 4.5, genre.color);
        label(ui, pos2(c_rect.left() + 30.0, c_rect.center().y), Align2::LEFT_CENTER, genre.name, medium(13.5), t_color, g_card_w - 95.0);

        let thumb_size = 50.0;
        let thumb_center = pos2(c_rect.right() - 34.0, c_rect.center().y);
        // Continuously tilted with subtle wobble on hover
        let tilt_angle = 0.20 + 0.08 * hover_anim;

        if saved.perf_placeholder_mood_art {
            let half = thumb_size * 0.5;
            let cos_a = tilt_angle.cos();
            let sin_a = tilt_angle.sin();
            let rot = |dx: f32, dy: f32| -> Pos2 {
                pos2(thumb_center.x + dx * cos_a - dy * sin_a, thumb_center.y + dx * sin_a + dy * cos_a)
            };
            let p0 = rot(-half, -half);
            let p1 = rot(half, -half);
            let p2 = rot(half, half);
            let p3 = rot(-half, half);
            let mut mesh = egui::Mesh::default();
            let bg = genre.color.gamma_multiply(0.20 + 0.20 * hover_anim);
            mesh.vertices.push(egui::epaint::Vertex { pos: p0, uv: Pos2::ZERO, color: bg });
            mesh.vertices.push(egui::epaint::Vertex { pos: p1, uv: Pos2::ZERO, color: bg });
            mesh.vertices.push(egui::epaint::Vertex { pos: p2, uv: Pos2::ZERO, color: bg });
            mesh.vertices.push(egui::epaint::Vertex { pos: p3, uv: Pos2::ZERO, color: bg });
            mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
            ui.painter().add(Shape::mesh(mesh));
            ui.painter().text(thumb_center, Align2::CENTER_CENTER, icon::MUSIC_NOTE, glyph(15.0), genre.color);
        } else {
            let img_url = format!("https://lh3.googleusercontent.com/{}={}", genre.thumb, "w120-h120-l75-rj");
            let border = if hover_anim > 0.05 { Some(genre.color) } else { Some(veil(0.20)) };
            let tex_id = embedded_g_tex.as_ref().map(|t| t.id());
            draw_tilted_artwork(ui, cx.art, &img_url, tex_id, thumb_center, thumb_size, tilt_angle, border);
        }

        if resp.clicked() {
            cx.out.push(Action::Go(Route::Search(genre.name.to_owned())));
        }
    }
}

fn library_page_view(ui: &mut Ui, cx: &mut Cx, saved: &crate::app::Saved) {
    let width = ui.available_width();
    let is_light = saved.theme_mode == ThemeMode::Light;
    let t_color = if is_light { Color32::from_rgb(24, 28, 36) } else { TEXT };
    let m_color = if is_light { Color32::from_rgb(100, 110, 125) } else { MUTED };
    let pal_color = saved.theme_palette.color();

    // 1. Top Header Row: "Library" title on left, "Import (EXPERIMENTAL)" & "+ New Playlist" on right
    let (head_rect, _) = ui.allocate_exact_size(vec2(width, 48.0), Sense::hover());
    label(ui, pos2(head_rect.left(), head_rect.center().y), Align2::LEFT_CENTER, "Library", bold(32.0), t_color, 200.0);

    // Top Right Buttons
    let mut right_x = head_rect.right();

    // "+ New Playlist" button
    let new_pl_w = 118.0;
    let new_pl_rect = Rect::from_min_size(pos2(right_x - new_pl_w, head_rect.center().y - 16.0), vec2(new_pl_w, 32.0));
    right_x -= new_pl_w + 10.0;
    let new_pl_resp = ui.interact(new_pl_rect, Id::new("lib-new-playlist-btn"), Sense::click());
    let new_pl_hov = hover_of(ui, &new_pl_resp);
    ui.painter().rect_filled(new_pl_rect, 16.0, veil(0.06 + 0.05 * new_pl_hov));
    ui.painter().rect_stroke(new_pl_rect, 16.0, Stroke::new(1.0_f32, veil(0.12 + 0.08 * new_pl_hov)), StrokeKind::Inside);
    ui.painter().text(pos2(new_pl_rect.left() + 16.0, new_pl_rect.center().y), Align2::CENTER_CENTER, icon::PLUS, solid(13.0), t_color);
    label(ui, pos2(new_pl_rect.left() + 28.0, new_pl_rect.center().y), Align2::LEFT_CENTER, "New Playlist", medium(12.5), t_color, 85.0);
    if new_pl_resp.clicked() {
        cx.out.push(Action::PromptCreatePlaylist);
    }

    // "Import EXPERIMENTAL" button
    let import_w = 184.0;
    let import_rect = Rect::from_min_size(pos2(right_x - import_w, head_rect.center().y - 16.0), vec2(import_w, 32.0));
    let import_resp = ui.interact(import_rect, Id::new("lib-import-btn"), Sense::click());
    let import_hov = hover_of(ui, &import_resp);
    ui.painter().rect_filled(import_rect, 16.0, veil(0.06 + 0.05 * import_hov));
    ui.painter().rect_stroke(import_rect, 16.0, Stroke::new(1.0_f32, veil(0.12 + 0.08 * import_hov)), StrokeKind::Inside);
    ui.painter().text(pos2(import_rect.left() + 16.0, import_rect.center().y), Align2::CENTER_CENTER, icon::VINYL_RECORD, glyph(13.0), t_color);
    label(ui, pos2(import_rect.left() + 28.0, import_rect.center().y), Align2::LEFT_CENTER, "Import", medium(12.5), t_color, 52.0);
    let exp_badge_rect = Rect::from_center_size(pos2(import_rect.right() - 44.0, import_rect.center().y), vec2(74.0, 18.0));
    ui.painter().rect_filled(exp_badge_rect, 9.0, Color32::from_rgba_unmultiplied(220, 38, 38, 36));
    ui.painter().rect_stroke(exp_badge_rect, 9.0, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(220, 38, 38, 120)), StrokeKind::Inside);
    ui.painter().text(exp_badge_rect.center(), Align2::CENTER_CENTER, "EXPERIMENTAL", bold(9.0), Color32::from_rgb(248, 113, 113));
    if import_resp.clicked() {
        cx.out.push(Action::BrowseStorageDir);
    }

    ui.add_space(14.0);

    // 2. Filter Chips: All, Playlists, Albums, Artists, Songs, Uploads, Local
    let filter_id = Id::new("library-active-filter-chip");
    let cur_filter_idx: usize = ui.data(|d| d.get_temp(filter_id)).unwrap_or(0);

    let filters: [(&str, &str); 7] = [
        (icon::SPARKLE, "All"),
        (icon::LIST, "Playlists"),
        (icon::VINYL_RECORD, "Albums"),
        (icon::USER, "Artists"),
        (icon::MUSIC_NOTE, "Songs"),
        (icon::CLOUD_ARROW_UP, "Uploads"),
        (icon::FOLDER, "Local"),
    ];

    let (chip_strip, _) = ui.allocate_exact_size(vec2(width, 36.0), Sense::hover());
    let mut c_x = chip_strip.left();
    for (idx, (sym, name)) in filters.iter().enumerate() {
        let is_active = idx == cur_filter_idx;
        let pill_w = 26.0 + name.len() as f32 * 7.5 + 16.0;
        let pill_rect = Rect::from_min_size(pos2(c_x, chip_strip.top()), vec2(pill_w, 32.0));
        let resp = ui.interact(pill_rect, Id::new(("lib-filter-pill", idx)), Sense::click());
        let hover = hover_of(ui, &resp);

        if is_active {
            ui.painter().rect_filled(pill_rect, 16.0, veil(0.14));
            ui.painter().rect_stroke(pill_rect, 16.0, Stroke::new(1.0_f32, pal_color), StrokeKind::Inside);
            ui.painter().text(pos2(pill_rect.left() + 14.0, pill_rect.center().y), Align2::CENTER_CENTER, *sym, glyph(12.5), pal_color);
            label(ui, pos2(pill_rect.left() + 26.0, pill_rect.center().y), Align2::LEFT_CENTER, *name, bold(12.0), Color32::WHITE, pill_w - 30.0);
        } else {
            ui.painter().rect_filled(pill_rect, 16.0, veil(0.04 + 0.04 * hover));
            ui.painter().rect_stroke(pill_rect, 16.0, Stroke::new(1.0_f32, veil(0.06)), StrokeKind::Inside);
            ui.painter().text(pos2(pill_rect.left() + 14.0, pill_rect.center().y), Align2::CENTER_CENTER, *sym, glyph(12.5), m_color);
            label(ui, pos2(pill_rect.left() + 26.0, pill_rect.center().y), Align2::LEFT_CENTER, *name, medium(12.0), if hover > 0.1 { t_color } else { m_color }, pill_w - 30.0);
        }

        if resp.clicked() {
            ui.data_mut(|d| d.insert_temp(filter_id, idx));
        }
        c_x += pill_w + 8.0;
    }
    ui.add_space(20.0);

    // 3. Filtered Content
    match cur_filter_idx {
        0 | 1 => {
            // Grid of Cards: "On Repeat" + Custom Playlists
            let card_size = 160.0;
            let card_gap = 18.0;
            let cols = ((width + card_gap) / (card_size + card_gap)).floor().max(2.0) as usize;

            let mut all_cards: Vec<(&str, String, String, Color32, Option<String>)> = Vec::new();
            // 1. "On Repeat" (Liked Songs)
            let liked_sub = format!("{} songs", saved.liked.len());
            all_cards.push(("On Repeat", liked_sub, String::new(), Color32::from_rgb(45, 24, 34), None));

            // 2. Custom Playlists
            for pl in &saved.playlists {
                let sub = format!("{} songs", pl.songs.len());
                let cover = pl.songs.first().map(|s| saved.covers.of(s).to_string());
                all_cards.push((&pl.name, sub, pl.id.clone(), Color32::from_rgb(26, 27, 36), cover));
            }

            let rows = (all_cards.len() + cols - 1) / cols;
            let (grid_rect, _) = ui.allocate_exact_size(vec2(width, rows as f32 * (card_size + 48.0 + card_gap)), Sense::hover());

            for (i, (title, subtitle, pl_id, bg_color, cover)) in all_cards.iter().enumerate() {
                let col = i % cols;
                let row = i / cols;
                let c_x = grid_rect.left() + col as f32 * (card_size + card_gap);
                let c_y = grid_rect.top() + row as f32 * (card_size + 48.0 + card_gap);
                let img_rect = Rect::from_min_size(pos2(c_x, c_y), Vec2::splat(card_size));
                let entire_card_rect = Rect::from_min_size(pos2(c_x, c_y), vec2(card_size, card_size + 44.0));
                let resp = ui.interact(entire_card_rect, Id::new(("lib-card", i)), Sense::click());
                let hover = hover_of(ui, &resp);

                // Square artwork / cover
                ui.painter().rect_filled(img_rect, 14.0, *bg_color);
                ui.painter().rect_stroke(img_rect, 14.0, Stroke::new(if hover > 0.0 { 1.5_f32 } else { 1.0_f32 }, if hover > 0.0 { pal_color } else { veil(0.08) }), StrokeKind::Inside);

                if let Some(c_url) = cover {
                    if !c_url.is_empty() {
                        artwork(ui, cx.art, c_url, img_rect, 14.0);
                    }
                } else if i == 0 {
                    // "On Repeat" glowing repeat icon
                    ui.painter().text(img_rect.center(), Align2::CENTER_CENTER, icon::ARROWS_CLOCKWISE, glyph(44.0), Color32::from_rgb(244, 63, 94));
                } else {
                    ui.painter().text(img_rect.center(), Align2::CENTER_CENTER, icon::LIST, glyph(36.0), pal_color);
                }

                // Play overlay on hover
                if hover > 0.0 {
                    let play_center = pos2(img_rect.right() - 26.0, img_rect.bottom() - 26.0);
                    ui.painter().circle_filled(play_center, 18.0, pal_color);
                    ui.painter().text(play_center, Align2::CENTER_CENTER, icon::PLAY, solid(14.0), Color32::WHITE);
                }

                // Title & Subtitle below card
                label(ui, pos2(c_x + 2.0, img_rect.bottom() + 12.0), Align2::LEFT_CENTER, title, bold(14.5), t_color, card_size - 4.0);
                label(ui, pos2(c_x + 2.0, img_rect.bottom() + 28.0), Align2::LEFT_CENTER, subtitle, sans(12.0), m_color, card_size - 4.0);

                if resp.clicked() {
                    if i == 0 {
                        // Switch to Songs filter to view or play liked songs
                        ui.data_mut(|d| d.insert_temp(filter_id, 4usize));
                    } else {
                        cx.out.push(Action::Go(Route::CustomPlaylist(pl_id.clone())));
                    }
                }
            }
        }
        4 => {
            // Songs Table (Liked tracks)
            let empty = ["No liked songs yet", "Tap the heart on any track to keep it here."];
            local(ui, cx, "Liked songs", icon::HEART, &saved.liked, empty, false);
        }
        3 => {
            // Artists View
            let mut artist_counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
            for s in &saved.liked {
                for (artist_name, _) in &s.artists {
                    *artist_counts.entry(artist_name.clone()).or_insert(0) += 1;
                }
                if s.artists.is_empty() && !s.subtitle.is_empty() {
                    *artist_counts.entry(s.subtitle.clone()).or_insert(0) += 1;
                }
            }
            if artist_counts.is_empty() {
                let empty = ["No artists in library", "Like some tracks to see your favorite artists here."];
                local(ui, cx, "Artists", icon::USER, &[], empty, false);
            } else {
                for (artist, count) in artist_counts {
                    let (row, r_resp) = ui.allocate_exact_size(vec2(width, 42.0), Sense::click());
                    let r_hov = hover_of(ui, &r_resp);
                    if r_hov > 0.0 {
                        ui.painter().rect_filled(row, 6.0, veil(0.06 * r_hov));
                    }
                    ui.painter().text(pos2(row.left() + 20.0, row.center().y), Align2::CENTER_CENTER, icon::USER, glyph(16.0), pal_color);
                    label(ui, pos2(row.left() + 44.0, row.center().y), Align2::LEFT_CENTER, &artist, bold(14.0), t_color, width - 120.0);
                    label(ui, pos2(row.right() - 20.0, row.center().y), Align2::RIGHT_CENTER, &format!("{} songs", count), sans(12.0), m_color, 80.0);
                    if r_resp.clicked() {
                        cx.out.push(Action::Go(Route::Search(artist)));
                    }
                }
            }
        }
        _ => {
            // Local / Albums / Uploads general info view
            let (info_rect, _) = ui.allocate_exact_size(vec2(width, 100.0), Sense::hover());
            ui.painter().rect_filled(info_rect, 12.0, veil(0.04));
            ui.painter().rect_stroke(info_rect, 12.0, Stroke::new(1.0_f32, veil(0.08)), StrokeKind::Inside);
            let note = "Storage directory: ".to_owned() + &saved.storage_dir;
            label(ui, pos2(info_rect.center().x, info_rect.center().y - 12.0), Align2::CENTER_CENTER, &note, medium(13.0), t_color, width - 40.0);
            label(ui, pos2(info_rect.center().x, info_rect.center().y + 12.0), Align2::CENTER_CENTER, "Configure and scan local files in Settings -> Data & storage", sans(12.0), m_color, width - 40.0);
        }
    }
}

fn draw_ambient_backdrop(
    ui: &Ui,
    app: &App,
    area: Rect,
    base_color: Color32,
    glow: Color32,
    header_thumb: Option<&str>,
    playing_cover: Option<&str>,
) {
    let disable_blurs = app.saved.performance_mode && app.saved.perf_disable_blurs;
    if disable_blurs || !app.saved.ambient_glow {
        return;
    }

    let active_cover = playing_cover.or(header_thumb);

    if app.saved.ambient_artwork_blur {
        match app.saved.artwork_bg_variant {
            crate::app::ArtworkBgVariant::CanvasTint => {
                let haze_h = 420.0_f32.min(area.height());
                let haze = Rect::from_min_size(area.min, vec2(area.width(), haze_h));
                wash(ui.painter(), haze, glow, base_color);
            }
            crate::app::ArtworkBgVariant::BlurredBackdrop => {
                let mut drew_texture = false;
                if let Some(cover_str) = active_cover {
                    if let Some(blur_pic) = app.art.get(ui.ctx(), cover_str, 1.0) {
                        let haze_h = 560.0_f32.min(area.height());
                        let haze = Rect::from_min_size(area.min, vec2(area.width(), haze_h));
                        let alpha = (app.saved.artwork_bg_opacity.clamp(0.05, 0.60) * 255.0) as u8;
                        let mut mesh = egui::Mesh::with_texture(blur_pic.id);
                        let tint_top = Color32::from_white_alpha(alpha);
                        let tint_mid = Color32::from_white_alpha((alpha as f32 * 0.45) as u8);
                        let tint_bot = Color32::from_white_alpha(0);

                        let mid_y = haze.top() + haze.height() * 0.60;
                        mesh.vertices.push(egui::epaint::Vertex { pos: haze.left_top(), uv: pos2(0.0, 0.0), color: tint_top });
                        mesh.vertices.push(egui::epaint::Vertex { pos: haze.right_top(), uv: pos2(1.0, 0.0), color: tint_top });
                        mesh.vertices.push(egui::epaint::Vertex { pos: pos2(haze.left(), mid_y), uv: pos2(0.0, 0.60), color: tint_mid });
                        mesh.vertices.push(egui::epaint::Vertex { pos: pos2(haze.right(), mid_y), uv: pos2(1.0, 0.60), color: tint_mid });
                        mesh.vertices.push(egui::epaint::Vertex { pos: haze.left_bottom(), uv: pos2(0.0, 1.0), color: tint_bot });
                        mesh.vertices.push(egui::epaint::Vertex { pos: haze.right_bottom(), uv: pos2(1.0, 1.0), color: tint_bot });

                        mesh.indices.extend_from_slice(&[0, 1, 2, 1, 3, 2, 2, 3, 4, 3, 5, 4]);
                        ui.painter().add(Shape::mesh(mesh));
                        drew_texture = true;
                    }
                }
                if !drew_texture {
                    let haze_h = 420.0_f32.min(area.height());
                    let haze = Rect::from_min_size(area.min, vec2(area.width(), haze_h));
                    wash(ui.painter(), haze, glow, base_color);
                }
            }
            crate::app::ArtworkBgVariant::AmbientRadialGlow => {
                let center = pos2(area.center().x, area.top() + 140.0);
                let radius = (area.width() * 0.38).clamp(240.0, 480.0);
                let pal_color = app.saved.theme_palette.color();
                let col_glow = glow.lerp_to_gamma(pal_color, 0.35);
                let col_center = Color32::from_rgba_unmultiplied(col_glow.r(), col_glow.g(), col_glow.b(), (app.saved.artwork_bg_opacity * 180.0) as u8);
                let col_edge = Color32::from_rgba_unmultiplied(base_color.r(), base_color.g(), base_color.b(), 0);

                let mut mesh = egui::Mesh::default();
                mesh.vertices.push(egui::epaint::Vertex { pos: center, uv: Pos2::ZERO, color: col_center });
                let steps = 28;
                for i in 0..=steps {
                    let angle = i as f32 * std::f32::consts::TAU / steps as f32;
                    let p = pos2(center.x + radius * 1.8 * angle.cos(), center.y + radius * 0.75 * angle.sin());
                    mesh.vertices.push(egui::epaint::Vertex { pos: p, uv: Pos2::ZERO, color: col_edge });
                }
                for i in 1..=steps {
                    mesh.indices.extend_from_slice(&[0, i, i + 1]);
                }
                ui.painter().add(Shape::mesh(mesh));
            }
            crate::app::ArtworkBgVariant::MinimalEdgeGlow => {
                let top_edge = Rect::from_min_size(area.min, vec2(area.width(), 32.0));
                let pal_color = app.saved.theme_palette.color();
                let col_top = Color32::from_rgba_unmultiplied(pal_color.r(), pal_color.g(), pal_color.b(), 55);
                let col_none = Color32::from_rgba_unmultiplied(base_color.r(), base_color.g(), base_color.b(), 0);
                wash(ui.painter(), top_edge, col_top, col_none);
                ui.painter().hline(area.x_range(), area.top(), Stroke::new(1.0_f32, pal_color.gamma_multiply(0.35)));
            }
        }
    } else {
        let haze_h = 420.0_f32.min(area.height());
        let haze = Rect::from_min_size(area.min, vec2(area.width(), haze_h));
        wash(ui.painter(), haze, glow, base_color);
    }
}

fn content(app: &mut App, ui: &mut Ui, out: &mut Vec<Action>) {
    let area = ui.max_rect();
    let base_color = app.saved.theme_base.color();
    let route = app.route().clone();
    if let Route::Settings(active_tab) = route {
        settings_header_and_tabs(ui, app, active_tab, out);
        let margin = Margin {
            left: PAGE_MARGIN,
            right: PAGE_MARGIN,
            top: 8,
            bottom: if app.saved.bar_layout != BarLayout::Docked { 96 } else { 56 },
        };
        let tab_id = Id::new("last-settings-active-tab");
        let prev_tab = ui.data(|d| d.get_temp::<SettingsTab>(tab_id));
        let mut scroll = egui::ScrollArea::vertical().id_salt(("settings-scroll", active_tab)).auto_shrink(false);
        if prev_tab != Some(active_tab) {
            ui.data_mut(|d| d.insert_temp(tab_id, active_tab));
            scroll = scroll.vertical_scroll_offset(0.0);
        }
        scroll.show(ui, |ui| {
            Frame::new().inner_margin(margin).show(ui, |ui| {
                ui.set_width(ui.available_width());
                settings_body(ui, app, active_tab, out);
            });
        });
        return;
    }

    let header_thumb = match app.pages.get(&route) {
        Some(Load::Ready(page)) => page.header.as_ref().map(|header| header.thumb.clone()),
        _ => None,
    };
    let playing_cover = app.now_playing().map(|item| app.saved.covers.of(item).to_owned());
    let picture = |source: Option<&str>, size: f32| source.and_then(|thumb| app.art.get(ui.ctx(), thumb, size));
    let tint = picture(header_thumb.as_deref(), HERO_ART).or_else(|| picture(playing_cover.as_deref(), BAR_ART)).map(|picture| picture.tint);
    let glow = mood(ui, "ambient", tint, base_color, 0.36);
    draw_ambient_backdrop(ui, app, area, base_color, glow, header_thumb.as_deref(), playing_cover.as_deref());

    let loading = matches!(app.pages.get(&route), Some(Load::Loading));
    let shown = entrance(ui, Id::new((&route, loading)));
    let playing_vid = app.now_playing().map(|item| item.video_id.clone()).unwrap_or_default();
    let is_single = matches!(route, Route::Search(_) | Route::Home);
    let paused = app.player.paused();
    let App {
        ref mut query,
        ref mut focus_search,
        ref art,
        ref saved,
        ref pages,
        ..
    } = *app;

    let mut cx = Cx {
        art,
        covers: &saved.covers,
        playing: &playing_vid,
        paused,
        liked: &saved.liked,
        single: is_single,
        playlists: &saved.playlists,
        blocked_artists: &saved.blocked_artists,
        out: &mut *out,
    };
    let margin = Margin {
        left: PAGE_MARGIN,
        right: PAGE_MARGIN,
        top: 8,
        bottom: if saved.bar_layout != BarLayout::Docked { 96 } else { 56 },
    };
    let route_id = Id::new("last-content-route");
    let prev_route = ui.data(|d| d.get_temp::<Route>(route_id));
    let route_changed = prev_route.as_ref() != Some(&route);
    if route_changed {
        ui.data_mut(|d| d.insert_temp(route_id, route.clone()));
    }
    let mut scroll_area = egui::ScrollArea::vertical().id_salt(&route).auto_shrink(false);
    if route_changed {
        scroll_area = scroll_area.vertical_scroll_offset(0.0);
    }
    scroll_area.show(ui, |ui| {
        Frame::new().inner_margin(margin).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_opacity(shown);
            match &route {
                Route::Liked => {
                    library_page_view(ui, &mut cx, saved);
                }
                Route::Recent => {
                    let empty = ["Nothing played yet", "Tracks you listen to are remembered here."];
                    local(
                        ui,
                        &mut cx,
                        "Recently played",
                        icon::CLOCK_COUNTER_CLOCKWISE,
                        &saved.recent,
                        empty,
                        true,
                    );
                }
                Route::CustomPlaylist(id) => {
                    if let Some(pl) = saved.playlists.iter().find(|p| &p.id == id) {
                        custom_playlist_page(ui, &mut cx, pl);
                    }
                }
                Route::Settings(_) => unreachable!(),
                Route::Search(q) => {
                    if q.trim().is_empty() {
                        search_discovery_view(query, focus_search, saved, ui, &mut cx);
                    } else {
                        page(ui, &mut cx, pages.get(&route), &saved.pinned, Some(q));
                    }
                }
                Route::Home => {
                    home_page_view(ui, &mut cx, saved, pages.get(&Route::Home));
                }
                Route::Browse(_) => page(ui, &mut cx, pages.get(&route), &saved.pinned, None),
                Route::Stats => stats_view(ui, &mut cx, saved),
            }
        });
    });
}

fn keycaps(ui: &Ui, right_center: Pos2, parts: &[String], tint: Color32) -> f32 {
    let mut right = right_center.x;
    for part in parts.iter().rev() {
        let galley = ui.painter().layout_no_wrap(part.clone(), bold(12.0), tint);
        let size = vec2((galley.size().x + 18.0).max(30.0), 28.0);
        let cap = Rect::from_min_size(pos2(right - size.x, right_center.y - size.y / 2.0), size);
        let (fill, edge) = if tint != TEXT && tint != MUTED && tint != FAINT {
            (tint.gamma_multiply(0.14), tint.gamma_multiply(0.6))
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
    let pal_color = app.saved.theme_palette.color();
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
        (true, _) => keycaps(ui, edge, &["Press keys\u{2026}".to_owned()], pal_color),
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
        let color = if taken && !recording { pal_color } else { MUTED };
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

// ---------- Unified settings kit: icon-first rows inside auto-height cards ----------

const ROW_PLAIN: f32 = 44.0;
const ROW_DESC: f32 = 62.0;

/// Centers settings content in the standard 740px column like the old cards did.
fn settings_column(ui: &mut Ui, body: impl FnOnce(&mut Ui)) {
    let full = ui.available_width();
    let width = full.min(SETTINGS_WIDTH);
    let pad = (full - width).max(0.0) / 2.0;
    ui.horizontal(|ui| {
        if pad > 0.0 {
            ui.add_space(pad);
        }
        ui.vertical(|ui| {
            ui.set_width(width);
            body(ui);
        });
    });
}

/// Paints the icon + title (+ description) half of a settings row and returns its rect.
fn row_base(ui: &mut Ui, sym: &str, title: &str, desc: &str) -> Rect {
    let full = ui.available_width();
    let height = if desc.is_empty() { ROW_PLAIN } else { ROW_DESC };
    let (row, _) = ui.allocate_exact_size(vec2(full, height), Sense::hover());
    let title_y = if desc.is_empty() { row.center().y } else { row.center().y - 10.0 };
    ui.painter()
        .text(pos2(row.left() + 6.0, title_y), Align2::CENTER_CENTER, sym, glyph(17.0), MUTED);
    label(ui, pos2(row.left() + 34.0, title_y), Align2::LEFT_CENTER, title, medium(13.5), TEXT, full - 320.0);
    if !desc.is_empty() {
        label(ui, pos2(row.left() + 34.0, title_y + 18.0), Align2::LEFT_CENTER, desc, sans(11.5), MUTED, full - 320.0);
    }
    row
}

/// Icon-first toggle row. Returns true when the switch flips.
fn toggle_row(ui: &mut Ui, id_salt: &str, sym: &str, title: &str, desc: &str, on: bool) -> bool {
    let accent = ui.ctx().data(|d| d.get_temp(Id::new("app-theme-accent"))).unwrap_or(ACCENT);
    let full = ui.available_width();
    let height = if desc.is_empty() { ROW_PLAIN } else { ROW_DESC };
    let (row, response) = ui.allocate_exact_size(vec2(full, height), Sense::click());
    let hover = hover_of(ui, &response);
    ui.painter().rect_filled(row, 8.0, veil(0.05 * hover));
    let title_y = if desc.is_empty() { row.center().y } else { row.center().y - 10.0 };
    ui.painter().text(
        pos2(row.left() + 6.0, title_y),
        Align2::CENTER_CENTER,
        sym,
        glyph(17.0),
        MUTED.lerp_to_gamma(TEXT, hover),
    );
    label(ui, pos2(row.left() + 34.0, title_y), Align2::LEFT_CENTER, title, medium(13.5), TEXT, full - 300.0);
    if !desc.is_empty() {
        label(ui, pos2(row.left() + 34.0, title_y + 18.0), Align2::LEFT_CENTER, desc, sans(11.5), MUTED, full - 300.0);
    }
    let lit = ui
        .ctx()
        .animate_bool_with_time(response.id.with(id_salt).with("switch"), on, HOVER_TIME);
    let track = Rect::from_center_size(pos2(row.right() - 27.0, row.center().y), vec2(30.0, 18.0));
    ui.painter().rect_filled(track, 9.0, veil(0.16).lerp_to_gamma(accent, lit));
    let knob = pos2(egui::lerp(track.left() + 9.0..=track.right() - 9.0, lit), track.center().y);
    ui.painter().circle_filled(knob, 6.0, TEXT.lerp_to_gamma(contrast_text_for(accent), lit));
    response.widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, true, on, title));
    describe(&response, title);
    response.clicked()
}

/// Icon-first slider row with a visible track. Returns the value while it changes.
fn slider_row(
    ui: &mut Ui,
    id_salt: &str,
    sym: &str,
    title: &str,
    desc: &str,
    value: f32,
    range: std::ops::RangeInclusive<f32>,
    format: impl Fn(f32) -> String,
) -> Option<f32> {
    let accent = ui.ctx().data(|d| d.get_temp(Id::new("app-theme-accent"))).unwrap_or(ACCENT);
    let row = row_base(ui, sym, title, desc);
    let val_w = 58.0;
    let track_right = row.right() - val_w - 12.0;
    let track_left = (track_right - 220.0).max(row.left() + 150.0);
    if track_right - track_left < 60.0 {
        return None;
    }
    let line = Rect::from_min_max(
        pos2(track_left, row.center().y - 3.0),
        pos2(track_right, row.center().y + 3.0),
    );
    let galley = ui.painter().layout_no_wrap(format(value), bold(12.5), accent);
    ui.painter().galley(
        pos2(row.right() - galley.size().x, row.center().y - galley.size().y / 2.0),
        galley,
        accent,
    );
    let resp = ui.interact(line.expand2(vec2(0.0, 9.0)), Id::new(id_salt), Sense::click_and_drag());
    point_at(&resp);
    let min = *range.start();
    let max = *range.end();
    let frac = if max > min { ((value - min) / (max - min)).clamp(0.0, 1.0) } else { 0.0 };
    ui.painter().rect_filled(line, 3.0, Color32::from_rgb(52, 54, 66));
    let fill = Rect::from_min_max(line.min, pos2(line.left() + line.width() * frac, line.max.y));
    ui.painter().rect_filled(fill, 3.0, accent);
    let thumb = pos2(line.left() + line.width() * frac, line.center().y);
    ui.painter().circle_filled(thumb, 7.0, Color32::WHITE);
    ui.painter().circle_filled(thumb, 3.5, accent);
    describe(&resp, title);
    if let (true, Some(pointer)) = ((resp.clicked() || resp.dragged()), resp.interact_pointer_pos()) {
        let new_frac = ((pointer.x - line.left()) / line.width().max(1.0)).clamp(0.0, 1.0);
        let new_value = min + new_frac * (max - min);
        if (new_value - value).abs() > f32::EPSILON {
            return Some(new_value);
        }
    }
    None
}

/// Icon-first row whose options render as an equal-width pill group below the title.
fn segmented_row<T: Copy + PartialEq>(
    ui: &mut Ui,
    id_salt: &str,
    sym: &str,
    title: &str,
    desc: &str,
    current: T,
    options: &[(T, &str)],
) -> Option<T> {
    let accent = ui.ctx().data(|d| d.get_temp(Id::new("app-theme-accent"))).unwrap_or(ACCENT);
    row_base(ui, sym, title, desc);
    let full = ui.available_width();
    let count = options.len().max(1) as f32;
    let pill_w = (full - 6.0 * (count - 1.0)) / count;
    let (pill_row, _) = ui.allocate_exact_size(vec2(full, 32.0), Sense::hover());
    let mut picked = None;
    for (index, &(value, opt_label)) in options.iter().enumerate() {
        let rect = Rect::from_min_size(
            pos2(pill_row.left() + index as f32 * (pill_w + 6.0), pill_row.top() + 1.0),
            vec2(pill_w, 30.0),
        );
        let resp = ui.interact(rect, Id::new((id_salt, index)), Sense::click());
        let hover = hover_of(ui, &resp);
        let active = current == value;
        if active {
            ui.painter().rect_filled(rect, 9.0, accent);
            ui.painter()
                .text(rect.center(), Align2::CENTER_CENTER, opt_label, bold(12.0), contrast_text_for(accent));
        } else {
            ui.painter().rect_filled(rect, 9.0, veil(0.06 + 0.04 * hover));
            ui.painter().rect_stroke(rect, 9.0, Stroke::new(1.0_f32, veil(0.10 + 0.06 * hover)), StrokeKind::Inside);
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                opt_label,
                medium(12.0),
                if hover > 0.1 { TEXT } else { MUTED },
            );
        }
        describe(&resp, opt_label);
        if resp.clicked() {
            picked = Some(value);
        }
    }
    ui.add_space(4.0);
    picked
}

/// Icon-first row with a trailing combo box.
fn combo_row<T: Copy + PartialEq>(
    ui: &mut Ui,
    id_salt: &str,
    sym: &str,
    title: &str,
    desc: &str,
    current: T,
    options: &[(T, &str)],
) -> Option<T> {
    let mut picked = None;
    let row = row_base(ui, sym, title, desc);
    let area = Rect::from_min_max(pos2(row.right() - 210.0, row.top()), pos2(row.right(), row.bottom()));
    let mut combo = ui.new_child(
        UiBuilder::new()
            .id_salt(id_salt)
            .max_rect(area)
            .layout(egui::Layout::right_to_left(Align::Center)),
    );
    let selected = options
        .iter()
        .find(|(value, _)| *value == current)
        .map(|(_, opt_label)| *opt_label)
        .unwrap_or("");
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(RichText::new(selected).font(sans(12.0)).color(TEXT))
        .width(200.0)
        .show_ui(&mut combo, |ui| {
            for &(value, opt_label) in options {
                if ui.selectable_label(value == current, opt_label).clicked() {
                    picked = Some(value);
                }
            }
        });
    picked
}

/// Icon-first row with a trailing pill button. `danger` renders the button in red.
fn button_row(ui: &mut Ui, id_salt: &str, sym: &str, title: &str, desc: &str, button: &str, danger: bool) -> bool {
    let accent = ui.ctx().data(|d| d.get_temp(Id::new("app-theme-accent"))).unwrap_or(ACCENT);
    let row = row_base(ui, sym, title, desc);
    let btn = Rect::from_min_size(pos2(row.right() - 110.0, row.center().y - 14.0), vec2(104.0, 28.0));
    let resp = ui.interact(btn, Id::new(id_salt), Sense::click());
    let hover = hover_of(ui, &resp);
    let tint = if danger {
        Color32::from_rgb(239, 68, 68)
    } else {
        accent
    };
    ui.painter().rect_filled(btn, 14.0, tint.gamma_multiply(0.18 + 0.12 * hover));
    ui.painter().rect_stroke(btn, 14.0, Stroke::new(1.0_f32, tint), StrokeKind::Inside);
    ui.painter().text(
        btn.center(),
        Align2::CENTER_CENTER,
        button,
        bold(12.0),
        if hover > 0.1 { TEXT } else { tint },
    );
    describe(&resp, button);
    resp.clicked()
}

/// Icon-first row with a visible trailing text input. Returns the text while typing.
fn input_row(
    ui: &mut Ui,
    id_salt: &str,
    sym: &str,
    title: &str,
    desc: &str,
    hint: &str,
    secret: bool,
    width: f32,
    current: &str,
) -> Option<String> {
    let row = row_base(ui, sym, title, desc);
    let field = Rect::from_min_size(pos2(row.right() - width - 8.0, row.center().y - 15.0), vec2(width, 30.0));
    ui.painter().rect_filled(field, 7.0, veil(0.07));
    ui.painter().rect_stroke(field, 7.0, Stroke::new(1.0_f32, veil(0.12)), StrokeKind::Inside);
    let mut buffer = current.to_owned();
    let edit = egui::TextEdit::singleline(&mut buffer)
        .hint_text(hint)
        .desired_width(width - 8.0)
        .font(sans(12.5))
        .password(secret)
        .frame(false);
    let resp = ui.push_id(id_salt, |ui| ui.put(field.shrink2(vec2(5.0, 2.0)), edit));
    if resp.inner.changed() {
        return Some(buffer);
    }
    None
}

fn settings_card_panel(ui: &mut Ui, id_salt: &str, title: &str, subtitle: &str, body: impl FnOnce(&mut Ui)) {
    if !title.is_empty() {
        ui.label(RichText::new(title).font(bold(12.5)).color(TEXT));
    }
    if !subtitle.is_empty() {
        ui.label(RichText::new(subtitle).font(sans(11.0)).color(MUTED));
        ui.add_space(3.0);
    }
    let frame = Frame::new()
        .fill(veil(0.05))
        .stroke(Stroke::new(1.0_f32, veil(0.08)))
        .corner_radius(CARD_RADIUS as u8)
        .inner_margin(Margin::same(12));
    ui.push_id(id_salt, |ui| {
        frame.show(ui, |ui| {
            body(ui);
        });
    });
    ui.add_space(10.0);
}

fn render_discord_preview(ui: &mut Ui, app: &App, out: &mut Vec<Action>, w: f32) {
    let pal_color = app.saved.theme_palette.color();
    ui.vertical(|ui| {
        ui.set_width(w);

        ui.label(RichText::new("DISCORD PREVIEW").font(bold(11.5)).color(MUTED));
        ui.add_space(4.0);

        let frame = Frame::new()
            .fill(veil(0.05))
            .stroke(Stroke::new(1.0_f32, veil(0.08)))
            .corner_radius(CARD_RADIUS as u8)
            .inner_margin(Margin::same(12));

        frame.show(ui, |ui| {
            // Status Badge
            ui.horizontal(|ui| {
                let (badge_txt, badge_col) = if app.playing() {
                    ("Now playing", Color32::from_rgb(225, 29, 72))
                } else {
                    ("Paused", Color32::from_rgb(100, 116, 139))
                };
                let (badge_rect, _) = ui.allocate_exact_size(vec2(80.0, 18.0), Sense::hover());
                ui.painter().rect_filled(badge_rect, 9.0, badge_col.gamma_multiply(0.20));
                ui.painter().text(badge_rect.center(), Align2::CENTER_CENTER, badge_txt, bold(10.0), badge_col);
            });
            ui.add_space(6.0);

            // Resolved sample / live song metadata
            let current_song = app.saved.current.and_then(|c| app.saved.queue.get(c));
            let live_title = current_song.map_or("Mangal Rang (From \"Ramayana\")", |s| s.title.as_str());
            let live_artist = current_song.map_or("A.R. Rahman, Arijit Singh, Shreya Ghoshal", |s| s.subtitle.as_str());
            let live_album = current_song.map_or("Mangal Rang (From \"Ramayana\")", |s| if s.extra.is_empty() { s.title.as_str() } else { s.extra.as_str() });
            let app_name_display = if app.saved.discord_app_name.is_empty() { "AURI" } else { &app.saved.discord_app_name };

            let format_prev_line = |content: crate::app::DiscordLineContent| -> &str {
                match content {
                    crate::app::DiscordLineContent::Title => live_title,
                    crate::app::DiscordLineContent::Artist => live_artist,
                    crate::app::DiscordLineContent::Album => live_album,
                    crate::app::DiscordLineContent::ArtistAlbum => live_artist,
                    crate::app::DiscordLineContent::Off => "",
                }
            };

            let card_w = w - 24.0;
            let (card_rect, _) = ui.allocate_exact_size(vec2(card_w, 195.0), Sense::hover());
            ui.painter().rect_filled(card_rect, 10.0, Color32::from_rgb(30, 31, 35));
            ui.painter().rect_stroke(card_rect, 10.0, Stroke::new(1.0_f32, Color32::from_rgb(43, 45, 52)), StrokeKind::Inside);

            // Header text: Listening to <App Name>
            let header_str = format!("Listening to {}", app_name_display);
            label(ui, pos2(card_rect.left() + 14.0, card_rect.top() + 14.0), Align2::LEFT_CENTER, &header_str, bold(11.0), Color32::from_rgb(181, 186, 193), card_w - 40.0);
            ui.painter().text(pos2(card_rect.right() - 14.0, card_rect.top() + 14.0), Align2::RIGHT_CENTER, "···", bold(14.0), MUTED);

            // Artwork & Lines Row
            let mut text_x = card_rect.left() + 14.0;
            if app.saved.discord_album_art && !app.saved.discord_hide_details {
                let art_r = Rect::from_min_size(pos2(card_rect.left() + 14.0, card_rect.top() + 32.0), vec2(54.0, 54.0));
                if let Some(it) = current_song {
                    artwork(ui, &app.art, app.saved.covers.of(it), art_r, 6.0);
                } else {
                    ui.painter().rect_filled(art_r, 6.0, Color32::from_rgb(43, 45, 52));
                    ui.painter().text(art_r.center(), Align2::CENTER_CENTER, icon::MUSIC_NOTE, glyph(24.0), pal_color);
                }
                if app.saved.discord_provider_badge {
                    let badge_pos = pos2(art_r.right() - 4.0, art_r.bottom() - 4.0);
                    ui.painter().circle_filled(badge_pos, 7.0, Color32::from_rgb(30, 31, 35));
                    ui.painter().circle_filled(badge_pos, 5.0, pal_color);
                }
                text_x = art_r.right() + 12.0;
            }

            let text_max_w = card_rect.right() - text_x - 14.0;
            if app.saved.discord_hide_details {
                label(ui, pos2(text_x, card_rect.top() + 42.0), Align2::LEFT_CENTER, "Playing music", bold(13.0), TEXT, text_max_w);
            } else {
                let l1 = format_prev_line(app.saved.discord_line1);
                let l2 = format_prev_line(app.saved.discord_line2);
                let l3 = format_prev_line(app.saved.discord_line3);

                if !l1.is_empty() {
                    let col = if app.saved.discord_line1_clickable { Color32::from_rgb(88, 166, 255) } else { TEXT };
                    label(ui, pos2(text_x, card_rect.top() + 38.0), Align2::LEFT_CENTER, l1, bold(12.5), col, text_max_w);
                }
                if !l2.is_empty() {
                    let col = if app.saved.discord_line2_clickable { Color32::from_rgb(88, 166, 255) } else { Color32::from_rgb(181, 186, 193) };
                    label(ui, pos2(text_x, card_rect.top() + 54.0), Align2::LEFT_CENTER, l2, sans(11.0), col, text_max_w);
                }
                if !l3.is_empty() {
                    label(ui, pos2(text_x, card_rect.top() + 69.0), Align2::LEFT_CENTER, l3, sans(10.0), Color32::from_rgb(148, 155, 164), text_max_w);
                }
            }

            // Progress bar
            let mut btn_y = card_rect.top() + 96.0;
            if app.saved.discord_progress_bar && !app.saved.discord_hide_details {
                let p_left = card_rect.left() + 14.0;
                let p_right = card_rect.right() - 14.0;
                label(ui, pos2(p_left, btn_y), Align2::LEFT_CENTER, "0:13", sans(10.0), MUTED, 30.0);
                let line_y = btn_y;
                let bar_x1 = p_left + 32.0;
                let bar_x2 = p_right - 32.0;
                ui.painter().hline(bar_x1..=bar_x2, line_y, Stroke::new(2.5_f32, Color32::from_rgb(60, 63, 72)));
                let fill_x = bar_x1 + (bar_x2 - bar_x1) * 0.28;
                ui.painter().hline(bar_x1..=fill_x, line_y, Stroke::new(2.5_f32, Color32::WHITE));
                ui.painter().circle_filled(pos2(fill_x, line_y), 3.5, Color32::WHITE);
                label(ui, pos2(p_right, btn_y), Align2::RIGHT_CENTER, "4:37", sans(10.0), MUTED, 30.0);
                btn_y += 18.0;
            }

            // Buttons
            let draw_btn = |label_text: &str, y: f32| {
                let b_rect = Rect::from_min_size(pos2(card_rect.left() + 14.0, y), vec2(card_w - 28.0, 24.0));
                ui.painter().rect_filled(b_rect, 5.0, Color32::from_rgb(43, 45, 52));
                ui.painter().text(b_rect.center(), Align2::CENTER_CENTER, label_text, medium(11.0), Color32::WHITE);
            };
            if !app.saved.discord_hide_details {
                if app.saved.discord_button1 != crate::app::DiscordButtonOption::None {
                    let b_label = if app.saved.discord_button1 == crate::app::DiscordButtonOption::GetApp {
                        format!("Get {}", app_name_display)
                    } else {
                        app.saved.discord_button1.label().to_string()
                    };
                    draw_btn(&b_label, btn_y);
                    btn_y += 28.0;
                }
                if app.saved.discord_button2 != crate::app::DiscordButtonOption::None {
                    let b_label = if app.saved.discord_button2 == crate::app::DiscordButtonOption::GetApp {
                        format!("Get {}", app_name_display)
                    } else {
                        app.saved.discord_button2.label().to_string()
                    };
                    draw_btn(&b_label, btn_y);
                }
            }

            ui.add_space(10.0);

            // Member List Mockup
            let (mem_rect, _) = ui.allocate_exact_size(vec2(card_w, 52.0), Sense::hover());
            ui.painter().rect_filled(mem_rect, 8.0, Color32::from_rgb(30, 31, 35));
            ui.painter().rect_stroke(mem_rect, 8.0, Stroke::new(1.0_f32, Color32::from_rgb(43, 45, 52)), StrokeKind::Inside);

            let av_rect = Rect::from_center_size(pos2(mem_rect.left() + 22.0, mem_rect.center().y), vec2(30.0, 30.0));
            ui.painter().circle_filled(av_rect.center(), 15.0, pal_color.gamma_multiply(0.65));
            ui.painter().text(av_rect.center(), Align2::CENTER_CENTER, "You", bold(10.5), Color32::WHITE);

            let mem_text_x = av_rect.right() + 10.0;
            label(ui, pos2(mem_text_x, mem_rect.center().y - 8.0), Align2::LEFT_CENTER, "You", bold(12.0), TEXT, card_w - 70.0);

            let member_status_str = match app.saved.discord_status_text {
                crate::app::DiscordStatusField::Line2Artist => format!("🎵 {}", live_artist),
                crate::app::DiscordStatusField::Line1Title => format!("🎵 {}", live_title),
                crate::app::DiscordStatusField::Line3Album => format!("🎵 {}", live_album),
                crate::app::DiscordStatusField::ListeningToApp => format!("🎵 Listening to {}", app_name_display),
                crate::app::DiscordStatusField::Custom => format!("🎵 {}", if app.saved.discord_name.is_empty() { live_title } else { &app.saved.discord_name }),
                crate::app::DiscordStatusField::None => "Online".to_string(),
            };
            label(ui, pos2(mem_text_x, mem_rect.center().y + 8.0), Align2::LEFT_CENTER, &member_status_str, sans(10.0), Color32::from_rgb(148, 155, 164), card_w - 70.0);

            ui.add_space(10.0);

            // Description note
            let note_str = if app.saved.discord {
                "Rich Presence is active. The preview updates live."
            } else {
                "Rich Presence is disabled. Preview follows your changes."
            };
            ui.label(RichText::new(note_str).font(sans(10.5)).color(MUTED));
            ui.add_space(8.0);

            // Reset button
            let (reset_rect, reset_resp) = ui.allocate_exact_size(vec2(card_w, 28.0), Sense::click());
            let r_hov = hover_of(ui, &reset_resp);
            ui.painter().rect_filled(reset_rect, 6.0, veil(0.06 * r_hov));
            ui.painter().text(reset_rect.center(), Align2::CENTER_CENTER, "↺ Reset to defaults", medium(11.5), if r_hov > 0.1 { pal_color } else { MUTED });
            if reset_resp.clicked() {
                out.push(Action::ResetDiscordDefaults);
            }
        });
    });
}

fn render_discord_controls(ui: &mut Ui, app: &App, out: &mut Vec<Action>, w: f32) {
    let pal_color = app.saved.theme_palette.color();
    ui.vertical(|ui| {
        ui.set_width(w);

        // Header Preview Toggle Pill
        let (preview_pill, _) = ui.allocate_exact_size(vec2(w, 28.0), Sense::hover());
        let btn_w = 140.0;
        let btn_rect = Rect::from_min_size(pos2(preview_pill.right() - btn_w, preview_pill.center().y - 13.0), vec2(btn_w, 26.0));
        let btn_resp = ui.interact(btn_rect, Id::new("settings-discord-toggle-preview-btn"), Sense::click());
        let hov = hover_of(ui, &btn_resp);
        let preview_active = app.settings_preview_open && !app.saved.queue_open;
        ui.painter().rect_filled(btn_rect, 13.0, if preview_active { pal_color.gamma_multiply(0.20) } else { veil(0.06 + 0.04 * hov) });
        ui.painter().rect_stroke(btn_rect, 13.0, Stroke::new(1.0_f32, if preview_active { pal_color } else { veil(0.12) }), StrokeKind::Inside);
        let text = if preview_active { "✕ Close Preview" } else { "👁 Live Preview ➔" };
        ui.painter().text(btn_rect.center(), Align2::CENTER_CENTER, text, bold(11.0), if preview_active { pal_color } else { if hov > 0.1 { TEXT } else { MUTED } });
        if btn_resp.clicked() {
            out.push(Action::ToggleSettingsPreview);
        }
        ui.add_space(6.0);

        // Card 1: RICH PRESENCE & PRIVACY
        settings_card_panel(ui, "disc-card-presence", "Rich presence & privacy", "Control how music activity is shared with Discord.", |ui| {
            let (status_text, stat_color) = match app.discord_linked {
                Some(true) if app.playing() => ("Connected · Showing active track", Color32::from_rgb(34, 197, 94)),
                Some(true) => ("Connected · Shows while track plays", Color32::from_rgb(34, 197, 94)),
                Some(false) => ("Discord is not running on this PC", Color32::from_rgb(239, 68, 68)),
                None => ("Connecting to Discord RPC...", Color32::from_rgb(234, 179, 8)),
            };
            ui.horizontal(|ui| {
                let (ind, _) = ui.allocate_exact_size(vec2(12.0, 18.0), Sense::hover());
                ui.painter().circle_filled(ind.center(), 3.5, stat_color);
                ui.label(RichText::new(status_text).font(sans(11.5)).color(stat_color));
            });
            ui.add_space(6.0);

            if toggle_row(
                ui,
                "disc-rich",
                icon::DISCORD_LOGO,
                "Rich presence",
                "Show the currently playing song on your Discord profile.",
                app.saved.discord,
            ) {
                out.push(Action::ShowOnDiscord(!app.saved.discord));
            }
            if toggle_row(
                ui,
                "disc-paused",
                icon::PAUSE,
                "Show when paused",
                "Keep the card up while the track is paused without a running progress bar.",
                app.saved.discord_show_when_paused,
            ) {
                out.push(Action::SetDiscordShowWhenPaused(!app.saved.discord_show_when_paused));
            }
            if toggle_row(
                ui,
                "disc-hide",
                icon::EYE_SLASH,
                "Hide details",
                "Stealth mode: say only that music is playing, without song title, artist or artwork.",
                app.saved.discord_hide_details,
            ) {
                out.push(Action::SetDiscordHideDetails(!app.saved.discord_hide_details));
            }
        });

        // Card 2: STATUS & APP NAME
        settings_card_panel(ui, "disc-card-status", "Status & app name", "Configure the status line shown next to your name in member lists.", |ui| {
            if let Some(st) = combo_row(
                ui,
                "disc-status-text",
                icon::USER,
                "Status text",
                "What Discord shows next to your name in the member list.",
                app.saved.discord_status_text,
                &[
                    (crate::app::DiscordStatusField::Line2Artist, "Line 2 · Artist"),
                    (crate::app::DiscordStatusField::Line1Title, "Line 1 · Song title"),
                    (crate::app::DiscordStatusField::Line3Album, "Line 3 · Album"),
                    (crate::app::DiscordStatusField::ListeningToApp, "Listening to App"),
                    (crate::app::DiscordStatusField::Custom, "Custom"),
                    (crate::app::DiscordStatusField::None, "None"),
                ],
            ) {
                out.push(Action::SetDiscordStatusText(st));
            }

            if let Some(name) = input_row(
                ui,
                "disc-app-name",
                icon::TEXT_T,
                "App name",
                "Replaces \"AURI\" wherever the app name appears. Leave empty for default.",
                "AURI",
                false,
                200.0,
                &app.saved.discord_app_name,
            ) {
                out.push(Action::SetDiscordAppName(name));
            }
        });

        // Card 3: CARD CONTENT & BUTTONS
        settings_card_panel(ui, "disc-card-content", "Profile card content", "Lines and action buttons shown on your Discord profile card.", |ui| {
            if let Some(line) = combo_row(
                ui,
                "disc-line-1",
                icon::TEXT_T,
                "Line 1 (bold)",
                "Header line below the app title.",
                app.saved.discord_line1,
                &[
                    (crate::app::DiscordLineContent::Title, "Song title"),
                    (crate::app::DiscordLineContent::Artist, "Artist"),
                    (crate::app::DiscordLineContent::Album, "Album"),
                    (crate::app::DiscordLineContent::Off, "None"),
                ],
            ) {
                out.push(Action::SetDiscordLine1(line));
            }
            if let Some(line) = combo_row(
                ui,
                "disc-line-2",
                icon::TEXT_T,
                "Line 2",
                "Secondary line below the title.",
                app.saved.discord_line2,
                &[
                    (crate::app::DiscordLineContent::Artist, "Artist"),
                    (crate::app::DiscordLineContent::Title, "Song title"),
                    (crate::app::DiscordLineContent::ArtistAlbum, "Artist · Album"),
                    (crate::app::DiscordLineContent::Album, "Album"),
                    (crate::app::DiscordLineContent::Off, "None"),
                ],
            ) {
                out.push(Action::SetDiscordLine2(line));
            }
            if let Some(line) = combo_row(
                ui,
                "disc-line-3",
                icon::TEXT_T,
                "Line 3",
                "Album details shown on artwork hover.",
                app.saved.discord_line3,
                &[
                    (crate::app::DiscordLineContent::Album, "Album"),
                    (crate::app::DiscordLineContent::Artist, "Artist"),
                    (crate::app::DiscordLineContent::Title, "Song title"),
                    (crate::app::DiscordLineContent::Off, "None"),
                ],
            ) {
                out.push(Action::SetDiscordLine3(line));
            }

            if toggle_row(ui, "disc-l1-click", icon::LINK, "Clickable line 1", "Make line 1 link directly to the track on YouTube Music.", app.saved.discord_line1_clickable) {
                out.push(Action::SetDiscordLine1Clickable(!app.saved.discord_line1_clickable));
            }
            if toggle_row(ui, "disc-l2-click", icon::LINK, "Clickable line 2", "Make line 2 link directly to the artist on YouTube Music.", app.saved.discord_line2_clickable) {
                out.push(Action::SetDiscordLine2Clickable(!app.saved.discord_line2_clickable));
            }
            if toggle_row(ui, "disc-album-art", icon::IMAGE, "Album art", "Show cover artwork next to the track.", app.saved.discord_album_art) {
                out.push(Action::SetDiscordAlbumArt(!app.saved.discord_album_art));
            }
            if toggle_row(ui, "disc-art-click", icon::LINK, "Clickable artwork", "Make the cover art link directly to the song on YouTube Music.", app.saved.discord_art_clickable) {
                out.push(Action::SetDiscordArtClickable(!app.saved.discord_art_clickable));
            }
            if toggle_row(ui, "disc-badge", icon::STAR, "Provider badge", "Mark the artwork with a small AURI icon.", app.saved.discord_provider_badge) {
                out.push(Action::SetDiscordProviderBadge(!app.saved.discord_provider_badge));
            }
            if toggle_row(ui, "disc-progress", icon::TIMER, "Progress bar", "Show the track elapsed scrubber bar.", app.saved.discord_progress_bar) {
                out.push(Action::SetDiscordProgressBar(!app.saved.discord_progress_bar));
            }

            if let Some(button) = combo_row(
                ui,
                "disc-btn-1",
                icon::PLAY,
                "First button",
                "",
                app.saved.discord_button1,
                &[
                    (crate::app::DiscordButtonOption::ListenOnYtm, "Listen on YouTube Music"),
                    (crate::app::DiscordButtonOption::GetApp, "Get AURI"),
                    (crate::app::DiscordButtonOption::None, "None"),
                ],
            ) {
                out.push(Action::SetDiscordButton1(button));
            }
            if let Some(button) = combo_row(
                ui,
                "disc-btn-2",
                icon::PLAY,
                "Second button",
                "",
                app.saved.discord_button2,
                &[
                    (crate::app::DiscordButtonOption::GetApp, "Get AURI"),
                    (crate::app::DiscordButtonOption::ListenOnYtm, "Listen on YouTube Music"),
                    (crate::app::DiscordButtonOption::None, "None"),
                ],
            ) {
                out.push(Action::SetDiscordButton2(button));
            }
        });
    });
}

fn discord_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    // The live preview renders in the right rail (the queue dock slot), so the
    // controls always keep the full 740px settings column.
    render_discord_controls(ui, app, out, ui.available_width().min(740.0));
}

fn render_gaming_preview(ui: &mut Ui, app: &App, out: &mut Vec<Action>, w: f32) {
    let pal_color = app.saved.theme_palette.color();
    ui.vertical(|ui| {
        ui.set_width(w);

        ui.label(RichText::new("OVERLAY PREVIEW").font(bold(11.5)).color(MUTED));
        ui.add_space(4.0);

        let frame = Frame::new()
            .fill(veil(0.05))
            .stroke(Stroke::new(1.0_f32, veil(0.08)))
            .corner_radius(CARD_RADIUS as u8)
            .inner_margin(Margin::same(12));

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                let (badge_rect, _) = ui.allocate_exact_size(vec2(100.0, 18.0), Sense::hover());
                ui.painter().rect_filled(badge_rect, 9.0, pal_color.gamma_multiply(0.20));
                ui.painter().text(badge_rect.center(), Align2::CENTER_CENTER, "Translucent HUD", bold(10.0), pal_color);
            });
            ui.add_space(6.0);

            // Mockup Screen Container
            let card_w = w - 24.0;
            let (screen_rect, _) = ui.allocate_exact_size(vec2(card_w, 240.0), Sense::hover());
            let alpha = (app.saved.game_overlay_opacity.clamp(0.15, 0.95) * 255.0) as u8;
            ui.painter().rect_filled(screen_rect, 10.0, Color32::from_black_alpha(alpha));
            ui.painter().rect_stroke(screen_rect, 10.0, Stroke::new(1.0_f32, Color32::from_rgb(43, 45, 52)), StrokeKind::Inside);

            // Clock & Title in mockup
            label(ui, pos2(screen_rect.left() + 12.0, screen_rect.top() + 14.0), Align2::LEFT_CENTER, "14:25", bold(15.0), Color32::WHITE, 80.0);
            label(ui, pos2(screen_rect.left() + 12.0, screen_rect.top() + 28.0), Align2::LEFT_CENTER, "SESSION 00:42 · ZERO LAG", bold(8.0), pal_color, 140.0);

            let status_r = Rect::from_min_size(pos2(screen_rect.right() - 100.0, screen_rect.top() + 10.0), vec2(88.0, 16.0));
            ui.painter().rect_filled(status_r, 8.0, veil(0.15));
            ui.painter().circle_filled(pos2(status_r.left() + 8.0, status_r.center().y), 2.5, Color32::from_rgb(34, 197, 94));
            ui.painter().text(pos2(status_r.left() + 16.0, status_r.center().y), Align2::LEFT_CENTER, "AURI: 18.2 MB", bold(8.0), TEXT);

            let current_song = app.saved.current.and_then(|c| app.saved.queue.get(c));
            let live_title = current_song.map_or("Mangal Rang (From \"Ramayana\")", |s| s.title.as_str());
            let live_artist = current_song.map_or("A.R. Rahman, Arijit Singh", |s| s.subtitle.as_str());

            let mut cur_y = screen_rect.top() + 44.0;

            // Mini Music Widget Mockup
            if app.saved.game_show_music_widget {
                let m_rect = Rect::from_min_size(pos2(screen_rect.left() + 8.0, cur_y), vec2(card_w - 16.0, 52.0));
                ui.painter().rect_filled(m_rect, 8.0, Color32::from_rgba_premultiplied(18, 20, 26, 240));
                ui.painter().rect_stroke(m_rect, 8.0, Stroke::new(1.0_f32, pal_color.gamma_multiply(0.35)), StrokeKind::Inside);

                let m_thumb = Rect::from_min_size(pos2(m_rect.left() + 6.0, m_rect.top() + 6.0), Vec2::splat(40.0));
                let mut drew = false;
                if let Some(cover_str) = current_song.map(|s| app.saved.covers.of(s)) {
                    if let Some(pic) = app.art.get(ui.ctx(), cover_str, 1.0) {
                        egui::Image::from_texture((pic.id, m_thumb.size())).corner_radius(4.0).paint_at(ui, m_thumb);
                        drew = true;
                    }
                }
                if !drew {
                    ui.painter().rect_filled(m_thumb, 4.0, Color32::from_rgb(30, 32, 40));
                    ui.painter().text(m_thumb.center(), Align2::CENTER_CENTER, icon::MUSIC_NOTE, glyph(14.0), MUTED);
                }

                label(ui, pos2(m_thumb.right() + 8.0, m_rect.top() + 12.0), Align2::LEFT_CENTER, live_title, bold(10.5), TEXT, m_rect.width() - 85.0);
                label(ui, pos2(m_thumb.right() + 8.0, m_rect.top() + 25.0), Align2::LEFT_CENTER, live_artist, sans(9.0), MUTED, m_rect.width() - 85.0);

                let scrob_x1 = m_thumb.right() + 8.0;
                let scrob_x2 = m_rect.right() - 32.0;
                let scrob_fill = scrob_x1 + (scrob_x2 - scrob_x1) * 0.35;
                ui.painter().hline(scrob_x1..=scrob_x2, m_rect.top() + 39.0, Stroke::new(2.0_f32, veil(0.15)));
                ui.painter().hline(scrob_x1..=scrob_fill, m_rect.top() + 39.0, Stroke::new(2.0_f32, pal_color));

                ui.painter().circle_filled(pos2(m_rect.right() - 16.0, m_rect.center().y), 10.0, pal_color);
                ui.painter().text(pos2(m_rect.right() - 16.0, m_rect.center().y), Align2::CENTER_CENTER, icon::PLAY, glyph(9.0), contrast_text_for(pal_color));

                cur_y += 58.0;
            }

            // Mini Audio & Hardware widget
            if app.saved.game_show_audio_widget {
                let a_rect = Rect::from_min_size(pos2(screen_rect.left() + 8.0, cur_y), vec2(card_w - 16.0, 40.0));
                ui.painter().rect_filled(a_rect, 8.0, Color32::from_rgba_premultiplied(18, 20, 26, 240));
                ui.painter().rect_stroke(a_rect, 8.0, Stroke::new(1.0_f32, veil(0.12)), StrokeKind::Inside);

                ui.painter().text(pos2(a_rect.left() + 14.0, a_rect.center().y), Align2::CENTER_CENTER, icon::SPEAKER_HIGH, glyph(12.0), pal_color);
                let v_x1 = a_rect.left() + 28.0;
                let v_x2 = a_rect.right() - 96.0;
                let v_fill = v_x1 + (v_x2 - v_x1) * app.saved.volume.clamp(0.0, 1.0);
                ui.painter().hline(v_x1..=v_x2, a_rect.center().y, Stroke::new(2.0_f32, veil(0.15)));
                ui.painter().hline(v_x1..=v_fill, a_rect.center().y, Stroke::new(2.0_f32, pal_color));

                let clean_pill = Rect::from_min_size(pos2(a_rect.right() - 86.0, a_rect.center().y - 10.0), vec2(76.0, 20.0));
                ui.painter().rect_filled(clean_pill, 5.0, veil(0.10));
                ui.painter().text(clean_pill.center(), Align2::CENTER_CENTER, "⚡ Clean RAM", bold(9.0), pal_color);

                cur_y += 46.0;
            }

            // Mini Queue widget
            if app.saved.game_show_queue_widget && cur_y < screen_rect.bottom() - 50.0 {
                let q_rect = Rect::from_min_size(pos2(screen_rect.left() + 8.0, cur_y), vec2(card_w - 16.0, (screen_rect.bottom() - cur_y - 36.0).max(34.0)));
                ui.painter().rect_filled(q_rect, 8.0, Color32::from_rgba_premultiplied(18, 20, 26, 240));
                ui.painter().rect_stroke(q_rect, 8.0, Stroke::new(1.0_f32, veil(0.12)), StrokeKind::Inside);

                label(ui, pos2(q_rect.left() + 10.0, q_rect.top() + 10.0), Align2::LEFT_CENTER, &format!("QUEUE ({} songs)", app.saved.queue.len()), bold(8.5), MUTED, q_rect.width() - 20.0);
                for (i, song) in app.saved.queue.iter().take(1).enumerate() {
                    let sy = q_rect.top() + 22.0 + i as f32 * 12.0;
                    if sy < q_rect.bottom() - 6.0 {
                        label(ui, pos2(q_rect.left() + 10.0, sy), Align2::LEFT_CENTER, &format!("{} · {}", song.title, song.subtitle), sans(8.0), TEXT, q_rect.width() - 20.0);
                    }
                }
            }

            // Mini Bottom Dock
            let dock_r = Rect::from_min_size(pos2(screen_rect.left() + 10.0, screen_rect.bottom() - 30.0), vec2(card_w - 20.0, 24.0));
            ui.painter().rect_filled(dock_r, 12.0, Color32::from_rgba_premultiplied(22, 24, 32, 248));
            ui.painter().rect_stroke(dock_r, 12.0, Stroke::new(1.0_f32, veil(0.15)), StrokeKind::Inside);

            let d_center = dock_r.center();
            ui.painter().text(pos2(dock_r.left() + 14.0, d_center.y), Align2::CENTER_CENTER, icon::MUSIC_NOTE, glyph(11.0), if app.saved.game_show_music_widget { pal_color } else { MUTED });
            ui.painter().text(pos2(dock_r.left() + 30.0, d_center.y), Align2::CENTER_CENTER, icon::LIST_NUMBERS, glyph(11.0), if app.saved.game_show_queue_widget { pal_color } else { MUTED });
            ui.painter().text(pos2(dock_r.left() + 46.0, d_center.y), Align2::CENTER_CENTER, icon::SPEAKER_HIGH, glyph(11.0), if app.saved.game_show_audio_widget { pal_color } else { MUTED });

            ui.painter().text(pos2(d_center.x + 16.0, d_center.y), Align2::CENTER_CENTER, "RETURN TO GAME", bold(8.5), pal_color);
            ui.painter().text(pos2(dock_r.right() - 14.0, d_center.y), Align2::CENTER_CENTER, icon::SIGN_OUT, glyph(11.0), Color32::from_rgb(239, 68, 68));

            ui.add_space(10.0);
            ui.label(RichText::new("Overlay widgets can be freely dragged in-game. Positions are saved automatically.").font(sans(10.5)).color(MUTED));
            ui.add_space(8.0);

            // Reset Widget Positions button
            let (pos_btn, pos_resp) = ui.allocate_exact_size(vec2(card_w, 28.0), Sense::click());
            let p_hov = hover_of(ui, &pos_resp);
            ui.painter().rect_filled(pos_btn, 6.0, veil(0.06 * p_hov));
            ui.painter().text(pos_btn.center(), Align2::CENTER_CENTER, "↺ Reset Widget Positions", medium(11.5), if p_hov > 0.1 { pal_color } else { MUTED });
            if pos_resp.clicked() {
                out.push(Action::ResetGameWidgetPositions);
            }
        });
    });
}

fn render_gaming_controls(ui: &mut Ui, app: &App, out: &mut Vec<Action>, w: f32) {
    let pal_color = app.saved.theme_palette.color();
    ui.vertical(|ui| {
        ui.set_width(w);

        // Header Preview Toggle Pill
        let (preview_pill, _) = ui.allocate_exact_size(vec2(w, 28.0), Sense::hover());
        let btn_w = 140.0;
        let btn_rect = Rect::from_min_size(pos2(preview_pill.right() - btn_w, preview_pill.center().y - 13.0), vec2(btn_w, 26.0));
        let btn_resp = ui.interact(btn_rect, Id::new("settings-gaming-toggle-preview-btn"), Sense::click());
        let hov = hover_of(ui, &btn_resp);
        let preview_active = app.settings_preview_open && !app.saved.queue_open;
        ui.painter().rect_filled(btn_rect, 13.0, if preview_active { pal_color.gamma_multiply(0.20) } else { veil(0.06 + 0.04 * hov) });
        ui.painter().rect_stroke(btn_rect, 13.0, Stroke::new(1.0_f32, if preview_active { pal_color } else { veil(0.12) }), StrokeKind::Inside);
        let text = if preview_active { "✕ Close Preview" } else { "👁 Live Preview ➔" };
        ui.painter().text(btn_rect.center(), Align2::CENTER_CENTER, text, bold(11.0), if preview_active { pal_color } else { if hov > 0.1 { TEXT } else { MUTED } });
        if btn_resp.clicked() {
            out.push(Action::ToggleSettingsPreview);
        }
        ui.add_space(6.0);

        // Card 1: INSTANT ACTIVATION & CONTROLS
        settings_card_panel(ui, "game-card-activation", "Instant gaming mode", "Zero-overhead background music mode designed for competitive gaming.", |ui| {
            let btn_rect = Rect::from_min_size(ui.cursor().min, vec2(ui.available_width(), 38.0));
            let btn_resp = ui.interact(btn_rect, Id::new("enter-gaming-mode-cta"), Sense::click());
            let b_hov = hover_of(ui, &btn_resp);
            ui.painter().rect_filled(btn_rect, 8.0, pal_color.lerp_to_gamma(Color32::WHITE, 0.15 * b_hov));
            ui.painter().text(btn_rect.center(), Align2::CENTER_CENTER, "⚡ Enter Gaming Mode Now", bold(13.0), contrast_text_for(pal_color));
            ui.allocate_exact_size(vec2(ui.available_width(), 38.0), Sense::hover());
            if btn_resp.clicked() {
                out.push(Action::ToggleGameMode);
            }
            ui.add_space(6.0);
            ui.label(RichText::new("Minimizes AURI and detaches from the taskbar. Music continues in the background with ~0% CPU. Press the hotkey in-game to access the HUD.").font(sans(11.0)).color(MUTED));
            ui.add_space(10.0);

            if let Some(hotkey) = combo_row(
                ui,
                "game-overlay-hotkey-sel",
                icon::KEYBOARD,
                "Overlay hotkey",
                "System hotkey that toggles the translucent overlay in-game.",
                app.saved.game_overlay_hotkey,
                &[
                    (crate::app::GameOverlayHotkey::CtrlShiftG, "Ctrl + Shift + G (Recommended)"),
                    (crate::app::GameOverlayHotkey::ShiftTab, "Shift + Tab (Steam style)"),
                    (crate::app::GameOverlayHotkey::F9, "F9"),
                    (crate::app::GameOverlayHotkey::F12, "F12"),
                    (crate::app::GameOverlayHotkey::CtrlF1, "Ctrl + F1"),
                ],
            ) {
                out.push(Action::SetGameOverlayHotkey(hotkey));
            }

            if let Some(op) = slider_row(
                ui,
                "game-overlay-opacity",
                icon::EYEDROPPER,
                "Backdrop opacity",
                "Darkness of the translucent game backdrop when the overlay is active.",
                app.saved.game_overlay_opacity,
                0.15..=0.90,
                |v| format!("{:.0}%", v * 100.0),
            ) {
                out.push(Action::SetGameOverlayOpacity(op));
            }
        });

        // Card 2: AUTO GAME DETECTION, WHITELIST & BLACKLIST
        settings_card_panel(ui, "game-card-detection", "Auto game detection & lists", "Automatically activates Gaming Mode when you launch a game.", |ui| {
            if toggle_row(
                ui,
                "game-auto-detect",
                icon::GAME_CONTROLLER,
                "Auto-detect full-screen games",
                "Enters zero-overhead mode when a game is in the foreground and restores AURI when done.",
                app.saved.auto_game_mode,
            ) {
                out.push(Action::SetAutoGameMode(!app.saved.auto_game_mode));
            }
            ui.add_space(8.0);

            // Whitelist subsection
            ui.label(RichText::new("Game Whitelist (Force Gaming Mode)").font(medium(12.5)).color(TEXT));
            ui.label(RichText::new("Foreground processes that immediately trigger Gaming Mode (even if windowed).").font(sans(10.5)).color(MUTED));
            ui.add_space(4.0);

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                for (idx, game) in app.saved.game_whitelist.iter().enumerate() {
                    let (chip_r, resp) = ui.allocate_exact_size(vec2(game.len() as f32 * 7.0 + 36.0, 24.0), Sense::click());
                    let chov = hover_of(ui, &resp);
                    ui.painter().rect_filled(chip_r, 12.0, veil(0.08 + 0.06 * chov));
                    ui.painter().rect_stroke(chip_r, 12.0, Stroke::new(1.0_f32, pal_color.gamma_multiply(0.40)), StrokeKind::Inside);
                    ui.painter().text(pos2(chip_r.left() + 10.0, chip_r.center().y), Align2::LEFT_CENTER, game, medium(11.0), TEXT);
                    let x_pos = pos2(chip_r.right() - 10.0, chip_r.center().y);
                    ui.painter().text(x_pos, Align2::CENTER_CENTER, "×", bold(13.0), if chov > 0.1 { Color32::from_rgb(239, 68, 68) } else { MUTED });
                    if resp.clicked() {
                        out.push(Action::RemoveGameWhitelist(idx));
                    }
                }
            });
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                let mut wl_input = ui.data_mut(|d| d.get_temp::<String>(Id::new("temp-game-wl-input")).unwrap_or_default());
                let edit = egui::TextEdit::singleline(&mut wl_input)
                    .hint_text("Executable or title (e.g. cs2.exe)")
                    .desired_width(ui.available_width() - 95.0)
                    .font(sans(11.5));
                let res = ui.add(edit);
                let add_btn = ui.add_sized([85.0, 26.0], egui::Button::new(RichText::new("+ Add Game").font(medium(11.5)).color(pal_color)));
                if (add_btn.clicked() || (res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) && !wl_input.trim().is_empty() {
                    out.push(Action::AddGameWhitelist(wl_input.clone()));
                    wl_input.clear();
                }
                ui.data_mut(|d| d.insert_temp(Id::new("temp-game-wl-input"), wl_input));
            });
            ui.add_space(14.0);

            // Blacklist subsection
            ui.label(RichText::new("App Blacklist (Never Trigger Gaming Mode)").font(medium(12.5)).color(TEXT));
            ui.label(RichText::new("Desktop apps that must NEVER trigger gaming mode (e.g. browsers, media players, IDEs).").font(sans(10.5)).color(MUTED));
            ui.add_space(4.0);

            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                for (idx, b_app) in app.saved.game_blacklist.iter().enumerate() {
                    let (chip_r, resp) = ui.allocate_exact_size(vec2(b_app.len() as f32 * 7.0 + 36.0, 24.0), Sense::click());
                    let chov = hover_of(ui, &resp);
                    ui.painter().rect_filled(chip_r, 12.0, veil(0.08 + 0.06 * chov));
                    ui.painter().rect_stroke(chip_r, 12.0, Stroke::new(1.0_f32, veil(0.12)), StrokeKind::Inside);
                    ui.painter().text(pos2(chip_r.left() + 10.0, chip_r.center().y), Align2::LEFT_CENTER, b_app, medium(11.0), TEXT);
                    let x_pos = pos2(chip_r.right() - 10.0, chip_r.center().y);
                    ui.painter().text(x_pos, Align2::CENTER_CENTER, "×", bold(13.0), if chov > 0.1 { Color32::from_rgb(239, 68, 68) } else { MUTED });
                    if resp.clicked() {
                        out.push(Action::RemoveGameBlacklist(idx));
                    }
                }
            });
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                let mut bl_input = ui.data_mut(|d| d.get_temp::<String>(Id::new("temp-game-bl-input")).unwrap_or_default());
                let edit = egui::TextEdit::singleline(&mut bl_input)
                    .hint_text("Executable (e.g. vlc.exe, msedge.exe)")
                    .desired_width(ui.available_width() - 95.0)
                    .font(sans(11.5));
                let res = ui.add(edit);
                let add_btn = ui.add_sized([85.0, 26.0], egui::Button::new(RichText::new("+ Add App").font(medium(11.5)).color(MUTED)));
                if (add_btn.clicked() || (res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) && !bl_input.trim().is_empty() {
                    out.push(Action::AddGameBlacklist(bl_input.clone()));
                    bl_input.clear();
                }
                ui.data_mut(|d| d.insert_temp(Id::new("temp-game-bl-input"), bl_input));
            });
        });

        // Card 3: FLOATING OVERLAY WIDGETS
        settings_card_panel(ui, "game-card-widgets", "Floating overlay widgets", "Enable or disable the individual movable HUD widgets.", |ui| {
            if toggle_row(
                ui,
                "game-widget-music",
                icon::MUSIC_NOTE,
                "Now Playing music widget",
                "Draggable floating card with song cover art, title, seek scrubber and media controls.",
                app.saved.game_show_music_widget,
            ) {
                out.push(Action::ToggleGameMusicWidget);
            }
            if toggle_row(
                ui,
                "game-widget-queue",
                icon::LIST,
                "Upcoming queue widget",
                "Draggable floating card with upcoming songs and one-click track jumping.",
                app.saved.game_show_queue_widget,
            ) {
                out.push(Action::ToggleGameQueueWidget);
            }
            if toggle_row(
                ui,
                "game-widget-audio",
                icon::SPEAKER_HIGH,
                "Audio & performance widget",
                "Draggable floating card with the master volume slider and an instant RAM purge button.",
                app.saved.game_show_audio_widget,
            ) {
                out.push(Action::ToggleGameAudioWidget);
            }
        });
    });
}

fn gaming_mode_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    // The live preview renders in the right rail (the queue dock slot), so the
    // controls always keep the full 740px settings column.
    render_gaming_controls(ui, app, out, ui.available_width().min(740.0));
}

fn hotkey_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    settings_card_panel(
        ui,
        "hotkeys-card",
        "Global hotkeys",
        "System-wide shortcuts that keep working while AURI is minimized or in-game.",
        |ui| {
            if !hotkeys::SUPPORTED {
                ui.label(
                    RichText::new("Global hotkeys are only available on Windows for now.")
                        .font(sans(12.5))
                        .color(MUTED),
                );
                return;
            }
            if toggle_row(ui, "hotkeys-master", icon::KEYBOARD, "Enable global hotkeys", "", app.saved.hotkeys) {
                out.push(Action::GlobalHotkeys(!app.saved.hotkeys));
            }
            let on = ui.ctx().animate_bool_with_time(Id::new("hotkeys-on"), app.saved.hotkeys, 0.16);
            ui.multiply_opacity(egui::lerp(0.4..=1.0, on));
            for command in Command::ALL {
                hotkey_row(ui, app, command, app.saved.hotkeys, out);
            }
            if app.saved.hotkey_combos != hotkeys::standard_combos()
                && button_row(
                    ui,
                    "hotkeys-reset",
                    icon::ARROWS_CLOCKWISE,
                    "Reset all hotkeys",
                    "Restore the default key bindings.",
                    "Reset",
                    false,
                )
            {
                out.push(Action::ResetHotkeys);
            }
        },
    );
    if hotkeys::SUPPORTED {
        ui.label(
            RichText::new("Each hotkey needs Ctrl, Alt or Shift as a modifier.")
                .font(sans(12.0))
                .color(FAINT),
        );
        ui.add_space(4.0);
    }
}

fn audio_dsp_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    let pal_color = app.saved.theme_palette.color();
    settings_card_panel(ui, "eq-loudness", "Loudness & limiting", "Uniform, clipping-free volume across every track.", |ui| {
        if toggle_row(
            ui,
            "loudness-toggle",
            icon::WAVEFORM,
            "Studio loudness normalizer (-14.0 LUFS)",
            "Normalizes tracks from YouTube Music broadcast loudness metadata for uniform listening.",
            app.saved.loudness_normalizer,
        ) {
            out.push(Action::ToggleLoudness);
        }
        row_base(
            ui,
            icon::FADERS,
            "Soft-knee peak limiter (active)",
            "Prevents clipping and harsh distortion on loud transients with studio-grade brickwall limiting.",
        );
    });

    settings_card_panel(
        ui,
        "eq-card",
        "10-band graphic equalizer",
        "Precision biquad peaking filter bank (31 Hz - 16 kHz) for studio mastering and gaming audio tuning.",
        |ui| {
            if toggle_row(ui, "eq-toggle", icon::EQUALIZER, "Enable equalizer", "", app.saved.eq_enabled) {
                out.push(Action::ToggleEq(!app.saved.eq_enabled));
            }
            if app.saved.eq_enabled {
                let size = vec2(ui.available_width(), SETTING_ROW);
                ui.add_space(2.0);

                if let Some(preset) = segmented_row(
                    ui,
                    "eq-preset-seg",
                    icon::SLIDERS,
                    "Presets",
                    "",
                    app.saved.eq_preset,
                    &[
                        (crate::dsp::EqPreset::Flat, "Flat"),
                        (crate::dsp::EqPreset::FpsFootsteps, "FPS Footsteps"),
                        (crate::dsp::EqPreset::BassPunch, "Bass Punch"),
                        (crate::dsp::EqPreset::VocalClarity, "Vocal"),
                        (crate::dsp::EqPreset::Cinematic, "Cinematic"),
                        (crate::dsp::EqPreset::Custom, "Custom"),
                    ],
                ) {
                    out.push(Action::SetEqPreset(preset));
                }

                if let Some(gain) = slider_row(
                    ui,
                    "eq-preamp",
                    icon::FADERS,
                    "Preamp",
                    "Global gain applied before the equalizer filters.",
                    app.saved.eq_preamp_db,
                    -12.0..=12.0,
                    |v| format!("{:+.1} dB", (v * 2.0).round() / 2.0),
                ) {
                    out.push(Action::SetEqPreamp((gain * 2.0).round() / 2.0));
                }

            ui.add_space(12.0);

            // 3. Connected Graph Model (Image 1)
            let graph_h = 220.0;
            let (graph_rect, _) = ui.allocate_exact_size(vec2(size.x, graph_h), Sense::hover());
            ui.painter().rect_filled(graph_rect, 14.0, veil(0.03));
            ui.painter().rect_stroke(graph_rect, 14.0, Stroke::new(1.0_f32, veil(0.08)), StrokeKind::Inside);

            let g_left = graph_rect.left() + 52.0;
            let g_right = graph_rect.right() - 24.0;
            let g_width = g_right - g_left;
            let y_center = graph_rect.top() + (graph_h - 32.0) / 2.0 + 12.0;
            let y_range = 65.0_f32;

            // Y-Axis labels (+12dB, 0dB, -12dB)
            ui.painter().text(pos2(graph_rect.left() + 14.0, y_center - y_range), Align2::LEFT_CENTER, "+12dB", sans(10.5), MUTED);
            ui.painter().text(pos2(graph_rect.left() + 14.0, y_center), Align2::LEFT_CENTER, "0dB", sans(10.5), MUTED);
            ui.painter().text(pos2(graph_rect.left() + 14.0, y_center + y_range), Align2::LEFT_CENTER, "-12dB", sans(10.5), MUTED);

            // 0 dB centerline
            ui.painter().hline(g_left - 10.0..=g_right + 10.0, y_center, Stroke::new(1.0_f32, veil(0.12)));

            // 10 Frequency positions and data points
            let mut pts = [Pos2::ZERO; 10];
            for b in 0..10 {
                let bx = g_left + b as f32 * (g_width / 9.0);
                let gain = app.saved.eq_gains[b];
                let by = y_center - (gain / 12.0) * y_range;
                pts[b] = pos2(bx, by);

                // Vertical grid lines
                ui.painter().vline(bx, (y_center - y_range - 4.0)..=(graph_rect.bottom() - 32.0), Stroke::new(1.0_f32, veil(0.06)));

                // Frequency label at bottom
                let freq_label = crate::dsp::EQ_BAND_LABELS[b];
                ui.painter().text(pos2(bx, graph_rect.bottom() - 16.0), Align2::CENTER_CENTER, freq_label, sans(10.5), MUTED);
            }

            // Catmull-Rom spline curve points
            let mut curve_pts = Vec::with_capacity(120);
            for i in 0..9 {
                let p0 = if i == 0 { pts[0] - (pts[1] - pts[0]) } else { pts[i - 1] };
                let p1 = pts[i];
                let p2 = pts[i + 1];
                let p3 = if i + 2 < 10 { pts[i + 2] } else { pts[9] + (pts[9] - pts[8]) };

                let steps = 14;
                for s in 0..steps {
                    let t = s as f32 / steps as f32;
                    let t2 = t * t;
                    let t3 = t2 * t;
                    let f0 = -0.5 * t3 + t2 - 0.5 * t;
                    let f1 = 1.5 * t3 - 2.5 * t2 + 1.0;
                    let f2 = -1.5 * t3 + 2.0 * t2 + 0.5 * t;
                    let f3 = 0.5 * t3 - 0.5 * t2;
                    let px = p0.x * f0 + p1.x * f1 + p2.x * f2 + p3.x * f3;
                    let py = p0.y * f0 + p1.y * f1 + p2.y * f2 + p3.y * f3;
                    curve_pts.push(pos2(px, py));
                }
            }
            curve_pts.push(pts[9]);

            // Area fill under curve down to baseline (or bottom of graph)
            if curve_pts.len() >= 2 {
                let mut mesh = egui::Mesh::default();
                let col_top = pal_color.gamma_multiply(0.25);
                let col_bot = Color32::from_rgba_unmultiplied(pal_color.r(), pal_color.g(), pal_color.b(), 12);
                let base_y = graph_rect.bottom() - 32.0;

                for w in 0..curve_pts.len() - 1 {
                    let p_a = curve_pts[w];
                    let p_b = curve_pts[w + 1];
                    let v_idx = mesh.vertices.len() as u32;

                    mesh.vertices.push(egui::epaint::Vertex { pos: p_a, uv: Pos2::ZERO, color: col_top });
                    mesh.vertices.push(egui::epaint::Vertex { pos: p_b, uv: Pos2::ZERO, color: col_top });
                    mesh.vertices.push(egui::epaint::Vertex { pos: pos2(p_b.x, base_y), uv: Pos2::ZERO, color: col_bot });
                    mesh.vertices.push(egui::epaint::Vertex { pos: pos2(p_a.x, base_y), uv: Pos2::ZERO, color: col_bot });

                    mesh.indices.extend_from_slice(&[v_idx, v_idx + 1, v_idx + 2, v_idx, v_idx + 2, v_idx + 3]);
                }
                ui.painter().add(Shape::mesh(mesh));
            }

            // Draw smooth curve line
            ui.painter().add(Shape::line(curve_pts, Stroke::new(2.5_f32, pal_color.lerp_to_gamma(Color32::WHITE, 0.35))));

            // Interactive draggable circle nodes
            for b in 0..10 {
                let pt = pts[b];
                let hit_rect = Rect::from_center_size(pos2(pt.x, y_center), vec2((g_width / 9.0).min(38.0), y_range * 2.2));
                let resp = ui.interact(hit_rect, Id::new(("eq-graph-node", b)), Sense::click_and_drag());
                let hov = hover_of(ui, &resp);

                if let (true, Some(pos)) = (resp.clicked() || resp.dragged(), resp.interact_pointer_pos()) {
                    let frac = ((y_center - pos.y) / y_range).clamp(-1.0, 1.0);
                    let mut new_gain = frac * 12.0;
                    new_gain = (new_gain * 2.0).round() / 2.0;
                    out.push(Action::SetEqGain(b, new_gain));
                }

                let radius = 6.0 + 2.0 * hov;
                ui.painter().circle_filled(pt, radius + 2.0, veil(0.20 * hov));
                ui.painter().circle_filled(pt, radius, pal_color.lerp_to_gamma(Color32::WHITE, 0.40));
                ui.painter().circle_stroke(pt, radius, Stroke::new(1.0_f32, Color32::WHITE));

                if hov > 0.05 {
                    let sign = if app.saved.eq_gains[b] > 0.0 { "+" } else { "" };
                    describe(&resp, &format!("{}: {}{:.1} dB", crate::dsp::EQ_BAND_LABELS[b], sign, app.saved.eq_gains[b]));
                }
            }

            ui.add_space(8.0);

            if button_row(
                ui,
                "eq-reset-btn",
                icon::ARROWS_CLOCKWISE,
                "Reset equalizer",
                "Restore the Flat preset and zero the preamp and bands.",
                "Reset",
                false,
            ) {
                out.push(Action::ResetEq);
            }
        }
    });
}

fn performance_settings_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    let live_ram = get_live_ram_mb();
    settings_card_panel(
        ui,
        "perf-engine",
        "Extreme performance engine",
        "Hardware resource limits for maximum in-game FPS and zero background lag.",
        |ui| {
            if toggle_row(
                ui,
                "perf-master",
                icon::LIGHTNING,
                "Master performance mode",
                "Enforces the resource limits below and suspends repaint-heavy animations.",
                app.saved.performance_mode,
            ) {
                out.push(Action::TogglePerformanceMode);
            }
            if let Some(profile) = segmented_row(
                ui,
                "perf-profile",
                icon::SPEEDOMETER,
                "Profile",
                "Presets retune every option on this page; Custom keeps your own choices.",
                app.saved.perf_profile,
                &[
                    (PerfProfile::CompetitiveGaming, "Competitive Gaming"),
                    (PerfProfile::Balanced, "Balanced"),
                    (PerfProfile::Audiophile, "Audiophile"),
                    (PerfProfile::Custom, "Custom"),
                ],
            ) {
                out.push(Action::SetPerfProfile(profile));
            }
        },
    );

    settings_card_panel(
        ui,
        "perf-tuning",
        "Resource tuning",
        "Fine-grained budgets. Changing any option switches the profile to Custom.",
        |ui| {
            if let Some(mb) = segmented_row(
                ui,
                "perf-audio-buffer",
                icon::DATABASE,
                "Audio stream buffer",
                "Sliding memory window used while streaming a track.",
                app.saved.perf_audio_buffer_mb,
                &[(1.5, "1.5 MB"), (4.0, "4 MB"), (8.0, "8 MB"), (16.0, "16 MB")],
            ) {
                out.push(Action::SetPerfAudioBuffer(mb));
            }
            if let Some(mb) = segmented_row(
                ui,
                "perf-art-cache",
                icon::IMAGE,
                "Cover art cache budget",
                "Texture memory reserved for album artwork.",
                app.saved.perf_art_cache_mb,
                &[(2, "2 MB"), (4, "4 MB"), (12, "12 MB"), (32, "32 MB")],
            ) {
                out.push(Action::SetPerfArtCache(mb));
            }
            if let Some(fps) = segmented_row(
                ui,
                "perf-tick-fps",
                icon::SPEEDOMETER,
                "HUD refresh rate",
                "How often the UI repaints while playing in performance mode.",
                app.saved.perf_tick_fps,
                &[(1, "1 FPS (static)"), (30, "30 FPS"), (60, "60 FPS (fluid)")],
            ) {
                out.push(Action::SetPerfTickFps(fps));
            }
            if toggle_row(
                ui,
                "perf-aggressive-ram",
                icon::BROOM,
                "Aggressive RAM purge",
                "Trims the working set every 30 seconds and on every track switch.",
                app.saved.perf_aggressive_ram,
            ) {
                out.push(Action::SetPerfAggressiveRam(!app.saved.perf_aggressive_ram));
            }
            if toggle_row(
                ui,
                "perf-purge-minimized",
                icon::BROOM,
                "Purge RAM when minimized",
                "Trims the working set whenever the window is minimized.",
                app.saved.auto_purge_ram,
            ) {
                out.push(Action::AutoPurgeRam(!app.saved.auto_purge_ram));
            }
            if toggle_row(
                ui,
                "perf-disable-blurs",
                icon::EYE_SLASH,
                "Disable ambient blurs & background glows",
                "Removes artwork-tinted washes and backdrop effects.",
                app.saved.perf_disable_blurs,
            ) {
                out.push(Action::SetPerfDisableBlurs(!app.saved.perf_disable_blurs));
            }
            if toggle_row(
                ui,
                "perf-compact-buffers",
                icon::DATABASE,
                "Compact audio & stream buffers",
                "Shrinks the sliding stream window below 2 MB.",
                app.saved.perf_compact_buffers,
            ) {
                out.push(Action::SetPerfCompactBuffers(!app.saved.perf_compact_buffers));
            }
            if toggle_row(
                ui,
                "perf-placeholder-mood",
                icon::IMAGE,
                "Lightweight vector mood art",
                "Saves RAM and network by using vector mood artwork.",
                app.saved.perf_placeholder_mood_art,
            ) {
                out.push(Action::SetPerfPlaceholderMoodArt(!app.saved.perf_placeholder_mood_art));
            }
        },
    );

    settings_card_panel(ui, "perf-live", "Live memory", "Physical RAM AURI is using right now.", |ui| {
        let row = row_base(ui, icon::CPU, "Physical working set", &format!("{:.1} MB", live_ram));
        let btn = Rect::from_min_size(pos2(row.right() - 110.0, row.center().y - 14.0), vec2(104.0, 28.0));
        let resp = ui.interact(btn, Id::new("perf-purge-now"), Sense::click());
        let hover = hover_of(ui, &resp);
        let accent = ui.ctx().data(|d| d.get_temp(Id::new("app-theme-accent"))).unwrap_or(ACCENT);
        ui.painter().rect_filled(btn, 14.0, accent.gamma_multiply(0.18 + 0.12 * hover));
        ui.painter().rect_stroke(btn, 14.0, Stroke::new(1.0_f32, accent), StrokeKind::Inside);
        ui.painter().text(
            btn.center(),
            Align2::CENTER_CENTER,
            "Purge now",
            bold(12.0),
            if hover > 0.1 { TEXT } else { accent },
        );
        describe(&resp, "Purge working set memory now");
        if resp.clicked() {
            out.push(Action::PurgeMemory);
        }
    });
}

fn mini_mode_settings_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    let pal_color = app.saved.mini_palette.color();

    // 1. Live Preview Card
    let prev_height = if app.saved.mini_variant == MiniVariant::StudioCard { 230.0 } else { 160.0 };
    settings_card_panel(ui, "mini-preview", "Live mini mode preview", "Dynamically reflects your opacity, tint, accent, grain and tools.", |ui| {
        let size = vec2(ui.available_width(), prev_height);
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
                    paint_glass_grain(ui.painter(), mini_rect, 14.0, app.saved.mini_grain_strength);
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
                    paint_glass_grain(ui.painter(), mini_rect, 27.0, app.saved.mini_grain_strength);
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
                    paint_glass_grain(ui.painter(), mini_rect, 14.0, app.saved.mini_grain_strength);
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

    settings_card_panel(
        ui,
        "mini-customize",
        "Variant, theme & features",
        "Pick the HUD layout, accent, background tint and which tools it shows.",
        |ui| {
            let size = vec2(ui.available_width(), SETTING_ROW);

            if let Some(variant) = segmented_row(
                ui,
                "mini-variant-seg",
                icon::PICTURE_IN_PICTURE,
                "Mini player variant",
                "",
                app.saved.mini_variant,
                &[
                    (MiniVariant::ClassicHud, "Classic HUD"),
                    (MiniVariant::CapsulePill, "Capsule Pill"),
                    (MiniVariant::StudioCard, "Studio Card"),
                ],
            ) {
                out.push(Action::SetMiniVariant(variant));
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
            if let Some(op) = slider_row(
                ui,
                "mini-opacity",
                icon::EYEDROPPER,
                "Glass card opacity",
                "",
                app.saved.mini_opacity,
                0.55..=1.0,
                |v| format!("{:.0}%", v * 100.0),
            ) {
                out.push(Action::SetMiniOpacity(op));
            }
            if toggle_row(
                ui,
                "mini-grain",
                icon::SPARKLE,
                "Frosted acrylic grain effect",
                "Adds a subtle procedural noise texture to the glass card.",
                app.saved.mini_grain,
            ) {
                out.push(Action::SetMiniGrain(!app.saved.mini_grain));
            }
            if app.saved.mini_grain {
                if let Some(strength) = slider_row(
                    ui,
                    "mini-grain-strength",
                    icon::WAVEFORM,
                    "Grain intensity",
                    "",
                    app.saved.mini_grain_strength,
                    0.05..=0.35,
                    |v| format!("{:.0}%", v * 100.0),
                ) {
                    out.push(Action::SetMiniGrainStrength(strength));
                }
            }

            // Feature toggles
            if toggle_row(ui, "mini-shuffle-repeat", icon::SHUFFLE, "Show shuffle & repeat controls", "", app.saved.mini_show_shuffle_repeat) {
                out.push(Action::SetMiniShowShuffleRepeat(!app.saved.mini_show_shuffle_repeat));
            }
            if toggle_row(ui, "mini-queue", icon::LIST, "Show upcoming queue drawer button", "", app.saved.mini_show_queue) {
                out.push(Action::SetMiniShowQueue(!app.saved.mini_show_queue));
            }
            if toggle_row(ui, "mini-volume", icon::SPEAKER_HIGH, "Show mini volume slider", "", app.saved.mini_show_volume) {
                out.push(Action::SetMiniShowVolume(!app.saved.mini_show_volume));
            }
            if toggle_row(ui, "mini-clock", icon::CLOCK, "Show track clock & elapsed time", "", app.saved.mini_show_clock) {
                out.push(Action::SetMiniShowClock(!app.saved.mini_show_clock));
            }
            if toggle_row(ui, "mini-always-top", icon::PUSH_PIN, "Always on top window overlay", "", app.saved.mini_always_on_top) {
                out.push(Action::SetMiniAlwaysOnTop(!app.saved.mini_always_on_top));
            }
            if toggle_row(
                ui,
                "mini-locked",
                icon::LOCK,
                "Lock mini player position",
                "Disables dragging the floating HUD.",
                app.saved.mini_locked,
            ) {
                out.push(Action::ToggleMiniLock);
            }
            if app.saved.mini_variant == MiniVariant::CapsulePill {
                if toggle_row(
                    ui,
                    "mini-orb",
                    icon::ARROWS_IN_SIMPLE,
                    "Retractable orb sub-mode",
                    "Rests as a compact spinning vinyl circle; expands on hover and auto-retracts 5s after the mouse leaves.",
                    app.saved.mini_capsule_retractable,
                ) {
                    out.push(Action::ToggleMiniCapsuleRetractable);
                }
            }
            ui.add_space(4.0);

            if button_row(
                ui,
                "mini-launch",
                icon::PICTURE_IN_PICTURE,
                "Switch to the floating glass mini player",
                "Mini Mode floats over games and apps with zero window borders.",
                "Open mini",
                false,
            ) {
                out.push(Action::ToggleMiniMode);
            }
        },
    );
}

fn render_appearance_preview(ui: &mut Ui, app: &App, out: &mut Vec<Action>, w: f32) {
    let pal_color = app.saved.theme_palette.color();
    ui.vertical(|ui| {
        ui.set_width(w);

        ui.label(RichText::new("LIVE PREVIEW").font(bold(11.5)).color(MUTED));
        ui.add_space(4.0);

        let frame = Frame::new()
            .fill(veil(0.05))
            .stroke(Stroke::new(1.0_f32, veil(0.08)))
            .corner_radius(CARD_RADIUS as u8)
            .inner_margin(Margin::same(12));

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                let (badge_rect, _) = ui.allocate_exact_size(vec2(90.0, 18.0), Sense::hover());
                ui.painter().rect_filled(badge_rect, 9.0, pal_color.gamma_multiply(0.20));
                ui.painter().text(badge_rect.center(), Align2::CENTER_CENTER, app.saved.theme_preset.name(), bold(10.0), pal_color);
            });
            ui.add_space(6.0);

            // Mockup Container
            let card_w = w - 24.0;
            let (mockup_rect, _) = ui.allocate_exact_size(vec2(card_w, 230.0), Sense::hover());
            ui.painter().rect_filled(mockup_rect, 10.0, Color32::from_rgb(18, 19, 24));
            ui.painter().rect_stroke(mockup_rect, 10.0, Stroke::new(1.0_f32, Color32::from_rgb(38, 40, 48)), StrokeKind::Inside);

            // Header in preview: Accent Dot + "Good afternoon" + "Play all" button
            let p_header_y = mockup_rect.top() + 18.0;
            ui.painter().circle_filled(pos2(mockup_rect.left() + 16.0, p_header_y), 5.0, pal_color);
            label(ui, pos2(mockup_rect.left() + 26.0, p_header_y), Align2::LEFT_CENTER, "Good afternoon", bold(12.5), TEXT, 120.0);

            let play_all_r = Rect::from_min_size(pos2(mockup_rect.right() - 80.0, p_header_y - 11.0), vec2(68.0, 22.0));
            ui.painter().rect_filled(play_all_r, 11.0, pal_color);
            ui.painter().text(play_all_r.center(), Align2::CENTER_CENTER, "Play all", bold(10.0), contrast_text_for(pal_color));

            // 3 Card Skeletons representing playlists
            let cards_y = mockup_rect.top() + 42.0;
            let c_w = (card_w - 36.0) / 3.0;
            for c_i in 0..3 {
                let c_r = Rect::from_min_size(pos2(mockup_rect.left() + 12.0 + c_i as f32 * (c_w + 6.0), cards_y), vec2(c_w, 88.0));
                ui.painter().rect_filled(c_r, 8.0, veil(0.06));
                ui.painter().rect_stroke(c_r, 8.0, Stroke::new(1.0_f32, veil(0.08)), StrokeKind::Inside);

                ui.painter().rect_filled(Rect::from_min_size(pos2(c_r.left() + 6.0, c_r.bottom() - 18.0), vec2(c_w - 12.0, 5.0)), 2.5, veil(0.12));
                ui.painter().rect_filled(Rect::from_min_size(pos2(c_r.left() + 6.0, c_r.bottom() - 10.0), vec2(c_w - 22.0, 4.0)), 2.0, veil(0.08));
            }

            // Mini Player Bar at bottom of mockup
            let bar_r = Rect::from_min_size(pos2(mockup_rect.left() + 8.0, mockup_rect.bottom() - 56.0), vec2(card_w - 16.0, 48.0));
            ui.painter().rect_filled(bar_r, 8.0, Color32::from_rgb(26, 28, 35));
            ui.painter().rect_stroke(bar_r, 8.0, Stroke::new(1.0_f32, veil(0.12)), StrokeKind::Inside);

            let current_song = app.saved.current.and_then(|c| app.saved.queue.get(c));
            let live_title = current_song.map_or("Mangal Rang (From \"Ramayana\")", |s| s.title.as_str());
            let live_artist = current_song.map_or("A.R. Rahman, Arijit Singh...", |s| s.subtitle.as_str());

            let m_thumb = Rect::from_min_size(pos2(bar_r.left() + 6.0, bar_r.top() + 6.0), Vec2::splat(36.0));
            let mut drew = false;
            if let Some(cover_str) = current_song.map(|s| app.saved.covers.of(s)) {
                if let Some(pic) = app.art.get(ui.ctx(), cover_str, 1.0) {
                    egui::Image::from_texture((pic.id, m_thumb.size())).corner_radius(4.0).paint_at(ui, m_thumb);
                    drew = true;
                }
            }
            if !drew {
                ui.painter().rect_filled(m_thumb, 4.0, Color32::from_rgb(38, 40, 50));
                ui.painter().text(m_thumb.center(), Align2::CENTER_CENTER, icon::MUSIC_NOTE, glyph(14.0), MUTED);
            }

            label(ui, pos2(m_thumb.right() + 8.0, bar_r.top() + 10.0), Align2::LEFT_CENTER, live_title, bold(10.5), TEXT, bar_r.width() - 75.0);
            label(ui, pos2(m_thumb.right() + 8.0, bar_r.top() + 23.0), Align2::LEFT_CENTER, live_artist, sans(9.0), MUTED, bar_r.width() - 75.0);

            // Scrubber line
            let pb_x1 = m_thumb.right() + 8.0;
            let pb_x2 = bar_r.right() - 26.0;
            let pb_fill = pb_x1 + (pb_x2 - pb_x1) * 0.45;
            ui.painter().hline(pb_x1..=pb_x2, bar_r.bottom() - 9.0, Stroke::new(2.5_f32, veil(0.15)));
            ui.painter().hline(pb_x1..=pb_fill, bar_r.bottom() - 9.0, Stroke::new(2.5_f32, pal_color));
            ui.painter().circle_filled(pos2(pb_fill, bar_r.bottom() - 9.0), 3.0, pal_color);

            ui.painter().circle_filled(pos2(bar_r.right() - 13.0, bar_r.center().y), 8.5, pal_color);
            ui.painter().text(pos2(bar_r.right() - 13.0, bar_r.center().y), Align2::CENTER_CENTER, icon::PLAY, glyph(8.5), contrast_text_for(pal_color));

            ui.add_space(10.0);
            ui.label(RichText::new("Sliders and accent swatches update the preview live; changes apply to AURI immediately.").font(sans(10.5)).color(MUTED));
            ui.add_space(8.0);

            // Reset button
            let (reset_rect, reset_resp) = ui.allocate_exact_size(vec2(card_w, 28.0), Sense::click());
            let r_hov = hover_of(ui, &reset_resp);
            ui.painter().rect_filled(reset_rect, 6.0, veil(0.06 * r_hov));
            ui.painter().text(reset_rect.center(), Align2::CENTER_CENTER, "↺ Reset to defaults", medium(11.5), if r_hov > 0.1 { pal_color } else { MUTED });
            if reset_resp.clicked() {
                out.push(Action::ResetThemeDefaults);
            }
        });
    });
}

fn render_appearance_controls(ui: &mut Ui, app: &App, out: &mut Vec<Action>, w: f32) {
    let pal_color = app.saved.theme_palette.color();
    ui.vertical(|ui| {
        ui.set_width(w);

        // Header Preview Toggle Pill
        let (preview_pill, _) = ui.allocate_exact_size(vec2(w, 28.0), Sense::hover());
        let btn_w = 140.0;
        let btn_rect = Rect::from_min_size(pos2(preview_pill.right() - btn_w, preview_pill.center().y - 13.0), vec2(btn_w, 26.0));
        let btn_resp = ui.interact(btn_rect, Id::new("settings-appearance-toggle-preview-btn"), Sense::click());
        let hov = hover_of(ui, &btn_resp);
        let preview_active = app.settings_preview_open && !app.saved.queue_open;
        ui.painter().rect_filled(btn_rect, 13.0, if preview_active { pal_color.gamma_multiply(0.20) } else { veil(0.06 + 0.04 * hov) });
        ui.painter().rect_stroke(btn_rect, 13.0, Stroke::new(1.0_f32, if preview_active { pal_color } else { veil(0.12) }), StrokeKind::Inside);
        let text = if preview_active { "✕ Close Preview" } else { "👁 Live Preview ➔" };
        ui.painter().text(btn_rect.center(), Align2::CENTER_CENTER, text, bold(11.0), if preview_active { pal_color } else { if hov > 0.1 { TEXT } else { MUTED } });
        if btn_resp.clicked() {
            out.push(Action::ToggleSettingsPreview);
        }
        ui.add_space(6.0);

        // Card 1: THEME PRESETS & ACCENT
        settings_card_panel(ui, "app-card-presets", "Theme presets & accent", "Base aesthetic palettes and the primary highlight accent color.", |ui| {
            let presets: [ThemePreset; 7] = [
                ThemePreset::Default,
                ThemePreset::Catppuccin,
                ThemePreset::TokyoNight,
                ThemePreset::Caffeine,
                ThemePreset::Neon,
                ThemePreset::Breeze,
                ThemePreset::Amoled,
            ];

            let pill_h = 30.0;
            let pill_spacing = 6.0;

            let row1 = &presets[0..3];
            let row2 = &presets[3..7];

            let (p_row1, _) = ui.allocate_exact_size(vec2(ui.available_width(), pill_h), Sense::hover());
            let w1 = (ui.available_width() - 2.0 * pill_spacing) / 3.0;
            for (idx, &preset) in row1.iter().enumerate() {
                let p_rect = Rect::from_min_size(pos2(p_row1.left() + idx as f32 * (w1 + pill_spacing), p_row1.top()), vec2(w1, pill_h));
                let resp = ui.interact(p_rect, Id::new(("theme-preset-pill", idx)), Sense::click());
                let hov = hover_of(ui, &resp);
                let is_cur = app.saved.theme_preset == preset;
                ui.painter().rect_filled(p_rect, 15.0, if is_cur { veil(0.12) } else { veil(0.04 + 0.04 * hov) });
                if is_cur {
                    ui.painter().rect_stroke(p_rect, 15.0, Stroke::new(1.5_f32, pal_color), StrokeKind::Inside);
                }
                let dot_col = preset.default_accent().color();
                let dot_center = pos2(p_rect.left() + 14.0, p_rect.center().y);
                ui.painter().circle_filled(dot_center, 5.5, dot_col);
                ui.painter().text(pos2(p_rect.left() + 26.0, p_rect.center().y), Align2::LEFT_CENTER, preset.name(), medium(11.5), if is_cur { TEXT } else { MUTED });
                if resp.clicked() {
                    out.push(Action::SetThemePreset(preset));
                }
            }
            ui.add_space(pill_spacing);

            let (p_row2, _) = ui.allocate_exact_size(vec2(ui.available_width(), pill_h), Sense::hover());
            let w2 = (ui.available_width() - 3.0 * pill_spacing) / 4.0;
            for (idx, &preset) in row2.iter().enumerate() {
                let p_rect = Rect::from_min_size(pos2(p_row2.left() + idx as f32 * (w2 + pill_spacing), p_row2.top()), vec2(w2, pill_h));
                let resp = ui.interact(p_rect, Id::new(("theme-preset-pill-2", idx)), Sense::click());
                let hov = hover_of(ui, &resp);
                let is_cur = app.saved.theme_preset == preset;
                ui.painter().rect_filled(p_rect, 15.0, if is_cur { veil(0.12) } else { veil(0.04 + 0.04 * hov) });
                if is_cur {
                    ui.painter().rect_stroke(p_rect, 15.0, Stroke::new(1.5_f32, pal_color), StrokeKind::Inside);
                }
                let dot_col = preset.default_accent().color();
                let dot_center = pos2(p_rect.left() + 12.0, p_rect.center().y);
                ui.painter().circle_filled(dot_center, 5.5, dot_col);
                ui.painter().text(pos2(p_rect.left() + 22.0, p_rect.center().y), Align2::LEFT_CENTER, preset.name(), medium(11.0), if is_cur { TEXT } else { MUTED });
                if resp.clicked() {
                    out.push(Action::SetThemePreset(preset));
                }
            }
            ui.add_space(10.0);

            // Light or Dark Segmented Control
            ui.horizontal(|ui| {
                ui.label(RichText::new("Light or dark").font(medium(12.5)).color(TEXT));
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    let (mode_rect, _) = ui.allocate_exact_size(vec2(210.0, 28.0), Sense::hover());
                    ui.painter().rect_filled(mode_rect, 14.0, veil(0.06));
                    ui.painter().rect_stroke(mode_rect, 14.0, Stroke::new(1.0_f32, veil(0.10)), StrokeKind::Inside);

                    let modes = [
                        (ThemeMode::Light, "☼ Light"),
                        (ThemeMode::Dark, "☾ Dark"),
                        (ThemeMode::System, "🖥 System"),
                    ];
                    let seg_w = mode_rect.width() / 3.0;
                    for (idx, (m, label_str)) in modes.iter().enumerate() {
                        let seg_r = Rect::from_min_size(pos2(mode_rect.left() + idx as f32 * seg_w, mode_rect.top()), vec2(seg_w, 28.0));
                        let resp = ui.interact(seg_r, Id::new(("theme-mode-seg", idx)), Sense::click());
                        let is_sel = app.saved.theme_mode == *m;
                        if is_sel {
                            ui.painter().rect_filled(seg_r, 12.0, veil(0.16));
                            ui.painter().rect_stroke(seg_r, 12.0, Stroke::new(1.0_f32, pal_color), StrokeKind::Inside);
                        }
                        ui.painter().text(seg_r.center(), Align2::CENTER_CENTER, label_str, bold(11.0), if is_sel { TEXT } else { MUTED });
                        if resp.clicked() {
                            out.push(Action::SetThemeMode(*m));
                        }
                    }
                });
            });
            ui.add_space(10.0);

            // Accent color swatches
            ui.label(RichText::new("Primary Accent").font(medium(12.5)).color(TEXT));
            ui.add_space(4.0);
            let palettes = [
                MiniPalette::SunsetCrimson,
                MiniPalette::AmberGold,
                MiniPalette::NeonEmerald,
                MiniPalette::ElectricCyan,
                MiniPalette::CobaltBlue,
                MiniPalette::CyberViolet,
                MiniPalette::SakuraPink,
                MiniPalette::Monochrome,
            ];

            let (swatch_row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::hover());
            let dot_radius = 12.0;
            let spacing = (ui.available_width() - (palettes.len() as f32 + 1.0) * (dot_radius * 2.0)) / (palettes.len() as f32);

            for (idx, &pal) in palettes.iter().enumerate() {
                let cx = swatch_row.left() + dot_radius + idx as f32 * (dot_radius * 2.0 + spacing);
                let cy = swatch_row.center().y;
                let c_center = pos2(cx, cy);
                let hit_rect = Rect::from_center_size(c_center, Vec2::splat(dot_radius * 2.2));
                let resp = ui.interact(hit_rect, Id::new(("accent-dot", idx)), Sense::click());
                let hov = hover_of(ui, &resp);
                let is_sel = app.saved.theme_palette == pal;
                let col = pal.color();

                ui.painter().circle_filled(c_center, dot_radius + 1.0 * hov, col);
                if is_sel {
                    ui.painter().circle_stroke(c_center, dot_radius + 2.5, Stroke::new(2.0_f32, Color32::WHITE));
                    ui.painter().text(c_center, Align2::CENTER_CENTER, "✓", bold(11.0), contrast_text_for(col));
                }
                if resp.clicked() {
                    out.push(Action::SetThemePalette(pal));
                }
            }
            ui.add_space(8.0);

            // Background tint slider
            if let Some(tint) = slider_row(
                ui,
                "background-tint",
                icon::EYEDROPPER,
                "Background tint",
                "Shades the greys across background panels.",
                app.saved.background_tint_shade,
                0.0..=1.0,
                |v| format!("{:.0}%", v * 100.0),
            ) {
                out.push(Action::SetBackgroundTintShade(tint));
            }
        });

        // Card 2: EFFECTS & AMBIENT
        settings_card_panel(ui, "app-card-effects", "Effects & ambient", "Glows, adaptive artwork tinting and element roundness.", |ui| {
            if toggle_row(
                ui,
                "app-adapt-artwork",
                icon::IMAGE,
                "Adapt colors to artwork",
                "Uses the current album cover to tint the app's background canvas.",
                app.saved.ambient_artwork_blur,
            ) {
                out.push(Action::SetAmbientArtworkBlur(!app.saved.ambient_artwork_blur));
            }

            if app.saved.ambient_artwork_blur {
                if let Some(variant) = segmented_row(
                    ui,
                    "artwork-bg-variant",
                    icon::IMAGE,
                    "Artwork backdrop style",
                    "",
                    app.saved.artwork_bg_variant,
                    &[
                        (crate::app::ArtworkBgVariant::CanvasTint, "Canvas Tint"),
                        (crate::app::ArtworkBgVariant::BlurredBackdrop, "Frosted Blur"),
                        (crate::app::ArtworkBgVariant::AmbientRadialGlow, "Radial Glow"),
                        (crate::app::ArtworkBgVariant::MinimalEdgeGlow, "Edge Glow"),
                    ],
                ) {
                    out.push(Action::SetArtworkBgVariant(variant));
                }

                // Backdrop Opacity slider
                if let Some(op) = slider_row(
                    ui,
                    "artwork-bg-opacity",
                    icon::EYEDROPPER,
                    "Backdrop opacity",
                    "",
                    app.saved.artwork_bg_opacity,
                    0.05..=0.60,
                    |v| format!("{:.0}%", v * 100.0),
                ) {
                    out.push(Action::SetArtworkBgOpacity(op));
                }

                // Backdrop Blur Radius slider
                if let Some(blur) = slider_row(
                    ui,
                    "artwork-bg-blur",
                    icon::DROP,
                    "Backdrop blur radius",
                    "",
                    app.saved.artwork_bg_blur,
                    0.1..=1.0,
                    |v| format!("{:.0}%", v * 100.0),
                ) {
                    out.push(Action::SetArtworkBgBlur(blur));
                }
            }

            if toggle_row(
                ui,
                "app-ambient-glow",
                icon::SPARKLE,
                "Dynamic ambient cover glow",
                "Projects subtle, diffused ambient backlights matching the active song artwork.",
                app.saved.ambient_glow,
            ) {
                out.push(Action::SetAmbientGlow(!app.saved.ambient_glow));
            }

            if toggle_row(
                ui,
                "app-selection-glow",
                icon::CURSOR,
                "Selection outline & glow",
                "Adds a luminous accent border around active navigation tabs and selections.",
                app.saved.selection_glow,
            ) {
                out.push(Action::SetSelectionGlow(!app.saved.selection_glow));
            }

            if let Some(cr) = slider_row(
                ui,
                "corner-roundness",
                icon::SQUARES_FOUR,
                "Corner radius",
                "Controls roundness of cards, buttons, and capsules throughout AURI.",
                app.saved.corner_roundness,
                0.0..=24.0,
                |v| format!("{:.0} px", v),
            ) {
                out.push(Action::SetCornerRoundness(cr));
            }
        });

        // Card 3: PLAYER VIEW & NAVIGATION
        settings_card_panel(ui, "app-card-layout", "Player view & navigation", "Layout of the player deck and side navigation panels.", |ui| {
            if toggle_row(
                ui,
                "app-sidebar",
                icon::LIST,
                "Navigation sidebar",
                "Shows the left navigation panel with your playlists, home, and library.",
                app.saved.sidebar_open,
            ) {
                out.push(Action::ToggleSidebar);
            }

            if toggle_row(
                ui,
                "app-queue",
                icon::QUEUE,
                "Song queue sidebar",
                "Shows the right side panel with upcoming tracks and current track artwork.",
                app.saved.queue_open,
            ) {
                out.push(Action::ToggleQueue);
            }
        });
    });
}

fn appearance_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    // The live preview renders in the right rail (the queue dock slot), so the
    // controls always keep the full 740px settings column.
    render_appearance_controls(ui, app, out, ui.available_width().min(740.0));
}

fn player_bar_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    let is_perf = app.saved.performance_mode;

    settings_card_panel(ui, "playback-quality", "Audio quality & streaming", "Bitrate used when streaming from YouTube Music.", |ui| {
        if let Some(quality) = segmented_row(
            ui,
            "stream-quality-seg",
            icon::SPEAKER_HIGH,
            "Stream quality",
            "High quality streams 160 kbps Opus; Data saver drops to 70 kbps.",
            app.saved.stream_quality,
            &[
                (StreamQuality::High, "High Quality (160 kbps Opus)"),
                (StreamQuality::Low, "Data Saver (70 kbps Opus)"),
            ],
        ) {
            out.push(Action::SetStreamQuality(quality));
        }
    });

    settings_card_panel(ui, "playback-transitions", "Track transitions", "How AURI moves from one track to the next.", |ui| {
        let xfade_desc = if is_perf {
            "Bypassed while Performance Mode is active to save CPU cycles."
        } else {
            "Blend the end of each track smoothly into the next."
        };
        if toggle_row(
            ui,
            "crossfade-toggle",
            icon::WAVEFORM,
            "Seamless track crossfade",
            xfade_desc,
            app.saved.crossfade_enabled,
        ) {
            out.push(Action::SetCrossfadeEnabled(!app.saved.crossfade_enabled));
        }
        if app.saved.crossfade_enabled {
            if let Some(secs) = slider_row(
                ui,
                "crossfade-duration",
                icon::TIMER,
                "Crossfade duration",
                "",
                app.saved.crossfade_secs as f32,
                1.0..=10.0,
                |v| format!("{} s", v.round() as u8),
            ) {
                if !is_perf {
                    out.push(Action::SetCrossfadeSecs(secs.round() as u8));
                }
            }
        }
    });

    settings_card_panel(ui, "playback-deck", "Deck appearance", "Layout and glassmorphism of the bottom player deck.", |ui| {
        if let Some(layout) = segmented_row(
            ui,
            "bar-layout-seg",
            icon::SLIDERS,
            "Deck layout",
            "Docked spans the window; capsules float above page content.",
            app.saved.bar_layout,
            &[
                (BarLayout::Docked, "Docked"),
                (BarLayout::FloatingPill, "Floating Island"),
                (BarLayout::MiniCapsule, "Mini Capsule"),
            ],
        ) {
            out.push(Action::SetBarLayout(layout));
        }
        if toggle_row(
            ui,
            "bar-glass",
            icon::DROP,
            "Glassmorphism & translucency",
            "Let page content glow through the deck.",
            app.saved.bar_glass,
        ) {
            out.push(Action::SetBarGlass(!app.saved.bar_glass));
        }
        if app.saved.bar_glass {
            if let Some(op) = slider_row(
                ui,
                "bar-opacity",
                icon::EYEDROPPER,
                "Deck opacity",
                "",
                app.saved.bar_opacity,
                0.20..=1.0,
                |v| format!("{:.0}%", v * 100.0),
            ) {
                out.push(Action::SetBarOpacity(op));
            }
        }
        if toggle_row(
            ui,
            "bar-grain",
            icon::SPARKLE,
            "Frosted acrylic grain filter",
            "Adds a subtle procedural noise texture to the deck.",
            app.saved.bar_grain,
        ) {
            out.push(Action::SetBarGrain(!app.saved.bar_grain));
        }
        if app.saved.bar_grain {
            if let Some(strength) = slider_row(
                ui,
                "bar-grain-strength",
                icon::WAVEFORM,
                "Grain intensity",
                "",
                app.saved.bar_grain_strength,
                0.05..=0.35,
                |v| format!("{:.0}%", v * 100.0),
            ) {
                out.push(Action::SetBarGrainStrength(strength));
            }
        }
    });
}

fn sync_together_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    let accent = ui.ctx().data(|d| d.get_temp(Id::new("app-theme-accent"))).unwrap_or(ACCENT);
    settings_card_panel(ui, "sync-card", "Listen Together", "Synchronized playback with friends through instant room codes.", |ui| {
        // 1. Session status with Host / Leave button
        let connected = app.sync_mode != crate::sync::SyncMode::Disconnected;
        let status_desc = match app.sync_mode {
            crate::sync::SyncMode::Disconnected => "Disconnected. Host a room or join a friend's code.".to_owned(),
            crate::sync::SyncMode::Host => format!("Hosting room {}. Broadcasting playback.", app.sync_room),
            crate::sync::SyncMode::Guest => format!("Syncing in room {}. Listening along.", app.sync_room),
        };
        let row = row_base(ui, icon::HEADPHONES, "Session status", &status_desc);
        let host_btn = Rect::from_min_size(pos2(row.right() - 110.0, row.center().y - 14.0), vec2(104.0, 28.0));
        let host_resp = ui.interact(host_btn, Id::new("sync-host-btn"), Sense::click());
        let host_hov = hover_of(ui, &host_resp);
        let (host_label, host_tone) = if connected {
            ("Leave room", Color32::from_rgb(244, 63, 94))
        } else {
            ("Host room", accent)
        };
        ui.painter().rect_filled(host_btn, 14.0, host_tone.gamma_multiply(0.18 + 0.12 * host_hov));
        ui.painter().rect_stroke(host_btn, 14.0, Stroke::new(1.0_f32, host_tone), StrokeKind::Inside);
        ui.painter().text(
            host_btn.center(),
            Align2::CENTER_CENTER,
            host_label,
            bold(12.0),
            if host_hov > 0.1 { TEXT } else { host_tone },
        );
        describe(&host_resp, host_label);
        if host_resp.clicked() {
            if connected {
                out.push(Action::StopSync);
            } else {
                let code = format!(
                    "AURI-{:04}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_or(0, |elapsed| elapsed.as_millis() % 10000)
                );
                out.push(Action::StartSyncHost(code));
            }
        }

        // 2. Join row: room code input + Join button
        let join_id = Id::new("sync-join-code");
        let join_code: String = ui
            .data(|d| d.get_temp(join_id))
            .unwrap_or_else(|| app.saved.sync_room_code.clone());
        let row = row_base(ui, icon::LINK, "Join a friend's room", "");
        let field = Rect::from_min_size(pos2(row.right() - 330.0, row.center().y - 15.0), vec2(216.0, 30.0));
        ui.painter().rect_filled(field, 7.0, veil(0.07));
        ui.painter().rect_stroke(field, 7.0, Stroke::new(1.0_f32, veil(0.12)), StrokeKind::Inside);
        let mut buffer = join_code.clone();
        let edit = egui::TextEdit::singleline(&mut buffer)
            .hint_text("Room code (e.g. AURI-1234)")
            .desired_width(204.0)
            .font(sans(12.5))
            .frame(false);
        let edit_resp = ui.put(field.shrink2(vec2(5.0, 2.0)), edit);
        if edit_resp.changed() {
            ui.data_mut(|d| d.insert_temp(join_id, buffer.clone()));
        }
        let join_btn = Rect::from_min_size(pos2(row.right() - 110.0, row.center().y - 14.0), vec2(104.0, 28.0));
        let join_resp = ui.interact(join_btn, Id::new("sync-join-btn"), Sense::click());
        let join_hov = hover_of(ui, &join_resp);
        ui.painter().rect_filled(join_btn, 14.0, veil(0.10 + 0.10 * join_hov));
        ui.painter().rect_stroke(join_btn, 14.0, Stroke::new(1.0_f32, veil(0.18)), StrokeKind::Inside);
        ui.painter().text(
            join_btn.center(),
            Align2::CENTER_CENTER,
            "Join",
            bold(12.0),
            if join_hov > 0.1 { TEXT } else { MUTED },
        );
        describe(&join_resp, "Join");
        if join_resp.clicked() && !buffer.trim().is_empty() {
            out.push(Action::StartSyncJoin(buffer.trim().to_uppercase()));
        }
    });

    settings_card_panel(
        ui,
        "sync-github",
        "Persistent GitHub Gist sync",
        "Keep room sessions and playback state in sync through a private GitHub Gist.",
        |ui| {
            if let Some(token) = input_row(
                ui,
                "sync-token",
                icon::GITHUB_LOGO,
                "GitHub token (PAT)",
                "Personal access token with the 'gist' scope.",
                "ghp_...",
                true,
                340.0,
                &app.saved.sync_github_token,
            ) {
                out.push(Action::SetSyncGithubToken(token));
            }
            if let Some(gist) = input_row(
                ui,
                "sync-gist",
                icon::DATABASE,
                "Gist ID",
                "Leave empty to create one automatically.",
                "Gist ID",
                false,
                340.0,
                &app.saved.sync_gist_id,
            ) {
                out.push(Action::SetSyncGistId(gist));
            }
        },
    );
}

fn lyrics_settings_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    settings_card_panel(ui, "lyrics-card", "Synced lyrics", "Karaoke-style timed lyrics while a song plays.", |ui| {
        if toggle_row(
            ui,
            "lyrics-synced",
            icon::MICROPHONE_STAGE,
            "Karaoke auto-sync scrolling",
            "Auto-scrolls to the active singing line.",
            app.saved.lyrics_synced_mode,
        ) {
            out.push(Action::SetLyricsSyncedMode(!app.saved.lyrics_synced_mode));
        }
        if let Some(align_center) = segmented_row(
            ui,
            "lyrics-align",
            icon::TEXT_ALIGN_CENTER,
            "Lyrics text alignment",
            "",
            app.saved.lyrics_align_center,
            &[(true, "Center"), (false, "Left")],
        ) {
            out.push(Action::SetLyricsAlignCenter(align_center));
        }
        if let Some(size) = segmented_row(
            ui,
            "lyrics-font-size",
            icon::TEXT_T,
            "Lyrics font size",
            "",
            app.saved.lyrics_font_size,
            &[(18.0, "Small 18 px"), (24.0, "Standard 24 px"), (30.0, "Large 30 px")],
        ) {
            out.push(Action::SetLyricsFontSize(size));
        }
        if let Some(dim) = slider_row(
            ui,
            "lyrics-dim-unsung",
            icon::EYE_SLASH,
            "Unsung line dimming",
            "Opacity of lines that are not being sung.",
            app.saved.lyrics_dim_unsung,
            0.10..=0.80,
            |v| format!("{:.0}%", v * 100.0),
        ) {
            out.push(Action::SetLyricsDimUnsung(dim));
        }
    });
}

fn storage_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    settings_card_panel(ui, "storage-dir", "Storage & downloads directory", "Where downloaded songs are saved on disk.", |ui| {
        let display_path = format!("{}\\downloads", app.saved.storage_dir);
        let row = row_base(ui, icon::FOLDER_SIMPLE, "Main storage directory", &display_path);
        let btn_w = 78.0;
        let buttons = [
            ("storage-browse-btn", "Browse", Action::BrowseStorageDir, 3),
            ("storage-open-btn", "Open", Action::OpenStorageDir, 2),
            ("storage-reset-btn", "Default", Action::ResetStorageDir, 1),
        ];
        for (id, label_str, action, slot) in buttons {
            let rect = Rect::from_min_size(
                pos2(row.right() - slot as f32 * btn_w - (slot - 1) as f32 * 6.0, row.center().y - 14.0),
                vec2(btn_w, 28.0),
            );
            let resp = ui.interact(rect, Id::new(id), Sense::click());
            let hover = hover_of(ui, &resp);
            ui.painter().rect_filled(rect, 6.0, veil(0.06 + 0.05 * hover));
            ui.painter().rect_stroke(rect, 6.0, Stroke::new(1.0_f32, veil(0.12)), StrokeKind::Inside);
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                label_str,
                medium(11.5),
                if hover > 0.1 { TEXT } else { MUTED },
            );
            describe(&resp, label_str);
            if resp.clicked() {
                out.push(action);
            }
        }
    });

    settings_card_panel(
        ui,
        "storage-cache",
        "Offline audio cache",
        "Songs you stream are cached on disk for instant, zero-latency replay at 0% network usage.",
        |ui| {
            let cached_bytes = crate::cache::cache_size_bytes();
            let cached_count = crate::cache::count_cached_tracks();
            let cache_desc = format!("{:.1} MB cached. {} songs ready offline.", cached_bytes as f64 / 1_048_576.0, cached_count);
            let row = row_base(ui, icon::CLOUD_ARROW_DOWN, "Cache-as-you-play", &cache_desc);
            let btn_w = 78.0;
            let buttons = [
                ("cache-purge-btn", "Purge", Action::PurgeAudioCache, 2),
                ("cache-open-btn", "Explore", Action::OpenCacheDir, 1),
            ];
            for (id, label_str, action, slot) in buttons {
                let rect = Rect::from_min_size(
                    pos2(row.right() - slot as f32 * btn_w - (slot - 1) as f32 * 6.0, row.center().y - 14.0),
                    vec2(btn_w, 28.0),
                );
                let resp = ui.interact(rect, Id::new(id), Sense::click());
                let hover = hover_of(ui, &resp);
                ui.painter().rect_filled(rect, 6.0, veil(0.06 + 0.05 * hover));
                ui.painter().rect_stroke(rect, 6.0, Stroke::new(1.0_f32, veil(0.12)), StrokeKind::Inside);
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    label_str,
                    medium(11.5),
                    if hover > 0.1 { TEXT } else { MUTED },
                );
                describe(&resp, label_str);
                if resp.clicked() {
                    out.push(action);
                }
            }
            ui.add_space(4.0);
            ui.label(
                RichText::new("Right-click any song to explicitly download it as a high-bitrate audio file.")
                    .font(sans(11.0))
                    .color(FAINT),
            );
        },
    );
}

fn about_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    settings_card_panel(ui, "about-version", "Application & release", "Version and update status of this build.", |ui| {
        let row = row_base(ui, icon::INFO, "Version", &format!("AURI v{}", crate::VERSION));
        let right_center = pos2(row.right() - 14.0, row.center().y);
        if app.update.is_some() {
            if text_button(ui, right_center, Id::new("update-btn"), "Update Available").clicked() {
                out.push(Action::OpenUpdateDialog);
            }
        } else {
            label(ui, right_center, Align2::RIGHT_CENTER, "Up to date", sans(12.5), FAINT, 120.0);
        }
    });

    settings_card_panel(ui, "about-audio", "High-fidelity audio engine", "Pure-Rust decoding and DSP pipeline.", |ui| {
        row_base(
            ui,
            icon::SPEAKER_HIGH,
            "Opus & Symphonia multi-stream decoder",
            "Native 160 kbps Opus and 70 kbps Data-Saver streaming through the Rodio sink.",
        );
        row_base(
            ui,
            icon::SLIDERS,
            "10-band biquad IIR DSP equalizer",
            "Peak & shelving filters with software pre-amp and a real-time loudness normalizer.",
        );
    });

    settings_card_panel(ui, "about-overlay", "Desktop overlay & gaming architecture", "Focus-free floating HUD technology.", |ui| {
        row_base(
            ui,
            icon::PICTURE_IN_PICTURE,
            "Pure Win32 desktop overlay HUD",
            "ITaskbarList3 deletion, WS_EX_TOOLWINDOW and WS_EX_NOACTIVATE for a focus-free HUD.",
        );
        row_base(
            ui,
            icon::LIGHTNING,
            "Esports & gaming mode engine",
            "Automatic crossfade bypass, frame throttling and aggressive RAM purges for 0% FPS impact.",
        );
    });

    ui.label(
        RichText::new("AURI is open source and updates directly from github.com/SuryakiranMK/AURI.")
            .font(sans(12.0))
            .color(FAINT),
    );
    ui.add_space(4.0);
}

fn general_section(ui: &mut Ui, app: &App, out: &mut Vec<Action>) {
    settings_card_panel(ui, "general-card", "General preferences", "Everyday window and behaviour options.", |ui| {
        if toggle_row(
            ui,
            "gen-close-to-tray",
            icon::MINUS_CIRCLE,
            "Minimize to system tray on window close",
            "Closing the window keeps AURI running in the tray.",
            app.saved.close_to_tray,
        ) {
            out.push(Action::SetCloseToTray(!app.saved.close_to_tray));
        }
        if toggle_row(
            ui,
            "gen-notifications",
            icon::BELL,
            "Desktop notifications on song change",
            "Show a system toast whenever a new track starts.",
            app.saved.song_notifications,
        ) {
            out.push(Action::SetSongNotifications(!app.saved.song_notifications));
        }
        if toggle_row(
            ui,
            "gen-startup",
            icon::POWER,
            "Launch AURI when Windows starts",
            "Starts AURI automatically when you sign in.",
            app.saved.launch_on_startup,
        ) {
            out.push(Action::SetLaunchOnStartup(!app.saved.launch_on_startup));
        }
        if toggle_row(
            ui,
            "gen-history",
            icon::CLOCK_COUNTER_CLOCKWISE,
            "Track playback history and listening stats",
            "Powers Recently played and the Stats page.",
            app.saved.history_enabled,
        ) {
            out.push(Action::SetHistoryEnabled(!app.saved.history_enabled));
        }
    });

    settings_card_panel(
        ui,
        "general-blocked",
        "Blocked artists",
        "Songs by blocked artists are auto-skipped and filtered from radio.",
        |ui| {
            if app.saved.blocked_artists.is_empty() {
                ui.label(RichText::new("No blocked artists. Songs by all artists are permitted.").font(sans(12.0)).color(MUTED));
            } else {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                    for artist in &app.saved.blocked_artists {
                        let chip_w = (artist.len() as f32 * 7.5 + 40.0).clamp(70.0, 220.0);
                        let (chip_rect, chip_resp) = ui.allocate_exact_size(vec2(chip_w, 28.0), Sense::click());
                        let chip_hov = hover_of(ui, &chip_resp);
                        ui.painter().rect_filled(chip_rect, 6.0, veil(0.08 + 0.05 * chip_hov));
                        ui.painter().rect_stroke(chip_rect, 6.0, Stroke::new(1.0_f32, veil(0.14)), StrokeKind::Inside);
                        label(
                            ui,
                            pos2(chip_rect.left() + 10.0, chip_rect.center().y),
                            Align2::LEFT_CENTER,
                            artist,
                            medium(11.5),
                            TEXT,
                            chip_w - 28.0,
                        );
                        ui.painter().text(
                            pos2(chip_rect.right() - 12.0, chip_rect.center().y),
                            Align2::CENTER_CENTER,
                            icon::X,
                            glyph(12.0),
                            Color32::from_rgb(239, 68, 68),
                        );
                        describe(&chip_resp, &format!("Unblock artist: {}", artist));
                        if chip_resp.clicked() {
                            out.push(Action::UnblockArtist(artist.clone()));
                        }
                    }
                });
            }
        },
    );

    settings_card_panel(ui, "general-stats", "Listening statistics", "Your listening history lives on this device only.", |ui| {
        if button_row(
            ui,
            "gen-reset-stats",
            icon::TRASH,
            "Reset listening statistics",
            "Clears all listening history, streaks, and top tracks.",
            "Reset data",
            true,
        ) {
            out.push(Action::ClearStats);
        }
    });
}

fn settings_header_and_tabs(ui: &mut Ui, app: &App, active_tab: SettingsTab, out: &mut Vec<Action>) {
    let full = ui.available_width();
    let width = (full - 48.0).clamp(680.0, 960.0);
    let pad = (full - width).max(0.0) / 2.0;

    ui.add_space(12.0);

    // Title Row
    let (title, _) = ui.allocate_exact_size(vec2(full, 50.0), Sense::hover());
    let at = pos2(title.left() + pad, title.bottom() - 6.0);
    label(ui, at, Align2::LEFT_BOTTOM, "Settings", display(38.0), TEXT, width);
    ui.add_space(10.0);

    // All 11 Tab Switchers in 2 clean rows
    let tab_items: [(SettingsTab, &str, &str); 11] = [
        (SettingsTab::General, "General", icon::GEAR_SIX),
        (SettingsTab::Appearance, "Appearance", icon::PALETTE),
        (SettingsTab::Playback, "Playback", icon::PLAY),
        (SettingsTab::Equalizer, "Equalizer", icon::SLIDERS),
        (SettingsTab::Performance, "Performance", icon::LIGHTNING),
        (SettingsTab::GamingMode, "Gaming Mode", icon::GAME_CONTROLLER),
        (SettingsTab::MiniMode, "Mini Mode", icon::PICTURE_IN_PICTURE),
        (SettingsTab::Discord, "Discord", icon::DISCORD_LOGO),
        (SettingsTab::GlobalHotkeys, "Hotkeys", icon::KEYBOARD),
        (SettingsTab::DataStorage, "Storage", icon::DATABASE),
        (SettingsTab::About, "About", icon::INFO),
    ];
    let (tabs_bar, _) = ui.allocate_exact_size(vec2(full, 80.0), Sense::hover());
    let tabs_left = tabs_bar.left() + pad;
    let pal_color = app.saved.theme_palette.color();

    let num_row0 = 6;
    for (i, &(tab, name, sym)) in tab_items.iter().enumerate() {
        let is_cur = active_tab == tab;
        let (row, col, count) = if i < num_row0 {
            (0, i, num_row0)
        } else {
            (1, i - num_row0, tab_items.len() - num_row0)
        };
        let row_w = (width - (count - 1) as f32 * 8.0) / count as f32;
        let tab_rect = Rect::from_min_size(
            pos2(tabs_left + col as f32 * (row_w + 8.0), tabs_bar.top() + row as f32 * 38.0),
            vec2(row_w, 34.0),
        );
        let resp = ui.interact(tab_rect, Id::new(("settings-sticky-tab", i)), Sense::click());
        let hover = hover_of(ui, &resp);
        let fill = if is_cur { veil(0.14) } else { veil(0.04 * hover) };
        ui.painter().rect_filled(tab_rect, 8.0, fill);
        if is_cur {
            if app.saved.selection_glow {
                ui.painter().rect_stroke(tab_rect.expand(1.0), 9.0, Stroke::new(1.0_f32, pal_color.gamma_multiply(0.40)), StrokeKind::Outside);
            }
            ui.painter().rect_stroke(
                tab_rect,
                8.0,
                Stroke::new(1.5_f32, pal_color.lerp_to_gamma(Color32::WHITE, 0.40)),
                StrokeKind::Inside,
            );
        }
        let tone = if is_cur { TEXT } else { MUTED.lerp_to_gamma(TEXT, hover) };
        let icon_color = if is_cur { pal_color } else { tone };
        let center = tab_rect.center();
        let galley = ui.painter().layout_no_wrap(name.to_string(), medium(13.0), tone);
        let icon_galley = ui.painter().layout_no_wrap(sym.to_string(), glyph(16.0), icon_color);
        let text_w = galley.size().x;
        let icon_w = icon_galley.size().x;
        let gap = 6.0;
        let total_w = icon_w + gap + text_w;
        let start_x = center.x - total_w / 2.0;
        ui.painter().galley(pos2(start_x, center.y - icon_galley.size().y / 2.0), icon_galley, icon_color);
        ui.painter().galley(pos2(start_x + icon_w + gap, center.y - galley.size().y / 2.0), galley, tone);
        if resp.clicked() {
            out.push(Action::Go(Route::Settings(tab)));
        }
    }
    ui.add_space(8.0);
    // Sticky bottom border separator
    let (sep, _) = ui.allocate_exact_size(vec2(full, 2.0), Sense::hover());
    ui.painter().hline(tabs_left..=(tabs_left + width), sep.center().y, Stroke::new(1.0_f32, veil(0.08)));
    ui.add_space(10.0);
}

fn settings_body(ui: &mut Ui, app: &App, active_tab: SettingsTab, out: &mut Vec<Action>) {
    match active_tab {
        SettingsTab::General => {
            settings_column(ui, |ui| {
                general_section(ui, app, out);
                sync_together_section(ui, app, out);
            });
        }
        SettingsTab::Appearance => {
            settings_column(ui, |ui| appearance_section(ui, app, out));
        }
        SettingsTab::Playback => {
            settings_column(ui, |ui| {
                player_bar_section(ui, app, out);
                lyrics_settings_section(ui, app, out);
            });
        }
        SettingsTab::Equalizer => {
            settings_column(ui, |ui| audio_dsp_section(ui, app, out));
        }
        SettingsTab::Performance => {
            settings_column(ui, |ui| performance_settings_section(ui, app, out));
        }
        SettingsTab::GamingMode => {
            settings_column(ui, |ui| gaming_mode_section(ui, app, out));
        }
        SettingsTab::MiniMode => {
            settings_column(ui, |ui| mini_mode_settings_section(ui, app, out));
        }
        SettingsTab::Discord => {
            settings_column(ui, |ui| discord_section(ui, app, out));
        }
        SettingsTab::GlobalHotkeys => {
            settings_column(ui, |ui| hotkey_section(ui, app, out));
        }
        SettingsTab::DataStorage => {
            settings_column(ui, |ui| storage_section(ui, app, out));
        }
        SettingsTab::About => {
            settings_column(ui, |ui| about_section(ui, app, out));
        }
    }
    ui.add_space(32.0);
}

fn nav_item(ui: &mut Ui, symbol: &str, text: &str, active: bool, trailing: &str, glow: bool, accent: Color32) -> Response {
    let (slot, response) = ui.allocate_exact_size(vec2(ui.available_width(), 42.0), Sense::click());
    let rect = slot.shrink2(vec2(12.0, 1.0));
    let hover = hover_of(ui, &response);
    ui.painter()
        .rect_filled(rect, ROW_RADIUS, veil(if active { 0.08 } else { 0.05 * hover }));
    if active && glow {
        ui.painter().rect_stroke(rect.expand(1.0), ROW_RADIUS + 1.0, Stroke::new(1.0_f32, accent.gamma_multiply(0.35)), StrokeKind::Outside);
        ui.painter().rect_stroke(rect, ROW_RADIUS, Stroke::new(1.5_f32, accent.lerp_to_gamma(Color32::WHITE, 0.40)), StrokeKind::Inside);
    }
    let tone = if active { TEXT } else { MUTED.lerp_to_gamma(TEXT, hover) };
    let icon_color = if active {
        if glow { accent.lerp_to_gamma(Color32::WHITE, 0.35) } else { accent }
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
    let accent = ui.ctx().data(|d| d.get_temp(Id::new("app-theme-accent"))).unwrap_or(ACCENT);
    ui.painter().circle_filled(center, radius, accent);
    ui.painter().circle_stroke(center, radius * 0.55, Stroke::new(radius * 0.09, PANEL));
    ui.painter().circle_filled(center, radius * 0.18, PANEL);
}

#[allow(dead_code)]
fn get_live_ram_mb() -> f32 {
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
        unsafe extern "system" {
            fn GetProcessMemoryInfo(process: isize, ppsmemCounters: *mut PROCESS_MEMORY_COUNTERS, cb: u32) -> i32;
            fn GetCurrentProcess() -> isize;
        }
        let mut pmc = std::mem::MaybeUninit::<PROCESS_MEMORY_COUNTERS>::zeroed();
        unsafe {
            let p = pmc.as_mut_ptr();
            (*p).cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
            if GetProcessMemoryInfo(GetCurrentProcess(), p, (*p).cb) != 0 {
                return (*p).working_set_size as f32 / (1024.0 * 1024.0);
            }
        }
    }
    0.0
}

fn stats_view(ui: &mut Ui, cx: &mut Cx, saved: &crate::app::Saved) {
    let full = ui.available_width();
    let pad = 24.0;
    let width = (full - 2.0 * pad).max(640.0);
    let is_light = saved.theme_mode == ThemeMode::Light;
    let t_color = if is_light { Color32::from_rgb(24, 28, 36) } else { TEXT };
    let m_color = if is_light { Color32::from_rgb(100, 110, 125) } else { MUTED };
    let pal_color = saved.theme_palette.color();

    ui.add_space(8.0);

    // 1. Header (Image 2): "Listening stats" on left, Filter Pills on right
    let (head_rect, _) = ui.allocate_exact_size(vec2(full, 48.0), Sense::hover());
    let title_pos = pos2(head_rect.left() + pad, head_rect.center().y);
    label(ui, title_pos, Align2::LEFT_CENTER, "Listening stats", bold(28.0), t_color, 250.0);

    // Time Filter Pills: [ 7 days ] [ 30 days ] [ 12 months ] [ All time ]
    let filter_pills = [
        (crate::app::StatsPeriod::SevenDays, "7 days"),
        (crate::app::StatsPeriod::ThirtyDays, "30 days"),
        (crate::app::StatsPeriod::TwelveMonths, "12 months"),
        (crate::app::StatsPeriod::AllTime, "All time"),
    ];
    let pill_right = head_rect.left() + pad + width;
    let total_pills_w = 4.0 * 76.0 + 3.0 * 6.0;
    let mut pill_x = pill_right - total_pills_w;
    for (period, name) in filter_pills {
        let is_cur = saved.stats_filter == period;
        let pill_rect = Rect::from_min_size(pos2(pill_x, head_rect.center().y - 14.0), vec2(76.0, 28.0));
        let resp = ui.interact(pill_rect, Id::new(("stat-period-pill", name)), Sense::click());
        let hov = hover_of(ui, &resp);

        if is_cur {
            let active_text_col = contrast_text_for(pal_color);
            ui.painter().rect_filled(pill_rect, 14.0, pal_color);
            if active_text_col != Color32::WHITE {
                ui.painter().rect_stroke(pill_rect, 14.0, Stroke::new(1.0_f32, Color32::from_rgb(160, 160, 170)), StrokeKind::Inside);
            }
            ui.painter().text(pill_rect.center(), Align2::CENTER_CENTER, name, bold(12.0), active_text_col);
        } else {
            ui.painter().rect_filled(pill_rect, 14.0, veil(0.05 + 0.04 * hov));
            ui.painter().rect_stroke(pill_rect, 14.0, Stroke::new(1.0_f32, veil(0.08)), StrokeKind::Inside);
            ui.painter().text(pill_rect.center(), Align2::CENTER_CENTER, name, medium(12.0), if hov > 0.1 { t_color } else { m_color });
        }
        if resp.clicked() {
            cx.out.push(Action::SetStatsPeriod(period));
        }
        pill_x += 76.0 + 6.0;
    }
    ui.add_space(16.0);

    // 2. Giant Stat Display (Image 2): "7 m" or "12 h 45 m" or "0 m"
    let total_ms = saved.stats_total_listen_ms;
    let total_secs = total_ms / 1000;
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let (stat_row, _) = ui.allocate_exact_size(vec2(full, 68.0), Sense::hover());
    let row_left = stat_row.left() + pad;

    let (num_str, unit_str) = if hours > 0 {
        (format!("{hours} h {mins}"), "m")
    } else {
        (format!("{mins}"), "m")
    };
    let num_galley = ui.painter().layout_no_wrap(num_str.clone(), bold(56.0), t_color);
    let num_w = num_galley.rect.width();
    ui.painter().galley(pos2(row_left, stat_row.top() + 4.0), num_galley, t_color);
    ui.painter().text(pos2(row_left + num_w + 6.0, stat_row.bottom() - 14.0), Align2::LEFT_BOTTOM, unit_str, sans(24.0), m_color);

    let sub_text = format!("listened {}", saved.stats_filter.subtitle_period());
    let (sub_row, _) = ui.allocate_exact_size(vec2(full, 22.0), Sense::hover());
    label(ui, pos2(sub_row.left() + pad, sub_row.center().y), Align2::LEFT_CENTER, &sub_text, sans(13.5), m_color, width);
    ui.add_space(20.0);

    // 3. Four Key Metric Cards in a row (Image 2): Plays, Songs, Artists, Day streak
    let card_w = (width - 18.0) / 4.0;
    let (cards_row, _) = ui.allocate_exact_size(vec2(full, 82.0), Sense::hover());
    let unique_songs = saved.stats_top_tracks.len();
    let unique_artists = saved.stats_top_artists.len();

    let day_streak = {
        let secs = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d.as_secs(),
            Err(_) => 0,
        };
        let cur_day = (secs / 86400) as u32;
        let mut streak = 0;
        let mut check_day = cur_day;
        while saved.stats_daily.iter().any(|d| d.day_key == check_day && (d.plays > 0 || d.listen_ms > 0)) {
            streak += 1;
            if check_day == 0 { break; }
            check_day -= 1;
        }
        if streak == 0 && saved.stats_total_plays > 0 {
            1
        } else {
            streak
        }
    };

    let metric_cards = [
        (icon::PLAY_CIRCLE, "Plays", saved.stats_total_plays.to_string(), Some("total".to_string())),
        (icon::MUSIC_NOTE, "Songs", unique_songs.to_string(), Some("tracks".to_string())),
        (icon::USER, "Artists", unique_artists.to_string(), Some("artists".to_string())),
        (icon::FIRE, "Day streak", day_streak.to_string(), Some("days".to_string())),
    ];

    for (i, &(sym, m_label, ref val_str, ref sub_opt)) in metric_cards.iter().enumerate() {
        let c_rect = Rect::from_min_size(
            pos2(cards_row.left() + pad + i as f32 * (card_w + 6.0), cards_row.top()),
            vec2(card_w, 80.0),
        );
        ui.painter().rect_filled(c_rect, 12.0, veil(0.04));
        ui.painter().rect_stroke(c_rect, 12.0, Stroke::new(1.0_f32, veil(0.08)), StrokeKind::Inside);

        ui.painter().text(pos2(c_rect.left() + 16.0, c_rect.top() + 18.0), Align2::LEFT_CENTER, sym, solid(13.0), pal_color);
        label(ui, pos2(c_rect.left() + 34.0, c_rect.top() + 18.0), Align2::LEFT_CENTER, m_label, sans(12.0), m_color, card_w - 44.0);

        let val_galley = ui.painter().layout_no_wrap(val_str.clone(), bold(22.0), t_color);
        let val_w = val_galley.rect.width();
        let val_pos = pos2(c_rect.left() + 16.0, c_rect.top() + 48.0 - val_galley.size().y / 2.0);
        ui.painter().galley(val_pos, val_galley, t_color);
        if let Some(sub) = sub_opt {
            let sub_x = c_rect.left() + 16.0 + val_w + 10.0;
            if sub_x + 28.0 < c_rect.right() - 8.0 {
                let avail = c_rect.right() - sub_x - 8.0;
                label(ui, pos2(sub_x, c_rect.top() + 48.0), Align2::LEFT_CENTER, sub, sans(11.0), m_color, avail);
            }
        }
    }
    ui.add_space(26.0);

    // 4. Listening Activity Bar Chart (Image 2)
    let (act_head, _) = ui.allocate_exact_size(vec2(full, 30.0), Sense::hover());
    let act_left = act_head.left() + pad;
    ui.painter().text(pos2(act_left + 10.0, act_head.center().y), Align2::CENTER_CENTER, icon::CHART_BAR, glyph(16.0), pal_color);
    label(ui, pos2(act_left + 26.0, act_head.center().y), Align2::LEFT_CENTER, "Listening activity", bold(18.0), t_color, 250.0);

    let (stat_sub, _) = ui.allocate_exact_size(vec2(full, 24.0), Sense::hover());
    let avg_mins = (total_secs / 60) / 30.max(1);
    let busiest = saved.stats_daily.iter().max_by_key(|d| d.listen_ms);
    let busiest_str = if let Some(b) = busiest {
        let b_mins = (b.listen_ms / 1000 / 60).max(1);
        format!("Today · {b_mins} min")
    } else if total_ms > 0 {
        format!("Today · {} min", (total_secs / 60).max(1))
    } else {
        "None yet".to_string()
    };
    label(ui, pos2(act_left + 4.0, stat_sub.center().y), Align2::LEFT_CENTER, &format!("Daily average: {avg_mins} min    Busiest day: {busiest_str}"), sans(12.0), m_color, width);
    ui.add_space(10.0);

    let chart_h = 170.0;
    let (chart_box, _) = ui.allocate_exact_size(vec2(full, chart_h), Sense::hover());
    let c_rect = Rect::from_min_size(pos2(chart_box.left() + pad, chart_box.top()), vec2(width, chart_h));

    let num_bars = match saved.stats_filter {
        crate::app::StatsPeriod::SevenDays => 7,
        crate::app::StatsPeriod::ThirtyDays | crate::app::StatsPeriod::AllTime => 30,
        crate::app::StatsPeriod::TwelveMonths => 12,
    };
    let bar_spacing = 3.0;
    let bar_w = ((width - (num_bars as f32 - 1.0) * bar_spacing) / num_bars as f32).max(6.0);
    let baseline_y = c_rect.bottom() - 24.0;
    let max_chart_h = chart_h - 45.0;

    ui.painter().hline(c_rect.x_range(), baseline_y, Stroke::new(1.0_f32, veil(0.08)));

    let cur_secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let cur_day_key = (cur_secs / 86400) as u32;
    let max_day_ms = saved.stats_daily.iter().map(|d| d.listen_ms).max().unwrap_or(1).max(60000);

    for idx in 0..num_bars {
        let day_offset = (num_bars - 1 - idx) as u32;
        let target_day = cur_day_key.saturating_sub(day_offset);
        let day_act = saved.stats_daily.iter().find(|d| d.day_key == target_day);
        let listen_ms = day_act.map_or(0, |d| d.listen_ms);
        let plays = day_act.map_or(0, |d| d.plays);

        let bar_x = c_rect.left() + idx as f32 * (bar_w + bar_spacing);
        let frac = if max_day_ms > 0 { (listen_ms as f32 / max_day_ms as f32).clamp(0.0, 1.0) } else { 0.0 };
        let bar_h = if listen_ms > 0 { (frac * max_chart_h).max(6.0) } else { 2.0 };
        let b_rect = Rect::from_min_max(pos2(bar_x, baseline_y - bar_h), pos2(bar_x + bar_w, baseline_y));

        let resp = ui.interact(b_rect.expand(2.0), Id::new(("stat-bar", idx)), Sense::hover());
        let hov = hover_of(ui, &resp);

        if listen_ms > 0 {
            let col = pal_color.lerp_to_gamma(Color32::WHITE, 0.25 * hov);
            ui.painter().rect_filled(b_rect, eframe::egui::CornerRadius { nw: 3, ne: 3, se: 0, sw: 0 }, col);
            if hov > 0.05 {
                let mins = (listen_ms / 1000 / 60).max(1);
                describe(&resp, &format!("{mins} min played ({plays} songs)"));
            }
        } else {
            ui.painter().rect_filled(b_rect, eframe::egui::CornerRadius { nw: 1, ne: 1, se: 0, sw: 0 }, veil(0.06));
        }

        if idx % 5 == 0 || idx == num_bars - 1 {
            let date_label = format!("Day {}", idx + 1);
            ui.painter().text(pos2(bar_x + bar_w / 2.0, baseline_y + 12.0), Align2::CENTER_CENTER, date_label, sans(10.0), m_color);
        }
    }
    ui.add_space(28.0);

    // 5. Two Column: Left Top Artists, Right Top Songs (Image 2)
    let col_w = (width - 24.0) / 2.0;
    let (col_row, _) = ui.allocate_exact_size(vec2(full, 390.0), Sense::hover());
    let left_x = col_row.left() + pad;
    let right_x = left_x + col_w + 24.0;

    // --- LEFT COLUMN: TOP ARTISTS ---
    ui.painter().text(pos2(left_x + 10.0, col_row.top() + 12.0), Align2::CENTER_CENTER, icon::USER, glyph(16.0), pal_color);
    label(ui, pos2(left_x + 24.0, col_row.top() + 12.0), Align2::LEFT_CENTER, "Top artists", bold(18.0), t_color, col_w - 30.0);

    if saved.stats_top_artists.is_empty() {
        let empty_rect = Rect::from_min_size(pos2(left_x, col_row.top() + 32.0), vec2(col_w, 160.0));
        ui.painter().rect_filled(empty_rect, 12.0, veil(0.03));
        label(ui, empty_rect.center(), Align2::CENTER_CENTER, "No artist history yet. Start playing songs!", sans(12.0), m_color, col_w - 20.0);
    } else {
        let top_art = &saved.stats_top_artists[0];
        let hero_rect = Rect::from_min_size(pos2(left_x, col_row.top() + 32.0), vec2(col_w, 172.0));
        ui.painter().rect_filled(hero_rect, 14.0, veil(0.04));
        ui.painter().rect_stroke(hero_rect, 14.0, Stroke::new(1.0_f32, veil(0.08)), StrokeKind::Inside);

        let disc_center = pos2(hero_rect.left() + 78.0, hero_rect.center().y);
        let cover_str = &top_art.thumb;
        if !cover_str.is_empty() {
            rotating_circular_artwork(ui, cx.art, cover_str, disc_center, 50.0, 0.0, pal_color);
        } else {
            ui.painter().circle_filled(disc_center, 50.0, veil(0.08));
            ui.painter().text(disc_center, Align2::CENTER_CENTER, icon::USER, glyph(42.0), pal_color);
        }

        let info_x = hero_rect.left() + 144.0;
        let info_w = col_w - 156.0;
        ui.painter().text(pos2(info_x, hero_rect.top() + 40.0), Align2::LEFT_CENTER, "01", bold(14.0), pal_color);
        label(ui, pos2(info_x, hero_rect.top() + 64.0), Align2::LEFT_CENTER, &top_art.artist, bold(18.0), t_color, info_w);
        let mins = (top_art.listen_ms / 60000).max(1);
        let sub_art = format!("{mins} min · {} plays", top_art.plays);
        label(ui, pos2(info_x, hero_rect.top() + 90.0), Align2::LEFT_CENTER, &sub_art, sans(13.0), m_color, info_w);
        let share_pct = if total_ms > 0 { ((top_art.listen_ms * 100) / total_ms).min(100) } else { 100 };
        label(ui, pos2(info_x, hero_rect.top() + 114.0), Align2::LEFT_CENTER, &format!("{share_pct}% of your listening"), sans(12.0), m_color, info_w);

        let mut row_y = hero_rect.bottom() + 14.0;
        for (idx, art) in saved.stats_top_artists.iter().enumerate().skip(1).take(2) {
            let a_row = Rect::from_min_size(pos2(left_x, row_y), vec2(col_w, 54.0));
            let rank_str = format!("{:02}", idx + 1);
            ui.painter().text(pos2(a_row.left() + 10.0, a_row.center().y), Align2::LEFT_CENTER, rank_str, medium(13.0), m_color);

            let a_disc = pos2(a_row.left() + 54.0, a_row.center().y);
            if !art.thumb.is_empty() {
                rotating_circular_artwork(ui, cx.art, &art.thumb, a_disc, 18.0, 0.0, pal_color);
            } else {
                ui.painter().circle_filled(a_disc, 18.0, veil(0.08));
                ui.painter().text(a_disc, Align2::CENTER_CENTER, icon::USER, glyph(16.0), pal_color);
            }

            let name_x = a_row.left() + 84.0;
            label(ui, pos2(name_x, a_row.center().y - 10.0), Align2::LEFT_CENTER, &art.artist, medium(13.5), t_color, col_w - 160.0);

            let p_track = Rect::from_min_size(pos2(name_x, a_row.center().y + 10.0), vec2(col_w - 170.0, 4.0));
            ui.painter().rect_filled(p_track, 2.0, veil(0.10));
            let top_ms = top_art.listen_ms.max(1);
            let p_frac = (art.listen_ms as f32 / top_ms as f32).clamp(0.05, 1.0);
            ui.painter().rect_filled(Rect::from_min_size(p_track.min, vec2(p_track.width() * p_frac, 4.0)), 2.0, pal_color);

            let art_mins = (art.listen_ms / 60000).max(1);
            ui.painter().text(pos2(a_row.right() - 8.0, a_row.center().y), Align2::RIGHT_CENTER, format!("{art_mins} min"), sans(12.0), m_color);

            row_y += 66.0;
        }
    }

    // --- RIGHT COLUMN: TOP SONGS ---
    ui.painter().text(pos2(right_x + 10.0, col_row.top() + 12.0), Align2::CENTER_CENTER, icon::MUSIC_NOTE, glyph(16.0), pal_color);
    label(ui, pos2(right_x + 24.0, col_row.top() + 12.0), Align2::LEFT_CENTER, "Top songs", bold(18.0), t_color, col_w - 30.0);

    if saved.stats_top_tracks.is_empty() {
        let empty_rect = Rect::from_min_size(pos2(right_x, col_row.top() + 32.0), vec2(col_w, 160.0));
        ui.painter().rect_filled(empty_rect, 12.0, veil(0.03));
        label(ui, empty_rect.center(), Align2::CENTER_CENTER, "No songs played yet. Tracks will rank here!", sans(12.0), m_color, col_w - 20.0);
    } else {
        let mut s_row_y = col_row.top() + 32.0;
        for (idx, song) in saved.stats_top_tracks.iter().take(5).enumerate() {
            let s_rect = Rect::from_min_size(pos2(right_x, s_row_y), vec2(col_w, 56.0));
            let resp = ui.interact(s_rect, Id::new(("top-song-row", idx)), Sense::click());
            let hov = hover_of(ui, &resp);

            if hov > 0.05 {
                ui.painter().rect_filled(s_rect, 8.0, veil(0.05 * hov));
            }

            ui.painter().text(pos2(s_rect.left() + 10.0, s_rect.center().y), Align2::LEFT_CENTER, format!("{}", idx + 1), bold(13.5), m_color);

            let thumb_rect = Rect::from_center_size(pos2(s_rect.left() + 54.0, s_rect.center().y), vec2(42.0, 42.0));
            artwork(ui, cx.art, &song.thumb, thumb_rect, 6.0);

            if hov > 0.1 {
                ui.painter().rect_filled(thumb_rect, 6.0, shade(0.40));
                ui.painter().text(thumb_rect.center(), Align2::CENTER_CENTER, icon::PLAY, glyph(16.0), Color32::WHITE);
            }

            let text_x = s_rect.left() + 86.0;
            label(ui, pos2(text_x, s_rect.center().y - 10.0), Align2::LEFT_CENTER, &song.title, medium(13.5), t_color, col_w - 190.0);
            label(ui, pos2(text_x, s_rect.center().y + 10.0), Align2::LEFT_CENTER, &song.artist, sans(12.0), m_color, col_w - 190.0);

            ui.painter().text(pos2(s_rect.right() - 60.0, s_rect.center().y), Align2::RIGHT_CENTER, format!("{} plays", song.plays), sans(11.5), m_color);

            let dur_str = if !song.duration.is_empty() { &song.duration } else { "3:20" };
            ui.painter().text(pos2(s_rect.right() - 8.0, s_rect.center().y), Align2::RIGHT_CENTER, dur_str, sans(11.5), m_color);

            if resp.clicked() {
                cx.out.push(Action::Play(
                    vec![Item {
                        title: song.title.clone(),
                        subtitle: song.artist.clone(),
                        thumb: song.thumb.clone(),
                        duration: song.duration.clone(),
                        video_id: song.video_id.clone(),
                        ..Default::default()
                    }],
                    0,
                ));
            }
            s_row_y += 62.0;
        }
    }
    ui.add_space(28.0);

    // 6. Top Albums Section (Image 3)
    if !saved.stats_top_albums.is_empty() {
        let (alb_head, _) = ui.allocate_exact_size(vec2(full, 30.0), Sense::hover());
        let alb_left = alb_head.left() + pad;
        ui.painter().text(pos2(alb_left + 10.0, alb_head.center().y), Align2::CENTER_CENTER, icon::DISC, glyph(16.0), pal_color);
        label(ui, pos2(alb_left + 24.0, alb_head.center().y), Align2::LEFT_CENTER, "Top albums", bold(18.0), t_color, width);
        ui.add_space(10.0);

        let (shelf_row, _) = ui.allocate_exact_size(vec2(full, 180.0), Sense::hover());
        let alb_card_w = 140.0;
        let mut alb_x = shelf_row.left() + pad;
        for (a_idx, alb) in saved.stats_top_albums.iter().take(5).enumerate() {
            let a_card = Rect::from_min_size(pos2(alb_x, shelf_row.top()), vec2(alb_card_w, 175.0));
            let a_resp = ui.interact(a_card, Id::new(("top-alb-card", a_idx)), Sense::click());
            let a_hov = hover_of(ui, &a_resp);

            let art_rect = Rect::from_min_size(a_card.min, vec2(alb_card_w, alb_card_w));
            artwork(ui, cx.art, &alb.thumb, art_rect, 10.0);
            if a_hov > 0.1 {
                ui.painter().rect_filled(art_rect, 10.0, shade(0.35));
                ui.painter().circle_filled(art_rect.center(), 18.0, pal_color);
                ui.painter().text(art_rect.center(), Align2::CENTER_CENTER, icon::PLAY, glyph(16.0), Color32::WHITE);
            }
            label(ui, pos2(a_card.left(), art_rect.bottom() + 10.0), Align2::LEFT_CENTER, &alb.title, medium(12.5), t_color, alb_card_w);
            label(ui, pos2(a_card.left(), art_rect.bottom() + 24.0), Align2::LEFT_CENTER, &alb.artist, sans(11.0), m_color, alb_card_w);

            alb_x += alb_card_w + 14.0;
        }
        ui.add_space(24.0);
    }

    // 7. "When you listen" - Donut Chart & Heatmap (Image 4)
    let (listen_head, _) = ui.allocate_exact_size(vec2(full, 30.0), Sense::hover());
    let listen_left = listen_head.left() + pad;
    ui.painter().text(pos2(listen_left + 10.0, listen_head.center().y), Align2::CENTER_CENTER, icon::CLOCK, glyph(16.0), pal_color);
    label(ui, pos2(listen_left + 24.0, listen_head.center().y), Align2::LEFT_CENTER, "When you listen", bold(18.0), t_color, width);
    ui.add_space(10.0);

    let (m_ms, a_ms, e_ms, n_ms) = {
        let mut m = 0u64;
        let mut a = 0u64;
        let mut e = 0u64;
        let mut n = 0u64;
        for d in &saved.stats_daily {
            for h in 0..24 {
                let ms = d.hourly_ms[h] as u64;
                if h >= 6 && h < 12 { m += ms; }
                else if h >= 12 && h < 17 { a += ms; }
                else if h >= 17 && h < 21 { e += ms; }
                else { n += ms; }
            }
        }
        (m, a, e, n)
    };
    let sum_daypart = (m_ms + a_ms + e_ms + n_ms).max(1);
    let pct_m = ((m_ms * 100) / sum_daypart) as u32;
    let pct_a = ((a_ms * 100) / sum_daypart) as u32;
    let pct_e = ((e_ms * 100) / sum_daypart) as u32;
    let pct_n = (100u32).saturating_sub(pct_m + pct_a + pct_e);

    let (top_pct, top_label, persona_title, persona_desc) = {
        if pct_m >= pct_a && pct_m >= pct_e && pct_m >= pct_n && pct_m > 0 {
            (pct_m, "Morning", "Morning listener", "You listen most between 6 AM and 12 PM.")
        } else if pct_a >= pct_e && pct_a >= pct_n && pct_a > 0 {
            (pct_a, "Afternoon", "Afternoon listener", "You listen most between 12 PM and 5 PM.")
        } else if pct_e >= pct_n && pct_e > 0 {
            (pct_e, "Evening", "Evening listener", "You listen most between 5 PM and 9 PM.")
        } else if pct_n > 0 {
            (pct_n, "Night", "Night owl", "You listen most between 9 PM and 6 AM.")
        } else {
            (0, "Anytime", "Everyday listener", "Play more tracks to reveal your listening personality.")
        }
    };

    let card_height = 310.0;
    let (container, _) = ui.allocate_exact_size(vec2(full, card_height), Sense::hover());
    let donut_w = (width * 0.38).clamp(340.0, 440.0);
    let heatmap_w = width - donut_w - 20.0;

    let donut_rect = Rect::from_min_size(pos2(container.left() + pad, container.top()), vec2(donut_w, card_height));
    let heat_rect = Rect::from_min_size(pos2(donut_rect.right() + 20.0, container.top()), vec2(heatmap_w, card_height));

    // Left Card: Donut Chart Container (Image 3)
    ui.painter().rect_filled(donut_rect, 14.0, veil(0.04));
    ui.painter().rect_stroke(donut_rect, 14.0, Stroke::new(1.0_f32, veil(0.08)), StrokeKind::Inside);

    let donut_center = pos2(donut_rect.center().x, donut_rect.top() + 95.0);
    let r_outer = 66.0_f32;
    let stroke_w = 18.0_f32;
    let r_mid = r_outer - stroke_w / 2.0;

    ui.painter().circle_stroke(donut_center, r_mid, Stroke::new(stroke_w, veil(0.06)));

    let col_m = Color32::from_rgb(244, 166, 175);
    let col_a = pal_color;
    let col_e = Color32::from_rgb(168, 85, 247);
    let col_n = Color32::from_rgb(99, 102, 241);

    if top_pct > 0 {
        let mut cur_angle = -std::f32::consts::FRAC_PI_2;
        let dayparts = [
            (pct_m, col_m),
            (pct_a, col_a),
            (pct_e, col_e),
            (pct_n, col_n),
        ];
        for (pct, col) in dayparts {
            if pct == 0 { continue; }
            let sweep = (pct as f32 / 100.0) * std::f32::consts::TAU;
            let steps = (sweep * 16.0).max(4.0) as usize;
            let mut points = Vec::with_capacity(steps + 1);
            for s in 0..=steps {
                let theta = cur_angle + (s as f32 / steps as f32) * sweep;
                points.push(pos2(donut_center.x + r_mid * theta.cos(), donut_center.y + r_mid * theta.sin()));
            }
            if points.len() >= 2 {
                ui.painter().add(Shape::Path(eframe::epaint::PathShape {
                    points,
                    closed: false,
                    fill: Color32::TRANSPARENT,
                    stroke: Stroke::new(stroke_w, col).into(),
                }));
            }
            cur_angle += sweep;
        }
    }

    ui.painter().text(pos2(donut_center.x, donut_center.y - 7.0), Align2::CENTER_CENTER, format!("{top_pct}%"), bold(24.0), t_color);
    ui.painter().text(pos2(donut_center.x, donut_center.y + 12.0), Align2::CENTER_CENTER, top_label, sans(11.0), m_color);

    let leg_y = donut_rect.top() + 185.0;
    let col1_x = donut_rect.left() + 28.0;
    let col2_x = donut_rect.left() + donut_w * 0.52;
    let leg_col_w = donut_w * 0.44;

    let draw_legend_item = |ui: &Ui, center_x: f32, y: f32, col: Color32, name: &str, pct: u32| {
        ui.painter().circle_filled(pos2(center_x, y), 4.5, col);
        ui.painter().text(pos2(center_x + 12.0, y), Align2::LEFT_CENTER, name, sans(12.0), m_color);
        ui.painter().text(pos2(center_x + leg_col_w - 10.0, y), Align2::RIGHT_CENTER, format!("{pct}%"), medium(12.0), t_color);
    };

    draw_legend_item(ui, col1_x, leg_y, col_m, "Morning", pct_m);
    draw_legend_item(ui, col2_x, leg_y, col_a, "Afternoon", pct_a);
    draw_legend_item(ui, col1_x, leg_y + 24.0, col_e, "Evening", pct_e);
    draw_legend_item(ui, col2_x, leg_y + 24.0, col_n, "Night", pct_n);

    let badge_y = donut_rect.bottom() - 36.0;
    ui.painter().text(pos2(col1_x, badge_y), Align2::LEFT_CENTER, icon::SUN, solid(13.0), pal_color);
    label(ui, pos2(col1_x + 18.0, badge_y), Align2::LEFT_CENTER, persona_title, bold(12.5), t_color, donut_w - 50.0);
    label(ui, pos2(col1_x, badge_y + 16.0), Align2::LEFT_CENTER, persona_desc, sans(10.5), m_color, donut_w - 40.0);

    // Right Card: Weekly Activity Heatmap (Image 4)
    ui.painter().rect_filled(heat_rect, 14.0, veil(0.04));
    ui.painter().rect_stroke(heat_rect, 14.0, Stroke::new(1.0_f32, veil(0.08)), StrokeKind::Inside);

    let heat_info_y = heat_rect.top() + 20.0;
    label(ui, pos2(heat_rect.left() + 24.0, heat_info_y), Align2::LEFT_CENTER, "Busiest day of the week", sans(11.5), m_color, 160.0);
    label(ui, pos2(heat_rect.left() + 24.0, heat_info_y + 16.0), Align2::LEFT_CENTER, "Sunday", bold(13.5), t_color, 160.0);

    label(ui, pos2(heat_rect.left() + 180.0, heat_info_y), Align2::LEFT_CENTER, "Busiest hour", sans(11.5), m_color, 160.0);
    label(ui, pos2(heat_rect.left() + 180.0, heat_info_y + 16.0), Align2::LEFT_CENTER, "12 PM", bold(13.5), t_color, 160.0);

    let grid_top = heat_rect.top() + 66.0;
    let grid_left = heat_rect.left() + 54.0;
    let grid_avail_w = heat_rect.width() - 74.0;
    let cell_gap = 3.0;
    let cell_w = ((grid_avail_w - 23.0 * cell_gap) / 24.0).max(8.0);
    let cell_h = 16.0;

    let day_names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let max_cell_ms = {
        let mut m = 0u32;
        for d in &saved.stats_daily {
            for h in 0..24 {
                m = m.max(d.hourly_ms[h]);
            }
        }
        m.max(60000)
    };

    for (d_idx, d_name) in day_names.iter().enumerate() {
        let row_y = grid_top + d_idx as f32 * (cell_h + cell_gap);
        ui.painter().text(pos2(heat_rect.left() + 24.0, row_y + cell_h / 2.0), Align2::LEFT_CENTER, *d_name, sans(11.0), m_color);

        for h in 0..24 {
            let cell_x = grid_left + h as f32 * (cell_w + cell_gap);
            let cell_rect = Rect::from_min_size(pos2(cell_x, row_y), vec2(cell_w, cell_h));
            let cell_resp = ui.interact(cell_rect, Id::new(("heat-cell", d_idx, h)), Sense::hover());
            let cell_hov = hover_of(ui, &cell_resp);

            let cell_ms: u32 = saved.stats_daily.iter()
                .filter(|d| (d.day_key % 7) as usize == (d_idx + 3) % 7)
                .map(|d| d.hourly_ms[h])
                .sum();

            if cell_ms > 0 {
                let frac = (cell_ms as f32 / max_cell_ms as f32).clamp(0.25, 1.0);
                let col = pal_color.gamma_multiply(frac).lerp_to_gamma(Color32::WHITE, 0.20 * cell_hov);
                ui.painter().rect_filled(cell_rect, 2.0, col);
                if cell_hov > 0.05 {
                    let hour_label = if h == 0 { "12 AM".to_string() } else if h < 12 { format!("{h} AM") } else if h == 12 { "12 PM".to_string() } else { format!("{} PM", h - 12) };
                    let mins = (cell_ms / 60000).max(1);
                    describe(&cell_resp, &format!("{d_name} {hour_label} · {mins} min played"));
                }
            } else {
                ui.painter().rect_filled(cell_rect, 2.0, veil(0.04 + 0.04 * cell_hov));
            }
        }
    }

    let hours_y = grid_top + 7.0 * (cell_h + cell_gap) + 12.0;
    let mark_hours = [(0, "12 AM"), (6, "6 AM"), (12, "12 PM"), (18, "6 PM")];
    for &(h, label_str) in &mark_hours {
        let x = grid_left + h as f32 * (cell_w + cell_gap);
        ui.painter().text(pos2(x, hours_y), Align2::LEFT_CENTER, label_str, sans(10.5), m_color);
    }

    let legend_y = hours_y + 22.0;
    let leg_right = heat_rect.right() - 24.0;
    ui.painter().text(pos2(leg_right - 70.0, legend_y), Align2::RIGHT_CENTER, "Less", sans(10.5), m_color);
    for sw in 0..4 {
        let sw_x = leg_right - 62.0 + sw as f32 * 11.0;
        let sw_col = if sw == 0 { veil(0.04) } else { pal_color.gamma_multiply(sw as f32 * 0.33) };
        ui.painter().rect_filled(Rect::from_min_size(pos2(sw_x, legend_y - 4.0), vec2(8.0, 8.0)), 2.0, sw_col);
    }
    ui.painter().text(pos2(leg_right, legend_y), Align2::LEFT_CENTER, "More", sans(10.5), m_color);
    ui.add_space(28.0);

    // 8. Bottom Footer (Image 4)
    let (foot_row, _) = ui.allocate_exact_size(vec2(full, 36.0), Sense::hover());
    let f_left = foot_row.left() + pad;
    let f_right = f_left + width;
    ui.painter().text(pos2(f_left, foot_row.center().y), Align2::LEFT_CENTER, "Counted on this device from what you play in AURI. Nothing is sent anywhere.", sans(11.5), m_color);

    let clear_btn_rect = Rect::from_center_size(pos2(f_right - 80.0, foot_row.center().y), vec2(150.0, 28.0));
    let cl_resp = ui.interact(clear_btn_rect, Id::new("stat-clear-history-btn"), Sense::click());
    let cl_hov = hover_of(ui, &cl_resp);
    ui.painter().rect_filled(clear_btn_rect, 6.0, veil(0.04 + 0.05 * cl_hov));
    let cl_col = if cl_hov > 0.1 { Color32::from_rgb(244, 63, 94) } else { m_color };
    ui.painter().text(pos2(clear_btn_rect.left() + 14.0, clear_btn_rect.center().y), Align2::CENTER_CENTER, icon::TRASH, glyph(13.0), cl_col);
    label(ui, pos2(clear_btn_rect.left() + 26.0, clear_btn_rect.center().y), Align2::LEFT_CENTER, "Clear listening history", sans(11.5), cl_col, 120.0);
    if cl_resp.clicked() {
        cx.out.push(Action::ClearStats);
    }
    ui.add_space(40.0);
}

#[allow(dead_code)]
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

fn sidebar(app: &mut App, ui: &mut Ui, anim_w: f32, out: &mut Vec<Action>) {
    let area = ui.max_rect();
    let is_light = app.saved.theme_mode == ThemeMode::Light;
    let t_color = if is_light { Color32::from_rgb(24, 28, 36) } else { TEXT };
    let m_color = if is_light { Color32::from_rgb(100, 110, 125) } else { MUTED };
    let pal_color = app.saved.theme_palette.color();

    ui.painter().vline(area.right() - 0.5, area.y_range(), Stroke::new(1.0_f32, veil(if is_light { 0.12 } else { 0.06 })));

    let is_search = matches!(app.route(), Route::Search(_));
    let is_library = matches!(app.route(), Route::Liked | Route::Recent);
    let is_settings = matches!(app.route(), Route::Settings(_));

    if anim_w < 110.0 {
        // --- DOCKED SIDEBAR (Image 1) ---
        // 1. Expand Arrow Button (round accent circle with > pointing right)
        let exp_btn_rect = Rect::from_center_size(pos2(area.left() + 30.0, area.top() + 32.0), vec2(30.0, 30.0));
        let exp_resp = ui.interact(exp_btn_rect, Id::new("docked-expand-btn"), Sense::click());
        let exp_hov = hover_of(ui, &exp_resp);
        ui.painter().circle_filled(exp_btn_rect.center(), 14.0, pal_color.gamma_multiply(0.12 + 0.12 * exp_hov));
        ui.painter().circle_stroke(exp_btn_rect.center(), 14.0, Stroke::new(1.2_f32, pal_color.gamma_multiply(0.70 + 0.30 * exp_hov)));
        ui.painter().text(exp_btn_rect.center(), Align2::CENTER_CENTER, icon::CARET_RIGHT, solid(14.0), pal_color);
        describe(&exp_resp, "Expand sidebar");
        if exp_resp.clicked() {
            out.push(Action::ToggleSidebar);
        }

        // 2. Docked Navigation Icons Column
        let links: [(&str, &str, Route, bool); 5] = [
            (icon::HOUSE, "Home", Route::Home, *app.route() == Route::Home),
            (icon::MAGNIFYING_GLASS, "Search", Route::Search(String::new()), is_search),
            (icon::BOOKMARK_SIMPLE, "Library", Route::Liked, is_library),
            (icon::CHART_BAR, "Stats", Route::Stats, *app.route() == Route::Stats),
            (icon::GEAR_SIX, "Settings", Route::Settings(SettingsTab::General), is_settings),
        ];

        let mut y = area.top() + 76.0;
        for (sym, name, route, active) in links {
            let item_rect = Rect::from_center_size(pos2(area.left() + 30.0, y), vec2(38.0, 38.0));
            let resp = ui.interact(item_rect, Id::new(("dock-nav-item", name)), Sense::click());
            let hov = hover_of(ui, &resp);

            if active {
                // Active indicator notch on left edge
                let notch = Rect::from_min_max(pos2(area.left(), item_rect.top() + 4.0), pos2(area.left() + 4.0, item_rect.bottom() - 4.0));
                ui.painter().rect_filled(notch, 2.0, pal_color);

                // Active background pill
                ui.painter().rect_filled(item_rect, 10.0, pal_color.gamma_multiply(0.20));
                if app.saved.selection_glow {
                    ui.painter().rect_stroke(item_rect, 10.0, Stroke::new(1.0_f32, pal_color.gamma_multiply(0.40)), StrokeKind::Inside);
                }
                ui.painter().text(item_rect.center(), Align2::CENTER_CENTER, sym, glyph(18.0), pal_color);
            } else {
                if hov > 0.05 {
                    ui.painter().rect_filled(item_rect, 10.0, veil(0.06 * hov));
                }
                let icon_col = if hov > 0.1 { t_color } else { m_color };
                ui.painter().text(item_rect.center(), Align2::CENTER_CENTER, sym, glyph(18.0), icon_col);
            }
            describe(&resp, name);
            if resp.clicked() {
                out.push(Action::Go(route));
            }
            y += 48.0;
        }

        return;
    }

    // Top Brand Row: Logo + Theme Light/Dark Toggle + Collapse Sidebar Button
    let (brand, response) = ui.allocate_exact_size(vec2(area.width(), 64.0), Sense::click());
    if response.clicked() {
        out.push(Action::Go(Route::Home));
    }
    let banner_rect = Rect::from_min_size(pos2(brand.left() + 18.0, brand.center().y - 18.0), vec2(100.0, 36.0));
    label(ui, pos2(banner_rect.left(), banner_rect.center().y), Align2::LEFT_CENTER, "AURI", bold(22.0), t_color, 100.0);

    // Collapse Sidebar Button
    let col_btn_rect = Rect::from_center_size(pos2(brand.right() - 24.0, brand.center().y), vec2(26.0, 26.0));
    let col_resp = ui.interact(col_btn_rect, Id::new("sidebar-collapse-btn"), Sense::click());
    let col_hover = hover_of(ui, &col_resp);
    ui.painter().rect_filled(col_btn_rect, 6.0, veil(0.04 + 0.06 * col_hover));
    ui.painter().text(col_btn_rect.center(), Align2::CENTER_CENTER, icon::CARET_LEFT, solid(16.0), if col_hover > 0.1 { t_color } else { m_color });
    describe(&col_resp, "Collapse sidebar");
    if col_resp.clicked() {
        out.push(Action::ToggleSidebar);
    }

    // Navigation Tabs (Image 1): Home, Search, Library, Stats, Settings
    let count = |items: &[Item]| if items.is_empty() { String::new() } else { items.len().to_string() };
    let is_search = matches!(app.route(), Route::Search(_));
    let is_library = matches!(app.route(), Route::Liked | Route::Recent);
    let is_settings = matches!(app.route(), Route::Settings(_));

    let links = [
        (icon::HOUSE, "Home", Route::Home, String::new(), *app.route() == Route::Home),
        (icon::MAGNIFYING_GLASS, "Search", Route::Search(String::new()), String::new(), is_search),
        (icon::BOOKMARK_SIMPLE, "Library", Route::Liked, count(&app.saved.liked), is_library),
        (icon::CHART_BAR, "Stats", Route::Stats, if app.saved.stats_total_plays > 0 { app.saved.stats_total_plays.to_string() } else { String::new() }, *app.route() == Route::Stats),
        (icon::GEAR_SIX, "Settings", Route::Settings(SettingsTab::General), String::new(), is_settings),
    ];

    for (symbol, text, route, trailing, active) in links {
        if nav_item(ui, symbol, text, active, &trailing, app.saved.selection_glow, pal_color).clicked() {
            out.push(Action::Go(route));
        }
    }

    ui.add_space(14.0);
    let (sep_row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 2.0), Sense::hover());
    ui.painter().hline(sep_row.x_range().shrink(14.0), sep_row.center().y, Stroke::new(1.0_f32, veil(0.08)));
    ui.add_space(12.0);

    // + New Playlist Button (Image 1)
    let (pl_slot, pl_resp) = ui.allocate_exact_size(vec2(ui.available_width() - 24.0, 36.0), Sense::click());
    let pl_rect = pl_slot.translate(vec2(12.0, 0.0));
    let pl_hov = hover_of(ui, &pl_resp);
    ui.painter().rect_filled(pl_rect, 8.0, veil(if is_light { 0.08 + 0.04 * pl_hov } else { 0.05 + 0.04 * pl_hov }));
    ui.painter().rect_stroke(pl_rect, 8.0, Stroke::new(1.0_f32, veil(0.10 + 0.08 * pl_hov)), StrokeKind::Inside);
    ui.painter().text(pos2(pl_rect.left() + 20.0, pl_rect.center().y), Align2::CENTER_CENTER, icon::PLUS, solid(14.0), pal_color);
    label(ui, pos2(pl_rect.left() + 38.0, pl_rect.center().y), Align2::LEFT_CENTER, "New Playlist", medium(13.0), t_color, pl_rect.width() - 50.0);
    if pl_resp.clicked() {
        out.push(Action::PromptCreatePlaylist);
    }
    ui.add_space(8.0);

    let (header_row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());
    label(ui, pos2(header_row.left() + 26.0, header_row.center().y), Align2::LEFT_CENTER, "PLAYLISTS", bold(11.0), FAINT, 120.0);

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
                    if ui.button(RichText::new("✓").color(pal_color)).clicked() {
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

    let bottom_limit = area.bottom();
    let dl_box_h = 46.0;
    let dl_lift = if app.download_active.is_some() { dl_box_h + 8.0 } else { 0.0 };
    if let Some(active) = &app.download_active {
        let dl_rect = Rect::from_min_size(
            pos2(area.left() + 10.0, bottom_limit - dl_box_h - 4.0),
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
    let offer = Rect::from_min_max(pos2(area.left(), bottom_limit - lift), pos2(area.right(), bottom_limit));
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
                ui.painter().rect_stroke(rect.expand(1.0), ROW_RADIUS + 1.0, Stroke::new(1.0_f32, pal_color.gamma_multiply(0.35)), StrokeKind::Outside);
                ui.painter().rect_stroke(rect, ROW_RADIUS, Stroke::new(1.4_f32, pal_color.lerp_to_gamma(Color32::WHITE, 0.40)), StrokeKind::Inside);
            }
            let icon_rect = Rect::from_min_size(pos2(rect.left() + 8.0, rect.center().y - 20.0), Vec2::splat(40.0));
            ui.painter().rect_filled(icon_rect, 6.0, pal_color.gamma_multiply(0.14));
            ui.painter().text(icon_rect.center(), Align2::CENTER_CENTER, icon::PLAYLIST, glyph(20.0), pal_color);
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
                ui.painter().rect_stroke(rect.expand(1.0), ROW_RADIUS + 1.0, Stroke::new(1.0_f32, pal_color.gamma_multiply(0.35)), StrokeKind::Outside);
                ui.painter().rect_stroke(rect, ROW_RADIUS, Stroke::new(1.4_f32, pal_color.lerp_to_gamma(Color32::WHITE, 0.40)), StrokeKind::Inside);
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
    let pal_color = app.saved.theme_palette.color();
    let row = slot.shrink2(vec2(12.0, 4.0));
    let response = ui.interact(row, Id::new("update-offer"), Sense::click());
    let hover = hover_of(ui, &response);
    ui.painter()
        .rect_filled(row, ROW_RADIUS, pal_color.gamma_multiply(0.14 + 0.08 * hover));
    let mark = pos2(row.left() + 24.0, row.center().y);
    if app.updating {
        egui::Spinner::new()
            .color(pal_color)
            .paint_at(ui, Rect::from_center_size(mark, Vec2::splat(16.0)));
    } else {
        ui.painter()
            .text(mark, Align2::CENTER_CENTER, icon::ARROW_CIRCLE_UP, solid(19.0), pal_color);
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
        out.push(Action::OpenUpdateDialog);
    }
}

#[allow(dead_code)]
fn account(app: &App, ui: &Ui, foot: Rect, out: &mut Vec<Action>) {
    let pal_color = app.saved.theme_palette.color();
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
                        let row = menu_row(ui, symbol, glyph(15.0), pal_color, &channel.name);
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
    let pal_color = app.saved.theme_palette.color();
    let eq_tone = if is_eq { pal_color } else { MUTED };
    if icon_button(ui, eq_slot, Id::new("sound-eq-btn"), icon::SLIDERS, glyph(18.0), eq_tone, "Equalizer").clicked() {
        out.push(Action::Go(Route::Settings(SettingsTab::Equalizer)));
    }

    // 3. Update Check Button (between Equalizer and Settings)
    let upd_slot = Rect::from_center_size(pos2(b3_x, mid_y), btn_size);
    let has_upd = app.update.is_some();
    let upd_icon = if has_upd { icon::ARROW_CIRCLE_UP } else { icon::ARROW_CLOCKWISE };
    let upd_tone = if has_upd { pal_color } else { MUTED };
    let upd_hint = if has_upd { "Update Available (Click to Install)" } else { "Check for Updates" };
    if icon_button(ui, upd_slot, Id::new("dock-update-btn"), upd_icon, if has_upd { solid(18.0) } else { glyph(18.0) }, upd_tone, upd_hint).clicked() {
        if has_upd {
            out.push(Action::OpenUpdateDialog);
        } else {
            out.push(Action::CheckUpdate);
        }
    }

    // 4. Settings Button
    let is_settings = matches!(app.route(), Route::Settings(tab) if *tab != SettingsTab::Equalizer);
    let set_slot = Rect::from_center_size(pos2(b4_x, mid_y), btn_size);
    let set_tone = if is_settings { pal_color } else { MUTED };
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
                    let pal_color = app.saved.theme_palette.color();
                    egui::Spinner::new()
                        .color(pal_color)
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

fn update_dialog(app: &mut App, ctx: &egui::Context, out: &mut Vec<Action>) {
    if !app.update_dialog_open {
        return;
    }
    let Some(version) = &app.update else {
        return;
    };
    let version_str = version.clone();
    let pal_color = app.saved.theme_palette.color();

    let frame = Frame::new()
        .fill(Color32::from_rgb(18, 20, 28))
        .stroke(Stroke::new(1.5_f32, pal_color.gamma_multiply(0.40)))
        .corner_radius(18)
        .inner_margin(Margin::same(26));

    let modal = egui::Modal::new(Id::new("update-available-modal"))
        .frame(frame)
        .backdrop_color(shade(0.65))
        .show(ctx, |ui| {
            ui.set_width(450.0);
            let width = ui.available_width();

            // Header Row: Icon + Title + Close Button
            let (head, _) = ui.allocate_exact_size(vec2(width, 40.0), Sense::hover());
            let icon_center = pos2(head.left() + 18.0, head.center().y);
            ui.painter().circle_filled(icon_center, 18.0, pal_color.gamma_multiply(0.20));
            ui.painter().text(icon_center, Align2::CENTER_CENTER, icon::SPARKLE, solid(20.0), pal_color);
            label(ui, pos2(head.left() + 46.0, head.center().y), Align2::LEFT_CENTER, "Software Update Ready", display(24.0), TEXT, width - 85.0);

            let close_slot = Rect::from_center_size(pos2(head.right() - 14.0, head.center().y), Vec2::splat(28.0));
            let closed = icon_button(ui, close_slot, Id::new("update-close-btn"), icon::X, glyph(16.0), MUTED, "Close").clicked() && !app.updating;

            ui.add_space(8.0);
            paragraph(ui, "A new update for AURI is available directly from GitHub Releases.", sans(13.0), MUTED, 2);
            ui.add_space(14.0);

            // Version Badges Comparison Row
            let (v_row, _) = ui.allocate_exact_size(vec2(width, 32.0), Sense::hover());
            let cur_pill = Rect::from_min_size(pos2(v_row.left(), v_row.top()), vec2(140.0, 30.0));
            ui.painter().rect_filled(cur_pill, 15.0, veil(0.08));
            ui.painter().rect_stroke(cur_pill, 15.0, Stroke::new(1.0_f32, veil(0.12)), StrokeKind::Inside);
            let cur_lbl = format!("Current: v{}", crate::VERSION);
            ui.painter().text(cur_pill.center(), Align2::CENTER_CENTER, &cur_lbl, medium(12.0), MUTED);

            let arrow_center = pos2(cur_pill.right() + 18.0, v_row.center().y);
            ui.painter().text(arrow_center, Align2::CENTER_CENTER, icon::ARROW_RIGHT, solid(16.0), pal_color);

            let new_pill = Rect::from_min_size(pos2(arrow_center.x + 18.0, v_row.top()), vec2(140.0, 30.0));
            ui.painter().rect_filled(new_pill, 15.0, pal_color.gamma_multiply(0.20));
            ui.painter().rect_stroke(new_pill, 15.0, Stroke::new(1.5_f32, pal_color), StrokeKind::Inside);
            let new_lbl = format!("Latest: v{}", version_str);
            ui.painter().text(new_pill.center(), Align2::CENTER_CENTER, &new_lbl, bold(12.5), Color32::WHITE);

            ui.add_space(16.0);

            // Highlights card
            let card_frame = Frame::new()
                .fill(veil(0.04))
                .stroke(Stroke::new(1.0_f32, veil(0.08)))
                .corner_radius(10)
                .inner_margin(Margin::same(14));
            card_frame.show(ui, |ui| {
                ui.label(RichText::new("What's New in This Version:").strong().size(12.5).color(TEXT));
                ui.add_space(4.0);
                ui.label(RichText::new("• Pure Desktop Overlay HUD in Mini Mode (Metrix-Engine)").size(11.5).color(TEXT));
                ui.label(RichText::new("• YouTube Music account avatar & 1-click Log Out menu").size(11.5).color(TEXT));
                ui.label(RichText::new("• Seamless Track Crossfade with automatic Gaming Mode bypass").size(11.5).color(TEXT));
                ui.label(RichText::new("• High-Quality 160k vs Data-Saver 70k Opus stream bitrate").size(11.5).color(TEXT));
                ui.label(RichText::new("• Blocked Artists auto-skip filter & offline audio cache manager").size(11.5).color(TEXT));
            });

            ui.add_space(20.0);

            // Bottom Actions / Status
            if app.updating {
                let (prog_row, _) = ui.allocate_exact_size(vec2(width, 36.0), Sense::hover());
                let spin_center = pos2(prog_row.left() + 20.0, prog_row.center().y);
                egui::Spinner::new().color(pal_color).paint_at(ui, Rect::from_center_size(spin_center, Vec2::splat(18.0)));
                label(ui, pos2(prog_row.left() + 42.0, prog_row.center().y), Align2::LEFT_CENTER, "Downloading & Installing update package...", medium(13.0), pal_color, width - 60.0);
            } else {
                let (act_row, _) = ui.allocate_exact_size(vec2(width, 36.0), Sense::hover());

                // Later button
                let later_rect = Rect::from_min_size(pos2(act_row.left(), act_row.top()), vec2(90.0, 36.0));
                let later_resp = ui.interact(later_rect, Id::new("update-later-btn"), Sense::click());
                let l_hov = hover_of(ui, &later_resp);
                ui.painter().rect_filled(later_rect, 8.0, veil(0.06 + 0.05 * l_hov));
                ui.painter().rect_stroke(later_rect, 8.0, Stroke::new(1.0_f32, veil(0.12)), StrokeKind::Inside);
                ui.painter().text(later_rect.center(), Align2::CENTER_CENTER, "Later", medium(12.5), MUTED);
                if later_resp.clicked() {
                    out.push(Action::CloseUpdateDialog);
                }

                // Release notes button
                let rel_rect = Rect::from_min_size(pos2(later_rect.right() + 8.0, act_row.top()), vec2(130.0, 36.0));
                let rel_resp = ui.interact(rel_rect, Id::new("update-relnotes-btn"), Sense::click());
                let r_hov = hover_of(ui, &rel_resp);
                ui.painter().rect_filled(rel_rect, 8.0, veil(0.06 + 0.05 * r_hov));
                ui.painter().rect_stroke(rel_rect, 8.0, Stroke::new(1.0_f32, veil(0.12)), StrokeKind::Inside);
                ui.painter().text(rel_rect.center(), Align2::CENTER_CENTER, "Release Notes", medium(12.5), TEXT);
                if rel_resp.clicked() {
                    out.push(Action::OpenRelease);
                }

                // Update Now button (prominent)
                let upd_rect = Rect::from_min_size(pos2(act_row.right() - 140.0, act_row.top()), vec2(140.0, 36.0));
                let upd_resp = ui.interact(upd_rect, Id::new("update-install-btn"), Sense::click());
                let u_hov = hover_of(ui, &upd_resp);
                ui.painter().rect_filled(upd_rect, 8.0, pal_color.gamma_multiply(0.85 + 0.15 * u_hov));
                ui.painter().text(upd_rect.center(), Align2::CENTER_CENTER, "Update Now", bold(13.0), Color32::WHITE);
                if upd_resp.clicked() {
                    out.push(Action::GetUpdate);
                }
            }
            closed
        });

    if (modal.inner || modal.should_close()) && !app.updating {
        out.push(Action::CloseUpdateDialog);
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
            if app.saved.queue_retractable {
                let col_resp = icon_button(ui, Rect::from_center_size(pos2(q_head.right() - 8.0, q_head.center().y), vec2(20.0, 20.0)), Id::new("queue-collapse-btn"), icon::CARET_RIGHT, solid(15.0), MUTED, "Collapse queue");
                if col_resp.clicked() {
                    out.push(Action::ToggleQueue);
                }
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
    let pal_color = app.saved.theme_palette.color();
    let (tag_row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 26.0), Sense::hover());
    let is_synced = lyrics.synced && app.saved.lyrics_synced_mode;
    let (sync_text, sync_icon) = if is_synced {
        ("Synchronized Karaoke Lyrics", icon::SPARKLE)
    } else {
        ("Plain Lyrics", icon::TEXT_T)
    };
    let tag_rect = Rect::from_min_size(tag_row.min, vec2(196.0, 22.0));
    ui.painter().rect_filled(tag_rect, 6.0, pal_color.gamma_multiply(0.12));
    ui.painter().rect_stroke(tag_rect, 6.0, Stroke::new(1.0_f32, pal_color.gamma_multiply(0.30)), StrokeKind::Inside);
    ui.painter().text(pos2(tag_rect.left() + 10.0, tag_rect.center().y), Align2::LEFT_CENTER, sync_icon, glyph(12.0), pal_color);
    ui.painter().text(pos2(tag_rect.left() + 26.0, tag_rect.center().y), Align2::LEFT_CENTER, sync_text, medium(11.0), pal_color);
    ui.add_space(8.0);

    let position = app.player.position_ms();
    let sung = if is_synced {
        lyrics.lines.iter().rposition(|(start, _)| *start <= position)
    } else {
        None
    };
    let follow = Id::new("lyrics-follow");
    let moved = ui.data(|data| data.get_temp::<Option<usize>>(follow)) != Some(sung);
    ui.data_mut(|data| data.insert_temp(follow, sung));
    let width = ui.available_width();
    let sense = if lyrics.synced { Sense::click() } else { Sense::hover() };
    let base_sz = app.saved.lyrics_font_size.clamp(16.0, 36.0);
    let align_center = app.saved.lyrics_align_center;
    let dim_unsung = app.saved.lyrics_dim_unsung.clamp(0.10, 0.80);

    for (index, (start, line)) in lyrics.lines.iter().enumerate() {
        if line.is_empty() {
            ui.add_space(18.0);
            continue;
        }
        let current = sung == Some(index);
        let font = if current { bold(base_sz) } else { medium((base_sz - 4.0).max(12.0)) };
        let galley = fit(ui, line, font, TEXT, width - 40.0, 4);
        let (rect, response) = ui.allocate_exact_size(vec2(width, galley.size().y + 14.0), sense);
        if response.clicked() && lyrics.synced {
            out.push(Action::Seek(*start));
        }
        if current && moved {
            ui.scroll_to_rect(rect, Some(Align::Center));
        }
        if !ui.is_rect_visible(rect) {
            continue;
        }
        let lit = ui
            .ctx()
            .animate_bool_with_time(response.id.with("lit"), current || !is_synced, 0.25);
        let hover = hover_of(ui, &response);

        if current {
            let glow_rect = rect.expand2(vec2(10.0, 2.0));
            ui.painter().rect_filled(glow_rect, 10.0, pal_color.gamma_multiply(0.14 * lit));
            ui.painter().rect_stroke(glow_rect, 10.0, Stroke::new(1.2_f32, pal_color.gamma_multiply(0.40 * lit)), StrokeKind::Inside);
        } else if hover > 0.05 {
            ui.painter().rect_filled(rect.expand2(vec2(6.0, 0.0)), 8.0, veil(0.04 * hover));
        }

        let text_x = if align_center {
            (rect.center().x - galley.size().x * 0.5).max(rect.left() + 10.0)
        } else if current {
            rect.left() + 22.0
        } else {
            rect.left() + 8.0
        };

        if current {
            let dot_x = if align_center { (text_x - 12.0).max(rect.left() + 4.0) } else { rect.left() + 8.0 };
            ui.painter().circle_filled(pos2(dot_x, rect.center().y), 4.0, pal_color);
        }

        let ink = if current {
            Color32::WHITE
        } else {
            TEXT.gamma_multiply(dim_unsung + (1.0 - dim_unsung) * 0.6 * hover)
        };
        ui.painter().galley_with_override_text_color(pos2(text_x, rect.min.y + 7.0), galley, ink);
        if lyrics.synced {
            point_at(&response);
            describe(&response, "Click to seek to this line");
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

    let filter_id = Id::new("queue-filter-text");
    let mut filter_text = ui.data(|d| d.get_temp::<String>(filter_id)).unwrap_or_default();

    let search_w = ui.available_width();
    let (search_row, _) = ui.allocate_exact_size(vec2(search_w, 26.0), Sense::hover());
    ui.painter().rect_filled(search_row, 13.0, veil(0.035));
    ui.painter().rect_stroke(search_row, 13.0, Stroke::new(1.0_f32, veil(0.07)), StrokeKind::Inside);
    ui.painter().text(pos2(search_row.left() + 10.0, search_row.center().y), Align2::LEFT_CENTER, icon::MAGNIFYING_GLASS, glyph(12.0), MUTED);

    let edit_rect = Rect::from_min_max(pos2(search_row.left() + 26.0, search_row.top() + 2.0), pos2(search_row.right() - 24.0, search_row.bottom() - 2.0));
    let mut edit_ui = ui.new_child(UiBuilder::new().max_rect(edit_rect));
    let mut new_text = filter_text.clone();
    let edit_resp = edit_ui.add(egui::TextEdit::singleline(&mut new_text).hint_text("").frame(false));
    if edit_resp.changed() {
        filter_text = new_text.clone();
        ui.data_mut(|d| d.insert_temp(filter_id, new_text));
    }
    if !filter_text.is_empty() {
        let clear_rect = Rect::from_center_size(pos2(search_row.right() - 12.0, search_row.center().y), vec2(16.0, 16.0));
        let clear_resp = ui.interact(clear_rect, Id::new("clear-queue-filter"), Sense::click());
        ui.painter().text(clear_rect.center(), Align2::CENTER_CENTER, icon::X, glyph(11.0), MUTED);
        if clear_resp.clicked() {
            ui.data_mut(|d| d.insert_temp(filter_id, String::new()));
            filter_text.clear();
        }
    }
    ui.add_space(6.0);

    if !filter_text.is_empty() {
        let query_lower = filter_text.to_lowercase();
        let matching: Vec<(usize, &Item)> = (upcoming..queue.len())
            .filter_map(|idx| {
                let it = &queue[idx];
                if it.title.to_lowercase().contains(&query_lower) || it.subtitle.to_lowercase().contains(&query_lower) {
                    Some((idx, it))
                } else {
                    None
                }
            })
            .collect();

        if matching.is_empty() {
            label(ui, pos2(ui.cursor().left() + 10.0, ui.cursor().top() + 12.0), Align2::LEFT_CENTER, "No upcoming tracks match filter", sans(12.5), MUTED, search_w);
            ui.add_space(32.0);
            return true;
        }

        for (actual_idx, item) in matching {
            let (slot, response) = ui.allocate_exact_size(vec2(ui.available_width(), QUEUE_ROW), Sense::click());
            let hover = hover_of(ui, &response);
            queue_row(ui, app, item, slot, hover);
            if response.clicked() {
                out.push(Action::Jump(actual_idx));
            }
        }
        return true;
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

    if let Some(pos) = pointer {
        let clip = ui.clip_rect();
        let edge_margin = 55.0;
        let mut scroll_dy = 0.0;
        if pos.y < clip.top() + edge_margin {
            let dist = (clip.top() + edge_margin - pos.y).clamp(0.0, edge_margin);
            scroll_dy = (dist / edge_margin).powf(1.4) * 16.0;
        } else if pos.y > clip.bottom() - edge_margin {
            let dist = (pos.y - (clip.bottom() - edge_margin)).clamp(0.0, edge_margin);
            scroll_dy = -((dist / edge_margin).powf(1.4) * 16.0);
        }
        if scroll_dy.abs() > 0.001 {
            ui.scroll_with_delta(vec2(0.0, scroll_dy));
            ui.ctx().request_repaint();
        }
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
        BarLayout::FloatingPill | BarLayout::MiniCapsule => (bar, bar.height() * 0.5),
    };
    let mid = deck.center().y + 2.0;
    let item = app.now_playing();
    let p_dur = app.player.duration_ms();
    let duration = if p_dur > 0 {
        p_dur
    } else {
        item.and_then(|it| parse_duration_str_to_ms(&it.duration)).unwrap_or(0)
    };
    let cur_pos = if app.playing() {
        app.player.position_ms()
    } else {
        app.resume_at.unwrap_or_else(|| app.player.position_ms())
    }.min(duration);
    if app.playing() {
        ui.ctx().request_repaint_after(Duration::from_millis(50));
    }
    let cover = item.map(|item| app.saved.covers.of(item));
    let tint = cover
        .and_then(|cover| app.art.get(ui.ctx(), cover, BAR_ART))
        .map(|picture| picture.tint);
    let pal_color = app.saved.theme_palette.color();

    // Floating bar drop shadow so it visibly pops above page content
    if app.saved.bar_layout != BarLayout::Docked {
        ui.painter().rect_filled(deck.translate(vec2(0.0, 4.0)), rounding, Color32::from_black_alpha(85));
        ui.painter().rect_filled(deck.translate(vec2(0.0, 10.0)).expand(2.0), rounding + 2.0, Color32::from_black_alpha(45));
        ui.painter().rect_filled(deck.translate(vec2(0.0, 18.0)).expand(5.0), rounding + 4.0, Color32::from_black_alpha(20));
    }

    // Deck Background: Glassmorphism / Opacity / Grain
    let opacity = if app.saved.bar_glass {
        app.saved.bar_opacity.clamp(0.15, 1.0)
    } else {
        1.0
    };
    let bg_color = if app.saved.bar_layout != BarLayout::Docked {
        if app.saved.theme_mode == ThemeMode::Light {
            Color32::from_rgba_unmultiplied(245, 245, 248, (opacity * 255.0) as u8)
        } else {
            Color32::from_rgba_unmultiplied(20, 21, 26, (opacity * 255.0) as u8)
        }
    } else {
        Color32::from_rgba_unmultiplied(PANEL.r(), PANEL.g(), PANEL.b(), (opacity * 255.0) as u8)
    };
    ui.painter().rect_filled(deck, rounding, bg_color);
    // Light accents (e.g. Monochrome Glass) would wash the whole deck out and
    // hide the white transport controls, so dim light glow tints back down.
    let active_tint = tint
        .or_else(|| Some(pal_color))
        .map(|c| if is_light_color(c) { c.gamma_multiply(0.45) } else { c });
    let glow_left = mood(ui, "bar-glow-left", active_tint, PANEL, 0.42 * opacity);
    let glow_mid = mood(ui, "bar-glow-mid", active_tint, PANEL, 0.14 * opacity);
    let glow_right = mood(ui, "bar-glow-right", active_tint, PANEL, 0.03 * opacity);
    if rounding > 0.0 {
        paint_rounded_gradient(ui.painter(), deck, rounding, glow_left, glow_right);
    } else {
        horizontal_wash_3(ui.painter(), deck, glow_left, glow_mid, glow_right);
    }
    if app.saved.bar_layout == BarLayout::FloatingPill || app.saved.bar_layout == BarLayout::MiniCapsule {
        ui.painter().rect_stroke(
            deck,
            rounding,
            Stroke::new(1.0_f32, Color32::from_white_alpha(36)),
            StrokeKind::Inside,
        );
    }
    if app.saved.bar_grain {
        paint_glass_grain(ui.painter(), deck, rounding, app.saved.bar_grain_strength);
    }

    if app.saved.bar_layout == BarLayout::MiniCapsule {
        let pal_color = app.saved.theme_palette.color();
        let art_center = pos2(deck.left() + 28.0, mid);
        let angle = if app.playing() && !app.saved.performance_mode {
            let minimized = ui.ctx().input(|i| i.viewport().minimized == Some(true));
            if !minimized {
                ui.ctx().request_repaint_after(Duration::from_millis(33));
            }
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
        let pos_ms = cur_pos;
        let frac = if duration > 0 { (pos_ms as f32 / duration as f32).clamp(0.0, 1.0) } else { 0.0 };
        ui.painter().rect_filled(scrub_track, 1.5, veil(0.14));
        let fill_rect = Rect::from_min_size(scrub_track.min, vec2(scrub_track.width() * frac, 3.0));
        ui.painter().rect_filled(fill_rect, 1.5, pal_color);
        let thumb_x = scrub_track.left() + scrub_track.width() * frac;
        ui.painter().circle_filled(pos2(thumb_x, scrub_track.center().y), 4.0, pal_color);
        ui.painter().circle_filled(pos2(thumb_x, scrub_track.center().y), 1.8, Color32::WHITE);

        return Scrub {
            bar: scrub_track,
            played: frac,
            buffered: if item.is_some() { app.player.buffered() } else { 0.0 },
            hover: scrub_resp.hovered() as i32 as f32,
            aim: None,
        };
    }

    let backdrop = ui.interact(deck, Id::new("bar-backdrop"), Sense::click());
    if backdrop.clicked() && item.is_some() {
        out.push(Action::Expand(!app.expanded));
    }

    let (track_rect, zone) = if app.saved.bar_layout == BarLayout::FloatingPill {
        let margin = rounding + 8.0;
        let track = Rect::from_min_size(pos2(deck.left() + margin, deck.bottom() - 4.5), vec2(deck.width() - 2.0 * margin, 3.0));
        let zone = track.expand2(vec2(0.0, 7.0));
        (track, zone)
    } else {
        let track = Rect::from_min_size(pos2(bar.left(), bar.top()), vec2(bar.width(), 3.0));
        let zone = Rect::from_min_size(pos2(bar.left(), bar.top() - 2.0), vec2(bar.width(), 12.0));
        (track, zone)
    };
    let sense = if duration > 0 { Sense::click_and_drag() } else { Sense::hover() };
    let scrubber = ui.interact(zone, Id::new("scrubber"), sense);
    if duration > 0 {
        point_at(&scrubber);
    }
    let fraction_at = |pointer: Pos2| ((pointer.x - track_rect.left()) / track_rect.width()).clamp(0.0, 1.0);
    let pointed = scrubber.interact_pointer_pos().map(fraction_at);
    let aimed = pointed.or(scrubber.hover_pos().map(fraction_at)).filter(|_| duration > 0);
    let mut position = cur_pos;
    if let (true, Some(fraction)) = (scrubber.dragged() || scrubber.is_pointer_button_down_on(), pointed) {
        position = (fraction * duration as f32) as u64;
    }
    if let (true, Some(fraction)) = (scrubber.clicked() || scrubber.drag_stopped(), pointed) {
        out.push(Action::Seek((fraction * duration as f32) as u64));
    }
    let played_frac = if duration > 0 { (position as f32 / duration as f32).clamp(0.0, 1.0) } else { 0.0 };
    let scrub = Scrub {
        bar: track_rect,
        played: played_frac,
        buffered: if item.is_some() { app.player.buffered() } else { 0.0 },
        hover: ui
            .ctx()
            .animate_bool_with_time(scrubber.id, scrubber.hovered() || scrubber.dragged(), HOVER_TIME),
        aim: aimed.map(|fraction| (fraction, (fraction * duration as f32) as u64)),
    };

    let scrub_h = 3.0 + 1.0 * scrub.hover;
    let track = Rect::from_min_size(pos2(track_rect.left(), track_rect.center().y - scrub_h * 0.5), vec2(track_rect.width(), scrub_h));
    let span = |fraction: f32| Rect::from_min_size(track.min, vec2(track.width() * fraction, scrub_h));
    let track_bg = if app.saved.bar_layout == BarLayout::Docked { veil(0.12) } else { veil(0.18) };
    ui.painter().rect_filled(track, 1.5, track_bg);
    ui.painter().rect_filled(span(scrub.buffered), 1.5, veil(0.28));
    ui.painter().rect_filled(span(scrub.played), 1.5, pal_color);
    let thumb_x = track.left() + track.width() * scrub.played;
    let thumb_r = 4.0 + 1.5 * scrub.hover;
    ui.painter().circle_filled(pos2(thumb_x, track.center().y), thumb_r, pal_color);
    ui.painter().circle_filled(pos2(thumb_x, track.center().y), thumb_r * 0.45, Color32::WHITE);

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
                (solid(17.0), pal_color, "Remove from Liked songs")
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
    // Thin dark halo keeps the white play button cleanly visible over glass
    // decks and light artwork bleeding through (light accent themes).
    ui.painter().circle_filled(center, 21.5 + lift - pressed, Color32::from_black_alpha(70));
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
    let pal_color = app.saved.theme_palette.color();
    let shuffle = if app.saved.shuffle { pal_color } else { MUTED };
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
        Repeat::All => (icon::REPEAT, pal_color, "Repeat one"),
        Repeat::One => (icon::REPEAT_ONCE, pal_color, "Stop repeating"),
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
    ui.painter().rect_filled(level, 2.0, TEXT.lerp_to_gamma(pal_color, grip));
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
    let queue = if app.saved.queue_open { pal_color } else { MUTED };
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
        if singing { pal_color } else { MUTED },
        "Lyrics",
    )
    .clicked()
    {
        out.push(Action::ShowLyrics(true));
        out.push(Action::Expand(!singing));
    }
    right -= 38.0;
    let mini_color = if app.saved.mini_mode { pal_color } else { MUTED };
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
    let _ = right;
    scrub
}

#[allow(dead_code)]
fn paint_scrubber(ctx: &egui::Context, scrub: &Scrub, accent_color: Color32) {
    let painter = ctx.layer_painter(egui::LayerId::background());
    let height = 2.0 + 2.0 * scrub.hover;
    let span = |fraction: f32| Rect::from_min_size(scrub.bar.min - vec2(0.0, height / 2.0), vec2(scrub.bar.width() * fraction, height));
    painter.rect_filled(span(1.0), 0.0, rgb(0x26272C));
    painter.rect_filled(span(scrub.buffered), 0.0, rgb(0x45474E));
    let played = span(scrub.played);
    // Dark under-edge so a light accent (Monochrome Glass) fill stays readable
    // over glass decks and light artwork bleed-through.
    if is_light_color(accent_color) && played.width() > 0.5 {
        painter.rect_filled(played.expand(1.0), 0.0, Color32::from_black_alpha(80));
    }
    painter.rect_filled(played, 0.0, accent_color);
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
            let accent = ui.ctx().data(|d| d.get_temp(Id::new("app-theme-accent"))).unwrap_or(ACCENT);
            ui.painter().rect_filled(slot, 15.0, PANEL.lerp_to_gamma(accent, hover));
            let ink = TEXT.lerp_to_gamma(ON_ACCENT, hover);
            ui.painter()
                .galley_with_override_text_color(slot.center() - name.size() / 2.0, name, ink);
            describe(&response, hint);
            if response.clicked() {
                out.push(action.clone());
            }
        });
}

pub fn parse_duration_str_to_ms(s: &str) -> Option<u64> {
    let parts: Vec<&str> = s.trim().split(':').collect();
    if parts.is_empty() {
        return None;
    }
    let mut total_secs: u64 = 0;
    for part in parts {
        let n: u64 = part.trim().parse().ok()?;
        total_secs = total_secs.checked_mul(60)?.checked_add(n)?;
    }
    Some(total_secs * 1000)
}

pub fn draw_mini(app: &mut App, ctx: &egui::Context) -> Vec<Action> {
    let mut out = Vec::new();
    let item = app.now_playing();
    let active = item.is_some();
    let p_dur = app.player.duration_ms();
    let duration = if p_dur > 0 {
        p_dur
    } else {
        item.and_then(|it| parse_duration_str_to_ms(&it.duration)).unwrap_or(0)
    };
    let pal_color = app.saved.mini_palette.color();
    let is_locked = app.saved.mini_locked;

    if app.playing() {
        let minimized = ctx.input(|i| i.viewport().minimized == Some(true));
        if !minimized {
            ctx.request_repaint_after(Duration::from_millis(40));
        }
    }

    ctx.style_mut(|s| {
        s.visuals.panel_fill = Color32::TRANSPARENT;
        s.visuals.window_fill = Color32::TRANSPARENT;
        s.visuals.window_stroke = Stroke::NONE;
    });

    egui::CentralPanel::default()
        .frame(Frame::NONE.fill(Color32::TRANSPARENT).inner_margin(Margin::ZERO))
        .show(ctx, |ui| {
            let (mut w, h) = app.saved.mini_variant.size(app.mini_queue_open);
            let mut is_retracted = false;

            if app.saved.mini_variant == MiniVariant::CapsulePill && app.saved.mini_capsule_retractable {
                let pointer_pos = ctx.input(|i| i.pointer.hover_pos());
                let last_hover_id = Id::new("mini-capsule-last-hover-time");
                let mut last_hover_time: f64 = ui.data(|d| d.get_temp(last_hover_id)).unwrap_or(0.0);
                let now = ctx.input(|i| i.time);

                let is_hovered = pointer_pos.map_or(false, |pos| ui.max_rect().expand(8.0).contains(pos));
                if is_hovered {
                    last_hover_time = now;
                    ui.data_mut(|d| d.insert_temp(last_hover_id, last_hover_time));
                }

                let elapsed = now - last_hover_time;
                let is_expanded = is_hovered || (elapsed < 5.0 && last_hover_time > 0.0);

                if !is_hovered && elapsed < 5.0 && last_hover_time > 0.0 {
                    let wait = (5.0 - elapsed).max(0.05);
                    ctx.request_repaint_after(Duration::from_secs_f64(wait));
                }

                if !is_expanded {
                    is_retracted = true;
                }

                let target_w = if is_retracted { 68.0 } else { 330.0 };
                let anim_w = ctx.animate_value_with_time(Id::new("mini-capsule-anim-w"), target_w, 0.45);
                w = anim_w;

                let prev_w: f32 = ui.data(|d| d.get_temp(Id::new("mini-prev-capsule-w"))).unwrap_or(330.0);
                if (prev_w - anim_w).abs() > 0.5 {
                    ui.data_mut(|d| d.insert_temp(Id::new("mini-prev-capsule-w"), anim_w));
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(vec2(anim_w, 68.0)));
                }
            }

            let card_rect = Rect::from_min_size(ui.max_rect().min, vec2(w, h));

            if app.saved.mini_variant != MiniVariant::CapsulePill {
                let rounding = match app.saved.mini_variant {
                    MiniVariant::ClassicHud => 16.0,
                    _ => 18.0,
                };
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
                    paint_glass_grain(ui.painter(), card_rect, rounding, app.saved.mini_grain_strength);
                }
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
                    let scrub_y = card_rect.top() + 96.0;
                    let scrub_rect = Rect::from_min_max(
                        pos2(card_rect.left() + 14.0, scrub_y),
                        pos2(card_rect.right() - 36.0, scrub_y + 3.5),
                    );
                    let scrub_resp = ui.interact(scrub_rect.expand2(vec2(0.0, 8.0)), Id::new("mini-bottom-scrubber"), Sense::click_and_drag());
                    let fraction_at = |pos: Pos2| ((pos.x - scrub_rect.left()) / scrub_rect.width()).clamp(0.0, 1.0);
                    let pointed = scrub_resp.interact_pointer_pos().map(fraction_at);

                    let mut pos_ms = if duration > 0 { app.player.position_ms().min(duration) } else { app.player.position_ms() };
                    if let (true, Some(frac)) = (scrub_resp.dragged() || scrub_resp.is_pointer_button_down_on(), pointed) {
                        if duration > 0 {
                            pos_ms = (frac * duration as f32) as u64;
                        }
                    }
                    if let (true, Some(frac)) = (scrub_resp.clicked() || scrub_resp.drag_stopped(), pointed) {
                        if duration > 0 {
                            out.push(Action::Seek((frac * duration as f32) as u64));
                        }
                    }
                    let frac = if duration > 0 { (pos_ms as f32 / duration as f32).clamp(0.0, 1.0) } else { 0.0 };
                    ui.painter().rect_filled(scrub_rect, 1.75, veil(0.14));
                    let fill_rect = Rect::from_min_size(scrub_rect.min, vec2(scrub_rect.width() * frac, 3.5));
                    ui.painter().rect_filled(fill_rect, 1.75, pal_color);
                    let thumb_x = scrub_rect.left() + scrub_rect.width() * frac;
                    ui.painter().circle_filled(pos2(thumb_x, scrub_rect.center().y), 4.5, pal_color);
                    ui.painter().circle_filled(pos2(thumb_x, scrub_rect.center().y), 2.0, Color32::WHITE);

                    if app.saved.mini_show_clock && duration > 0 {
                        let dur_text = format!("{} / {}", clock(pos_ms), clock(duration));
                        ui.painter().text(pos2(card_rect.right() - 40.0, card_rect.top() + 86.0), Align2::RIGHT_CENTER, dur_text, sans(10.0), FAINT);
                    }
                }
                MiniVariant::CapsulePill => {
                    let disc_center = pos2(card_rect.left() + 34.0, card_rect.top() + 34.0);
                    let bg_base = app.saved.mini_bg.color(pal_color);
                    let alpha = (app.saved.mini_opacity.clamp(0.05, 1.0) * 255.0) as u8;
                    let bg_fill = Color32::from_rgba_unmultiplied(bg_base.r(), bg_base.g(), bg_base.b(), alpha);
                    let pal_stroke = Color32::from_rgba_unmultiplied(pal_color.r(), pal_color.g(), pal_color.b(), 180);

                    // 1. Right Pill Body (height 48px, rounded 24px) extending from circle center to anim_w
                    if w > 74.0 {
                        let pill_rect = Rect::from_min_max(
                            pos2(disc_center.x, disc_center.y - 24.0),
                            pos2(card_rect.left() + w, disc_center.y + 24.0),
                        );
                        let rounding = eframe::egui::CornerRadius { nw: 0, ne: 24, se: 24, sw: 0 };
                        ui.painter().rect_filled(pill_rect, rounding, bg_fill);
                        ui.painter().rect_stroke(pill_rect, rounding, Stroke::new(1.0_f32, pal_stroke), StrokeKind::Inside);
                        if app.saved.mini_grain {
                            paint_glass_grain(ui.painter(), pill_rect, 24.0, app.saved.mini_grain_strength);
                        }
                    }

                    // 2. Vinyl circular disc on left (diameter 66px, radius 33px)
                    ui.painter().circle_filled(disc_center, 33.0, bg_fill);
                    ui.painter().circle_stroke(disc_center, 33.0, Stroke::new(1.0_f32, pal_stroke));

                    // 3. Spinning vinyl album art inside disc (radius 24px) - always spins while playing
                    let angle = if app.playing() {
                        let minimized = ctx.input(|i| i.viewport().minimized == Some(true));
                        if !minimized {
                            ctx.request_repaint_after(Duration::from_millis(33));
                        }
                        (ctx.input(|i| i.time) * 1.5) as f32
                    } else {
                        0.0
                    };
                    let cover = item.map_or("", |it| app.saved.covers.of(it));
                    rotating_circular_artwork(ui, &app.art, cover, disc_center, 24.0, angle, pal_color);

                    if w > 160.0 {
                        // Title & Artist beside the disc
                        let title_x = card_rect.left() + 74.0;
                        let title_str = item.map_or("AURI", |it| it.title.as_str());
                        let sub_str = item.map_or("Ready", |it| it.subtitle.as_str());
                        label(ui, pos2(title_x, disc_center.y - 8.0), Align2::LEFT_CENTER, title_str, medium(12.0), TEXT, 70.0);
                        label(ui, pos2(title_x, disc_center.y + 8.0), Align2::LEFT_CENTER, sub_str, sans(10.0), MUTED, 70.0);

                        // Centered 3 Transport Buttons
                        let play_center = pos2(card_rect.left() + 185.0, disc_center.y);
                        let play_rect = Rect::from_center_size(play_center, vec2(28.0, 28.0));
                        let play_resp = ui.interact(play_rect, Id::new("pill-play"), Sense::click());
                        ui.painter().circle_filled(play_center, 13.0, if active { TEXT } else { veil(0.14) });
                        let symbol = if app.playing() { icon::PAUSE } else { icon::PLAY };
                        ui.painter().text(play_center, Align2::CENTER_CENTER, symbol, solid(14.0), if active { PANEL } else { FAINT });
                        if play_resp.clicked() {
                            out.push(Action::Toggle);
                        }

                        let prev_rect = Rect::from_center_size(pos2(play_center.x - 26.0, play_center.y), vec2(22.0, 22.0));
                        if icon_button(ui, prev_rect, Id::new("pill-prev"), icon::SKIP_BACK, solid(13.0), if active { TEXT } else { FAINT }, "Previous").clicked() {
                            out.push(Action::Previous);
                        }

                        let next_rect = Rect::from_center_size(pos2(play_center.x + 26.0, play_center.y), vec2(22.0, 22.0));
                        if icon_button(ui, next_rect, Id::new("pill-next"), icon::SKIP_FORWARD, solid(13.0), if active { TEXT } else { FAINT }, "Next").clicked() {
                            out.push(Action::Next);
                        }

                        // Volume Button
                        let vol_btn_rect = Rect::from_center_size(pos2(play_center.x + 52.0, play_center.y), vec2(22.0, 22.0));
                        let vol_resp = icon_button(
                            ui,
                            vol_btn_rect,
                            Id::new("pill-vol-btn"),
                            icon::SPEAKER_HIGH,
                            glyph(14.0),
                            if app.saved.volume > 0.0 { pal_color } else { MUTED },
                            &format!("Volume: {:.0}%", app.saved.volume * 100.0),
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

                        // Lock Pin & Restore
                        let pin_rect = Rect::from_center_size(pos2(card_rect.left() + w - 40.0, disc_center.y), vec2(22.0, 22.0));
                        let pin_tone = if is_locked { pal_color } else { MUTED };
                        let pin_hint = if is_locked { "Mini Mode Position Locked" } else { "Lock Mini Mode Position" };
                        if icon_button(ui, pin_rect, Id::new("pill-pin"), icon::PUSH_PIN, glyph(13.0), pin_tone, pin_hint).clicked() {
                            out.push(Action::ToggleMiniLock);
                        }

                        let restore_rect = Rect::from_center_size(pos2(card_rect.left() + w - 18.0, disc_center.y), vec2(22.0, 22.0));
                        if icon_button(ui, restore_rect, Id::new("pill-restore"), icon::ARROWS_OUT_SIMPLE, glyph(13.0), MUTED, "Restore Normal Mode").clicked() {
                            out.push(Action::ToggleMiniMode);
                        }
                    }
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
                    let scrub_resp = ui.interact(scrub_rect.expand2(vec2(0.0, 8.0)), Id::new("studio-scrub"), Sense::click_and_drag());
                    let fraction_at = |pos: Pos2| ((pos.x - scrub_rect.left()) / scrub_rect.width()).clamp(0.0, 1.0);
                    let pointed = scrub_resp.interact_pointer_pos().map(fraction_at);

                    let mut pos_ms = if duration > 0 { app.player.position_ms().min(duration) } else { app.player.position_ms() };
                    if let (true, Some(frac)) = (scrub_resp.dragged() || scrub_resp.is_pointer_button_down_on(), pointed) {
                        if duration > 0 {
                            pos_ms = (frac * duration as f32) as u64;
                        }
                    }
                    if let (true, Some(frac)) = (scrub_resp.clicked() || scrub_resp.drag_stopped(), pointed) {
                        if duration > 0 {
                            out.push(Action::Seek((frac * duration as f32) as u64));
                        }
                    }
                    let frac = if duration > 0 { (pos_ms as f32 / duration as f32).clamp(0.0, 1.0) } else { 0.0 };
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

            // Expandable Mini-Queue Drawer (for ClassicHud and StudioCard)
            if app.mini_queue_open {
                let q_top = match app.saved.mini_variant {
                    MiniVariant::ClassicHud => Some(card_rect.top() + 104.0),
                    MiniVariant::StudioCard => Some(card_rect.top() + 388.0),
                    _ => None,
                };
                if let Some(top_y) = q_top {
                    ui.painter().hline(card_rect.x_range().shrink(12.0), top_y, Stroke::new(1.0_f32, veil(0.08)));
                    let q_content_top = top_y + 4.0;
                    ui.painter().text(pos2(card_rect.left() + 16.0, q_content_top + 10.0), Align2::LEFT_CENTER, "UPCOMING QUEUE", bold(10.0), pal_color);
                    let start = app.saved.current.map_or(0, |c| c + 1);
                    let upcoming: Vec<(usize, &Item)> = app.saved.queue.iter().enumerate().skip(start).take(3).collect();
                    if upcoming.is_empty() {
                        label(ui, pos2(card_rect.center().x, q_content_top + 45.0), Align2::CENTER_CENTER, "No more songs in queue", sans(11.5), FAINT, card_rect.width() - 40.0);
                    } else {
                        for (rel_i, &(idx, song)) in upcoming.iter().enumerate() {
                            let row_y = q_content_top + 22.0 + rel_i as f32 * 40.0;
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
            }
        });

    out
}

fn format_ms(ms: u64) -> String {
    let total_secs = ms / 1000;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{:02}:{:02}", mins, secs)
}

#[cfg(windows)]
pub fn current_local_time_str() -> (String, String) {
    use windows_sys::Win32::Foundation::SYSTEMTIME;
    use windows_sys::Win32::System::SystemInformation::GetLocalTime;
    unsafe {
        let mut st = std::mem::MaybeUninit::<SYSTEMTIME>::zeroed();
        GetLocalTime(st.as_mut_ptr());
        let st = st.assume_init();
        let time_str = format!("{:02}:{:02}", st.wHour, st.wMinute);
        let date_str = format!("{:02}/{:02}/{}", st.wDay, st.wMonth, st.wYear);
        (time_str, date_str)
    }
}

#[cfg(not(windows))]
pub fn current_local_time_str() -> (String, String) {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::SYSTEMTIME;
        use windows_sys::Win32::System::SystemInformation::GetLocalTime;
        unsafe {
            let mut st: SYSTEMTIME = std::mem::zeroed();
            GetLocalTime(&mut st);
            let time = format!("{:02}:{:02}", st.wHour, st.wMinute);
            let day_name = match st.wDayOfWeek {
                0 => "Sunday",
                1 => "Monday",
                2 => "Tuesday",
                3 => "Wednesday",
                4 => "Thursday",
                5 => "Friday",
                _ => "Saturday",
            };
            let month_name = match st.wMonth {
                1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
                5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
                9 => "Sep", 10 => "Oct", 11 => "Nov", _ => "Dec",
            };
            let date = format!("{}, {} {}", day_name, month_name, st.wDay);
            (time, date)
        }
    }
    #[cfg(not(windows))]
    ("16:20".to_string(), "Today".to_string())
}

#[cfg(windows)]
fn query_live_telemetry() -> (f32, u32) {
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
    #[repr(C)]
    struct MEMORYSTATUSEX {
        dw_length: u32,
        dw_memory_load: u32,
        ull_total_phys: u64,
        ull_avail_phys: u64,
        ull_total_page_file: u64,
        ull_avail_page_file: u64,
        ull_total_virtual: u64,
        ull_avail_virtual: u64,
        ull_avail_extended_virtual: u64,
    }
    #[link(name = "psapi")]
    unsafe extern "system" {
        fn GetProcessMemoryInfo(h_process: isize, counters: *mut PROCESS_MEMORY_COUNTERS, cb: u32) -> i32;
        fn GetCurrentProcess() -> isize;
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GlobalMemoryStatusEx(stat: *mut MEMORYSTATUSEX) -> i32;
    }
    unsafe {
        let mut counters: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
        counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
        let proc_mb = if GetProcessMemoryInfo(GetCurrentProcess(), &mut counters, counters.cb) != 0 {
            counters.working_set_size as f32 / (1024.0 * 1024.0)
        } else {
            18.5
        };
        let mut mem_stat: MEMORYSTATUSEX = std::mem::zeroed();
        mem_stat.dw_length = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        let sys_pct = if GlobalMemoryStatusEx(&mut mem_stat) != 0 {
            mem_stat.dw_memory_load
        } else {
            40
        };
        (proc_mb, sys_pct)
    }
}

#[cfg(not(windows))]
fn query_live_telemetry() -> (f32, u32) {
    (18.5, 40)
}

pub fn draw_game_overlay(app: &mut App, ctx: &egui::Context) -> Vec<Action> {
    let mut actions = Vec::new();
    let pal_color = app.saved.theme_palette.color();
    let (ram_mb, sys_ram_pct) = query_live_telemetry();
    let fps = (1.0 / ctx.input(|i| i.predicted_dt).max(0.001)).round() as u32;

    egui::CentralPanel::default()
        .frame(Frame::NONE)
        .show(ctx, |ui| {
            let rect = ui.max_rect();

            // 1. Translucent backdrop (customizable opacity)
            let alpha = (app.saved.game_overlay_opacity.clamp(0.15, 0.95) * 255.0) as u8;
            ui.painter().rect_filled(rect, 0.0, Color32::from_black_alpha(alpha));

            // Background click to dismiss overlay
            let bg_resp = ui.interact(rect, Id::new("game-overlay-backdrop-click"), Sense::click());
            if bg_resp.clicked() {
                actions.push(Action::ToggleGameOverlay);
            }

            // 2. Top Bar: Digital Clock & Session Info (Image 5 style)
            let (clock_str, date_str) = current_local_time_str();
            label(ui, pos2(rect.left() + 60.0, rect.top() + 42.0), Align2::LEFT_CENTER, &clock_str, bold(44.0), Color32::WHITE, 200.0);

            let session_str = if let Some(st) = app.game_mode_start_time {
                let el = st.elapsed().as_secs();
                format!("SESSION {:02}:{:02}", el / 60, el % 60)
            } else {
                "SESSION 00:00".to_string()
            };
            label(
                ui,
                pos2(rect.left() + 62.0, rect.top() + 74.0),
                Align2::LEFT_CENTER,
                &format!("{} · {}", session_str, date_str),
                bold(11.5),
                pal_color,
                320.0,
            );

            // Top Right Live Hardware Telemetry & Close button
            let top_right_pill = Rect::from_min_size(pos2(rect.right() - 390.0, rect.top() + 34.0), vec2(320.0, 28.0));
            ui.painter().rect_filled(top_right_pill, 14.0, veil(0.14));
            ui.painter().rect_stroke(top_right_pill, 14.0, Stroke::new(1.0_f32, veil(0.10)), StrokeKind::Inside);
            ui.painter().circle_filled(pos2(top_right_pill.left() + 14.0, top_right_pill.center().y), 3.5, Color32::from_rgb(34, 197, 94));
            let tele_str = format!("AURI RAM: {:.1} MB · Host RAM: {}% · {} FPS", ram_mb, sys_ram_pct, fps);
            ui.painter().text(
                pos2(top_right_pill.left() + 24.0, top_right_pill.center().y),
                Align2::LEFT_CENTER,
                tele_str,
                bold(10.5),
                TEXT,
            );

            let top_close_rect = Rect::from_center_size(pos2(rect.right() - 45.0, rect.top() + 48.0), Vec2::splat(28.0));
            let top_close_resp = ui.interact(top_close_rect, Id::new("game-overlay-top-close"), Sense::click());
            let tc_hov = hover_of(ui, &top_close_resp);
            ui.painter().circle_filled(top_close_rect.center(), 14.0, veil(0.10 * tc_hov));
            ui.painter().text(top_close_rect.center(), Align2::CENTER_CENTER, icon::X, glyph(16.0), if tc_hov > 0.1 { TEXT } else { MUTED });
            if top_close_resp.clicked() {
                actions.push(Action::ToggleGameOverlay);
            }

            let item = app.saved.current.and_then(|c| app.saved.queue.get(c)).cloned();
            let is_playing = app.playing();

            // 3. FLOATING WIDGET: MUSIC / NOW PLAYING (Draggable)
            if app.saved.game_show_music_widget {
                let music_pos = pos2(
                    rect.left() + app.saved.game_music_pos[0].clamp(0.0, (rect.width() - 480.0).max(0.0)),
                    rect.top() + app.saved.game_music_pos[1].clamp(0.0, (rect.height() - 230.0).max(0.0)),
                );
                let music_rect = Rect::from_min_size(music_pos, vec2(460.0, 215.0));
                ui.painter().rect_filled(music_rect, 14.0, Color32::from_rgba_premultiplied(18, 20, 26, 240));
                ui.painter().rect_stroke(music_rect, 14.0, Stroke::new(1.0_f32, pal_color.gamma_multiply(0.35)), StrokeKind::Inside);

                // Draggable Header
                let m_header = Rect::from_min_size(music_pos, vec2(460.0, 32.0));
                let m_drag = ui.interact(m_header, Id::new("drag-music-widget"), Sense::click_and_drag());
                if m_drag.dragged() {
                    let delta = m_drag.drag_delta();
                    actions.push(Action::SetGameMusicPos(
                        app.saved.game_music_pos[0] + delta.x,
                        app.saved.game_music_pos[1] + delta.y,
                    ));
                }

                label(ui, pos2(music_rect.left() + 16.0, music_rect.top() + 16.0), Align2::LEFT_CENTER, "⋮⋮  NOW PLAYING", bold(11.0), MUTED, 200.0);
                let m_close = Rect::from_center_size(pos2(music_rect.right() - 18.0, music_rect.top() + 16.0), Vec2::splat(20.0));
                let mc_resp = ui.interact(m_close, Id::new("close-music-widget"), Sense::click());
                if mc_resp.clicked() {
                    actions.push(Action::ToggleGameMusicWidget);
                }
                ui.painter().text(m_close.center(), Align2::CENTER_CENTER, icon::X, glyph(13.0), MUTED);

                // Cover art & Title
                let art_rect = Rect::from_min_size(pos2(music_rect.left() + 16.0, music_rect.top() + 36.0), vec2(70.0, 70.0));
                if let Some(ref it) = item {
                    artwork(ui, &app.art, app.saved.covers.of(it), art_rect, 8.0);
                } else {
                    ui.painter().rect_filled(art_rect, 8.0, veil(0.08));
                    ui.painter().text(art_rect.center(), Align2::CENTER_CENTER, icon::MUSIC_NOTE, glyph(28.0), pal_color);
                }

                let text_x = art_rect.right() + 14.0;
                let text_w = music_rect.right() - text_x - 16.0;
                if let Some(ref it) = item {
                    label(ui, pos2(text_x, music_rect.top() + 52.0), Align2::LEFT_CENTER, &it.title, bold(15.0), TEXT, text_w);
                    label(ui, pos2(text_x, music_rect.top() + 74.0), Align2::LEFT_CENTER, &it.subtitle, sans(12.0), MUTED, text_w);
                } else {
                    label(ui, pos2(text_x, music_rect.top() + 64.0), Align2::LEFT_CENTER, "No track playing", sans(13.0), MUTED, text_w);
                }

                // Scrubber Bar
                let pos_ms = app.player.position_ms();
                let dur_ms = app.player.duration_ms();
                let pos_str = format_ms(pos_ms);
                let dur_str = format_ms(dur_ms);

                let scrob_y = music_rect.top() + 124.0;
                label(ui, pos2(music_rect.left() + 16.0, scrob_y), Align2::LEFT_CENTER, &pos_str, sans(10.5), MUTED, 40.0);
                label(ui, pos2(music_rect.right() - 16.0, scrob_y), Align2::RIGHT_CENTER, &dur_str, sans(10.5), MUTED, 40.0);

                let line_left = music_rect.left() + 58.0;
                let line_right = music_rect.right() - 58.0;
                let bar_rect = Rect::from_min_max(pos2(line_left, scrob_y - 6.0), pos2(line_right, scrob_y + 6.0));
                let bar_resp = ui.interact(bar_rect, Id::new("game-overlay-scrubber"), Sense::click_and_drag());
                let bar_hov = hover_of(ui, &bar_resp);

                let frac = if dur_ms > 0 { (pos_ms as f32 / dur_ms as f32).clamp(0.0, 1.0) } else { 0.0 };
                ui.painter().hline(line_left..=line_right, scrob_y, Stroke::new(3.0_f32, veil(0.15)));
                let fill_x = line_left + (line_right - line_left) * frac;
                ui.painter().hline(line_left..=fill_x, scrob_y, Stroke::new(3.0_f32, pal_color));
                ui.painter().circle_filled(pos2(fill_x, scrob_y), 4.5 + 1.5 * bar_hov, pal_color);

                if bar_resp.clicked() || bar_resp.dragged() {
                    if let Some(pos) = bar_resp.interact_pointer_pos() {
                        let new_frac = ((pos.x - line_left) / (line_right - line_left)).clamp(0.0, 1.0);
                        let target_ms = (new_frac * dur_ms as f32) as u64;
                        actions.push(Action::Seek(target_ms));
                    }
                }

                // Controls Row
                let ctrl_y = music_rect.top() + 168.0;
                let center_x = music_rect.center().x;

                // Play / Pause Circle
                let play_rect = Rect::from_center_size(pos2(center_x, ctrl_y), Vec2::splat(36.0));
                let play_resp = ui.interact(play_rect, Id::new("game-overlay-play"), Sense::click());
                let play_hov = hover_of(ui, &play_resp);
                ui.painter().circle_filled(play_rect.center(), 18.0, pal_color.lerp_to_gamma(Color32::WHITE, 0.15 * play_hov));
                let p_sym = if is_playing { icon::PAUSE } else { icon::PLAY };
                ui.painter().text(play_rect.center(), Align2::CENTER_CENTER, p_sym, solid(18.0), contrast_text_for(pal_color));
                if play_resp.clicked() {
                    actions.push(Action::Toggle);
                }

                // Prev & Next
                let prev_r = Rect::from_center_size(pos2(center_x - 48.0, ctrl_y), Vec2::splat(28.0));
                if icon_button(ui, prev_r, Id::new("game-overlay-prev"), icon::SKIP_BACK, solid(16.0), TEXT, "Previous track").clicked() {
                    actions.push(Action::Previous);
                }

                let next_r = Rect::from_center_size(pos2(center_x + 48.0, ctrl_y), Vec2::splat(28.0));
                if icon_button(ui, next_r, Id::new("game-overlay-next"), icon::SKIP_FORWARD, solid(16.0), TEXT, "Next track").clicked() {
                    actions.push(Action::Next);
                }

                // Shuffle & Repeat
                let shuf_r = Rect::from_center_size(pos2(center_x - 90.0, ctrl_y), Vec2::splat(24.0));
                let shuf_col = if app.saved.shuffle { pal_color } else { MUTED };
                if icon_button(ui, shuf_r, Id::new("game-overlay-shuf"), icon::SHUFFLE, glyph(15.0), shuf_col, "Shuffle").clicked() {
                    actions.push(Action::ToggleShuffle);
                }

                let rep_r = Rect::from_center_size(pos2(center_x + 90.0, ctrl_y), Vec2::splat(24.0));
                let (rep_sym, rep_col) = match app.saved.repeat {
                    Repeat::Off => (icon::REPEAT, MUTED),
                    Repeat::All => (icon::REPEAT, pal_color),
                    Repeat::One => (icon::REPEAT_ONCE, pal_color),
                };
                if icon_button(ui, rep_r, Id::new("game-overlay-rep"), rep_sym, glyph(15.0), rep_col, "Repeat").clicked() {
                    actions.push(Action::CycleRepeat);
                }
            }

            // 4. FLOATING WIDGET: AUDIO & PERFORMANCE (Draggable)
            if app.saved.game_show_audio_widget {
                let aud_pos = pos2(
                    rect.left() + app.saved.game_audio_pos[0].clamp(0.0, (rect.width() - 480.0).max(0.0)),
                    rect.top() + app.saved.game_audio_pos[1].clamp(0.0, (rect.height() - 130.0).max(0.0)),
                );
                let aud_rect = Rect::from_min_size(aud_pos, vec2(460.0, 115.0));
                ui.painter().rect_filled(aud_rect, 14.0, Color32::from_rgba_premultiplied(18, 20, 26, 240));
                ui.painter().rect_stroke(aud_rect, 14.0, Stroke::new(1.0_f32, veil(0.12)), StrokeKind::Inside);

                // Draggable Header
                let a_header = Rect::from_min_size(aud_pos, vec2(460.0, 32.0));
                let a_drag = ui.interact(a_header, Id::new("drag-audio-widget"), Sense::click_and_drag());
                if a_drag.dragged() {
                    let delta = a_drag.drag_delta();
                    actions.push(Action::SetGameAudioPos(
                        app.saved.game_audio_pos[0] + delta.x,
                        app.saved.game_audio_pos[1] + delta.y,
                    ));
                }

                label(ui, pos2(aud_rect.left() + 16.0, aud_rect.top() + 16.0), Align2::LEFT_CENTER, "⋮⋮  AUDIO & HARDWARE", bold(11.0), MUTED, 240.0);
                let a_close = Rect::from_center_size(pos2(aud_rect.right() - 18.0, aud_rect.top() + 16.0), Vec2::splat(20.0));
                if ui.interact(a_close, Id::new("close-audio-widget"), Sense::click()).clicked() {
                    actions.push(Action::ToggleGameAudioWidget);
                }
                ui.painter().text(a_close.center(), Align2::CENTER_CENTER, icon::X, glyph(13.0), MUTED);

                // Volume slider
                let vol_icon_r = Rect::from_min_size(pos2(aud_rect.left() + 16.0, aud_rect.top() + 42.0), vec2(24.0, 24.0));
                ui.painter().text(vol_icon_r.center(), Align2::CENTER_CENTER, icon::SPEAKER_HIGH, glyph(16.0), pal_color);

                let vol_left = aud_rect.left() + 46.0;
                let vol_right = aud_rect.right() - 75.0;
                let vol_bar_r = Rect::from_min_max(pos2(vol_left, aud_rect.top() + 44.0), pos2(vol_right, aud_rect.top() + 62.0));
                let vol_resp = ui.interact(vol_bar_r, Id::new("game-overlay-volume-bar"), Sense::click_and_drag());
                let vol_frac = app.saved.volume.clamp(0.0, 1.0);
                let mid_vy = aud_rect.top() + 54.0;
                ui.painter().hline(vol_left..=vol_right, mid_vy, Stroke::new(3.0_f32, veil(0.15)));
                let v_fill = vol_left + (vol_right - vol_left) * vol_frac;
                ui.painter().hline(vol_left..=v_fill, mid_vy, Stroke::new(3.0_f32, pal_color));
                ui.painter().circle_filled(pos2(v_fill, mid_vy), 4.5, pal_color);
                label(ui, pos2(aud_rect.right() - 16.0, mid_vy), Align2::RIGHT_CENTER, &format!("{:.0}%", vol_frac * 100.0), sans(11.5), TEXT, 50.0);

                if vol_resp.clicked() || vol_resp.dragged() {
                    if let Some(pos) = vol_resp.interact_pointer_pos() {
                        let new_v = ((pos.x - vol_left) / (vol_right - vol_left)).clamp(0.0, 1.0);
                        actions.push(Action::Volume(new_v));
                    }
                }

                // RAM Clean button & badge
                let clean_r = Rect::from_min_size(pos2(aud_rect.left() + 16.0, aud_rect.top() + 76.0), vec2(100.0, 24.0));
                let clean_resp = ui.interact(clean_r, Id::new("game-overlay-clean-ram"), Sense::click());
                let c_hov = hover_of(ui, &clean_resp);
                ui.painter().rect_filled(clean_r, 6.0, veil(0.08 + 0.08 * c_hov));
                ui.painter().text(clean_r.center(), Align2::CENTER_CENTER, "⚡ Clean RAM", bold(11.0), if c_hov > 0.1 { pal_color } else { TEXT });
                if clean_resp.clicked() {
                    actions.push(Action::PurgeMemory);
                }

                let badge_r = Rect::from_min_size(pos2(aud_rect.left() + 126.0, aud_rect.top() + 76.0), vec2(240.0, 24.0));
                ui.painter().rect_filled(badge_r, 6.0, pal_color.gamma_multiply(0.15));
                ui.painter().text(badge_r.center(), Align2::CENTER_CENTER, format!("AURI: {:.1} MB · Host: {}% · {} FPS", ram_mb, sys_ram_pct, fps), bold(10.0), pal_color);
            }

            // 5. FLOATING WIDGET: QUEUE (Draggable)
            if app.saved.game_show_queue_widget {
                let queue_w = 400.0_f32.min(rect.width() * 0.35);
                let q_pos = pos2(
                    rect.left() + app.saved.game_queue_pos[0].clamp(0.0, (rect.width() - queue_w - 20.0).max(0.0)),
                    rect.top() + app.saved.game_queue_pos[1].clamp(0.0, (rect.height() - 360.0).max(0.0)),
                );
                let q_rect = Rect::from_min_size(q_pos, vec2(queue_w, 345.0));
                ui.painter().rect_filled(q_rect, 14.0, Color32::from_rgba_premultiplied(18, 20, 26, 240));
                ui.painter().rect_stroke(q_rect, 14.0, Stroke::new(1.0_f32, veil(0.12)), StrokeKind::Inside);

                // Draggable Header
                let q_header = Rect::from_min_size(q_pos, vec2(queue_w, 32.0));
                let q_drag = ui.interact(q_header, Id::new("drag-queue-widget"), Sense::click_and_drag());
                if q_drag.dragged() {
                    let delta = q_drag.drag_delta();
                    actions.push(Action::SetGameQueuePos(
                        app.saved.game_queue_pos[0] + delta.x,
                        app.saved.game_queue_pos[1] + delta.y,
                    ));
                }

                let count_str = format!("⋮⋮  UPCOMING QUEUE ({} songs)", app.saved.queue.len());
                label(ui, pos2(q_rect.left() + 16.0, q_rect.top() + 16.0), Align2::LEFT_CENTER, &count_str, bold(11.0), MUTED, queue_w - 40.0);
                let q_close = Rect::from_center_size(pos2(q_rect.right() - 18.0, q_rect.top() + 16.0), Vec2::splat(20.0));
                if ui.interact(q_close, Id::new("close-queue-widget"), Sense::click()).clicked() {
                    actions.push(Action::ToggleGameQueueWidget);
                }
                ui.painter().text(q_close.center(), Align2::CENTER_CENTER, icon::X, glyph(13.0), MUTED);

                // Queue song list
                let list_rect = q_rect.shrink2(vec2(8.0, 0.0));
                let list_rect = Rect::from_min_max(pos2(list_rect.left(), q_rect.top() + 34.0), pos2(list_rect.right(), q_rect.bottom() - 10.0));
                let mut child_ui = ui.new_child(UiBuilder::new().max_rect(list_rect));
                egui::ScrollArea::vertical()
                    .id_salt("game-overlay-queue-scroll")
                    .show(&mut child_ui, |ui| {
                        for (idx, song) in app.saved.queue.iter().enumerate() {
                            let is_cur = app.saved.current == Some(idx);
                            let (row_r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 38.0), Sense::click());
                            let r_hov = hover_of(ui, &resp);
                            if is_cur {
                                ui.painter().rect_filled(row_r, 6.0, pal_color.gamma_multiply(0.20));
                                ui.painter().circle_filled(pos2(row_r.left() + 14.0, row_r.center().y), 3.5, pal_color);
                            } else {
                                ui.painter().rect_filled(row_r, 6.0, veil(0.04 * r_hov));
                                label(ui, pos2(row_r.left() + 10.0, row_r.center().y), Align2::LEFT_CENTER, &format!("{}", idx + 1), sans(10.5), MUTED, 18.0);
                            }

                            let text_w = row_r.width() - 40.0;
                            let title_col = if is_cur { pal_color } else { TEXT };
                            label(ui, pos2(row_r.left() + 32.0, row_r.center().y - 7.0), Align2::LEFT_CENTER, &song.title, medium(12.0), title_col, text_w);
                            label(ui, pos2(row_r.left() + 32.0, row_r.center().y + 8.0), Align2::LEFT_CENTER, &song.subtitle, sans(10.5), MUTED, text_w);

                            if resp.clicked() {
                                actions.push(Action::Jump(idx));
                            }
                        }
                    });
            }

            // 6. BOTTOM DOCK (Steam Overlay Style - Image 5)
            let dock_w = 700.0_f32.min(rect.width() - 40.0);
            let dock_rect = Rect::from_center_size(pos2(rect.center().x, rect.bottom() - 55.0), vec2(dock_w, 48.0));
            ui.painter().rect_filled(dock_rect, 24.0, Color32::from_rgba_premultiplied(22, 24, 32, 248));
            ui.painter().rect_stroke(dock_rect, 24.0, Stroke::new(1.0_f32, veil(0.15)), StrokeKind::Inside);

            let mut dock_x = dock_rect.left() + 18.0;

            // Widget toggles
            let mut widget_icon = |id_str: &str, sym: &str, active: bool, tooltip: &str, act: Action| {
                let r = Rect::from_center_size(pos2(dock_x, dock_rect.center().y), Vec2::splat(32.0));
                let resp = ui.interact(r, Id::new(id_str), Sense::click());
                let hov = hover_of(&ui, &resp);
                let col = if active { pal_color } else { MUTED.lerp_to_gamma(TEXT, hov) };
                if active {
                    ui.painter().circle_filled(r.center(), 14.0, pal_color.gamma_multiply(0.18));
                }
                ui.painter().text(r.center(), Align2::CENTER_CENTER, sym, glyph(17.0), col);
                describe(&resp, tooltip);
                if resp.clicked() {
                    actions.push(act);
                }
                dock_x += 38.0;
            };

            widget_icon("dock-music-btn", icon::MUSIC_NOTE, app.saved.game_show_music_widget, "Toggle Music Widget", Action::ToggleGameMusicWidget);
            widget_icon("dock-queue-btn", icon::LIST_NUMBERS, app.saved.game_show_queue_widget, "Toggle Queue Widget", Action::ToggleGameQueueWidget);
            widget_icon("dock-audio-btn", icon::SPEAKER_HIGH, app.saved.game_show_audio_widget, "Toggle Audio & Hardware Widget", Action::ToggleGameAudioWidget);

            // Divider
            dock_x += 4.0;
            ui.painter().vline(dock_x, dock_rect.shrink2(vec2(0.0, 12.0)).y_range(), Stroke::new(1.0_f32, veil(0.12)));
            dock_x += 16.0;

            // Center Big Button: CLICK HERE TO RETURN TO THE GAME
            let center_btn_w = (dock_rect.right() - dock_x - 146.0).max(190.0);
            let center_btn_r = Rect::from_min_size(pos2(dock_x, dock_rect.top() + 6.0), vec2(center_btn_w, 36.0));
            let ret_resp = ui.interact(center_btn_r, Id::new("return-to-game-dock-btn"), Sense::click());
            let ret_hov = hover_of(ui, &ret_resp);
            ui.painter().rect_filled(center_btn_r, 18.0, veil(0.08 + 0.10 * ret_hov));
            ui.painter().rect_stroke(center_btn_r, 18.0, Stroke::new(1.0_f32, pal_color.gamma_multiply(0.30 + 0.30 * ret_hov)), StrokeKind::Inside);

            ui.painter().text(
                pos2(center_btn_r.center().x, center_btn_r.center().y - 6.0),
                Align2::CENTER_CENTER,
                "CLICK HERE TO RETURN TO THE GAME",
                bold(11.0),
                if ret_hov > 0.1 { pal_color } else { TEXT },
            );
            ui.painter().text(
                pos2(center_btn_r.center().x, center_btn_r.center().y + 8.0),
                Align2::CENTER_CENTER,
                format!("Esc or {} also closes the overlay", app.saved.game_overlay_hotkey.label()),
                sans(9.5),
                MUTED,
            );

            if ret_resp.clicked() {
                actions.push(Action::ToggleGameOverlay);
            }

            dock_x += center_btn_w + 14.0;

            // Divider
            ui.painter().vline(dock_x, dock_rect.shrink2(vec2(0.0, 12.0)).y_range(), Stroke::new(1.0_f32, veil(0.12)));
            dock_x += 16.0;

            // Settings button
            let set_r = Rect::from_center_size(pos2(dock_x, dock_rect.center().y), Vec2::splat(32.0));
            let set_resp = ui.interact(set_r, Id::new("dock-settings-btn"), Sense::click());
            let s_hov = hover_of(ui, &set_resp);
            ui.painter().text(set_r.center(), Align2::CENTER_CENTER, icon::GEAR_SIX, glyph(17.0), if s_hov > 0.1 { pal_color } else { MUTED });
            describe(&set_resp, "Open Settings");
            if set_resp.clicked() {
                actions.push(Action::ToggleGameOverlay);
                actions.push(Action::Go(Route::Settings(SettingsTab::GamingMode)));
            }
            dock_x += 38.0;

            // Mini Mode button
            let mini_r = Rect::from_center_size(pos2(dock_x, dock_rect.center().y), Vec2::splat(32.0));
            let mini_resp = ui.interact(mini_r, Id::new("dock-mini-btn"), Sense::click());
            let m_hov = hover_of(ui, &mini_resp);
            ui.painter().text(mini_r.center(), Align2::CENTER_CENTER, icon::PICTURE_IN_PICTURE, glyph(17.0), if m_hov > 0.1 { pal_color } else { MUTED });
            describe(&mini_resp, "Switch to Mini Player Mode");
            if mini_resp.clicked() {
                actions.push(Action::ExitGameModeToMini);
            }
            dock_x += 38.0;

            // Exit Gaming Mode button
            let exit_r = Rect::from_center_size(pos2(dock_x, dock_rect.center().y), Vec2::splat(32.0));
            let exit_resp = ui.interact(exit_r, Id::new("dock-exit-btn"), Sense::click());
            let e_hov = hover_of(ui, &exit_resp);
            ui.painter().text(exit_r.center(), Align2::CENTER_CENTER, icon::SIGN_OUT, glyph(17.0), if e_hov > 0.1 { Color32::from_rgb(239, 68, 68) } else { MUTED });
            describe(&exit_resp, "Exit Gaming Mode & Restore Normal UI");
            if exit_resp.clicked() {
                actions.push(Action::ExitGameMode);
            }
        });

    actions
}

#[allow(dead_code)]
pub fn draw_game_hud(app: &mut App, ctx: &egui::Context) -> Vec<Action> {
    draw_game_overlay(app, ctx)
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
