//! Spotify "Extended Streaming History".
//!
//! Request it from the privacy page and Spotify eventually mails a zip holding
//! one `Streaming_History_Audio_<year>.json` per year, each a flat JSON array
//! of listens. It is the richest history available: besides the timestamp and
//! the track it records how many milliseconds actually played, why the track
//! started and why it *ended* (`trackdone`, `fwdbtn`, `endplay`), whether
//! shuffle was on, and the platform. `reason_end` in particular is a far
//! better skip signal than any threshold heuristic.
//!
//! This is the import that justifies DuckDB: `read_json_auto` over a glob
//! reads nine years and ~30k records in one statement, with the schema
//! inferred and unioned across files whose columns changed over the years.

use std::path::Path;

use anyhow::{Context, Result};

use super::{sql_literal, ImportReport};
use crate::progress::Reporter;
use crate::Analytics;

/// Records shorter than this are noise — a track opened and abandoned, or a
/// skip through a playlist. Matches the Rocksky CLI's import default so both
/// tools agree on what counts as a listen.
pub const DEFAULT_MIN_SECONDS: u64 = 30;

/// Import every `Streaming_History_Audio_*.json` under `path`.
///
/// `path` may be the export directory or a single file.
pub fn import(
    analytics: &Analytics,
    path: &Path,
    min_seconds: u64,
    reporter: &dyn Reporter,
) -> Result<ImportReport> {
    let files = collect(path)?;
    let mut report = ImportReport {
        origin: "spotify".into(),
        ..Default::default()
    };

    reporter.stage("reading Spotify history", Some(files.len() as u64));
    for file in &files {
        // One file at a time rather than a single glob: a 4 MB year and a
        // 13 MB year take visibly different amounts of time, and an error
        // should name the file that caused it rather than the whole export.
        let counted = import_file(analytics, file, min_seconds, &mut report)
            .with_context(|| format!("could not import {}", file.display()))?;
        report.files.push(file.display().to_string());
        reporter.advance(1);
        tracing::debug!(file = %file.display(), counted, "imported");
    }

    reporter.finish(&format!(
        "{} listens from {} file(s)",
        report.inserted,
        files.len()
    ));
    Ok(report)
}

fn import_file(
    analytics: &Analytics,
    file: &Path,
    min_seconds: u64,
    report: &mut ImportReport,
) -> Result<u64> {
    let literal = sql_literal(file);
    let min_ms = (min_seconds * 1000) as i64;

    // `union_by_name` because the export's columns have changed over the
    // years: the 2017 file has no `audiobook_uri`, the 2026 one does, and
    // positional union would misalign them.
    let source = format!("read_json_auto('{literal}', union_by_name := true)");

    let before = analytics.count()?;

    // Podcasts, audiobooks and videos share the file with music and carry no
    // track metadata at all; they are not listens in any sense this engine
    // means.
    let total: i64 =
        analytics
            .conn()
            .query_row(&format!("SELECT count(*) FROM {source}"), [], |r| r.get(0))?;
    let not_a_track: i64 = analytics.conn().query_row(
        &format!(
            "SELECT count(*) FROM {source}
             WHERE master_metadata_track_name IS NULL
                OR master_metadata_album_artist_name IS NULL"
        ),
        [],
        |r| r.get(0),
    )?;
    let too_short: i64 = analytics.conn().query_row(
        &format!(
            "SELECT count(*) FROM {source}
             WHERE master_metadata_track_name IS NOT NULL
               AND master_metadata_album_artist_name IS NOT NULL
               AND coalesce(ms_played, 0) < {min_ms}"
        ),
        [],
        |r| r.get(0),
    )?;

    analytics.conn().execute_batch(&format!(
        r#"
        INSERT OR REPLACE INTO listens
          (origin, origin_key, played_at, track_id, title, artist, album,
           ms_played, length_ms, skipped, reason_start, reason_end, shuffle,
           platform, country, source, match_key)
        SELECT
          'spotify',
          -- The timestamp alone is not unique: two devices can log the same
          -- second. Pairing it with the track uri is.
          ts || '|' || coalesce(spotify_track_uri, master_metadata_track_name),
          ts::TIMESTAMPTZ,
          NULL,
          master_metadata_track_name,
          master_metadata_album_artist_name,
          master_metadata_album_album_name,
          ms_played,
          NULL,
          coalesce(skipped, false),
          reason_start,
          reason_end,
          shuffle,
          platform,
          conn_country,
          'spotify',
          match_key(master_metadata_album_artist_name, master_metadata_track_name)
        FROM {source}
        WHERE master_metadata_track_name IS NOT NULL
          AND master_metadata_album_artist_name IS NOT NULL
          AND coalesce(ms_played, 0) >= {min_ms}
        "#
    ))?;

    let inserted = (analytics.count()? - before).max(0) as u64;
    report.read += total as u64;
    report.inserted += inserted;
    report.skip(
        "not a track (podcast, audiobook, video)",
        not_a_track as u64,
    );
    report.skip(&format!("played under {min_seconds}s"), too_short as u64);
    Ok(inserted)
}

/// The `Streaming_History_Audio_*.json` files under `path`, sorted.
///
/// `Streaming_History_Video_*.json` is deliberately not matched: it is in the
/// same export and has the same shape, but a watched video is not a listen.
fn collect(path: &Path) -> Result<Vec<std::path::PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    if !path.is_dir() {
        anyhow::bail!("{} does not exist", path.display());
    }

    let mut files: Vec<_> = std::fs::read_dir(path)
        .with_context(|| format!("could not read {}", path.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|p| {
            p.file_name().and_then(|n| n.to_str()).is_some_and(|name| {
                name.starts_with("Streaming_History_Audio_") && name.ends_with(".json")
            })
        })
        .collect();
    files.sort();

    if files.is_empty() {
        anyhow::bail!(
            "no Streaming_History_Audio_*.json files in {} — point this at the \
             folder Spotify's export unzips to",
            path.display()
        );
    }
    Ok(files)
}
