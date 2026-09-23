//! Working out what the library actually contains.
//!
//! Two passes, in order, both of them optional extras on top of a scan that
//! has already finished:
//!
//! 1. **Fingerprinting** every local file — a Chromaprint of its audio, which
//!    is an identity the tags cannot lie about. Local, cheap-ish, and useful
//!    on its own: it is what a duplicate check compares.
//! 2. **Identifying** the ones whose tags are missing, by asking AcoustID what
//!    that fingerprint is, and filling in the blanks from the MusicBrainz
//!    recording it names. This is what Picard does, minus the window.
//!
//! Neither runs as part of `refresh_music_library`, for the reason key and
//! tempo do not: they decode audio and talk to the network, which is minutes
//! of work with nobody waiting on it, and whether to await or detach that is
//! the caller's decision to make — a command that exits must await it, a
//! daemon that keeps running should not.
//!
//! Both are resumable and neither repeats itself. A fingerprint written is a
//! track out of the queue; a lookup recorded — hit *or* miss — is a track out
//! of the other one.

use crate::Pace;
use futures::stream::{self, StreamExt};
use music_player_entity::{album, track};
use music_player_storage::track_fingerprint;
use music_player_types::types::Song;
use sea_orm::EntityTrait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tracing::{info, warn};

use crate::acoustid::{AcoustId, Tags};

/// How many tracks are fetched from the database at a time. A paging size,
/// not a limit on the work: the pass keeps asking until nothing is left.
const PAGE: u64 = 500;

/// How much of the time a background pass may spend working, as a divisor of
/// the idle that follows each track — the same ratio the analysis pass uses,
/// and for the same reason: nothing is waiting on this, and a player that
/// stutters while its library is being catalogued is a worse player.
const BACKGROUND_REST: u32 = 3;

/// The longest a background worker pauses between tracks, however slow the
/// last one was.
const MAX_REST: Duration = Duration::from_secs(5);

/// How many files are fingerprinted at once.
///
/// Fingerprinting is CPU-bound and, unlike full analysis, bounded: two minutes
/// of audio per track however long the track is. That is what makes running
/// several at once reasonable here when it is not there — but only several.
/// Left unbounded it would take every core, and the machine is meant to be
/// playing music.
fn parallelism(pace: Pace) -> usize {
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    match pace {
        // The user is watching `music-player scan` and waiting for it. Still
        // capped: past a handful of workers this is waiting on the disk, not
        // on the CPU, and more of them only makes the queue longer.
        Pace::Foreground => cores.clamp(1, 8),
        // A quarter of the machine at most, and then each worker rests.
        Pace::Background => (cores / 4).clamp(1, 2),
    }
}

/// Fingerprint every local file that has not been. Returns how many it did.
///
/// Skips anything already fingerprinted, so running this over a library that
/// has been through it costs one query and nothing else.
pub async fn fingerprint_library(db: &music_player_storage::Database, pace: Pace) -> usize {
    let conn = db.get_connection();
    let total = track_fingerprint::unfingerprinted_count(conn).await as usize;
    if total == 0 {
        return 0;
    }
    let workers = parallelism(pace);
    info!(tracks = total, workers, "fingerprinting the library");

    let done = AtomicUsize::new(0);
    // Tracks that cannot be fingerprinted at all — a missing file, a codec we
    // do not read, something too short to have an identity. A success writes a
    // row and the track leaves the queue; a failure does not, so it would sit
    // at the head of every page and be retried for ever. Counting them is what
    // the offset skips past.
    let mut failed = 0usize;

    loop {
        let pending = track_fingerprint::unfingerprinted_tracks(conn, failed as u64, PAGE).await;
        if pending.is_empty() {
            break;
        }

        let outcomes: Vec<bool> = stream::iter(pending)
            .map(|track| {
                let done = &done;
                async move {
                    let started = Instant::now();
                    let uri = track.uri.clone();
                    // Decoding is CPU-bound and takes seconds. On the async
                    // runtime it would stall every other task on that thread,
                    // including playback's own timer.
                    let computed = tokio::task::spawn_blocking(move || {
                        music_player_analysis::fingerprint_file(&uri)
                    })
                    .await;

                    let succeeded = match computed {
                        Ok(Ok(fingerprint)) => {
                            match track_fingerprint::put(conn, &track.id, &fingerprint).await {
                                Ok(()) => {
                                    let index = done.fetch_add(1, Ordering::Relaxed) + 1;
                                    info!(
                                        "[{index}/{total}] fingerprinted {} — {}",
                                        track.artist, track.title
                                    );
                                    true
                                }
                                Err(cause) => {
                                    warn!(track = track.id, "could not store fingerprint: {cause}");
                                    false
                                }
                            }
                        }
                        Ok(Err(cause)) => {
                            info!(
                                "could not fingerprint {} — {}: {cause}",
                                track.artist, track.title
                            );
                            false
                        }
                        Err(cause) => {
                            warn!(track = track.id, "fingerprinting panicked: {cause}");
                            false
                        }
                    };

                    // Give the machine back to whatever the user is actually
                    // doing. Proportional to the work just done, so the ratio
                    // holds however fast the disk is — and per worker, so it
                    // does not matter how many of them there are.
                    if pace == Pace::Background {
                        tokio::time::sleep((started.elapsed() * BACKGROUND_REST).min(MAX_REST))
                            .await;
                    }
                    succeeded
                }
            })
            .buffer_unordered(workers)
            .collect()
            .await;

        failed += outcomes.iter().filter(|ok| !**ok).count();
    }

    let done = done.into_inner();
    info!("fingerprints: {done}/{total} tracks");
    done
}

