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
        let item: artist::ActiveModel = song.into();
        item.insert(&txn).await.ok();

        let item: album::ActiveModel = song.into();
        let id = album_id(&song.album, &song.album_artist);
        if album::Entity::find_by_id(id).one(&txn).await?.is_none() {
            item.insert(&txn).await?;
        }

        let mut item: track::ActiveModel = song.into();
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

        let item: artist_tracks::ActiveModel = song.into();
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
    // Genres from the files' own tags. Cheap and local, so it runs whether or
    // not the network enrichment below succeeds.
    if let Err(e) = index_track_genres(&db).await {
        tracing::warn!("genre indexing failed: {e}");
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

    // Key and tempo are deliberately *not* run from here.
    //
    // They decode every new track in full, which is minutes of work for a
    // library of any size, and the right way to do that differs by caller: a
    // command that exits must await it, a daemon that keeps running should
    // detach it. A library function that spawned work as a side effect could
    // serve neither, and did both at once when the two were combined.

    Ok(songs)
}

/// How many tracks are fetched from the database at a time.
///
/// A paging size, not a limit on the work: the pass keeps asking for the next
/// batch until nothing is left, so a whole library gets analysed however large
/// it is. Batched only so a hundred thousand rows are not held in memory at
/// once to answer a question about one track at a time.
const ANALYSIS_PAGE: u64 = 500;

/// Fill in key and tempo for tracks that have none. Returns how many it did.
///
/// Skips anything already analysed, so running this over a library that has
/// been through it costs one query and nothing else.
///
/// Awaitable rather than only spawnable, because the two callers need opposite
/// things: `music-player scan` is a command that exits when it returns, so a
/// detached task there would be killed before it decoded anything — the reason
/// a scan left every key null. The daemon, which keeps running, spawns it.
pub async fn analyse_missing_key_and_bpm(db: &Database) -> usize {
    let conn = db.get_connection();

    // Anything already analysed but not yet copied onto its track row. One
    // statement, no decoding — and it has to come first, because the pass below
    // only visits tracks that have never been analysed at all.
    let filled = music_player_storage::track_analysis::backfill_key_and_bpm(conn).await;
    if filled > 0 {
        info!("key and tempo: filled in {filled} tracks from stored analysis");
    }

    let total = music_player_storage::track_analysis::unanalysed_local_count(conn).await as usize;
    if total == 0 {
        return 0;
    }
    info!(tracks = total, "analysing key and tempo");

    let mut done = 0usize;
    // Tracks that could not be analysed at all — a missing file, a codec we do
    // not read. A success writes a row and the track leaves the query; a
    // failure does not, so it would sit at the head of every page and be
    // retried for ever. Counting them is what the offset skips past.
    let mut failed = 0usize;

    loop {
        let pending = music_player_storage::track_analysis::unanalysed_local_tracks(
            conn,
            failed as u64,
            ANALYSIS_PAGE,
        )
        .await;
        if pending.is_empty() {
            break;
        }

        for track in &pending {
            // `ensure` serialises on its own semaphore, so this stays one
            // decode at a time however many callers there are — it must not
            // compete with the audio that is playing.
            //
            // Logged per track, and at info: this is minutes of work with
            // nothing else to show for it, and a pass that printed only a total
            // at the end is indistinguishable from one that has hung.
            let index = done + failed + 1;
            match music_player_storage::track_analysis::ensure(conn, "", track).await {
                Ok(analysis) => {
                    done += 1;
                    info!(
                        "[{index}/{total}] {} — {}  {}  {}",
                        track.artist,
                        track.title,
                        // A track can decode fine and still have no clear key
                        // or tempo; saying so beats printing a zero.
                        analysis.key.as_deref().unwrap_or("--"),
                        analysis
                            .bpm
                            .map(|bpm| format!("{bpm:.0} BPM"))
                            .unwrap_or_else(|| "--- BPM".to_string()),
                    );
                }
                Err(cause) => {
                    failed += 1;
                    info!(
                        "[{index}/{total}] {} — {}  could not analyse: {cause}",
                        track.artist, track.title
                    );
                }
            }
        }
    }

    info!("key and tempo: {done}/{total} tracks analysed");
    done
}

/// The same pass, detached — for the daemon, which outlives it.
pub fn spawn_key_and_bpm_analysis(db: Database) {
    tokio::spawn(async move {
        analyse_missing_key_and_bpm(&db).await;
    });
}

