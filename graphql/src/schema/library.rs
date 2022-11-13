use std::sync::Arc;

use async_graphql::{futures_util::FutureExt, *};
use music_player_entity::{album as album_entity, artist as artist_entity, track as track_entity};
use music_player_scanner::scan_directory;
use music_player_storage::Database;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use tokio::sync::Mutex;

use super::objects::{album::Album, artist::Artist, search_result::SearchResult, track::Track};

#[derive(Default)]
pub struct LibraryQuery;

#[Object]
impl LibraryQuery {
    async fn tracks(&self, ctx: &Context<'_>) -> Result<Vec<Track>, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let results: Vec<(track_entity::Model, Vec<artist_entity::Model>)> =
            track_entity::Entity::find()
                .limit(100)
                .order_by_asc(track_entity::Column::Title)
                .find_with_related(artist_entity::Entity)
                .all(db.lock().await.get_connection())
                .await?;

        let albums: Vec<(track_entity::Model, Option<album_entity::Model>)> =
            track_entity::Entity::find()
                .limit(100)
                .order_by_asc(track_entity::Column::Title)
                .find_also_related(album_entity::Entity)
                .all(db.lock().await.get_connection())
                .await?;

        let albums: Vec<Option<album_entity::Model>> = albums
            .into_iter()
            .map(|(_track, album)| album.clone())
            .collect();
        let mut albums = albums.into_iter();

        Ok(results
            .into_iter()
            .map(|(track, artists)| {
                let album = albums.next().unwrap().unwrap();
                Track::from(track_entity::Model {
                    artists,
                    album,
                    ..track
                })
            })
            .collect())
    }

    async fn artists(&self, ctx: &Context<'_>) -> Result<Vec<Artist>, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let results = artist_entity::Entity::find()
            .order_by_asc(artist_entity::Column::Name)
            .all(db.lock().await.get_connection())
            .await?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn albums(&self, ctx: &Context<'_>) -> Result<Vec<Album>, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let results = album_entity::Entity::find()
            .order_by_asc(album_entity::Column::Title)
            .all(db.lock().await.get_connection())
            .await?;
        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn track(&self, ctx: &Context<'_>, id: ID) -> Result<Track, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let id = id.to_string();
        let results: Vec<(track_entity::Model, Vec<artist_entity::Model>)> =
            track_entity::Entity::find()
                .filter(track_entity::Column::Id.eq(id.clone()))
                .find_with_related(artist_entity::Entity)
                .all(db.lock().await.get_connection())
                .await?;
        if results.len() == 0 {
            return Err(Error::new("Track not found"));
        }
        let track = results[0].0.clone();
        let album =
            album_entity::Entity::find_by_id(track.album_id.unwrap_or_default().to_string())
                .one(db.lock().await.get_connection())
                .await?;
        Ok(track_entity::Model {
            artists: results[0].1.clone(),
            album: album.unwrap(),
            id: track.id,
            title: track.title,
            duration: track.duration,
            uri: track.uri,
            artist: track.artist,
            ..Default::default()
        }
        .into())
    }

    async fn artist(&self, ctx: &Context<'_>, id: ID) -> Result<Artist, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let id = id.to_string();
        let result = artist_entity::Entity::find_by_id(id.clone())
            .one(db.lock().await.get_connection())
            .await?;

        if result.is_none() {
            return Err(Error::new("Artist not found"));
        }

        let mut artist = result.unwrap();
        let results: Vec<(track_entity::Model, Option<album_entity::Model>)> =
            track_entity::Entity::find()
                .filter(track_entity::Column::ArtistId.eq(id.clone()))
                .order_by_asc(track_entity::Column::Title)
                .find_also_related(album_entity::Entity)
                .all(db.lock().await.get_connection())
                .await?;

        artist.tracks = results
            .into_iter()
            .map(|(track, album)| {
                let mut track = track;
                track.artists = vec![artist.clone()];
                track.album = album.unwrap();
                track
            })
            .collect();

        artist.albums = album_entity::Entity::find()
            .filter(album_entity::Column::ArtistId.eq(id.clone()))
            .order_by_asc(album_entity::Column::Title)
            .all(db.lock().await.get_connection())
            .await?;

        Ok(artist.into())
    }

    async fn album(&self, ctx: &Context<'_>, id: ID) -> Result<Album, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let result = album_entity::Entity::find_by_id(id.to_string())
            .one(db.lock().await.get_connection())
            .await?;
        if result.is_none() {
            return Err(Error::new("Album not found"));
        }
        let mut album = result.unwrap();
        let mut tracks = album
            .find_related(track_entity::Entity)
            .order_by_asc(track_entity::Column::Track)
            .all(db.lock().await.get_connection())
            .await?;
        for track in &mut tracks {
            track.artists = track
                .find_related(artist_entity::Entity)
                .all(db.lock().await.get_connection())
                .await?;
        }
        album.tracks = tracks;
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
