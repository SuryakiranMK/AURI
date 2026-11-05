# PROJECT ROADMAP & IMPLEMENTATION MILESTONES: PROJECT AURI

---

## Phase 1: Research, Architecture & Technical Specification (Current Phase)
- [x] Inspect workspace and initialize Git repository (`https://github.com/SuryakiranMK/AURI.git`).
- [x] Analyze design reference images (Desktop layout, Floating mini-player, Gaming HUD).
- [x] Deep research into competing lightweight architectures (foobar2000, Tubefast, Limusic).
- [x] Technical evaluation of programming languages, UI frameworks, audio engines, and Windows integration.
- [x] Produce comprehensive architectural documentation:
  - `PRODUCT_SPECIFICATION.md`
  - `ARCHITECTURE.md`
  - `TECH_STACK_EVALUATION.md`
  - `PERFORMANCE_TARGETS.md`
  - `PROJECT_ROADMAP.md`
- [ ] Align with user on architectural proposals, toolchain installation, and doubts.

---

## Phase 2: Visual Prototype (Interactive Multi-Mode UI)
**Objective**: Build a responsive, dark-first UI matching the design aesthetic using mock data before coupling to the audio engine.

- [ ] Setup UI scaffolding with dark theme tokens (`#0F0F12` background, refined borders, neon accents).
- [ ] Implement **Normal Mode**:
  - Left navigation sidebar (Home, Albums, Artists, Local, Favorites, Playlists, Settings).
  - Top header with search bar and user profile.
  - Hero artist/album banner with action buttons.
  - Album grid and popular track list with hover interactions.
  - Right sidebar Play Queue with integrated now-playing card.
- [ ] Implement **Mini Mode**:
  - Compact floating card with artwork thumbnail, title, artist, seekbar, and transport controls.
  - Borderless window dragging support and Always-on-Top toggle.
- [ ] Implement **Gaming Mode HUD**:
  - Minimal HUD widget / floating pill with translucent dark glass aesthetic.
- [ ] Implement seamless window transitions between Normal, Mini, and Gaming modes.
- [ ] Benchmark prototype idle memory and GPU repaint rates.

---

## Phase 3: Minimal Audio Engine (Local Playback Core)
**Objective**: High-fidelity, glitch-free local audio playback with zero UI dependency.

- [ ] Implement WASAPI output sink via CPAL with real-time MMCSS scheduling (`AvSetMmThreadCharacteristicsW`).
- [ ] Implement Symphonia-based audio demuxer and decoder pipeline (FLAC, MP3, AAC, Opus, WAV).
- [ ] Build lock-free audio ring buffer with bounded memory.
- [ ] Implement core playback controls: Play, Pause, Resume, Stop, Seek, Volume, Mute.
- [ ] Implement Queue Manager: Next, Previous, Shuffle (Fisher-Yates), Repeat (Off, One, All).
- [ ] Create automated headless test suite verifying playback stability, sample-accurate seeking, and memory bounds.

---

## Phase 4: Background Playback & Windows Integration
**Objective**: Complete decoupling of playback from the graphical interface.

- [ ] Headless background mode: Audio playback persists when windows are closed or hidden.
- [ ] System Tray integration (`Shell_NotifyIconW`) with dark context menu and quick playback actions.
- [ ] Global keyboard shortcuts (`RegisterHotKey`) for system-wide playback control.
- [ ] Windows System Media Transport Controls (SMTC) integration for native Windows volume overlay and hardware media keys.
- [ ] Anti-cheat safe Game Detection module (`SetWinEventHook` on `EVENT_SYSTEM_FOREGROUND`).
- [ ] Automatic Gaming Mode trigger and working set trimmer (`EmptyWorkingSet`).

---

## Phase 5: Music Provider Integration (Streaming & Metadata)
**Objective**: Modular music source integration supporting local libraries and online streaming.

- [ ] Define abstract `MusicProvider` trait with asynchronous search, metadata, and stream resolution.
- [ ] Implement `LocalFilesystemProvider`: Fast directory scanner, ID3/FLAC metadata parser, artwork extractor.
- [ ] Implement `StreamingProvider` (YouTube Music Innertube API client):
  - Search endpoint parser (tracks, albums, artists).
  - Direct audio stream resolver (Opus/WebM chunks).
  - Sliding 8 MB memory window for chunked progressive streaming.
- [ ] Unified queue integration allowing interleaved local and streaming tracks.

---

## Phase 6: Testing, Benchmarking & Optimization
**Objective**: Rigorous verification against performance budgets during active gaming sessions.

- [ ] Develop automated benchmark script (`scripts/benchmark.ps1`) tracking Private Bytes, Working Set, CPU%, and GPU%.
- [ ] Profile and eliminate memory allocations, unneeded font glyphs, and redraw triggers.
- [ ] Execute real-world gaming stress test: Run AURI in Gaming Mode alongside demanding AAA games (with anti-cheat active) to verify 0.0% GPU load and zero audio dropouts.
- [ ] Compile release binaries with aggressive LTO and size optimizations.
