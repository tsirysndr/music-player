//! Artist pictures for the leaderboards.
//!
//! Album art comes free with enrichment — `matchSong` returns it per track and
//! it is stored in `song_match`. Artist pictures are the awkward case: the
//! same response carries one, but a database enriched before that column
//! existed has none, and re-resolving eleven thousand tracks to recover ten
//! faces would be absurd.
//!
//! So this resolves per *artist*, only for the handful a screen actually
//! shows, and caches the answer — including a miss, so an artist the catalogue
//! does not know is asked about once rather than on every repaint.

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{mpsc, Mutex};

use crate::Analytics;

/// Small on purpose: this runs while a screen is waiting to draw, and the
/// number of artists on a leaderboard is in the tens.
const CONCURRENCY: usize = 4;

/// Requests per second, as in [`crate::enrich`].
///
/// Firing even eight lookups at once with no pacing got several of them
/// refused — and, because a refusal used to be cached, artists as obvious as
/// Daft Punk ended up permanently recorded as having no picture while the
/// same request by hand returned one immediately.
const REQUESTS_PER_SECOND: u64 = 5;

/// Short, for the same reason. A slow lookup should fall back to the
/// placeholder rather than hold the panel.
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(6);

/// One artist waiting for a picture: cache key, display name, and a few of
/// their tracks to look the picture up by.
///
/// Several titles rather than one because the catalogue answers per *track*:
/// a given entry may carry no artist picture even when another by the same
/// artist does, and giving up after the first try records a miss that is not
/// true of the artist at all.
pub type Pending = (String, String, Vec<String>);

/// Make sure every one of `artists` has an entry in `artist_art`.
///
/// Artists already cached are left alone, so the common case does no work at
/// all and touches no network.
///
/// Convenience for callers that can hold a [`Analytics`] across an await — the
/// CLI and the tests. A caller inside a spawned task cannot (the connection is
/// not `Sync`) and should use [`pending_artists`], [`fetch_pictures`] and
/// [`store_pictures`], which keep the database work out of the async part.
pub async fn resolve_artists(analytics: &Analytics, artists: &[String]) -> Result<u64> {
    let pending = pending_artists(analytics, artists)?;
    let found = fetch_pictures(pending).await;
    store_pictures(analytics, &found)
}

/// Artists from `artists` that still need a lookup.
pub fn pending_artists(analytics: &Analytics, artists: &[String]) -> Result<Vec<Pending>> {
    uncached(analytics, artists)
}

/// Write what [`fetch_pictures`] found, caching misses so they are not asked
/// about again.
pub fn store_pictures(analytics: &Analytics, found: &[(String, Option<String>)]) -> Result<u64> {
    if found.is_empty() {
        return Ok(0);
    }
    let mut statement = analytics.conn().prepare(
        "INSERT OR REPLACE INTO artist_art (artist_key, picture, resolved, matched_at)
         VALUES (?, ?, ?, now())",
    )?;
    let mut resolved = 0;
    for (key, picture) in found {
        statement.execute(duckdb::params![key, picture, picture.is_some()])?;
        if picture.is_some() {
            resolved += 1;
        }
    }
    Ok(resolved)
}

/// Look every pending artist up. Touches no database, so this is safe to
/// await from a spawned task.
pub async fn fetch_pictures(pending: Vec<Pending>) -> Vec<(String, Option<String>)> {
    if pending.is_empty() {
        return Vec::new();
    }

    let base = Arc::new(
        std::env::var("ROCKSKY_API_URL").unwrap_or_else(|_| "https://api.rocksky.app".to_string()),
    );
    let Ok(http) = reqwest::Client::builder().timeout(TIMEOUT).build() else {
        return Vec::new();
    };
    let queue = Arc::new(Mutex::new(pending));
    let ticker = Arc::new(Mutex::new(tokio::time::interval(
        std::time::Duration::from_millis(1000 / REQUESTS_PER_SECOND),
    )));
    let (tx, mut rx) = mpsc::channel::<(String, Option<String>)>(32);

    let mut workers = Vec::new();
    for _ in 0..CONCURRENCY.min(queue.lock().await.len().max(1)) {
        let queue = Arc::clone(&queue);
        let ticker = Arc::clone(&ticker);
        let base = Arc::clone(&base);
        let http = http.clone();
        let tx = tx.clone();
        workers.push(tokio::spawn(async move {
            loop {
                let Some((key, artist, titles)) = queue.lock().await.pop() else {
                    break;
                };
                ticker.lock().await.tick().await;

                let mut picture = None;
                let mut errored = false;
                for title in &titles {
                    match lookup(&http, &base, &artist, title).await {
                        Ok(Some(found)) => {
                            picture = Some(found);
                            break;
                        }
                        Ok(None) => {}
                        Err(cause) => {
                            errored = true;
                            tracing::debug!(%artist, %title, "artist lookup failed: {cause}");
                        }
                    }
                }
                // A refused request is not proof the artist has no picture.
                // Caching that as one would blank them out permanently, so
                // leave them uncached and retry on a later repaint.
                if picture.is_none() && errored {
                    continue;
                }
                if tx.send((key, picture)).await.is_err() {
                    break;
                }
            }
        }));
    }
    drop(tx);

    let mut found = Vec::new();
    while let Some(result) = rx.recv().await {
        found.push(result);
    }
    for worker in workers {
        let _ = worker.await;
    }
    found
}

