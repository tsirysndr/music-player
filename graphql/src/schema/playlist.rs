use std::sync::Arc;

use async_graphql::*;
use cuid::cuid;
use music_player_entity::{
    folder as folder_entity, playlist as playlist_entity,
    playlist_tracks as playlist_tracks_entity, track as track_entity,
};
use music_player_storage::Database;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QueryOrder,
};
use tokio::sync::Mutex;

use super::objects::{folder::Folder, playlist::Playlist};

#[derive(Default)]
pub struct PlaylistQuery;

#[Object]
impl PlaylistQuery {
    async fn playlist(&self, ctx: &Context<'_>, id: ID) -> Result<Playlist, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let db = db.lock().await;
        let results: Vec<(playlist_entity::Model, Vec<track_entity::Model>)> =
            playlist_entity::Entity::find_by_id(id.to_string())
                .find_with_related(track_entity::Entity)
                .all(db.get_connection())
                .await?;
        if results.len() == 0 {
            return Err(Error::new("Playlist not found"));
        }
        let (mut playlist, tracks) = results[0].clone();
        playlist.tracks = tracks;
        Ok(playlist.into())
    }

    async fn playlists(&self, ctx: &Context<'_>) -> Result<Vec<Playlist>, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let db = db.lock().await;
        playlist_entity::Entity::find()
            .order_by_asc(playlist_entity::Column::Name)
            .all(db.get_connection())
            .await
            .map(|playlists| playlists.into_iter().map(Into::into).collect())
            .map_err(|e| Error::new(e.to_string()))
    }

    async fn folder(&self, ctx: &Context<'_>, id: ID) -> Result<Folder, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let db = db.lock().await;
        let results: Vec<(folder_entity::Model, Vec<playlist_entity::Model>)> =
            folder_entity::Entity::find_by_id(id.to_string())
                .find_with_related(playlist_entity::Entity)
                .all(db.get_connection())
                .await?;
        if results.len() == 0 {
            return Err(Error::new("Folder not found"));
        }
        let (mut folder, playlists) = results[0].clone();
        folder.playlists = playlists;
        Ok(folder.into())
    }

    async fn folders(&self, ctx: &Context<'_>) -> Result<Vec<Folder>, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let db = db.lock().await;
        folder_entity::Entity::find()
            .order_by_asc(folder_entity::Column::Name)
            .all(db.get_connection())
            .await
            .map(|folders| folders.into_iter().map(Into::into).collect())
            .map_err(|e| Error::new(e.to_string()))
    }
}

#[derive(Default)]
pub struct PlaylistMutation;

