# AURI v2.4.3 - Auto-Unmute Device Switch, Listen Together Polish & Updater Fix

## Highlights & Improvements

### 🔄 In-App Updater Checksum Fix
- **BOM-Resilient Checksum Parsing**: Fixed `listed_checksum` in the updater to gracefully strip UTF-8 Byte Order Marks (`\u{feff}`) and whitespace, completely resolving the *"the release lists no checksum for the program"* error when updating from the in-app modal.
- Clean UTF-8 release asset verification pipeline.

### 🔊 Auto-Unmute & Natural Windows Mute Flow Restoration
- **Intelligent Endpoint Auto-Unmute**: When you select an audio endpoint (such as laptop built-in speakers while 3.5mm headphones are plugged in) that Windows has hardware-muted, AURI detects the mute state via WASAPI and automatically unmutes the endpoint for playback.
- **Polite Mute State Restoration**: When switching back to System Default or headphones (or when closing AURI), AURI automatically restores the device's original muted state so internal laptop speakers are safely re-muted without disrupting Windows or hardware jack sensing.

### 👥 Redesigned Listen Together Flyout Layout & Alignment
- **Centered Header**: Replaced arbitrary cursor offsets with a cleanly anchored header block featuring an animated circular `icon::USERS` badge and proportional typography.
- **Glassmorphism Host Card**: Sleek rounded card with a dedicated `icon::BROADCAST` badge, balanced title/subtitle hierarchy, and a crisp rounded action pill.
- **Divider Rule**: Added an elegant horizontal rule spanning across the "OR JOIN WITH CODE" section.
- **Pixel-Perfect Join Row**: Unified text input box and Join button with matched 36px heights, centered vertical text edit margins, active focus strokes, and responsive button lighting.

---

### Verification & Checksums
- Built with Rust 2024 / Profile: `release` (LTO, strip).
- All 78 unit and integration tests passing.
- See `SHA256SUMS.txt` for binary verification.
