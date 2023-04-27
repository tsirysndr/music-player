use std::sync::Arc;

use async_graphql::{futures_util::FutureExt, *};
use music_player_addons::CurrentSourceDevice;
use music_player_entity::{album as album_entity, artist as artist_entity, track as track_entity};
use music_player_scanner::scan_directory;
use music_player_storage::{
    repo::{album::AlbumRepository, artist::ArtistRepository, track::TrackRepository},
    searcher::Searcher,
    Database,
};
use music_player_types::types::{RemoteCoverUrl, RemoteTrackUrl};
use sea_orm::{ActiveModelTrait, ActiveValue};
use tokio::sync::Mutex;

use super::objects::{album::Album, artist::Artist, search_result::SearchResult, track::Track};

#[derive(Default)]
pub struct LibraryQuery;

#[Object]
impl LibraryQuery {
    async fn tracks(
        &self,
        ctx: &Context<'_>,
        filter: Option<String>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<Track>, Error> {
        let current_device = ctx.data::<Arc<Mutex<CurrentSourceDevice>>>().unwrap();
        let mut device = current_device.lock().await;

        if device.client.is_some() {
            let source = device.client.as_mut().unwrap();
            let result = source
                .tracks(filter, offset.unwrap_or(0), limit.unwrap_or(100))
                .await?;

            let base_url = device
                .source_device
                .as_ref()
                .unwrap()
                .base_url
                .as_ref()
                .unwrap();

            let tracks: Vec<Track> = result.into_iter().map(Into::into).collect();

            return Ok(tracks
                .into_iter()
                .map(|track| {
                    track
                        .with_remote_track_url(base_url.as_str())
                        .with_remote_cover_url(base_url.as_str())
                })
                .collect());
        }

        let db = ctx.data::<Database>().unwrap();
        let results = TrackRepository::new(db.get_connection())
            .find_all(
                filter,
                Some(offset.unwrap_or(0) as u64),
                limit.unwrap_or(100) as u64,
            )
            .await?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn artists(
        &self,
        ctx: &Context<'_>,
        filter: Option<String>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<Artist>, Error> {
        let current_device = ctx.data::<Arc<Mutex<CurrentSourceDevice>>>().unwrap();
        let mut device = current_device.lock().await;

        if device.client.is_some() {
            let source = device.client.as_mut().unwrap();
            let artists = source
                .artists(filter, offset.unwrap_or(0), limit.unwrap_or(100))
                .await?;

            let base_url = device
                .source_device
                .as_ref()
                .unwrap()
                .base_url
                .as_ref()
                .unwrap();

            return Ok(artists
                .into_iter()
                .map(|artist| {
                    artist
                        .with_remote_cover_url(base_url.as_str())
                        .with_remote_track_url(base_url.as_str())
                })
                .map(Into::into)
                .collect());
        }

        let db = ctx.data::<Database>().unwrap();

        let results = ArtistRepository::new(db.get_connection())
            .find_all(filter, offset.map(|x| x as u64), limit.map(|x| x as u64))
            .await?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn albums(
        &self,
        ctx: &Context<'_>,
        filter: Option<String>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<Album>, Error> {
        let current_device = ctx.data::<Arc<Mutex<CurrentSourceDevice>>>().unwrap();
        let mut device = current_device.lock().await;

        if device.client.is_some() {
            let source = device.client.as_mut().unwrap();
            let albums = source
                .albums(filter, offset.unwrap_or(0), limit.unwrap_or(100))
                .await?;

            let base_url = device
                .source_device
                .as_ref()
                .unwrap()
                .base_url
                .as_ref()
                .unwrap();

            let result: Vec<Album> = albums.into_iter().map(Into::into).collect();

            return Ok(result
                .into_iter()
                .map(|album| {
                    album
                        .with_remote_cover_url(base_url.as_str())
                        .with_remote_track_url(base_url.as_str())
                })
                .collect());
        }

        let db = ctx.data::<Database>().unwrap();

        let results = AlbumRepository::new(db.get_connection())
            .find_all(filter, offset.map(|x| x as u64), limit.map(|x| x as u64))
            .await?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn track(&self, ctx: &Context<'_>, id: ID) -> Result<Track, Error> {
        let current_device = ctx.data::<Arc<Mutex<CurrentSourceDevice>>>().unwrap();
        let mut device = current_device.lock().await;
        let id = id.to_string();

        if device.client.is_some() {
            let source = device.client.as_mut().unwrap();
            let track = source.track(&id).await?;

            let base_url = device
                .source_device
                .as_ref()
                .unwrap()
                .base_url
                .as_ref()
                .unwrap();

            return Ok(Track::from(track)
                .with_remote_track_url(base_url.as_str())
                .with_remote_cover_url(base_url.as_str()));
        }

        let db = ctx.data::<Database>().unwrap();

        let track = TrackRepository::new(db.get_connection()).find(&id).await?;

        Ok(track.into())
    }

    async fn artist(&self, ctx: &Context<'_>, id: ID) -> Result<Artist, Error> {
        let current_device = ctx.data::<Arc<Mutex<CurrentSourceDevice>>>().unwrap();
        let mut device = current_device.lock().await;
        let id = id.to_string();

        if device.client.is_some() {
            let source = device.client.as_mut().unwrap();
            let artist = source.artist(&id).await?;

            let base_url = device
                .source_device
                .as_ref()
                .unwrap()
                .base_url
                .as_ref()
                .unwrap();

            return Ok(artist
                .with_remote_track_url(base_url.as_str())
                .with_remote_cover_url(base_url.as_str())
                .into());
        }

        let db = ctx.data::<Database>().unwrap();

        let artist = ArtistRepository::new(db.get_connection()).find(&id).await?;

        Ok(artist.into())
    }

    async fn album(&self, ctx: &Context<'_>, id: ID) -> Result<Album, Error> {
        let current_device = ctx.data::<Arc<Mutex<CurrentSourceDevice>>>().unwrap();

        let id = id.to_string();

        let mut device = current_device.lock().await;
        if device.client.is_some() {
            let source = device.client.as_mut().unwrap();
            let album = source.album(&id).await?;

            let base_url = device
                .source_device
                .as_ref()
                .unwrap()
                .base_url
                .as_ref()
                .unwrap();

            return Ok(Album::from(album)
                .with_remote_cover_url(base_url.as_str())
                .with_remote_track_url(base_url.as_str()));
        }

        let db = ctx.data::<Database>().unwrap();

        let album = AlbumRepository::new(db.get_connection()).find(&id).await?;

        Ok(album.into())
    }

    async fn search(&self, ctx: &Context<'_>, keyword: String) -> Result<SearchResult, Error> {
        let searcher = ctx.data::<Arc<Mutex<Searcher>>>().unwrap();
        let indexer = searcher.lock().await;
        let artists = indexer.artist.search(&keyword)?;
        let albums = indexer.album.search(&keyword)?;
        let tracks = indexer.track.search(&keyword)?;
        Ok(SearchResult {
            artists: artists.into_iter().map(|x| Into::into(x)).collect(),
            tracks: tracks.into_iter().map(|x| Into::into(x)).collect(),
            albums: albums.into_iter().map(|x| Into::into(x)).collect(),
        })
    }
}

#[derive(Default)]
pub struct LibraryMutation;

#[Object]
impl LibraryMutation {
    async fn scan(&self, _ctx: &Context<'_>) -> Result<bool, Error> {
        scan_directory(
            move |song, db| {
                async move {
                    let id = format!("{:x}", md5::compute(song.artist.to_string()));
                    let item = artist_entity::ActiveModel {
                        id: ActiveValue::set(id),
                        name: ActiveValue::Set(song.artist.clone()),
                    };
                    match item.insert(db.get_connection()).await {
                        Ok(_) => (),
                        Err(_) => (),
                    }

                    let id = format!(
                        "{:x}",
                        md5::compute(format!("{}{}", song.album, song.artist))
                    );
                    let item = album_entity::ActiveModel {
                        id: ActiveValue::set(id),
                        artist: ActiveValue::set(song.artist.clone()),
                        title: ActiveValue::Set(song.album.clone()),
                        artist_id: ActiveValue::Set(Some(format!(
                            "{:x}",
                            md5::compute(song.artist.to_string())
                        ))),
                        year: ActiveValue::Set(song.year),
                        cover: ActiveValue::Set(song.cover.clone()),
                    };
                    match item.insert(db.get_connection()).await {
                        Ok(_) => (),
                        Err(_) => (),
                    }
                    let id = format!("{:x}", md5::compute(song.uri.as_ref().unwrap()));
                    let item = track_entity::ActiveModel {
                        id: ActiveValue::set(id),
                        title: ActiveValue::Set(song.title.clone()),
                        artist: ActiveValue::set(song.artist.clone()),
                        genre: ActiveValue::Set(song.genre.clone()),
                        year: ActiveValue::Set(song.year),
                        track: ActiveValue::Set(song.track),
                        bitrate: ActiveValue::Set(song.bitrate),
                        sample_rate: ActiveValue::Set(song.sample_rate),
                        bit_depth: ActiveValue::Set(song.bit_depth),
                        channels: ActiveValue::Set(song.channels),
                        duration: ActiveValue::Set(Some(song.duration.as_secs_f32())),
                        uri: ActiveValue::Set(song.uri.clone().unwrap_or_default()),
                        album_id: ActiveValue::Set(Some(format!(
                            "{:x}",
                            md5::compute(format!("{}{}", song.album, song.artist))
                        ))),
                        artist_id: ActiveValue::Set(Some(format!(
                            "{:x}",
                            md5::compute(song.artist.to_string())
                        ))),
                    };

                    match item.insert(db.get_connection()).await {
                        Ok(_) => (),
                        Err(_) => (),
                    }
                }
                .boxed()
            },
            &Database::new().await,
            &Searcher::new(),
        )
        .await?;

        Ok(false)
    }
}
