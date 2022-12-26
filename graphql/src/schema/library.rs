use std::{collections::HashMap, sync::Arc, sync::Mutex as StdMutex};

use async_graphql::{futures_util::FutureExt, *};
use music_player_addons::CurrentDevice;
use music_player_entity::{album as album_entity, artist as artist_entity, track as track_entity};
use music_player_scanner::scan_directory;
use music_player_storage::{
    repo::{album::AlbumRepository, artist::ArtistRepository, track::TrackRepository},
    Database,
};
use music_player_types::types;
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
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<Track>, Error> {
        let connected_device = ctx
            .data::<Arc<StdMutex<HashMap<String, types::Device>>>>()
            .unwrap();
        let current_device = ctx.data::<Arc<Mutex<CurrentDevice>>>().unwrap();
        let mut device = current_device.lock().await;

        if device.source.is_some() {
            let source = device.source.as_mut().unwrap();
            let result = source
                .tracks(offset.unwrap_or(0), limit.unwrap_or(100))
                .await?;

            let device = connected_device.lock().unwrap();
            let device = device.get("current_device").unwrap();
            let base_url = device.base_url.as_ref().unwrap();

            let tracks: Vec<Track> = result.into_iter().map(Into::into).collect();

            return Ok(tracks
                .into_iter()
                .map(|mut track| {
                    track.uri = format!("{}/tracks/{}", base_url, track.id.to_string());
                    track
                })
                .collect());
        }

        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();

        let results = TrackRepository::new(db.lock().await.get_connection())
            .find_all(100)
            .await?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn artists(
        &self,
        ctx: &Context<'_>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<Artist>, Error> {
        let current_device = ctx.data::<Arc<Mutex<CurrentDevice>>>().unwrap();
        let mut device = current_device.lock().await;

        if device.source.is_some() {
            let source = device.source.as_mut().unwrap();
            let artists = source
                .artists(offset.unwrap_or(0), limit.unwrap_or(100))
                .await?;
            return Ok(artists.into_iter().map(Into::into).collect());
        }

        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();

        let results = ArtistRepository::new(db.lock().await.get_connection())
            .find_all()
            .await?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn albums(
        &self,
        ctx: &Context<'_>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<Album>, Error> {
        let connected_device = ctx
            .data::<Arc<StdMutex<HashMap<String, types::Device>>>>()
            .unwrap();
        let current_device = ctx.data::<Arc<Mutex<CurrentDevice>>>().unwrap();
        let mut device = current_device.lock().await;

        if device.source.is_some() {
            let source = device.source.as_mut().unwrap();
            let albums = source
                .albums(offset.unwrap_or(0), limit.unwrap_or(100))
                .await?;

            let device = connected_device.lock().unwrap();
            let device = device.get("current_device").unwrap();
            let base_url = device.base_url.as_ref().unwrap();

            let result: Vec<Album> = albums.into_iter().map(Into::into).collect();

            return Ok(result
                .into_iter()
                .map(|mut album| {
                    album.cover = match album.cover {
                        Some(cover) => Some(format!("{}/covers/{}", base_url, cover)),
                        None => None,
                    };
                    album
                })
                .collect());
        }

        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();

        let results = AlbumRepository::new(db.lock().await.get_connection())
            .find_all()
            .await?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn track(&self, ctx: &Context<'_>, id: ID) -> Result<Track, Error> {
        let current_device = ctx.data::<Arc<Mutex<CurrentDevice>>>().unwrap();
        let mut device = current_device.lock().await;
        let id = id.to_string();

        if device.source.is_some() {
            let source = device.source.as_mut().unwrap();
            let track = source.track(&id).await?;
            return Ok(track.into());
        }

        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();

        let track = TrackRepository::new(db.lock().await.get_connection())
            .find(&id)
            .await?;

        Ok(track.into())
    }

    async fn artist(&self, ctx: &Context<'_>, id: ID) -> Result<Artist, Error> {
        let current_device = ctx.data::<Arc<Mutex<CurrentDevice>>>().unwrap();
        let mut device = current_device.lock().await;
        let id = id.to_string();

        if device.source.is_some() {
            let source = device.source.as_mut().unwrap();
            let artist = source.artist(&id).await?;
            return Ok(artist.into());
        }

        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();

        let artist = ArtistRepository::new(db.lock().await.get_connection())
            .find(&id)
            .await?;

        Ok(artist.into())
    }

    async fn album(&self, ctx: &Context<'_>, id: ID) -> Result<Album, Error> {
        let current_device = ctx.data::<Arc<Mutex<CurrentDevice>>>().unwrap();

        let id = id.to_string();

        let mut device = current_device.lock().await;
        if device.source.is_some() {
            let source = device.source.as_mut().unwrap();
            let album = source.album(&id).await?;
            return Ok(album.into());
        }

        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();

        let album = AlbumRepository::new(db.lock().await.get_connection())
            .find(&id)
            .await?;

        Ok(album.into())
    }

    async fn search(&self, ctx: &Context<'_>, keyword: String) -> Result<SearchResult, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let db = db.lock().await;
        let indexer = db.get_searcher();
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
        scan_directory(move |song, db| {
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
        })
        .await?;

        Ok(false)
    }
}
