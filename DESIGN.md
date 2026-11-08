# AURI Design Specifications & Architecture

> **AURI** is a minimalist, ultra-lightweight, gaming-first desktop music client designed in pure Rust. It delivers pristine 48 kHz uncompressed Opus streaming with zero browser engine overhead, an integrated studio loudness normalizer, a floating gaming HUD, and a sub-35 MB memory footprint.

---

## 1. Product Philosophy & Goals

1. **Zero Browser Engine Overhead**: No Electron, Chromium, CEF, or WebView. The application runs native machine code with an immediate-mode GPU UI (`eframe` / `egui 0.33`), maintaining instant responsiveness even while resource-intensive games are running.
2. **Sub-35 MB RAM Target**: Uses an aggressive memory management strategy (trimmed texture cache, compact audio window, and Windows `EmptyWorkingSet()` purge) to run in 25–35 MB of RAM—over 10× lighter than official web clients.
3. **Pristine Audio Fidelity**: Streams highest-quality 160 kbps VBR Opus (itag 251) at 48,000 Hz through a dedicated Rust decoder, enhanced with an ITU-R BS.1770 -7.0 LUFS loudness normalizer and soft-knee peak limiter.
4. **Gamer-Centric Multitasking**: Features an always-on-top, draggable, curved Floating Mini Mode and global keyboard shortcuts (`RegisterHotKey`) that operate without leaving full-screen games.
5. **Clean & Private**: Direct, telemetry-free connection to YouTube Music's internal API with updates strictly hosted on the user's official GitHub repository (`SuryakiranMK/AURI`).

---

## 2. Visual Design System

### 2.1 Color Palette
| Token | Hex / RGBA | Role |
| :--- | :--- | :--- |
| `BASE` | `#0A0B0E` | Main application background |
| `PANEL` | `#07080A` | Sidebar and bottom player bar surface |
| `RAISED` | `#13151D` | Cards, popups, and elevated list items |
| `ACCENT` | `#8B5CF6` | AURI Electric Violet (glows, active states, focus rings) |
| `TEXT` | `#ECEDEF` | Primary crisp high-contrast typography |
| `MUTED` | `rgba(166, 166, 166, 0.65)` | Secondary subtitles, durations, bylines |
| `FAINT` | `rgba(122, 122, 122, 0.48)` | Borders, separators, empty state messages |

### 2.2 Brand & Typography
- **Brand Banner**: Full uncropped 1024×341 (3:1 aspect ratio) graphic rendered at 240×80 px in the sidebar header with smooth alpha background falloff, eliminating rectangular seams and preserving intentional negative space.
- **Iconography**: Phosphor Regular and Fill glyph sets embedded as static vector fonts.
- **Micro-Interactions**: Linear gamma interpolation for hover states (`HOVER_TIME = 0.10s`), spring-like active switches, and dynamic search focus rings tinted with `ACCENT`.

---

## 3. Window & Interface Modes

### 3.1 Full Studio Mode (Default)
- **Window Size**: 1360 × 860 (minimum 940 × 600).
- **Sidebar (244px)**:
  - Brand Banner (clickable to return Home).
  - Navigation links: Home, Liked Songs, Recently Played (with live count badges).
  - Library Section: Pinned playlists, saved albums, artists.
  - Footer: Account avatar/status and dedicated Settings button (`icon::GEAR_SIX`).
- **Top Header**:
  - History navigation (Back `<` / Forward `>`).
  - Pill search bar with dynamic violet focus ring, instant debouncing (350ms), keyboard shortcut (`Ctrl+K` or `/`), and clear button (`icon::X`).
- **Main Stage**:
  - Dynamic content grid (Quick Picks, Related Artists, Recommendations).
  - Background ambient haze generated from album art dominant color.
- **Bottom Player Bar (84px)**:
  - Left: 56px album artwork with rounded corners, track title, artist links, and like heart.
  - Center: Transport controls (Shuffle, Previous, Play/Pause, Next, Repeat) and micro-scrubber with elapsed/total duration.
  - Right: Studio Loudness Normalizer badge, Floating Mini Mode toggle, Working Set RAM Purge button, Speaker mute toggle, and Volume slider.

### 3.2 Floating Gaming Mini Mode
- **Dimensions**: Compact 380 × 110 px floating HUD.
- **Window Level**: Always-on-top (`egui::WindowLevel::AlwaysOnTop`).
- **Border**: 16px corner radius with glass violet border (`#8B5CF6` at 55% opacity).
- **Draggability**: User can click and drag anywhere on the window background to reposition it (`ViewportCommand::StartDrag`).
- **Controls**:
  - 72×72 px album artwork.
  - Song title and artist byline.
  - Interactive scrub bar with live position seek.
  - Previous, Play/Pause, Next buttons.
  - Mini volume slider.
  - Restore button (`icon::ARROWS_OUT_SIMPLE`) to return to full window mode.
- **Performance**: Automatically triggers working set memory purge upon entry.

### 3.3 Dedicated Settings Page (`Route::Settings`)
A full central settings view accessible via the bottom gear icon:
1. **Discord Rich Presence**: Toggle status, custom presence headline ("Listening to AURI", "Listening to Artist", "Listening to Track", or custom text).
2. **Global Hotkeys**: Windows `RegisterHotKey` engine for background playback control while in games (Play/Pause, Next, Previous, Volume Up, Volume Down). Includes interactive hotkey recorder and conflict detection.
3. **Audio & DSP Engine**: Toggle -7.0 LUFS loudness normalization and inspect active soft-knee peak limiter status.
4. **Gaming & RAM Optimization**: Toggle automatic working set trim on minimize/mini-mode, plus one-click "Purge Working Set Memory Now" button.
5. **About & Updates**: Displays current release version and connects directly to GitHub for verified in-app updates.

---

## 4. Audio Engine & DSP Architecture

```
YouTube Music Stream (VisionOS Token)
  │
  ├──► Highest Quality Opus itag 251 (160 kbps VBR @ 48 kHz)
  │      (Fallback: AAC itag 140 @ 128 kbps)
  │
  ▼
Pure-Rust Opus Decoder (vendor/opus-decoder)
  │
  ▼
32-bit Floating Point PCM Audio Buffer (48,000 Hz Stereo)
  │
  ▼
AURI DSP Engine:
  ├── 1. Loudness Normalizer (Target: -7.0 LUFS)
  │      Gain = 10^(( -7.0 - loudness_db ) / 20)
  │
  └── 2. Soft-Knee Peak Limiter (Threshold: 0.95, Knee: 0.05)
         Smooth compression prevents digital clipping & distortion
  │
  ▼
Rodio Audio Output -> Windows WASAPI / DirectSound
```

---

## 5. Memory Reduction Strategy (<35 MB Footprint)

1. **Cover Art Cache (`src/art.rs`)**:
   - Truncated budget from 16 MB down to **4 MB**.
   - Background worker threads reduced from 4 down to **2**.
2. **Audio Window Buffer (`src/player.rs`)**:
   - Decoded PCM buffer trimmed from 8 MB down to **2.5 MB** (~75 seconds of audio buffer).
   - Range fetch size reduced to 512 KB.
3. **Route History & Page Cache (`src/app.rs`)**:
   - In-memory cached pages reduced from 24 down to **6**.
4. **OS Working Set Purge (`psapi.dll -> EmptyWorkingSet`)**:
   - Periodically purges inactive memory pages from physical RAM into standby pool.
   - Automatically invoked when switching to Mini Mode or minimizing the window.
