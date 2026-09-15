//! Rocksky auto-scrobbling for the daemon.
//!
//! Ticks once a second off the shared tracklist state and submits a scrobble
//! when a play crosses Last.fm's rule: half the track or 4 minutes, whichever
//! comes first. Mirrors rocksky's `playerd` scrobbler.
//!
//! Internet radio takes a separate path ([`radio_tick`]): a live stream has no
//! duration to take half of, and its "track" is whatever the station last
//! announced over ICY.
//!
//! Requires the access token written by `rocksky login` to
//! `~/.rocksky/token.json`; without it the scrobbler stays off. It can also be
//! disabled with `scrobble = false` in `settings.toml`.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use music_player_playback::player::{icy_now_playing, RADIO_ID_PREFIX};
use music_player_settings::read_settings;
use music_player_tracklist::Tracklist;
use rocksky_sdk::{AppView, ScrobbleInput};
use tracing::{info, warn};

pub(crate) const TICK: Duration = Duration::from_secs(1);

/// Last.fm's rule: half the track, capped at 4 minutes.
pub(crate) const SCROBBLE_CAP_MS: u32 = 4 * 60 * 1000;

/// Back-off before retrying a scrobble the server rejected (offline, expired
/// token…), so a failing submit doesn't retry every tick.
const RETRY_AFTER: Duration = Duration::from_secs(30);

/// A position drop this large, landing near the start, means the track began
/// again (repeat, or the same file queued twice) rather than a seek.
pub(crate) const REPLAY_EPSILON_MS: u32 = 5_000;

pub(crate) struct Current {
    pub(crate) key: String,
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) artist: String,
    pub(crate) album_artist: String,
    pub(crate) album: String,
    pub(crate) duration_ms: u32,
    pub(crate) position_ms: u32,
    pub(crate) track_number: Option<i32>,
    /// Where the audio comes from — what play_stats derives the source from.
    pub(crate) uri: String,
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

pub(crate) fn current_track(tracklist: &Arc<Mutex<Tracklist>>) -> Option<Current> {
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
        id: track.id.clone(),
        title: track.title.clone(),
        artist: track.artist.clone(),
        album_artist: track.album.artist.clone(),
        album: track.album.title.clone(),
        duration_ms: (track.duration.unwrap_or(0.0) * 1000.0) as u32,
        position_ms,
        track_number: track.track.map(|n| n as i32).filter(|n| *n > 0),
        uri: track.uri.clone(),
    })
}

pub(crate) fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

/// Per-play bookkeeping. One instance lives in the loop.
#[derive(Default)]
pub(crate) struct Watcher {
    pub(crate) key: Option<String>,
    /// Wall clock when this play started — the scrobble's `timestamp`.
    pub(crate) started_at: i64,
    last_position_ms: u32,
    /// Whether this play has been counted/submitted. `advance` keeps
    /// returning true until the CALLER sets this (the scrobbler only marks it
    /// after the network submit succeeded, so a failed submit retries) — a
    /// caller with nothing to retry must mark it immediately or it will
    /// count the same play once per tick.
    pub(crate) submitted: bool,
    pub(crate) retry_after: Option<Instant>,
}

