# PROJECT AURI: DEVELOPMENT & RELEASE SCHEDULE

This document is the **single source of truth** for all planned, in-progress, completed, and rescheduled updates for AURI. 

### Operational Rules:
1. **Schedule-First**: Every proposal, feature, architecture change, idea, or bug fix must first be scheduled into this table with an appropriate version number before any implementation begins.
2. **One Update at a Time**: Work proceeds strictly sequentially, completing one version milestone at a time, unless explicitly redirected with a high-priority user instruction.
3. **Traceability**: All updates must detail exactly what is added, modified, or optimized.
4. **Status Lifecycle**:
   - `Completed` [x]: Verified, tested, committed, and pushed.
   - `In Progress` [/]: Currently being engineered.
   - `Next Up` [!]: Next scheduled update in sequence.
   - `Planned` [ ]: Scheduled for upcoming release.
   - `Rescheduled` [~]: Postponed or re-scoped.

---

## Master Schedule & Version Table

| Version | Milestone / Update Title | Category | Scope & Details of What Is Updated | Status | Target / Notes |
|:---:|---|---|---|:---:|---|
| **v0.0.1** | **Repository & Phase 1 Specifications** | Architecture & Docs | - Initialize Git repository & link to GitHub remote.<br>- Deep research & competitive benchmarking (foobar2000, Tubefast, Limusic).<br>- Create `PRODUCT_SPECIFICATION.md`, `ARCHITECTURE.md`, `TECH_STACK_EVALUATION.md`, `PERFORMANCE_TARGETS.md`, `PROJECT_ROADMAP.md`.<br>- Verify host toolchains (Rust 1.99.1, Cargo, MSVC C++ Linker & Windows SDK). | **Completed** | Foundation & Planning |
| **v0.1.0** | **Multi-Mode UI Prototype & Settings Switcher** | Visual UI / Prototype | - Setup Rust UI framework with custom dark theme tokens (`#0F0F12`, `#17171C`, `#23232B`).<br>- **Normal Mode**: Implement Variant A1 (Right-panel embedded player card) & Variant A2 (Docked persistent bottom bar). Sidebar with navigation + quick library shortcuts.<br>- **Mini Mode**: Implement Variant B1 (Compact card with art) & Variant B2 (Micro-pill). Auto-dimming (~80% unhovered) and Always-on-Top toggle.<br>- **Gaming HUD**: Translucent overlay widget (Reference 3 style) with hotkey summon (`Win+Alt+M`).<br>- **Settings Panel**: Live switcher to toggle between UI variants.<br>- Populate views with realistic mock library data.<br>- Standalone portable release binary (`auri.exe`, 8.9 MB). | **Completed** | Phase 2 Visual Prototype |
| **v0.2.0** | **Minimal Audio Engine & WASAPI Sink** | Audio Core | - Implement native WASAPI shared mode audio output sink via CPAL.<br>- Integrate `symphonia` pure-Rust decoders (FLAC, MP3, AAC, Opus, WAV).<br>- Register real-time MMCSS audio thread (`AvSetMmThreadCharacteristicsW`).<br>- Lock-free circular audio ring buffer with sample-rate conversion.<br>- Core playback controls: Play, Pause, Resume, Stop, Seek, Volume, Mute.<br>- Standalone headless playback test verifying zero audio stutter. | **Completed** | Phase 3 Playback Core |
| **v0.3.0** | **Local Music Library & Queue Manager** | Library & Queue | - Local folder scanner and indexer (recursive directory scan).<br>- ID3v2, Vorbis comment, and FLAC metadata parser.<br>- Bounded artwork memory cache (max 16 MB LRU cache).<br>- Dynamic Playback Queue: Next, Previous, Shuffle (Fisher-Yates), Repeat (Off, One, All).<br>- Connect Local Files view in Normal Mode to the real audio engine. | **Next Up** | Local Playback Complete |
| **v0.4.0** | **Windows Deep Integration & Headless Playback** | OS Integration | - Decouple UI lifecycle from audio: Closing/hiding windows leaves playback running.<br>- System Tray integration (`Shell_NotifyIconW`) with dark context menu (Play/Pause, Next, Modes, Exit).<br>- Windows System Media Transport Controls (SMTC) integration for native volume flyout and keyboard media keys.<br>- Global hotkeys via `RegisterHotKey` (`Win+Alt+Space`, `Win+Alt+Right`, etc.). | **Planned** | Background Engine Ready |
| **v0.5.0** | **Anti-Cheat Safe Game Detection & Working Set Trimmer** | Performance & Gaming | - `SetWinEventHook` foreground listener detecting active fullscreen games without code injection.<br>- Configurable game process detection list.<br>- Automatic transition to Gaming Mode upon game focus.<br>- Window hiding (`SW_HIDE`) and swapchain suspension (0.0% GPU load).<br>- Memory purging via Windows `EmptyWorkingSet()` down to < 35 MB RAM. | **Planned** | Gaming Mode Complete |
| **v0.6.0** | **Streaming Provider Tier 1 (YouTube Music Innertube)** | Streaming Engine | - Define abstract `MusicProvider` trait.<br>- Async Innertube API client for YouTube Music search and metadata.<br>- Progressive audio stream resolution (WebM Opus 160kbps / MP4 AAC 128kbps).<br>- Sliding 8 MB memory window buffer for network streams.<br>- Unified queue supporting seamless mixing of local and streaming tracks. | **Planned** | Online Streaming Ready |
| **v0.7.0** | **Performance Benchmarking Suite & Latency Tuning** | Optimization & QA | - Implement automated profiling script (`scripts/benchmark.ps1`).<br>- Measure Private Bytes, Working Set, CPU%, and GPU% across all 3 modes.<br>- Live testing alongside demanding AAA games with active anti-cheat.<br>- Binary optimization: strip symbols, LTO enabled, panic abort. | **Planned** | Hardening & Tuning |
| **v1.0.0** | **AURI Initial Official Release** | Major Release | - Complete, unified desktop application (Normal Mode, Mini Mode, Gaming Mode).<br>- Full UI variant switcher in Settings.<br>- Dual-mode local and YouTube Music streaming.<br>- Release binary package (.exe installer / standalone portable zip).<br>- Finalized user documentation and performance benchmark reports. | **Planned** | **AURI 1.0.0 Baseline** |
| **v1.0.1** | **Hotfixes & Immediate Post-Launch Refinements** | Maintenance | - Address any edge-case playback glitches or UI DPI scaling issues reported during initial testing.<br>- Audio buffer auto-tuning based on host system load. | **Planned** | Post-1.0 Patch |
| **v1.1.0** | **Streaming Provider Tier 2 (Optional Cookie Sync)** | Feature Enhancement | - Optional session cookie import to fetch user's private YouTube Music playlists and Liked Songs.<br>- Local caching of private library metadata without embedded browser overhead. | **Planned** | Future Minor Release |
| **v1.2.0** | **Audio Visualizer & Equalizer Customization** | Audio Enhancement | - Optional 10-band parametric equalizer.<br>- Low-overhead FFT audio spectrum visualizer for Normal Mode and Companion HUD with auto-disable during Gaming Mode. | **Planned** | Future Minor Release |

