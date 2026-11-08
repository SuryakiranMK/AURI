## Project AURI v1.7.0 — Time-Synced Scrolling Lyrics, Offline Audio Cache & Queue Search Filter

### 🎤 Synchronized Scrolling Karaoke Lyrics
- **LRCLIB & YTM Timed Lyrics Integration**: Full millisecond-precision synchronized lyrics engine supporting both synced time-stamped lyrics (`[mm:ss.xx]`) and unsynced fallbacks.
- **Luminous Active Line Glow**: Active singing line highlighted with high-contrast bold white typography (`22.0pt`), accent-colored luminous pill background, and singing note dot indicator (`✦`).
- **Smooth Center Auto-Scrolling**: Intelligently auto-scrolls the active singing verse to the exact vertical center of the lyrics viewport.
- **Click-to-Seek**: Click any lyric line to instantly jump playback to that timestamp.

### ⚡ Offline Audio Cache-as-You-Play
- **Transparent Disk Audio Caching**: Songs streamed over the network are automatically and atomically cached to disk (`<storage_dir>\cache\<video_id>.opus`) in the background.
- **Zero-Latency Offline Replay**: Replaying cached songs immediately opens local audio files directly into Symphonia's media stream with 100% buffer fill, consuming **0% network bandwidth** and eliminating stream startup latency.
- **Preemptive Lookahead Caching**: Background lookahead preloads and caches the upcoming track before the current song finishes.
- **Intelligent LRU Auto-Trimming & Cap**: Automatically bounds the cache to a 1 GB maximum limit, pruning oldest files when threshold is reached.
- **Settings Cache Management**: Live cache statistics (MB cached and track count) with 1-click **[Purge]** and **[Explore]** in *Settings → General → Storage, Cache & Local Downloads*.

### 🔍 Real-Time Queue Search Filter
- **In-Queue Search Bar**: Integrated filter input (`Filter upcoming tracks...`) with clear button (`✕`) at the top of the queue panel.
- **Instant Track Jumping**: Filter matches in real time and click any result to jump immediately to that track.
- **Retained Drag-and-Drop**: Preserves smooth drag-and-drop reordering whenever the search filter is cleared.

### 🛡️ Performance & Memory
- Physical RAM footprint strictly maintained under **35 MB normal** / **< 20 MB gaming mode**.
- Standalone single executable under **10 MB** with zero runtime external dependencies.

