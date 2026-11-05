# TECHNOLOGY STACK EVALUATION: PROJECT AURI

---

## 1. Programming Language Evaluation

We evaluated three primary systems languages against our core objectives of extreme performance, low memory footprint, safe concurrency, and maintainability.

| Metric / Criteria | **Rust (Selected)** | **C++ (C++20/23)** | **C (C11/C17)** | **C# (.NET 9 NativeAOT)** |
|---|---|---|---|---|
| **Memory Control** | Deterministic, zero-cost RAII, manual allocation bounding | Full manual control | Full manual control | Managed GC overhead, hard to stay < 35MB |
| **Audio Concurrency** | Thread-safe without data races (`Send`/`Sync`), memory safe | Prone to data races & dangling pointers under heavy async load | Manual synchronization, fragile under concurrency | High overhead for lock-free audio thread |
| **Ecosystem for Audio** | Excellent (`symphonia`, `cpal`, `rodio`, `hound`) | Excellent (`miniaudio`, `portaudio`, `libmpv`) | Minimal modern codecs without heavy C libs | Interop overhead with native codecs |
| **Windows API Interop** | Excellent (`windows-rs`, `windows-sys`) | Native Windows SDK | Native Windows SDK | P/Invoke overhead |
| **Binary Size** | ~5 MB to 10 MB (stripped, LTO) | ~3 MB to 6 MB | ~1 MB to 3 MB | ~15 MB to 30 MB (AOT) |
| **Security & Safety** | Memory-safe, bounds-checked, zero buffer overflows | Vulnerable to memory corruption | Vulnerable to memory corruption | Safe runtime |

**Decision**: **Rust** is selected. It provides the low-level memory layout and performance of C/C++ while providing compiler-enforced thread safety for concurrent audio decoding and buffer management, eliminating entire classes of threading crashes and memory leaks.

---

## 2. UI Framework Evaluation

| Criteria | **Slint (Selected)** | **egui** | **Native Win32 + Direct2D** | **Tauri (WebView2)** | **Electron** |
|---|---|---|---|---|---|
| **UI Paradigm** | Retained Mode (DSL + Reactive properties) | Immediate Mode (Redraws on state/tick) | Retained / Event-driven Win32 | HTML/DOM via Chromium engine | Full Chromium + Node.js |
| **Idle RAM (Baseline)** | **~25 MB - 35 MB** (with software or femtovg backend) | **~80 MB - 110 MB** (wgpu / GL texture allocations) | **~15 MB - 25 MB** | **~180 MB - 280 MB** (multi-process WebView2) | **~350 MB - 800 MB+** |
| **Idle CPU** | **0.0%** (sleeps when no inputs/state changes) | **0.0% - 1.5%** (repaints on animations/mouse) | **0.0%** | **0.5% - 2.0%** | **1.0% - 4.0%** |
| **GPU Load when Hidden** | **0.0%** (Can destroy swapchain or halt window) | Requires explicit context suspension | **0.0%** | Persistent GPU process | Persistent GPU process |
| **Styling Capabilities** | High: Modern animations, smooth curves, dark styling | Low/Functional: Game-dev/debug look, hard to match Apple Music | Very High effort: requires thousands of lines of custom Win32 GDI+/D2D | High: CSS/Web | High: CSS/Web |
| **Process Model** | Single Process | Single Process | Single Process | Multi-Process (3+ processes) | Multi-Process (6+ processes) |

**Decision**: **Slint** is selected for the primary visual interface (with software or lightweight native rendering). It delivers the retained-mode memory efficiency of native C++, modern declarative styling capable of achieving the sleek Spotify/Apple Music aesthetic, zero-cost idling (0% CPU), and can be completely suspended in Gaming Mode.

---

## 3. Audio Engine Evaluation

| Engine | Architecture | Streaming Support | Memory Overhead | Evaluation |
|---|---|---|---|---|
| **Symphonia + CPAL / WASAPI (Selected)** | 100% Native Safe Rust, zero C runtime dependencies | Progressive chunked stream decoding | **Extremely Low** (~4 MB decode buffer) | **Top Choice**: Cleanest architecture, completely memory-bounded, native WASAPI output. |
| **Rodio** | High-level Rust audio playback library built on CPAL | Good for local files, needs adapter for HTTP streams | Low (~8 MB) | Excellent high-level wrapper, can be used atop Symphonia. |
| **libmpv** | Heavyweight C media player core | Excellent out of the box | Medium (~40 MB - 70 MB baseline) | Overkill for pure audio, adds 25MB+ C DLL dependencies. |
| **miniaudio** | Single-header C audio library | Excellent local & custom callbacks | Very Low (~3 MB) | Strong alternative, but requires C FFI in Rust. |

**Decision**: **Symphonia + direct CPAL / WASAPI backend**. Symphonia provides pure, safe Rust implementations of audio demuxers and decoders (FLAC, MP3, AAC, Opus, Vorbis, WAV) with zero external dynamic library dependencies. Audio is handed directly to Windows WASAPI for low-latency, glitch-free output.

---

## 4. Windows Integration Stack

- **Audio Playback**: WASAPI Event-Driven Shared Mode via `cpal` / `windows::Win32::Media::Audio`.
- **System Media Transport Controls (SMTC)**: `windows::Media::SystemMediaTransportControls` (WinRT/C++ integration via `windows` crate). Enables Windows volume flyout, keyboard media keys, and Windows 11 lock-screen controls.
- **Global Shortcuts**: `RegisterHotKey` for custom gaming shortcuts (e.g. `Win+Alt+Space` for play/pause).
- **Game Detection**: `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)` listening to foreground application changes, querying process identity safely without touching game address space.
- **System Tray**: Native Win32 `Shell_NotifyIconW` with minimal dark context menu.

---

## 5. Summary of Final Architectural Choices

```
+-------------------------------------------------------------+
| Core Language:       Rust (2021 Edition)                    |
| UI Framework:        Slint (Retained Mode, Custom Styling)   |
| Audio Decoder:       Symphonia (Pure Safe Native Rust)      |
| Audio Output:        WASAPI (Windows Audio Session API)     |
| Networking:          Tokio + reqwest (Async HTTP streaming) |
| Architecture:        Single-Process Decoupled Architecture  |
+-------------------------------------------------------------+
```
