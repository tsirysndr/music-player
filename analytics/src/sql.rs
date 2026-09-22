//! The DuckDB schema, as SQL.
//!
//! Kept as text rather than built through a query builder so that the
//! definitions here are the same ones you can paste into `duckdb
//! ~/Library/Application Support/music-player/analytics.duckdb` and read. Every
//! statement is idempotent: `migrate` runs the whole script on each open, and
//! a new release adds statements to the end.

/// Normalisation and matching helpers, and the unified listen table.
///
/// The macros are the interesting part. They encode two things learned from
/// running this against a real nine-year export:
///
/// * Titles must be folded with a *Unicode* class. An earlier `[^a-z0-9]+`
///   erased any title written in Japanese or Cyrillic down to the empty
///   string, collapsing every one of them onto a single bogus key.
/// * Only the **primary** artist credit is comparable across catalogues.
///   Spotify records the album artist alone ("Riton"); Last.fm records the
///   full credit ("Riton, Oliver Heldens, Vula"). Matching on the whole string
///   finds nothing.
pub const SCHEMA: &str = r#"
CREATE OR REPLACE MACRO norm(s) AS
  trim(regexp_replace(
    regexp_replace(
      regexp_replace(lower(coalesce(s, '')),
        '\s*[\(\[](feat|ft|featuring|with)\.?[^\)\]]*[\)\]]', '', 'g'),
      '\s*-\s*(\d{4}\s+)?(remaster(ed)?|remix|radio edit|single version|album version|mono|stereo|live|deluxe|bonus track)([^-]*)$', '', 'g'),
    '[^\p{L}\p{N}]+', ' ', 'g'));

CREATE OR REPLACE MACRO primary_artist(a) AS
  norm(regexp_replace(coalesce(a, ''),
       '\s*(,|&| x | vs\.? | with | feat\.?| ft\.?| featuring ).*$', '', 'i'));

-- Never NULL: a title that normalises away entirely (pure punctuation, say)
-- falls back to its raw text so the listen still joins to itself.
CREATE OR REPLACE MACRO match_key(artist, title) AS
  coalesce(nullif(primary_artist(artist), ''), lower(trim(coalesce(artist, ''))))
  || chr(31) ||
  coalesce(nullif(norm(title), ''), lower(trim(coalesce(title, ''))));

-- Which origin to believe when the same listen arrives twice. Our own player
-- knows the local track id, how much was actually heard and whether it was
-- skipped; Spotify carries ms_played and an explicit reason the track ended;
-- Rocksky and Last.fm carry a bare scrobble.
CREATE OR REPLACE MACRO origin_rank(o) AS
  CASE o WHEN 'local'   THEN 0
         WHEN 'spotify' THEN 1
         WHEN 'rocksky' THEN 2
         WHEN 'lastfm'  THEN 3
         ELSE 4 END;

CREATE TABLE IF NOT EXISTS listens (
  origin       VARCHAR NOT NULL,   -- local | spotify | lastfm | rocksky
  origin_key   VARCHAR NOT NULL,   -- unique within origin; makes re-import idempotent
  played_at    TIMESTAMPTZ NOT NULL,
  track_id     VARCHAR,            -- local library id, when the listen has one
  title        VARCHAR NOT NULL,
  artist       VARCHAR NOT NULL,
  album        VARCHAR,
  ms_played    BIGINT,
  length_ms    BIGINT,
  skipped      BOOLEAN,
  reason_start VARCHAR,
  reason_end   VARCHAR,
  shuffle      BOOLEAN,
  platform     VARCHAR,
  country      VARCHAR,
  source       VARCHAR,            -- device or server the play came from
  match_key    VARCHAR NOT NULL,
  PRIMARY KEY (origin, origin_key)
);

