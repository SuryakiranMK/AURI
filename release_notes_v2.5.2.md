# AURI v2.5.2 - Gaming HUD Variants, Taskbar Fix, LRU Cache Quota & Layout Polish

AURI v2.5.2 is a major feature and refinement update delivering all 8 user-requested enhancements across Gaming Mode, Quick HUD customization, taskbar isolation, offline storage management with LRU pruning, auto game detection, Listen Together alignment, and in-app update/rollback tooling.

---

### 1. 🪟 Complete Taskbar Icon Dismissal
- **Enforced Tool Window Style**: Dynamically sets `WS_EX_TOOLWINDOW` and strips `WS_EX_APPWINDOW` with `SWP_FRAMECHANGED` whenever the Quick HUD is summoned or parked offscreen.
- **No Residual Taskbar Items**: Clicking outside the HUD or pressing the global hotkey (`Ctrl+Shift+G`) completely dismisses the HUD and removes any taskbar entry immediately.

---

### 2. 📍 Live Drag Position Preservation (Zero Teleporting)
- **Position Query via `GetWindowRect`**: When the user drags the HUD to any custom screen location, expanding the upcoming queue or changing overlay variants queries the live HWND rect (`rect.left > -10000`).
- **Eliminated Corner Reset**: The HUD retains its live on-screen position instead of resetting to the top-right corner.

---

### 3. 👁️ Settings Live Preview Isolation & Congestion Overhaul
- **Spacious Preview Panel**: Expanded the Settings right rail panel from a cramped 312px to up to 480px, with horizontal and vertical scrolling for larger scales and wider variants.
- **Isolated Preview Controls**: All preview buttons and sliders are rendered with isolated action queues and non-mutating hover feedback (`Sense::hover()`), preventing any accidental track skipping, volume jumps, or audio device changes while tweaking settings.
- **Extensive Customizability**: Added controls for scale multiplier (0.75x to 1.30x), background opacity (50% to 100%), corner curvature (6px to 24px), background materials (Obsidian, Slate, Frosted, OLED, Accent Tint), and widget display toggles (scrubber, artwork, volume, audio switcher, heart).

---

### 4. 🎨 6 Distinct Quick HUD Variants
- **Classic Card (380×144 / 380×350)**: Balanced 3-row layout with full playback controls, album art, interactive scrubber, volume bar, and expandable upcoming queue drawer.
- **Compact Pill (320×58 / 320×240)**: Minimalist capsule layout designed for competitive esports, featuring 42×42 art, compact titles, essential transport controls, and zero screen clutter.
- **Cyberpunk Bevel (380×144 / 380×350)**: Chamfered high-tech border with cyan neon (`#00F0FF`) and magenta (`#FF0055`) accents, tech telemetry badge (`[SYS::AURI // 160k OPUS]`), and stylized controls.
- **Streamer Banner (440×84 / 440×280)**: Broadcast layout optimized for OBS captures and ultrawide displays, featuring prominent typography, "LIVE ON AIR" badge, full scrubber, and grouped controls.
- **Mini Ticker (280×44 / 280×220)**: Ultra-thin floating ticker strip with single-line track marquee and compact buttons, ideal for placing along screen edges.
- **Glass HUD (360×136 / 360×340)**: Frosted glass styling with real-time glass grain rendering, subtle white glow border, and soft rounded pill controls.

---

### 5. 💾 Storage Cache Quota & LRU Auto-Eviction
- **Auto-Cache Toggle**: Option under Settings ➔ Storage to disable automatic downloading of streamed songs into the offline disk cache.
- **Allotted Storage Limit (Default 200 MB)**: Slider and quick presets (`200 MB`, `500 MB`, `1 GB`, `2 GB`) allowing users to define the exact maximum disk quota for cached songs.
- **Automatic LRU Deletion**: When cache quota is reached, oldest cached tracks (by last access time) are automatically purged to accommodate newer songs.
- **Live Cache Usage Bar**: Real-time progress bar showing MB used, percentage, offline track count, and a "Trim to Limit Now" manual purge action.

---

### 6. 🎮 Auto Game Detection & Blacklist Rework
- **Eliminated False Positives**: Game detection now verifies window styles (`WS_POPUP` vs `WS_CAPTION`) and multi-monitor work areas (`info.rcMonitor`), preventing maximized desktop apps (browsers, IDEs, terminals) from falsely triggering Gaming Mode.
- **Built-in System Blacklist**: Integrated out-of-the-box exclusions for Chrome, Firefox, Edge, Discord, VS Code, Windows Terminal, and explorer.
- **Custom Whitelist & Blacklist**: Added direct text input with `+ Add Exe` button and a running process picker to easily customize process lists.

---

### 7. 👥 Listen Together Settings Alignment & Layout Polish
- **Clean Responsive Cards**: Re-architected the Listen Together section in Settings with structured rows, distinct status indicators (Disconnected / Hosting / Syncing), and balanced action buttons.
- **Custom Room Codes & 1-Click Join**: Neatly laid out inputs for custom host room codes (`e.g. chill-room`) and 1-click clipboard copy.

---

### 8. 🔄 About Page Updates & Version Rollback
- **Check for Updates**: Prominent action button under Settings ➔ About to poll GitHub releases for the latest versions.
- **Release History & 1-Click Rollback**: Comprehensive card detailing past releases (`v2.5.2`, `v2.5.1`, `v2.5.0`, `v2.4.4`, `v2.4.3`, `v2.4.2`) with direct links to download prior binaries.

---

### 📦 Checksums & Verification
- **auri.exe (x86_64-pc-windows-msvc)**: `5119cb53be0bf66bfd8cc9c52449f0ffe836e82464155a3872480e8fa725fe9b`