#[Object]
impl PlaylistMutation {
    async fn create_playlist(
        &self,
        ctx: &Context<'_>,
        name: String,
        description: Option<String>,
        folder_id: Option<ID>,
    ) -> Result<Playlist, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let db = db.lock().await;
        let folder_id = match folder_id {
            Some(folder_id) => {
                let folder = folder_entity::Entity::find_by_id(folder_id.to_string())
                    .one(db.get_connection())
                    .await?;
                if folder.is_none() {
                    return Err(Error::new("Folder not found"));
                }
                Some(folder_id.to_string())
            }
            None => None,
        };
        let playlist = playlist_entity::ActiveModel {
            id: ActiveValue::set(cuid().unwrap()),
            name: ActiveValue::Set(name),
            description: ActiveValue::Set(description),
            folder_id: ActiveValue::Set(folder_id),
            created_at: ActiveValue::set(chrono::Utc::now()),
        };
        match playlist.insert(db.get_connection()).await {
            Ok(playlist) => Ok(playlist.into()),
            Err(err) => Err(Error::new(err.to_string())),
        }
    }

    async fn delete_playlist(&self, ctx: &Context<'_>, id: ID) -> Result<Playlist, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let db = db.lock().await;
        let playlist = playlist_entity::Entity::find_by_id(id.to_string())
            .one(db.get_connection())
            .await?;

        match playlist {
            Some(playlist) => {
                playlist.clone().delete(db.get_connection()).await?;
                Ok(playlist.into())
            }
            None => Err(Error::new("Playlist not found")),
        }
    }

    async fn add_track_to_playlist(
        &self,
        ctx: &Context<'_>,
        id: ID,
        track_id: ID,
    ) -> Result<Playlist, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let db = db.lock().await;
        let track = track_entity::Entity::find_by_id(track_id.to_string())
            .one(db.get_connection())
            .await?;
        match track {
            Some(track) => {
                let playlist = playlist_entity::Entity::find_by_id(id.to_string())
                    .one(db.get_connection())
                    .await?;
                match playlist {
                    Some(playlist) => {
                        let playlist_track = playlist_tracks_entity::ActiveModel {
                            id: ActiveValue::set(cuid().unwrap()),
                            playlist_id: ActiveValue::Set(playlist.id.clone()),
                            track_id: ActiveValue::Set(track.id.clone()),
                            created_at: ActiveValue::set(chrono::Utc::now()),
                        };
                        playlist_track
                            .insert(db.get_connection())
                            .await
                            .map_err(|err| Error::new(err.to_string()))?;
                        Ok(playlist.into())
                    }
                    None => Err(Error::new("Playlist not found")),
                }
            }
            None => Err(Error::new("Track not found")),
        }
    }

    async fn remove_track_from_playlist(
        &self,
        ctx: &Context<'_>,
        id: ID,
        position: usize,
    ) -> Result<Playlist, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let db = db.lock().await;
        let playlist_track = playlist_tracks_entity::Entity::find()
            .filter(playlist_tracks_entity::Column::PlaylistId.eq(id.to_string()))
            .all(db.get_connection())
            .await?;
        if playlist_track.len() <= position {
            return Err(Error::new("Track not found"));
        }
        playlist_track[position]
            .clone()
            .delete(db.get_connection())
            .await
            .map_err(|err| Error::new(err.to_string()))?;
        let playlist =
            playlist_entity::Entity::find_by_id(playlist_track[position].clone().playlist_id)
                .one(db.get_connection())
                .await?;
        Ok(playlist.unwrap().into())
    }

    async fn rename_playlist(
        &self,
        ctx: &Context<'_>,
        id: ID,
        name: String,
    ) -> Result<Playlist, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let db = db.lock().await;
        let playlist = playlist_entity::Entity::find_by_id(id.to_string())
            .one(db.get_connection())
            .await?;
        let mut playlist: playlist_entity::ActiveModel = match playlist {
            Some(playlist) => playlist.into(),
            None => return Err(Error::new("Playlist not found")),
        };
        playlist.name = ActiveValue::Set(name);
        match playlist.update(db.get_connection()).await {
            Ok(playlist) => Ok(playlist.into()),
            Err(err) => Err(Error::new(err.to_string())),
        }
    }

    async fn create_folder(&self, ctx: &Context<'_>, name: String) -> Result<Folder, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let db = db.lock().await;
        let folder = folder_entity::ActiveModel {
            id: ActiveValue::set(cuid().unwrap()),
            name: ActiveValue::Set(name),
            created_at: ActiveValue::set(chrono::Utc::now()),
        };
        match folder.insert(db.get_connection()).await {
            Ok(folder) => Ok(folder.into()),
            Err(err) => Err(Error::new(err.to_string())),
        }
    }

    async fn delete_folder(&self, ctx: &Context<'_>, id: ID) -> Result<Folder, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let db = db.lock().await;
        let folder = folder_entity::Entity::find_by_id(id.to_string())
            .one(db.get_connection())
            .await?;
        match folder {
            Some(folder) => {
                folder.clone().delete(db.get_connection()).await?;
                Ok(folder.into())
            }
            None => Err(Error::new("Folder not found")),
        }
    }

    async fn rename_folder(
        &self,
        ctx: &Context<'_>,
        id: ID,
        name: String,
    ) -> Result<Folder, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let db = db.lock().await;
        let folder = folder_entity::Entity::find_by_id(id.to_string())
            .one(db.get_connection())
            .await?;
        let mut folder: folder_entity::ActiveModel = match folder {
            Some(folder) => folder.into(),
            None => return Err(Error::new("Folder not found")),
        };
        folder.name = ActiveValue::Set(name);
        match folder.update(db.get_connection()).await {
            Ok(folder) => Ok(folder.into()),
            Err(err) => Err(Error::new(err.to_string())),
        }
    }

    async fn move_playlist_to_folder(
        &self,
        ctx: &Context<'_>,
        id: ID,
        folder_id: ID,
    ) -> Result<Folder, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let db = db.lock().await;
        let folder = folder_entity::Entity::find_by_id(folder_id.to_string())
            .one(db.get_connection())
            .await?;
        match folder {
            Some(folder) => {
                let playlist = playlist_entity::Entity::find_by_id(id.to_string())
                    .one(db.get_connection())
                    .await?;
                match playlist {
                    Some(playlist) => {
                        let mut playlist: playlist_entity::ActiveModel = playlist.into();
                        playlist.folder_id = ActiveValue::Set(Some(folder.id.clone()));
                        match playlist.update(db.get_connection()).await {
                            Ok(_) => Ok(folder.into()),
                            Err(err) => Err(Error::new(err.to_string())),
                        }
                    }
                    None => Err(Error::new("Playlist not found")),
                }
            }
            None => Err(Error::new("Folder not found")),
        }
    }
}
