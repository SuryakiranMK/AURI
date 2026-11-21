# Player API Authentication & Anonymity: Architectural Rationale & Future Ideas

> **Status**: Kept Unauthenticated By Design.
> **Scope**: YouTube Music Player API streaming requests (`src/ytm.rs::player`).

---

## 1. Why Player API Calls Are Unauthenticated By Design

In AURI, while library browsing, playlists, and user account data can be authenticated when signed in, audio stream extraction via `PLAYER_API` deliberately omits `Cookie` (`SAPISIDHASH` / `SSID`) and user authorization headers:

1. **Anti-Tracking & Account Safety**:
   - Tying every automated stream chunk request to a personal Google account risks triggering anti-bot heuristics, captcha challenges, or account flagging from Google.
   - Decoupling playback streams keeps the user's primary Google account completely safe from automated activity flags.
2. **Apple VisionOS Anonymity Bypass**:
   - The Innertube client profile used for audio extraction is Apple VisionOS (`VISIONOS 1.02`). VisionOS returns raw direct Opus 251 audio streams without requiring JavaScript signature deciphering.
   - Injecting Web/Desktop browser session cookies into a VisionOS client payload can cause schema mismatches or rejected visitor sessions.
3. **Multi-User / Guest Privacy**:
   - In Listen Together (P2P sync) and guest mode, audio stream URLs can be shared or fetched independently without leaking session credentials or account tokens.

---

## 2. Future Brainstorming & Controlled Opt-In Ideas

If high-bitrate (256kbps AAC) YouTube Premium playback or age-restricted playback is explored in the future, it should **never** replace the default anonymous engine. Potential future exploration angles:

1. **Explicit Opt-In Toggle in Settings**:
   - *"Enable Premium 256kbps Streams (Requires Account Binding)"*
   - Default remains `OFF` (anonymous VisionOS streams).
   - Only when explicitly toggled on by the user does the player attach authorized session tokens.
2. **Dedicated Separate Client Profile for Authenticated Streams**:
   - Keep the anonymous VisionOS client as primary.
   - Introduce an isolated secondary client profile (e.g. `ANDROID_MUSIC` or `WEB_REMIX`) strictly used when a song explicitly returns `LOGIN_REQUIRED` or when Premium high-bitrate is requested.
3. **Token Isolation / Scoped Bearer Tokens**:
   - Investigate lightweight OAuth2 TV / device tokens rather than passing full browser cookie jars to stream endpoints.