/// A picture for `artist`, via one of their tracks.
///
/// There is no "get artist by name" that takes a bare string, but `matchSong`
/// resolves a track and returns the artist's picture with it — which is why
/// the representative title is carried this far.
///
/// `Ok(None)` is a definitive "the catalogue has no picture"; `Err` is a
/// problem with the request, which must not be cached as one.
async fn lookup(
    http: &reqwest::Client,
    base: &str,
    artist: &str,
    title: &str,
) -> Result<Option<String>> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Response {
        #[serde(default)]
        artist_picture: Option<String>,
    }

    let response = http
        .get(format!("{base}/xrpc/app.rocksky.song.matchSong"))
        .query(&[("title", title), ("artist", artist), ("album", "")])
        .send()
        .await?
        .error_for_status()?
        .json::<Response>()
        .await?;

    Ok(response
        .artist_picture
        .filter(|picture| !picture.trim().is_empty()))
}

/// Artists with no `artist_art` row yet, each with a track to look up by.
///
/// The picture is taken from `song_match` where enrichment already stored one,
/// which costs no request at all.
fn uncached(analytics: &Analytics, artists: &[String]) -> Result<Vec<Pending>> {
    if artists.is_empty() {
        return Ok(Vec::new());
    }

    // Fold in anything enrichment already knows before asking the network.
    analytics.conn().execute_batch(
        "INSERT OR REPLACE INTO artist_art (artist_key, picture, resolved, matched_at)
         SELECT primary_artist(l.artist),
                any_value(m.artist_picture),
                true,
                now()
         FROM listens l
         JOIN song_match m ON m.match_key = l.match_key
         WHERE m.artist_picture IS NOT NULL
           AND primary_artist(l.artist) NOT IN (SELECT artist_key FROM artist_art WHERE resolved)
         GROUP BY primary_artist(l.artist)",
    )?;

    let list = artists
        .iter()
        .map(|artist| format!("'{}'", artist.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(",");

    // `mode()` picks the most-played spelling of the artist; the titles are
    // their most-played tracks, most-played first, because a well-known track
    // is likelier to be in the catalogue with a picture attached.
    const CANDIDATES: usize = 3;
    let retry_days = crate::enrich::RETRY_MISSES_AFTER_DAYS;
    let mut statement = analytics.conn().prepare(&format!(
        "WITH ranked AS (
           SELECT primary_artist(artist) AS artist_key,
                  artist,
                  title,
                  count(*) AS plays,
                  row_number() OVER (PARTITION BY primary_artist(artist)
                                     ORDER BY count(*) DESC) AS rank
           FROM enriched_listens
           WHERE artist IN ({list})
             AND primary_artist(artist) NOT IN (
                 SELECT artist_key FROM artist_art
                 -- A recorded miss expires, for the same reason it does in
                 -- enrichment: an empty answer under load is not an absence.
                 WHERE resolved
                    OR matched_at >= now() - INTERVAL {retry_days} DAY
               )
           GROUP BY artist_key, artist, title
         )
         SELECT artist_key,
                mode(artist) AS artist,
                list(title ORDER BY plays DESC) AS titles
         FROM ranked
         WHERE rank <= {CANDIDATES}
         GROUP BY artist_key"
    ))?;
    let rows = statement.query_map([], |row| {
        let titles: Vec<String> = row
            .get::<_, duckdb::types::Value>(2)
            .map(value_to_titles)
            .unwrap_or_default();
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, titles))
    })?;
    Ok(rows
        .collect::<std::result::Result<Vec<Pending>, _>>()?
        .into_iter()
        .filter(|(_, _, titles)| !titles.is_empty())
        .collect())
}

/// DuckDB hands a `list(...)` back as a nested value; flatten it to the
/// strings inside.
fn value_to_titles(value: duckdb::types::Value) -> Vec<String> {
    match value {
        duckdb::types::Value::List(items) => items
            .into_iter()
            .filter_map(|item| match item {
                duckdb::types::Value::Text(text) => Some(text),
                _ => None,
            })
            .collect(),
        duckdb::types::Value::Text(text) => vec![text],
        _ => Vec::new(),
    }
}