-- The local library, mirrored for joins: titles, and whatever the audio
-- analysis has worked out about each track.
CREATE TABLE IF NOT EXISTS tracks (
  track_id   VARCHAR PRIMARY KEY,
  title      VARCHAR,
  artist     VARCHAR,
  album      VARCHAR,
  genre      VARCHAR,
  year       INTEGER,
  duration_ms BIGINT,
  bpm        DOUBLE,
  song_key   VARCHAR,
  valence    DOUBLE,
  arousal    DOUBLE,
  moods      VARCHAR,
  lufs       DOUBLE,
  -- The album cover file, as the daemon's cover server names it.
  cover      VARCHAR,
  match_key  VARCHAR
);

-- Metadata resolved from Rocksky's matchSong for listens that have no local
-- track: nine years of imported history otherwise carry a title and an artist
-- and nothing else — no album, genre, year or duration to group by.
CREATE TABLE IF NOT EXISTS song_match (
  match_key   VARCHAR PRIMARY KEY,
  resolved    BOOLEAN NOT NULL,   -- false records a miss, so it is not retried forever
  title       VARCHAR,
  artist      VARCHAR,
  album       VARCHAR,
  album_art   VARCHAR,
  genre       VARCHAR,
  year        INTEGER,
  duration_ms BIGINT,
  mb_id       VARCHAR,
  isrc        VARCHAR,
  spotify_link VARCHAR,
  uri         VARCHAR,
  -- `matchSong` returns this alongside the album art; stored so the artist
  -- leaderboards have a face without a second lookup.
  artist_picture VARCHAR,
  matched_at  TIMESTAMPTZ NOT NULL
);

-- Artist pictures, resolved once per artist.
--
-- Separate from `song_match` because it is keyed by artist, not by track, and
-- because rows enriched before `artist_picture` existed carry none — this is
-- what backfills them for the handful of artists a leaderboard actually
-- shows, rather than re-resolving eleven thousand tracks.
CREATE TABLE IF NOT EXISTS artist_art (
  artist_key VARCHAR PRIMARY KEY,   -- primary_artist(artist)
  picture    VARCHAR,
  resolved   BOOLEAN NOT NULL,      -- false records a miss, so it is not retried forever
  matched_at TIMESTAMPTZ NOT NULL
);

-- Staging for row-at-a-time producers (the Rocksky API, the SQLite mirror).
--
-- DuckDB is columnar: a single-row `INSERT` into a table with a primary key
-- costs an index probe and a fresh row group, and doing that 60,000 times
-- takes minutes of CPU. Appending to an unconstrained staging table and then
-- folding it into `listens` with one set-based statement per batch turns that
-- back into the bulk operation the engine is built for.
CREATE TABLE IF NOT EXISTS listens_staging (
  origin       VARCHAR,
  origin_key   VARCHAR,
  played_at    TIMESTAMPTZ,
  track_id     VARCHAR,
  title        VARCHAR,
  artist       VARCHAR,
  album        VARCHAR,
  ms_played    BIGINT,
  length_ms    BIGINT,
  skipped      BOOLEAN,
  source       VARCHAR
);

-- The deduplicated corpus, materialised by [`DEDUP`].
--
-- Declared empty here so the views below can bind on a database that has
-- never had an import run against it. A table rather than a view because the
-- dedup test is a correlated `NOT EXISTS` over every listen: as a view that
-- re-runs on every query that touches it, measured at 4.2s for a bare
-- `count(*)` over 110k rows against 0.019s on the base table — and the
-- Statistics panel issues eight such queries.
CREATE TABLE IF NOT EXISTS canonical_listens (
  origin       VARCHAR,
  origin_key   VARCHAR,
  played_at    TIMESTAMPTZ,
  track_id     VARCHAR,
  title        VARCHAR,
  artist       VARCHAR,
  album        VARCHAR,
  ms_played    BIGINT,
  length_ms    BIGINT,
  skipped      BOOLEAN,
  reason_start VARCHAR,
  reason_end   VARCHAR,
  shuffle      BOOLEAN,
  platform     VARCHAR,
  country      VARCHAR,
  source       VARCHAR,
  match_key    VARCHAR
);

