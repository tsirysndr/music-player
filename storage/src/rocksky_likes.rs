//! Restore liked songs from the user's atproto repo into the local library.
//!
//! `app.rocksky.like` records only carry a subject uri, so a like is resolved
//! in two steps:
//!
//! 1. **extract** — the user's repo is downloaded once as a CAR archive (see
//!    [`crate::repo_sync`], which is what keeps it to one download per repo)
//!    and scanned for both `app.rocksky.like` and `app.rocksky.song`. Songs
//!    liked from someone else's repo are not in this CAR, so those subjects are
//!    fetched individually with `getRecord`. The result lands in the
//!    `rocksky_like` table, which is the durable copy: a like whose song is not
//!    in the library yet stays there and is re-matched later.
//! 2. **match** — every unmatched like is looked up in the library on
//!    lower(title) + lower(artist) + lower(album). The migration adds
//!    case-insensitive indexes over both sides, so this is an index probe per
//!    like rather than a scan. A match writes the song's `at://` uri to
//!    `track.aturi` (and the album's/artist's, when they resolve too).
//!
//! A Jetstream subscription then applies likes and unlikes as they happen.
//!
//! Everything here is a no-op when no atproto identity can be resolved.

use std::collections::HashMap;

use anyhow::Error;
use music_player_entity::{album, artist, rocksky_like, track};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    QueryFilter, Statement,
};
use serde::Deserialize;

use crate::{atproto, repo_sync};

const LIKE_COLLECTION: &str = "app.rocksky.like";
const SONG_COLLECTION: &str = "app.rocksky.song";

/// How many out-of-repo song records to resolve at once.
const RESOLVE_PARALLEL: usize = 8;

