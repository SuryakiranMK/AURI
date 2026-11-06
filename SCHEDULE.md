# PROJECT AURI: MASTER DEVELOPMENT & RELEASE SCHEDULE
**A Minimalist, Ultra-Lightweight, Gaming-First Desktop Music Player**

This document serves as the **contract and single source of truth** for all planned, in-progress, completed, and rescheduled milestones for the new synthesized build of AURI.

---

### Core Operational Principles:
1. **Schedule-First**: Every proposed feature, architectural decision, or visual element must be scheduled here with clear deliverables before any code is written.
2. **Strict Phase Sequencing (User Directed)**:
   - **Step 1: Core Engine & Audio Fidelity First**: Make local & streaming playback work with studio-grade audio quality before building heavy UI.
   - **Step 2: Complete Normal Mode**: Fully engineer the main desktop experience (Apple Music / Spotify inspired sleek UI) until 100% complete.
   - **Step 3: Mini Mode**: Build the compact floating controller (Reference 2).
   - **Step 4: Gaming Mode**: Implement headless UI suspension (0% GPU, <35 MB RAM), anti-cheat safe game detection, and companion HUD.
3. **Audio Quality is Paramount**: Implement loudness normalization (-7.0 LUFS target) and dual-engine architecture (Native WASAPI vs Studio libmpv).
4. **No Code Implementation Before Greenlight**: Work will begin only after the user reviews and approves this schedule.

---

## Master Schedule Table

