<div align="center">

# Adam

[![GitHub release](https://img.shields.io/github/v/release/silentone12725/apple-music-linux?include_prereleases&style=for-the-badge)](https://github.com/silentone12725/apple-music-linux/releases/latest)
[![License](https://img.shields.io/badge/license-MIT-blue?style=for-the-badge)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-linux-blue?style=for-the-badge)](https://github.com/silentone12725/apple-music-linux/releases/latest)
[![Discord](https://img.shields.io/badge/Discord-join%20the%20server-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/tY4v52KpVV)

An Apple Music client for Linux: lossless and hi-res audio, music videos, downloads, and its own native UI.

</div>

> [!IMPORTANT]
> **Disclaimer**
>
> **No Affiliation**
>
> This project and its contributors are not affiliated with, authorized by, endorsed by, or in any way officially connected with Apple Inc. or any of its subsidiaries or affiliates. This is an independent, unofficial client developed for personal use.
>
> **Trademarks**
>
> "Apple Music", "Apple", and related names, marks, and logos are registered trademarks of Apple Inc. Any use of these trademarks is for identification and reference purposes only and does not imply any association with the trademark holder.
>
> **Limitation of Liability**
>
> This application is provided "AS IS". The developers are not liable for any claim, damages, or legal consequences arising from its use. You use it entirely at your own risk. An active Apple Music subscription is required.

## Contents

[What Adam is](#what-adam-is) · [Features](#features) · [Not in this version](#not-in-this-version) · [Requirements](#requirements) · [Install](#install) · [Sign in](#sign-in) · [Opening links](#opening-links-in-adam) · [Where your data lives](#where-your-data-lives) · [Build from source](#build-from-source) · [Packaging](#packaging) · [Project structure](#project-structure) · [Changelog](#changelog) · [Community](#community) · [References](#references)

## What Adam is

Adam is one program, `adam`, written in Rust. It draws its own window with [gpui](https://www.gpui.rs) and calls Apple Music's API itself. There is no browser and no web player inside.

`adam` starts a second copy of itself, `adam engine`, as a helper process. The engine signs in, fetches and decrypts streams, and plays them through libvlc. The window sends it commands and shows what it plays.

Earlier versions (1.5.0 and before) were an Electron app around Apple's web player. That app is replaced; its sources stay in `electron/`, `engine/` and `drm/` for reference.

## Features

### Playback
- **Lossless and Hi-Res Lossless**: ALAC up to 24-bit 192 kHz, decrypted through Apple's FairPlay libraries.
- **AAC 256 kbps** for everything else, also played through libvlc.
- **Music videos** in the app window, 480p to 4K, with full screen.
- **Quality choice**: AAC, Lossless (up to 24/48) or Hi-Res Lossless (up to 24/192).
- **Queue**: play next, play later, Up Next (jump, remove), shuffle, repeat, autoplay when the queue ends, stations.
- **Lyrics**: line-synced, with word timings where Apple has them, and translations.
- **Resume on launch**: the queue and the track you were playing come back.
- **Play history** in Up Next, and plays reported to Apple so Recently Played and recommendations keep working.
- A song cache and a music-video cache, with size limits you set.

### Browse and library
- Search, New, Radio, Home, charts, and album, playlist, artist, station, curator and music-video pages.
- Your library: recently added, recently played, albums, songs, artists, playlists, music videos, library search.
- Add to and remove from your library, love and dislike, share links.
- Playlists: create, rename, delete, and add, remove and reorder tracks.
- An encrypted local copy of your library for fast pages (Settings → Library → Sync now).

### Downloads
- Save a track, an album or a playlist; music videos download as one file with sound and picture.
- Quality, artwork size, embedded artwork and lyrics, file and folder name templates, and a folder you choose.
- Optional conversion of lossless downloads to FLAC with FFmpeg.
- A Downloads page with live progress, cancel, try again and clear finished.

### Appearance
- Light, dark or follow the system.
- Pages can take their colours from the artwork.
- Theme presets and your own colours, sidebar opacity, zoom, reduce animations, hide Radio.
- Window blur, where the compositor blurs windows.

### Desktop
- **MPRIS**: your desktop's media controls and bars such as Waybar see and control Adam.
- **Media keys**, also on bare X11 window managers (Adam grabs them itself there).
- **System tray** with playback controls.
- **Mini player**, an always-on-top window.
- **Discord Rich Presence**, **Last.fm** and **ListenBrainz** scrobbling.
- **Keyboard shortcuts** you can reassign.
- **Links**: `adam://` links, Apple Music URLs and Apple's own schemes ([below](#opening-links-in-adam)). Only one Adam runs; a second start hands its link to the first.
- Wayland and X11.

## Not in this version

These existed in the Electron app, or are planned, and the Rust app does not have them:

- **Windows.** The Rust app has no Windows build or installer. Linux only.
- **Crossfade** and **Sound Check**: the settings are shown but switched off ("Coming in a later version").
- **Custom CSS themes**: dropped. Theme presets and your own colours replace them.
- **Music-video subtitles.**
- **Animated artwork** and **timed lyrics** in downloads: the settings are saved, downloads do not use them yet.
- **Gapless playback**: the next track starts after about 0.1 s of silence.
- **Dolby Atmos**, **AutoEQ** and **Cavern** binaural rendering: the code exists (`crates/adam-dsp`), playback does not use it yet.
- **Flatpak**: no longer built (see [Packaging](#packaging)).

## Requirements

- Linux on x86-64 or aarch64 (arm64), with a Wayland compositor or an X server.
- Graphics through Vulkan or OpenGL (Mesa or your GPU vendor's driver).
- PulseAudio or PipeWire (with its PulseAudio service).
- `curl` or `wget`, and `sha256sum`, to download Apple's FairPlay libraries. `zstd` for the `.run` installer.
- On aarch64: `ffmpeg` on your `PATH` for music-video downloads and FLAC conversion. The x86-64 packages bundle a minimal FFmpeg.
- An Apple Music subscription.

The packages bundle the parts of VLC that Adam uses, so you do not need VLC installed. The other libraries Adam loads (xkbcommon, xcb, Wayland, fontconfig, freetype, Vulkan or EGL, libpulse, GnuTLS, libxml2) are part of every desktop installation.

## Install

### `.run` installer (recommended)

No root needed for a user install.

```bash
chmod +x adam-x86_64.run      # adam-aarch64.run on arm64
./adam-x86_64.run
```

It installs to `~/.local/lib/adam`, puts the `adam` command in `~/.local/bin`, and adds a desktop entry and icons. Then it downloads Apple's FairPlay libraries ([below](#apples-fairplay-libraries)).

```bash
./adam-x86_64.run --system       # install to /opt/adam and /usr/local/bin (run with sudo)
./adam-x86_64.run --force        # reinstall the same build
./adam-x86_64.run --no-lossless  # skip the FairPlay download
./adam-x86_64.run --uninstall    # remove it (add --system for a system install)
./adam-x86_64.run --help
```

**Upgrading from the Electron app (1.5.0 and before):** run the new installer. It replaces the old install directory completely, so no Electron files stay behind. Your settings, Apple session and caches in `~/.config/adam` and `~/.cache/adam` are kept. Uninstalling also leaves them; delete them yourself to remove everything.

### AppImage

```bash
chmod +x Adam-*-x86_64.AppImage
./Adam-*-x86_64.AppImage
```

It uses the static AppImage runtime, so it needs no `libfuse2`. If mounting fails, add `--appimage-extract-and-run`. It does not add a desktop entry; [AppImageLauncher](https://github.com/TheAssassin/AppImageLauncher) can.

### Apple's FairPlay libraries

Adam signs in and decrypts through Apple's own Android libraries, which Adam may not redistribute. They are downloaded to your machine instead: about 65 MB, from the [wrapper project](https://github.com/WorldObservationLog/wrapper) with [a fork](https://github.com/silentone12725/wrapper) as fallback, pinned to fixed commits. Each file is checked against its SHA-256 in `fairplay-libs.lock`; a file that does not match is refused. There is one list for x86-64 and one for aarch64.

- The `.run` installer downloads them at the end of a user install.
- Otherwise Adam downloads them on its first start, then restarts its engine to use them.
- They go to `~/.config/adam/drm/rootfs/system/lib64`. To repair them, run `sh ~/.local/lib/adam/fetch-fairplay-libs.sh`.

Without them you can browse the catalog, but you cannot sign in or play.

## Sign in

There is one sign-in: your Apple Account. Open **Settings → Account → Sign in** and enter your email and password, then the six-digit code Apple sends you. If you cannot reach a trusted device, Apple can text the code to your phone number instead.

The same sign-in covers the catalog, your library, lyrics, playback, lossless and downloads. Settings → Account shows the engine, FairPlay and session status.

## Opening links in Adam

Adam opens Apple Music links from other apps and from the command line (`adam <link>`):

- **`adam://`** links always work: `adam://song/<id>`, `adam://album/<id>`, `adam://playlist/<id>`, `adam://play/<s|a|p|r>/<id>`, `adam://open/<s|a|p|r>/<id>` (add `?sf=us` for a storefront), or `adam://music.apple.com/...` with any Apple Music path.
- **Apple Music URLs** (`https://music.apple.com/...` on any Apple Music host, including old `itunes.apple.com` links and `?i=` song links) play when they point at a song, album, playlist, artist or music video, and open in the app for any other page.
- **Apple's own schemes**: `music://` `musics://` `itms://` `itmss://` `itunes://` `itals://` `italss://` `itvls://` `itvlss://` `itsradio://` `itunesradio://` `daap://` `itpc://`. **Adam claims none of them until you turn them on in Settings → Links**, one by one, because claiming a scheme takes it away from the app that handles it now. `daap://` (shared iTunes libraries) and `itpc://` (podcasts) cannot be played, so Adam shows a notice for them.

A plain `https://music.apple.com` link clicked in another app opens in your browser: the system does not let an app claim another company's website. To get from the browser into Adam, turn on `music://` and click **Open in Music** on the website, or use a browser extension that rewrites `music.apple.com` to `adam://music.apple.com`.

## Where your data lives

| What | Where |
|---|---|
| Settings, encrypted store (tokens, scrobbler keys) | `~/.config/adam` |
| Apple session | `~/.config/adam/drm/files` |
| Apple's FairPlay libraries | `~/.config/adam/drm/rootfs/system/lib64` |
| Library copy, song and music-video caches, artwork | `~/.cache/adam` |
| The app (`.run` install) | `~/.local/lib/adam`, or `/opt/adam` with `--system` |

`XDG_CONFIG_HOME` and `XDG_CACHE_HOME` move the first two. The Electron app used the same directories, so an upgrade keeps your settings and session.

## Build from source

The x86-64 libhybris and FFmpeg binaries are stored with [Git LFS](https://git-lfs.com). Install it before you clone:

```bash
git lfs install
git clone https://github.com/silentone12725/apple-music-linux
cd apple-music-linux
```

You need current stable Rust (`rustup`), a C compiler, `pkg-config`, the development files of fontconfig, freetype, xkbcommon (with its X11 part), xcb and Wayland, and libvlc with its plugins.

```bash
# Debian / Ubuntu
sudo apt install build-essential pkg-config libfontconfig-dev libfreetype-dev \
  libxkbcommon-dev libxkbcommon-x11-dev libxcb1-dev libwayland-dev libvlc-dev vlc-plugin-base
# Fedora
sudo dnf install gcc pkgconf-pkg-config fontconfig-devel freetype-devel \
  libxkbcommon-devel libxkbcommon-x11-devel libxcb-devel wayland-devel vlc-devel
# Arch
sudo pacman -S base-devel fontconfig freetype2 libxkbcommon libxkbcommon-x11 libxcb wayland vlc
```

With Nix, `shell.nix` has all of this; `scripts/dev-run.sh` enters it by itself.

Build and start:

```bash
scripts/dev-run.sh                 # debug build, then start Adam
scripts/dev-run.sh --release       # optimised build
scripts/dev-run.sh adam://album/1440833098
```

`dev-run.sh` puts libhybris beside the built executable, where the engine looks for it. The repository has libhybris for x86-64 only (`drm/`); on aarch64 set `ADAM_HYBRIS_DIR` to a directory with aarch64 builds of `libhybris-core.so` and `hybris-linker/q.so`. Without them you can browse, but not sign in.

Useful variables:

- `ADAM_DEV_HOME=<dir>`: keep settings, caches and the Apple session inside `<dir>`, apart from an installed Adam.
- `ADAM_ENGINE_PORT=<port>`: the engine's port (default 20025). Set another one while an installed Adam runs.
- `RUST_LOG=debug`: more log output.

Checks: `scripts/lint.sh` runs rustfmt, clippy and shellcheck. The crate map and the code conventions are in [crates/README.md](crates/README.md).

## Packaging

```bash
scripts/build-package.sh       # dist/adam-<arch>/: the installable directory
scripts/build-installer.sh     # dist/adam-<arch>.run
scripts/build-appimage.sh      # dist/Adam-<version>-<arch>.AppImage
```

All three build for the machine's own CPU. They copy the needed parts of the system VLC (`scripts/bundle-vlc.sh`) and check that every bundled file runs on glibc 2.35, so build release packages on Ubuntu 22.04 or another system that old. Ubuntu 22.04's VLC needs a matching minimal FFmpeg: build it with `scripts/build-vlc-ffmpeg.sh DIR` and set `ADAM_VLC_FFMPEG_LIBDIR=DIR`. On aarch64 this is always needed, and so is `ADAM_HYBRIS_DIR`. `--local` skips the portability checks, for a package that only has to run on the machine that built it (for example on NixOS).

The package layout is described at the top of `scripts/build-package.sh`; `NOTICE.md` in each package lists what it bundles and the licences. A Flatpak is not built: it would need Adam and VLC built inside the Flatpak SDK.

## Project structure

```
apple-music-linux/
├── crates/        The Rust workspace: the adam binary, its UI, app logic, engine and libraries
│                  (map in crates/README.md)
├── packaging/     Desktop entry, icons, licence notice, the aarch64 FairPlay library list
├── scripts/       dev-run, packaging, VLC and FFmpeg bundling, FairPlay download, lint
├── third_party/   Minimal FFmpeg (x86-64, LGPL, Git LFS), patched gpui_wgpu
├── nix/, shell.nix  Nix development shells
├── drm/           libhybris for x86-64, the Android stubs, the x86-64 FairPlay library list;
│                  also the old C DRM library (reference)
├── engine/        The old Go engine (reference)
├── electron/      The old Electron app (reference)
├── assets/        Tray icon; screenshots of the old Electron app
├── releases/      Release notes
└── CHANGELOG.md
```

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for what changed in each release.

## Community

Questions, bug reports, feature requests and polls live on the [Adam Discord server](https://discord.gg/tY4v52KpVV).


## References

<details>
<summary>Libraries and projects Adam builds on</summary>

**App**
- [gpui](https://www.gpui.rs) from [Zed](https://github.com/zed-industries/zed): the UI toolkit
- [Inter](https://github.com/rsms/inter): the UI font (OFL)
- [tokio](https://tokio.rs), [reqwest](https://github.com/seanmonstar/reqwest), [rustls](https://github.com/rustls/rustls), [hyper](https://hyper.rs) and [axum](https://github.com/tokio-rs/axum): async runtime, HTTP and TLS
- [rusqlite](https://github.com/rusqlite/rusqlite): the encrypted library cache
- [zbus](https://github.com/dbus2/zbus) and [ksni](https://github.com/iovxw/ksni): MPRIS and the system tray
- [RustCrypto](https://github.com/RustCrypto): AES, RSA, hashes
- [Last.fm](https://www.last.fm/api) and [ListenBrainz](https://listenbrainz.readthedocs.io): scrobbling APIs

**Playback**
- [libvlc](https://www.videolan.org/vlc/libvlc.html): audio and music-video playback, bundled as a subset
- [FFmpeg](https://ffmpeg.org): a minimal LGPL build for downloads, and the codec libraries VLC loads
- [libhybris](https://github.com/libhybris/libhybris): loads Apple's Android FairPlay libraries
- [wrapper](https://github.com/WorldObservationLog/wrapper): the source of those libraries and the FairPlay approach
- [musickit-sdk-linux](https://github.com/silentone12725/musickit-sdk-linux): the engine as a standalone SDK
- [AutoEQ](https://autoeq.app) and [Cavern](https://github.com/VoidXH/Cavern): headphone correction and Atmos rendering (not used by playback yet)

**Earlier versions**: [Electron](https://electronjs.org), [MusicKit JS](https://developer.apple.com/documentation/musickitjs), [mp4ff](https://github.com/Eyevinn/mp4ff), [go-mp4tag](https://github.com/zhaarey/go-mp4tag), [grafov/m3u8](https://github.com/grafov/m3u8), [mpris-service](https://github.com/dbkr/mpris-service), [esbuild](https://esbuild.github.io), [electron-builder](https://www.electron.build).

Contributors are listed in [CONTRIBUTORS.md](CONTRIBUTORS.md).

</details>