/// Ask AcoustID to name the tracks whose tags do not. Returns how many it
/// filled in.
///
/// Only tracks that are actually missing something are asked about. A
/// well-tagged file keeps its own tags — they came from whoever made the file
/// and are more likely right than a guess from a database — and asking about
/// it would spend a slot in someone else's rate limit to learn nothing.
pub async fn identify_unknown_tracks(db: &music_player_storage::Database, pace: Pace) -> usize {
    let conn = db.get_connection();
    let total = track_fingerprint::unidentified_count(conn).await as usize;
    if total == 0 {
        return 0;
    }

    let Some(client) = AcoustId::new(&api_key()) else {
        info!(
            tracks = total,
            "{total} tracks have missing tags and a fingerprint to identify them by, but no \
             AcoustID api key is configured — set `acoustid_api_key` in settings.toml (a free \
             key comes from https://acoustid.org/new-application)"
        );
        return 0;
    };
    info!(tracks = total, "identifying tracks with missing tags");

    // AcoustID asks for no more than three requests a second. Sequential with
    // a gap rather than parallel: this is somebody else's free service, the
    // work is not urgent, and a pass that hammers it gets the key blocked.
    let gap = match pace {
        Pace::Foreground => Duration::from_millis(350),
        Pace::Background => Duration::from_secs(2),
    };

    let mut tagged = 0usize;
    // Every track the pass got an answer about, whether or not the answer
    // changed anything. What the progress counter counts — a pass that only
    // counted the tracks it renamed would appear to stall on a library whose
    // files are simply not in the database.
    let mut answered = 0usize;
    let mut skipped = 0usize;
    // Consecutive request failures. A lookup that errors is almost always the
    // network rather than the track, and working through a whole library one
    // timeout at a time to discover that is worse than stopping and trying
    // again on the next scan.
    let mut consecutive_failures = 0usize;
    const GIVE_UP_AFTER: usize = 5;

    'pass: loop {
        let pending = track_fingerprint::unidentified_tracks(conn, PAGE).await;
        if pending.is_empty() {
            break;
        }
        // Everything in this page has already been asked about and stayed —
        // only the failures do — so there is nothing new to do.
        if pending.len() <= skipped {
            break;
        }

        for waiting in pending.into_iter().skip(skipped) {
            let index = answered + skipped + 1;
            match client.lookup(&waiting.fingerprint, waiting.duration).await {
                Ok(found) => {
                    consecutive_failures = 0;
                    answered += 1;
                    let known = found.is_some();
                    let identity = found.as_ref().map(|f| f.identity.clone());
                    if let Err(cause) =
                        track_fingerprint::record_lookup(conn, &waiting.track.id, identity.as_ref())
                            .await
                    {
                        warn!(track = waiting.track.id, "could not store lookup: {cause}");
                        skipped += 1;
                        continue;
                    }

                    let tags = found.map(|f| f.tags).unwrap_or_default();
                    match apply_tags(db, &waiting.track, &tags).await {
                        Ok(true) => {
                            tagged += 1;
                            info!(
                                "[{index}/{total}] {} — {}",
                                tags.artist.as_deref().unwrap_or("?"),
                                tags.title.as_deref().unwrap_or("?"),
                            );
                        }
                        Ok(false) if known => {
                            // Matched, but everything it could say the file
                            // already said — or AcoustID knows the fingerprint
                            // and nothing is attached to it.
                            info!(
                                "[{index}/{total}] {} matched, nothing to fill in",
                                waiting.track.uri
                            );
                        }
                        Ok(false) => {
                            info!("[{index}/{total}] {} is not in AcoustID", waiting.track.uri);
                        }
                        Err(cause) => {
                            warn!(track = waiting.track.id, "could not apply tags: {cause}");
                        }
                    }
                }
                Err(cause) => {
                    consecutive_failures += 1;
                    skipped += 1;
                    warn!("acoustid lookup failed: {cause}");
                    if consecutive_failures >= GIVE_UP_AFTER {
                        warn!("giving up on identification for now: {GIVE_UP_AFTER} lookups in a row failed");
                        break 'pass;
                    }
                }
            }

            tokio::time::sleep(gap).await;
        }
    }

    info!("identification: {tagged}/{total} tracks tagged from AcoustID");
    if tagged > 0 {
        // A track that moved from "None" to a real album leaves the placeholder
        // album and artist rows behind it with nothing pointing at them.
        if let Err(cause) = crate::prune_orphaned_library_rows(db).await {
            warn!("could not tidy up after identification: {cause}");
        }
    }
    tagged
}

