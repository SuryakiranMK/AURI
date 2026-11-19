### AURI v2.3.0: 1-Click Discord & Web Listen Together, 2MB High-Resilience Streaming Engine, Clean Tray Exit & Dynamic Release Notes

#### Key Highlights & Changelog
1. **1-Click Discord & Web Listen Together**:
   - **Discord Rich Presence Button**: When hosting or participating in a Listen Together room, Discord presence dynamically features Button 1: `🎧 Listen Together ({count})` pointing directly to `https://suryakiranmk.github.io/AURI/join?room=<CODE>`.
   - **Modern Dark-Glass Web Redirect Portal**: Hosted at `docs/join/index.html`, automatically invokes `auri://join/<CODE>` while providing a one-click "📋 Copy Code" button, an "Open AURI Desktop" trigger, and GitHub fallback download links.
   - **Windows User-Space Protocol Registration**: Registers `auri://` under `HKCU\Software\Classes\auri` on launch with zero UAC elevation prompts.
   - **Authenticated Single-Instance Loopback IPC**: Secondary instances forward join codes or window focus to the running instance via local loopback TCP socket (`127.0.0.1:{port}`) authenticated by cryptographically secure per-session tokens.
   - **Strict Input Sanitization & In-App Security Modal**: Strictly validates room code tokens against `^[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{2}$`, preventing command injection, shell escaping, or malicious URLs. Prompts a native in-app confirmation modal (`[Join Session]` / `[Decline]`) before switching rooms to block unauthorized web hijack attempts.
   - **Instant Share Link**: Hosts get a dedicated "Copy 1-Click Discord/Web Link" button right inside the Listen Together header popup.

2. **2MB High-Resilience Audio Streaming Engine (0:31 / 1m Cutoff Fix)**:
   - **Quadrupled Buffer Chunks**: Raised chunk range requests from 512 KB to 2 MB (`2 * 1024 * 1024`), buffering over 2 minutes of Opus audio per request.
   - **Standard HTTP Range Headers**: Attached RFC-compliant `Range: bytes={from}-{to}` and `Accept-Encoding: identity` headers to prevent CDN throttles or uncompressed stream drops.
   - **Self-Healing Stream URL Refresh**: Automatically re-resolves expired Google Video playback URLs via `yt.stream()` on HTTP 403 / 416 errors without stalling or terminating playback.
   - **Extended Stall Backoff**: Raised stall retry ceiling from 8 to 16 with exponential backoff.

3. **Clean Windows Tray Background Exit**:
   - **Atomic Force-Quit Flag & Watchdog**: Added `FORCE_QUIT: AtomicBool` and an unblocking watchdog thread in `src/tray.rs` to terminate paused or hidden background event loops within 350ms.
   - **No Ghost Processes**: Guarantees zero lingering background processes and prevents instant-close bugs on subsequent app launches.

4. **Dynamic Update Dialog & Release Notes**:
   - **Automated GitHub Release Highlights**: Dynamically fetches and parses the latest release notes and bullet points directly from GitHub's Releases API instead of hardcoded changelogs.
   - **One-Click Release Notes Navigation**: Added a prominent `Release Notes ↗` button in the update dialog linking straight to the GitHub release page.

#### Checksums
```
8bffdd27fa32ecd70b62ae32a9bce09eb90f8dd2de0176f5c21f9e428555a7c7  auri.exe
```
