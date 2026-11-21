## What's New in v2.3.3

### 🎧 Background Playback & System Tray Media Controls
- **Background Playback Fix**: Fixed an issue where playback would stop after a song finished when minimized to the system tray with "Close to Tray" enabled.
- **System Tray Context Menu**: Right-clicking the AURI tray icon now provides quick media controls:
  - **▶ Play / ⏸ Pause**
  - **⏭ Next Track**
  - **⏮ Previous Track**
  - **Open AURI**
  - **Quit AURI**
- **Background Song Notifications**: Shows Windows tray balloon notifications when tracks advance while minimized in the background.

### ⏭️ Seamless Auto-Skip for Unavailable Tracks
- **Dead/Removed Track Skip**: If a song is removed or taken down by YouTube (`UNPLAYABLE`, copyright restriction, age-gated, or 404/410), AURI now automatically skips to the next track in the queue rather than stalling.
- **In-App Toast Notice**: Displays a clear notification when a track is skipped:
  > `⚠️ "[Song Title]" is unavailable on YouTube. Skipping to next...`
- **Loop Protection**: Built-in guard against runaway loops if multiple consecutive tracks fail.

### 📋 Selectable Diagnostics Logs
- **Mouse Selection**: Log entries and active error banners in the Diagnostics / Log Viewer modal (`Ctrl + Shift + L`) can now be selected with the mouse to copy text easily.
- **Right-Click Copy Context Menu**: Right-click any log line to copy either the raw error message or the full timestamped log line directly to the clipboard.
