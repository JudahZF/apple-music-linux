# Adam Rust workspace

Adam is one Rust binary, `adam`. It has two roles:

- `adam` starts the app: Adam's own window, drawn with gpui. It calls Apple Music's API for
  what it shows and starts the engine as a child process (and restarts it after a crash).
- `adam engine` starts the engine: a local HTTPS API (127.0.0.1:20025) that signs in, fetches,
  decrypts and plays through libvlc. The player (queue, state machine, quality choice) lives
  in the engine; the app sends commands and draws the state the engine publishes.

Run it from a checkout with `scripts/dev-run.sh`.

## Crates

| Crate | Purpose |
|---|---|
| `adam` | The binary. Selects the app or the engine role. |
| `adam-ui` | The UI kit, built with gpui: design tokens, components, content views over the page models, the window frame. |
| `adam-ui-shell` | The main window: start-up, frame, rail, deck, queue, Now Playing, sign-in sheet, windows. |
| `adam-ui-pages` | The page area: one view per catalog and library route. |
| `adam-ui-panels` | The Settings and Downloads pages. |
| `adam-app` | Application logic without a UI toolkit: routes, page models and loaders, library sync, engine link, account, settings, desktop integrations. `adam-ui` draws what this crate provides. |
| `adam-player` | The player controller: queue, state machine, quality choice, history, resume. Runs in the engine. |
| `adam-signin` | The Apple Music web sign-in window (a web view that captures the music user token). Not wired into the binary: Linux signs in through FairPlay; it is the base for a platform without the FairPlay libraries. Builds in `nix/signin-shell.nix`. |
| `adam-desktop` | Desktop state and services: preferences, encrypted store, themes, wallpaper lookup, links, engine supervisor. |
| `adam-integrations` | MPRIS, system tray, Discord Rich Presence, Last.fm and ListenBrainz scrobbling, Windows media controls. |
| `adam-engine` | HTTPS API server and request handlers. |
| `adam-apple` | Apple Music media provider: catalog → HLS → playable sessions. |
| `adam-playback` | Playback session lifecycle and the pre-warm scheduler. |
| `adam-export` | Track and music-video export. |
| `adam-fairplay` | Lossless path: CBCS fragment decryption pipeline and ALAC sample decryption through FairPlay. |
| `adam-streams` | AAC and music-video path: segment download, Widevine keys, sample decryption, licence provider. |
| `adam-drm` | DRM manager: FairPlay account state, sign-in, sign-out. |
| `adam-drm-native` | Loads Apple's Android FairPlay libraries through libhybris. |
| `adam-widevine` | Widevine CDM and key requests. |
| `adam-framering` | Shared-memory ring of video frames: the engine writes, the UI reads. |
| `adam-vlc` | In-process libvlc player for lossless playback. |
| `adam-dsp` | AutoEQ, binaural rendering through FFmpeg, Cavern. |
| `adam-library` | Encrypted local library cache (SQLite). |
| `adam-diskcache` | Disk cache with LRU eviction. |
| `adam-ampapi` | Apple Music catalog API client, lyrics, music-video labels. |
| `adam-hls` | HLS playlist parsing. |
| `adam-mp4` | MP4 / fragmented MP4 box parsing and rewriting. |
| `adam-core` | Pipeline types, provider traits, app directories, config, small containers. |

## Conventions

- **Async**: tokio. Cancellation is a `tokio_util::sync::CancellationToken`. Deadlines use
  `tokio::time::timeout`. Blocking foreign calls (FairPlay, libvlc) run on their own threads or
  in `spawn_blocking`, never on a runtime worker.
- **Errors**: each library crate has a `thiserror` enum. Callers match on variants, not on
  message text. `anyhow` is for the binary and the request handlers only.
- **Logging**: `tracing`. Verbose diagnostics are `debug!` or `trace!`.
- **HTTP client**: one shared `reqwest::Client` (rustls) passed in by the caller.
- **Wire formats**: JSON field names, routes, status codes and headers of the engine API are a
  contract with `adam-app`'s engine client (its contract test runs the real engine). Change
  both sides together.
- **Unsafe**: every crate has `#![forbid(unsafe_code)]`, except the FFI crates
  (`adam-drm-native`, `adam-vlc`). `adam-framering` has one `unsafe` module (the mapping). A crate that must call a Windows API uses
  `deny(unsafe_code)` on Windows with one allowed module (`adam-engine`: the peer check). Each
  `unsafe` block has a `// SAFETY:` comment.
- **Platforms**: Linux and Windows. Platform-specific code is behind `cfg`.
- **Tests**: a test pins behaviour that must not change: parsers, state machines, wire formats,
  security checks. Tests that need the network or an Apple account are `#[ignore]`.