// ── Records ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
struct Subject {
    uri: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LikeRecord {
    subject: Subject,
    #[serde(default)]
    created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SongRecord {
    #[serde(default)]
    title: String,
    #[serde(default)]
    artist: String,
    #[serde(default)]
    album: String,
    #[serde(default)]
    album_artist: String,
}

/// A CAR block also needs its own uri, which the archive does not carry next to
/// the record — the like's rkey is recovered from the MST-independent scan by
/// pairing on the subject instead. See [`extract`].
#[derive(Debug, Clone)]
struct Like {
    uri: String,
    song_uri: String,
    created_at: String,
    song: Option<SongRecord>,
}

// ── Extraction ──────────────────────────────────────────────────────────────

/// Read every like in `car` — `did`'s repo archive — together with the song it
/// points at.
///
/// The CAR gives likes and locally-hosted songs in one pass; `listRecords`
/// supplies the like uris (a CAR block has no record path) and `getRecord`
/// fills in songs that live in other people's repos.
async fn extract(did: &str, car: &[u8]) -> Result<Vec<Like>, Error> {
    // One archive covers both collections: it is walked through its MST, so
    // each record arrives with the path — and therefore the uri — it is stored
    // under.
    let likes: Vec<(String, LikeRecord)> =
        match atproto::records_from_car::<LikeRecord>(car, did, LIKE_COLLECTION) {
            Ok(likes) if !likes.is_empty() => likes,
            Ok(_) => Vec::new(),
            Err(e) => {
                tracing::warn!("could not read likes from the CAR ({e}); using listRecords");
                atproto::list_records(did, LIKE_COLLECTION).await?
            }
        };
    tracing::info!(likes = likes.len(), %did, "found likes in the repo");
    if likes.is_empty() {
        return Ok(Vec::new());
    }

    // Songs hosted in this repo come out of the same archive, so only songs
    // liked from someone else's repo need a request.
    let own_songs: HashMap<String, SongRecord> =
        atproto::records_from_car::<SongRecord>(car, did, SONG_COLLECTION)
            .unwrap_or_default()
            .into_iter()
            .collect();
    tracing::info!(
        own_songs = own_songs.len(),
        "resolved the songs hosted in this repo"
    );

    let mut out: Vec<Like> = likes
        .into_iter()
        .map(|(uri, record)| Like {
            uri,
            song: own_songs.get(&record.subject.uri).cloned(),
            song_uri: record.subject.uri,
            created_at: record.created_at,
        })
        .collect();

    // Anything left points at a song in someone else's repo.
    let missing: Vec<usize> = out
        .iter()
        .enumerate()
        .filter(|(_, like)| like.song.is_none())
        .map(|(i, _)| i)
        .collect();
    if !missing.is_empty() {
        tracing::info!(
            count = missing.len(),
            "resolving liked songs hosted in other repos"
        );
    }
    for (done, chunk) in missing.chunks(RESOLVE_PARALLEL).enumerate() {
        let fetched = futures_util::future::join_all(
            chunk
                .iter()
                .map(|i| atproto::get_record::<SongRecord>(&out[*i].song_uri)),
        )
        .await;
        for (i, song) in chunk.iter().zip(fetched) {
            match song {
                Ok(song) => out[*i].song = Some(song),
                Err(e) => tracing::debug!(uri = %out[*i].song_uri, "could not resolve a song: {e}"),
            }
        }
        tracing::info!(
            progress = format!(
                "{}/{}",
                ((done + 1) * RESOLVE_PARALLEL).min(missing.len()),
                missing.len()
            ),
            "resolving liked songs"
        );
    }
    Ok(out)
}

// ── Local table ─────────────────────────────────────────────────────────────

async fn upsert(conn: &DatabaseConnection, like: &Like) -> Result<(), Error> {
    let Some(song) = &like.song else {
        return Ok(());
    };
    rocksky_like::ActiveModel {
        uri: ActiveValue::Set(like.uri.clone()),
        song_uri: ActiveValue::Set(like.song_uri.clone()),
        title: ActiveValue::Set(song.title.clone()),
        artist: ActiveValue::Set(song.artist.clone()),
        album: ActiveValue::Set(song.album.clone()),
        album_artist: ActiveValue::Set(song.album_artist.clone()),
        created_at: ActiveValue::Set(like.created_at.clone()),
        // Left as-is; the matching pass fills it in.
        track_id: ActiveValue::NotSet,
    }
    .save(conn)
    .await?;
    Ok(())
}

/// Import every like in the repo into `rocksky_like`. Returns how many landed.
///
/// A no-op when the repo was downloaded recently enough that `rocksky_like`
/// already holds it — see [`crate::repo_sync`].
pub async fn import(conn: &DatabaseConnection) -> Result<usize, Error> {
    let Some(did) = atproto::resolve_did().await else {
        return Ok(0);
    };
    let Some(car) = repo_sync::repo_car(conn, &did).await? else {
        tracing::info!("keeping the liked songs already imported from the repo");
        return Ok(0);
    };
    let likes = extract(&did, &car).await?;
    let total = likes.len();
    let mut imported = 0;
    for (i, like) in likes.iter().enumerate() {
        if like.song.is_none() {
            continue;
        }
        upsert(conn, like).await?;
        imported += 1;
        if (i + 1) % 25 == 0 || i + 1 == total {
            tracing::info!(progress = format!("{}/{total}", i + 1), "importing likes");
        }
    }
    tracing::info!(imported, total, "liked songs imported from the repo");
    Ok(imported)
}

// ── Matching ────────────────────────────────────────────────────────────────

/// Find the local track for a liked song: title + artist + album, all compared
/// case-insensitively. Album is matched through `track.album_id` so a
/// same-titled track from a different album is not claimed by mistake; a like
/// whose album is blank falls back to title + artist.
async fn find_track(
    conn: &DatabaseConnection,
    title: &str,
    artist: &str,
    album: &str,
) -> Result<Option<String>, Error> {
    let backend = conn.get_database_backend();
    let statement = if album.is_empty() {
        Statement::from_sql_and_values(
            backend,
            "SELECT id FROM track WHERE LOWER(title) = LOWER(?) AND LOWER(artist) = LOWER(?) \
             LIMIT 1",
            [title.into(), artist.into()],
        )
    } else {
        Statement::from_sql_and_values(
            backend,
            "SELECT track.id AS id FROM track \
             JOIN album ON album.id = track.album_id \
             WHERE LOWER(track.title) = LOWER(?) AND LOWER(track.artist) = LOWER(?) \
               AND LOWER(album.title) = LOWER(?) LIMIT 1",
            [title.into(), artist.into(), album.into()],
        )
    };
    let row = conn.query_one(statement).await?;
    Ok(match row {
        Some(row) => Some(row.try_get::<String>("", "id")?),
        None => None,
    })
}

/// Match every still-unmatched like against the library, stamping `track.aturi`
/// (plus the album's and artist's) on each hit.
///
/// Safe to re-run: it only looks at likes with no `track_id` yet, which is what
/// makes a rescan pick up songs that were liked before the file existed.
pub async fn match_library(conn: &DatabaseConnection) -> Result<usize, Error> {
    let pending = rocksky_like::Entity::find()
        .filter(rocksky_like::Column::TrackId.is_null())
        .all(conn)
        .await?;
    if pending.is_empty() {
        return Ok(0);
    }
    tracing::info!(
        pending = pending.len(),
        "matching likes against the library"
    );

    let mut matched = 0;
    for like in pending {
        let Some(track_id) = find_track(conn, &like.title, &like.artist, &like.album).await? else {
            continue;
        };
        let song_uri = like.song_uri.clone();
        let uri = like.uri.clone();

        let mut model: rocksky_like::ActiveModel = like.into();
        model.track_id = ActiveValue::Set(Some(track_id.clone()));
        model.update(conn).await?;

        stamp_track(conn, &track_id, &song_uri).await?;
        matched += 1;
        tracing::debug!(%uri, %track_id, "like matched to a local track");
    }
    tracing::info!(matched, "likes linked to local tracks");
    Ok(matched)
}

/// Write the song uri onto the track, and derive the album's and artist's uris
/// from the same repo so those rows are linked too.
async fn stamp_track(
    conn: &DatabaseConnection,
    track_id: &str,
    song_uri: &str,
) -> Result<(), Error> {
    let Some(found) = track::Entity::find_by_id(track_id.to_owned())
        .one(conn)
        .await?
    else {
        return Ok(());
    };
    let (album_id, artist_id) = (found.album_id.clone(), found.artist_id.clone());

    let mut model: track::ActiveModel = found.into();
    model.aturi = ActiveValue::Set(Some(song_uri.to_owned()));
    model.update(conn).await?;

    // The album and artist records live in the same repo as the song; their
    // rkeys differ, so only the repo is known here. Record the collection uri
    // of the hosting repo so the rows point at the right account, and leave a
    // more precise uri to whoever resolves the rkey.
    let Some((did, _, _)) = atproto::split_at_uri(song_uri) else {
        return Ok(());
    };
    if let Some(album_id) = album_id {
        if let Some(found) = album::Entity::find_by_id(album_id).one(conn).await? {
            if found.aturi.is_none() {
                let mut model: album::ActiveModel = found.into();
                model.aturi = ActiveValue::Set(Some(format!("at://{did}/app.rocksky.album")));
                model.update(conn).await?;
            }
        }
    }
    if let Some(artist_id) = artist_id {
        if let Some(found) = artist::Entity::find_by_id(artist_id).one(conn).await? {
            if found.aturi.is_none() {
                let mut model: artist::ActiveModel = found.into();
                model.aturi = ActiveValue::Set(Some(format!("at://{did}/app.rocksky.artist")));
                model.update(conn).await?;
            }
        }
    }
    Ok(())
}

/// Local track ids of every like that resolved to a file in the library — what
/// a client needs to show the account's likes without knowing about atproto.
pub async fn matched_track_ids(conn: &DatabaseConnection) -> Result<Vec<String>, Error> {
    Ok(rocksky_like::Entity::find()
        .filter(rocksky_like::Column::TrackId.is_not_null())
        .all(conn)
        .await?
        .into_iter()
        .filter_map(|like| like.track_id)
        .collect())
}

// ── Live sync ───────────────────────────────────────────────────────────────

async fn apply_commit(conn: &DatabaseConnection, commit: atproto::JetstreamCommit, did: &str) {
    let uri = format!("at://{did}/{}/{}", commit.collection, commit.rkey);
    match commit.operation.as_str() {
        "create" | "update" => {
            let Some(record) = commit
                .record
                .and_then(|value| serde_json::from_value::<LikeRecord>(value).ok())
            else {
                return;
            };
            let song = match atproto::get_record::<SongRecord>(&record.subject.uri).await {
                Ok(song) => song,
                Err(e) => {
                    tracing::warn!(uri = %record.subject.uri, "could not resolve a liked song: {e}");
                    return;
                }
            };
            let like = Like {
                uri: uri.clone(),
                song_uri: record.subject.uri,
                created_at: record.created_at,
                song: Some(song),
            };
            if let Err(e) = upsert(conn, &like).await {
                tracing::warn!(%uri, "could not store a like: {e}");
                return;
            }
            tracing::info!(%uri, "song liked on Rocksky");
            if let Err(e) = match_library(conn).await {
                tracing::warn!("could not match the new like: {e}");
            }
        }
        "delete" => {
            // Clear the track link before dropping the row, so an unliked track
            // does not keep pointing at a record that no longer exists.
            if let Ok(Some(like)) = rocksky_like::Entity::find_by_id(uri.clone())
                .one(conn)
                .await
            {
                if let Some(track_id) = like.track_id.clone() {
                    if let Ok(Some(found)) = track::Entity::find_by_id(track_id).one(conn).await {
                        let mut model: track::ActiveModel = found.into();
                        model.aturi = ActiveValue::Set(None);
                        let _ = model.update(conn).await;
                    }
                }
                let _ = rocksky_like::Entity::delete_by_id(uri.clone())
                    .exec(conn)
                    .await;
                tracing::info!(%uri, "song unliked on Rocksky");
            }
        }
        _ => {}
    }
}

/// Import and match the account's likes, then follow the repo for changes until
/// the process exits. Returns immediately when no identity can be resolved.
pub async fn sync(conn: DatabaseConnection) {
    let Some(did) = atproto::resolve_did().await else {
        tracing::info!("Rocksky like sync off: no atproto identity");
        return;
    };
    if let Err(e) = import(&conn).await {
        tracing::warn!("could not import Rocksky likes: {e}");
    }
    if let Err(e) = match_library(&conn).await {
        tracing::warn!("could not match Rocksky likes against the library: {e}");
    }
    let applier_conn = conn.clone();
    let applier_did = did.clone();
    atproto::subscribe(&did, &[LIKE_COLLECTION], |commit| {
        let conn = applier_conn.clone();
        let did = applier_did.clone();
        async move { apply_commit(&conn, commit, &did).await }
    })
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The like's uri is reconstructed from the MST path inside the CAR, so a
    /// wrong path would silently produce wrong primary keys. Checks the CAR
    /// uris against the ones `listRecords` reports for the live account.
    /// Ignored by default: needs an atproto identity and the network.
    #[tokio::test]
    #[ignore]
    async fn car_like_uris_match_list_records() {
        let did = atproto::resolve_did()
            .await
            .expect("a rocksky token or atproto credentials are required");
        let car = atproto::get_repo_car(&did).await.unwrap();
        let from_car =
            atproto::records_from_car::<LikeRecord>(&car, &did, LIKE_COLLECTION).unwrap();
        let from_list: Vec<(String, LikeRecord)> =
            atproto::list_records(&did, LIKE_COLLECTION).await.unwrap();

        let mut car_uris: Vec<&str> = from_car.iter().map(|(uri, _)| uri.as_str()).collect();
        let mut list_uris: Vec<&str> = from_list.iter().map(|(uri, _)| uri.as_str()).collect();
        car_uris.sort_unstable();
        list_uris.sort_unstable();
        assert_eq!(
            car_uris, list_uris,
            "CAR MST paths disagree with listRecords"
        );
        assert!(!car_uris.is_empty(), "the account has no likes to check");
    }
}
