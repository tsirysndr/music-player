#[cfg(test)]
mod tests;

use anyhow::Error;
use futures::stream::{self, StreamExt};
use music_player_entity::{album, artist, artist_tracks, playlist_tracks, track};
use music_player_storage::Database;
use music_player_types::types::{album_id, Song};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DbBackend, EntityTrait,
    QueryFilter, Statement, TransactionTrait,
};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
};
use tracing::{debug, error, info};

use music_player_settings::{get_application_directory, read_settings, Settings};
use rockbox_metadata::{AlbumArt, AlbumArtKind, Metadata};
use walkdir::WalkDir;

/// Walk the music directory and parse every audio file's metadata in
/// parallel (one blocking task per file, bounded by the CPU count). Album
/// covers are extracted as part of the same pass.
async fn parse_music_library(enable_log: bool) -> Result<Vec<Song>, Error> {
    let config = read_settings().unwrap();
    let settings = config.try_deserialize::<Settings>().unwrap();

    let paths: Vec<String> = WalkDir::new(&settings.music_directory)
        .follow_links(true)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .map(|entry| format!("{}", entry.path().display()))
        // rockbox-metadata handles 40+ formats; let it decide whether the
        // file is really parsable and only pre-filter on the mime family.
        .filter(|path| {
            mime_guess::from_path(path).first_or_octet_stream().type_() == mime_guess::mime::AUDIO
        })
        .collect();

    let parallelism = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    debug!(files = paths.len(), parallelism, "scanning music library");

    let songs: Vec<Song> = stream::iter(paths)
        .map(|path| {
            tokio::task::spawn_blocking(move || match rockbox_metadata::read(&path) {
                Ok(meta) => {
                    let mut song: Song = (&meta).into();
                    song.uri = Some(path.clone());

                    song.cover =
                        extract_and_save_album_cover(&path, &meta, &song.album, &song.album_artist);
                    if enable_log {
                        // user-facing progress output for the CLI `scan` command
                        println!("{}", path);
                    }
                    Some(song)
                }
                Err(e) => {
                    error!(path, "failed to parse: {}", e);
                    None
                }
            })
        })
        .buffer_unordered(parallelism)
        .filter_map(|joined| async move { joined.ok().flatten() })
        .collect()
        .await;
    Ok(songs)
}

/// Upsert the parsed songs in one transaction — a single commit instead of
/// four autocommitted inserts per track, which is what makes scans of
/// thousands of files fast. Existing rows keep their primary keys, so
/// re-scanning only adds what is new.
async fn save_songs(db: &Database, songs: &[Song]) -> Result<(), Error> {
    let txn = db.get_connection().begin().await?;
    for song in songs {
        let item: artist::ActiveModel = song.try_into().unwrap();
        item.insert(&txn).await.ok();

        let item: album::ActiveModel = song.try_into().unwrap();
        let id = album_id(&song.album, &song.album_artist);
        if album::Entity::find_by_id(id).one(&txn).await?.is_none() {
            item.insert(&txn).await?;
        }

        let mut item: track::ActiveModel = song.try_into().unwrap();
        let id = format!(
            "{:x}",
            md5::compute(song.uri.as_deref().unwrap_or_default())
        );
        if track::Entity::find_by_id(id).one(&txn).await?.is_some() {
            // Re-scanning must repair metadata and album associations, not
            // merely ignore an existing primary key.
            //
            // `created_at` is the exception: it records when the track was
            // first seen, so restamping it here would make every "added in the
            // last week" smart playlist list the whole library after a rescan.
            item.created_at = ActiveValue::NotSet;
            item.update(&txn).await?;
        } else {
            item.insert(&txn).await?;
        }

        let item: artist_tracks::ActiveModel = song.try_into().unwrap();
        item.insert(&txn).await.ok();
    }
    txn.commit().await?;
    Ok(())
}

/// Read the embedded album art out of the audio file and save it under
/// `<app_dir>/covers/<album-id>.{jpg,png}`, returning the file name.
///
/// Art that needs ID3 de-unsynchronization or base64 decoding (rare —
/// mostly Vorbis `METADATA_BLOCK_PICTURE`) is skipped.
fn extract_and_save_album_cover(
    path: &str,
    meta: &Metadata,
    album: &str,
    album_artist: &str,
) -> Option<String> {
    let art: AlbumArt = meta.album_art?;
    if art.id3_unsync || art.vorbis_base64 || art.size == 0 {
        return None;
    }

    let mut file = File::open(path).ok()?;
    file.seek(SeekFrom::Start(art.offset)).ok()?;
    let mut data = vec![0u8; art.size as usize];
    file.read_exact(&mut data).ok()?;

    let extension = match art.kind {
        AlbumArtKind::Jpeg => "jpg",
        AlbumArtKind::Png => "png",
        // Sniff the magic bytes when the container did not say.
        AlbumArtKind::Unknown if data.starts_with(&[0xFF, 0xD8]) => "jpg",
        AlbumArtKind::Unknown if data.starts_with(b"\x89PNG") => "png",
        _ => return None,
    };

    let covers_path = format!("{}/covers", get_application_directory());
    let album = album_id(album, album_artist);
    let filename = format!("{}/{}.{}", covers_path, album, extension);
    let mut file = File::create(filename).ok()?;
    file.write_all(&data).ok()?;
    Some(format!("{}.{}", album, extension))
}

