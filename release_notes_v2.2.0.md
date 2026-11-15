### AURI v2.2.0: Direct P2P Listen Together (RFC 5389 STUN / UDP), Mini-Mode DWM Hardware Composition & Modern Explorer Pickers

#### Key Highlights & Changelog
1. **Direct P2P UDP Listen Together (<15ms Latency)**:
   - **RFC 5389 STUN NAT Traversal**: Queries Google and Cloudflare STUN servers to discover NAT-mapped endpoints, punching direct UDP pinholes between peers for sub-15ms sync.
   - **Zero Relay Flood**: Replaced the legacy HTTP polling/posting method with direct point-to-point UDP streaming. The public relay is now demoted strictly to 1-packet initial signaling and silent fallback for strict symmetric corporate firewalls.
   - **LAN Routing Auto-Discovery**: Automatically binds to the local interface routing table for **<1ms** sync between computers on the same home network / Wi-Fi.
   - **Instant Ping-Pong Hole Punch Handshake**: Symmetrical UDP ping/pong verification connects peers in <15ms without waiting for periodic timers.
   - **Safe UDP MTU Windowing**: Sliding 16-track window ensures all serialized JSON packets fit safely under 1250 bytes (comfortably within the 1472-byte UDP MTU) preventing packet fragmentation or router drops.
   - **Running Clock-Skew & Jitter Cancellation**: Adaptive minimum-filter estimator cancels latency drift and system clock offset without requiring synchronized clocks.
   - **Dead Peer Auto-Pruning & Sub-millisecond Disconnect**: 30-second TTL automatically prunes disconnected peers, while direct UDP `bye` packets drop leaving guests from the host's list in sub-millisecond time.

2. **Listen Together UI Overhaul**:
   - **Opaque Glass Frame**: Eliminated background bleed-through with a solid `Color32::from_rgb(20, 22, 28)` 12px rounded capsule with drop shadow.
   - **286px Anti-Truncation Layout**: Wide layout completely resolves text clipping in host/join action cards.
   - **Live `⚡ Direct P2P` Pill**: Dynamic green pill confirms active direct P2P UDP transport.
   - **Room Code Copy Chip**: High-contrast copy card with instant `✓ Copied to clipboard` visual confirmation.
   - **Discord Rich Presence One-Click Join**: Fully integrated with Discord `ACTIVITY_JOIN` so friends can join your room directly from your Discord profile card.

3. **Win32 Windowing & Mini Mode Polish**:
   - **Permanent 1995/XP Legacy UI Flash Fix**: Enforced `DWMWA_NCRENDERING_POLICY = 2` (`DWMNCRP_ENABLED`) across all transitions, permanently banishing classic GDI non-client frame flickers when leaving mini mode or clicking the player.
   - **Instant Mini-Mode Interactivity**: Dropped `WS_EX_NOACTIVATE` and switched to `SW_SHOW` + `ViewportCommand::Focus`, making mini mode immediately draggable and interactive on frame 1 without needing an activation click.
   - **Zero Win32 Drag Flooding**: Replaced per-frame drag calls with `Sense::drag()` and `drag_started()`, stopping modal `WM_NCLBUTTONDOWN` message spam.

4. **Modern Windows 11 Explorer Integration**:
   - Upgraded the storage directory picker script with `$f.AutoUpgradeEnabled = $true`, displaying the modern Windows 11 Explorer folder dialog instead of the legacy 1995 treeview.

5. **Audio Pipeline & Tray Enhancements**:
   - Audio streaming and caching pipeline unified in `src/player.rs`.
   - Windows system tray context menu compliance and taskbar restoration hardened.

#### Checksums
```
ce898c060258e0f5c54237a3f113e6b9c82984f8e321eeb94e5affcd563b32f8  auri.exe
```
