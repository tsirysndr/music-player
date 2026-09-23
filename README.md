## Music Player (written in Rust)

<p>
  <a href="https://flakehub.com/flake/tsirysndr/music-player" target="_blank">
    <img src="https://img.shields.io/endpoint?url=https://flakehub.com/f/tsirysndr/music-player/badge" />
  </a>
  <a href="LICENSE" target="_blank">
    <img alt="License: MIT" src="https://img.shields.io/badge/License-MIT-blue.svg" />
  </a>
  <a href="https://buf.build/tsiry/musicserverapis/docs/main:music.v1alpha1">
    <img src="https://img.shields.io/badge/apidocs-yes-cyan.svg" />
  </a>
  <a href="https://crates.io/crates/music-player" target="_blank">
    <img src="https://img.shields.io/crates/v/music-player.svg" />
  </a>
  <a href="https://codecov.io/gh/tsirysndr/music-player" target="_blank">
    <img src="https://codecov.io/gh/tsirysndr/music-player/branch/chore/tests/graph/badge.svg?token=" />
  </a>
  <a href="https://github.com/tsirysndr/music-player/releases" target="_blank">
    <img alt="GitHub all releases" src="https://img.shields.io/github/downloads/tsirysndr/music-player/total">
   </a>
  <a href="https://github.com/tsirysndr/music-player/actions/workflows/ci.yml" target="_blank">
    <img src="https://github.com/tsirysndr/music-player/actions/workflows/ci.yml/badge.svg" />
  </a>
   <a href="https://github.com/tsirysndr/music-player/actions/workflows/release.yml" target="_blank">
    <img alt="release" src="https://github.com/tsirysndr/music-player/actions/workflows/release.yml/badge.svg" />
  </a>
  <a href="https://github.com/tsirysndr/music-player/actions/workflows/rust-clippy.yml" target="_blank">
    <img alt="rust-clippy" src="https://github.com/tsirysndr/music-player/actions/workflows/rust-clippy.yml/badge.svg?branch=master" />
  </a>
  <a href="https://feat-webui--6343b23f7b47cd6de45a5849.chromatic.com/" target="_blank">
  <img src="https://raw.githubusercontent.com/storybooks/brand/master/badge/badge-storybook.svg">
  </a>
  <a href="https://discord.gg/reJ9gUNsMV" target="_blank">
    <img alt="discord-server" src="https://img.shields.io/discord/1026789060515205181?label=discord&logo=discord&color=5865F2">
  </a>
</p>

<p style="margin-top: 20px; margin-bottom: 50px;">
<img src="./.github/assets/preview-slint.png" width="100%" />
</p>

<p style="margin-top: 20px; margin-bottom: 50px;">
<img src="./preview-tui.png" width="100%" />
</p>

