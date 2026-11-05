# PRODUCT SPECIFICATION: PROJECT AURI
**A Minimalist, Ultra-Lightweight, Gaming-First Desktop Music Player**

---

## 1. Product Overview & Identity

### 1.1 Mission Statement
**AURI** is a native Windows desktop music player engineered from the ground up for power users, gamers, developers, and professionals who demand uncompromised system responsiveness while listening to music. 

Modern desktop music players (Spotify, YouTube Music Web, Electron-based clients) consume anywhere from 300 MB to 1.5 GB of RAM, spawn 6 to 12 background processes, and continuously consume 1–5% CPU and non-trivial GPU resources just to maintain a web runtime. AURI eliminates this bloat entirely. It operates with the surgical efficiency of legendary native Win32 audio tools (such as foobar2000), wrapped in a modern, dark-first, minimalist design aesthetic inspired by Apple Music and Spotify.

### 1.2 Core Value Proposition
- **Extreme Lightweight Footprint**: Sub-70 MB RAM in Normal Mode, sub-35 MB in Gaming Mode, < 0.5% average CPU during active audio playback.
- **Genuine Gaming / Performance Mode**: Automatically or manually suspends graphical rendering, freezes non-essential tasks, trims the OS working set, and runs headless with 0.0% GPU load.
- **Decoupled Audio Core**: Audio playback operates independently of the UI. If the UI is hidden, minimized, or crashes, the audio stream remains uninterrupted.
- **Unified Local & Streaming**: First-class, zero-latency local library playback (FLAC, MP3, AAC, Opus, WAV) alongside a modular streaming provider interface.

---

## 2. Operating Modes & User Experience

AURI features three distinct, meticulously tuned operating modes:

```
+--------------------------------------------------------------------------+
|                                AURI MODES                                |
+--------------------------------------------------------------------------+
|  [Normal Mode]             [Mini Mode]               [Gaming Mode]       |
|  Full Desktop UI           Floating Companion        Headless / HUD      |
|  75 MB RAM target          50 MB RAM target          < 35 MB RAM target  |
|  Interactive Library       Always-on-Top Pill        0% GPU / Suspended  |
+--------------------------------------------------------------------------+
```

### 2.1 Mode A: Normal Mode (Full Desktop Player)
- **Target Audience**: Active browsing, playlist curating, music discovery, library exploration.
- **Visual Design**:
  - Dark-first aesthetic (`#0F0F12` background, `#17171C` cards, `#23232B` borders, refined neon/vibrant accent).
  - Left navigation sidebar: Home, Albums, Artists, Playlists, Favorites, Local Files, Settings.
  - Center workspace: Search bar, hero artist/album banner, grid view for albums/playlists, sorted table for tracks with duration, artist, and heart icon.
  - Right sidebar: Interactive Play Queue with embedded mini-controller card.
- **Performance Characteristics**:
  - Lazy rendering: Only visible list items are laid out and rendered.
  - Artwork thumbnail caching: Bounded LRU memory cache (max 16 MB).
  - Smart frame throttling: Redraws only on state changes or animations; sleeps at 0% CPU when idle.

### 2.2 Mode B: Mini Mode (Floating Controller)
- **Target Audience**: Multitasking, working across spreadsheets or code editors, casual browsing.
- **Visual Design**:
  - Compact rounded card (e.g., 360 x 96 px or 320 x 64 px micro variant).
  - High-res square artwork thumbnail, smooth scrolling marquee for long track titles, artist name.
  - Seek slider with elapsed and remaining duration (`2:15 / 4:30`).
  - Transport controls: Previous, Play/Pause toggle, Next, Favorite toggle, Volume slider.
  - Draggable from any blank surface; optional "Always on Top" (`WS_EX_TOPMOST`) toggle.
- **Performance Characteristics**:
  - Zero heavy background layout updates.
  - Render updates capped to seekbar step (1 second or hover interactions).
  - Memory target: < 50 MB.

