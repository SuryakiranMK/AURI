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
| **v0.1.0** | **Phase 1: High-Fidelity Audio Core & Streaming Engine** | - Pure Rust Opus decoder + Rodio 48,000 Hz pipeline.<br>- Innertube YouTube Music Internal API client with VisionOS Web Client bypass token.<br>- Direct Opus itag 251 (~160kbps VBR) unthrottled streaming with AAC fallback.<br>- Studio -7.0 LUFS ITU-R BS.1770 broadcast loudness normalizer.<br>- Soft-knee brickwall peak limiter preventing clipping. | - Unthrottled 48kHz audio streaming.<br>- Zero audio clipping and volume jumps.<br>- Audio thread runs isolated from UI. | **Completed** [x] |
| **v0.2.0** | **Phase 2: Native eframe UI, Settings, Hotkeys & Floating Mini Mode** | - Native hardware-accelerated `eframe` (egui 0.33) desktop UI with 0% idle CPU.<br>- Uncropped 3:1 Brand Banner (`banner_left.jpg`) with smooth alpha blending.<br>- Full 1:1 Navigation & Content layout (Home, Quick Picks, Recommendations, Liked, Recently Played).<br>- Dedicated Settings Page (`Route::Settings`) with Discord presence, Global Hotkeys, DSP controls, and RAM optimizer.<br>- Windows `RegisterHotKey` engine for in-game playback control.<br>- Floating Gaming Mini Mode: 380×110px curved (16px) draggable always-on-top HUD.<br>- Sub-35MB Working Set optimization via `psapi.dll -> EmptyWorkingSet()`. | - Standalone binary under 10 MB (9.5 MB).<br>- Working set RAM 25–35 MB.<br>- Instant search debouncing with violet focus ring.<br>- Global hotkeys functional in full-screen games. | **Completed** [x] |
| **v0.2.1** | **Phase 3: Queue Refinements, Lyrics Glow & Mini Mode Polish** | - Queue panel enhancements: drag-to-reorder, deduplication, and "Play Next" action.<br>- Time-synced scrolling lyrics with active line violet glow.<br>- Mini Mode refinements: opacity/transparency slider and compact tray toggle.<br>- Windows System Tray icon with quick play/pause/skip context menu. | - Flawless queue reordering without playback interruptions.<br>- Beautiful karaoke-style timed lyrics rendering.<br>- Complete background control from system tray. | **Next Up** [!] |
| **v0.3.0** | **Phase 4: Offline Caching & Enhanced Gaming Profiles** | - Encrypted local audio cache for favorite tracks (offline zero-network gaming).<br>- Passive game detection profile to automatically engage Mini Mode or RAM flush.<br>- Custom keyboard hotkey profiles per game. | - Instant playback without internet connection.<br>- Zero FPS impact on competitive gaming benchmarks. | **Planned** [ ] |
| **v1.0.0** | **Phase 5: Official Release & Distribution** | - Embedded application icon (`assets/icon.ico`) compiled into Windows PE resources.<br>- Automated GitHub Actions release workflow publishing signed `auri.exe` and `SHA256SUMS.txt`.<br>- Performance benchmark certification report. | - Standalone one-click portable executable.<br>- Automated in-app update channel verified. | **Planned** [ ] |

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