-- Detected clock offsets between sources, filled in by [`DEDUP`].
--
-- Declared here, empty, rather than only created by the detection pass: the
-- views below reference it, and they have to be creatable on a database that
-- has never had an import run against it.
CREATE TABLE IF NOT EXISTS origin_offset (
  lo       VARCHAR,
  hi       VARCHAR,
  offset_s DOUBLE,
  support  BIGINT
);

-- How far each origin has been ingested, so a re-run is incremental.
-- `CREATE TABLE IF NOT EXISTS` does not add a column to a table that already
-- exists, so an older database needs this stated separately. DuckDB has no
-- `ADD COLUMN IF NOT EXISTS`; the statement is expected to fail once the
-- column is there, and `migrate` tolerates that.
-- (applied by Analytics::add_missing_columns)

CREATE TABLE IF NOT EXISTS import_state (
  origin     VARCHAR PRIMARY KEY,
  cursor     VARCHAR,       -- highest play_history.id, last scrobble time, ...
  rows       BIGINT NOT NULL DEFAULT 0,
  updated_at TIMESTAMPTZ NOT NULL
);
"#;

/// Cross-origin deduplication.
///
/// The same listen reaches us from several exports at once, and not at the
/// same instant. Measured against a real library, 11k Last.fm scrobbles sat
/// *exactly three hours* after their Spotify twin, because whatever fed
/// Last.fm submitted local (UTC+3) time as though it were UTC. Unioned
/// naively that inflates a nine-year history by nearly 30% and silently
/// double-counts the most-played tracks.
///
/// The offset is **detected, never assumed**. Hard-coding +3h would be right
/// for one user in one timezone and quietly wrong for everyone else, so we
/// take the modal whole-minute offset between each ordered pair of origins and
/// only trust it when a decent number of pairs agree.
pub const DEDUP: &str = r#"
DELETE FROM origin_offset;
INSERT INTO origin_offset
WITH pairs AS (
  SELECT a.origin AS lo, b.origin AS hi,
         round(epoch(a.played_at - b.played_at) / 60.0) AS off_min
  FROM listens a
  JOIN listens b
    ON a.match_key = b.match_key
   AND origin_rank(a.origin) > origin_rank(b.origin)
   AND abs(epoch(a.played_at - b.played_at)) <= 6 * 3600
),
tally AS (
  SELECT lo, hi, off_min, count(*) AS n,
         row_number() OVER (PARTITION BY lo, hi ORDER BY count(*) DESC) AS rk
  FROM pairs GROUP BY lo, hi, off_min
)
SELECT lo, hi,
       -- Below this much agreement it is chance, not a clock offset; assume
       -- the clocks match and let the tolerance window do the work.
       CASE WHEN n >= 200 THEN off_min * 60 ELSE 0 END AS offset_s,
       n AS support
FROM tally WHERE rk = 1;

CREATE OR REPLACE TABLE canonical_listens AS
SELECT l.*
FROM listens l
WHERE NOT EXISTS (
  SELECT 1
  FROM listens b
  JOIN origin_offset o ON o.lo = l.origin AND o.hi = b.origin
  WHERE origin_rank(b.origin) < origin_rank(l.origin)
    AND b.match_key = l.match_key
    AND abs(epoch(l.played_at - b.played_at) - o.offset_s) <= 240
);
"#;

