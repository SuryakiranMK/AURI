# PROJECT AURI: MASTER DEVELOPMENT & RELEASE SCHEDULE
**A Minimalist, Ultra-Lightweight, Gaming-First Desktop Music Player**

This document serves as the **contract and single source of truth** for all planned, in-progress, completed, and rescheduled milestones for Project AURI.

---

### Core Operational Principles:
1. **Schedule-First**: Every proposed feature, architectural decision, or visual element must be tracked here with clear deliverables before deployment.
2. **Strict Release Protocol (User Directed)**:
   - **Continuous Release Execution**: Always compile, verify, and publish the latest GitHub release (`auri.exe` + `SHA256SUMS.txt`) immediately after each scheduled update milestone is live.
   - **Performance Verification**: Every release must adhere to strict performance ceilings: **< 35 MB RAM**, **0.0% idle GPU/CPU**, and **sub-10 MB binary size**.
3. **Audio Quality is Paramount**: Direct 160 kbps Opus stream decoding at 48,000 Hz, integrated studio loudness normalization (-7.0 LUFS target), and soft-knee peak limiting to prevent clipping.
4. **Permanent Architecture Standard**: Standardized on high-performance pure-Rust immediate-mode architecture (`eframe` / `egui 0.33` with Glow/OpenGL acceleration). Slint migration canceled permanently to avoid unnecessary heap model overhead.

---

## Master Schedule & Release Table

| Milestone | Phase & Category | Detailed Scope of Deliverables | Acceptance Criteria & Target | Status |
|:---:|---|---|---|:---:|
| **v1.0.0** | **Phase 1: High-Fidelity Audio Core & Streaming Engine** | - Pure-Rust Opus decoding via `vendor/opus-decoder` outputting 32-bit floating point PCM at 48 kHz.<br>- Direct YouTube Music Internal API integration with VisionOS bypass token.<br>- Built-in ITU-R BS.1770 -7.0 LUFS loudness normalizer & soft-knee peak limiter.<br>- Synchronous lean network client (`ureq`) with sliding stream window buffer. | - Unthrottled 160k Opus playback.<br>- Equalized volume across all tracks.<br>- Zero digital clipping. | **Released** [x] |
| **v1.1.0** | **Phase 2: Tubefast Design Parity & Persistent History** | - Translucent warm charcoal search bar with page history navigation (`<` / `>`) and `Ctrl+K` shortcut.<br>- 2-column Quick Picks & dynamic recommendation shelves with real album art.<br>- 84px persistent bottom playback bar with top scrubber and volume controls.<br>- Recently played disk persistence across app restarts. | - Visual layout matching target designs.<br>- Sub-50ms search debounce.<br>- History persistence verified. | **Released** [x] |
| **v1.2.0** | **Phase 3: Settings Page, Global Hotkeys, Mini Mode & RAM Optimization** | - **Dedicated Settings Page (`Route::Settings`)**: Full central settings view with Discord Rich Presence, Global Hotkeys, Audio DSP, Gaming RAM Optimization, and In-App Update check.<br>- **Windows Global Hotkeys (`src/hotkeys.rs`)**: Dedicated background thread utilizing `RegisterHotKey` with in-game hotkey recorder (`Ctrl+Alt+Space`, etc.) and conflict detection.<br>- **Floating Gaming Mini Mode**: Draggable, 16px curved radius always-on-top HUD (380×110) with seekbar, transport controls, album art, and mini volume slider.<br>- **Uncropped 3:1 Brand Banner**: Full 1024×341 graphic with soft alpha background falloff, eliminating rectangular seams while preserving intentional negative space.<br>- **Sub-35MB RAM Optimization**: Trimmed art cache (4 MB), compact audio buffer (2.5 MB), and automated `EmptyWorkingSet()` OS purge on minimize/mini-mode. | - Physical RAM drops to **25–35 MB**.<br>- Global hotkeys work during full-screen games.<br>- Binary size remains under 10 MB.<br>- Standalone GitHub release published. | **Current Release** [x] |
| **v1.3.0** | **Phase 4: Time-Synced Lyrics & Advanced Queue Management** | - **Synchronized Scrolling Lyrics**: Full LRCLIB integration with active karaoke-style line glow and smooth vertical scrolling.<br>- **Interactive Queue Reordering**: Drag-and-drop track reordering, duplicate detection, and "Play Next" priority queuing.<br>- **Mini Mode Enhancements**: Compact tray mini-widget and customizable opacity slider. | - Lyrics scroll with sub-frame accuracy.<br>- Queue reorders without audio stutter.<br>- RAM remains strictly < 35 MB. | **Next Up** [!] |
| **v1.4.0** | **Phase 5: Offline Cache & High-Performance Audio Sinks** | - **Offline Caching**: Local encrypted/uncompressed track caching for offline zero-network gaming playback.<br>- **WASAPI Exclusive Mode**: Optional bit-perfect bypass of Windows audio mixer for audiophile setups.<br>- **Companion Tray HUD**: Native Windows system tray flyout widget with media controls. | - Flawless offline playback.<br>- 0.0% network activity when playing cached tracks.<br>- Zero FPS impact during gaming. | **Planned** [ ] |

---

## Standing Release Checklist (Executed After Every Milestone)
- [x] Run `cargo test` to ensure 100% test pass rate.
- [x] Run `cargo build --release` with fat LTO, strip, and abort panic.
- [x] Compute SHA-256 checksum for `target/release/auri.exe`.
- [x] Generate signed tag and publish release to `https://github.com/SuryakiranMK/AURI`.
- [x] Verify in-app updater detects and validates the new release.


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