### 2.3 Mode C: Gaming / Performance Mode (Signature Feature)
- **Target Audience**: Competitive and intensive PC gaming, high-load rendering, benchmarking.
- **Operation**:
  - **Headless Core**: Main window is hidden (`ShowWindow(SW_HIDE)`).
  - **GPU Render Loop Halted**: Swapchain presentation stops entirely. GPU consumption is strictly **0.0%**.
  - **Working Set Trimmed**: Invokes Windows `SetProcessWorkingSetSize` / `EmptyWorkingSet` to purge transient UI memory pages.
  - **Background Work Suspended**: Artwork loading, non-critical metadata scraping, lyrics synchronization, and all non-playback timers are suspended.
  - **Interaction**:
    - Controlled via global Windows media keys (Play, Pause, Prev, Next, Mute, Volume Up/Down).
    - Windows System Media Transport Controls (SMTC) overlay for native Windows volume popup.
    - System tray icon with right-click context menu (Play/Pause, Next, Switch Mode, Exit).
    - Optional zero-overhead click-through companion HUD (on secondary monitor or top-corner edge).
  - **Anti-Cheat Safe Game Detection**:
    - Uses standard Win32 `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)` and `QueryFullProcessImageNameW`.
    - **Never** injects code, hooks memory, creates overlays in game processes, or violates anti-cheat policies (EasyAntiCheat, BattlEye, Denuvo, Vanguard).

---

## 3. Audio & Playback Specifications

### 3.1 Audio Formats & Codecs
- **Lossless**: FLAC (up to 24-bit / 192 kHz), WAV (PCM 16/24/32-bit, float).
- **Lossy**: MP3, AAC / M4A, Ogg Vorbis, Opus (standard YouTube stream codec).
- **Sample Rate Conversion**: High-quality sinc or linear interpolation resampler when hardware requires sample rate adaptation.

### 3.2 Output Architecture
- **Primary Engine**: Windows Audio Session API (**WASAPI**) via Shared Mode for seamless mixing with game audio, or Exclusive Mode (user-configurable for audiophiles).
- **Buffer Management**: Lock-free ring buffer (sliding window max 8 MB for streaming; deterministic ring buffer for local files) to ensure uninterrupted audio during system load spikes.
- **Real-Time Thread Scheduling**: Audio processing threads register with Windows Multimedia Class Scheduler Service (**MMCSS**) using `AvSetMmThreadCharacteristicsW("Audio", ...)`.

---

## 4. Music Provider Architecture

```
                 +-----------------------+
                 |    MusicProvider      |  (Abstract Trait / Interface)
                 +-----------+-----------+
                             |
             +---------------+---------------+
             |                               |
    +--------v--------+             +--------v--------+
    |  LocalProvider  |             |  StreamProvider |
    |  (Filesystem,   |             | (YouTube Music, |
    |   ID3/FLAC Tags)|             |  Modular HTTP)  |
    +-----------------+             +-----------------+
```

- **Modular Design**: The audio core relies entirely on a generic `MusicProvider` trait.
- **Local Provider**: Direct file access, ID3/Vorbis tag parsing, directory indexing with low-memory SQLite or binary cache.
- **Streaming Provider**: Modular HTTP stream resolver. Designed to parse stream URLs and metadata without embedding a browser. Complies with security policies: no credential scraping, no DRM circumvention, and no ads injection.

---

## 5. Non-Functional Requirements & Constraints

1. **Memory Budget**:
   - Idle Background / Gaming Mode: < 35 MB
   - Mini Mode: < 50 MB
   - Normal Mode: < 75 MB
2. **CPU Utilization**:
   - Idle: < 0.1%
   - Active Local Playback: < 0.5% (on Intel i7-11800H baseline)
3. **GPU Utilization**:
   - Gaming Mode: 0.0%
   - Idle Normal Mode: 0.0% - 0.2%
4. **Binary & Deployment**:
   - Single portable executable.
   - Zero external runtime dependencies (no separate WebView2 runtime process, no Chromium, no Electron).
