# AURI v2.4.2 - Polished Audio Output Flyout & Title Bar Polish

## Highlights & Visual Polish

### 🎧 Redesigned Audio Output Source Flyout
- **Pixel-Perfect Phosphor Icons**: Fixed icon font resolution across the output selector menu, eliminating missing symbol boxes (`□`) and fallback character glitches on device rows, refresh actions, checkmarks, and volume controls.
- **Glassmorphism Device Cards**: Devices and the "System Default" target are now presented in sleek rounded cards with active accent fills, subtle hover outlines, and distinct badge checkmarks.
- **Custom Interactive Volume Scrubber**: Replaced the default slider with AURI's native interactive volume bar supporting click-and-drag scrubbing, mouse scroll wheel volume adjustments, and instant mute toggling.
- **Device Refresh**: One-click refresh button in the header quickly re-queries the Windows WASAPI audio device graph for newly connected USB DACs, Bluetooth headsets, or HDMI outputs.

### 👥 Distinct Icon for Listen Together
- Switched Listen Together toolbar icon, invite dialog, and settings cards from `icon::HEADPHONES` to `icon::USERS`.
- Eliminates visual ambiguity between multiplayer social listening and audio hardware output selection.

### 🧹 Decluttered Title Bar
- Removed the standalone update check button from the top toolbar to keep AURI's top title bar clean and distraction-free while gaming or working.

---

### Verification & Checksums
- Built with Rust 2024 / Profile: `release` (LTO, strip).
- All 77 unit and integration tests passing.
- See `SHA256SUMS.txt` for binary verification.
