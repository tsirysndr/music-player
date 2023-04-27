use std::sync::Arc;

use async_graphql::*;
use cuid::cuid;
use futures_util::Stream;
use music_player_addons::CurrentSourceDevice;
use music_player_entity::{
    folder as folder_entity, playlist as playlist_entity,
    playlist_tracks as playlist_tracks_entity, select_result, track as track_entity,
};
use music_player_storage::{repo::playlist::PlaylistRepository, Database};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, JoinType, ModelTrait, QueryFilter,
    QueryOrder, QuerySelect, RelationTrait,
};
use tokio::sync::Mutex;

use crate::simple_broker::SimpleBroker;

use super::{
    objects::{folder::Folder, playlist::Playlist, track::Track},
    MutationType,
};

#[derive(Default)]
pub struct PlaylistQuery;

#[Object]
impl PlaylistQuery {
    async fn playlist(&self, ctx: &Context<'_>, id: ID) -> Result<Playlist, Error> {
        let db = ctx.data::<Database>().unwrap();

        let current_device = ctx.data::<Arc<Mutex<CurrentSourceDevice>>>().unwrap();
        let mut device = current_device.lock().await;

        if device.client.is_some() {
            let source = device.client.as_mut().unwrap();
            let result = source.playlist(&id).await?;
            return Ok(result.into());
        }

        let result = PlaylistRepository::new(db.get_connection())
            .find(id.as_str())
            .await?;

        Ok(result.into())
    }

    async fn playlists(&self, ctx: &Context<'_>) -> Result<Vec<Playlist>, Error> {
        let db = ctx.data::<Database>().unwrap();

        let current_device = ctx.data::<Arc<Mutex<CurrentSourceDevice>>>().unwrap();
        let mut device = current_device.lock().await;

        if device.client.is_some() {
            let source = device.client.as_mut().unwrap();
            let result = source.playlists(0, 10).await?;
            return Ok(result.into_iter().map(Into::into).collect());
        }

        PlaylistRepository::new(db.get_connection())
            .find_all()
            .await
            .map(|playlists| playlists.into_iter().map(Into::into).collect())
            .map_err(|e| Error::new(e.to_string()))
    }

    async fn main_playlists(&self, ctx: &Context<'_>) -> Result<Vec<Playlist>, Error> {
        let db = ctx.data::<Database>().unwrap();
        PlaylistRepository::new(db.get_connection())
            .main_playlists()
            .await
            .map(|playlists| playlists.into_iter().map(Into::into).collect())
            .map_err(|e| Error::new(e.to_string()))
    }

    async fn recent_playlists(&self, ctx: &Context<'_>) -> Result<Vec<Playlist>, Error> {
        let db = ctx.data::<Database>().unwrap();
        PlaylistRepository::new(db.get_connection())
            .recent_playlists()
            .await
            .map(|playlists| playlists.into_iter().map(Into::into).collect())
            .map_err(|e| Error::new(e.to_string()))
    }

