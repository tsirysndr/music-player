//! atproto integration for the daemon.
//!
//! Three background tasks, none of which is required to use music-player:
//!
//! * **radio bookmarks** — imports the account's `fm.atradio.favorite` records
//!   into `saved_radio`, then follows the repo on Jetstream.
//! * **liked songs** — imports `app.rocksky.like` records, matches them against
//!   the library, and follows the repo for likes and unlikes.
//! * **listening status** — publishes the station playing right now as the
//!   account's `fm.atradio.actor.status` record, and deletes it on stop.
//!
//! Nothing starts unless `atproto = true` in settings.toml (the default) *and*
//! an identity resolves — a `rocksky login` token, an `atradio login` session,
//! or `ATPROTO_IDENTIFIER` / `ATPROTO_APP_PASSWORD` in the environment. With
//! none of those, [`spawn`] logs once and returns without opening a socket.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use music_player_entity::saved_radio;
use music_player_settings::read_settings;
use music_player_storage::{atradio, rocksky_likes};
use music_player_tracklist::Tracklist;
use tracing::{info, warn};

/// How often the tracklist is polled for a change of station.
const TICK: Duration = Duration::from_secs(2);

/// Back-off after a failed status write, so a broken session doesn't retry
/// every tick.
const RETRY_AFTER: Duration = Duration::from_secs(60);

fn enabled() -> bool {
    read_settings()
        .ok()
        .and_then(|config| config.get_bool("atproto").ok())
        .unwrap_or(true)
}

pub fn spawn(tracklist: Arc<Mutex<Tracklist>>) {
    if !enabled() {
        info!("atproto integration disabled (atproto = false in settings.toml)");
        return;
    }
    tokio::spawn(async move {
        // One identity check up front: with no account linked there is nothing
        // for any of these tasks to do, so none of them start.
        let Some(did) = atradio::resolve_did().await else {
            info!(
                "atproto integration off: no account linked (run `rocksky login` or \
                 `atradio login`, or set ATPROTO_IDENTIFIER + ATPROTO_APP_PASSWORD)"
            );
            return;
        };
        info!(%did, "atproto integration on");

        let db = music_player_storage::shared().await;
        let bookmarks = db.get_connection().clone();
        let likes = db.get_connection().clone();
        tokio::spawn(atradio::sync(bookmarks));
        tokio::spawn(rocksky_likes::sync(likes));
        tokio::spawn(status_loop(tracklist));
    });
}

/// The station playing right now, as a bookmark-shaped row, or `None` when
/// playback is stopped or the current track is not a radio stream.
fn current_station(tracklist: &Arc<Mutex<Tracklist>>) -> Option<saved_radio::Model> {
    let (track, is_playing) = {
        let tracklist = tracklist.lock().unwrap();
        let (track, _) = tracklist.current_track();
        (track, tracklist.playback_state().is_playing)
    };
    if !is_playing {
        return None;
    }
    let track = track?;
    // Radio entries are queued as `radio:<station id>`; the station id is what
    // atradio joins on.
    let id = track.id.strip_prefix("radio:")?.to_owned();
    Some(saved_radio::Model {
        id,
        name: track.title.clone(),
        stream_url: track.uri.clone(),
        source: track.artist.clone(),
        genre: String::new(),
        country: String::new(),
        logo: track.album.cover.clone().unwrap_or_default(),
        bitrate: track.bitrate.unwrap_or_default(),
    })
}

async fn status_loop(tracklist: Arc<Mutex<Tracklist>>) {
    let db = music_player_storage::shared().await;
    // The station id currently published, so the record is only rewritten when
    // the station actually changes.
    let mut published: Option<String> = None;
    let mut retry_after: Option<Instant> = None;
    loop {
        tokio::time::sleep(TICK).await;
        if retry_after.is_some_and(|at| Instant::now() < at) {
            continue;
        }
        retry_after = None;

        match current_station(&tracklist) {
            Some(station) if published.as_deref() != Some(station.id.as_str()) => {
                let mut station = station;
                atradio::enrich_from_bookmark(db.get_connection(), &mut station).await;
                match atradio::set_status(&station).await {
                    Ok(()) => published = Some(station.id),
                    Err(e) => {
                        warn!("could not publish the atradio listening status: {e}");
                        retry_after = Some(Instant::now() + RETRY_AFTER);
                    }
                }
            }
            Some(_) => {}
            None if published.is_some() => match atradio::clear_status().await {
                Ok(()) => published = None,
                Err(e) => {
                    warn!("could not clear the atradio listening status: {e}");
                    retry_after = Some(Instant::now() + RETRY_AFTER);
                }
            },
            None => {}
        }
    }
}
