# AURI v2.4.4 - Native Discord Join, Custom Room Codes & Listen Together Polish

## Highlights & Improvements

### 🎮 Native Discord Rich Presence Party Join (Zero-Click In-App)
- **Full-Duplex Discord IPC**: Upgraded the Discord IPC pipe in `src/discord.rs` to a full-duplex reader/writer architecture. Now handles framing opcodes, event subscriptions (`SUBSCRIBE` for `ACTIVITY_JOIN` and `ACTIVITY_JOIN_REQUEST`), and ping/pong responses cleanly.
- **Native Party Secrets**: Populated `party.id` and `secrets.join` directly in the Discord Rich Presence activity payload.
- **Zero-Browser In-App Joining**: When friends click "Join" on your Discord profile or channel card, Discord dispatches `ACTIVITY_JOIN` over the local IPC pipe, opening the join confirmation modal in AURI with **zero web browser popups or tabs**.
- **Cold-Start Protocol Registration**: Registered `discord-1558230893519306802://` alongside `auri://` in the Windows Registry (`HKCU\Software\Classes`), allowing Discord to launch AURI directly into a session even if closed.
- **Dead Link Removal**: Completely excised the obsolete and non-functional GitHub Pages redirect links from both Discord buttons and the host flyout.

### 🏷️ Custom Room Code Configuration
- **Custom Room Code Setting**: Added a new setting under **Settings ➔ Listen Together** allowing users to configure a permanent custom room name/code when hosting.
- **Flexible Room Validation**: Updated `is_valid_room_code` in `src/sync.rs` to support both standard Crockford codes (`XXXX-XXXX-XX`) and custom room names (3–32 characters, ASCII alphanumeric, hyphens, and underscores).
- **Hashed Relay Topics**: Topic generation continues to hash room codes via SHA-256 (`topic_for()`), guaranteeing that custom room names are never exposed in plaintext on the relay network.
- **Host Integration**: Both the top-bar flyout and Settings "Host" buttons automatically prioritize your custom room code if configured, falling back to random codes if blank.

### 📋 Smart "Clip-to-Join" Clipboard Auto-Detection
- **Focus & Flyout Detection**: When AURI window gains focus or when the Listen Together flyout is opened, AURI checks the system clipboard via `arboard`.
- **Intelligent Pre-Filling**: If a valid room code or invite URL (`auri://...`, `discord-...://...`) is detected in the clipboard, AURI automatically prefills the flyout join input and notifies the user with a 1-click prompt.
- **Deduplication**: Automatically suppresses codes matching the active session or previously dismissed codes.

### ✨ Listen Together Flyout Polish
- **Subtle Placeholder Text**: Replaced the previous bright white placeholder with a dimmed, subtle opacity `"Enter join code..."` hint text (`veil(0.32)`), making it distinct from typed text.
- **Streamlined Actions**: Polished the host view with clean room code copying and active listener counts.

---

### Verification & Checksums
- Built with Rust 2024 / Profile: `release` (LTO, strip).
- All 81 unit and integration tests passing.
- See `SHA256SUMS.txt` for binary verification.
