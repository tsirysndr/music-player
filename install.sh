#!/bin/sh
# Install music-player from a GitHub release.
#
#   curl -fsSL https://raw.githubusercontent.com/tsirysndr/music-player/master/install.sh | sh
#
# Environment:
#   MUSIC_PLAYER_VERSION      Version to install: "0.3.0" or "v0.3.0".
#                             Default: the latest release.
#   MUSIC_PLAYER_INSTALL_DIR  Where the binary goes. Default: /usr/local/bin
#                             when writable, else ~/.local/bin.
#   MUSIC_PLAYER_DESKTOP      Set to 1 to also install the Slint desktop app
#                             (music-player-desktop), when the release ships
#                             a build for this platform.
#
# Release assets are produced by .github/workflows/release*.yml as
# music-player_<tag>_<target>.tar.gz (+ .sha256), for:
#   x86_64-apple-darwin, aarch64-apple-darwin,
#   x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu,
#   armv7-unknown-linux-gnueabihf

set -eu

REPO="tsirysndr/music-player"

say() { printf '%s\n' "$*"; }
fail() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

command -v curl >/dev/null 2>&1 || fail "curl is required"

# ── OS + architecture → release target ──────────────────────────────────────
os="$(uname -s)"
arch="$(uname -m)"
case "$os" in
  Darwin)
    case "$arch" in
      arm64 | aarch64) target="aarch64-apple-darwin" ;;
      x86_64) target="x86_64-apple-darwin" ;;
      *) fail "unsupported macOS architecture: $arch" ;;
    esac
    ;;
  Linux)
    case "$arch" in
      x86_64 | amd64) target="x86_64-unknown-linux-gnu" ;;
      aarch64 | arm64) target="aarch64-unknown-linux-gnu" ;;
      armv7l | armv7) target="armv7-unknown-linux-gnueabihf" ;;
      *) fail "unsupported Linux architecture: $arch" ;;
    esac
    ;;
  *)
    fail "unsupported OS: $os — grab a Windows build from https://github.com/$REPO/releases"
    ;;
esac

# ── Resolve the release tag ─────────────────────────────────────────────────
if [ -n "${MUSIC_PLAYER_VERSION:-}" ]; then
  case "$MUSIC_PLAYER_VERSION" in
    v*) tag="$MUSIC_PLAYER_VERSION" ;;
    *) tag="v$MUSIC_PLAYER_VERSION" ;;
  esac
else
  tag="$(
    curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" |
      grep -o '"tag_name": *"[^"]*"' |
      head -1 |
      sed 's/.*"\(v[^"]*\)"/\1/'
  )"
  [ -n "$tag" ] || fail "could not resolve the latest release of github.com/$REPO"
fi

base="https://github.com/$REPO/releases/download/$tag"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# ── Install directory ───────────────────────────────────────────────────────
if [ -n "${MUSIC_PLAYER_INSTALL_DIR:-}" ]; then
  dir="$MUSIC_PLAYER_INSTALL_DIR"
  mkdir -p "$dir"
elif [ -d /usr/local/bin ] && [ -w /usr/local/bin ]; then
  dir="/usr/local/bin"
else
  dir="$HOME/.local/bin"
  mkdir -p "$dir"
fi

# ── Download + verify + install one binary ──────────────────────────────────
# fetch <binary-name> <required: yes|no>
fetch() {
  binary="$1"
  required="$2"
  asset="${binary}_${tag}_${target}.tar.gz"

  say "installing $binary $tag for $target"
  if ! curl -fSL --progress-bar -o "$tmp/$asset" "$base/$asset"; then
    [ "$required" = "yes" ] &&
      fail "download failed: $base/$asset (does $tag ship a $target build?)"
    say "warning: $tag ships no $binary build for $target — skipping"
    return 0
  fi

  if curl -fsSL -o "$tmp/$asset.sha256" "$base/$asset.sha256" 2>/dev/null; then
    if command -v shasum >/dev/null 2>&1; then
      (cd "$tmp" && shasum -a 256 -c "$asset.sha256" >/dev/null) || fail "checksum mismatch for $asset"
    elif command -v sha256sum >/dev/null 2>&1; then
      (cd "$tmp" && sha256sum -c "$asset.sha256" >/dev/null) || fail "checksum mismatch for $asset"
    else
      say "warning: no shasum/sha256sum found — skipping checksum verification"
    fi
  else
    say "warning: no checksum published for $asset — skipping verification"
  fi

  tar -xzf "$tmp/$asset" -C "$tmp" "$binary"
  install -m 755 "$tmp/$binary" "$dir/$binary" 2>/dev/null ||
    { cp "$tmp/$binary" "$dir/$binary" && chmod 755 "$dir/$binary"; }
  say "installed $binary -> $dir/$binary"
}

fetch music-player yes
[ "${MUSIC_PLAYER_DESKTOP:-0}" = "1" ] && fetch music-player-desktop no

case ":$PATH:" in
  *":$dir:"*) ;;
  *) say "note: $dir is not on your PATH — add it, e.g.: export PATH=\"$dir:\$PATH\"" ;;
esac
if [ "$os" = "Linux" ]; then
  say "note: music-player needs ALSA at runtime — on Debian/Ubuntu: sudo apt-get install libasound2"
fi
say "run: music-player   # starts the daemon; run it again from another terminal for the TUI"
