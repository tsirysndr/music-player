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
use tracing::{debug, info, warn};

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

/// How many times the catalogue is asked about an announcement before it is
/// written off as not a song.
///
/// `matchSong` reaches out to external metadata providers, and when they do
/// not answer it returns an empty body rather than an error — which reads
/// exactly like "this is not a song". Songs the catalogue plainly knows were
/// being dropped on one such answer, so a miss has to repeat before it counts.
pub(crate) const MATCH_ATTEMPTS: u32 = 3;

/// What the station is announcing right now, as it came off the wire: the
/// engine's `Artist - Title` split when it found one, else the whole
/// `StreamTitle` with nothing in the artist slot.
///
/// The halves are not believed here — [`readings`] decides what they mean.
fn icy_announcement() -> Option<(String, String)> {
    let icy = icy_now_playing()?;
    let title = icy.title.trim().to_string();
    if title.is_empty() {
        return None;
    }
    Some((icy.artist.trim().to_string(), title))
}

/// What a station may put between the two halves of an announcement. The
/// spaces are required: an unspaced hyphen belongs to `Jay-Z` and `Blink-182`
/// at least as often as it separates anything.
const SEPARATORS: [&str; 3] = [" - ", " – ", " — "];

/// Every reading of an announcement worth asking the catalogue about, as
/// `(artist, title)`, likeliest first. Empty when the announcement names no
/// two halves at all — a station ident, a jingle, a bare song title.
///
/// `Artist - Title` is the common form and the one the engine assumes, but
/// nothing enforces it: stations announce `Title - Artist` too, and reading
/// that one the usual way scrobbles a song with its own title in the artist
/// slot. Rather than guess from the text, both readings are offered and the
/// catalogue settles it — the one that resolves is the true one. Splitting
/// here also catches the en and em dashes, which the engine does not treat as
/// separators at all.
fn readings(artist: &str, title: &str) -> Vec<(String, String)> {
    let (artist, title) = (artist.trim(), title.trim());
    if !artist.is_empty() && !title.is_empty() {
        return vec![
            (artist.to_string(), title.to_string()),
            (title.to_string(), artist.to_string()),
        ];
    }
    for separator in SEPARATORS {
        let Some((left, right)) = title.split_once(separator) else {
            continue;
        };
        let (left, right) = (left.trim(), right.trim());
        if !left.is_empty() && !right.is_empty() {
            return vec![
                (left.to_string(), right.to_string()),
                (right.to_string(), left.to_string()),
            ];
        }
    }
    Vec::new()
}

/// Letters and digits, lowercased. A station's punctuation is not to be
/// trusted — `Say It Ain\`t So` comes off the wire with a backtick, and
/// `Guns n Roses` without the apostrophe the catalogue spells it with.
fn squashed(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Whether an answered name is the asked-for one. Containment either way: the
/// catalogue answers `The Killers` for a station's `Killers`, credits a
/// collaboration to every artist on it, and returns `Say It Ain't So
/// (Original Mix)` for a plain `Say It Ain't So`.
fn corresponds(answered: &str, asked: &str) -> bool {
    let (answered, asked) = (squashed(answered), squashed(asked));
    !answered.is_empty()
        && !asked.is_empty()
        && (answered.contains(&asked) || asked.contains(&answered))
}

/// Whether the catalogue answered about the halves it was asked about, rather
/// than merely answering something.
///
/// `matchSong` is fuzzy to a fault: handed a song title as if it were an
/// artist it still resolves *something* — `Muse`/`Madness` the wrong way round
/// comes back as Marillion's "Muse in the Madness". So a reading is only taken
/// as the right one when both halves come back recognisable, which is what
/// tells `Artist - Title` from `Title - Artist` apart.
fn confirms(matched: &serde_json::Value, artist: &str, title: &str) -> bool {
    let answered = |key: &str| {
        matched
            .get(key)
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string()
    };
    corresponds(&answered("artist"), artist) && corresponds(&answered("title"), title)
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
    /// How many times the catalogue has answered nothing for this
    /// announcement.
    match_misses: u32,
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
            self.match_misses = 0;
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

    /// Note that the catalogue answered nothing for this announcement.
    /// Returns true once it has been asked [`MATCH_ATTEMPTS`] times, which is
    /// when the announcement is written off; until then it is asked again
    /// after [`RETRY_AFTER`].
    pub(crate) fn missed_match(&mut self) -> bool {
        self.match_misses += 1;
        if self.match_misses >= MATCH_ATTEMPTS {
            return true;
        }
        self.retry_after = Some(Instant::now() + RETRY_AFTER);
        false
    }

    /// Drop what was on the air, so the next announcement — even an identical
    /// one — counts as a new play. For when the stream itself ends.
    pub(crate) fn forget(&mut self) {
        self.key = None;
    }
}

/// The `YYYY-MM-DD` half of a release date, or `None` when there is no
/// calendar date in it at all.
///
/// The catalogue answers with whatever it stored, and most records carry a
/// full ISO-8601 timestamp — `2012-10-01T00:00:00.000Z`. `createScrobble`
/// validates the field strictly and answers `400 Invalid scrobble:
/// releaseDate Invalid date format. Use YYYY-MM-DD.`, which loses the whole
/// scrobble over a field nothing needed. Trimming it keeps the date; anything
/// that is not one is dropped rather than guessed at.
fn calendar_date(value: &str) -> Option<String> {
    let date = value.split('T').next()?;
    let mut parts = date.split('-');
    let (year, month, day) = (parts.next()?, parts.next()?, parts.next()?);
    let shaped = parts.next().is_none()
        && (year.len(), month.len(), day.len()) == (4, 2, 2)
        && [year, month, day]
            .iter()
            .all(|part| part.bytes().all(|b| b.is_ascii_digit()));
    shaped.then(|| date.to_string())
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
        release_date: text("releaseDate").and_then(|date| calendar_date(&date)),
        year: number("year").map(|y| y as i32).filter(|y| *y > 0),
        title,
        artist,
        ..Default::default()
    })
}