| Milestone | Phase & Category | Detailed Scope of Deliverables | Acceptance Criteria & Target | Status |
|:---:|---|---|---|:---:|
| **v0.1.0** | **Phase 1: High-Fidelity Audio Core & Streaming Engine** | - **Dual-Engine Abstraction**: Unified `AudioEngine` trait with runtime switching between Native WASAPI Engine and Studio libmpv Engine.<br>- **Native WASAPI Engine**: Pure Rust `symphonia` decoders (FLAC, MP3, AAC, Opus, WAV) + WASAPI shared mode + Sinc polyphase resampler.<br>- **Loudness Normalization**: Extract `loudnessDb` from YouTube Music; apply $-7.0\text{ LUFS}$ target gain formula with soft peak limiter (`limit=1`) to eliminate uneven volume and clipping.<br>- **Studio libmpv Engine**: Integrated C `libmpv` backend with double-deck gapless lookahead, crossfade, and Soxr resampling.<br>- **Lean Network Client (Tubefast Style)**: Synchronous `ureq` HTTP client (zero async/Tokio runtime bloat) with fast token rotation.<br>- **Sliding 8 MB Stream Buffer**: Progressive HTTP Range requests (`&range=from-to`) with automatic chunk eviction.<br>- **Multi-Criteria Stream Selector**: Prioritize high-bitrate Opus itag 251 (~160kbps) with fallback to AAC itag 140 (~128kbps).<br>- **Unified Local & Cloud Queue**: Dynamic queue supporting continuous playback across local audio files and YouTube Music streams. | - Playback works flawlessly offline (local files) and online (YouTube Music).<br>- Equalized, punchy sound with zero clipping.<br>- Zero buffer underrun pops/clicks.<br>- Audio thread runs isolated from UI. | **Planned** [ ] |
| **v0.2.0** | **Phase 2A: Normal Mode — Sleek Design System & Navigation** | - **Slint Theme & Design System**: Dark-first tokens (`#0C0C0E` app bg, `#121217` surface, `#181820` card, `#6366F1` indigo accent, `#EC4899` secondary).<br>- **Crisp Vector Iconography**: 24 pure SVG vector icons (Play, Pause, Next, Prev, Volume, Search, Heart, Shuffle, Repeat, Settings, Folder, Album, Queue, etc.).<br>- **Left Navigation Sidebar**: Brand header (ready for new app icon in next prompt), navigation links (Home, Search, Albums, Local Files, Favorites, Settings), and live RAM monitor.<br>- **Top Navigation Bar**: Clickable search pill with focus outline, volume control slider with mute toggle, and active audio engine badge.<br>- **Right Play Queue Panel (Reference 1)**: Embedded now-playing card with circular artwork motif, track title, artist, seekbar, transport controls, and dynamic Up Next list. | - Zero generic wide gray boxes.<br>- Clean typography, balanced padding, and Apple Music level elegance.<br>- Responsive layout up to 4K resolutions. | **Planned** [ ] |
| **v0.2.1** | **Phase 2B: Normal Mode — Home View & Content Polish** | - **Featured Hero / Artist Banner**: Clean header with artist portrait, title, and Play/Shuffle actions.<br>- **Quick Picks / Trending Songs Table**: Authentic Apple Music / Spotify table structure: track index, 40x40px album artwork, title, artist, album, format badge, duration, heart toggle, and play trigger on hover.<br>- **Featured Albums Grid**: Responsive grid of square album cards (140x140px) with subtle hover scale (1.04x), album title, and release year.<br>- **Smooth Micro-Animations**: 120ms background transitions, interactive slider thumb expansion on hover, and active state indicators.<br>- **Instant Play Integration**: Clicking any track row or album plays immediately via the active audio engine. | - Visually stunning, high-density music showcase.<br>- Zero layout clipping or awkward void stretching.<br>- Smooth hover interactions. | **Planned** [ ] |
| **v0.2.2** | **Phase 2C: Normal Mode — Unified Search & Library Views** | - **Unified Search Engine**: Simultaneous real-time search across local scanned files and YouTube Music online catalog.<br>- **Search Results View**: Dedicated results list with local vs streaming source badges, duration, and instant play trigger.<br>- **Albums View**: Dedicated library grid of all scanned and discovered music albums.<br>- **Local Files View**: Full tabular local library browser with format badges (FLAC 24-bit, MP3 320k, WAV), file sizes, and quick play.<br>- **Favorites View**: Starred tracks collection with quick Play All and Shuffle actions.<br>- **Settings View**: Interactive preference panel: toggle between **Native WASAPI Engine** and **Studio libmpv Engine**, loudness normalization on/off, audio buffer latency, and library folder scanner. | - Complete functional parity with desktop music players.<br>- Search results appear without blocking UI.<br>- Settings toggle between audio engines in real time. | **Planned** [ ] |
| **v0.3.0** | **Phase 3: Mini Mode — Compact Floating Controller (Reference 2)** | - **Reference 2 Exact Fidelity**: Downscale window to a true compact floating controller.<br>- **Variant A (Card with Artwork)**: 310×84px rounded card, 40×40px album art, track title, artist, slim 3px seekbar, elapsed/total timestamps (`2:22 / 9:40`), signature solid white circular play button, previous/next, heart, and volume.<br>- **Variant B (Micro-Pill)**: 280×48px ultra-minimalist strip without artwork.<br>- **Native Window Management**: Enforce strict window dimensions via Win32 `SetWindowPos` and `ShowWindow(SW_RESTORE)` to prevent maximized window stretching.<br>- **Borderless Dragging**: Drag window from any blank area via `WM_NCHITTEST` (`HTCAPTION`).<br>- **Always-on-Top Toggle**: Pin button toggling `WS_EX_TOPMOST` with visual state indicator. | - Compact, pixel-perfect floating music widget.<br>- Zero distraction during multitasking.<br>- Memory stays < 45 MB in Mini Mode. | **Planned** [ ] |
| **v0.4.0** | **Phase 4: Gaming / Performance Mode & Anti-Cheat Safe Integration** | - **Pure Headless Mode**: Main window hidden (`SW_HIDE`), swapchain and rendering halted (strictly **0.0% GPU load**).<br>- **Companion HUD Mode (Reference 3)**: Ultra-compact translucent dark overlay widget with vinyl motif, hi-res format badge (*"FLAC 96 KHZ 24-BIT"*), and controls.<br>- **Anti-Cheat Safe Game Detector**: Passive Win32 `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)` tracking active games without DLL injection, code injection, or anti-cheat collisions.<br>- **Working Set RAM Trimmer**: Purge unused UI memory via Windows `EmptyWorkingSet(GetCurrentProcess())`, dropping memory footprint to **< 35 MB RAM**.<br>- **System Tray & Media Keys**: Windows SMTC volume overlay, keyboard media keys (`Play/Pause`, `Next`, `Prev`), gaming shortcuts (`Win+Alt+Space`, `Win+Alt+M`), and dark Win32 tray popup menu. | - 0.0% GPU load during gaming.<br>- Physical RAM drops below 35 MB.<br>- Zero audio dropouts even at 100% CPU game loads.<br>- 100% safe with BattlEye, Easy Anti-Cheat, Vanguard, and VAC. | **Planned** [ ] |
| **v1.0.0** | **Phase 5: Final Polish, Optimization & Official Release** | - **App Icon Embedding**: Compile the official application icon (to be provided by user) into Windows PE resources (`auri.exe`) and taskbar/window titlebars.<br>- **Automated Benchmarking**: Run `scripts/benchmark.ps1` measuring Working Set, Private Bytes, CPU%, and GPU% across all three modes.<br>- **Binary Hardening**: Build optimized release profile (`lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = true`).<br>- **Standalone Distribution Package**: Create self-contained portable Windows x64 zip package (`AURI-v1.0.0-Windows-x64.zip`). | - Single standalone executable.<br>- Official benchmark report certifying compliance with `PERFORMANCE_TARGETS.md`. | **Planned** [ ] |

