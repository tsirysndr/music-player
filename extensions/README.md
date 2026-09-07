# Extensions

Extend music-player with WebAssembly modules, in any language that compiles to
WASI. Extensions are sandboxed: they get no filesystem, no clock, no sockets and
no library access unless their manifest asks for it and the user agrees.

Built on [Extism](https://extism.org). The host/guest contract lives in
[`schema.yaml`](schema.yaml), an [XTP schema](https://docs.xtp.dylibso.com) —
one file that generates typed bindings and a working skeleton for **Rust, Go,
TypeScript, Python, C#, Zig and C++**.

```sh
music-player extension init com.example.lyrics -c metadata -l rust
cd lyrics && cargo build --release --target wasm32-wasip1
```

---

## Contents

- [The five capabilities](#the-five-capabilities)
- [What you can build](#what-you-can-build)
- [The manifest](#the-manifest)
- [Permissions and the sandbox](#permissions-and-the-sandbox)
- [Host functions](#host-functions)
- [Installing extensions](#installing-extensions)
- [Where extensions live](#where-extensions-live)
- [Writing one](#writing-one)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

---

## The five capabilities

An extension declares what it plugs into. The host calls **only** the exports
covered by a declared capability — an extension that declares `metadata` is
never asked to handle an event, even if its module exports a handler.

| Capability   | Exports                                                                                             | For                                     |
| ------------ | --------------------------------------------------------------------------------------------------- | --------------------------------------- |
| `events`     | `on_track_played`, `on_track_skipped`, `on_track_liked`, `on_playlist_created`, `on_scan_completed` | Reacting to what happens                |
| `metadata`   | `get_metadata`                                                                                      | Supplying lyrics, artwork, bios, genres |
| `commands`   | `commands`, `run_command`                                                                           | Actions in the UI                       |
| `predicates` | `predicates`, `evaluate`                                                                            | Custom smart-playlist filters           |
| `source`     | `source_info`, `list_albums`, `list_artists`, `list_tracks`, `list_playlists`, `get_stream_url`     | A media source of your own              |

You can declare several. A last.fm extension might be `events` (scrobble) plus
`metadata` (artist bios) plus `commands` ("love this track").

---

## What you can build

### Events — react to listening

Called when something happens. Fire-and-forget: the host does not wait for a
reply, and a handler that fails is logged and skipped.

- **Scrobblers** — post plays to last.fm, ListenBrainz, Maloja, libre.fm
- **Sync bridges** — mirror likes to Spotify/Apple Music/Plex
- **Statistics** — write a listening journal, feed Grafana, build year-in-review
- **Home automation** — dim the lights when playback starts, publish to MQTT
- **Discord / Slack rich presence** — show what you are playing
- **Backups** — snapshot a playlist to Git every time it changes
- **Notifications** — desktop toast, push to a phone, post to a channel

### Metadata — fill in what the tags do not have

Called on demand; answers merged across providers, first non-empty per field.

- **Lyrics** — LRCLIB, Genius, Musixmatch, or a local `.lrc` folder
- **Artwork** — Cover Art Archive, Deezer, fanart.tv, Discogs
- **Artist bios** — Wikipedia, MusicBrainz, Discogs
- **Genre enrichment** — last.fm tags, Discogs styles, an ML classifier
- **Credits** — producers, engineers, session players
- **Translations / romanisation** — kanji → rōmaji, Cyrillic → Latin

### Commands — actions in the UI

Listed once at load, then invoked by name. They appear in the command palette
and on the extensions page.

- **Export** — the current playlist to M3U, CSV, JSON, a Spotify link
- **Import** — from Spotify/Apple Music/Rekordbox/iTunes XML
- **Tag repair** — normalise artist names, fix capitalisation, strip "(Remastered)"
- **Duplicate finder** — report tracks present more than once
- **Library reports** — decade breakdown, longest albums, unplayed corners
- **Share** — post the current track anywhere
- **Analysis** — BPM/key detection, ReplayGain scanning

### Predicates — extend the smart-playlist vocabulary

Add filter terms the built-in schema does not have. Usable as `ext:<name>` in
any smart playlist:

```
ext:mood==energetic;year>2015
ext:bpm>120;genre==house
ext:language==japanese
```

- **Mood / energy** — from audio analysis or a service
- **BPM and key** — for DJ sets, harmonic mixing
- **Language** — filter by what a track is sung in
- **Acoustic properties** — danceability, instrumentalness, loudness
- **External state** — "in my Spotify library", "on my phone"

Called once per candidate track, so keep it cheap and cache aggressively.

### Sources — bring your own catalogue

Expose a browsable, playable catalogue that is not in the local library. Stream
urls are resolved at play time, so signed or expiring links stay fresh.

- **Streaming services** — Bandcamp, SoundCloud, Deezer, Tidal, Qobuz, YouTube Music
- **Podcast feeds** — an RSS podcast catalogue as albums and episodes
- **Radio directories** — Radio Browser, TuneIn, RadioParadise ([example](examples/radio-source))
- **Cloud storage** — Google Drive, Dropbox, S3, WebDAV
- **Other servers** — Ampache, Funkwhale, Nextcloud Music, Airsonic
- **Archives** — Internet Archive, Free Music Archive, netlabels
- **Local oddities** — a NAS share, a MinIO bucket, a friend's exported library

---

## The manifest

Every extension ships a `plugin.toml` (preferred — comments, no trailing-comma
traps) or a `plugin.json`. Both describe the same thing; TOML wins if both exist.

```toml
id = "com.example.lyrics"          # required, stable, reverse-DNS
name = "Lyrics"                    # required, shown in the UI
version = "1.2.0"
author = "Your Name"
description = "Fetches lyrics from LRCLIB"
homepage = "https://example.com/lyrics"
repository = "https://github.com/you/lyrics"
license = "MIT"                    # SPDX identifier
logo = "icon.png"                  # a bundled file, or an https:// url
topics = ["lyrics", "offline"]     # free-form tags for search and grouping
readme = "README.md"               # shown on the extension's details page

entry = "plugin.wasm"              # defaults to plugin.wasm

capabilities = ["metadata"]        # required, and enforced

[permissions]
allowedHosts = ["lrclib.net"]      # nothing else is reachable
libraryRead = true                 # may read the user's library

[permissions.config]               # only declared keys are visible
apiKey = ""
language = "en"
```

`id` may contain letters, digits, `.`, `-` and `_`. It is the database key and
the directory name, so it is kept to a charset that never needs escaping.

---

## Permissions and the sandbox

**Capabilities are what an extension does *for the player*. Permissions are what
it may *reach*.** Calling an external API is a permission, not a capability:
any of the five can need the network — a metadata provider fetching lyrics, a
source listing a catalogue, an events extension scrobbling — so gating it on a
capability would either block honest ones or wave through the rest. One axis
answers "what does this plug into", the other "what can it touch". Both are
declared, and both are shown to the user before they enable anything.

To let an extension call an API, list the hosts:

```toml
[permissions]
allowedHosts = ["api.example.com", "*.cdn.example.com"]
```

Then use your PDK's HTTP client — in Rust, `extism_pdk::http::request`. A
request to any other host is refused by the sandbox, not by the extension's own
good behaviour.

WebAssembly grants nothing by default. Each permission opens exactly one door,
and the manifest is what a user reads before enabling an extension.

|                | Without it                | With it                                         |
| -------------- | ------------------------- | ----------------------------------------------- |
| `allowedHosts` | No network at all         | HTTP to the listed hosts only                   |
| `libraryRead`  | `query_*` return empty    | Read tracks, albums, artists, playlists, radios |
| `config`       | `get_config` returns `""` | Read the declared keys                          |

Also true regardless of the manifest:

- **No filesystem.** An extension cannot read or write any file.
- **No clock, no randomness** beyond what WASI provides.
- **10-second call timeout.** A runaway loop fails its own call, not the player.
- **Isolated memory.** One extension cannot see another's state.
- **Failures are contained.** A panicking, looping or malformed extension is
  logged and skipped — a scan, a play or a playlist still completes.

Permissions are enforced **host-side**. An extension cannot lie its way past
them by claiming something different at runtime than its manifest declared.

---

## Host functions

What an extension may call back into. All are generated for you by
`xtp plugin init`.

| Function                        | Needs         | Returns                                     |
| ------------------------------- | ------------- | ------------------------------------------- |
| `log(string)`                   | —             | Writes to the host log, tagged with your id |
| `get_config(key)`               | declared key  | The value, or `""`                          |
| `query_library(LibraryQuery)`   | `libraryRead` | Tracks matching an RSQL filter              |
| `query_albums(LibraryQuery)`    | `libraryRead` | Albums matching a filter                    |
| `query_artists(LibraryQuery)`   | `libraryRead` | Artists matching a filter                   |
| `query_playlists(LibraryQuery)` | `libraryRead` | Playlists matching a filter                 |
| `get_playlist_tracks(id)`       | `libraryRead` | One playlist's tracks, in order             |
| `get_saved_radios()`            | `libraryRead` | The user's bookmarked stations              |

### Filtering with RSQL

The library queries take an [RSQL](../rsql) filter — the same language smart
playlists use, so there is one vocabulary to learn:

```
genre==rock;year>2000                    # AND
liked==true,playcount>10                 # OR
(genre==jazz,genre==blues);bitrate>=320  # grouped
artist=like=*hood*                       # substring, * is the wildcard
genre=in=(rock,jazz,ambient)             # any of
lastplayed<30d                           # relative age: s h d w m y
added>2024-01-01                         # ISO date
genre=null=                              # missing or empty
```

**Track fields:** `title` `artist` `album` `genre` `year` `track` `duration`
`bitrate` `samplerate` `uri` `liked` `playcount` `skipcount` `lastplayed` `added`
**Album fields:** `title` `artist` `year` `cover`
**Artist fields:** `name`
**Playlist fields:** `name` `description` `smart` `created`

Each table is checked against its own field list, so a typo is an error you can
read rather than an empty result you cannot explain.

---

## Installing extensions

### With the CLI

```sh
music-player extension init com.example.lyrics -c metadata   # scaffold one
music-player extension list                                  # what is installed
music-player extension install <url>                         # install from a url
music-player extension uninstall com.example.lyrics          # remove it
```

`extension` is aliased to `ext`, `init` to `new`, `list` to `ls`, `install` to
`add`, and `uninstall` to `remove`/`rm`.

`init` writes the manifest and — when the [`xtp`](https://docs.xtp.dylibso.com)
CLI is on PATH — generates typed bindings and a working skeleton for the
language you pick. The schema is compiled into the binary, so this works from
any directory.

### By hand

**From a directory** — drop it into an extensions path and restart:

```
~/.config/music-player/extensions/com.example.lyrics/
├── plugin.toml
├── plugin.wasm
└── icon.png
```

**From a URL** — a published `.wasm`, or a manifest that names one:

```sh
# A manifest url brings its own metadata and permissions.
music-player extension install https://example.com/lyrics/plugin.toml

# A bare .wasm carries none, so its capabilities have to be given.
music-player extension install https://example.com/plugin.wasm -c metadata
```

Downloads are staged in a temporary directory and moved into place only once the
module loads, so a failed install cannot leave a half-written extension behind.
They land in `<app dir>/extensions-cache/`.

---

## Where extensions live

By default `<app dir>/extensions`. Override with an ordered list in
`settings.toml`:

```toml
extension_paths = [
  "~/.config/music-player/extensions",   # yours
  "/usr/share/music-player/extensions",  # system-wide
]
```

`~` and `$VAR` are expanded. **Earlier paths win**: an id found twice is loaded
from the first, so a local copy shadows a system one. The download cache is
always searched last.

## Switching one off

Every extension is on until you say otherwise. The flag lives in the
`extension` table of the music-player database, keyed by manifest id, and only
what you have actually changed is stored — an extension with no row is enabled,
so dropping a new one in works without a second step.

Toggle it from the **Extensions** view in the web UI or the Slint desktop app.
Both also have a rescan button, which picks up an extension added or removed on
disk and drops the stored flags for anything no longer there.

Switching one off takes effect the next time the daemon starts. Unloading a
WebAssembly module mid-session would pull the ground out from under whatever
happens to be in the middle of a call into it, so a module that is already
running keeps running.

Nothing else about an extension is stored: its name, version, capabilities and
permissions are read from the manifest on disk every time, because that is what
changes when one is upgraded.

---

## Writing one

### 1. Generate the skeleton

```sh
music-player extension init com.example.lyrics -c metadata -l rust
```

Or drive `xtp` yourself, against this repo's schema:

```sh
xtp plugin init --schema-file extensions/schema.yaml --template rust --path my-extension
```

Templates: `rust` `go` `typescript` `python` `csharp` `zig` `cpp`.

You get `src/pdk.rs` (generated bindings — do not edit) and `src/lib.rs` (yours).

### 2. Implement the exports you declared

```rust
pub(crate) fn get_metadata(
    input: types::MetadataRequest,
) -> Result<types::MetadataResponse, Error> {
    let key = pdk::get_config("apiKey".to_string())?;
    let url = format!("https://lrclib.net/api/get?artist_name={}&track_name={}",
        urlencode(&input.track.artist), urlencode(&input.track.title));

    let response = http::request::<()>(&HttpRequest::new(&url), None)?;
    if response.status_code() != 200 {
        return Ok(types::MetadataResponse::default());
    }
    let found: Lyrics = serde_json::from_slice(&response.body())?;
    Ok(types::MetadataResponse {
        lyrics: Some(found.synced_lyrics),
        ..Default::default()
    })
}
```

`pdk.rs` exports every function in the schema, so you will need a stub for the
ones you do not implement. That is harmless — the manifest is what gates which
ones are ever called.

### 3. Build and install

```sh
cargo build --release --target wasm32-wasip1

# `extension list` prints the search paths; the first is where to copy to.
mkdir -p "$(music-player extension list | sed -n '2s/^ *//p')/com.example.lyrics"
cp target/wasm32-wasip1/release/plugin.wasm plugin.toml \
   "$(music-player extension list | sed -n '2s/^ *//p')/com.example.lyrics/"
```

Restart. The log tells you what happened:

```
INFO searching for extensions paths=["~/.config/music-player/extensions", ...]
INFO loaded extension extension=com.example.lyrics version=1.2.0 capabilities=metadata
INFO extensions ready count=1 extensions=["com.example.lyrics"]
```

---

## Examples

| Example                                         | Capability   | Shows                                                                                        |
| ----------------------------------------------- | ------------ | -------------------------------------------------------------------------------------------- |
| [`scrobble-logger`](examples/scrobble-logger)   | `events`     | Handling only the events you care about; config; failing softly when a webhook is down       |
| [`lyrics-provider`](examples/lyrics-provider)   | `metadata`   | Answering only for what you know; synced-over-plain fallback; "not found" as an empty answer |
| [`library-commands`](examples/library-commands) | `commands`   | Declaring commands up front; reading the whole library with RSQL; human + structured results |
| [`mood-predicate`](examples/mood-predicate)     | `predicates` | Adding `ext:mood` / `ext:era`; honouring `!=`; keeping per-track evaluation cheap            |
| [`radio-source`](examples/radio-source)         | `source`     | A full media source: browse, search, resolve streams at play time, call an allow-listed host |

Each example is a complete, buildable extension. Copy one and edit.

---

## Troubleshooting

**It is not in the list.** The directory needs a `plugin.toml` or `plugin.json`
*and* the `.wasm` that its `entry` names. Check the log for `ignoring an
extension`.

**It loaded but nothing calls it.** Check `capabilities` — the host calls an
export only when the matching capability is declared.

**A network call fails.** Add the host to `allowedHosts`. The sandbox refuses
everything else; there is no wildcard-everything option by design.

**`get_config` returns empty.** Only keys declared under `[permissions.config]`
are visible.

**The library queries return nothing.** Set `libraryRead = true`. A denied read
logs `library read denied` and answers with an empty list rather than failing
your call.

**A call times out.** The limit is 10 seconds. Cache what you can — `evaluate`
in particular runs once per candidate track.

**`xtp schema validate` fails with `invalid character '<'`.** The hosted XTP
service is retired and that subcommand needs it. `xtp plugin init` works
offline, and generating bindings is itself a thorough validation of the schema.
