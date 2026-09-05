# @tsiry/music-player

npm installer for [music-player](https://github.com/tsirysndr/music-player) —
an extensible music player daemon written in Rust. Like
[mpd](https://github.com/MusicPlayerDaemon/MPD) or
[Mopidy](https://github.com/mopidy/mopidy), with batteries included:

- 🎵 **Rockbox playback engine** — 40+ formats (MP3, FLAC, Vorbis, Opus,
  MP4/AAC/ALAC, WavPack, APE, WMA, chiptunes, …) with EQ presets, crossfade
  and ReplayGain
- 🔎 **Instant full-text search** backed by SQLite FTS5
- 🖥️ **Terminal UI** with an fzf-style fuzzy finder (`/`) and a `?` help overlay
- 🌐 **Web UI** served by the daemon at <http://localhost:5053>
- 📡 **gRPC + GraphQL APIs** for building your own clients
- ☁️ Browse & stream from **Subsonic/Navidrome** and **Jellyfin** servers
- 📻 Cast to **Chromecast** and **UPnP/DLNA** renderers
- 🎧 **[Rocksky](https://rocksky.app) scrobbling** on the AT Protocol

This package contains no compiled code itself: on install it downloads the
prebuilt binary for your platform from the project's
[GitHub releases](https://github.com/tsirysndr/music-player/releases) and
verifies its SHA-256 checksum.

## Install

```sh
npm install -g @tsiry/music-player
music-player
```

or run it without installing:

```sh
npx @tsiry/music-player
```

## Quick start

```sh
# start the daemon: scans $HOME/Music, serves the web UI + GraphQL on :5053,
# gRPC on :5051 and WebSocket events on :5052
music-player

# from another terminal — the same binary is also the client:
music-player          # opens the terminal UI (fuzzy search with /, help with ?)
music-player scan     # (re)index the music library
music-player tracks   # list tracks
music-player search "daft punk"
music-player pause
music-player next
```

Open <http://localhost:5053> for the web UI, or
<http://localhost:5053/graphiql> to explore the GraphQL API.

## Picking a version

By default the **newest GitHub release that ships a build for your platform**
is installed. Pin a specific release with the `MUSIC_PLAYER_VERSION`
environment variable (the leading `v` is optional):

```sh
MUSIC_PLAYER_VERSION=v0.2.0 npx @tsiry/music-player
MUSIC_PLAYER_VERSION=0.2.0 npm install -g @tsiry/music-player
```

The binary is cached inside the package after the first download. Changing
`MUSIC_PLAYER_VERSION` re-downloads the matching release; without the
variable, the cached binary is reused.

## Supported platforms

| OS      | Architectures      | Release target                                            |
| ------- | ------------------ | --------------------------------------------------------- |
| macOS   | arm64, x64         | `aarch64-apple-darwin`, `x86_64-apple-darwin`             |
| Linux   | x64, arm64, armv7  | `*-unknown-linux-gnu`, `armv7-unknown-linux-gnueabihf`    |
| Windows | x64                | `x86_64-pc-windows-gnu`                                   |

Anything else: build from source with
`cargo install --git https://github.com/tsirysndr/music-player`.

## Configuration

The daemon reads `~/.config/music-player/settings.toml` (created on first
run); every key can also be set through a `MUSIC_PLAYER_*` environment
variable. Highlights:

```toml
music_directory = "/home/me/Music"
audio_output = "cpal"          # or stdout | fifo:PATH | unix:PATH | tcp:ADDR
library_refresh_interval = 30  # background rescan every N minutes (0 = off)
scrobble = true                # Rocksky scrobbling (needs `rocksky login`)

subsonic_url = "https://music.example.com"   # optional streaming sources
subsonic_username = "alice"
subsonic_password = "secret"
```

See the [project README](https://github.com/tsirysndr/music-player#configuration)
for the full reference.

## How the installer works

1. `postinstall` resolves the release to use — `MUSIC_PLAYER_VERSION` if set,
   otherwise the newest release that publishes an asset for your platform —
   and downloads `music-player_<tag>_<target>.tar.gz` from GitHub.
2. The tarball is verified against the release's published `.sha256` checksum.
3. The binary is unpacked next to the `music-player` bin shim, which simply
   `exec`s it and forwards signals.

If the download fails during install (offline machine, firewalled CI…), the
install still succeeds and the shim downloads the binary on first run.
Installs with `--ignore-scripts` work the same way.

## Troubleshooting

- **`unsupported platform …`** — no prebuilt binary exists for your
  OS/architecture; build from source (see above).
- **`no recent release publishes a … build`** — the last few releases didn't
  include your platform; pin an older one with `MUSIC_PLAYER_VERSION`.
- **HTTP 403 from api.github.com** — GitHub rate limit (common on shared CI);
  retry later or pin a version to skip the release lookup... the download
  itself is not rate limited.
- **Corporate proxy** — the installer uses Node's built-in `fetch`, which
  ignores `HTTPS_PROXY` by default; on Node ≥ 24 run with
  `NODE_USE_ENV_PROXY=1` to honor it (custom CAs: `NODE_EXTRA_CA_CERTS`).

## License

MIT — see [LICENSE](https://github.com/tsirysndr/music-player/blob/master/LICENSE).
