## What's New in v2.4.1

### 🎧 Top-Right Audio Output Source Selector
- **Dedicated Toolbar Icon Button**: Added a sound output source selector directly in the top-right toolbar next to the Equalizer.
- **Dynamic Device Glyph**: Automatically morphs its icon based on active audio output (Headphones 🎧, Speakers 🔊, Bluetooth Earbuds 📶, Monitor/HDMI 🖥️).
- **Obsidian Frosted-Glass Quick Flyout**: Click the icon to open a zero-Alt-Tab floating menu right beneath the button without leaving your current view.
  - **System Default Mode**: Seamlessly follows Windows default device changes.
  - **Direct Endpoint Binding**: Lock AURI specifically to your gaming headset or USB DAC while Windows or Discord uses other audio devices.
  - **Live Endpoint Enumeration**: Lists all detected WASAPI output hardware with device-category glyphs and active checkmark badges.
  - **1-Click Refresh**: Re-scan newly plugged-in USB DACs or Bluetooth headphones with the refresh button.
  - **Integrated Quick Volume Control**: Embedded volume slider and mute toggle right in the flyout for instant level verification upon switching devices.

### ⚡ Seamless Stream Hot-Swapping & Fallback
- **Zero-Dropout Migration**: Switching outputs mid-song preserves playback position, volume, and paused state—the track never stops or restarts.
- **Safe Disconnect Fallback**: If a wireless headset turns off or is unplugged, AURI automatically falls back to System Default without crashing or going permanently silent.
- **Preferences Persistence**: Remembers your preferred output device in `storage.json` across app launches, with matching controls in **Settings $\rightarrow$ Sound / DSP**.
