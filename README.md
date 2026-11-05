# AURI
### A Minimalist, Ultra-Lightweight, Gaming-First Desktop Music Player for Windows

**AURI** is an ultra-lightweight native Windows music player designed specifically for gamers, developers, and power users who need high-fidelity, uninterrupted audio playback with near-zero system resource overhead.

---

## Highlights

- **Three Operating Modes**:
  - **Normal Mode**: Complete, dark-first desktop music experience inspired by Apple Music & Spotify.
  - **Mini Mode**: Compact, draggable floating music controller with Always-on-Top support.
  - **Gaming / Performance Mode**: Signature headless mode with suspended UI, 0.0% GPU load, trimmed working set (<35 MB RAM), and native Windows SMTC/media key controls.
- **Extreme Performance**:
  - Working Set RAM: < 35 MB in Gaming Mode, < 75 MB in Normal Mode.
  - CPU Utilization: < 0.5% average on modern CPUs during playback.
  - GPU Usage: Strictly 0.0% when hidden in Gaming Mode.
  - Anti-Cheat Safe: Non-intrusive game detection without memory hooks, DLL injection, or overlays.
- **Audio Core**:
  - Pure native Rust audio decoding engine (`Symphonia`) supporting FLAC, MP3, AAC, Opus, and WAV.
  - Low-latency Windows Audio Session API (**WASAPI**) output with real-time MMCSS scheduling.
  - Memory-bounded sliding buffer pool (max 8 MB per stream).

---

## Documentation

- [Product Specification](PRODUCT_SPECIFICATION.md)
- [Architecture Design](ARCHITECTURE.md)
- [Technology Stack Evaluation](TECH_STACK_EVALUATION.md)
- [Performance Targets & Benchmark Suite](PERFORMANCE_TARGETS.md)
- [Project Roadmap](PROJECT_ROADMAP.md)
