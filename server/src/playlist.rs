use music_player_entity::{playlist, playlist_tracks, track};
use music_player_storage::Database;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, Set,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    api::v1alpha1::{
        playlist_service_server::PlaylistService, AddItemRequest, AddItemResponse,
        CreateFolderRequest, CreateFolderResponse, CreateRequest, CreateResponse,
        DeleteFolderRequest, DeleteFolderResponse, DeleteRequest, DeleteResponse,
        FindAllFoldersRequest, FindAllFoldersResponse, FindAllRequest, FindAllResponse,
        GetFolderDetailsRequest, GetFolderDetailsResponse, GetItemsRequest, GetItemsResponse,
        GetPlaylistDetailsRequest, GetPlaylistDetailsResponse, RemoveItemRequest,
        RemoveItemResponse, RenameFolderRequest, RenameFolderResponse, RenameRequest,
        RenameResponse,
    },
    metadata::v1alpha1::Track,
};

pub struct Playlist {
    db: Arc<Mutex<Database>>,
}

impl Playlist {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self { db }
    }
}

#[tonic::async_trait]
impl PlaylistService for Playlist {
    async fn create(
        &self,
        request: tonic::Request<CreateRequest>,
    ) -> Result<tonic::Response<CreateResponse>, tonic::Status> {
        let item = playlist::ActiveModel {
            id: ActiveValue::set(Uuid::new_v4().to_string()),
            name: ActiveValue::set(request.get_ref().name.clone()),
            ..Default::default()
        };
        match item.insert(self.db.lock().await.get_connection()).await {
            Ok(saved) => {
                for track in request.get_ref().tracks.iter() {
                    let item = playlist_tracks::ActiveModel {
                        id: ActiveValue::set(Uuid::new_v4().to_string()),
                        playlist_id: ActiveValue::set(saved.id.clone()),
                        track_id: ActiveValue::set(track.id.clone()),
                        created_at: ActiveValue::set(chrono::Utc::now()),
                    };
                    match item.insert(self.db.lock().await.get_connection()).await {
                        Ok(_) => (),
                        Err(_) => (),
                    }
                }
                Ok(tonic::Response::new(CreateResponse {
                    id: saved.id,
                    name: saved.name,
                    tracks: request.get_ref().tracks.clone(),
                    ..Default::default()
                }))
            }
            Err(e) => Err(tonic::Status::internal(e.to_string())),
        }
    }

    async fn delete(
        &self,
        request: tonic::Request<DeleteRequest>,
    ) -> Result<tonic::Response<DeleteResponse>, tonic::Status> {
        playlist::Entity::delete_by_id(request.get_ref().id.clone())
            .exec(self.db.lock().await.get_connection())
            .await
            .map(|_| {
                tonic::Response::new(DeleteResponse {
                    id: request.get_ref().id.clone(),
                    ..Default::default()
                })
            })
            .map_err(|_| tonic::Status::internal("Failed to delete playlist"))
    }

    async fn get_items(
        &self,
        request: tonic::Request<GetItemsRequest>,
    ) -> Result<tonic::Response<GetItemsResponse>, tonic::Status> {
        let result = playlist::Entity::find_by_id(request.get_ref().id.clone())
            .one(self.db.lock().await.get_connection())
            .await;
        match result {
            Ok(playlist) => {
                if playlist.is_none() {
                    return Err(tonic::Status::not_found("Playlist not found"));
                }
                playlist
                    .clone()
                    .unwrap()
                    .find_related(track::Entity)
                    .all(self.db.lock().await.get_connection())
                    .await
                    .map(|tracks| {
                        tonic::Response::new(GetItemsResponse {
                            id: playlist.clone().unwrap().id,
                            name: playlist.clone().unwrap().name,
                            tracks: tracks
                                .into_iter()
                                .map(|track| Track {
                                    id: track.id,
                                    title: track.title,
                                    uri: track.uri,
                                    duration: track.duration.unwrap_or_default(),
                                    disc_number: i32::try_from(track.track.unwrap_or_default())
                                        .unwrap(),
                                    ..Default::default()
                                })
                                .collect(),
                            ..Default::default()
                        })
                    })
                    .map_err(|_| tonic::Status::internal("Failed to get playlist items"))
            }
            Err(_) => return Err(tonic::Status::internal("Failed to get playlist")),
        }
    }

    async fn rename(
        &self,
        request: tonic::Request<RenameRequest>,
    ) -> Result<tonic::Response<RenameResponse>, tonic::Status> {
        let updates = playlist::ActiveModel {
            name: Set(request.get_ref().name.clone()),
            ..Default::default()
        };
        playlist::Entity::update(updates)
            .filter(playlist::Column::Id.eq("test"))
            .exec(self.db.lock().await.get_connection())
            .await
            .map(|updated| {
                tonic::Response::new(RenameResponse {
                    id: updated.id,
                    name: updated.name,
                    ..Default::default()
                })
            })
            .map_err(|_| tonic::Status::internal("Failed to rename playlist"))
    }