/// Index every track's genre tag into the `genre` and `track_genres` tables.
///
/// The tag is free text holding one or more genres, spelled however the
/// tagger felt — so it is split, normalised for identity, and stored under a
/// derived id. Idempotent: the same track scanned twice links the same rows.
pub async fn index_track_genres(db: &Database) -> Result<(), Error> {
    use music_player_entity::{genre, track_genres};

    let conn = db.get_connection();
    let tracks = track::Entity::find().all(conn).await?;
    let mut linked = 0usize;

    for t in &tracks {
        for name in genre::split_tag(&t.genre) {
            let genre_id = genre::id_for(&name);

            // `ON CONFLICT DO NOTHING`: two tracks sharing a genre race here
            // on a rescan, and the second one losing is the correct outcome.
            let row = genre::ActiveModel {
                id: ActiveValue::Set(genre_id.clone()),
                name: ActiveValue::Set(name.clone()),
            };
            let _ = genre::Entity::insert(row)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(genre::Column::Id)
                        .do_nothing()
                        .to_owned(),
                )
                .exec(conn)
                .await;

            let link = track_genres::ActiveModel {
                id: ActiveValue::Set(track_genres::id_for(&t.id, &genre_id)),
                track_id: ActiveValue::Set(t.id.clone()),
                genre_id: ActiveValue::Set(genre_id),
            };
            if track_genres::Entity::insert(link)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(track_genres::Column::Id)
                        .do_nothing()
                        .to_owned(),
                )
                .exec(conn)
                .await
                .is_ok()
            {
                linked += 1;
            }
        }
    }

    info!(
        "genres: {linked} track links from tags across {} tracks",
        tracks.len()
    );
    Ok(())
}

/// Fills `artist.picture` for every artist that has none yet, and links every
/// artist to the genres Rocksky reports, in batches of names against its
/// public `app.rocksky.artist.getArtists` endpoint (matched by name; misses
/// stay NULL and are retried on the next scan).
pub async fn update_artist_pictures(db: &Database) -> Result<(), Error> {
    #[derive(serde::Deserialize)]
    struct RockskyArtist {
        name: String,
        picture: Option<String>,
        /// The genres Rocksky has for this artist. Much better coverage than
        /// the files' own tags, which most releases ship empty.
        #[serde(default)]
        genres: Vec<String>,
    }
    #[derive(serde::Deserialize)]
    struct RockskyArtists {
        artists: Vec<RockskyArtist>,
    }

    const BATCH: usize = 50;
    let api_url =
        std::env::var("ROCKSKY_API_URL").unwrap_or_else(|_| "https://api.rocksky.app".to_string());
    let conn = db.get_connection();
    // Every artist, not only those missing a picture: an artist can have a
    // picture already and still have no genres linked, and the genres are the
    // point of the second half of this call.
    let missing: Vec<artist::Model> = artist::Entity::find().all(conn).await?;
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
        let by_name: std::collections::HashMap<String, RockskyArtist> = found
            .artists
            .into_iter()
            .map(|a| (a.name.to_lowercase(), a))
            .collect();
        for local in chunk {
            let Some(remote) = by_name.get(&local.name.to_lowercase()) else {
                continue;
            };
            if let (Some(picture), None) = (&remote.picture, &local.picture) {
                let mut active: artist::ActiveModel = local.clone().into();
                active.picture = ActiveValue::Set(Some(picture.clone()));
                active.update(conn).await?;
                updated += 1;
            }
            link_artist_genres(conn, &local.id, &remote.genres).await?;
        }
    }
    info!(
        "artist pictures: {updated}/{} filled from Rocksky",
        missing.len()
    );
    Ok(())
}

/// Link one artist to the genres Rocksky reports, creating any that are new.
///
/// Separate from the track-tag pass because the two sources answer different
/// questions — this one says what an *artist* is, which is why a compilation's
/// tracks can end up in a genre none of their tags mention.
async fn link_artist_genres(
    conn: &sea_orm::DatabaseConnection,
    artist_id: &str,
    genres: &[String],
) -> Result<(), Error> {
    use music_player_entity::{artist_genres, genre};

    for name in genres.iter().filter(|name| !name.trim().is_empty()) {
        let genre_id = genre::id_for(name);
        let row = genre::ActiveModel {
            id: ActiveValue::Set(genre_id.clone()),
            name: ActiveValue::Set(name.trim().to_string()),
        };
        let _ = genre::Entity::insert(row)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(genre::Column::Id)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(conn)
            .await;

        let link = artist_genres::ActiveModel {
            id: ActiveValue::Set(artist_genres::id_for(artist_id, &genre_id)),
            artist_id: ActiveValue::Set(artist_id.to_string()),
            genre_id: ActiveValue::Set(genre_id),
        };
        let _ = artist_genres::Entity::insert(link)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(artist_genres::Column::Id)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(conn)
            .await;
    }
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
