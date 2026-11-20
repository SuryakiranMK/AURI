## What's New in v2.3.2

### 🔍 System Diagnostics & Live Subsystem Logging
- **In-App Diagnostic Logs Modal**: Added a live console viewer accessible via the new terminal icon in the top-right toolbar or with **`Ctrl + Shift + L`**.
- **1-Click Copy Diagnostic Report**: If a track stalls or fails to play on any device or network, click **"📋 Copy Logs"** to instantly generate a complete diagnostic report (audio device binding, player state, YouTube client response, CDN HTTP chunk status, and decoder probe) to easily share and resolve issues.
- **Audio Device Discovery & Loss Logging**: Automatically detects and logs WASAPI default audio device binding, device loss, and reconnects with color-coded status badges.
- **Modular & Lightweight**: Zero runtime impact on audio playback with circular 1,000-line buffer and capped disk log at `%APPDATA%\auri\data\auri.log`.
- **Top-Right Quick Access**: Immediate one-click access directly from the titlebar.
