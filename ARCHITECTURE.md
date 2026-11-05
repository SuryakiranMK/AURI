# ARCHITECTURE DESIGN: PROJECT AURI

---

## 1. High-Level Architectural Overview

AURI is designed with a **Single-Process, Decoupled-Subsystem Architecture**. 
To achieve sub-35 MB memory footprint and instantaneous responsiveness, AURI intentionally avoids multi-process IPC architectures (such as Chromium/Electron/Tauri WebView processes) which incur duplicate runtime overhead, inter-process serialization penalties, and substantial baseline memory costs.

```
+=============================================================================+
|                                AURI PROCESS                                 |
+=============================================================================+
|                                                                             |
|  +-----------------------+              +--------------------------------+  |
|  |     UI Subsystem      |              |         Audio Engine           |  |
|  | (Slint / Native Win32)|              |     (WASAPI + Symphonia)       |  |
|  |                       |              |                                |  |
|  | - Normal Mode Window  |              | - Real-time Audio Thread       |  |
|  | - Mini Mode Window    |              | - Lock-free Ring Buffer        |  |
|  | - Tray & Hotkeys      |              | - Codec Decoders               |  |
|  +-----------+-----------+              +---------------+----------------+  |
|              |                                          ^                   |
|              | UI Events / Commands                     | Audio Packets     |
|              v                                          |                   |
|  +-----------+------------------------------------------+----------------+  |
|  |                    Playback Core & State Controller                   |  |
|  |                                                                       |  |
|  | - Atomic Playback State (Playing, Paused, Position, Volume)           |  |
|  | - Queue Management (Shuffle, Repeat, Reorder)                         |  |
|  | - Provider Router (Local Files vs. Streaming)                         |  |
|  +-----------------------------------+-----------------------------------+  |
|                                      |                                      |
|                                      v                                      |
|  +-----------------------------------+-----------------------------------+  |
|  |                      Background Worker Subsystem                      |  |
|  |                                                                       |  |
|  | - Stream Fetcher & Progressive Caching (Sliding 8MB Window)           |  |
|  | - Bounded Artwork Cache (LRU max 16MB)                                |  |
|  | - Windows Game Detector (Hook-free WinEvent Listener)                 |  |
|  | - System Media Transport Controls (SMTC) Sync                        |  |
|  +-----------------------------------------------------------------------+  |
|                                                                             |
+=============================================================================+
```

---

## 2. Subsystem Details

### 2.1 The Real-Time Audio Engine
The audio subsystem is completely isolated from the UI rendering thread and the network fetcher.

- **Thread Priority**: Dedicated audio thread registered with Windows MMCSS (`AvSetMmThreadCharacteristicsW("Audio", ...)`), ensuring the OS scheduler guarantees timely buffer delivery even under 100% CPU game loads.
- **Output Driver**: Direct **WASAPI** (Windows Audio Session API) client in shared event-driven mode. Low buffer latency (default 10ms - 20ms buffer periods) avoids audio crackling without loading the CPU.
- **Decoder Pipeline**: Built using `symphonia`, providing pure native, safe decoders for FLAC, MP3, AAC, Vorbis, and Opus.
- **Bounded Buffer Memory**: Uses a pre-allocated circular ring buffer. Local tracks are streamed from disk in chunks; network streams use a sliding window of max 8 MB to prevent memory growth on hour-long tracks.

### 2.2 Playback Core & Atomic State
- **Decoupled Life Cycle**: The playback core continues running regardless of whether the UI is active, minimized, hidden, or transitioning modes.
- **Communication Protocol**: UI commands (`Play`, `Pause`, `Seek(u64)`, `SetVolume(f32)`, `Next`) are sent across a lock-free, zero-allocation bounded channel (e.g. `crossbeam-channel` or atomic ring).
- **State Broadcast**: Current playback position and state are updated atomically (via `AtomicU64` / `AtomicBool`) so the UI thread can read state without acquiring mutexes or blocking the audio thread.

