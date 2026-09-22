//! Resolving imported listens against Rocksky's catalogue.
//!
//! A Spotify or Last.fm export carries a title, an artist and a time. It has
//! no album for half the rows, no genre, no release year and no duration — so
//! "what genres did I listen to in 2021" and "how old is the music I play"
//! cannot be answered from an import alone, which is most of nine years of
//! history.
//!
//! `app.rocksky.song.matchSong` resolves a bare title + artist to a canonical
//! track with all of that attached. It needs no authentication, so this works
//! without a Rocksky account.
//!
//! Three things make it safe to run over a large history:
//!
//! * **Cached.** Every answer is written to `song_match`, keyed by the same
//!   normalised key the listens use, so a track is asked about once no matter
//!   how many times it was played. 30k listens is only ~5k distinct tracks.
//! * **Negative-cached.** A miss is recorded as `resolved = false` rather than
//!   left absent, so an unknown track is not re-asked on every run.
//! * **Rate limited.** The API allows 1000 requests per 30-second window;
//!   this stays at a fifth of that, and is resumable, so an interrupted run
//!   picks up where it stopped instead of starting over.

use std::sync::Arc;

use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::sync::{mpsc, Mutex};

use crate::progress::Reporter;
use crate::Analytics;

/// Requests per second. The documented ceiling is ~33/s (1000 per 30s); a
/// fifth of it leaves room for whatever else is talking to the API and keeps
/// this well clear of the abuse guard.
const REQUESTS_PER_SECOND: u64 = 6;

/// How many requests are in flight at once.
///
/// The ticker below is what caps the *rate*; this decides whether that cap is
/// ever reached. `matchSong` takes around two seconds a call, so four workers
/// top out near 2/s — a third of the budget, and measured at 87 minutes for an
/// 11k-track backfill. Enough workers to cover the latency puts the ticker
/// back in charge, which is where the limiting belongs.
const CONCURRENCY: usize = 16;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EnrichReport {
    pub attempted: u64,
    pub resolved: u64,
    pub missed: u64,
}

/// What `matchSong` returns, reduced to the fields worth storing.
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct Match {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    artist: Option<String>,
    #[serde(default)]
    album: Option<String>,
    #[serde(default)]
    album_art: Option<String>,
    #[serde(default)]
    genre: Option<String>,
    #[serde(default)]
    year: Option<i32>,
    /// Milliseconds.
    #[serde(default)]
    duration: Option<i64>,
    #[serde(default)]
    mb_id: Option<String>,
    #[serde(default)]
    isrc: Option<String>,
    #[serde(default)]
    spotify_link: Option<String>,
    #[serde(default)]
    uri: Option<String>,
    #[serde(default)]
    artist_picture: Option<String>,
}

/// One track waiting to be resolved.
struct Pending {
    match_key: String,
    title: String,
    artist: String,
    album: Option<String>,
}

