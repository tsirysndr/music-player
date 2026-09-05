use std::sync::Arc;

use async_graphql::{futures_util::FutureExt, *};
use music_player_addons::CurrentSourceDevice;
use music_player_storage::{
    repo::{album::AlbumRepository, artist::ArtistRepository, track::TrackRepository},
    searcher::Searcher,
    Database,
};
use music_player_types::types::{RemoteCoverUrl, RemoteTrackUrl};
use sea_orm::ActiveModelTrait;
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
        let searcher = ctx.data::<Arc<Searcher>>().unwrap();
        let artists = searcher.search_artist(&keyword).await?;
        let albums = searcher.search_album(&keyword).await?;
        let tracks = searcher.search_song(&keyword).await?;
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
        music_player_scanner::refresh_music_library(false, Database::new().await)
            .await
            .map_err(|e| Error::new(e.to_string()))?;

        Ok(false)
    }
}
