# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

This file starts at 0.2.1. For earlier releases see the
[git tags](https://github.com/tsirysndr/music-player/tags).

## [0.4.0] — 2026-09-13

### Added

#### A listening history, and screens that read it
- `track_stats` keeps running counters — one row per track, overwritten in
  place — which is what smart playlists read, and which cannot answer anything
  about *when*. A new **`play_history`** table is the event log those counters
  summarise: one row per listen, appended and never updated, with how much was
  actually heard — kept as milliseconds rather than a ratio, so a later change
  of mind about what counts as a skip can be applied to history already
  collected. Five SQL views (`v_most_played`, `v_most_skipped`,
  `v_never_played`, `v_recently_added`, `v_recently_played`) live in the
  migration rather than in Rust, so every consumer sees one definition. A track
  skipped ten times has still never been *listened to*, and the views say so.
- **Most Played** and **Statistics** join the desktop sidebar: a ranked
  play-count list, and a stat-card strip (tracks / plays / skips / never
  played) over the recently-played and most-skipped sections. They read the
  local analytics views, which live on this machine even when browsing a
  remote server. Recently Played collapses to the latest listen per track, so
  replaying a song does not fill the screen with duplicate rows.

#### The spectrum is measured now
- The engine (rockbox-playback 0.7.0) measures a real **16-band log-spaced
  spectrum** per output buffer — a one-pole low-pass ladder over the signed
  mix, cheap enough for the audio callback — and `Levels` carries the bands
  through tracklist and the gRPC stream, zero for older engines.
- The desktop's full-screen equalizer switches from two-scalar synthesis to
  those measured bands: per-band auto-gain (each column rides its own decaying
  peak, so a hi-hat column reaches the top on hi-hats, not on kicks),
  interpolated across the 96 bars. The data is real now, and decoration on top
  of measurement reads as noise; the old synthesis stays as the fallback
  against an older daemon.

#### The seek bar is the waveform
- The desktop player bar draws the current track's own waveform as the seek
  bar: played bars in accent, the rest dimmed. A track the background pass has
  not reached yet is analysed **on demand** the moment it plays, instead of
  showing a flat rail until the scanner gets there.

#### Five new builtin skins
- **Nord**, **Oceanic**, **Tape**, **Phosphor** and **Parchment**, with this
  client's fonts and skeleton/syntax tokens filled in. The two light skins sit
  last in the cycle, so cycling runs through the dark ones before changing the
  room.

#### Context menus that say what they act on
- Every desktop context menu now leads with what it is about — mini album art,
  title, artist — via a shared header: the track-row menu, the album-card menu
  and the palette menu, so a stray click cannot act on the wrong song.
- Palette search results grew an ellipsis menu (play / play next / add to
  queue / add to playlist), clamped on-screen and opening upward at the bottom.
  The playlist picker pins a **"Create new playlist"** row on top; the typed
  query becomes the name and the pending track or album lands in it in one
  call.
- **"Add to playlist…"** is one always-present item opening the raycast-style
  picker, replacing an inline top-3 whose browse row only appeared with four or
  more playlists. Artist-detail and album-detail rows gained the missing
  wiring.

#### Knobs with a value arc
- Every knob draws an arc over its own border, from its origin round to the
  current value. Volume, precut and the crossfade knobs fill from their
  minimum; balance, bass, treble and ReplayGain pre-amp anchor at their neutral
  centre, so a cut sweeps anti-clockwise out of it and a boost clockwise.

#### Codec and sample rate on the remote-player wire
- The Rocksky remote-player websocket now sends the **codec** (from the local
  file's extension; streams send nothing) alongside the sample rate, so the
  format badge can render fully on the other end.

### Changed

- **sea-orm 0.9 → 2.0.2** (five majors in one step, sqlx 0.6 → 0.9 with it):
  raw statements moved to the new `_raw` forms, migrations use
  `execute_unprepared`, expression methods come from `ExprTrait`, and
  `UpdateOne` updates by the model's own primary key. The full workspace suite
  passed after the migration.
- **rockbox-playback 0.7.0 resolves from crates.io** — the temporary
  `[patch.crates-io]` is gone, and the workspace no longer needs a sibling
  rockbox-zig checkout to build.
- A migration **drops the keys and tempos that were guessed before tags were
  read**: the first analysis build detected both from the audio, and measured
  against tagged files roughly a third of those rows disagreed with what the
  file plainly said. That build wrote Camelot notation and every build since
  writes traditional, so the guesses are identifiable exactly; the rows are
  recomputed from the files. Irreversible by design — what it removes was
  wrong.
- The fullscreen player **drops the waveform strip** under the album art — the
  seek bar is already the waveform, and the strip crowded the art. Statistics
  drops the never-played listing for the count card alone.
- The player bar is truly centred — both side columns share one width — and
  responsive: below 1050px the VFD goes, below 820px the panel buttons follow.
  Mute and the volume knob stay at every size; they are controls, not
  decoration. The window opens at the full 1200×780 layout, so everything is
  on screen from the start.
- The sidebar account row **opens a menu** instead of signing out on click.
  Clicking your own name should not end the session — and signing out also
  stops scrobbling and station sync, which is not something to do on a stray
  click.
- The aarch64 Linux release **builds on an arm runner** instead of
  cross-compiling through a hand-built sysroot that pkg-config never knew
  about. `armv7-unknown-linux-gnueabihf` is dropped: there is no 32-bit ARM
  runner, so keeping it would mean keeping the sysroot.
- Install docs for the **macOS cask** and **Arch Linux (AUR)** package.

### Fixed

- **A play counted once, not once per second.** The stats recorder never
  marked its watcher as submitted, so a single listen bumped the play count
  every tick past the threshold — 141 "plays" of one track. It is marked the
  moment the play is counted, and the inflated rows were cleared.
- **The embedded daemon records plays at all.** The desktop app's built-in
  daemon never spawned the play-stats recorder — only the standalone server
  did — so playing music in the app updated no analytics.
- **Signing in mid-session turns atradio on.** The status publisher checked
  for an identity once at daemon startup and returned permanently; a user who
  signed in from the UI minutes later published nothing until the next
  restart. The gate now polls a local read every 30 seconds and starts the
  sync tasks as soon as an identity appears.
- **Sign-out sticks.** A daemon configured by environment variables
  re-established a session on the very next read after sign-out, so the button
  appeared to do nothing. A deliberate sign-out is recorded in a file — it has
  to be readable before anything else is set up, and there is exactly one bit
  to store — and signing in again clears it.
- **Every heart on screen updates.** Liking a track refreshed only the Tracks
  and Liked lists; detail pages and the queues were skipped, which read as the
  button being broken. The liked flags are now patched into the live models in
  place — every view at once, scroll position untouched — and the network call
  no longer holds up play/pause behind a round-trip with no timeout.
- **One liked-tracks order — recently liked first — for list and queue.** The
  Liked screen and the play-liked queue were built from different sources, so
  clicking row N could start a different track. An ordered id list is now the
  single source for both, most recently liked on top.
- The **disc number survives** the tag conversion, so multi-disc albums group
  again — and a file with no disc tag stays `None`, so a single-disc album
  does not grow a "DISC 1" header.
- The now-playing frames **always carry a sample rate**: a queue restored from
  an old session predates the field, and the status loop now falls back to the
  library row.
- The VFD is gated on the player bar's **effective width** rather than the
  window's, so it can no longer appear in a bar too narrow for it and clip the
  volume knob.
- The nix flake builds again.

## [0.3.0] — 2026-09-10

### Added

#### Hearing the music: tempo, key, mood, loudness and waveforms
- A new `music-player-analysis` crate decodes a track **once** and measures four
  things from that single pass: a **waveform**, its **loudness** (EBU R128, via
  `ebur128`), its **tempo**, and its **mood** as a point in valence/arousal
  space (via [`oximedia-mir`](https://crates.io/crates/oximedia-mir)). Decoding
  is nearly the whole cost, so doing it per feature would decode each track
  three times.
- **The file is asked before anything is computed.** A track's own `initialkey`
  and BPM tags win over detection: a tag written by Mixxx, Rekordbox or Traktor
  was produced deliberately, and correlation-based key detection picks the tonic
  well but the *mode* badly — the major and minor profiles for one tonic look
  alike, and confusing them puts a key at the opposite end of the circle of
  fifths. Camelot (`4A`) and traditional (`Fm`, `F minor`, `Abm`, a bare `F`)
  spellings are both read; anything else is left alone, because a wrong key is
  worse than none.
- Results are cached in SQLite (`track_analysis`, migration `m20260909_000002`),
  keyed by library as well as track — ids are only unique within a provider, and
  without that a Navidrome track and a local one could share a row.
- **Key and tempo columns** in every track list, in both clients, with the key
  drawn as a colour on the row's left edge. The colour follows the circle of
  fifths, so keys that mix look alike and finding a compatible track is spotting
  neighbouring colours rather than reading labels. Relative major and minor
  share a hue, with the minor darker. Keys are named traditionally — `D`, `Fm` —
  because that is what every other tool in a library shows.
- The columns appear only when the library can answer for them (the daemon's own
  or another music-player), rather than sitting empty against Subsonic or
  Jellyfin, where an empty column reads as broken rather than absent. A null key
  shows nothing; a null tempo shows a dash, since a blank in a numeric column
  reads as a fault.
- `key` and `bpm` are **RSQL fields**, so a smart playlist can be
  `key==Fm;bpm>=120;bpm<=130`. Tempo compares rounded, so `bpm>=120` does not
  miss a track stored as 119.97.
- The scanner fills both in after indexing, logging each track as it goes.
  `music-player scan` runs it at full speed — the user is watching — while the
  daemon's periodic refresh rests three times as long as it just worked, so it
  drifts along at roughly a quarter of one core instead of fighting the player
  it exists to improve.

#### Auto-DJ
- The daemon keeps the queue five tracks deep, each chosen to follow the last by
  tempo and mood. Built as a **chain, not a ranking**: every pick is measured
  against the one before it, so an hour travels a long way while each transition
  stays close. Ranking everything against one seed gives a set that never leaves
  the first track's neighbourhood and then falls off a cliff.
- Half and double time count as one tempo — 140 and 70 are the same tempo
  counted differently, and without that they look further apart than anything
  else in the library and never follow each other.
- It never repeats a track, never plays the same artist twice in a row, and
  never interrupts what is playing — that last is the whole difference between
  it and shuffle.
- A **target** (energy, brightness, tempo) steers rather than filters, weighted
  so it can overcome about one step of chain distance and no more: weaker and
  "make it more energetic" does nothing, stronger and every pick jumps to the
  most extreme match.

#### An MCP server, so an agent can DJ
- `music-player mcp` serves the [Model Context Protocol](https://modelcontextprotocol.io)
  on stdin/stdout, so Claude, Codex, Copilot and anything else that speaks MCP
  can run the player: ask what is on, search the library, work the transport,
  build a queue, and read the analysis above. It drives a running daemon over
  the same gRPC API the desktop and TUI use, so a set an agent queues is the
  queue every client shows.
- Twenty-one tools, chosen rather than enumerated — every one is context the
  model pays for on each turn, so the five transport actions are one tool with
  an argument and listings are one `browse_library` rather than three. The unit
  of exchange is a track id, so an agent never has to construct a track or know
  what a uri is.
- A failed tool returns a result flagged `isError` rather than a JSON-RPC error:
  a daemon that is not running is something the model should see and say, not
  something the host swallows as a transport fault.
- `skills/music-player/SKILL.md` teaches an agent to *DJ* rather than merely to
  call the tools — queue instead of interrupt, sequence a set deliberately, work
  from what the library actually holds — and carries CLI install instructions
  for users who have none.

#### Waveforms and an equalizer in the full-screen player
- The track's **waveform** sits under the artwork in both clients and doubles as
  a seek bar: you can aim at the quiet part you remember rather than at a
  percentage. Bars are normalised to percentiles rather than to the maximum, so
  one stray click cannot flatten the whole track.
- A live **equalizer** runs full-bleed along the bottom edge, driven by the
  daemon's `levels` feed — so it moves with audio actually leaving the output,
  including on a cast device the client never decodes. The daemon measures four
  numbers rather than a spectrum, so the response is shaped: bars lean on the
  channel nearest them (the display is stereo) and low bars are steadier than
  high ones, because bass is sustained and treble is transient. `v` toggles it.

#### Browse by genre
- A **Genres** entry and screen in the web client, with `genre`, `track_genres`
  and `artist_genres` tables (migration `m20260909_000001`). Two link tables
  rather than one, because the sources disagree usefully: a file's own tag is
  precise but often absent, while the Rocksky artist enrichment has far better
  coverage but describes the artist rather than the track.
- `genres` and `genreTracks` on the provider trait, so a remote server's genres
  work the same way.

#### Sign in with an Atmosphere account
- An avatar, display name and muted `@handle` at the foot of the sidebar in both
  clients, with a sign-in modal taking a handle and an **app password** —
  atproto issues them for exactly this, they can be revoked one at a time, and
  the daemon has no browser to run an OAuth flow in.
- One account for the whole daemon: the same session Rocksky scrobbling and
  atradio use, so signing in here turns those on and signing out turns them off.
  A daemon already logged in from the CLI shows as signed in. Signing in pulls
  the account's repo in the background.

#### Caching the tracks that are about to play
- Finite remote tracks are downloaded at the halfway point of the current one,
  so a track change does not wait on the network. **Off by default** (`cache =
  true` in settings.toml): it spends gigabytes of disk on copies of audio you
  already have on a server, which is not a thing to start doing because someone
  installed a music player.
- Cached copies are used at all three points a uri reaches the engine, including
  the gapless lookahead — the transition the cache exists for. A finished
  prefetch re-points a lookahead queued before it landed.
- The queue's uris are never rewritten: a track keeps the remote uri it came
  with, and resolution happens only where a path is handed to the engine.
- The mapping lives in the filenames rather than in a database beside them, so a
  cache directory deleted by hand cannot leave an index insisting the files are
  still there. `music-player cache` and `music-player cache clear` show and free
  it.

#### More libraries to read from
- **Kodi** and **Plex** providers, and a **Rocksky** provider preconfigured for
  `https://navidrome.rocksky.app`.
- A **Raycast-style server switcher** on `C` in every client, including the TUI,
  with a shortcut to add a server. A **Play to** picker replaces the queue
  button in the miniplayer, listing Chromecast, UPnP/DLNA and peer instances.
- **Federated search**: one result list holding rows from the local library and
  the connected server, each labelled with where it came from, and a context
  menu whose actions depend on the source — a local track cannot be added to a
  remote server's playlist.

#### Extensions in both UIs
- An **Extensions** entry in the sidebar of the web UI and the Slint desktop
  app, listing every installed WebAssembly extension with its version, author,
  what it plugs into, and what it asked to reach outside its sandbox (network
  hosts, library access) — a module is granted nothing by default, so what its
  manifest asked for is what a user needs to see before leaving it enabled.
- Each one can be **switched on or off** from either client, with an
  All/Enabled/Disabled quick filter and a free-text search over the id, name,
  description, author, topics and capabilities. The flag is stored immediately;
  a module the daemon already loaded keeps running until it next starts, since
  unloading WebAssembly mid-session would pull the ground out from under
  whatever is calling into it.
- A **rescan** button, for an extension added or removed on disk while the view
  is open. It also drops the stored flags for anything no longer installed — a
  leftover row is harmless while it matches nothing, but it would switch an
  extension straight back off if it were ever reinstalled.
- A new `extension` table (migration `m20260907_000002`) holding just the
  enabled flag, keyed by manifest id. Everything else about an extension is
  read from its manifest on disk, which is what changes when one is upgraded;
  a copy in the database would be a second source of truth that goes stale.
  Only what the user has actually changed is stored, so an extension with no
  row is enabled — the same default the registry applies.
- `extensions(filter:)`, `setExtensionEnabled` and `rescanExtensions` in the
  GraphQL API, backed by a new manifest-only scan
  (`music_player_extensions::catalog`). It reads the manifests instead of
  loading the modules, so drawing a list costs a directory walk rather than a
  WebAssembly instantiation per extension. Installing and scaffolding stay with
  the `music-player extension` CLI — those run code from a source the user has
  to have chosen deliberately, which should not be reachable from a browser tab.

#### An artist page on the desktop
- The Slint client has an **artist page** at last: a round portrait, the
  artist's albums as a grid and every one of their songs below it. Clicking an
  artist opened the search palette with their name in it before, which was a
  stand-in rather than a destination. It is built from the library already
  cached on the UI thread, so it costs no round trip.
- Album and artist results in the desktop's command palette **open their page**
  instead of playing. Opening one is what a search result is for, and the play
  button is right there once you land. Tracks and playlists still play — there
  is no page for them to open.

#### Servers, in the web UI
- A **Servers** page (`/servers`) and sidebar entry, listing the other
  music-player instances, Subsonic/Navidrome and Jellyfin servers, and cast
  targets the daemon can see, with connect, "play here instead" and a rescan.
  It was a modal behind the sidebar's status row before.
- Adding or removing a saved server stays desktop-only: those credentials live
  in a local file the daemon does not read.

#### Audio settings, in the web UI
- The desktop's audio dialog, control for control: a ten-band EQ with a precut
  knob, bass/treble/balance, ReplayGain (mode, pre-amp, clip prevention),
  crossfade with its four fade knobs, and dithering. Opens with `e` or the
  equalizer button in the player bar.
- Everything writes straight through and the daemon hands the whole state back,
  so what is on screen is what the engine took rather than what was asked for.
  The reply is written into the query cache rather than triggering a refetch —
  dragging an EQ band is a write per pointer move.
- `audioSettings`, `setAudioSetting` and `setEqBandGain` in the GraphQL API,
  mirroring the gRPC `MixerService`.

#### Volume and mute shortcuts
- **`+` / `-` change the volume and `m` mutes**, in the TUI, the Slint desktop
  and the web UI. `=` and `_` do the same, since they are the unshifted faces
  of those keys on most layouts.
- Mute never touches the stored level: the daemon keeps it, so unmuting
  restores it rather than guessing. Setting a level unmutes, because asking for
  a level is asking to hear it. Where a volume readout exists it says `muted`
  rather than showing a number you cannot hear, and both UIs grew a mute button
  beside the knob — a shortcut nobody can see is a shortcut nobody uses.

#### Elsewhere
- **`f` opens the fullscreen player** in both clients, and does nothing when
  nothing is playing rather than opening an empty canvas. A station counts: it
  has a name and usually a logo before any ICY metadata arrives.
- A **global search modal** in the web UI, opened with `⌘K`, `Ctrl-K` or `/`.
  It is the one place that searches everything: tracks, albums, artists,
  playlists, extensions, servers and internet radio. The per-page filters on
  Playlists and Extensions are gone, and no page puts a search box in the
  header — the header is for the global search only.
- The web UI has a **test suite** — Vitest on jsdom with Testing Library and
  MSW, ~320 tests over the design system, the Jotai state layer, the hooks, the
  RSQL tokenizer and every data-fetching page. The fixtures are generated from
  a real music-player library rather than written by hand, so the tests see the
  ids, fractional durations and untidy tag data the app actually gets. A `webui
  tests` workflow runs it on any push or PR touching `webui/musicplayer`.

### Changed

#### Providers and renderers are separate crates
- Remote-library backends moved into a **`provider`** crate and the output
  targets (Chromecast, UPnP/DLNA, the local player) into a **`renderer`** crate;
  `addons` is gone, and `source` is `provider` throughout. The dependency graph
  enforces the point: `renderer` depends on the client, which depends on the
  server, so `provider` has to sit below the server — which is what guarantees
  that **switching servers cannot interrupt playback**. Where music is read from
  and where it comes out are different questions.
- `MusicProvider` has six required methods and defaults for everything else
  (`search`, `playlists`, `liked_tracks`, `genres`, `genre_tracks`, playlist
  writes), so adding a backend is one file and one registration line. Clients
  build their add-server forms from the registry, so a new backend needs no
  client change beyond an icon.
- `surf` is replaced by **`reqwest`** everywhere, with one shared pooled client:
  the rest of the daemon is tokio, and surf dragged an async-std runtime in
  beside it.

#### Elsewhere
- Sqlite runs in **WAL** mode. In rollback-journal mode a commit briefly locks
  out every reader, and with background analysis committing per track that is
  thousands of small stalls across every screen.
- Search is **debounced** in both clients, so typing does not fire a query per
  keystroke.
- Loading skeletons on every Slint screen, a content-loader animation at the
  bottom during infinite scroll, and shuffle buttons on playlist rows and
  playlist detail.
- The full-screen player's scrim is tinted with the **window colour** rather
  than a fixed near-black: every control on that canvas is themed, and a dark
  scrim under a light skin left all of them dark on dark.

#### The web UI now matches the desktop app
- The browser client is a port of the Slint desktop UI rather than a separate
  design: the five skins from `desktop/skins/*.toml` (Synthwave, Late Night,
  Neutron, Lunar, Porcelain) as CSS custom properties under the same token
  names, the same two fonts (Roboto Mono, JetBrains Mono), and React versions
  of `desktop/ui/components.slint` down to the VFD readout, the LED level
  meters, the rotary volume knob and the self-scrolling marquee title. A skin
  picker sits where the desktop's does and persists across visits.
- Rebuilt on **Tailwind CSS v4** and **HeroUI** (React Aria underneath, so
  overlays get focus trapping and dismissal). Base Web, Styletron, styled-components,
  Emotion and the `@styled-icons/*` packs are gone, along with the virtualised
  grid/list and infinite-scroll dependencies they pulled in.
- Responsive: below `lg` the sidebar becomes a bottom tab bar with the overflow
  sections behind a "More" sheet, the player bar drops the VFD and the knob,
  and the queue becomes a full-height sheet instead of a fixed rail.
- Every form is react-hook-form with a zod resolver, including the ones that
  were hand-validated before (edit playlist, new/edit folder, add station).
- The RSQL smart-playlist editor takes its syntax colours from the skin's
  `syntax_*` tokens, the same five the desktop reads out of its skin `.toml`.
- Storybook drops the Styletron/Base Web providers and gains a **Skin** picker
  in the toolbar, plus stories for the new design-system primitives and the
  Extensions page.
- Playlists have their own page (`/playlists`) rather than only appearing in
  the sidebar.
- The fullscreen player is the desktop's: artwork over a blurred copy of
  itself, stopping short of the player bar, which turns translucent underneath
  it and keeps the title and transport rather than showing a second set.
- Both clients use **Roboto Mono** as the interface font with JetBrains Mono
  for the monospaced readouts, replacing Space Grotesk on the desktop.
- The web UI's icons are **generated from `desktop/assets/icons/*.svg`** by
  `webui/musicplayer/scripts/generate-icons.py`, so the two clients cannot
  drift apart a glyph at a time. Tabler covers only what the desktop has no
  icon for.
- Context menus lost their outer padding and the rounded corners on the
  highlighted row, in both clients.
- Bumped to **React 19**, which HeroUI requires.

### Fixed
- **Opening a remote library took a minute of loading skeletons.** The desktop
  fetched albums, then artists, then tracks, each paged to exhaustion, one after
  another — and against a remote server a single listing costs seconds. They are
  independent, so they run together now and the wait is the slowest of them
  rather than the sum: measured against Rocksky, 10.21s to 1.99s.
- **A library scan never finished.** "Needs analysing" meant *has no key* when
  it should have meant *has never been analysed*, and the gap between them is
  every track that analyses fine and yields nothing — a spoken intro, something
  atonal. Those kept a null key for ever, so every page offered them again, the
  cached result came back instantly, and the counter climbed past the size of
  the library. Tracks that could not be analysed at all jammed the head of the
  list for the same reason; pages are offset past them now.
- **`track.key` and `track.bpm` stayed empty on a rescan.** Those columns are
  newer than the analysis rows, and a cached analysis returned before the code
  that writes them. A scan now starts by copying stored analysis onto the rows
  missing it — one statement, no decoding.
- **The desktop never showed the signed-in account.** gRPC and http start
  separately and it asked the moment gRPC answered, before anything was
  listening; one attempt, and the failure wrote an empty handle, which reads
  exactly like "nobody is signed in". It retries now, and a failure leaves the
  row alone.
- **`AddTracks` was `unimplemented!()`** — calling it panicked the handler and
  killed the connection. It takes whole tracks rather than ids, so queueing from
  a remote library works: ids would be looked up in a local table that holds
  nothing when a provider is connected.
- Liked tracks stopped at 500. `getStarred2` is unpaged but was going through
  the same limit as the paged endpoints, so hearts were right for the first 500
  and wrong after.
- Infinite scroll stopped at 500 on albums, artists, tracks and liked.
- The web UI's VU meter stopped animating — the GraphQL subscription was not
  reaching it — while the desktop's was fine.
- Repeat and shuffle were never persisted across a restart.
- Adding a remote track to a playlist silently did nothing.
- Album artwork was missing from macOS notifications when playing from a remote
  server.
- Album detail and Liked were empty against a remote server, and opening an
  album fetched its details twice.
- Bitrate and sample rate were dropped for remote tracks, so the readout probed
  the stream for what the server had already said.
- Clippy warnings across the workspace, including locks held across awaits.
- `cargo test` no longer fails to compile: two test fixtures in
  `src/extension.rs` set `readme` twice in the same struct literal, which broke
  the whole `music-player` test binary and so the unit-test CI job.
- The web UI's queue drawer showed `00:00` for every row: a track's duration is
  in seconds and it was being formatted as milliseconds. `Types/Track` now says
  which unit it is, since `NowPlaying` uses the other one.
- The queue and mobile-menu backdrops were exposed to screen readers as a
  second button with the same name as the real close button. They are pointer
  affordances, so they are hidden from assistive tech now; the keyboard route
  out was always the close button.
- The web UI's sidebar showed a red status dot whenever playback was not handed
  to another device, which is the normal case. It tracks whether the daemon is
  reachable now, as the desktop's does.
- Both heart glyphs were drawn on a 20-unit grid inside a 24-unit box, so they
  sat up and to the left of centre inside their button. The liked state also
  rendered as a filled heart either way; it is an outline until liked now.
- The art placeholder's disc glyph scales with its box, so a 150px cover no
  longer shows a tiny one.

## [0.2.1] — 2026-09-06

### Added

#### Internet radio
- Search and browse stations from [Radio Browser](https://www.radio-browser.info)
  and TuneIn, across 33 genre categories, with bookmarks stored locally. Available
  in the web UI, the Slint desktop app and over GraphQL.
- A fullscreen now-playing player in the web UI, opened from the miniplayer
  artwork.
- Station logos in the desktop app: fetched a few at a time after the rows are on
  screen, thumbnailed, and cached on disk between runs. The `image` crate gained
  `gif` and `ico` so the formats directories actually serve will decode.
- Content-loader skeletons while stations load — a shimmer driven by
  `animation-tick()` in Slint, and `react-content-loader` in the web UI. The
  colors are theme tokens (`skeleton_bg` / `skeleton_fg` in every skin,
  `loaderBackground` / `loaderForeground` in the web themes).
- A quick-filter box at the right of the desktop's bookmarked tab and category
  header, filtering the stations already on screen without another request.
- The station playing now can be bookmarked straight from the player bar in both
  clients (new `toggleCurrentRadioBookmark` mutation, which rebuilds the station
  from the queued track so one queued in an earlier session still works).

#### AT Protocol sync
Off unless an account is linked; `atproto = false` in `settings.toml` disables it
outright. See the README for setup.
- Radio bookmarks import from the user's atproto repo and write back to their
  PDS; bookmarks that existed only locally are pushed up.
- Liked songs (`app.rocksky.like`) import and are matched against the library on
  title + artist + album, case-insensitive and indexed, linking rows through a new
  optional `aturi` column on `track` / `album` / `artist`. A like whose file is not
  in the library yet is kept and re-matched after the next scan.
- The station playing now is published as the account's `fm.atradio.actor.status`
  record, written directly to the PDS, and deleted when playback stops.
- Reads pull the whole repo as one CAR archive (`com.atproto.sync.getRepo`) and
  walk its Merkle Search Tree for record paths, falling back to `listRecords`. The
  archive is downloaded once per day and shared between both importers.
- Live updates come from several public Jetstream instances at once, de-duplicated
  by repo revision (`time_us` is stamped per instance and is not comparable).
- Identity resolves from the Rocksky login token, an `atradio login` session, or
  `ATPROTO_IDENTIFIER` / `ATPROTO_APP_PASSWORD`; the daemon logs which one it used,
  or exactly what is missing.

#### Desktop app
- A new [Slint](https://slint.dev) desktop client that talks to a running daemon
  over gRPC and boots the full daemon in-process when nothing is listening.
- Skinnable throughout: five skins ship embedded (Synthwave, Late Night, Neutron,
  Lunar, Porcelain), and extra `.toml` skins dropped in the config directory show
  up in the sidebar switcher.
- A jetAudio-style VFD readout with queue position, codec, bitrate and sample
  rate, plus animated VU meters.
- macOS Now Playing integration, and a synthwave app icon across all platforms.

#### Playback
- Real crossfade and gapless playback: the engine holds the current track plus one
  lookahead so transitions happen in-engine, and the lookahead is re-synced on
  queue mutations (repeat-one deliberately skips it).
- MPRIS media controls on Linux through a pure-Rust backend (souvlaki), so
  `playerctl`, desktop widgets and media keys work.
- Queue persistence, armed only by `PlayerCommand::RestoreQueue` (sent when the
  daemon boots), so tests and `music-player open` never touch the saved queue.

#### Library
- A **Liked** entry in the web UI sidebar and a `/liked` page, backed by a new
  `likedTracks` query.
- Play and shuffle buttons beside the desktop's "Liked tracks" title.
- Artist pictures batch-filled from the Rocksky API after every scan (new
  `artist.picture` column).
- Heart clicks sync to Rocksky from both the web UI and the desktop app.
- `bitrate` and `sample_rate` added to the `Track` proto.

#### Packaging
- `install.sh` for curl-based installation, Linux packages published to Gemfury,
  and the npm package renamed to `@tsiry/music-player`.

### Changed
- Web UI sidebar icons now match the desktop app.
- Release workflows rewritten for Tauri 2 and without fluentci.
- The desktop VFD reads `RADIO` for a live stream instead of counting elapsed
  time, and the web miniplayer shows the station logo (or a shared placeholder)
  in place of album art, with a `LIVE` readout instead of a seek bar.
- Album art in the desktop is keyed by album id rather than list position, and its
  failures are logged with the album and URL.
- Radio and liked pages in the web UI scroll as a whole page.

### Fixed
- **Album art never loaded in the desktop app.** When the desktop boots the daemon
  in-process, gRPC listens before the HTTP server serving `/covers/` does, and the
  library — with all its cover fetches — landed in that window. Every cover failed
  in one burst of connection-refused and never retried, leaving the whole grid on
  placeholders. Art now waits for the cover endpoint to answer first.
- **The desktop ran out of file descriptors and its rpc thread panicked**
  (`Too many open files`). A bookmark lookup on the once-a-second now-playing poll
  called `Database::new()` every tick, and each call opens its own sqlite pool.
  There is now a single process-wide handle, `music_player_storage::shared()`.
  The same mistake in the atproto reads — a `reqwest::Client` per request and a
  `plc.directory` lookup per record — is fixed with a shared client and a memoized
  `pds_endpoint`.
- **Radio bookmark and like imports failed with `RecordNotFound`.** `saved_radio`,
  `rocksky_like` and `atproto_repo_sync` all have primary keys we assign, so
  sea-orm's `save()` always took a new row for an existing one and issued an
  `UPDATE` matching nothing. All three use an explicit on-conflict upsert.
- **atradio writes stopped once the stored access token expired.** The retry called
  `login_password`, which resumes an existing session when it finds one, so it
  handed back the same dead token and reported success. The session is now dropped
  before signing in again, and the daemon refreshes at startup.
- The fullscreen miniplayer left a 96px gap at the bottom of the viewport, and its
  text and icons were unreadable over the dark backdrop in the light theme.
- Internet radio could not be scrolled vertically in the web UI: `index.css` pins
  `body { overflow-y: hidden }`, so every page must bring its own scroller and this
  one had none.
- The desktop VFD stayed at `00:00` for radio, because the elapsed clock was
  clamped to a length a live stream does not have.
- Radio bookmark buttons in the web UI now use the same heart icons as the rest of
  the app, and stations without a logo get the shared placeholder everywhere.
- Artist pictures never reached the web UI from the local library.
- Desktop metadata and album associations.

## [0.2.0] — 2026-09-05

Stack modernization: the Rockbox playback engine, SQLite FTS5 search, a ratatui
TUI and a Tauri 2 desktop app.

[0.4.0]: https://github.com/tsirysndr/music-player/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/tsirysndr/music-player/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/tsirysndr/music-player/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/tsirysndr/music-player/releases/tag/v0.2.0
