//! The questions the engine can answer.
//!
//! Each function is one question, returning plain serialisable data so the
//! CLI, the gRPC service and the MCP tools all render the same numbers rather
//! than each computing their own slightly different version.
//!
//! Everything reads [`crate::sql`]'s `enriched_listens` or `session_listens`,
//! which means everything is automatically deduplicated across origins and
//! enriched with whatever metadata could be resolved. A query that reached
//! `listens` directly would double-count a history imported from two sources.

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::Analytics;

/// The period a question is asked about.
///
/// `None` on both ends means "everything", which for an imported history can
/// be a decade — worth remembering when a caller renders the result.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Window {
    /// Unix seconds, inclusive.
    pub since: Option<i64>,
    /// Unix seconds, exclusive.
    pub until: Option<i64>,
}

impl Window {
    pub fn all() -> Self {
        Self::default()
    }

    /// The last `days` days.
    pub fn last_days(days: i64) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            since: Some(now - days * 86_400),
            until: None,
        }
    }

    /// A `WHERE`-ready predicate over `played_at`.
    ///
    /// Inlined rather than bound because these clauses are composed into
    /// larger queries and CTEs; the values are `i64`, so there is nothing to
    /// escape.
    fn predicate(&self) -> String {
        let mut clauses = vec!["TRUE".to_string()];
        if let Some(since) = self.since {
            clauses.push(format!("played_at >= to_timestamp({since})"));
        }
        if let Some(until) = self.until {
            clauses.push(format!("played_at < to_timestamp({until})"));
        }
        clauses.join(" AND ")
    }
}

/// Which listens a question is about: a period, and whether to count only
/// music that is in the local library.
///
/// Separate from [`Window`] because it is a different dimension — "this year"
/// and "only what I own" restrict along different axes and compose freely.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scope {
    pub window: Window,
    /// Restrict to tracks the local library has.
    ///
    /// Matched on the normalised title+artist key rather than on the listen
    /// carrying a local track id, so a play imported from Spotify still
    /// counts when the same track sits in the library. The question this
    /// answers is "my history, for music I actually own", not "plays that
    /// happened to go through this player".
    pub local_only: bool,
}

impl Scope {
    pub fn all() -> Self {
        Self::default()
    }
}

impl From<Window> for Scope {
    fn from(window: Window) -> Self {
        Self {
            window,
            local_only: false,
        }
    }
}

impl Scope {
    /// A `WHERE`-ready predicate over `enriched_listens`.
    fn predicate(&self) -> String {
        let mut clause = self.window.predicate();
        if self.local_only {
            clause.push_str(
                " AND match_key IN \
                 (SELECT match_key FROM tracks WHERE match_key IS NOT NULL)",
            );
        }
        clause
    }
}

/// The headline numbers.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Overview {
    pub listens: i64,
    pub distinct_tracks: i64,
    pub distinct_artists: i64,
    pub distinct_albums: i64,
    /// Total time with something playing.
    ///
    /// A floor, not an estimate, and deliberately so at both ends: a source
    /// that records no duration (a bare scrobble) contributes nothing, and a
    /// single listen contributes at most one track's length — see
    /// `ms_counted` in [`crate::sql::DEDUP`] for why a radio stream would
    /// otherwise dominate the figure.
    pub hours_played: f64,
    pub sessions: i64,
    pub first_listen: Option<String>,
    pub last_listen: Option<String>,
    /// Days with at least one listen.
    pub active_days: i64,
    /// The longest run of consecutive active days.
    pub longest_streak: i64,
}

pub fn overview(a: &Analytics, scope: Scope) -> Result<Overview> {
    let filter = scope.predicate();

    let mut overview: Overview = a.conn().query_row(
        &format!(
            r#"
            SELECT count(*),
                   count(DISTINCT match_key),
                   count(DISTINCT artist),
                   count(DISTINCT album),
                   coalesce(sum(ms_counted), 0) / 3600000.0,
                   coalesce(min(played_at)::VARCHAR, ''),
                   coalesce(max(played_at)::VARCHAR, ''),
                   count(DISTINCT played_at::DATE)
            FROM enriched_listens WHERE {filter}
            "#
        ),
        [],
        |row| {
            Ok(Overview {
                listens: row.get(0)?,
                distinct_tracks: row.get(1)?,
                distinct_artists: row.get(2)?,
                distinct_albums: row.get(3)?,
                hours_played: row.get(4)?,
                first_listen: non_empty(row.get::<_, String>(5)?),
                last_listen: non_empty(row.get::<_, String>(6)?),
                active_days: row.get(7)?,
                ..Default::default()
            })
        },
    )?;

    overview.sessions = a.conn().query_row(
        &format!("SELECT count(DISTINCT session_id) FROM session_listens WHERE {filter}"),
        [],
        |row| row.get(0),
    )?;

    // Consecutive active days: number the distinct days, and subtract the row
    // number from the date. Days in the same unbroken run give the same
    // result, so the size of the largest group is the longest streak.
    overview.longest_streak = a.conn().query_row(
        &format!(
            r#"
            WITH days AS (
              SELECT DISTINCT played_at::DATE AS day FROM enriched_listens WHERE {filter}
            ),
            grouped AS (
              SELECT day, day - (row_number() OVER (ORDER BY day))::INTEGER AS run FROM days
            )
            SELECT coalesce(max(n), 0) FROM (SELECT count(*) AS n FROM grouped GROUP BY run)
            "#
        ),
        [],
        |row| row.get(0),
    )?;

    Ok(overview)
}