impl Watcher {
    /// Track identity/replay bookkeeping. Returns true when this tick is the
    /// one that crosses the scrobble threshold.
    pub(crate) fn advance(&mut self, current: &Current) -> bool {
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

/// How long a station must keep announcing the same song before it is
/// scrobbled. Radio has no duration to take half of, so Last.fm's floor stands
/// in for it: 30 seconds is long enough that a jingle between two songs, or a
/// station tuned away from straight away, never reaches a scrobble.
pub(crate) const RADIO_MIN_PLAY: Duration = Duration::from_secs(30);

/// The song a station is announcing right now, as `(artist, title)`.
///
/// `None` unless the `StreamTitle` actually named both — a bare title, an
/// empty announcement or a station ident does not identify a song, and a
/// station's own name in the artist slot is the player's display fallback, not
/// an artist. Those are exactly the announcements that must not be scrobbled.
fn icy_song() -> Option<(String, String)> {
    let icy = icy_now_playing()?;
    let artist = icy.artist.trim().to_string();
    let title = icy.title.trim().to_string();
    if artist.is_empty() || title.is_empty() {
        return None;
    }
    Some((artist, title))
}

/// Per-announcement bookkeeping, the live-stream counterpart of [`Watcher`].
#[derive(Default)]
pub(crate) struct RadioWatcher {
    /// The announcement being played, as `artist\u{1}title`.
    pub(crate) key: Option<String>,
    /// Wall clock when this announcement first appeared — the scrobble's
    /// `timestamp`.
    pub(crate) started_at: i64,
    /// How long it has been on the air. Counted in ticks rather than off a
    /// clock so a paused stream does not age towards a scrobble it never
    /// played.
    played: Duration,
    /// Whether this announcement has been settled — scrobbled, or matched
    /// against nothing and deliberately dropped. Either way it is not looked
    /// at again until the metadata changes.
    pub(crate) submitted: bool,
    pub(crate) retry_after: Option<Instant>,
}

impl RadioWatcher {
    /// Count one playing tick of `(artist, title)`. Returns true on the tick
    /// that crosses [`RADIO_MIN_PLAY`], which is the one that scrobbles.
    pub(crate) fn advance(&mut self, artist: &str, title: &str) -> bool {
        let key = format!("{artist}\u{1}{title}");
        if self.key.as_deref() != Some(key.as_str()) {
            // The metadata changed: a new song is on the air.
            self.key = Some(key);
            self.started_at = unix_now();
            self.played = Duration::ZERO;
            self.submitted = false;
            self.retry_after = None;
            return false;
        }
        self.played += TICK;

        if self.submitted {
            return false;
        }
        if let Some(at) = self.retry_after {
            if Instant::now() < at {
                return false;
            }
        }
        self.played >= RADIO_MIN_PLAY
    }

    /// Drop what was on the air, so the next announcement — even an identical
    /// one — counts as a new play. For when the stream itself ends.
    pub(crate) fn forget(&mut self) {
        self.key = None;
    }
}

/// Turn an `app.rocksky.song.matchSong` answer into a scrobble, carrying over
/// the metadata a radio stream never sends: the album, its artwork, the real
/// duration, the track number.
///
/// `None` when the catalogue did not recognise the announcement. An unmatched
/// one is as likely to be an ad, a show title or a mangled `StreamTitle` as a
/// song, and none of those belong in a listening history.
fn matched_scrobble(matched: &serde_json::Value, timestamp: i64) -> Option<ScrobbleInput> {
    let text = |key: &str| {
        matched
            .get(key)
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    };
    let number = |key: &str| matched.get(key).and_then(serde_json::Value::as_i64);
    let title = text("title")?;
    let artist = text("artist")?;
    Some(ScrobbleInput {
        album_artist: text("albumArtist").unwrap_or_else(|| artist.clone()),
        album: text("album"),
        duration: number("duration").filter(|d| *d > 0).map(|d| d as u64),
        album_art: text("albumArt"),
        timestamp: Some(timestamp),
        track_number: number("trackNumber").map(|n| n as i32).filter(|n| *n > 0),
        genres: text("genre").map(|genre| vec![genre]),
        release_date: text("releaseDate"),
        year: number("year").map(|y| y as i32).filter(|y| *y > 0),
        title,
        artist,
        ..Default::default()
    })
}

/// One tick of a playing internet-radio stream: scrobble the announced song
/// once it has been on the air long enough and the catalogue recognises it.
async fn radio_tick(appview: &AppView, radio: &mut RadioWatcher) {
    let Some((artist, title)) = icy_song() else {
        // Nothing that names a song. The announcement is kept as it was, so a
        // jingle in the middle of a song does not restart — or re-scrobble —
        // the play around it.
        return;
    };
    if !radio.advance(&artist, &title) {
        return;
    }

    // Match before scrobbling: the catalogue is what turns "Artist - Title"
    // off a wire into a real song, and a miss is the signal that this was
    // never a song to begin with.
    let matched = match appview.match_song(&title, &artist, None, None, None).await {
        Ok(matched) => matched,
        Err(e) => {
            radio.retry_after = Some(Instant::now() + RETRY_AFTER);
            warn!(
                "could not match {artist} - {title}, retrying in {}s: {e}",
                RETRY_AFTER.as_secs()
            );
            return;
        }
    };
    let Some(input) = matched_scrobble(&matched, radio.started_at) else {
        radio.submitted = true;
        info!(%artist, %title, "the station announced nothing the catalogue knows; not scrobbling");
        return;
    };

    match appview.create_scrobble(&input).await {
        Ok(_) => {
            radio.submitted = true;
            info!(artist = %input.artist, title = %input.title, "scrobbled from radio");
        }
        Err(e) => {
            radio.retry_after = Some(Instant::now() + RETRY_AFTER);
            warn!(
                "radio scrobble failed, retrying in {}s: {}",
                RETRY_AFTER.as_secs(),
                e
            );
        }
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
    let mut radio = RadioWatcher::default();
    loop {
        tokio::time::sleep(TICK).await;

        // Checked whatever is playing: the ICY state outlives the tracklist
        // snapshot, and its disappearance — the stream stopped, or a local
        // file took over — is what ends the song that was on the air. A pause
        // leaves it alone, so resuming continues the same play.
        if icy_now_playing().is_none() {
            radio.forget();
        }

        let Some(current) = current_track(&tracklist) else {
            watcher.key = None;
            continue;
        };
        if current.id.starts_with(RADIO_ID_PREFIX) {
            // A live stream has neither a duration to measure a play against
            // nor tags of its own; the rules in `radio_tick` replace both.
            watcher.key = None;
            radio_tick(&appview, &mut radio).await;
            continue;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Ticks of the same announcement, up to the count given.
    fn play(radio: &mut RadioWatcher, artist: &str, title: &str, ticks: u32) -> bool {
        (0..ticks).fold(false, |crossed, _| radio.advance(artist, title) || crossed)
    }

    #[test]
    fn scrobbles_a_song_that_stays_on_the_air() {
        let ticks = RADIO_MIN_PLAY.as_secs() as u32 / TICK.as_secs() as u32;
        let mut radio = RadioWatcher::default();
        // The first tick only records the announcement, so the threshold is
        // crossed one tick after that.
        assert!(!play(&mut radio, "Aretha Franklin", "Respect", ticks));
        assert!(radio.advance("Aretha Franklin", "Respect"));
    }

    #[test]
    fn a_short_announcement_never_scrobbles() {
        let mut radio = RadioWatcher::default();
        assert!(!play(
            &mut radio,
            "Station ID",
            "You are listening to us",
            20
        ));
    }

    #[test]
    fn changed_metadata_starts_a_new_play() {
        let ticks = RADIO_MIN_PLAY.as_secs() as u32 / TICK.as_secs() as u32 + 1;
        let mut radio = RadioWatcher::default();
        assert!(play(&mut radio, "Aretha Franklin", "Respect", ticks));
        radio.submitted = true;
        // Same song, already scrobbled: nothing more comes of it.
        assert!(!play(&mut radio, "Aretha Franklin", "Respect", ticks));
        // The next song has to earn its own threshold, and then scrobbles.
        assert!(!radio.advance("Otis Redding", "Try a Little Tenderness"));
        assert!(!radio.submitted);
        assert!(play(
            &mut radio,
            "Otis Redding",
            "Try a Little Tenderness",
            ticks
        ));
    }

    #[test]
    fn a_stopped_stream_lets_the_same_song_play_again() {
        let ticks = RADIO_MIN_PLAY.as_secs() as u32 / TICK.as_secs() as u32 + 1;
        let mut radio = RadioWatcher::default();
        assert!(play(&mut radio, "Aretha Franklin", "Respect", ticks));
        radio.submitted = true;
        radio.forget();
        assert!(play(&mut radio, "Aretha Franklin", "Respect", ticks));
    }

    #[test]
    fn an_unmatched_announcement_is_not_scrobbled() {
        // What `matchSong` answers when it recognises nothing.
        assert!(matched_scrobble(&serde_json::json!({}), 0).is_none());
        assert!(matched_scrobble(&serde_json::json!({"title": "Respect"}), 0).is_none());
        assert!(matched_scrobble(
            &serde_json::json!({"title": " ", "artist": "Aretha Franklin"}),
            0
        )
        .is_none());
    }

    #[test]
    fn a_match_fills_in_what_the_stream_could_not_say() {
        let input = matched_scrobble(
            &serde_json::json!({
                "title": "Respect",
                "artist": "Aretha Franklin",
                "album": "I Never Loved a Man the Way I Love You",
                "duration": 147000,
                "trackNumber": 1,
                "albumArt": "https://example.test/cover.jpg",
                "genre": "Soul",
                "year": 1967,
            }),
            1_700_000_000,
        )
        .expect("a matched song scrobbles");
        assert_eq!(input.title, "Respect");
        assert_eq!(
            input.album.as_deref(),
            Some("I Never Loved a Man the Way I Love You")
        );
        // No album artist of its own: the artist stands in, as elsewhere.
        assert_eq!(input.album_artist, "Aretha Franklin");
        assert_eq!(input.duration, Some(147000));
        assert_eq!(input.track_number, Some(1));
        assert_eq!(input.genres.as_deref(), Some(&["Soul".to_string()][..]));
        assert_eq!(input.timestamp, Some(1_700_000_000));
    }
}
