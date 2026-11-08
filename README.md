# AURI: Minimalist, Ultra-Lightweight, Gaming-First Desktop Music Player

<p align="center">
  <img src="assets/banner_wide.jpg" alt="AURI Banner" width="100%" />
</p>

[![Release](https://img.shields.io/badge/Release-v2.0.0-8B5CF6.svg?style=flat-square)](https://github.com/SuryakiranMK/AURI/releases/tag/v2.0.0)
[![License](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011-0078D6.svg?style=flat-square)](https://github.com/SuryakiranMK/AURI)
[![Rust](https://img.shields.io/badge/Rust-1.90%2B%20(2024)-orange.svg?style=flat-square)](https://www.rust-lang.org/)
[![Gaming RAM](https://img.shields.io/badge/Gaming%20RAM-%3C%2035%20MB-10B981.svg?style=flat-square)](https://github.com/SuryakiranMK/AURI)
[![Tests](https://img.shields.io/badge/Tests-31%20Passed-10B981.svg?style=flat-square)](https://github.com/SuryakiranMK/AURI)

**AURI** is an ultra-lightweight, native Windows desktop music player engineered specifically for PC gamers, power users, and audiophiles. Built upon a high-performance native Rust core with zero browser engines, AURI delivers instant YouTube Music streaming, audiophile loudness normalization, and an ultra-compact floating Gaming Mini Mode while consuming **under 35 MB of RAM and 0.0% background GPU**.

---

## ⚡ Key Highlights & Architecture

- **Single Lightweight Binary**: Standalone native Windows executable (~9.9 MB). Zero Electron, zero Chromium, zero webview overhead.
- **Dynamic YouTube Music InnerTube**: Native client utilizing Apple visionOS bypass tokens for full-fidelity Opus (160 kbps) and AAC (128 kbps) direct streams without throttling.
- **-7.0 LUFS Loudness Normalizer & Peak Limiter**: ITU-R BS.1770 compliant loudness normalizer and zero-allocation soft-knee peak limiter delivering uniform, punchy volume across tracks without digital clipping distortion.
- **Floating Gaming Mini Mode**: Movable, curved-edge floating HUD with always-on-top pinning, album art, transport buttons, mini seekbar, and volume control designed to hover unobtrusively over games or secondary monitors.
- **One-Click Working Set RAM Optimizer**: Instant memory purge reducing memory consumption to sub-35 MB during intense gaming sessions.
- **Windows SMTC & Native Media Keys**: Full integration with Windows 10/11 System Media Transport Controls, lock screen widgets, and keyboard media keys.
- **Apple-Grade Dark Glassmorphic Aesthetic**: Deep obsidian theme with electric violet accents (`#8B5CF6`), subtle translucent veils, and custom typography (Onest & Big Shoulders Display).

---

## 🎮 Operating Modes

### 1. Normal Mode (Full Desktop Experience)
- Full-featured sidebar navigation: **Home**, **Liked songs**, and **Recently played** with local persistence.
- Dynamic **Quick picks** and seed-based **More like...** recommendation carousels.
- Integrated search bar with instant debounced queries and `Ctrl+K` shortcut.
- Rich bottom player bar with seek scrubber, volume slider, mute toggle, queue manager, and lyrics drawer.

### 2. Gaming Mini Mode (Floating HUD)
- Compact, curved floating window (radius 16px) with sleek violet glow.
- Draggable across multi-monitor setups.
- Real-time album art, title/artist display, elapsed/duration counter, seek progress bar, transport controls, and volume slider.
- One-click restore button to return to Normal Mode.

---

## 🚀 Building & Running from Source

### Prerequisites
- Windows 10 or Windows 11 (x86_64)
- Rust toolchain (`edition = 2024`, stable 1.90+)
- Visual Studio C++ Build Tools (MSVC Linker & Windows SDK)

### Build Standalone Release Executable
```powershell
# Clone repository
git clone https://github.com/SuryakiranMK/AURI.git
cd AURI

# Run automated tests (31 unit and playback tests)
cargo test

# Compile optimized standalone release binary
cargo build --release
```

The resulting optimized executable will be located at `target\release\auri.exe`.
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