---

## Change Log & Revision History

### Version: v0.2.0 (Completed Milestone)
- **Date**: 2026-11-05
- **Category**: Audio Core & Native WASAPI Output
- **What Was Done**:
  - Implemented native Windows Audio Session API (WASAPI) shared-mode output via `cpal::traits::DeviceTrait` & `StreamConfig`.
  - Integrated `symphonia` 0.6 pure-Rust demuxing and decoding engine for FLAC, MP3, AAC, Opus, Vorbis, and WAV files.
  - Registered Windows Multimedia Class Scheduler Service (MMCSS) priority via `AvSetMmThreadCharacteristicsW` in `src/audio/mmcss.rs` for glitch-free real-time audio playback under high GPU/CPU gaming workloads.
  - Implemented zero-allocation `LinearResampler` (`src/audio/resampler.rs`) for transparent sample rate conversion between file rates (e.g. 44.1 kHz, 96 kHz) and hardware mix rate (48 kHz).
  - Implemented melodic tone synthesizer (`src/audio/generator.rs`) generating smooth sine chimes (440 Hz A4, 330 Hz E4, 523 Hz C5) with click-free attack/decay envelopes for offline verification.
  - Implemented lock-free circular ring buffer (`ringbuf`) to decouple audio decode worker thread from the real-time WASAPI callback thread.
  - Connected UI callbacks (`on_toggle_play`, `on_seek`, `on_volume_changed`, `on_prev_track`, `on_next_track`) to `AudioEngine` in `src/main.rs`.
  - Added 10 FPS smooth position and seekbar synchronizer with `slint::Timer`.
  - Implemented and passed 7 unit tests covering resampler ratios, tone generator frame counting, click envelopes, sample bounds, and MMCSS thread priority registration.
  - Built optimized standalone release executable (`auri.exe`, 10.4 MB, zero external DLL dependencies).

### Version: v0.1.0 (Completed Milestone)
- **Date**: 2026-11-05
- **Category**: Visual UI Prototype & Multi-Mode Settings Switcher
- **What Was Done**:
  - Implemented pure native Slint UI with lightweight Femtovg hardware-accelerated 2D backend.
  - Built dark-first Apple Music/Spotify inspired theme system (`ui/theme.slint`).
  - Created **Normal Mode** (`ui/normal_mode.slint`) supporting both **Variant A1** (Reference 1 Right-Panel Card) and **Variant A2** (Docked Persistent Bottom Bar). Included hero artist section, album grid, popular tracks list, similar artists, and library shortcuts.
  - Created **Mini Mode** (`ui/mini_mode.slint`) supporting **Variant B1** (Compact Card with artwork, Reference 2A) and **Variant B2** (Micro-Pill, Reference 2B) with quick collapse, Always-on-Top, and idle opacity dimming.
  - Created **Gaming Mode Companion HUD** (`ui/gaming_mode.slint`) with vinyl record motif, high-res audio format badge (*"FLAC 96 KHZ 24-BIT"*), seekbar, transport controls, and spectrum visualizer.
  - Created live **Settings Switcher** enabling dynamic switching between all UI variants.
  - Built single-file portable release executable (`target/release/auri.exe`) measuring only **8.9 MB**.
  - Verified process startup and private memory footprint (57 MB baseline).

### Version: v0.0.1 (Completed Milestone)
- **Date**: 2026-11-05
- **Category**: Architecture, Tech Stack Evaluation, and Specification
- **What Was Done**:
  - Initialized Git repository and connected to `https://github.com/SuryakiranMK/AURI.git`.
  - Authored comprehensive specification and design documents (`PRODUCT_SPECIFICATION.md`, `ARCHITECTURE.md`, `TECH_STACK_EVALUATION.md`, `PERFORMANCE_TARGETS.md`, `PROJECT_ROADMAP.md`).
  - Evaluated and finalized Rust 1.99.1 + Slint + Symphonia + WASAPI single-process architecture.
  - Installed and verified Rust toolchain with Visual Studio MSVC C++ Linker and Windows SDK.
  - Established UI Variants Matrix supporting both Reference 1 (embedded card) and classic bottom bar, Reference 2 Mini Mode variants, and Gaming Mode (Headless + Companion HUD).

