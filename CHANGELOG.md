# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

This file starts at 0.2.1. For earlier releases see the
[git tags](https://github.com/tsirysndr/music-player/tags).

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
