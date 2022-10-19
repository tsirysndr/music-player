use std::sync::Arc;

use async_graphql::{futures_util::FutureExt, *};
use music_player_entity::{album as album_entity, artist as artist_entity, track as track_entity};
use music_player_scanner::scan_directory;
use music_player_storage::Database;
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait, QueryOrder};
use tokio::sync::Mutex;

use super::objects::{album::Album, artist::Artist, track::Track};

#[derive(Default)]
pub struct LibraryQuery;

#[Object]
impl LibraryQuery {
    async fn tracks(&self, ctx: &Context<'_>) -> Vec<Track> {
        vec![]
    }

    async fn artists(&self, ctx: &Context<'_>) -> Result<Vec<String>, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let results = artist_entity::Entity::find()
            .order_by_asc(artist_entity::Column::Name)
            .all(db.lock().await.get_connection())
            .await?;
        println!("{:?}", results);
        Ok(vec![])
    }

    async fn albums(&self, ctx: &Context<'_>) -> Result<Vec<String>, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let _results = album_entity::Entity::find()
            .order_by_asc(album_entity::Column::Title)
            .all(db.lock().await.get_connection())
            .await?;
        Ok(vec![])
    }

    async fn track(&self, ctx: &Context<'_>, id: ID) -> Result<Track, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let _result = track_entity::Entity::find_by_id(id.to_string())
            .one(db.lock().await.get_connection())
            .await?;
        Ok(Track {
            ..Default::default()
        })
    }

    async fn artist(&self, ctx: &Context<'_>, id: ID) -> Result<Artist, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let _result = artist_entity::Entity::find_by_id(id.to_string())
            .one(db.lock().await.get_connection())
            .await?;
        Ok(Artist {
            ..Default::default()
        })
    }

    async fn album(&self, ctx: &Context<'_>, id: ID) -> Result<Album, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let _result = album_entity::Entity::find_by_id(id.to_string())
            .one(db.lock().await.get_connection())
            .await?;
        Ok(Album {
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
