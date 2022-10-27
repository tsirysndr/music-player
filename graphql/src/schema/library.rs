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

use super::objects::{album::Album, artist::Artist, track::Track};

#[derive(Default)]
pub struct LibraryQuery;

#[Object]
impl LibraryQuery {
    async fn tracks(&self, ctx: &Context<'_>) -> Result<Vec<Track>, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let results = track_entity::Entity::find()
            .limit(100)
            .order_by_asc(track_entity::Column::Title)
            .all(db.lock().await.get_connection())
            .await?;
        Ok(results
            .into_iter()
            .map(|track| Track {
                id: ID(track.id),
                title: track.title,
                uri: track.uri,
                duration: track.duration,
                track_number: track.track,
                artists: vec![Artist {
                    id: ID(format!("{:x}", md5::compute(track.artist.clone()))),
                    name: track.artist.clone(),
                    ..Default::default()
                }],
                album: Album {
                    id: ID(track.album_id.unwrap()),
                    title: track.album,
                    year: track.year,
                    ..Default::default()
                },
                ..Default::default()
            })
            .collect())
    }

    async fn artists(&self, ctx: &Context<'_>) -> Result<Vec<Artist>, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let results = artist_entity::Entity::find()
            .order_by_asc(artist_entity::Column::Name)
            .all(db.lock().await.get_connection())
            .await?;

        Ok(results
            .into_iter()
            .map(|artist| Artist {
                id: ID(artist.id),
                name: artist.name,
                ..Default::default()
            })
            .collect())
    }

    async fn albums(&self, ctx: &Context<'_>) -> Result<Vec<Album>, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let results = album_entity::Entity::find()
            .order_by_asc(album_entity::Column::Title)
            .all(db.lock().await.get_connection())
            .await?;
        Ok(results
            .into_iter()
            .map(|album| Album {
                id: ID(album.id),
                title: album.title,
                artist: album.artist,
                year: album.year,
                cover: album.cover,
                ..Default::default()
            })
            .collect())
    }

    async fn track(&self, ctx: &Context<'_>, id: ID) -> Result<Track, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let result = track_entity::Entity::find_by_id(id.to_string())
            .one(db.lock().await.get_connection())
            .await?;
        if result.is_none() {
            return Err(Error::new("Track not found"));
        }
        let track = result.unwrap();
        Ok(Track {
            id: ID(track.id),
            title: track.title,
            uri: track.uri,
            duration: track.duration,
            track_number: track.track,
            artists: vec![Artist {
                name: track.artist,
                ..Default::default()
            }],
            album: Album {
                id: ID(track.album_id.unwrap()),
                title: track.album,
                year: track.year,
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn artist(&self, ctx: &Context<'_>, id: ID) -> Result<Artist, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let result = artist_entity::Entity::find_by_id(id.to_string())
            .one(db.lock().await.get_connection())
            .await?;

        if result.is_none() {
            return Err(Error::new("Artist not found"));
        }

        let artist = result.unwrap();
        let name = artist.name.clone();
        let tracks = track_entity::Entity::find()
            .filter(track_entity::Column::Artist.eq(artist.name.to_owned()))
            .order_by_asc(track_entity::Column::Title)
            .all(db.lock().await.get_connection())
            .await?;

        Ok(Artist {
            id: ID(artist.id),
            name: artist.name.to_owned(),
            songs: tracks
                .into_iter()
                .map(|track| Track {
                    id: ID(track.id),
                    title: track.title,
                    track_number: track.track,
                    duration: track.duration,
                    artists: vec![Artist {
                        name: name.to_owned(),
                        ..Default::default()
                    }],
                    album: Album {
                        id: ID(track.album_id.unwrap()),
                        title: track.album,
                        year: track.year,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        })
    }

    async fn album(&self, ctx: &Context<'_>, id: ID) -> Result<Album, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let result = album_entity::Entity::find_by_id(id.to_string())
            .one(db.lock().await.get_connection())
            .await?;
        if result.is_none() {
            return Err(Error::new("Album not found"));
        }
        let album = result.unwrap();
        let tracks = album
            .find_related(track_entity::Entity)
            .order_by_asc(track_entity::Column::Track)
            .all(db.lock().await.get_connection())
            .await?;
        Ok(Album {
            id: ID(album.id),
            title: album.title,
            artist: album.artist,
            year: album.year,
            tracks: tracks
                .into_iter()
                .map(|track| Track {
                    id: ID(track.id),
                    title: track.title,
                    track_number: track.track,
                    duration: track.duration,
                    artists: vec![Artist {
                        name: track.artist,
                        ..Default::default()
                    }],
                    ..Default::default()
                })
                .collect(),
            cover: album.cover,
            ..Default::default()
        })
    }

    async fn search(&self, ctx: &Context<'_>) -> bool {
        ctx.data::<Arc<Mutex<Database>>>().unwrap();
        false
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
                    title: ActiveValue::Set(song.album.clone()),
                    artist: ActiveValue::Set(song.artist.clone()),
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
                    artist: ActiveValue::Set(song.artist.clone()),
                    album: ActiveValue::Set(song.album.clone()),
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
