//! Rocksky scrobbles, over the public AppView.
//!
//! `app.rocksky.actor.getActorScrobbles` needs no authentication — it reads
//! the actor's public scrobble feed — so this works for any handle or DID, not
//! only the signed-in user. Pages are newest-first, which is what makes the
//! import incremental: walk until a scrobble already in the database turns up
//! and stop, because everything past it is older and therefore already here.

use anyhow::{Context, Result};
use serde::Deserialize;

use super::ImportReport;
use crate::progress::Reporter;
use crate::Analytics;

/// The AppView's page ceiling.
const PAGE: u32 = 100;

/// The API allows 1000 requests per 30-second window. At one page per request
/// this is comfortably inside it while still importing 100k scrobbles in a
/// couple of minutes, and leaves room for whatever else the app is doing.
const REQUESTS_PER_SECOND: u32 = 10;

/// Stop after this many consecutive pages that add nothing new.
///
/// One such page is not enough to stop on: a page can be entirely duplicates
/// and still be followed by older scrobbles that are missing, if an earlier
/// import was interrupted part-way.
const STALE_PAGES_BEFORE_STOP: u32 = 3;

#[derive(Deserialize)]
struct Page {
    #[serde(default)]
    scrobbles: Vec<Scrobble>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Scrobble {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    uri: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    artist: Option<String>,
    #[serde(default)]
    album: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
}

/// Import an actor's scrobbles.
///
/// `actor` is a handle (`tsiry-sandratraina.com`) or a DID. When `full` is
/// false the walk stops once it has seen [`STALE_PAGES_BEFORE_STOP`] pages of
/// scrobbles that were already stored.
pub async fn import(
    analytics: &Analytics,
    actor: &str,
    full: bool,
    reporter: &dyn Reporter,
) -> Result<ImportReport> {
    let base =
        std::env::var("ROCKSKY_API_URL").unwrap_or_else(|_| "https://api.rocksky.app".to_string());
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let mut report = ImportReport {
        origin: "rocksky".into(),
        ..Default::default()
    };
    reporter.stage(&format!("fetching scrobbles for {actor}"), None);

    let mut staging = crate::staging::Staging::new(analytics);

    let mut offset = 0u32;
    let mut stale_pages = 0u32;
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(
        (1000 / REQUESTS_PER_SECOND) as u64,
    ));

    loop {
        interval.tick().await;

        let page: Page = http
            .get(format!("{base}/xrpc/app.rocksky.actor.getActorScrobbles"))
            .query(&[
                ("did", actor.to_string()),
                ("limit", PAGE.to_string()),
                ("offset", offset.to_string()),
            ])
            .send()
            .await
            .with_context(|| format!("could not reach {base}"))?
            .error_for_status()
            .context("the Rocksky AppView rejected the request")?
            .json()
            .await
            .context("could not decode the scrobble page")?;

        if page.scrobbles.is_empty() {
            break;
        }

        for scrobble in &page.scrobbles {
            report.read += 1;

            // The at:// uri is globally unique and stable; the bare id is a
            // fallback for older records that predate it.
            let Some(key) = scrobble.uri.clone().or_else(|| scrobble.id.clone()) else {
                report.skip("no stable identifier", 1);
                continue;
            };
            let (Some(title), Some(artist), Some(created_at)) = (
                scrobble.title.as_deref().filter(|s| !s.trim().is_empty()),
                scrobble.artist.as_deref().filter(|s| !s.trim().is_empty()),
                scrobble.created_at.as_deref(),
            ) else {
                report.skip("missing track, artist or timestamp", 1);
                continue;
            };

            staging.push(crate::staging::Row {
                origin: "rocksky".into(),
                origin_key: key,
                played_at: crate::staging::When::Rfc3339(created_at.to_owned()),
                title: title.to_owned(),
                artist: artist.to_owned(),
                album: scrobble.album.clone(),
                source: "rocksky".into(),
                ..Default::default()
            })?;
        }
        reporter.advance(page.scrobbles.len() as u64);

        // Staging batches across pages, so "did this page add anything" is
        // only answerable once a batch has been folded in. Ask the table
        // directly instead: a page whose keys are all present already is a
        // page we have seen.
        //
        // Only worth asking when the answer can stop the walk. Under `--full`
        // it cannot, and the query is a scan of the whole table per page.
        if !full {
            let known: i64 = analytics.conn().query_row(
                &format!(
                    "SELECT count(*) FROM listens WHERE origin = 'rocksky' AND origin_key IN ({})",
                    page.scrobbles
                        .iter()
                        .filter_map(|s| s.uri.as_deref().or(s.id.as_deref()))
                        .map(|key| format!("'{}'", key.replace('\'', "''")))
                        .collect::<Vec<_>>()
                        .join(",")
                ),
                [],
                |row| row.get(0),
            )?;

            if known >= page.scrobbles.len() as i64 {
                stale_pages += 1;
                if stale_pages >= STALE_PAGES_BEFORE_STOP {
                    tracing::debug!(offset, "reached scrobbles already imported");
                    break;
                }
            } else {
                stale_pages = 0;
            }
        }

        // A short page is the end of the feed.
        if (page.scrobbles.len() as u32) < PAGE {
            break;
        }
        offset += PAGE;
    }

    staging.flush()?;
    report.inserted = staging.inserted;

    if report.inserted > 0 {
        crate::sync::record_cursor(analytics, "rocksky", actor, report.inserted)?;
    }
    reporter.finish(&format!("{} scrobbles", report.inserted));
    Ok(report)
}
