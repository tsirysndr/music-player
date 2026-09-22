//! Last.fm exports, CSV or JSON.
//!
//! There is no official export, so the shapes in the wild come from the
//! popular third-party dumpers. Two are handled:
//!
//! * CSV — `uts,utc_time,artist,artist_mbid,album,album_mbid,track,track_mbid`
//! * JSON — the `user.getRecentTracks` response shape, where `artist` and
//!   `album` may each be a bare string or `{"#text": …}`
//!
//! A Last.fm scrobble carries less than a Spotify record: a time, a track, an
//! artist, sometimes a MusicBrainz id. There is no play duration and no skip,
//! because a scrobble is only submitted once a track has been listened to.
//! That is why `origin_rank` trusts it least.

use std::path::Path;

use anyhow::{Context, Result};

use super::{sql_literal, ImportReport};
use crate::progress::Reporter;
use crate::Analytics;

pub fn import(analytics: &Analytics, path: &Path, reporter: &dyn Reporter) -> Result<ImportReport> {
    let mut report = ImportReport {
        origin: "lastfm".into(),
        files: vec![path.display().to_string()],
        ..Default::default()
    };
    reporter.stage("reading Last.fm history", Some(1));

    let is_json = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("json"));

    let before = analytics.count()?;
    if is_json {
        import_json(analytics, path, &mut report)?
    } else {
        import_csv(analytics, path, &mut report)?
    };
    report.inserted = (analytics.count()? - before).max(0) as u64;

    reporter.advance(1);
    reporter.finish(&format!("{} scrobbles", report.inserted));
    Ok(report)
}

fn import_csv(analytics: &Analytics, path: &Path, report: &mut ImportReport) -> Result<()> {
    let literal = sql_literal(path);
    // `all_varchar` because the exporters are inconsistent about quoting and
    // a column that is usually numeric can carry an empty string; parsing
    // everything as text and casting explicitly is what keeps one malformed
    // row from failing the whole file.
    let source = format!(
        "read_csv('{literal}', header := true, all_varchar := true, ignore_errors := true)"
    );

    let total: i64 = analytics
        .conn()
        .query_row(&format!("SELECT count(*) FROM {source}"), [], |r| r.get(0))
        .with_context(|| format!("could not read {}", path.display()))?;

    let unusable: i64 = analytics.conn().query_row(
        &format!(
            "SELECT count(*) FROM {source}
             WHERE nullif(trim(track), '') IS NULL
                OR nullif(trim(artist), '') IS NULL
                OR coalesce(try_cast(uts AS BIGINT), 0) <= 0"
        ),
        [],
        |r| r.get(0),
    )?;

    analytics.conn().execute_batch(&format!(
        r#"
        INSERT OR REPLACE INTO listens
          (origin, origin_key, played_at, title, artist, album, source, match_key)
        SELECT 'lastfm',
               uts || '|' || track,
               to_timestamp(try_cast(uts AS BIGINT)),
               track,
               artist,
               nullif(trim(album), ''),
               'lastfm',
               match_key(artist, track)
        FROM {source}
        WHERE nullif(trim(track), '') IS NOT NULL
          AND nullif(trim(artist), '') IS NOT NULL
          AND coalesce(try_cast(uts AS BIGINT), 0) > 0
        "#
    ))?;

    report.read += total as u64;
    report.skip("missing track, artist or timestamp", unusable as u64);
    Ok(())
}

/// Parsed in Rust rather than in SQL, unlike every other import here.
///
/// The JSON dumps disagree with each other in ways DuckDB's struct typing
/// handles badly: `artist` is a bare string in one dumper and `{"#text": …}`
/// in the next, `date` is sometimes `{"uts": …}` and sometimes a bare number,
/// and the envelope may be a plain array or `{"recenttracks": {"track": …}}`.
/// Expressing that as nested `TRY(...)` casts produces SQL that is hard to
/// read and fails a whole file when one record is shaped unexpectedly.
fn import_json(analytics: &Analytics, path: &Path, report: &mut ImportReport) -> Result<()> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("could not read {}", path.display()))?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("{} is not valid JSON", path.display()))?;

    let entries = parsed
        .as_array()
        .cloned()
        .or_else(|| parsed.pointer("/recenttracks/track")?.as_array().cloned())
        .or_else(|| parsed.get("track")?.as_array().cloned())
        .or_else(|| parsed.get("scrobbles")?.as_array().cloned())
        .unwrap_or_default();

    let mut statement = analytics.conn().prepare(
        "INSERT OR REPLACE INTO listens
           (origin, origin_key, played_at, title, artist, album, source, match_key)
         VALUES ('lastfm', ?, to_timestamp(?), ?, ?, ?, 'lastfm', match_key(?, ?))",
    )?;

    let mut now_playing = 0;
    let mut unusable = 0;
    for entry in &entries {
        report.read += 1;

        // A now-playing entry has no timestamp; it is what is on right now,
        // not something that was listened to.
        if entry.pointer("/@attr/nowplaying").and_then(|v| v.as_str()) == Some("true") {
            now_playing += 1;
            continue;
        }

        let title = text(
            entry
                .get("name")
                .or_else(|| entry.get("track"))
                .or_else(|| entry.get("title")),
        );
        let artist = text(entry.get("artist"));
        let uts = entry
            .pointer("/date/uts")
            .or_else(|| entry.get("date"))
            .or_else(|| entry.get("uts"))
            .or_else(|| entry.get("timestamp"))
            .and_then(number);

        let (Some(title), Some(artist), Some(uts)) = (title, artist, uts) else {
            unusable += 1;
            continue;
        };
        if uts <= 0 {
            unusable += 1;
            continue;
        }

        let album = text(entry.get("album"));
        statement.execute(duckdb::params![
            format!("{uts}|{title}"),
            uts,
            title,
            artist,
            album,
            artist,
            title,
        ])?;
    }

    report.skip("now playing (no timestamp yet)", now_playing);
    report.skip("missing track, artist or timestamp", unusable);
    Ok(())
}

/// A field that may be a bare string or an object carrying `#text` / `name`.
fn text(value: Option<&serde_json::Value>) -> Option<String> {
    let value = value?;
    let raw = match value {
        serde_json::Value::String(s) => s.clone(),
        other => other
            .get("#text")
            .or_else(|| other.get("name"))
            .and_then(|v| v.as_str())
            .map(str::to_owned)?,
    };
    let trimmed = raw.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

/// A timestamp that may be a number or a string holding one.
fn number(value: &serde_json::Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_str()?.trim().parse().ok())
}