/// One row of a leaderboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rank {
    pub name: String,
    /// Artist for a track row, empty for an artist row.
    pub secondary: String,
    pub listens: i64,
    pub hours: f64,
    pub last_played: Option<String>,
    /// Artwork to show beside the row: the album cover for a track or album,
    /// the artist's picture for an artist. A local library cover is a bare
    /// filename for the daemon's cover server; anything resolved from the
    /// catalogue is an absolute URL. `None` means draw the placeholder.
    pub art: Option<String>,
}

/// What a leaderboard is over.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TopKind {
    Artists,
    Tracks,
    Albums,
    Genres,
}

pub fn top(a: &Analytics, kind: TopKind, scope: Scope, limit: u32) -> Result<Vec<Rank>> {
    let filter = scope.predicate();
    // Group by the *normalised* key so "Boards of Canada" and "boards of
    // canada" are one artist, but display the spelling seen most often.
    let (group, name, secondary) = match kind {
        TopKind::Artists => ("primary_artist(artist)", "artist", "''"),
        TopKind::Tracks => ("match_key", "title", "artist"),
        TopKind::Albums => ("lower(coalesce(album, ''))", "album", "artist"),
        TopKind::Genres => ("lower(coalesce(genre, ''))", "genre", "''"),
    };
    // An artist wears their own face; everything else wears its cover. A
    // genre has neither.
    let art = match kind {
        TopKind::Artists => {
            "(SELECT a.picture FROM artist_art a \
              WHERE a.artist_key = primary_artist(mode(l.artist)) AND a.resolved)"
        }
        TopKind::Genres => "NULL",
        _ => "any_value(l.cover)",
    };

    let mut statement = a.conn().prepare(&format!(
        r#"
        SELECT mode({name}) AS name,
               mode({secondary}) AS secondary,
               count(*) AS listens,
               coalesce(sum(ms_counted), 0) / 3600000.0 AS hours,
               max(played_at)::VARCHAR AS last_played,
               {art} AS art
        FROM enriched_listens l
        WHERE {filter} AND nullif(trim({group}), '') IS NOT NULL
        GROUP BY {group}
        ORDER BY listens DESC, last_played DESC
        LIMIT ?
        "#
    ))?;

    let rows = statement.query_map([limit as i64], |row| {
        Ok(Rank {
            name: row.get(0)?,
            secondary: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            listens: row.get(2)?,
            hours: row.get(3)?,
            last_played: non_empty(row.get::<_, String>(4)?),
            art: row.get(5)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<_, _>>()?)
}

/// One cell of the listening clock.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockCell {
    /// 0 = Sunday, matching DuckDB's `dayofweek`.
    pub weekday: i32,
    pub hour: i32,
    pub listens: i64,
    pub avg_bpm: Option<f64>,
    pub avg_valence: Option<f64>,
}

/// When listening happens, and what it sounds like at that hour.
///
/// In local time deliberately: "what do I put on at 7am" is a question about
/// the user's morning, not about UTC.
pub fn clock(a: &Analytics, scope: Scope) -> Result<Vec<ClockCell>> {
    let filter = scope.predicate();
    let mut statement = a.conn().prepare(&format!(
        r#"
        SELECT dayofweek(played_at)::INTEGER AS weekday,
               hour(played_at)::INTEGER      AS hour,
               count(*)                      AS listens,
               avg(bpm), avg(valence)
        FROM enriched_listens WHERE {filter}
        GROUP BY weekday, hour
        ORDER BY weekday, hour
        "#
    ))?;
    let rows = statement.query_map([], |row| {
        Ok(ClockCell {
            weekday: row.get(0)?,
            hour: row.get(1)?,
            listens: row.get(2)?,
            avg_bpm: row.get(3)?,
            avg_valence: row.get(4)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<_, _>>()?)
}

/// How sessions behave.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub sessions: i64,
    pub avg_tracks: f64,
    pub median_minutes: f64,
    pub longest_minutes: f64,
    /// Tracks that most often open a session — what gets reached for first.
    pub openers: Vec<Rank>,
}

pub fn session_stats(a: &Analytics, scope: Scope, limit: u32) -> Result<SessionStats> {
    let filter = scope.predicate();

    let mut stats: SessionStats = a.conn().query_row(
        &format!(
            r#"
            SELECT count(*), coalesce(avg(tracks), 0),
                   coalesce(median(minutes), 0), coalesce(max(minutes), 0)
            FROM sessions
            WHERE session_id IN (SELECT session_id FROM session_listens WHERE {filter})
            "#
        ),
        [],
        |row| {
            Ok(SessionStats {
                sessions: row.get(0)?,
                avg_tracks: row.get(1)?,
                median_minutes: row.get(2)?,
                longest_minutes: row.get(3)?,
                openers: Vec::new(),
            })
        },
    )?;

    let mut statement = a.conn().prepare(&format!(
        r#"
        SELECT mode(title), mode(artist), count(*), 0.0, max(played_at)::VARCHAR,
               any_value(cover)
        FROM session_listens
        WHERE {filter} AND position_in_session = 1
        GROUP BY match_key
        ORDER BY count(*) DESC
        LIMIT ?
        "#
    ))?;
    let rows = statement.query_map([limit as i64], |row| {
        Ok(Rank {
            name: row.get(0)?,
            secondary: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            listens: row.get(2)?,
            hours: row.get(3)?,
            last_played: non_empty(row.get::<_, String>(4)?),
            art: row.get(5)?,
        })
    })?;
    stats.openers = rows.collect::<std::result::Result<_, _>>()?;
    Ok(stats)
}

/// What gets abandoned, and how far in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipRow {
    pub name: String,
    pub secondary: String,
    pub listens: i64,
    pub skips: i64,
    pub skip_rate: f64,
    /// Median fraction of the track heard before moving on.
    pub median_completion: Option<f64>,
    /// Album cover, as in [`Rank::art`].
    pub art: Option<String>,
}

/// Skip forensics.
///
/// Only sources that record how much was played can answer this — our own
/// player and Spotify. A Last.fm scrobble is submitted *after* a track was
/// listened to, so counting its rows here would dilute every rate toward zero
/// and make the worst offenders look fine.
pub fn skips(a: &Analytics, scope: Scope, limit: u32, min_listens: i64) -> Result<Vec<SkipRow>> {
    let filter = scope.predicate();
    let mut statement = a.conn().prepare(&format!(
        r#"
        WITH measurable AS (
          SELECT artist, title, match_key, ms_played, duration_ms, skipped, reason_end, cover
          FROM enriched_listens
          WHERE {filter} AND origin IN ('local', 'spotify')
            -- An internet radio stream records one row per announced song
            -- with no duration, and every one of them looks "skipped" when
            -- the station moves on. Left in, station idents and adverts take
            -- every top place with a 100% rate and bury the real answer.
            -- A skip is only meaningful against a track of known length;
            -- Spotify is exempt because it states outright why a track ended.
            AND (origin = 'spotify' OR coalesce(duration_ms, 0) > 0)
        )
        SELECT mode(title), mode(artist), count(*) AS listens,
               -- Spotify's own reason beats any duration heuristic: `fwdbtn`
               -- is the user pressing next, which is what a skip *is*.
               count(*) FILTER (skipped OR reason_end = 'fwdbtn') AS skips,
               count(*) FILTER (skipped OR reason_end = 'fwdbtn') * 1.0 / count(*) AS rate,
               median(CASE WHEN coalesce(duration_ms, 0) > 0
                           THEN least(ms_played * 1.0 / duration_ms, 1.0) END) AS completion,
               any_value(cover) AS art
        FROM measurable
        GROUP BY match_key
        HAVING count(*) >= ?
        ORDER BY rate DESC, listens DESC
        LIMIT ?
        "#
    ))?;
    let rows = statement.query_map([min_listens, limit as i64], |row| {
        Ok(SkipRow {
            name: row.get(0)?,
            secondary: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            listens: row.get(2)?,
            skips: row.get(3)?,
            skip_rate: row.get(4)?,
            median_completion: row.get(5)?,
            art: row.get(6)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<_, _>>()?)
}

/// One time bucket of the taste timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftPoint {
    pub bucket: String,
    pub listens: i64,
    pub avg_bpm: Option<f64>,
    pub avg_valence: Option<f64>,
    pub avg_arousal: Option<f64>,
    /// Median release year of what was played — how old the music is.
    pub median_year: Option<f64>,
    /// Share of listens that were of a track never heard before.
    pub discovery_rate: f64,
}

/// How taste moves over time.
///
/// `bucket` is a DuckDB date part: `week`, `month`, `quarter`, `year`.
pub fn drift(a: &Analytics, scope: Scope, bucket: &str) -> Result<Vec<DriftPoint>> {
    // Interpolated into SQL, so it must not come straight from a caller.
    let bucket = match bucket {
        "day" | "week" | "month" | "quarter" | "year" => bucket,
        other => anyhow::bail!("unsupported bucket '{other}' (day, week, month, quarter or year)"),
    };
    let filter = scope.predicate();

    let mut statement = a.conn().prepare(&format!(
        r#"
        WITH first_seen AS (
          SELECT match_key, min(played_at) AS first_at FROM enriched_listens GROUP BY match_key
        )
        SELECT date_trunc('{bucket}', l.played_at)::DATE::VARCHAR AS bucket,
               count(*),
               avg(l.bpm), avg(l.valence), avg(l.arousal),
               median(l.year),
               count(*) FILTER (l.played_at = f.first_at) * 1.0 / count(*)
        FROM enriched_listens l
        JOIN first_seen f USING (match_key)
        WHERE {filter}
        GROUP BY bucket
        ORDER BY bucket
        "#
    ))?;
    let rows = statement.query_map([], |row| {
        Ok(DriftPoint {
            bucket: row.get(0)?,
            listens: row.get(1)?,
            avg_bpm: row.get(2)?,
            avg_valence: row.get(3)?,
            avg_arousal: row.get(4)?,
            median_year: row.get(5)?,
            discovery_rate: row.get(6)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<_, _>>()?)
}

/// An observed track-to-track transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub from_title: String,
    pub from_artist: String,
    pub to_title: String,
    pub to_artist: String,
    pub times: i64,
}

/// What actually follows what, within a session.
///
/// This is the transition graph the player's own history contains — better
/// input for an automatic mix than tempo matching alone, because it encodes
/// choices that were actually made and enjoyed rather than ones that merely
/// beat-match.
pub fn transitions(a: &Analytics, scope: Scope, limit: u32) -> Result<Vec<Transition>> {
    let filter = scope.predicate();
    let mut statement = a.conn().prepare(&format!(
        r#"
        WITH pairs AS (
          SELECT title, artist, match_key,
                 lead(title)     OVER w AS next_title,
                 lead(artist)    OVER w AS next_artist,
                 lead(match_key) OVER w AS next_key
          FROM session_listens
          WHERE {filter}
          WINDOW w AS (PARTITION BY session_id ORDER BY played_at)
        )
        SELECT mode(title), mode(artist), mode(next_title), mode(next_artist), count(*)
        FROM pairs
        WHERE next_key IS NOT NULL
          -- A track repeating itself is a loop, not a transition.
          AND next_key <> match_key
        GROUP BY match_key, next_key
        ORDER BY count(*) DESC
        LIMIT ?
        "#
    ))?;
    let rows = statement.query_map([limit as i64], |row| {
        Ok(Transition {
            from_title: row.get(0)?,
            from_artist: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            to_title: row.get(2)?,
            to_artist: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
            times: row.get(4)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<_, _>>()?)
}

/// How concentrated listening is, and what is going unplayed.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Rotation {
    /// Share of all listens taken by the top 1% of tracks.
    pub top_1_percent_share: f64,
    pub top_10_percent_share: f64,
    /// Gini coefficient over per-track play counts: 0 is perfectly even, 1 is
    /// everything on one track.
    pub gini: f64,
    /// Local library tracks, and how many have ever been played.
    pub library_tracks: i64,
    pub library_played: i64,
    /// Tracks once played often, not heard in a long time.
    pub rediscover: Vec<Rank>,
}

pub fn rotation(a: &Analytics, scope: Scope, limit: u32) -> Result<Rotation> {
    let filter = scope.predicate();

    let mut rotation: Rotation = a.conn().query_row(
        &format!(
            r#"
            WITH counts AS (
              SELECT match_key, count(*) AS n FROM enriched_listens WHERE {filter} GROUP BY match_key
            ),
            ranked AS (
              SELECT n,
                     row_number() OVER (ORDER BY n DESC) AS rank,
                     count(*)     OVER ()                AS tracks,
                     sum(n)       OVER ()                AS total
              FROM counts
            ),
            -- Gini from the Lorenz curve: with counts sorted ascending,
            -- (2*i - N - 1) weights each share by how far it sits from the
            -- middle of the distribution.
            lorenz AS (
              SELECT n, row_number() OVER (ORDER BY n ASC) AS i,
                     count(*) OVER () AS tracks, sum(n) OVER () AS total
              FROM counts
            )
            SELECT
              coalesce(sum(n) FILTER (rank <= greatest(tracks / 100, 1)) * 1.0 / nullif(max(total), 0), 0),
              coalesce(sum(n) FILTER (rank <= greatest(tracks / 10, 1))  * 1.0 / nullif(max(total), 0), 0),
              -- `tracks` and `total` come from an unbounded window, so they
              -- are the same on every row but still have to be aggregated to
              -- be used alongside a sum.
              coalesce((SELECT sum((2 * i - tracks - 1) * n) * 1.0
                               / nullif(max(tracks) * max(total), 0) FROM lorenz), 0)
            FROM ranked
            "#
        ),
        [],
        |row| {
            Ok(Rotation {
                top_1_percent_share: row.get(0)?,
                top_10_percent_share: row.get(1)?,
                gini: row.get(2)?,
                ..Default::default()
            })
        },
    )?;

    // Library coverage is about the local library only: an imported Spotify
    // history has no library to be a share of.
    let (library_tracks, library_played): (i64, i64) = a.conn().query_row(
        "SELECT count(*),
                count(*) FILTER (track_id IN (SELECT track_id FROM enriched_listens
                                              WHERE track_id IS NOT NULL))
         FROM tracks",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    rotation.library_tracks = library_tracks;
    rotation.library_played = library_played;

    // Loved once, untouched since: played enough to have meant something, and
    // silent for longer than a season.
    let mut statement = a.conn().prepare(
        "SELECT mode(title), mode(artist), count(*), 0.0, max(played_at)::VARCHAR,
                any_value(cover)
         FROM enriched_listens
         GROUP BY match_key
         HAVING count(*) >= 5 AND max(played_at) < now() - INTERVAL 180 DAY
         ORDER BY count(*) DESC
         LIMIT ?",
    )?;
    let rows = statement.query_map([limit as i64], |row| {
        Ok(Rank {
            name: row.get(0)?,
            secondary: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            listens: row.get(2)?,
            hours: row.get(3)?,
            last_played: non_empty(row.get::<_, String>(4)?),
            art: row.get(5)?,
        })
    })?;
    rotation.rediscover = rows.collect::<std::result::Result<_, _>>()?;
    Ok(rotation)
}

/// What was playing on this date in previous years.
pub fn on_this_day(a: &Analytics, limit: u32) -> Result<Vec<Rank>> {
    let mut statement = a.conn().prepare(
        "SELECT mode(title), mode(artist), count(*), 0.0, max(played_at)::VARCHAR,
                any_value(cover)
         FROM enriched_listens
         WHERE month(played_at) = month(now()) AND day(played_at) = day(now())
           AND year(played_at) < year(now())
         GROUP BY match_key
         ORDER BY count(*) DESC, max(played_at) DESC
         LIMIT ?",
    )?;
    let rows = statement.query_map([limit as i64], |row| {
        Ok(Rank {
            name: row.get(0)?,
            secondary: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            listens: row.get(2)?,
            hours: row.get(3)?,
            last_played: non_empty(row.get::<_, String>(4)?),
            art: row.get(5)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<_, _>>()?)
}

/// Where the listens came from — useful mostly for sanity-checking an import.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriginRow {
    pub origin: String,
    pub raw: i64,
    pub canonical: i64,
    pub first: Option<String>,
    pub last: Option<String>,
}

pub fn origins(a: &Analytics) -> Result<Vec<OriginRow>> {
    let mut statement = a.conn().prepare(
        "SELECT l.origin,
                count(*),
                (SELECT count(*) FROM canonical_listens c WHERE c.origin = l.origin),
                min(l.played_at)::VARCHAR, max(l.played_at)::VARCHAR
         FROM listens l GROUP BY l.origin ORDER BY count(*) DESC",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(OriginRow {
            origin: row.get(0)?,
            raw: row.get(1)?,
            canonical: row.get(2)?,
            first: non_empty(row.get::<_, String>(3)?),
            last: non_empty(row.get::<_, String>(4)?),
        })
    })?;
    Ok(rows.collect::<std::result::Result<_, _>>()?)
}

fn non_empty(s: String) -> Option<String> {
    (!s.is_empty()).then_some(s)
}