---

## Detailed Technical Roadmap by Phase

### Phase 1: High-Fidelity Audio Core & Streaming Engine (Milestone v0.1.0)
1. **Audio Engine Architecture**:
   - Define abstract `AudioSink` trait with `play`, `pause`, `seek`, `set_volume`, `set_loudness_gain`, and `get_position`.
   - **Native WASAPI Implementation**: Direct Windows Audio Session API shared-mode sink via `cpal` with real-time MMCSS thread scheduling (`AvSetMmThreadCharacteristicsW("Audio", ...)`).
   - **Loudness Normalization**: Implement YouTube Music's $-7.0\text{ LUFS}$ target calculation:
     $$\text{gain\_db} = \text{clamp}(-24.0, +3.0, -7.0 - (\text{loudnessDb} - 14.0))$$
     Combined with a soft-knee peak limiter to prevent clipping on boosted tracks.
   - **Studio libmpv Implementation**: C `libmpv` double-deck wrapper for gapless lookahead and crossfade.
2. **Network Streaming Engine**:
   - Tubefast-style synchronous `ureq` HTTP client with token rotation.
   - 8 MB sliding window buffer (`&range=from-to`) ensuring bounded memory.
   - Innertube audio stream resolver extracting direct Opus (~160kbps) and AAC (~128kbps) streams.
3. **Queue Manager**:
   - Lock-free queue supporting Next, Previous, Shuffle (Fisher-Yates), Repeat (Off, One, All), and seamless local + cloud mixing.

### Phase 2: Normal Mode Desktop Player (Milestones v0.2.0, v0.2.1, v0.2.2)
1. **Design System & Theme Tokens**:
   - Modern dark-first color scheme inspired by Apple Music and Spotify (`#0C0C0E` background, `#181820` cards, refined borders, and vibrant accents).
   - 24 vector SVG icons.
2. **Content Views**:
   - **Home**: Curated banner, Quick Picks table with track index, real artwork, artist, duration, and play trigger. Square album cards with hover elevation.
   - **Search**: Fast unified search querying local folders and YouTube Music catalog simultaneously.
   - **Library**: Dedicated views for Albums, Local Files table, Favorites, and Playlists.
   - **Settings**: Live toggle between Native WASAPI Engine and Studio libmpv Engine, loudness normalization toggle, and buffer latency slider.
3. **Right Play Queue Panel (Reference 1)**:
   - Dedicated now-playing card with circular artwork motif, seekbar, controls, and dynamic queue list.

### Phase 3: Mini Mode Floating Controller (Milestone v0.3.0)
1. **Reference 2 Fidelity**:
   - Compact 310×84px card with 40×40px album art, title, artist, slim seekbar, elapsed/total time, white circular play button, and controls.
   - Micro-pill 280×48px option.
2. **Native Window Management**:
   - Un-maximize and enforce strict dimensions via Win32 `SetWindowPos` and `ShowWindow(SW_RESTORE)`.
   - Smooth borderless dragging (`WM_NCHITTEST` / `HTCAPTION`).
   - Always-on-Top toggle (`WS_EX_TOPMOST`).

### Phase 4: Gaming Mode & Background Performance (Milestone v0.4.0)
1. **Pure Headless Mode**:
   - `ShowWindow(SW_HIDE)` drops GPU swapchain rendering to **0.0% GPU load**.
2. **Memory Purging**:
   - Invoke `EmptyWorkingSet(GetCurrentProcess())` to purge transient UI memory down to **< 35 MB RAM**.
3. **Anti-Cheat Safe Game Detector**:
   - Win32 `SetWinEventHook` monitoring foreground window changes without injecting DLLs or touching game memory.
4. **Companion HUD (Reference 3)** & Tray Integration:
   - Minimal translucent HUD widget.
   - Global media shortcuts (`Win+Alt+Space`, `Win+Alt+M`) and Windows SMTC volume flyout.

### Phase 5: Verification, Benchmarking & Packaging (Milestone v1.0.0)
1. Embed official application icon (to be provided in next prompt).
2. Automated performance verification using `scripts/benchmark.ps1`.
3. Standalone release package (`auri.exe`).
