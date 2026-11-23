# AURI v2.5.0 - Major Update: Reworked Gaming Mode, Quick HUD with Queue & Background Default

AURI v2.5.0 is a major release featuring a complete ground-up rework of Gaming Mode, a brand new compact Quick HUD with click-outside auto-dismissal, integrated queue viewing & manipulation, and background playback enabled by default.

---

### 🎮 Ground-Up Clean-Slate Gaming Mode (2MB RAM Footprint)
- **Zero Fullscreen Layering**: Completely purged legacy fullscreen DWM transparent window hacks that could cause black screens or trap mouse clicks on modern GPU drivers.
- **Pure Background Hiding**: In Gaming Mode, AURI completely hides via Win32 `SW_HIDE`, aggressively trimming memory working sets to ~2MB RAM and consuming 0% CPU/GPU.
- **Non-blocking Global Hotkey**: Background polling thread watches for `Ctrl+Shift+G` (or custom hotkey) on rising edge only, waking the UI without continuous frame loops.

---

### ⚡ Compact Quick HUD Card with Click-Outside Auto-Dismissal
- **Sleek 380×144 Floating HUD**: Displays instantly at your configured screen anchor (Top-Right, Top-Center, Top-Left, Bottom-Right, Bottom-Center).
- **Auto-Dismiss on Click-Outside**: Clicking anywhere outside the HUD onto your game, browser, or desktop automatically closes the overlay and restores game focus.
- **Top Row Controls**: Lightning badge, drag grip, direct Audio Output switcher flyout, Exit Gaming Mode (`icon::SIGN_OUT`), and Close HUD (`icon::X`).
- **Track & Scrubber**: 60×60 album art, title & artist with clean ellipsis, and interactive progress scrubber with timestamps.
- **Transport & Volume**: Play/Pause, Previous, Next, Heart/Favorite toggle, and volume slider with mute toggle.

---

### 📜 Interactive Queue Viewing & Manipulation in Quick HUD
- **Quick Queue Button**: Added `icon::QUEUE` in the Quick HUD header.
- **Dynamic Vertical Expansion**: Expands from 144px to 350px smoothly while maintaining screen margin anchoring.
- **Upcoming Track List**: Scrollable list of upcoming tracks with thumbnail artwork, title, artist, and durations.
- **Click-to-Play**: Jump directly to any upcoming track in the queue with a single click.
- **Remove Track**: Hover `[X]` button on any row removes that track from the upcoming queue.
- **Shuffle & Clear Controls**: Dedicated buttons to shuffle remaining tracks or clear upcoming queue.

---

### 🛡️ Run in Background Enabled by Default
- `close_to_tray` is now default `true` across all configurations.
- Closing AURI with the window close button (`[X]`) or `Alt+F4` keeps audio playing seamlessly in the system tray without interrupting music.

---

### 🎨 1:1 Live Interactive Settings Preview & Extreme Customization
- **Settings ➔ Gaming Mode** features an interactive 1:1 scale preview reflecting live background styles (Obsidian, Midnight, Carbon, Frosted, OLED, Accent Tint), corner radius (6–20px), widget toggles, and expandable queue drawer.
- Customizable screen positioning, hotkey rebinding, and game whitelist/blacklist.

---

### 📦 Checksums & Verification
- **auri.exe (x86_64-pc-windows-msvc)**: `1b1e86477b71d4186ed3059d799da532ed2878ebad9f09b2c80e09f8fa2fcff6`