An extensible music player daemon, server and client, written in Rust — like [mpd](https://github.com/MusicPlayerDaemon/MPD) or [Mopidy](https://github.com/mopidy/mopidy).

Audio decoding and playback are powered by the [Rockbox](https://www.rockbox.org) firmware's battle-tested engine, via the [rockbox-playback](https://crates.io/crates/rockbox-playback), [rockbox-dsp](https://crates.io/crates/rockbox-dsp) and [rockbox-metadata](https://crates.io/crates/rockbox-metadata) crates: 40+ audio formats, gapless-grade buffering, EQ/crossfade/ReplayGain DSP, and native HTTP streaming. The daemon indexes your library into SQLite (with FTS5 full-text search) and exposes it over gRPC, GraphQL and a web UI — controllable from the terminal UI, the browser, or the desktop app.

> [!NOTE]
> **Looking for more?**
> If you're interested in this project, you might want to check out [rockboxd](https://github.com/tsirysndr/rockboxd),
> a music player daemon built on the [Rockbox](https://www.rockbox.org) Open Source Firmware. It offers advanced audio playback
> features, bringing the best of Rockbox to modern platforms with the power of [Zig](https://ziglang.org/) and [Rust](https://www.rust-lang.org).
>

<p style="margin-top: 20px; margin-bottom: 20px;">
  <img src="./preview.svg" width="800" />
</p>

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Start the server](#start-the-server)
- [Usage](#usage)
- [Terminal UI](#terminal-ui)
- [Web UI & Desktop](#web-ui--desktop)
- [GraphQL API](#graphql-api)
- [Search](#search)
- [Configuration](#configuration)
  - [Audio output](#audio-output)
  - [Track cache](#track-cache)
  - [Subsonic / Navidrome & Jellyfin](#subsonic--navidrome--jellyfin)
  - [Rocksky scrobbling](#rocksky-scrobbling)
  - [AT Protocol sync](#at-protocol-sync)
- [Audio analysis & auto-DJ](#audio-analysis--auto-dj)
- [Acoustic fingerprints & missing tags](#acoustic-fingerprints--missing-tags)
- [AI agents (MCP)](#ai-agents-mcp)
- [Listening analytics](#listening-analytics)
- [Casting](#casting)
- [Star History](#star-history)

## Features

- 🎵 **Rockbox playback engine** — 40+ formats (MP3, FLAC, Vorbis, Opus, MP4/AAC/ALAC, WavPack, APE, WMA, chiptunes, …) with the Rockbox DSP chain (EQ presets, crossfade, ReplayGain)
- 🔎 **Instant full-text search** backed by SQLite FTS5, kept in sync automatically by database triggers
- 🖥️ **Terminal UI** (ratatui) with an fzf-style fuzzy finder, neovim-inspired status line and `?` help overlay
- 🌐 **Web UI** (React 18 + Tailwind v4 + HeroUI) — the same skins, fonts and components as the Slint desktop app, responsive down to a phone
- 🖱️ **Desktop app** — a skinnable [Slint](https://slint.dev) app with an embedded daemon
- 📡 **gRPC + GraphQL APIs** (tonic 0.14, grpc-web enabled) for building your own clients
- ☁️ **Browse & stream from Subsonic/Navidrome and Jellyfin servers**
- 📻 **Cast to Chromecast and UPnP/DLNA renderers**, or control another music-player daemon
- 🎧 **Rocksky scrobbling** — scrobble your plays to [Rocksky](https://rocksky.app) on the [AT Protocol](https://atproto.com)
- 📻 **Internet radio** — search and browse thousands of stations (Radio Browser + TuneIn), bookmark them, with a fullscreen now-playing player
- 🛰️ **AT Protocol sync** — radio bookmarks and liked songs restored from your atproto repo, written back to your PDS, and kept live over Jetstream
- 🔍 **Acoustic fingerprinting** (Chromaprint) — every file identified by its audio, and badly tagged ones named from AcoustID/MusicBrainz the way Picard does it
- 📊 **Listening analytics** (DuckDB) — import years of Spotify, Last.fm and Rocksky history, deduplicated across sources, then ask it about sessions, skips, taste drift and what actually follows what
- 🔌 Flexible **audio output**: system device (cpal), stdout, FIFO, Unix or TCP socket

## Installation

With curl (downloads the prebuilt binary for your platform from GitHub releases):

```bash
curl -fsSL https://raw.githubusercontent.com/tsirysndr/music-player/master/install.sh | sh
# pin a specific release:
curl -fsSL https://raw.githubusercontent.com/tsirysndr/music-player/master/install.sh | MUSIC_PLAYER_VERSION=v0.4.4 sh
# also install the Slint desktop app (music-player-desktop):
curl -fsSL https://raw.githubusercontent.com/tsirysndr/music-player/master/install.sh | MUSIC_PLAYER_DESKTOP=1 sh
```

Using [npm](https://www.npmjs.com/) (downloads the prebuilt binary from GitHub releases):

```bash
npm install -g @tsiry/music-player   # or: npx @tsiry/music-player
# pin a specific release: MUSIC_PLAYER_VERSION=v0.4.4 npx @tsiry/music-player
```

Using [Homebrew](https://brew.sh/) (macOS/Linux):

```bash
brew install tsirysndr/tap/musicplayer
```

On macOS, the desktop app is also available as a cask:

```bash
brew install --cask tsirysndr/tap/musicplayer
```

> [!NOTE]
> The macOS app is not code-signed or notarized, so Gatekeeper blocks it on
> first launch. Open it once, then go to **System Settings → Privacy & Security**
> and click **Open Anyway** next to the message about Music Player. Confirm with
> **Open** in the dialog that follows — macOS remembers the choice afterwards.

On Debian, Ubuntu, and other APT-based systems (`amd64` and `arm64`):

```bash
echo "deb [trusted=yes] https://apt.fury.io/tsiry/ /" \
  | sudo tee /etc/apt/sources.list.d/music-player.list
sudo apt-get update
sudo apt-get install music-player
```

On Fedora, RHEL, Rocky Linux, AlmaLinux, and other DNF-based systems
(`x86_64` and `aarch64`):

```bash
sudo tee /etc/yum.repos.d/music-player.repo <<'EOF'
[music-player]
name=Music Player
baseurl=https://yum.fury.io/tsiry/
enabled=1
gpgcheck=0
EOF
sudo dnf install music-player
```

The Debian and RPM packages include both `music-player` and the Slint desktop
application, `music-player-desktop`. Launch the desktop app from your
application menu or run `music-player-desktop`.

On Arch Linux and derivatives, from the
[AUR](https://aur.archlinux.org/packages/music-player-bin):

```bash
yay -S music-player-bin   # or: paru -S music-player-bin
```

Using [Nix](https://nixos.org/) (macOS/Linux):

```bash
cachix use tsirysndr
nix profile install --experimental-features "nix-command flakes" github:tsirysndr/music-player
```

Or download the latest release for your platform [here](https://github.com/tsirysndr/music-player/releases).

### Compiling from source

Without Nix:

```bash
# Install dependencies
brew install protobuf # macOS
sudo apt-get install -y libasound2-dev protobuf-compiler # Ubuntu/Debian
choco install protoc # Windows using Chocolatey Package Manager
# Compile
git clone https://github.com/tsirysndr/music-player.git
cd music-player
./scripts/fetch-duckdb.sh # prebuilt static DuckDB for the analytics engine
cd webui/musicplayer
nvm install # install node version specified in .nvmrc (optional on windows)
bun install && bun run build # build webui
cd ../..
cargo install --path .
```

`fetch-duckdb.sh` downloads the static libraries from DuckDB's own GitHub
release into `vendor/`, so nothing has to compile the DuckDB C++ sources. It
must run before `cargo`, not from a build script: `libduckdb-sys` resolves the
library when its own crate compiles, and Cargo gives no way to order another
crate's build script ahead of that.

With Nix:

```bash
git clone https://github.com/tsirysndr/music-player.git
cd music-player
nix develop --experimental-features "nix-command flakes"
./scripts/fetch-duckdb.sh
cd webui/musicplayer
bun install && bun run build # build webui
cd ../..
cargo install --path .
```

## 📦 Downloads

<!-- download start -->

**Latest (Desktop app, v0.4.4):**

- `macOS (.dmg)`: arm64: [Music_Player_v0.4.4_aarch64.dmg](https://github.com/tsirysndr/music-player/releases/download/v0.4.4/Music_Player_v0.4.4_aarch64.dmg) intel: [Music_Player_v0.4.4_x64.dmg](https://github.com/tsirysndr/music-player/releases/download/v0.4.4/Music_Player_v0.4.4_x64.dmg)
- `macOS (.tar.gz)`: arm64: [music-player-desktop_v0.4.4_aarch64-apple-darwin.tar.gz](https://github.com/tsirysndr/music-player/releases/download/v0.4.4/music-player-desktop_v0.4.4_aarch64-apple-darwin.tar.gz) intel: [music-player-desktop_v0.4.4_x86_64-apple-darwin.tar.gz](https://github.com/tsirysndr/music-player/releases/download/v0.4.4/music-player-desktop_v0.4.4_x86_64-apple-darwin.tar.gz)
- `Linux`: amd64: [music-player-desktop_v0.4.4_x86_64-unknown-linux-gnu.tar.gz](https://github.com/tsirysndr/music-player/releases/download/v0.4.4/music-player-desktop_v0.4.4_x86_64-unknown-linux-gnu.tar.gz) arm64: [music-player-desktop_v0.4.4_aarch64-unknown-linux-gnu.tar.gz](https://github.com/tsirysndr/music-player/releases/download/v0.4.4/music-player-desktop_v0.4.4_aarch64-unknown-linux-gnu.tar.gz)

**Latest (CLI, v0.4.4):**

- `macOS`: arm64: [music-player_v0.4.4_aarch64-apple-darwin.tar.gz](https://github.com/tsirysndr/music-player/releases/download/v0.4.4/music-player_v0.4.4_aarch64-apple-darwin.tar.gz) intel: [music-player_v0.4.4_x86_64-apple-darwin.tar.gz](https://github.com/tsirysndr/music-player/releases/download/v0.4.4/music-player_v0.4.4_x86_64-apple-darwin.tar.gz)
- `Linux`: amd64: [music-player_v0.4.4_x86_64-unknown-linux-gnu.tar.gz](https://github.com/tsirysndr/music-player/releases/download/v0.4.4/music-player_v0.4.4_x86_64-unknown-linux-gnu.tar.gz) arm64: [music-player_v0.4.4_aarch64-unknown-linux-gnu.tar.gz](https://github.com/tsirysndr/music-player/releases/download/v0.4.4/music-player_v0.4.4_aarch64-unknown-linux-gnu.tar.gz)

**Latest (Linux packages, v0.4.4 — CLI + desktop app):**

- `Debian/Ubuntu`: amd64: [music-player_0.4.4_amd64.deb](https://github.com/tsirysndr/music-player/releases/download/v0.4.4/music-player_0.4.4_amd64.deb) arm64: [music-player_0.4.4_arm64.deb](https://github.com/tsirysndr/music-player/releases/download/v0.4.4/music-player_0.4.4_arm64.deb)
- `Fedora/RHEL`: x86_64: [music-player-0.4.4-1.x86_64.rpm](https://github.com/tsirysndr/music-player/releases/download/v0.4.4/music-player-0.4.4-1.x86_64.rpm) aarch64: [music-player-0.4.4-1.aarch64.rpm](https://github.com/tsirysndr/music-player/releases/download/v0.4.4/music-player-0.4.4-1.aarch64.rpm)

Every asset ships a matching `.sha256` file next to it.

[Other version...](https://github.com/tsirysndr/music-player/releases)


## Start the server

```bash
music-player
```

The daemon scans your music directory (`$HOME/Music` by default), serves gRPC on `:5051`, WebSocket events on `:5052`, and the web UI + GraphQL on `:5053`.

The same binary is both the server and the client — the first instance starts the daemon, and any later invocation detects it and talks to it:

```bash
# terminal 1 — becomes the server
music-player

# terminal 2 — detects the daemon, acts as a client
music-player scan
```

## Usage

```
USAGE:
    music-player [SUBCOMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    albums      List all albums
    artists     List all artists
    help        Print this message or the help of the given subcommand(s)
    next        Play the next song
    pause       Pause the current song
    open        Open audio file
    playlist    Manage playlists
    prev        Play the previous song
    queue       Manage the queue
    play        Resume the current song
    scan        Scan music library: $HOME/Music
    search      Search for a song, album, artist or playlist
    stop        Stop the current song
    tracks      List all tracks
```

## Terminal UI

Run `music-player` while a daemon is running (or connect to a remote one with `music-player connect -s <host>`) to open the TUI. It ships with:

- an **fzf-style fuzzy finder** (`/`) over tracks, albums and artists, ranked as you type with match highlighting
- a **neovim-inspired status line**: mode indicator, now playing, position/duration, volume, and the connected server
- a context-sensitive **keybinding hint bar**, and a full help overlay on `?`

Main keys:

| Key         | Action                                                  |
| ----------- | ------------------------------------------------------- |
| `?`         | Help overlay with all keybindings                       |
| `/`         | Fuzzy search (Tab switches Tracks/Albums/Artists scope) |
| `Space`     | Play / pause                                            |
| `n` / `p`   | Next / previous track                                   |
| `<` / `>`   | Seek −5s / +5s                                          |
| `+` / `-`   | Volume up / down (`=` / `_` work unshifted)             |
| `m`         | Mute / unmute                                           |
| `C`         | Switch which server the library is read from            |
| `P`         | Choose where the audio comes out (Play to)              |
| `z`         | Add selected track to the queue                         |
| `q` / `Esc` | Back / quit                                             |

## Web UI & Desktop

The web UI is served by the daemon at [http://localhost:5053](http://localhost:5053) — React 19, TanStack Query and Jotai, with live playback position (GraphQL subscriptions) and a seekable progress bar.

It is a port of the Slint desktop UI rather than a separate design: the same
five skins, the same two fonts, and React versions of the desktop's own
components down to the VFD readout, the LED level meters and the rotary volume
knob. Pick a skin from the sidebar, as on the desktop. Below a laptop width the
sidebar becomes a bottom tab bar and the player bar drops to the essentials.
See [`webui/musicplayer/README.md`](webui/musicplayer/README.md) for how the
design system is put together.

Both the Slint desktop and the web UI answer to the same keys:

| Key       | Action                                        |
| --------- | --------------------------------------------- |
| `/`       | Global search (`⌘K` / `Ctrl-K` in the web UI) |
| `r`       | Internet radio                                |
| `Space`   | Play / pause (desktop)                        |
| `+` / `-` | Volume up / down (`=` / `_` work unshifted)   |
| `m`       | Mute / unmute                                 |
| `e`       | Audio settings (EQ, ReplayGain, crossfade)    |
| `q`       | Show / hide the play queue                    |
| `b`       | Show / hide the sidebar                       |
| `f`       | Fullscreen player (while something plays)     |
| `s`       | Cycle skin                                    |
| `Esc`     | Close dialog / go back                        |
| `?`       | Help overlay (desktop)                        |

## GraphQL API

```bash
# Start the server
music-player
```

Open [http://localhost:5053/graphiql](http://localhost:5053/graphiql) in your browser.

<p style="margin-top: 20px; margin-bottom: 20px;">
 <img src="./preview-api.png" width="100%" />
</p>

## Search

The library is indexed into **SQLite FTS5** virtual tables that are kept in sync by database triggers — no separate index to maintain, and search works instantly over tracks (title/artist/album/genre), albums and artists with prefix matching:

```bash
music-player search "fire"        # CLI
# GraphQL: query { search(keyword: "fire") { tracks { title } albums { title } artists { name } } }
```

## Configuration

Settings live in `~/.config/music-player/settings.toml` (created on first run). Every key can also be set through a `MUSIC_PLAYER_*` environment variable (e.g. `MUSIC_PLAYER_HTTP_PORT=5053`).

```toml
music_directory = "/home/me/Music"
port = 5051        # gRPC
ws_port = 5052     # WebSocket events
http_port = 5053   # Web UI + GraphQL
device_name = "Music Player"
library_refresh_interval = 30  # rescan the music directory every N minutes (0 = off)
radio_browser_url = "https://de1.api.radio-browser.info"
tunein_url = "https://opml.radiotime.com"
scrobble = true    # Rocksky scrobbling
atproto = true     # AT Protocol sync (bookmarks, likes, listening status)
atproto_car_max_age_hours = 24  # re-download the atproto repo archive at most once a day
cache = false      # cache remote tracks on disk before they play (see Track cache)
acoustid_api_key = ""  # identify badly tagged files from their audio (see Acoustic fingerprints)
```

The library can also be refreshed manually at any time — `music-player scan` from the CLI, or the `scan` mutation in GraphQL. Re-scans only pick up what's new; existing entries are untouched.

### Audio output

By default audio goes to the system output device. `audio_output` redirects the decoded stream (raw S16LE stereo) somewhere else:

```toml
audio_output = "cpal"              # system audio device (default)
audio_output = "stdout"            # raw PCM to stdout
audio_output = "fifo:/tmp/mp.pcm"  # named pipe
audio_output = "unix:/tmp/mp.sock" # unix socket
audio_output = "tcp:0.0.0.0:9000"  # tcp socket, e.g.: ffplay -f s16le -ar 44100 -ac 2 tcp://host:9000
```

### Track cache

Playing from a remote server fetches each track as it starts, so there is a short
cut at every track change while the next stream opens. With `cache = true` the
daemon downloads the next track once the current one is halfway through, and
plays it from disk instead:

```toml
cache = true
```

Off by default: it spends gigabytes of your disk on copies of audio you already
have on a server, which is your decision to make rather than the default. Only
finite streams are cached — internet radio has no end and no next track.

```bash
music-player cache        # where it is, and how much it holds
music-player cache clear  # delete it all
```

The cache holds 2 GB before evicting whatever was played least recently;
`MUSIC_PLAYER_CACHE_MAX_BYTES` changes that. Files are named by a hash of the
track, so deleting the directory by hand is safe — it is simply empty again.

### Subsonic / Navidrome & Jellyfin

Browse and stream your remote library from any Subsonic-compatible server ([Navidrome](https://www.navidrome.org/), Airsonic, gonic) or [Jellyfin](https://jellyfin.org/):

```toml
subsonic_url = "https://music.example.com"
subsonic_username = "alice"
subsonic_password = "secret"

jellyfin_url = "https://jellyfin.example.com"
jellyfin_username = "alice"
jellyfin_password = "secret"
```

Restart the daemon: the servers show up as source devices (in the web UI's device picker, or `listDevices` in GraphQL). Connect to one and its artists/albums/tracks/playlists are browsable, with tracks streamed straight from the server.

### Rocksky scrobbling

If you're logged into [Rocksky](https://rocksky.app) (`rocksky login` writes `~/.rocksky/token.json`), the daemon scrobbles what you play — using the classic rule (half the track, or 4 minutes, whichever comes first). Disable it with:

```toml
scrobble = false
```

### AT Protocol sync

Your radio bookmarks and liked songs live in your own [atproto](https://atproto.com) repo, so they follow you between devices and clients ([atradio.fm](https://atradio.fm), [Rocksky](https://rocksky.app), music-player).

Nothing here is required: with no account linked, music-player skips all of it and never touches the network for it. Turn the whole integration off with:

```toml
atproto = false
```

**Linking an account.** Reading your repo only needs your identity, which comes from `rocksky login` (`~/.rocksky/token.json`). Writing back needs a session — either `atradio login`, or password credentials in the environment:

```bash
export ATPROTO_IDENTIFIER=alice.bsky.social   # handle, DID or email
export ATPROTO_APP_PASSWORD=xxxx-xxxx-xxxx-xxxx
```

The session file is shared with the [atradio](https://atradio.fm) CLI, so signing in once covers both. The daemon logs at startup whether it is authenticated, and what is missing if not.

**What syncs.**

|                      |                                                                                                                                                                                                                                                                                                                                     |
| -------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Radio bookmarks**  | `fm.atradio.favorite` records are imported into your local bookmarks on startup, and bookmarking a station writes the record to your PDS (unbookmarking deletes it). Bookmarks that only existed locally are pushed up.                                                                                                             |
| **Liked songs**      | `app.rocksky.like` records are imported, then matched against your library on title + artist + album (case-insensitive, indexed). A match links the track to the song record through the new `aturi` column on `track` / `album` / `artist`. A like whose file isn't in the library yet is kept and re-matched after the next scan. |
| **Listening status** | While a station plays, it is published as your `fm.atradio.actor.status` record, written straight to your PDS; the record is deleted when playback stops.                                                                                                                                                                           |

**How it reads and stays in sync.** The initial import downloads your repo once as a CAR archive (`com.atproto.sync.getRepo`) and walks its Merkle Search Tree, so one request covers every collection — `listRecords` is the fallback. After that the daemon subscribes to several public Jetstream instances at once and de-duplicates events by repo revision, so a like or bookmark added on another device shows up here within seconds without depending on any single instance staying up.

**Downloading the repo again.** Bookmarks and likes share the same archive, so a start pulls it at most once, and the date of the last successful download is kept in the database: a restart within `atproto_car_max_age_hours` keeps what is already imported instead of pulling the whole repo again (Jetstream has been applying changes in the meantime). To force a fresh download:

```bash
music-player --force-car-sync
```

```toml
atproto_force_car_sync = true   # force it on every start
atproto_car_max_age_hours = 24  # otherwise re-download only once a day (0 = every start)
```

## Audio analysis & auto-DJ

Every track can be decoded once and measured: a **waveform**, its **loudness**
(EBU R128, via `ebur128`), its **tempo**, and its **mood** as a point in
valence/arousal space (via [`oximedia-mir`](https://crates.io/crates/oximedia-mir)).
Results are cached in SQLite, so a track is analysed once and never again — the
audio does not change, so neither does the answer.

That is enough to answer "what should play after this?" without genres, tags or
anyone else's listening history:

- **Auto-DJ** keeps the queue five tracks deep, each chosen to follow the last
  by tempo and mood. It never repeats a track, never plays the same artist twice
  in a row, and never interrupts what is playing. Half and double time count as
  the same tempo, so it will move between 87 and 174 BPM the way a DJ does.
- A **target** — energy, brightness, a tempo — steers it. It is a direction, not
  a filter: the set leans that way over the next few tracks rather than jumping
  to the most extreme thing that matches.
- The **waveform** appears under the artwork in the full-screen player on both
  the desktop and the web client, and doubles as a seek bar — you can aim at the
  quiet part you remember rather than at a percentage.
- **Key and tempo** get a column in every track list, with the key drawn as a
  colour on the row's left edge. The colour follows the circle of fifths, so
  keys that mix look alike — finding a compatible track is spotting neighbouring
  colours rather than reading labels. Both are filterable in smart playlists:
  `key==Fm;bpm>=120;bpm<=130`.

A file's own `initialkey` and BPM tags are read before anything is computed. A
tag written by Mixxx, Rekordbox or Traktor is a better answer than a
re-derivation — and detection gets the *mode* wrong often enough to matter,
since the major and minor profiles for one tonic look alike. Only tracks with
no tag are analysed.

Analysis is on demand, not automatic: it costs a decode per track, and a
download first for a remote server. Start a pass from an agent
(`analyze_library`) or leave it alone — everything else works without it, and an
unanalysed track simply shows a flat line where its waveform would be.

## Acoustic fingerprints & missing tags

Tags lie. Files get ripped with no tags at all, downloaded as `track03.mp3`, or
tagged by something that wrote "Unknown Artist" and moved on. The audio does
not lie, so every local file also gets a **Chromaprint** acoustic fingerprint —
the same fingerprint [AcoustID](https://acoustid.org) indexes, computed by a
pure-Rust port, so there is no C library to install and every platform gets the
same answer. It is robust to the encoder, the bitrate and the volume: two rips
of the same recording fingerprint alike even when nothing about their bytes or
their tags does.

Fingerprinting runs after a scan, never during one. Only the first two minutes
of each file are decoded, a few files at a time, and in the daemon it runs at a
background pace with the rest of the time given back — nothing is waiting on
it, and a player that stutters while its library is being catalogued is a worse
player. A fingerprint is stored once and never recomputed.

With an AcoustID key configured, the files whose tags are **missing** are then
looked up and filled in from the MusicBrainz recording their audio matches —
title, artist, album, year, track and disc number, plus the recording and
release MBIDs — which is what MusicBrainz Picard does, minus the window:

```toml
acoustid_api_key = "your-key"   # or export ACOUSTID_API_KEY
```

Keys are free and take a minute: [acoustid.org/new-application](https://acoustid.org/new-application).
Without one nothing is sent anywhere; the fingerprints are still computed and
stored, and turning the key on later is a lookup rather than a rescan.

Two rules keep this honest. **Tags that exist are never overwritten** — they
came from whoever made the file and beat a guess from a database, so only the
blanks are filled. And a candidate recording whose length disagrees with the
file by more than 15 seconds is rejected, because a popular song matches its
own remixes and radio edits, and tagging the album version as the extended mix
is worse than leaving it blank. Every answer is stored, including "not found",
so nothing is ever asked twice.

## AI agents (MCP)

`music-player mcp` serves the [Model Context Protocol](https://modelcontextprotocol.io)
on stdin/stdout, so Claude, Codex, Copilot and anything else that speaks MCP can
run the player: ask what is on, search the library, work the transport, build a
queue, and DJ from the audio analysis below. It drives a running daemon over the
same gRPC API the desktop and the TUI use, so a set an agent queues is the queue
every client shows.

Register it with Claude Code:

```bash
claude mcp add music-player -- music-player mcp
```

or, for hosts configured by file:

```json
{
  "mcpServers": {
    "music-player": { "command": "music-player", "args": ["mcp"] }
  }
}
```

### Try it

Once the MCP server is registered, these all work from a chat prompt. The
interesting ones are the last few — they are questions about how music *sounds*,
which a search box cannot answer.

```
> what's playing?

  Nothing is loaded right now. Want me to put something on?

> put on something upbeat

  Queued 9 tracks, opening with Justice — D.A.N.C.E. and building through
  Daft Punk and Chromeo. Nothing was interrupted; your queue was empty.

> analyse my library so you can pick by feel

  Started a pass over 812 tracks. It runs in the background — about a second
  each for local files. I'll work from what's already done in the meantime.

> more like this but calmer, and keep it going

  Handed the queue to auto-DJ with the energy target down at 0.25. It's picked
  Bonobo — Kiara next, then Tycho. It'll keep five tracks queued and drift
  gently downward from here rather than dropping straight into ambient.

> what is this track, actually?

  Bicep — Glue: 128 BPM, energy 0.72, fairly bright, -8.4 LUFS. That tempo is
  why it sat so well after the last one.

> nothing over 100 bpm for the next hour

  Steering auto-DJ to 90 BPM. The two tracks already queued above that are
  still there — say the word and I'll clear them.

> stop choosing for me

  Auto-DJ off. The four tracks it already queued will play out.
```

`skills/music-player/SKILL.md` goes further: it is a [skill](https://code.claude.com/docs/en/skills)
that teaches an agent to *DJ* rather than merely to call the tools — queue
instead of interrupt, sequence a set deliberately, and work from what the
library actually holds. Copy it into `~/.claude/skills/music-player/`.

## Listening analytics

The player records every listen to `play_history`, which answers "how many
times" but nothing about *when*. Point the analytics engine at that log — plus
any history you can export from elsewhere — and the interesting questions
become ordinary queries.

It is a [DuckDB](https://duckdb.org) **lens** over your data: it reads the
player's SQLite database and never writes to it, and the whole analytics file
can be deleted and rebuilt from its sources at any time.

### Importing

```bash
music-player analytics sync                       # what this player has recorded
music-player analytics import ~/Downloads/Spotify\ Extended\ Streaming\ History
music-player analytics import ~/Downloads/lastfm-scrobbles.csv
music-player analytics rocksky your-handle.com    # your Rocksky scrobbles
music-player analytics enrich                     # albums, genres, years via Rocksky
```

Each import is idempotent — running it twice imports nothing the second time —
and reports what it skipped rather than silently dropping it.

DuckDB allows one writer or many readers at a time, so while an import is
running the reporting commands (and the desktop panel) will say the database
is busy. `enrich` over a large first-time backfill is the long one — tens of
minutes for a decade of history, rate-limited to stay well inside the API's
budget. It is resumable and cached, so later runs only pick up what is new.

**Sources are deduplicated against each other.** The same listen often arrives
from two places at once, and rarely at the same instant: a scrobbler that
submits local time as UTC leaves its scrobbles hours away from their Spotify
twin. Rather than assume a correction, the engine *measures* the modal clock
offset between each pair of sources and folds duplicates within a few minutes
of it, keeping the copy from the source that knows most (this player, which
records how much was actually heard, over Spotify, over a bare scrobble).

### Asking

```bash
music-player analytics overview          # the headline numbers
music-player analytics top artists       # or tracks, albums, genres
music-player analytics clock             # when you listen, as a heatmap
music-player analytics sessions          # how long, how many tracks, what opens one
music-player analytics skips             # what you abandon, and how far in
music-player analytics drift --bucket year   # how your taste moved
music-player analytics transitions       # what actually follows what
music-player analytics rotation          # how concentrated your listening is
music-player analytics on-this-day
music-player analytics query "SELECT ..."    # anything the above does not cover
```

Every command takes `--days N` to restrict the window, `--limit N` to size the
output, and `--local-only` to count only music your library actually has —
matched on title and artist, so a play imported from Spotify still counts when
you own the track.

A **session** is a run of listens with no gap longer than thirty minutes —
the unit most questions are really about, and the one a play count cannot
express. `transitions` counts pairs within a session, which makes it the
transition graph your own listening has built: better input for an automatic
mix than tempo matching alone, because it encodes sequences you actually chose.

`skips` only counts sources that record play duration — this player and a
Spotify import. A scrobble is submitted *after* a track has been listened to,
so counting it would dilute every rate toward zero.

### On the desktop

The **Statistics** tab grows a *Listening history* panel below its existing
counters: the headline figures, a weekday × hour heatmap of when you listen,
listens over time, and all-time top artists and tracks. It is kept apart from
the counters above it because the two answer different questions — those are
about the library or server connected right now, this is every source ever
imported, deduplicated. A switch restricts the panel to music your library has.

The panel hides itself until something has been imported.

### Elsewhere

The same figures are on the daemon's gRPC API as `AnalyticsService`, and as
`listening_*` tools on the [MCP server](#ai-agents-mcp), so an agent can answer
"what was I listening to last summer" and build a set around the answer.

## Casting

Playback isn't limited to the machine running the daemon — from the web UI or GraphQL you can cast to:

- **Chromecast** devices
- **UPnP/DLNA** media renderers
- another **music-player** daemon on your network (auto-discovered via mDNS)

## ✨ Star History

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=tsirysndr/music-player&type=Date&theme=dark" />
  <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=tsirysndr/music-player&type=Date" />
  <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=tsirysndr/music-player&type=Date" />
</picture>
