//! Mirroring the player's own SQLite data into DuckDB.
//!
//! Deliberately *not* done with DuckDB's `sqlite_scanner`: that extension is
//! not part of the prebuilt static release, so loading it would mean an
//! `INSTALL sqlite` download at runtime — a network dependency on a code path
//! that has to work on a laptop that has never been online. Reading through
//! sea-orm costs a few milliseconds at these volumes and keeps the rule that
//! DuckDB only ever *reads* the player's data.

use anyhow::{Context, Result};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};

use crate::Analytics;

/// What a sync moved.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SyncReport {
    /// Listens appended from `play_history`.
    pub listens: u64,
    /// Rows refreshed in the `tracks` dimension.
    pub tracks: u64,
}

/// Bring DuckDB up to date with SQLite.
///
/// Incremental over `play_history`, which is append-only: everything above the
/// stored watermark is new. The `tracks` dimension is refreshed wholesale
/// instead, because a retag or a fresh analysis rewrites rows in place and
/// there is no watermark that would catch it — at a few thousand tracks that
/// is cheaper than tracking it properly.
pub async fn sync(analytics: &Analytics, db: &DatabaseConnection) -> Result<SyncReport> {
    let tracks = sync_tracks(analytics, db).await?;
    let listens = sync_listens(analytics, db).await?;
    analytics.refresh_dedup()?;
    Ok(SyncReport { listens, tracks })
}

/// The highest `play_history.id` already mirrored.
fn watermark(analytics: &Analytics) -> Result<i64> {
    let cursor: Option<String> = analytics
        .conn()
        .query_row(
            "SELECT cursor FROM import_state WHERE origin = 'local'",
            [],
            |row| row.get(0),
        )
        .optional()?
        .flatten();
    Ok(cursor.and_then(|c| c.parse().ok()).unwrap_or(0))
}

async fn sync_listens(analytics: &Analytics, db: &DatabaseConnection) -> Result<u64> {
    let since = watermark(analytics)?;

    // The local `track` row is preferred over the snapshot columns because it
    // follows retags, exactly as the SQLite analytics views do; the snapshot
    // is what remote plays have instead of a joinable row.
    let rows = db
        .query_all_raw(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            r#"
            SELECT h.id, h.track_id, h.played_at, h.ms_played, h.length_ms,
                   h.skipped, h.source,
                   COALESCE(NULLIF(t.title, ''), h.title)   AS title,
                   COALESCE(NULLIF(t.artist, ''), h.artist) AS artist
            FROM play_history h
            LEFT JOIN track t ON t.id = h.track_id
            WHERE h.id > ?
            ORDER BY h.id
            "#,
            [since.into()],
        ))
        .await
        .context("could not read play_history")?;

    let mut staging = crate::staging::Staging::new(analytics);
    let mut highest = since;
    for row in &rows {
        let id: i64 = row.try_get("", "id")?;
        let track_id: Option<String> = row.try_get("", "track_id").ok();
        let played_at: i64 = row.try_get("", "played_at")?;
        let ms_played: i64 = row.try_get("", "ms_played").unwrap_or(0);
        let length_ms: i64 = row.try_get("", "length_ms").unwrap_or(0);
        let skipped: i64 = row.try_get("", "skipped").unwrap_or(0);
        let source: String = row.try_get("", "source").unwrap_or_default();
        let title: String = row.try_get("", "title").unwrap_or_default();
        let artist: String = row.try_get("", "artist").unwrap_or_default();

        // A listen with no title is unusable: it cannot be keyed, grouped or
        // displayed. Skipping it is better than a row of blanks in every chart.
        if title.trim().is_empty() {
            highest = highest.max(id);
            continue;
        }

        staging.push(crate::staging::Row {
            origin: "local".into(),
            origin_key: id.to_string(),
            played_at: crate::staging::When::Epoch(played_at),
            track_id,
            title,
            artist,
            ms_played: Some(ms_played),
            length_ms: Some(length_ms),
            skipped: Some(skipped != 0),
            source,
            ..Default::default()
        })?;
        highest = highest.max(id);
    }
    staging.flush()?;

    if highest > since {
        record_cursor(analytics, "local", &highest.to_string(), staging.inserted)?;
    }
    Ok(staging.inserted)
}

