# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

This file starts at 0.2.1. For earlier releases see the
[git tags](https://github.com/tsirysndr/music-player/tags).

## [Unreleased]

### Added

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

[0.2.1]: https://github.com/tsirysndr/music-player/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/tsirysndr/music-player/releases/tag/v0.2.0