/// One tick of a playing internet-radio stream: scrobble the announced song
/// once it has been on the air long enough and the catalogue recognises it.
async fn radio_tick(appview: &AppView, radio: &mut RadioWatcher) {
    let Some((announced_artist, announced_title)) = icy_announcement() else {
        return;
    };
    let readings = readings(&announced_artist, &announced_title);
    if readings.is_empty() {
        // Nothing that names a song. The announcement is kept as it was, so a
        // jingle in the middle of a song does not restart — or re-scrobble —
        // the play around it.
        return;
    }
    if !radio.advance(&announced_artist, &announced_title) {
        return;
    }

    // Match before scrobbling: the catalogue is what turns two halves off a
    // wire into a real song, which reading of them is the right one, and
    // whether this was a song at all.
    // The likeliest reading is asked about first and, when the catalogue
    // confirms it, nothing else is asked at all. Only when it does not is the
    // other reading tried — and it has to be confirmed to win, since an
    // unconfirmed answer is what the wrong reading produces too. If neither is
    // confirmed the first reading's answer still stands: the catalogue spells
    // plenty of songs differently from the station that announced them, and
    // dropping those would cost more plays than it saves.
    let mut found = None;
    let mut unconfirmed = None;
    for (index, (artist, title)) in readings.iter().enumerate() {
        let matched = match appview.match_song(title, artist, None, None, None).await {
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
        let confirmed = confirms(&matched, artist, title);
        let Some(input) = matched_scrobble(&matched, radio.started_at) else {
            continue;
        };
        if confirmed {
            if index > 0 {
                info!(
                    artist = %announced_artist, title = %announced_title,
                    "the station announced the title first; scrobbling as {artist} - {title}"
                );
            }
            found = Some(input);
            break;
        }
        unconfirmed.get_or_insert(input);
    }

    let Some(input) = found.or(unconfirmed) else {
        if radio.missed_match() {
            radio.submitted = true;
            info!(
                artist = %announced_artist, title = %announced_title,
                "the station announced nothing the catalogue knows; not scrobbling"
            );
        } else {
            debug!(
                artist = %announced_artist, title = %announced_title,
                "the catalogue answered nothing; asking again in {}s",
                RETRY_AFTER.as_secs()
            );
        }
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

    /// `readings`, as `(artist, title)` pairs, for terser assertions.
    fn read(artist: &str, title: &str) -> Vec<(String, String)> {
        readings(artist, title)
    }

    #[test]
    fn both_orders_are_offered_for_the_catalogue_to_settle() {
        // What the engine split: the usual reading first, the reverse behind
        // it, because a station announcing title first is the rarer one.
        assert_eq!(
            read("Muse", "Madness"),
            vec![
                ("Muse".to_string(), "Madness".to_string()),
                ("Madness".to_string(), "Muse".to_string()),
            ]
        );
    }

    #[test]
    fn a_dash_the_engine_does_not_know_still_splits() {
        // The engine only splits on " - ", so these arrive whole.
        for announcement in ["Muse – Madness", "Muse — Madness"] {
            assert_eq!(
                read("", announcement),
                vec![
                    ("Muse".to_string(), "Madness".to_string()),
                    ("Madness".to_string(), "Muse".to_string()),
                ],
                "{announcement}"
            );
        }
    }

    #[test]
    fn an_announcement_that_names_no_two_halves_is_not_a_song() {
        // Idents and jingles, which must not count as a play at all.
        assert!(read("", "AlternativeRadio.us").is_empty());
        assert!(read("", "Radio For The Rest Of Us").is_empty());
        // An unspaced hyphen is part of a name far more often than it is a
        // separator, so it is left alone.
        assert!(read("", "Jay-Z").is_empty());
        assert!(read("", "Blink-182").is_empty());
        // Half an announcement names nothing either.
        assert!(read("", "Muse - ").is_empty());
        assert!(read("", " - Madness").is_empty());
    }

    #[test]
    fn only_the_first_split_separates_the_halves() {
        // Artists rarely carry " - "; titles do.
        assert_eq!(
            read("Muse", "Madness - Live at Rome").first().unwrap(),
            &("Muse".to_string(), "Madness - Live at Rome".to_string())
        );
        assert_eq!(
            read("", "Muse – Madness – Live at Rome").first().unwrap(),
            &("Muse".to_string(), "Madness – Live at Rome".to_string())
        );
    }

    #[test]
    fn the_padding_stations_send_is_not_part_of_a_name() {
        // Every `StreamTitle` this station sends ends in a space.
        assert_eq!(
            read("Killers", "When You Were Young "),
            read("Killers", "When You Were Young")
        );
        assert_eq!(read("", " Muse – Madness "), read("", "Muse – Madness"));
        // Whitespace in the artist slot is an empty artist slot.
        assert!(read("   ", "AlternativeRadio.us").is_empty());
        assert_eq!(read("  ", "Muse - Madness").len(), 2);
    }

    #[test]
    fn a_spaced_hyphen_wins_over_the_longer_dashes() {
        // Both are present: the one the engine itself would have used decides,
        // so a title carrying an en dash is not split on it.
        assert_eq!(
            read(
                "",
                "Godspeed You! Black Emperor - Dead Flag Blues – Reprise"
            )
            .first()
            .unwrap(),
            &(
                "Godspeed You! Black Emperor".to_string(),
                "Dead Flag Blues – Reprise".to_string()
            )
        );
    }

    #[test]
    fn an_ad_break_is_read_like_anything_else() {
        // The station announces its ads as a perfectly well-formed pair, so
        // both readings are asked about and both miss — which is what writes
        // the break off. Nothing here can tell it from a song on its own.
        assert_eq!(read("Live365", "Advertisement").len(), 2);
    }

    #[test]
    fn no_reading_ever_carries_an_empty_half() {
        for (artist, title) in [
            ("Muse", "Madness"),
            ("", "Muse - Madness"),
            ("", "Muse – Madness"),
            ("", "Muse — Madness"),
            ("Live365", "Advertisement"),
            ("", "AlternativeRadio.us"),
            ("", " - "),
            ("", ""),
        ] {
            for (artist, title) in read(artist, title) {
                assert!(!artist.is_empty() && !title.is_empty(), "{artist}/{title}");
            }
        }
    }

    /// A `matchSong` answer, as the fields `confirms` reads.
    fn answer(artist: &str, title: &str) -> serde_json::Value {
        serde_json::json!({"artist": artist, "title": title})
    }

    #[test]
    fn a_confirmed_reading_is_one_the_catalogue_answered_about() {
        // Both halves come back recognisable.
        assert!(confirms(&answer("Muse", "Madness"), "Muse", "Madness"));
        // A fuller name than the station announced, on either half.
        assert!(confirms(
            &answer("The Killers", "When You Were Young"),
            "Killers",
            "When You Were Young"
        ));
        assert!(confirms(
            &answer("Weezer", "Say It Ain't So (Original Mix)"),
            "Weezer",
            "Say It Ain`t So"
        ));
        // A collaboration credits everyone on it.
        assert!(confirms(
            &answer("Elley Duhé, Whethan", "MONEY ON THE DASH"),
            "Whethan",
            "Money On The Dash"
        ));
        // Punctuation the station dropped.
        assert!(confirms(
            &answer("Guns N' Roses", "Patience"),
            "Guns n Roses",
            "Patience"
        ));
    }

    #[test]
    fn the_answer_to_a_backwards_reading_is_not_confirmed() {
        // What the catalogue really answers for `Muse`/`Madness` read the
        // wrong way round — an answer, but not about what was asked.
        let backwards = answer("Marillion", "Muse in the Madness");
        assert!(!confirms(&backwards, "Madness", "Muse"));
        // Read the right way round, the same announcement confirms.
        assert!(confirms(&answer("Muse", "Madness"), "Muse", "Madness"));

        // And the other one seen live: asking for `Nirvana` as a title.
        let backwards = answer("Nirvana", "On A Plain (Live In Tokyo, Japan/1992)");
        assert!(!confirms(&backwards, "In Bloom", "Nirvana"));
        assert!(confirms(
            &answer("Nirvana", "In Bloom"),
            "Nirvana",
            "In Bloom"
        ));
    }

    #[test]
    fn an_answer_missing_a_half_confirms_nothing() {
        assert!(!confirms(&serde_json::json!({}), "Muse", "Madness"));
        assert!(!confirms(&answer("Muse", ""), "Muse", "Madness"));
        assert!(!confirms(&answer("", "Madness"), "Muse", "Madness"));
        // Nothing asked for is nothing to confirm against either.
        assert!(!confirms(&answer("Muse", "Madness"), "", "Madness"));
    }

    #[test]
    fn punctuation_and_case_are_not_part_of_a_name() {
        assert_eq!(squashed("Guns N' Roses"), "gunsnroses");
        assert_eq!(squashed("Guns n Roses"), "gunsnroses");
        assert_eq!(squashed("AC/DC"), "acdc");
        assert_eq!(squashed("P!nk"), "pnk");
        assert_eq!(squashed("Blink-182"), "blink182");
        assert_eq!(squashed(" - "), "");
        // Accents are not punctuation: they distinguish names.
        assert_eq!(squashed("Elley Duhé"), "elleyduhé");
        assert!(corresponds(
            "Say It Ain't So (Original Mix)",
            "Say It Ain`t So"
        ));
        assert!(!corresponds("Madness", "Muse"));
        assert!(!corresponds("", "Muse"));
    }

    #[test]
    fn an_empty_answer_is_asked_again_before_it_is_written_off() {
        let ticks = RADIO_MIN_PLAY.as_secs() as u32 / TICK.as_secs() as u32 + 1;
        let mut radio = RadioWatcher::default();
        assert!(play(&mut radio, "Pixies", "Where Is My Mind", ticks));
        // The providers behind matchSong time out; the announcement is kept
        // and asked about again rather than written off on one empty answer.
        assert!(!radio.missed_match());
        assert!(!radio.missed_match());
        assert!(radio.missed_match());

        // A new announcement starts its own count: the song after a genuinely
        // unknown one still gets every attempt.
        assert!(!radio.advance("Nirvana", "In Bloom"));
        assert!(!radio.missed_match());
    }

    #[test]
    fn a_release_date_is_cut_down_to_a_calendar_date() {
        // What the catalogue answers for most records.
        assert_eq!(
            calendar_date("2012-10-01T00:00:00.000Z").as_deref(),
            Some("2012-10-01")
        );
        // And what it answers for the rest, which already passes validation.
        assert_eq!(calendar_date("2005-12-12").as_deref(), Some("2005-12-12"));
        // Nothing to salvage: sent as no release date at all rather than as a
        // guess the endpoint would reject.
        assert_eq!(calendar_date("2012"), None);
        assert_eq!(calendar_date("2012-10"), None);
        assert_eq!(calendar_date("October 2012"), None);
        assert_eq!(calendar_date("2012-1-1"), None);
        assert_eq!(calendar_date("20xx-10-01"), None);
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
                "releaseDate": "1967-03-10T00:00:00.000Z",
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
        // Trimmed on the way in: the timestamp the catalogue stores is not a
        // date the scrobble endpoint accepts.
        assert_eq!(input.release_date.as_deref(), Some("1967-03-10"));
        assert_eq!(input.timestamp, Some(1_700_000_000));
    }
}