pub async fn scan_music_library(enable_log: bool, db: Database) -> Result<Vec<Song>, Error> {
    let started = std::time::Instant::now();
    let songs = parse_music_library(enable_log).await?;
    save_songs(&db, &songs).await?;
    info!(
        tracks = songs.len(),
        duration = %format_args!("{:.2?}", started.elapsed()),
        "music library indexed"
    );
    Ok(songs)
}

/// Bring the database in sync with the disk: drop tracks whose files no
/// longer exist (plus any albums/artists left orphaned), then scan for new
/// files. This is what "refresh library" means for the CLI `scan` command,
/// the GraphQL/gRPC scan calls and the periodic background refresh.
pub async fn refresh_music_library(enable_log: bool, db: Database) -> Result<Vec<Song>, Error> {
    prune_missing_tracks(&db).await?;
    let songs = scan_music_library(enable_log, db.clone()).await?;
    // A metadata refresh can move tracks to newly identified albums.
    // Remove legacy title-only album rows after those updates have landed.
    prune_orphaned_library_rows(&db).await?;
    // Re-sync the Typesense collections when that backend is configured
    // (no-op on FTS5 — its triggers already track the writes above). A
    // failed sync must not fail the scan; search just falls back to FTS5.
    let searcher = music_player_storage::searcher::Searcher::new(db.get_connection().clone());
    if let Err(e) = searcher.reindex().await {
        tracing::warn!("typesense reindex failed: {e}");
    }
    // Artist pictures come from the Rocksky API in batch; network problems
    // must not fail the scan either.
    if let Err(e) = update_artist_pictures(&db).await {
        tracing::warn!("artist picture update failed: {e}");
    }
    // Every stored filter now has different answers — tracks arrived, tracks
    // left — so the smart playlists are stale until they are re-run.
    if let Err(e) = music_player_storage::smart_playlist::regenerate_all(db.get_connection()).await
    {
        tracing::warn!("smart playlist refresh failed: {e}");
    }
    Ok(songs)
}

/// Fills `artist.picture` for every artist that has none yet, in batches of
/// names against Rocksky's public `app.rocksky.artist.getArtists` endpoint
/// (matched by name; misses stay NULL and are retried on the next scan).
pub async fn update_artist_pictures(db: &Database) -> Result<(), Error> {
    #[derive(serde::Deserialize)]
    struct RockskyArtist {
        name: String,
        picture: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct RockskyArtists {
        artists: Vec<RockskyArtist>,
    }

    const BATCH: usize = 50;
    let api_url =
        std::env::var("ROCKSKY_API_URL").unwrap_or_else(|_| "https://api.rocksky.app".to_string());
    let conn = db.get_connection();
    let missing: Vec<artist::Model> = artist::Entity::find()
        .filter(artist::Column::Picture.is_null())
        .all(conn)
        .await?;
    if missing.is_empty() {
        return Ok(());
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;
    let mut updated = 0usize;
    for chunk in missing.chunks(BATCH) {
        let names: Vec<&str> = chunk.iter().map(|a| a.name.as_str()).collect();
        let response = client
            .get(format!("{}/xrpc/app.rocksky.artist.getArtists", api_url))
            .query(&[("names", names.join(","))])
            .send()
            .await?
            .error_for_status()?;
        let found: RockskyArtists = response.json().await?;
        let by_name: std::collections::HashMap<String, Option<String>> = found
            .artists
            .into_iter()
            .map(|a| (a.name.to_lowercase(), a.picture))
            .collect();
        for local in chunk {
            let Some(Some(picture)) = by_name.get(&local.name.to_lowercase()) else {
                continue;
            };
            let mut active: artist::ActiveModel = local.clone().into();
            active.picture = ActiveValue::Set(Some(picture.clone()));
            active.update(conn).await?;
            updated += 1;
        }
    }
    info!(
        "artist pictures: {updated}/{} filled from Rocksky",
        missing.len()
    );
    Ok(())
}

/// Remove every track whose local file is gone, then any album or artist that
/// no longer has tracks. Remote (http/https) uris are left alone. The FTS5
/// triggers keep the search index in sync with the deletes.
async fn prune_missing_tracks(db: &Database) -> Result<(), Error> {
    let conn = db.get_connection();
    let tracks = track::Entity::find().all(conn).await?;
    for t in tracks {
        if !t.uri.starts_with('/') || std::path::Path::new(&t.uri).exists() {
            continue;
        }
        artist_tracks::Entity::delete_many()
            .filter(artist_tracks::Column::TrackId.eq(t.id.clone()))
            .exec(conn)
            .await?;
        playlist_tracks::Entity::delete_many()
            .filter(playlist_tracks::Column::TrackId.eq(t.id.clone()))
            .exec(conn)
            .await?;
        track::Entity::delete_by_id(t.id).exec(conn).await?;
    }
    prune_orphaned_library_rows(db).await
}

async fn prune_orphaned_library_rows(db: &Database) -> Result<(), Error> {
    let conn = db.get_connection();
    conn.execute(Statement::from_string(
        DbBackend::Sqlite,
        "DELETE FROM album WHERE id NOT IN (SELECT album_id FROM track WHERE album_id IS NOT NULL)"
            .to_owned(),
    ))
    .await?;
    conn.execute(Statement::from_string(
        DbBackend::Sqlite,
        "DELETE FROM artist WHERE id NOT IN (SELECT artist_id FROM track WHERE artist_id IS NOT NULL) AND id NOT IN (SELECT artist_id FROM artist_track)"
            .to_owned(),
    ))
    .await?;
    Ok(())
}
