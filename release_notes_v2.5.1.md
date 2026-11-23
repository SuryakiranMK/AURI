# AURI v2.5.1 - Gaming Mode Stability, Hotkey Fixes, Window Lifecycle & Position Restoration

AURI v2.5.1 is an essential stability release resolving all reported regressions in Gaming Mode, hotkey responsiveness, Quick HUD drag interactions, and window lifecycle management.

---

### ⚡ Hotkey & Gaming Mode Fixes
- **Off-Screen Parking Architecture**: Replaced destructive Win32 `SW_HIDE` with off-screen parking (`-32000, -32000`) while preserving `WS_VISIBLE`. This prevents OpenGL `SwapBuffers` failures and ensures the winit message loop keeps pumping reliably at 0% CPU.
- **Fixed Hotkey Responsiveness**: Reduced hotkey polling sleep cycle from 30ms to 15ms so rapid key taps are never missed.
- **Focus-Loss Auto-Dismissal Hardening**: Dismissal now requires the HUD to have actively acquired focus before registering focus loss, preventing premature closures caused by Windows foreground locking over active games.

---

### 🖱️ Drag Handle & Header Hitbox Isolation
- **No Header Button Interference**: Bounded the top drag handle to stop before the right header buttons (`rect.right() - 118.0px`), eliminating drag vs. click conflicts when interacting with the Queue toggle, Audio Output picker, Exit Gaming Mode, or Close buttons.
- **Preserved Custom Position**: Expanding or collapsing the Queue panel now respects the user's custom dragged window position without teleporting the HUD back to the screen corner.
- **Safe Monitor Fallback**: Gracefully calculates center/edge monitor bounds if monitor queries temporarily fail.

---

### 🔲 Fixed Quick HUD Dimensions
- **Enforced Fixed Card Bounding**: `draw_game_hud` now renders into an explicitly allocated fixed `380×144` (or `380×350`) card rect, preventing the HUD from expanding or stretching across the entire screen during asynchronous OS resize transitions.
- **Proper Un-Maximization**: Pre-clears `WS_MAXIMIZE` using `ShowWindow(SW_RESTORE)` before applying HUD dimensions.

---

### 🖥️ Window Lifecycle, Maximization & Startup Centering
- **Guarded Size Watchdog**: Guarded the frame-0 size watchdog against un-maximizing the window when layout size is reported before the first paint pass.
- **Startup Centering**: Added explicit screen centering (`app::screen_size()`) when un-maximized, preventing Windows from defaulting to the top-left corner (`CW_USEDEFAULT`).
- **Single-Instance Restoration**: Fixed `enforce_single_instance` so secondary launches or tray restorations do not strip `WS_MAXIMIZE`.
- **Exit Gaming Mode Restoration**: Properly restores window resizability (`Resizable(true)`), decorations, and maximized state without hardcoded `(100, 100)` offsets.

---

### 📦 Checksums & Verification
- **auri.exe (x86_64-pc-windows-msvc)**: `1c79c6e5874eb53e3e601594d88e32fe6733db653b5c7961520833880269f6a5`
