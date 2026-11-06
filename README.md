# AURI: Minimalist, Ultra-Lightweight, Gaming-First Desktop Music Player

[![Release](https://img.shields.io/badge/Release-v1.0.0-8B5CF6.svg?style=flat-square)](https://github.com/SuryakiranMK/AURI/releases/tag/v1.0.0)
[![License](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011-0078D6.svg?style=flat-square)](https://github.com/SuryakiranMK/AURI)
[![Rust](https://img.shields.io/badge/Rust-1.99.1%20(2021)-orange.svg?style=flat-square)](https://www.rust-lang.org/)
[![Working Set](https://img.shields.io/badge/Gaming%20RAM-%3C%2032.5%20MB-10B981.svg?style=flat-square)](BENCHMARK_REPORT.md)
[![GPU Load](https://img.shields.io/badge/Headless%20GPU-0.0%25-10B981.svg?style=flat-square)](BENCHMARK_REPORT.md)

**AURI** is an ultra-lightweight, native Windows desktop music player engineered specifically for PC gamers, power users, and developers. It completely replaces heavy browser-based players (like YouTube Music in Chrome/Edge, which consume 500–850 MB of RAM and multiple CPU processes) with a single, standalone native executable consuming **< 35 MB of RAM and strictly 0.0% GPU when playing in the background**.

---

## ⚡ Key Highlights & Architecture

- **Single Native Process**: Built in pure Rust with Slint hardware-accelerated 2D rendering (Femtovg backend). Zero Electron, zero Chromium, and zero permanently running helper processes.
- **Pure-Rust Audio Core**: Powered by `symphonia` 0.6 demuxing and decoding engines supporting FLAC, MP3, AAC, Opus, Vorbis, and WAV.
- **Windows WASAPI & MMCSS**: Shared-mode WASAPI output with Windows Multimedia Class Scheduler Service (`AvSetMmThreadCharacteristicsW`), guaranteeing glitch-free, real-time audio playback even under 100% CPU/GPU gaming loads.
- **YouTube Music Streaming (Tier 1 Innertube)**: Native, zero-browser YouTube Music API client resolving progressive WebM Opus (160 kbps) and MP4 AAC (128 kbps) streams with a strict sliding 8 MB memory buffer.
- **Anti-Cheat Safe Game Detection**: Completely passive `SetWinEventHook` foreground listener that detects 30+ popular games (`cs2.exe`, `valorant.exe`, `dota2.exe`, `r5apex.exe`, `eldenring.exe`, `helldivers2.exe`, etc.) without code injection, DLL injection, or memory hooking. 100% safe with **Easy Anti-Cheat, BattlEye, Vanguard, Ricochet, and VAC**.
- **Working Set RAM Trimmer**: Integrated Windows native `EmptyWorkingSet()` memory purging that drops RAM footprint to **< 32.5 MB** upon entering Gaming Mode or hiding the interface.
- **Decoupled Window Lifecycle**: The audio engine runs independently in background threads. Closing or hiding the window leaves playback running headlessly with 0.0% GPU overhead.

---

## 🎮 Three Operating Modes & Live UI Variants

AURI provides three distinct operating modes switchable live from the interface or system tray:

### 1. Normal Mode (Full Desktop Experience)
- Complete, elegant dark-first aesthetic inspired by Apple Music and Spotify.
- **Variant A1 (Reference 1 Style)**: Hero artist view, album cover grid, popular tracks list, and embedded now-playing card inside the Right Play Queue.
- **Variant A2 (Docked Persistent Bar)**: Classic 70px persistent bottom player bar with full seek controls and collapsible play queue.
- Live interactive search bar querying YouTube Music and local directories simultaneously.

### 2. Mini Mode (Compact Floating Controller)
- **Variant B1 (Compact Card with Art)**: 380x120px floating controller with artwork, track title, artist, seekbar, and transport buttons.
- **Variant B2 (Micro-Pill)**: Ultra-thin 340x60px micro-player designed to sit unobtrusively at the corner of a second monitor.
- Supports **Always-on-Top** pin toggle and **Idle Dimming** (~80% opacity when unhovered).

### 3. Gaming Mode (Signature Dual-Paradigm)
- **Pure Headless Mode**: Completely hides the graphical interface (`SW_HIDE`), stopping all Femtovg swapchain rendering (**0.0% GPU load**) and purging RAM to **< 35 MB**. Audio playback continues seamlessly via real-time WASAPI callback.
- **Companion HUD**: Translucent dark overlay with vinyl record motif, high-resolution format badge (*"FLAC 96 KHZ 24-BIT"*), and spectrum visualizer summoned instantly via `Win+Alt+M`.

---

## 📊 Performance Benchmarks (AURI vs. Alternatives)

Audited via our automated profiling suite ([`scripts/benchmark.ps1`](scripts/benchmark.ps1)):

| Application | Working Set (RAM) | Committed Private Bytes | GPU Overhead | Processes |
|:---|:---:|:---:|:---:|:---:|
| **YouTube Music (Chrome)** | 450 MB – 850 MB | 500 MB – 900 MB | 1.5% – 5.0% (Active) | 8 – 14 |
| **Spotify Desktop** | 320 MB – 600 MB | 350 MB – 650 MB | 1.0% – 3.0% (Active) | 5 – 8 |
| **AURI Normal Mode** | **102 MB** (Peak) | **59.8 MB** | < 0.5% | **1** |
| **AURI Mini Mode** | **~42 MB** | **38.5 MB** | < 0.1% | **1** |
| **AURI Gaming Mode (Headless)** | **< 32.5 MB** | **28.2 MB** | **0.0% (Suspended)** | **1** |

*See [`BENCHMARK_REPORT.md`](BENCHMARK_REPORT.md) for full execution methodology and sample logs.*

---

## ⌨️ Global Keyboard Shortcuts & System Integration

AURI registers global hotkeys and hardware media keys that work globally even while full-screen in a game:

| Shortcut | Action |
|---|---|
| `Media Play/Pause` or `Win + Alt + Space` | Toggle Play / Pause |
| `Media Next` or `Win + Alt + Right` | Next Track in Queue |
| `Media Previous` or `Win + Alt + Left` | Previous Track in Queue |
| `Win + Alt + Up` | Volume +5% |
| `Win + Alt + Down` | Volume -5% |
| `Win + Alt + M` | Cycle Operating Mode (Normal ➔ Mini ➔ HUD ➔ Headless) |
| `System Tray Double-Click` | Show / Hide Main Window |
| `System Tray Right-Click` | Dark Context Menu (Play/Pause, Next, Prev, Modes, Exit) |

---

## 🚀 Building & Running from Source

### Prerequisites
- Windows 10 or Windows 11 (x86_64)
- Rust toolchain (stable, 1.80+)
- Visual Studio C++ Build Tools (MSVC Linker & Windows SDK)

### Build Standalone Release Executable
```powershell
# Clone repository
git clone https://github.com/SuryakiranMK/AURI.git
cd AURI

# Run full test suite (25 tests)
cargo test -- --test-threads=1

# Compile optimized standalone release binary
cargo build --release

# Run benchmark suite
powershell -ExecutionPolicy Bypass -File scripts\benchmark.ps1
```

The resulting binary (`target\release\auri.exe`, ~12.5 MB) is completely self-contained with embedded Windows PE application icons, manifests, audio decoders, and GUI assets.

---

## 📁 Project Architecture & Documentation

- [`SCHEDULE.md`](SCHEDULE.md): Single source of truth master schedule and complete version history.
- [`PRODUCT_SPECIFICATION.md`](PRODUCT_SPECIFICATION.md): Core functional requirements and design specifications.
- [`ARCHITECTURE.md`](ARCHITECTURE.md): Audio engine, event dispatching, and thread decoupling architecture.
- [`PERFORMANCE_TARGETS.md`](PERFORMANCE_TARGETS.md): Resource budgets and performance benchmarks.
- [`BENCHMARK_REPORT.md`](BENCHMARK_REPORT.md): Official performance audit and profiling report.

---

## 📜 License

Project AURI is released under the [MIT License](LICENSE).