### 2.3 UI Subsystem & Three-Mode Controller
- **Normal Mode**: Standard top-level Win32 window rendered with hardware or high-efficiency software 2D. List views are virtualized (only items on screen are instantiated).
- **Mini Mode**: Compact, borderless top-level window with custom dragging (`WM_NCHITTEST` / `HTCAPTION`) and optional `WS_EX_TOPMOST`.
- **Gaming Mode Suspension Pipeline**:
  ```
  [Game Detected or User Toggle]
               |
               v
  1. Window Hidden (ShowWindow(SW_HIDE))
  2. Render Loop Suspended (0 frame draw calls)
  3. Artwork Decoders Suspended
  4. Memory Purged: EmptyWorkingSet(GetCurrentProcess())
  5. WASAPI Audio & Global Hotkeys remain 100% active
  ```

---

## 3. Anti-Cheat Safe Game Detection Strategy

Many gamers encounter bans or game crashes caused by intrusive overlay software (e.g. third-party hook DLLs injected into games). AURI strictly forbids code injection.

### Detection Mechanism:
1. **No In-Game Hooks**: AURI does **not** call `SetWindowsHookEx(WH_CBT)`, `CreateRemoteThread`, or touch game memory.
2. **WinEvent Hook (`SetWinEventHook`)**: Listens only for `EVENT_SYSTEM_FOREGROUND` from the Windows desktop window manager.
3. **Evaluation**:
   - When the foreground window changes, AURI queries its process name via `GetWindowThreadProcessId` + `QueryFullProcessImageNameW`.
   - Checks process name against a configurable list of known game executables or checks for exclusive fullscreen mode (`ChangeDisplaySettings` / window rect matching display dimensions).
   - If a game is active: Transitions AURI into Gaming Mode.
   - When the game exits or loses focus: Prompts or auto-restores Normal/Mini mode according to user settings.

---

## 4. Provider-Independent Architecture & Streaming Analysis

### 4.1 Modular Provider Abstraction
The playback core is decoupled from content providers via an asynchronous Rust trait:

```
+-------------------------------------------------------------+
|                      MusicProvider Trait                    |
+-------------------------------------------------------------+
|  async fn search(&self, query: &str) -> Result<Vec<Track>>; |
|  async fn resolve_stream(&self, id: &str) -> Result<Stream>;|
|  async fn get_metadata(&self, id: &str) -> Result<Metadata>;|
+-------------------------------------------------------------+
```

- **Sliding-Window Bounded Buffering**:
  For authorized network streams, chunks are buffered via an **8 MB sliding memory window**. Prefetching is strictly bounded: as the playhead advances, older samples are reclaimed to ensure memory consumption remains flat regardless of playback duration.
- **Local File Provider**:
  Zero-latency native filesystem reader supporting FLAC, MP3, AAC, Opus, and WAV. Metadata is indexed into an in-memory lightweight catalog, avoiding persistent database overhead.

### 4.2 YouTube Music Integration Limitations (Documented)
Following thorough evaluation of official YouTube developer terms and APIs:
1. **Terms of Service Restrictions**: YouTube explicitly prohibits audio-only stream extraction, background playback (screen-off or hidden window), and separation of audio from video canvas.
2. **Absence of Official Audio API**: Google provides no authorized raw audio streaming SDK for YouTube Music.
3. **Rejection of Undocumented Innertube Extraction**: Scraping undocumented endpoints violates Terms of Service, risks IP throttling, and requires continuous maintenance against token obfuscation changes.
4. **Architectural Stance**: AURI will **not** compromise its lightweight native architecture or violate compliance to force YouTube Music integration. Authorized streaming endpoints (e.g., Subsonic / Navidrome / Jellyfin personal servers, authorized web streams) will utilize the 8 MB sliding window buffer.

---

## 5. Threading & Concurrency Model

AURI strictly bounds its thread pool to minimize context switching overhead:

| Thread | Role | Priority |
|---|---|---|
| **Thread 1: UI / Main** | Win32 message loop, Slint/Direct2D rendering, event handling | Normal |
| **Thread 2: Audio Worker** | WASAPI feeder, buffer mixing, Symphonia decoding | MMCSS (Real-time Audio) |
| **Thread 3: Async I/O** | Tokio lightweight runtime for HTTP streaming and file I/O | Below Normal |
| **Thread 4: WinEvent Hook** | System tray, media keys, game detection listener | Normal |

Total persistent threads: **4 threads**. Compare this with 35–60 threads typical of Electron/Chromium-based players.

