//! Rocksky auto-scrobbling for the daemon.
//!
//! Ticks once a second off the shared tracklist state and submits a scrobble
//! when a play crosses Last.fm's rule: half the track or 4 minutes, whichever
//! comes first. Mirrors rocksky's `playerd` scrobbler.
//!
//! Requires the access token written by `rocksky login` to
//! `~/.rocksky/token.json`; without it the scrobbler stays off. It can also be
//! disabled with `scrobble = false` in `settings.toml`.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use music_player_settings::read_settings;
use music_player_tracklist::Tracklist;
use rocksky_sdk::{AppView, ScrobbleInput};
use tracing::{info, warn};

const TICK: Duration = Duration::from_secs(1);

/// Last.fm's rule: half the track, capped at 4 minutes.
const SCROBBLE_CAP_MS: u32 = 4 * 60 * 1000;

/// Back-off before retrying a scrobble the server rejected (offline, expired
/// token…), so a failing submit doesn't retry every tick.
const RETRY_AFTER: Duration = Duration::from_secs(30);

/// A position drop this large, landing near the start, means the track began
/// again (repeat, or the same file queued twice) rather than a seek.
const REPLAY_EPSILON_MS: u32 = 5_000;

struct Current {
    key: String,
    title: String,
    artist: String,
    album_artist: String,
    album: String,
    duration_ms: u32,
    position_ms: u32,
    track_number: Option<i32>,
}

fn token_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/"))
        .join(".rocksky")
        .join("token.json")
}

/// The access token written by `rocksky login`, if any.
fn read_token() -> Option<String> {
    let raw = std::fs::read_to_string(token_path()).ok()?;
    let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
    value
        .get("token")
        .and_then(|t| t.as_str())
        .filter(|t| !t.trim().is_empty())
        .map(String::from)
}

fn current_track(tracklist: &Arc<Mutex<Tracklist>>) -> Option<Current> {
    let (track, index, position_ms, is_playing) = {
        let tracklist = tracklist.lock().unwrap();
        let (track, index) = tracklist.current_track();
        let state = tracklist.playback_state();
        (track, index, state.position_ms, state.is_playing)
    };
    if !is_playing {
        return None;
    }
    let track = track?;
    Some(Current {
        // Queue position plus title: a local file has no stable play id, and
        // the position alone would miss a repeat of the same index.
        key: format!("{index}\u{1}{}", track.title),
        title: track.title.clone(),
        artist: track.artist.clone(),
        album_artist: track.album.artist.clone(),
        album: track.album.title.clone(),
        duration_ms: (track.duration.unwrap_or(0.0) * 1000.0) as u32,
        position_ms,
        track_number: track.track.map(|n| n as i32).filter(|n| *n > 0),
    })
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

/// Per-play bookkeeping. One instance lives in the loop.
#[derive(Default)]
struct Watcher {
    key: Option<String>,
    /// Wall clock when this play started — the scrobble's `timestamp`.
    started_at: i64,
    last_position_ms: u32,
    submitted: bool,
    retry_after: Option<Instant>,
}

impl Watcher {
    /// Track identity/replay bookkeeping. Returns true when this tick is the
    /// one that crosses the scrobble threshold.
    fn advance(&mut self, current: &Current) -> bool {
        let is_new = self.key.as_deref() != Some(current.key.as_str());
        // Repeat keeps the key but rewinds the clock: that is a new play, not
        // a seek backwards.
        let replayed = !is_new
            && current.position_ms + REPLAY_EPSILON_MS < self.last_position_ms
            && current.position_ms < REPLAY_EPSILON_MS;
        if is_new || replayed {
            self.key = Some(current.key.clone());
            self.started_at = unix_now();
            self.last_position_ms = current.position_ms;
            self.submitted = false;
            self.retry_after = None;
            return false;
        }
        self.last_position_ms = current.position_ms;

        if self.submitted || current.duration_ms == 0 {
            return false;
        }
        if let Some(at) = self.retry_after {
            if Instant::now() < at {
                return false;
            }
        }
        current.position_ms >= (current.duration_ms / 2).min(SCROBBLE_CAP_MS)
    }
}

/// Start the scrobbler if it is enabled and a Rocksky token is available.
pub fn spawn(tracklist: Arc<Mutex<Tracklist>>) {
    let enabled = read_settings()
        .ok()
        .and_then(|config| config.get_bool("scrobble").ok())
        .unwrap_or(true);
    if !enabled {
        return;
    }
    let Some(token) = read_token() else {
        info!(
            "Rocksky scrobbling disabled: no token at {} (run `rocksky login`)",
            token_path().display()
        );
        return;
    };
    let api_url = read_settings()
        .ok()
        .and_then(|config| config.get_string("rocksky_api_url").ok())
        .unwrap_or_else(|| "https://api.rocksky.app".to_string());
    tokio::spawn(scrobble_loop(tracklist, api_url, token));
}

async fn scrobble_loop(tracklist: Arc<Mutex<Tracklist>>, api_url: String, token: String) {
    let appview = AppView::new(api_url).with_token(token);
    let mut watcher = Watcher::default();
    loop {
        tokio::time::sleep(TICK).await;

        let Some(current) = current_track(&tracklist) else {
            watcher.key = None;
            continue;
        };
        if !watcher.advance(&current) {
            continue;
        }
        if current.title.is_empty() || current.artist.is_empty() {
            // Nothing identifies this track — don't post a nameless scrobble.
            continue;
        }

        let album_artist = if current.album_artist.is_empty() {
            current.artist.clone()
        } else {
            current.album_artist.clone()
        };
        let input = ScrobbleInput {
            title: current.title.clone(),
            artist: current.artist.clone(),
            album_artist,
            album: Some(current.album.clone()).filter(|a| !a.is_empty()),
            duration: Some(current.duration_ms as u64).filter(|d| *d > 0),
            timestamp: Some(watcher.started_at),
            track_number: current.track_number,
            ..Default::default()
        };
        match appview.create_scrobble(&input).await {
            Ok(_) => {
                watcher.submitted = true;
                info!(artist = %input.artist, title = %input.title, "scrobbled");
            }
            Err(e) => {
                watcher.retry_after = Some(Instant::now() + RETRY_AFTER);
                warn!(
                    "scrobble failed, retrying in {}s: {}",
                    RETRY_AFTER.as_secs(),
                    e
                );
            }
        }
    }
}
