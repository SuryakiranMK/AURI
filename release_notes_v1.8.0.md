## Project AURI v1.8.0 — Retractable Orb Mini Mode, Quick Picks Isolation, Zero-Spike Framerate Engine, Taskbar Controls & Listen Together GitHub Sync

### 🪐 Capsule Pill Mini Mode: Retractable Orb Sub-Mode
- **Circular Orb Resting State**: In the new toggleable sub-mode (*Settings → Mini Mode → Retractable Orb Sub-Mode*), Capsule Pill collapses into a compact 56×56 circular floating orb showing only the rotating vinyl album artwork.
- **Hover Expansion**: Moving the cursor over the orb expands it from the right into the full 320×56 pill with track details, centered transport buttons, and volume control.
- **5-Second Auto-Retraction**: Once the cursor leaves the player, it stays open for 5 seconds before smoothly contracting back into the spinning vinyl circular orb.

### 🎯 Quick Picks Isolation
- **Targeted Container Box**: Strictly isolated the elevated card container module to the "Quick picks" section.
- **Clean Homepage Shelves**: Preserved clean, unboxed compact grids for all other recommendation shelves ("More like...", etc.).

### ⚡ Zero CPU/GPU Spikes Framerate Engine
- **Throttled Animations**: Framerate-capped rotating vinyl from 60 FPS to 30 FPS, and shimmer wave animation to 20 FPS.
- **Animation Suspension**: Completely halts repainting (0 extra redraws) when the player is paused, minimized to the taskbar, or when Performance Mode is active, eliminating the ~6% CPU/GPU spikes observed during idle.

### 📑 Queue Panel Dock & Guaranteed Startup Visibility
- **Default Docking**: Right queue panel remains permanently docked by default without collapsible arrows or edge tabs unless explicitly enabled in *Settings → Appearance*.
- **Guaranteed Startup Visibility**: The right panel is forced open and visible on every application launch regardless of starting window width.

### 🪟 Windows Taskbar Hover Preview Controls
- **TaskbarButtonCreated Handling**: Correctly initializes thumbnail buttons upon taskbar button registration.
- **DWORD-Aligned Transparency Mask**: Standardized 1-bit transparency mask and robust WM_COMMAND dispatch for Windows 10 & 11 taskbar thumbnail previews.

### 🎧 AURI Listen Together & GitHub Sync (`src/sync.rs`)
- **Instant Room Codes**: Host generates a shareable 6-letter room code (e.g., `AURI-1234`) with zero setup needed.
- **Synchronized Playback**: Friends listening along automatically sync tracks, play/pause states, and auto-seek if playback drifts beyond 2.5 seconds.
- **GitHub Gist Sync**: Optional GitHub Personal Access Token (`ghp_...`) and Gist ID support for persistent private room synchronization.
- **Topbar Live Indicator**: Glowing green `🎧 Sync: [ROOM]` pill in the top bar to manage or leave the session.

### 🛡️ Performance Constraints
- Standalone single executable under **10 MB** (9.89 MB).
- **0.0% idle CPU and GPU load**.
- Physical RAM strictly under **35 MB normal** / **< 20 MB gaming mode**.