    async fn folder(&self, ctx: &Context<'_>, id: ID) -> Result<Folder, Error> {
        let db = ctx.data::<Database>().unwrap();
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
        let db = ctx.data::<Database>().unwrap();
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
        let db = ctx.data::<Database>().unwrap();
        let mut folder: Option<folder_entity::Model> = None;
        let folder_id = match folder_id {
            Some(folder_id) => {
                folder = folder_entity::Entity::find_by_id(folder_id.to_string())
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
            folder_id: ActiveValue::Set(folder_id.clone()),
            created_at: ActiveValue::set(chrono::Utc::now()),
        };
        match playlist.insert(db.get_connection()).await {
            Ok(playlist) => {
                if let Some(folder) = folder {
                    SimpleBroker::publish(FolderChanged {
                        folder: folder.into(),
                        mutation_type: MutationType::Updated,
                        playlist: Some(playlist.clone().into()),
                    });
                }
                SimpleBroker::publish(PlaylistChanged {
                    playlist: playlist.clone().into(),
                    mutation_type: MutationType::Created,
                    track: None,
                });
                Ok(playlist.into())
            }
            Err(err) => Err(Error::new(err.to_string())),
        }
    }

    async fn delete_playlist(&self, ctx: &Context<'_>, id: ID) -> Result<Playlist, Error> {
        let db = ctx.data::<Database>().unwrap();
        let playlist = playlist_entity::Entity::find_by_id(id.to_string())
            .one(db.get_connection())
            .await?;

        match playlist {
            Some(playlist) => {
                playlist.clone().delete(db.get_connection()).await?;
                SimpleBroker::publish(PlaylistChanged {
                    playlist: playlist.clone().into(),
                    mutation_type: MutationType::Deleted,
                    track: None,
                });
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
        let db = ctx.data::<Database>().unwrap();
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
                        SimpleBroker::publish(PlaylistChanged {
                            playlist: playlist.clone().into(),
                            mutation_type: MutationType::Updated,
                            track: Some(track.clone().into()),
                        });
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
        let db = ctx.data::<Database>().unwrap();
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
        SimpleBroker::publish(PlaylistChanged {
            playlist: playlist.clone().unwrap().into(),
            mutation_type: MutationType::Updated,
            track: None,
        });
        Ok(playlist.unwrap().into())
    }

    async fn rename_playlist(
        &self,
        ctx: &Context<'_>,
        id: ID,
        name: String,
    ) -> Result<Playlist, Error> {
        let db = ctx.data::<Database>().unwrap();
        let playlist = playlist_entity::Entity::find_by_id(id.to_string())
            .one(db.get_connection())
            .await?;
        let mut playlist: playlist_entity::ActiveModel = match playlist {
            Some(playlist) => playlist.into(),
            None => return Err(Error::new("Playlist not found")),
        };
        playlist.name = ActiveValue::Set(name);
        match playlist.update(db.get_connection()).await {
            Ok(playlist) => {
                SimpleBroker::publish(PlaylistChanged {
                    playlist: playlist.clone().into(),
                    mutation_type: MutationType::Renamed,
                    track: None,
                });
                Ok(playlist.into())
            }
            Err(err) => Err(Error::new(err.to_string())),
        }
    }

    async fn create_folder(&self, ctx: &Context<'_>, name: String) -> Result<Folder, Error> {
        let db = ctx.data::<Database>().unwrap();
        let folder = folder_entity::ActiveModel {
            id: ActiveValue::set(cuid().unwrap()),
            name: ActiveValue::Set(name),
            created_at: ActiveValue::set(chrono::Utc::now()),
        };
        match folder.insert(db.get_connection()).await {
            Ok(folder) => {
                SimpleBroker::publish(FolderChanged {
                    folder: folder.clone().into(),
                    mutation_type: MutationType::Created,
                    playlist: None,
                });
                Ok(folder.into())
            }
            Err(err) => Err(Error::new(err.to_string())),
        }
    }

    async fn delete_folder(&self, ctx: &Context<'_>, id: ID) -> Result<Folder, Error> {
        let db = ctx.data::<Database>().unwrap();
        let folder = folder_entity::Entity::find_by_id(id.to_string())
            .one(db.get_connection())
            .await?;
        match folder {
            Some(folder) => {
                folder.clone().delete(db.get_connection()).await?;
                SimpleBroker::publish(FolderChanged {
                    folder: folder.clone().into(),
                    mutation_type: MutationType::Deleted,
                    playlist: None,
                });
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
        let db = ctx.data::<Database>().unwrap();
        let folder = folder_entity::Entity::find_by_id(id.to_string())
            .one(db.get_connection())
            .await?;
        let mut folder: folder_entity::ActiveModel = match folder {
            Some(folder) => folder.into(),
            None => return Err(Error::new("Folder not found")),
        };
        folder.name = ActiveValue::Set(name);
        match folder.update(db.get_connection()).await {
            Ok(folder) => {
                SimpleBroker::publish(FolderChanged {
                    folder: folder.clone().into(),
                    mutation_type: MutationType::Renamed,
                    playlist: None,
                });
                Ok(folder.into())
            }
            Err(err) => Err(Error::new(err.to_string())),
        }
    }

    async fn move_playlist_to_folder(
        &self,
        ctx: &Context<'_>,
        id: ID,
        folder_id: ID,
    ) -> Result<Folder, Error> {
        let db = ctx.data::<Database>().unwrap();
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
                        let moved_playlist = playlist.clone();
                        let mut playlist: playlist_entity::ActiveModel = playlist.into();
                        playlist.folder_id = ActiveValue::Set(Some(folder.id.clone()));
                        match playlist.update(db.get_connection()).await {
                            Ok(_) => {
                                SimpleBroker::publish(PlaylistChanged {
                                    playlist: moved_playlist.into(),
                                    mutation_type: MutationType::Moved,
                                    track: None,
                                });
                                SimpleBroker::publish(FolderChanged {
                                    folder: folder.clone().into(),
                                    mutation_type: MutationType::Updated,
                                    playlist: None,
                                });
                                Ok(folder.into())
                            }
                            Err(err) => Err(Error::new(err.to_string())),
                        }
                    }
                    None => Err(Error::new("Playlist not found")),
                }
            }
            None => Err(Error::new("Folder not found")),
        }
    }

    async fn move_playlists_to_folder(
        &self,
        ctx: &Context<'_>,
        ids: Vec<ID>,
        folder_id: ID,
    ) -> Result<Folder, Error> {
        let db = ctx.data::<Database>().unwrap();

        let folder = folder_entity::Entity::find_by_id(folder_id.to_string())
            .one(db.get_connection())
            .await?;
        match folder {
            Some(folder) => {
                for id in ids {
                    let playlist = playlist_entity::Entity::find_by_id(id.to_string())
                        .one(db.get_connection())
                        .await?;
                    match playlist {
                        Some(playlist) => {
                            let moved_playlist = playlist.clone();
                            let mut playlist: playlist_entity::ActiveModel = playlist.into();
                            playlist.folder_id = ActiveValue::Set(Some(folder.id.clone()));
                            let p = playlist.update(db.get_connection()).await?;
                            SimpleBroker::publish(PlaylistChanged {
                                playlist: moved_playlist.into(),
                                mutation_type: MutationType::Moved,
                                track: None,
                            });
                            Ok(p)
                        }
                        None => Err(Error::new("Playlist not found")),
                    }?;
                }
                Ok(folder.into())
            }
            None => Err(Error::new("Folder not found")),
        }
    }
}

#[derive(Clone)]
struct FolderChanged {
    folder: Folder,
    mutation_type: MutationType,
    playlist: Option<Playlist>,
}

#[Object]
impl FolderChanged {
    async fn folder(&self) -> &Folder {
        &self.folder
    }