/// Fill in a track's missing tags, leaving everything it already knows alone.
///
/// Returns whether anything changed. Written through the scanner's own upsert
/// rather than by updating the row directly, because a changed album or artist
/// is a *different* album or artist row — ids are derived from the names — and
/// only that path creates them and re-links the track.
async fn apply_tags(
    db: &music_player_storage::Database,
    existing: &track::Model,
    tags: &Tags,
) -> Result<bool, anyhow::Error> {
    use music_player_storage::track_fingerprint::is_unknown;

    if tags.is_empty() {
        return Ok(false);
    }
    let conn = db.get_connection();
    let current_album = match &existing.album_id {
        Some(id) => album::Entity::find_by_id(id.clone()).one(conn).await?,
        None => None,
    };

    // Only where the file says nothing. An AcoustID match is a good guess and
    // the tags on the file are somebody's decision; where they disagree and
    // both are present, the file wins.
    let fill = |mine: &str, theirs: &Option<String>| match theirs {
        Some(found) if is_unknown(mine) => found.clone(),
        _ => mine.to_string(),
    };

    let album_title = current_album
        .as_ref()
        .map(|a| a.title.clone())
        .unwrap_or_default();
    let album_artist = current_album
        .as_ref()
        .map(|a| a.artist.clone())
        .unwrap_or_default();

    let song = Song {
        title: fill(&existing.title, &tags.title),
        artist: fill(&existing.artist, &tags.artist),
        album: fill(&album_title, &tags.album),
        album_artist: fill(&album_artist, &tags.album_artist),
        genre: existing.genre.clone(),
        year: existing.year.or(tags.year),
        track: existing.track.or(tags.track),
        disc: existing.disc.or(tags.disc),
        bitrate: existing.bitrate,
        sample_rate: existing.sample_rate,
        bit_depth: existing.bit_depth,
        channels: existing.channels,
        duration: Duration::from_secs_f32(existing.duration.unwrap_or_default().max(0.0)),
        uri: Some(existing.uri.clone()),
        cover: None,
    };

    let unchanged = song.title == existing.title
        && song.artist == existing.artist
        && song.album == album_title
        && song.album_artist == album_artist
        && song.year == existing.year
        && song.track == existing.track
        && song.disc == existing.disc;
    if unchanged {
        return Ok(false);
    }

    // The album art is filed under the album's id, and the id is derived from
    // its name — so an album that has just been given one has its cover
    // sitting under the placeholder's id where nothing will look for it.
    let song = match song.album != album_title || song.album_artist != album_artist {
        true => re_file_cover(song).await,
        false => song,
    };

    // The artist link is keyed by (artist name, uri), so renaming the artist
    // adds a second one rather than moving the first — and the placeholder
    // artist stays in every artist listing, still claiming the track. Clear
    // the track's links first and let the upsert write the one that is true.
    if song.artist != existing.artist || song.album_artist != album_artist {
        use music_player_entity::artist_tracks;
        use sea_orm::{ColumnTrait, QueryFilter};
        artist_tracks::Entity::delete_many()
            .filter(artist_tracks::Column::TrackId.eq(existing.id.clone()))
            .exec(conn)
            .await?;
    }

    crate::save_songs(db, std::slice::from_ref(&song)).await?;
    Ok(true)
}

/// Re-extract the embedded cover under the album id the track now has.
///
/// Cheap — it reads the file's header, not its audio — and skipped entirely
/// for a file with no art, which is most of the files this pass touches.
async fn re_file_cover(song: Song) -> Song {
    let Some(path) = song.uri.clone() else {
        return song;
    };
    tokio::task::spawn_blocking(move || {
        let mut song = song;
        if let Ok(meta) = rockbox_metadata::read(&path) {
            song.cover =
                crate::extract_and_save_album_cover(&path, &meta, &song.album, &song.album_artist);
        }
        song
    })
    .await
    .unwrap_or_else(|_| Song::default())
}

/// The AcoustID application key, from settings.toml or the environment.
///
/// `ACOUSTID_API_KEY` is honoured as well as the setting because it is what
/// every other tool that talks to AcoustID reads, and a user who already has
/// one exported should not have to write it down again.
fn api_key() -> String {
    if let Ok(key) = std::env::var("ACOUSTID_API_KEY") {
        if !key.trim().is_empty() {
            return key;
        }
    }
    music_player_settings::read_settings()
        .ok()
        .and_then(|config| {
            config
                .try_deserialize::<music_player_settings::Settings>()
                .ok()
        })
        .and_then(|settings| settings.acoustid_api_key)
        .unwrap_or_default()
}

/// Both passes, in order: fingerprint what is new, then identify what is
/// nameless. Awaitable, for `music-player scan`, which exits when it returns.
pub async fn fingerprint_and_identify(db: &music_player_storage::Database, pace: Pace) -> usize {
    fingerprint_library(db, pace).await;
    identify_unknown_tracks(db, pace).await
}

/// The same, detached — for the daemon, which outlives it.
pub fn spawn_fingerprint_and_identify(db: music_player_storage::Database) {
    tokio::spawn(async move {
        fingerprint_and_identify(&db, Pace::Background).await;
    });
}