/// Resolve metadata for listens that have none.
///
/// `limit` caps how many tracks are looked up in one run, so this can be given
/// a budget rather than being all-or-nothing.
pub async fn enrich(
    analytics: &Analytics,
    limit: u32,
    reporter: &dyn Reporter,
) -> Result<EnrichReport> {
    let pending = pending(analytics, limit)?;
    if pending.is_empty() {
        reporter.finish("everything already resolved");
        return Ok(EnrichReport::default());
    }

    reporter.stage("resolving track metadata", Some(pending.len() as u64));

    let base = Arc::new(
        std::env::var("ROCKSKY_API_URL").unwrap_or_else(|_| "https://api.rocksky.app".to_string()),
    );
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()?;

    // A single shared queue rather than a chunk per worker: the lookups take
    // wildly different times, and static chunks leave workers idle at the end.
    let queue = Arc::new(Mutex::new(pending));
    // One shared ticker is what actually enforces the rate limit — a per-task
    // sleep would multiply by the number of tasks.
    let ticker = Arc::new(Mutex::new(tokio::time::interval(
        std::time::Duration::from_millis(1000 / REQUESTS_PER_SECOND),
    )));

    // The DuckDB connection is not `Sync`, so it cannot be touched from the
    // worker tasks. They send results back and this task alone writes them.
    let (tx, mut rx) = mpsc::channel::<(String, Option<Match>)>(64);

    let mut workers = Vec::with_capacity(CONCURRENCY);
    for _ in 0..CONCURRENCY {
        let queue = Arc::clone(&queue);
        let ticker = Arc::clone(&ticker);
        let base = Arc::clone(&base);
        let http = http.clone();
        let tx = tx.clone();

        workers.push(tokio::spawn(async move {
            loop {
                let Some(next) = queue.lock().await.pop() else {
                    break;
                };
                ticker.lock().await.tick().await;

                let found = match_song(&http, &base, &next).await;
                let found = match found {
                    Ok(found) => found,
                    Err(cause) => {
                        // A lookup failure is not a miss: recording it as one
                        // would poison the negative cache and the track would
                        // never be retried. Leave it pending instead.
                        tracing::debug!(track = %next.title, "matchSong failed: {cause}");
                        continue;
                    }
                };
                if tx.send((next.match_key, found)).await.is_err() {
                    break;
                }
            }
        }));
    }
    drop(tx);

    let mut statement = analytics.conn().prepare(
        "INSERT OR REPLACE INTO song_match
           (match_key, resolved, title, artist, album, album_art, genre, year,
            duration_ms, mb_id, isrc, spotify_link, uri, artist_picture, matched_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, now())",
    )?;

    let mut report = EnrichReport::default();
    while let Some((key, found)) = rx.recv().await {
        report.attempted += 1;
        match found {
            Some(m) => {
                statement.execute(duckdb::params![
                    key,
                    true,
                    m.title,
                    m.artist,
                    m.album,
                    m.album_art,
                    m.genre,
                    m.year,
                    m.duration,
                    m.mb_id,
                    m.isrc,
                    m.spotify_link,
                    m.uri,
                    m.artist_picture,
                ])?;
                report.resolved += 1;
            }
            None => {
                let none: Option<String> = None;
                statement.execute(duckdb::params![
                    key,
                    false,
                    none,
                    none,
                    none,
                    none,
                    none,
                    None::<i32>,
                    None::<i64>,
                    none,
                    none,
                    none,
                    none,
                    none,
                ])?;
                report.missed += 1;
            }
        }
        reporter.advance(1);
    }

    for worker in workers {
        let _ = worker.await;
    }

    reporter.finish(&format!(
        "{} resolved, {} not found",
        report.resolved, report.missed
    ));
    Ok(report)
}

/// `Ok(None)` is a definitive "the catalogue does not have this"; `Err` is a
/// problem with the request, which must not be cached as a miss.
async fn match_song(
    http: &reqwest::Client,
    base: &str,
    pending: &Pending,
) -> Result<Option<Match>> {
    let response = http
        .get(format!("{base}/xrpc/app.rocksky.song.matchSong"))
        .query(&[
            ("title", pending.title.as_str()),
            ("artist", pending.artist.as_str()),
            ("album", pending.album.as_deref().unwrap_or_default()),
        ])
        .send()
        .await
        .with_context(|| format!("could not reach {base}"))?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    let response = response.error_for_status()?;
    let found: Match = response
        .json()
        .await
        .context("could not decode the match")?;
    // A 200 with no title is the API's other way of saying "no match".
    let matched = found
        .title
        .as_deref()
        .is_some_and(|title| !title.trim().is_empty());
    Ok(matched.then_some(found))
}

/// Tracks that have been listened to, have no local library row, and have not
/// been looked up yet — most-played first, so a capped run spends its budget
/// where it matters.
fn pending(analytics: &Analytics, limit: u32) -> Result<Vec<Pending>> {
    let mut statement = analytics.conn().prepare(
        "SELECT l.match_key, mode(l.title), mode(l.artist), mode(l.album)
         FROM listens l
         LEFT JOIN tracks     t ON t.track_id  = l.track_id
         LEFT JOIN song_match m ON m.match_key = l.match_key
         WHERE t.track_id IS NULL AND m.match_key IS NULL
         GROUP BY l.match_key
         -- Internet radio records one row per announced song with an explicit
         -- zero length; station idents and adverts arrive that way and are not
         -- tracks. Asking the catalogue about `Advertisement` by `Live365`
         -- returns a 500, which is treated as a transient failure rather than
         -- a miss — so it is never cached and gets re-asked on every run,
         -- forever. A key whose every listen states a length of zero is a
         -- stream, not something to resolve.
         HAVING max(coalesce(l.length_ms, -1)) <> 0
         ORDER BY count(*) DESC
         LIMIT ?",
    )?;
    let rows = statement.query_map([limit as i64], |row| {
        Ok(Pending {
            match_key: row.get(0)?,
            title: row.get(1)?,
            artist: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            album: row.get(3)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<_, _>>()?)
}

/// The titles [`enrich`] would look up, for the tests.
#[cfg(test)]
pub(crate) fn pending_for_test(analytics: &Analytics, limit: u32) -> Result<Vec<String>> {
    Ok(pending(analytics, limit)?
        .into_iter()
        .map(|p| p.title)
        .collect())
}