/// The derived views.
///
/// Split from [`DEDUP`] because they are cheap to redefine and must always
/// match the code that reads them, while the offset detection above is a
/// self-join over the whole corpus. Applied on every read-write open, so an
/// upgraded release picks up a changed view definition without the user
/// having to know to re-run anything.
pub const VIEWS: &str = r#"
-- Every canonical listen with whatever is known about the track, wherever it
-- came from: the local library first, then a resolved Rocksky match.
CREATE OR REPLACE VIEW enriched_listens AS
SELECT l.origin, l.played_at, l.track_id, l.title, l.artist,
       coalesce(l.album, t.album, m.album)          AS album,
       -- The catalogue returns the literal string "None" for a track it has
       -- no genre for, which otherwise ranks as a genre in its own right.
       nullif(nullif(trim(coalesce(t.genre, m.genre)), ''), 'None') AS genre,
       coalesce(t.year, m.year)                     AS year,
       coalesce(l.length_ms, t.duration_ms, m.duration_ms) AS duration_ms,
       -- A local cover is a bare filename for the daemon's cover server; a
       -- resolved one is an absolute URL. Both are fetchable, and the caller
       -- does not need to know which it got.
       coalesce(t.cover, m.album_art)               AS cover,
       l.ms_played,
       -- What a listen is allowed to contribute to "time played".
       --
       -- An internet radio stream is recorded as one row per announced song,
       -- but `ms_played` keeps counting for the whole stream *session*, so a
       -- single announcement can claim eight hours. Measured on a real
       -- library that put 2,056 of 3,705 reported hours onto one station —
       -- more than half the total — and pinned 320 hours to a "track" called
       -- Advertisement.
       --
       -- So a listen contributes at most the track's own length. Where no
       -- length is known (a Spotify export carries none) it is capped at
       -- twenty minutes: the longest genuine single play in that same export
       -- was eighteen. This undercounts a legitimately long mix and a long
       -- radio sitting, and that is the better error — the alternative
       -- silently doubles the headline figure.
       least(coalesce(l.ms_played, 0),
             coalesce(nullif(l.length_ms, 0), nullif(t.duration_ms, 0),
                      nullif(m.duration_ms, 0), 1200000)) AS ms_counted,
       l.skipped, l.reason_end, l.shuffle, l.source,
       t.bpm, t.song_key, t.valence, t.arousal, t.lufs,
       l.match_key
FROM canonical_listens l
LEFT JOIN tracks     t ON t.track_id  = l.track_id
LEFT JOIN song_match m ON m.match_key = l.match_key AND m.resolved;

-- ── sessions ────────────────────────────────────────────────────────────────
-- A listening session is a run of listens with no long silence in it. This is
-- the unit most questions are really about: "what do I put on first", "how
-- long do I listen for", "what follows what" are all session questions, and
-- none of them can be answered from a play count.
--
-- Thirty minutes because it is long enough to survive making a coffee and
-- short enough that the morning and the evening do not merge into one.
CREATE OR REPLACE VIEW session_listens AS
WITH ordered AS (
  SELECT *,
         lag(played_at) OVER (ORDER BY played_at) AS previous_at
  FROM enriched_listens
),
marked AS (
  SELECT *,
         CASE WHEN previous_at IS NULL
                OR epoch(played_at - previous_at) > 1800
              THEN 1 ELSE 0 END AS starts_session
  FROM ordered
),
-- The running sum has to be materialised in its own step before it can be
-- partitioned on: a window function is not allowed inside another window's
-- definition.
numbered AS (
  SELECT *,
         sum(starts_session) OVER (ORDER BY played_at
                                   ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS session_id
  FROM marked
)
SELECT *,
       row_number() OVER (PARTITION BY session_id ORDER BY played_at) AS position_in_session
FROM numbered;

CREATE OR REPLACE VIEW sessions AS
SELECT session_id,
       min(played_at) AS started_at,
       max(played_at) AS ended_at,
       count(*)       AS tracks,
       count(DISTINCT artist) AS artists,
       -- Wall-clock, not the sum of track lengths: the gap between the last
       -- track starting and it finishing is unknown, so this understates by
       -- at most one track rather than inventing a duration.
       epoch(max(played_at) - min(played_at)) / 60.0 AS minutes,
       sum(coalesce(ms_played, 0)) / 60000.0         AS minutes_played,
       avg(bpm)     AS avg_bpm,
       avg(valence) AS avg_valence,
       avg(arousal) AS avg_arousal
FROM session_listens
GROUP BY session_id;
"#;