    async fn remove_item(
        &self,
        request: tonic::Request<RemoveItemRequest>,
    ) -> Result<tonic::Response<RemoveItemResponse>, tonic::Status> {
        let item = playlist_tracks::Entity::find()
            .filter(
                playlist_tracks::Column::PlaylistId
                    .eq(request.get_ref().id.clone())
                    .and(playlist_tracks::Column::TrackId.eq(request.get_ref().track_id.clone())),
            )
            .one(self.db.lock().await.get_connection())
            .await;
        if item.is_err() {
            return Err(tonic::Status::internal(
                "Failed to remove item from playlist",
            ));
        }
        playlist_tracks::Entity::delete(playlist_tracks::ActiveModel {
            id: Set(item.unwrap().unwrap().id),
            ..Default::default()
        })
        .exec(self.db.lock().await.get_connection())
        .await
        .map(|_| {
            tonic::Response::new(RemoveItemResponse {
                id: request.get_ref().id.clone(),
                ..Default::default()
            })
        })
        .map_err(|_| tonic::Status::internal("Failed to remove item from playlist"))
    }

    async fn add_item(
        &self,
        request: tonic::Request<AddItemRequest>,
    ) -> Result<tonic::Response<AddItemResponse>, tonic::Status> {
        let item = playlist_tracks::ActiveModel {
            id: ActiveValue::set(Uuid::new_v4().to_string()),
            playlist_id: ActiveValue::set(request.get_ref().id.clone()),
            track_id: ActiveValue::set(request.get_ref().track_id.clone()),
            created_at: ActiveValue::set(chrono::Utc::now()),
        };
        match item.insert(self.db.lock().await.get_connection()).await {
            Ok(saved) => Ok(tonic::Response::new(AddItemResponse {
                id: saved.id,
                ..Default::default()
            })),
            Err(e) => Err(tonic::Status::internal(e.to_string())),
        }
    }

    async fn find_all(
        &self,
        _request: tonic::Request<FindAllRequest>,
    ) -> Result<tonic::Response<FindAllResponse>, tonic::Status> {
        playlist::Entity::find()
            .all(self.db.lock().await.get_connection())
            .await
            .map(|playlists| {
                tonic::Response::new(FindAllResponse {
                    playlists: playlists
                        .into_iter()
                        .map(|playlist| GetPlaylistDetailsResponse {
                            id: playlist.id,
                            name: playlist.name,
                            ..Default::default()
                        })
                        .collect(),
                    ..Default::default()
                })
            })
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let response = FindAllResponse {
            ..Default::default()
        };
        Ok(tonic::Response::new(response))
    }

    async fn get_playlist_details(
        &self,
        request: tonic::Request<GetPlaylistDetailsRequest>,
    ) -> Result<tonic::Response<GetPlaylistDetailsResponse>, tonic::Status> {
        let result = playlist::Entity::find_by_id(request.get_ref().id.clone())
            .one(self.db.lock().await.get_connection())
            .await;
        match result {
            Ok(playlist) => {
                if playlist.is_none() {
                    return Err(tonic::Status::not_found("Playlist not found"));
                }
                playlist
                    .clone()
                    .unwrap()
                    .find_related(track::Entity)
                    .all(self.db.lock().await.get_connection())
                    .await
                    .map(|tracks| {
                        tonic::Response::new(GetPlaylistDetailsResponse {
                            id: playlist.clone().unwrap().id,
                            name: playlist.clone().unwrap().name,
                            tracks: tracks
                                .into_iter()
                                .map(|track| Track {
                                    id: track.id,
                                    title: track.title,
                                    uri: track.uri,
                                    duration: track.duration.unwrap_or_default(),
                                    disc_number: i32::try_from(track.track.unwrap_or_default())
                                        .unwrap(),
                                    ..Default::default()
                                })
                                .collect(),
                            ..Default::default()
                        })
                    })
                    .map_err(|_| tonic::Status::internal("Failed to get playlist items"))
            }
            Err(_) => return Err(tonic::Status::internal("Failed to get playlist")),
        }
    }

    async fn create_folder(
        &self,
        _request: tonic::Request<CreateFolderRequest>,
    ) -> Result<tonic::Response<CreateFolderResponse>, tonic::Status> {
        todo!()
    }

    async fn delete_folder(
        &self,
        _request: tonic::Request<DeleteFolderRequest>,
    ) -> Result<tonic::Response<DeleteFolderResponse>, tonic::Status> {
        todo!()
    }

    async fn rename_folder(
        &self,
        _request: tonic::Request<RenameFolderRequest>,
    ) -> Result<tonic::Response<RenameFolderResponse>, tonic::Status> {
        todo!()
    }

    async fn get_folder_details(
        &self,
        _request: tonic::Request<GetFolderDetailsRequest>,
    ) -> Result<tonic::Response<GetFolderDetailsResponse>, tonic::Status> {
        todo!()
    }

    async fn find_all_folders(
        &self,
        _request: tonic::Request<FindAllFoldersRequest>,
    ) -> Result<tonic::Response<FindAllFoldersResponse>, tonic::Status> {
        todo!()
    }
}