    async fn mutation_type(&self) -> MutationType {
        self.mutation_type
    }

    async fn playlist(&self) -> Option<&Playlist> {
        self.playlist.as_ref()
    }
}

#[derive(Clone)]
struct PlaylistChanged {
    playlist: Playlist,
    mutation_type: MutationType,
    track: Option<Track>,
}

#[Object]
impl PlaylistChanged {
    async fn playlist(&self) -> &Playlist {
        &self.playlist
    }

    async fn mutation_type(&self) -> MutationType {
        self.mutation_type
    }

    async fn track(&self) -> Option<&Track> {
        self.track.as_ref()
    }
}

#[derive(Default)]
pub struct PlaylistSubscription;

#[Subscription]
impl PlaylistSubscription {
    async fn playlists(&self) -> impl Stream<Item = Vec<Playlist>> {
        SimpleBroker::<Vec<Playlist>>::subscribe()
    }

    async fn playlist(&self, _id: ID) -> impl Stream<Item = PlaylistChanged> {
        SimpleBroker::<PlaylistChanged>::subscribe()
    }

    async fn folders(&self) -> impl Stream<Item = Vec<Folder>> {
        SimpleBroker::<Vec<Folder>>::subscribe()
    }

    async fn folder(&self, _id: ID) -> impl Stream<Item = FolderChanged> {
        SimpleBroker::<FolderChanged>::subscribe()
    }
}