async fn sync_tracks(analytics: &Analytics, db: &DatabaseConnection) -> Result<u64> {
    // `track_analysis` keys on (source, track_id) and the local library is the
    // empty source; joining without that filter would attach a remote server's
    // analysis to a local track that happens to share an id.
    let rows = db
        .query_all_raw(Statement::from_string(
            DatabaseBackend::Sqlite,
            r#"
            SELECT t.id, t.title, t.artist, t.genre, t.year, t.duration,
                   al.title AS album,
                   al.cover AS cover,
                   COALESCE(t.bpm, a.bpm)  AS bpm,
                   COALESCE(t.key, a.key)  AS song_key,
                   a.valence, a.arousal, a.moods, a.lufs
            FROM track t
            LEFT JOIN album al ON al.id = t.album_id
            LEFT JOIN track_analysis a
                   ON a.track_id = t.id AND (a.source = '' OR a.source = 'local')
            "#,
        ))
        .await
        .context("could not read the track library")?;

    let mut statement = analytics.conn().prepare(
        "INSERT OR REPLACE INTO tracks
           (track_id, title, artist, album, genre, year, duration_ms,
            bpm, song_key, valence, arousal, moods, lufs, cover, match_key)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, match_key(?, ?))",
    )?;

    let mut count = 0u64;
    for row in &rows {
        let id: String = row.try_get("", "id")?;
        let title: String = row.try_get("", "title").unwrap_or_default();
        let artist: String = row.try_get("", "artist").unwrap_or_default();
        let album: Option<String> = row.try_get("", "album").ok();
        let genre: Option<String> = row.try_get("", "genre").ok();
        let year: Option<i32> = row.try_get("", "year").ok();
        // The library stores seconds as a float; everything downstream counts
        // in milliseconds.
        let duration_ms = row
            .try_get::<Option<f64>>("", "duration")
            .ok()
            .flatten()
            .map(|seconds| (seconds * 1000.0) as i64);
        let bpm: Option<f64> = row.try_get("", "bpm").ok().flatten();
        let song_key: Option<String> = row.try_get("", "song_key").ok().flatten();
        let valence: Option<f64> = row.try_get("", "valence").ok().flatten();
        let arousal: Option<f64> = row.try_get("", "arousal").ok().flatten();
        let moods: Option<String> = row.try_get("", "moods").ok().flatten();
        let lufs: Option<f64> = row.try_get("", "lufs").ok().flatten();
        let cover: Option<String> = row.try_get("", "cover").ok().flatten();

        statement.execute(duckdb::params![
            id,
            title,
            artist,
            album,
            genre,
            year,
            duration_ms,
            bpm,
            song_key,
            valence,
            arousal,
            moods,
            lufs,
            cover,
            artist,
            title,
        ])?;
        count += 1;
    }
    Ok(count)
}

/// Remember how far an origin has been ingested.
pub(crate) fn record_cursor(
    analytics: &Analytics,
    origin: &str,
    cursor: &str,
    added: u64,
) -> Result<()> {
    analytics.conn().execute(
        "INSERT OR REPLACE INTO import_state (origin, cursor, rows, updated_at)
         VALUES (?, ?, coalesce((SELECT rows FROM import_state WHERE origin = ?), 0) + ?, now())",
        duckdb::params![origin, cursor, origin, added as i64],
    )?;
    Ok(())
}

/// `query_row` returns an error for "no rows"; this turns that into `None`.
trait Optional<T> {
    fn optional(self) -> Result<Option<T>>;
}

impl<T> Optional<T> for std::result::Result<T, duckdb::Error> {
    fn optional(self) -> Result<Option<T>> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(duckdb::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
