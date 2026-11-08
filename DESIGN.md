# PROJECT AURI — DESIGN SPECIFICATION & ARCHITECTURE (DESIGN.md)

> **Version**: 0.3.0  
> **Platform**: Windows 10/11 Desktop (x86_64)  
> **Core Identity**: Minimalist, Ultra-Lightweight, Gaming-First Desktop Music Player  
> **Design Language**: Apple Glassmorphic Dark UI • OLED Blacks • Studio Precision  

---

## 1. Design Philosophy & Aesthetic Pillars

AURI fuses the visual luxury of **Apple Music & macOS Sonoma glassmorphism** with the ultra-low overhead requirements of competitive PC gaming. 

```
┌────────────────────────────────────────────────────────────────────────┐
│                          AURI DESIGN TRINITY                           │
├───────────────────┬────────────────────────────┬───────────────────────┤
│ 1. Zero Impact    │ 2. Studio Fidelity         │ 3. Apple Glassmorphic │
│    < 35 MB RAM    │    Native WASAPI Exclusive │    Frosted Glass      │
│    0.0% GPU in idle│   -7.0 LUFS Normalization │    Vibrant Gradients  │
│    Zero DLL hooks │    Soft-Knee Peak Limiter  │    Subpixel Alignment │
└───────────────────┴────────────────────────────┴───────────────────────┘
```

### Reference Architecture Synergy
- **Reference 1 (Apple Music & Glassmorphism)**: Dark ambient backgrounds, precision track tables, generous corner radii, prominent cover art showcase, and micro-interactions.
- **Reference 2 (Tubefast)**: VisionOS/Android InnerTube streaming core, zero-allocation network buffering, gapless stream fetcher, and `FEmusic_home` recommendation engine.
- **Reference 3 (Tide Music)**: Slint UI architecture, compact/collapsed mini player geometry, clean vector path icons, and instant reactivity.

---

## 2. Color Palette & Surface Tokens

| Token | Hex / RGBA | Usage |
| :--- | :--- | :--- |
| **`bg_app`** | `#07080B` | Deep OLED base background |
| **`bg_sidebar`** | `#0B0D14F2` | Translucent frosted navigation bar |
| **`bg_card`** | `#12141FD9` | Primary content glass containers |
| **`bg_card_hover`** | `#1C1F2FE6` | Interactive hover highlight state |
| **`border_glass`** | `#FFFFFF14` | 1px subtle boundary stroke |
| **`border_highlight`** | `#FFFFFF2E` | 1.5px glowing focused stroke |
| **`accent`** | `#6366F1` | Primary Indigo brand accent |
| **`accent_gradient`** | `#6366F1` → `#EC4899` | Play buttons, active badges, vinyl accents |
| **`text_primary`** | `#FFFFFF` | 100% white headers and song titles |
| **`text_secondary`** | `#9CA3AF` | Neutral silver artist and album names |
| **`text_muted`** | `#6B7280` | Subtle timestamps and format badges |
| **`badge_bg`** | `#1E2235B3` | FLAC 24b / Opus 160k format tags |

---

## 3. Component Design & Precision Metrics

### 3.1. Top Navigation Bar (Height: 56px)
- **Search Pill**:
  - Dimensions: 320px width, 38px height, `border-radius: 19px` (full pill).
  - Background: `#10121DCC` with 1px `#FFFFFF18` border.
  - Layout: Search icon (16px, vertically centered) + Input text (13px, vertical alignment center) + Clear 'X' button (22px, strictly vertically centered with zero offset).
- **Engine Badge**:
  - Dimensions: 140px width, 30px height, pill shape.
  - Glowing status dot (6px emerald `#10B981`) + "Native WASAPI (Ultra-Low RAM)".
- **Volume Controller**:
  - Layout: `cross-axis-alignment: center;` (matching horizontal center line).
  - Speaker Icon: 26px square, 15px icon, instant click-to-mute.
  - Slider: 88px width, 24px height, track thickness 3px (expands to 5px on hover), thumb 10px circular white.
- **Mode Quick Switchers**:
  - Mini Mode (44x32px), Fullscreen / Gaming Mode toggle.

### 3.2. Home Screen & Feed Architecture
- **Featured Hero Showcase**:
  - Dynamic banner displaying currently playing track or top trending global hit.
  - Left: 68x68px high-res cover art (or stylized artist monogram).
  - Center: "FEATURED" pill badge + bold title (22px) + artist + 1-click "Play All" button with vector SVG play/pause icon.
- **Quick Picks / Trending Tracks**:
  - 6 to 10 tracks displayed with **real 40x40px album artwork thumbnails**, track index, title, artist, format badge, duration, and play action.
- **Trending Albums & Releases**:
  - 5 to 6 horizontal cards, each featuring **140x140px high-resolution album artwork**, title, artist, and year.
- **Top Artists**:
  - Circular avatar images with genre tags and 1-click follow/search.

### 3.3. Right Sidebar — Now Playing & Queue Showcase (Width: 280px)
- **Album Art Showcase**:
  - Dimensions: **210px × 210px** (generous, immersive display).
  - Corner Radius: 16px with subtle glossy border and ambient shadow.
- **Track Meta**:
  - Title: 15px bold white, left-aligned, auto-elided.
  - Artist: 12px neutral silver.
- **Transport Cluster**:
  - Seekbar with micro elapsed (`1:24`) and duration (`3:45`) labels.
  - Transport Buttons: Shuffle, Previous, **Prominent 40px Circular Play Button (Gradient Indigo to Pink)**, Next, Repeat.
  - Inline Volume Control with speaker icon and slider.
- **Up Next Queue**:
  - Scrollable playlist showing upcoming tracks with real thumbnails and click-to-play.

---

## 4. Mini Mode Architecture (Tide + Apple Dynamic Island)

### 4.1. Visual Design
- **Dimensions**: 460px width, 114px height.
- **Curvature**: `border-radius: 22px` (capsule aesthetic).
- **Surface**: Dark OLED glass `#0C0D16F6` with 1.5px `#383B5266` boundary.
- **Top Row**:
  - Cover Artwork: 46x46px with 10px radius and active playback accent glow.
  - Track Information: Title (13px bold), Artist (11px secondary).
  - Transport Controls: Previous (26px), Play/Pause (30px gradient), Next (26px).
  - Quick Window Actions: Favorite Heart, Always-on-Top Pin, Restore Normal Mode, Close.
- **Bottom Row**:
  - Elapsed Time (`1:18`).
  - Interactive Seekbar (`horizontal_stretch: 1`).
  - Total Duration (`3:34`).
  - Speaker Button (mute/unmute).
  - Dedicated Volume Slider (70px width).

### 4.2. Drag & Drop Movement
- Slint `TouchArea` intercepts left mouse button down on card surfaces.
- Calls `ReleaseCapture()` + `SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION, 0)`.
- Windows Desktop Window Manager handles hardware-accelerated movement, snapping, and multi-monitor transitions with 0.0% CPU overhead.

---

## 5. Audio Engine Dynamic Processing

- **Loudness Normalizer Target**: `-7.0 LUFS` (YouTube Music desktop standard).
- **Dynamic Gain Equation**: `gain_db = (TARGET_LUFS - (loudness_db - 14.0)).clamp(-24.0, +3.0)`.
- **Soft-Knee Peak Limiter**: Zero-allocation transparent compression threshold at `0.95` to eliminate digital clipping.
- **Atomic Real-Time Toggling**: `loudness_enabled` and `limiter_enabled` atomic flags allow instantaneous live toggle mid-playback without stream resets or audio drops.
