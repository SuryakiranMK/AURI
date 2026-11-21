## What's New in v2.4.0

### 🎧 Audio Engine & DSP Hardening
- **Sample-Accurate Phase Tracking**: Fixed `PcmSource::current_span_len()` to track unconsumed samples, keeping Rodio resamplers in perfect phase across track and chunk transitions.
- **Mono-to-Stereo Auto-Interleaving**: Automatically interleaves mono streams into uniform stereo pairs, resolving 2x playback speed and octave shift on mono tracks.
- **Buffer Underrun Pop Elimination**: Applied a 64-frame smooth cosine decay fade from the last active audio frame during buffer underrun, eliminating digital DC pops and popping during network jitter.
- **$C^2$-Continuous Algebraic Soft Limiter**: Upgraded the peak limiter to an ultra-smooth algebraic saturation curve with zero second-derivative jump at threshold, eliminating odd-harmonic distortion and buzzing on deep sub-bass and 808s while staying completely allocation-free.
- **Impulse Click Elimination**: Automatically resets equalizer biquad filter states when transitioning between differing sample rates (e.g. 44.1 kHz AAC $\leftrightarrow$ 48.0 kHz Opus).

### 🖼️ Persistent Artwork Disk Cache
- **Instant Cover Loading**: Album artwork is now saved to disk in `%APPDATA%\auri\cache\art\{hash}.img`.
- **Zero-Network Launch**: Previously viewed covers load instantly in **<0.1 ms** across app restarts without re-fetching over the internet.
- **Automated LRU Cache Trimming**: Automatically maintains a clean 64MB storage cap, keeping disk footprint lean and tidy.

### ⚡ Graphics & Ergonomics
- **Ultra-Lightweight Glass Grain**: Replaced per-dot allocations with a single pre-allocated GPU mesh and bitwise integer hashing, slashing CPU utilization from 2–4% down to **<0.05%**.
- **Lyrics View Scroll Grace Period**: Added a 3.5-second manual scrolling grace window. When scrolling through lyrics, auto-scroll gracefully suspends so you can read freely without the viewport fighting your mouse.

### 🛡️ Cache Concurrency & Container Validation
- **Collision-Free Temp Files**: Added monotonic process-isolated temp file paths (`{video_id}.{tag}.{pid}_{count}.part`), eliminating Windows `ERROR_SHARING_VIOLATION` lock crashes between concurrent preloading and streaming threads.
- **Deep Container Validation**: Structural verification for WebM (EBML segment size), Ogg Opus (EOS trailers), and MP4 (atom bounds) ensures partial or interrupted streams are never mistaken for complete tracks.
- **Dynamic Visitor Token Refresh**: Automatically detects and updates YouTube visitor tokens whenever rotated.
