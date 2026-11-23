#!/usr/bin/env bash
# Builds and starts Adam (the Rust app) from a checkout:
#
#   scripts/dev-run.sh [--release] [ADAM ARGS...]      e.g.  scripts/dev-run.sh adam://album/1440833098
#
# What it sets up:
# - Where Nix is installed, re-runs itself inside `nix-shell` (repository shell.nix) when not
#   already in it: the libraries gpui loads at run time, and libvlc with VLC_PLUGIN_PATH for the
#   engine. Elsewhere the distribution's libraries and VLC are used (README.md, "Build from source").
# - `cargo build -p adam`: one binary, the app and its engine (`adam engine`).
# - libhybris for the FairPlay session, next to the executable where the engine looks for it:
#   libhybris-core.so and hybris-linker/q.so. They come from $ADAM_HYBRIS_DIR if set, else
#   from drm/ in this checkout on x86-64. Git has only x86-64 builds; on aarch64 set
#   ADAM_HYBRIS_DIR to a directory with aarch64 builds of both files. Without them AAC still
#   plays and a sign-in reports the missing file.
# - The FairPlay download: fetch-fairplay-libs.sh, the lock file for this CPU and the Android
#   stubs beside the executable. On first start the app downloads Apple's libraries into
#   <config>/adam/drm/rootfs/system/lib64 (about 25 files; it needs the network).
# - Optional scratch profile: with ADAM_DEV_HOME=<dir>, HOME, the config and cache directories
#   and the single-instance socket are inside <dir>, so the run shares nothing with an
#   installed Adam (prefs, store, Apple session, library cache). Apple's FairPlay libraries
#   are then looked for in <dir>/.config/adam/drm/rootfs/system/lib64. Without ADAM_DEV_HOME
#   the real profile is used.
#
# Other variables the app reads: ADAM_ENGINE_PORT (engine port, default 20025; set another one
# when an installed Adam runs), RUST_LOG (log filter, e.g. `debug`).
set -euo pipefail

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

if [ -z "${IN_NIX_SHELL:-}" ] && command -v nix-shell >/dev/null 2>&1; then
    exec nix-shell "$repo/shell.nix" --run "$(printf '%q ' "$0" "$@")"
fi

profile=debug
cargo_flags=()
if [ "${1:-}" = "--release" ]; then
    profile=release
    cargo_flags=(--release)
    shift
fi

cargo build --manifest-path "$repo/Cargo.toml" -p adam "${cargo_flags[@]}"
bin_dir=$repo/target/$profile

hybris=${ADAM_HYBRIS_DIR:-}
if [ -z "$hybris" ] && [ "$(uname -m)" = x86_64 ]; then
    hybris=$repo/drm
fi
if [ -n "$hybris" ] && [ -f "$hybris/libhybris-core.so" ] && [ -f "$hybris/hybris-linker/q.so" ]; then
    mkdir -p "$bin_dir/hybris-linker"
    cp -u "$hybris/libhybris-core.so" "$bin_dir/"
    cp -u "$hybris/hybris-linker/q.so" "$bin_dir/hybris-linker/"
else
    echo "dev-run: no libhybris for $(uname -m) (set ADAM_HYBRIS_DIR); sign-in will not work" >&2
fi

# Apple's FairPlay libraries: the app downloads them on first start with the script beside the
# executable, as an installed Adam does (`scripts/build-package.sh` lays out the same files).
case "$(uname -m)" in
    x86_64) lock=$repo/drm/fairplay-libs.lock ;;
    aarch64) lock=$repo/packaging/fairplay-libs-aarch64.lock ;;
    *) lock="" ;;
esac
if [ -n "$lock" ]; then
    install -m755 "$repo/scripts/fetch-fairplay-libs.sh" "$bin_dir/fetch-fairplay-libs.sh"
    install -m644 "$lock" "$bin_dir/fairplay-libs.lock"
    stubs=$bin_dir/android-stubs
    if [ ! -f "$stubs/libdl.so" ]; then
        mkdir -p "$stubs"
        # shellcheck disable=SC2054 # -Wl, takes a comma
        stub_flags=(-shared -fPIC -nostdlib -O2 -Wl,--hash-style=both)
        for lib in libandroid.so libOpenSLES.so; do
            "${CC:-cc}" "${stub_flags[@]}" -Wl,-soname,"$lib" -o "$stubs/$lib" "$repo/drm/stubs/empty_stub.c"
        done
        "${CC:-cc}" "${stub_flags[@]}" -Wl,-soname,libdl.so -Wl,--version-script="$repo/drm/stubs/libdl.map" \
            -o "$stubs/libdl.so" "$repo/drm/stubs/empty_stub.c"
    fi
else
    echo "dev-run: no FairPlay library list for $(uname -m); lossless will not work" >&2
fi

if [ -n "${ADAM_DEV_HOME:-}" ]; then
    mkdir -p "$ADAM_DEV_HOME"/{.config,.cache,run}
    chmod 700 "$ADAM_DEV_HOME/run"
    # The display and the audio server stay the desktop's: name them by full path before the
    # runtime directory moves.
    desktop_runtime=${XDG_RUNTIME_DIR:-/run/user/$(id -u)}
    case ${WAYLAND_DISPLAY:-} in
        "" | /*) ;;
        *) export WAYLAND_DISPLAY=$desktop_runtime/$WAYLAND_DISPLAY ;;
    esac
    export PULSE_SERVER=${PULSE_SERVER:-unix:$desktop_runtime/pulse/native}
    export HOME=$ADAM_DEV_HOME
    export XDG_CACHE_HOME=$ADAM_DEV_HOME/.cache
    export XDG_RUNTIME_DIR=$ADAM_DEV_HOME/run
fi

exec "$bin_dir/adam" "$@"
