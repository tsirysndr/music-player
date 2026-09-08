use music_player_entity::{album, playlist, playlist_tracks, track};
use music_player_provider::{Page, ProviderState};
use music_player_storage::{repo::playlist::PlaylistRepository, Database};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, Set,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::{
    metadata::v1alpha1::Track,
    music::v1alpha1::{
        playlist_service_server::PlaylistService, AddItemRequest, AddItemResponse,
        CreateFolderRequest, CreateFolderResponse, CreateRequest, CreateResponse,
        DeleteFolderRequest, DeleteFolderResponse, DeleteRequest, DeleteResponse,
        FindAllFoldersRequest, FindAllFoldersResponse, FindAllRequest, FindAllResponse,
        GetFolderDetailsRequest, GetFolderDetailsResponse, GetItemsRequest, GetItemsResponse,
        GetPlaylistDetailsRequest, GetPlaylistDetailsResponse, PreviewSmartPlaylistRequest,
        PreviewSmartPlaylistResponse, RegenerateSmartPlaylistRequest,
        RegenerateSmartPlaylistResponse, RemoveItemRequest, RemoveItemResponse,
        RenameFolderRequest, RenameFolderResponse, RenameRequest, RenameResponse, SmartPlaylist,
    },
};

/// Turn the wire form of a smart playlist's rule into the query it stands for.
fn spec_of(smart: &SmartPlaylist) -> music_player_rsql::QuerySpec {
    music_player_rsql::QuerySpec {
        filter: smart.filter.clone(),
        sort_by: Some(smart.sort_by.clone()).filter(|s| !s.trim().is_empty()),
        sort_order: if smart.sort_order.eq_ignore_ascii_case("desc") {
            music_player_rsql::SortOrder::Desc
        } else {
            music_player_rsql::SortOrder::Asc
        },
        // 0 means unlimited on the wire; `None` is what the builder wants.
        limit: Some(smart.limit).filter(|n| *n > 0),
    }
}

pub struct Playlist {
    db: Database,
    providers: Arc<ProviderState>,
}

impl Playlist {
    pub fn new(db: Database, providers: Arc<ProviderState>) -> Self {
        Self { db, providers }
    }
}

#[tonic::async_trait]
impl PlaylistService for Playlist {
    async fn create(
        &self,
        request: tonic::Request<CreateRequest>,
    ) -> Result<tonic::Response<CreateResponse>, tonic::Status> {
        let smart = request.get_ref().smart.clone();
        // A filter that does not compile is rejected before anything is
        // created: an empty playlist left behind by a typo is worse than an
        // error the caller can show.
        if let Some(smart) = &smart {
            if let Err(e) = music_player_rsql::build(&spec_of(smart), &music_player_rsql::TRACKS) {
                return Err(tonic::Status::invalid_argument(e.message));
            }
        }
        let item = playlist::ActiveModel {
            id: ActiveValue::set(Uuid::new_v4().to_string()),
            name: ActiveValue::set(request.get_ref().name.clone()),
            created_at: ActiveValue::set(chrono::Utc::now()),
            is_smart: ActiveValue::set(smart.is_some()),
            rsql: ActiveValue::set(smart.as_ref().map(|s| s.filter.clone())),
            sort_by: ActiveValue::set(
                smart
                    .as_ref()
                    .map(|s| s.sort_by.clone())
                    .filter(|s| !s.trim().is_empty()),
            ),
            sort_order: ActiveValue::set(
                smart
                    .as_ref()
                    .map(|s| s.sort_order.clone())
                    .filter(|s| !s.trim().is_empty()),
            ),
            max_tracks: ActiveValue::set(smart.as_ref().map(|s| s.limit).filter(|n| *n > 0)),
            ..Default::default()
        };
        match item.insert(self.db.get_connection()).await {
            Ok(saved) => {
                // A smart playlist fills itself; the caller's `tracks` are
                // ignored, so it comes back populated rather than empty.
                if smart.is_some() {
                    if let Err(e) = music_player_storage::smart_playlist::regenerate(
                        self.db.get_connection(),
                        &saved.id,
                    )
                    .await
                    {
                        return Err(tonic::Status::internal(e.to_string()));
                    }
                    return Ok(tonic::Response::new(CreateResponse {
                        id: saved.id,
                        name: saved.name,
                        tracks: vec![],
                    }));
                }
                for track in request.get_ref().tracks.iter() {
                    let item = playlist_tracks::ActiveModel {
                        id: ActiveValue::set(Uuid::new_v4().to_string()),
                        playlist_id: ActiveValue::set(saved.id.clone()),
                        track_id: ActiveValue::set(track.id.clone()),
                        created_at: ActiveValue::set(chrono::Utc::now()),
                    };
                    // A row that will not insert is not worth failing the
                    // whole create over — the playlist itself is saved.
                    let _ = item.insert(self.db.get_connection()).await;
                }
                Ok(tonic::Response::new(CreateResponse {
                    id: saved.id,
                    name: saved.name,
                    tracks: request.get_ref().tracks.clone(),
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
            .exec(self.db.get_connection())
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
            .one(self.db.get_connection())
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
                    .all(self.db.get_connection())
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
            .exec(self.db.get_connection())
            .await
            .map(|updated| {
                tonic::Response::new(RenameResponse {
                    id: updated.id,
                    name: updated.name,
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
            .one(self.db.get_connection())
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
        .exec(self.db.get_connection())
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
        match item.insert(self.db.get_connection()).await {
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
        if let Some(current) = self.providers.current().await {
            let playlists = current
                .provider
                .playlists(Page::default())
                .await
                .map_err(crate::library::provider_status)?;
            let playlists = music_player_provider::url::decorate_all(playlists, &current.config);
            return Ok(tonic::Response::new(FindAllResponse {
                playlists: playlists.into_iter().map(Into::into).collect(),
            }));
        }

        let result = PlaylistRepository::new(self.db.get_connection())
            .find_all()
            .await
            .map_err(|_| tonic::Status::internal("Failed to get playlist"))?;
        Ok(tonic::Response::new(FindAllResponse {
            playlists: result.into_iter().map(Into::into).collect(),
        }))
    }

    async fn get_playlist_details(
        &self,
        request: tonic::Request<GetPlaylistDetailsRequest>,
    ) -> Result<tonic::Response<GetPlaylistDetailsResponse>, tonic::Status> {
        if let Some(current) = self.providers.current().await {
            let playlist = current
                .provider
                .playlist(&request.get_ref().id)
                .await
                .map_err(crate::library::provider_status)?;
            let playlist = music_player_provider::url::decorate(playlist, &current.config);
            return Ok(tonic::Response::new(GetPlaylistDetailsResponse {
                track_count: playlist.len(),
                id: playlist.id,
                name: playlist.name,
                description: playlist.description.unwrap_or_default(),
                tracks: playlist.tracks.into_iter().map(Into::into).collect(),
            }));
        }

        let result = PlaylistRepository::new(self.db.get_connection())
            .find(&request.get_ref().id)
            .await
            .map_err(|_| tonic::Status::internal("Failed to get playlist"))?;
        Ok(tonic::Response::new(result.into()))
    }

    async fn regenerate_smart_playlist(
        &self,
        request: tonic::Request<RegenerateSmartPlaylistRequest>,
    ) -> Result<tonic::Response<RegenerateSmartPlaylistResponse>, tonic::Status> {
        let count = music_player_storage::smart_playlist::regenerate(
            self.db.get_connection(),
            &request.get_ref().id,
        )
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(RegenerateSmartPlaylistResponse {
            count: count as u32,
        }))
    }

    async fn preview_smart_playlist(
        &self,
        request: tonic::Request<PreviewSmartPlaylistRequest>,
    ) -> Result<tonic::Response<PreviewSmartPlaylistResponse>, tonic::Status> {
        /// How many matches to return in full, for the form to show.
        const SAMPLE: usize = 10;

        let Some(smart) = request.get_ref().smart.clone() else {
            return Ok(tonic::Response::new(PreviewSmartPlaylistResponse::default()));
        };
        let spec = spec_of(&smart);

        // A filter the user is still typing is expected to be invalid half the
        // time, so that comes back as `error` rather than as a gRPC failure.
        let ids = match music_player_storage::smart_playlist::matching_track_ids(
            self.db.get_connection(),
            &spec,
        )
        .await
        {
            Ok(ids) => ids,
            Err(e) => {
                return Ok(tonic::Response::new(PreviewSmartPlaylistResponse {
                    error: e.to_string(),
                    ..Default::default()
                }))
            }
        };

        let count = ids.len() as u32;
        let head: Vec<String> = ids.into_iter().take(SAMPLE).collect();
        let mut tracks = if head.is_empty() {
            vec![]
        } else {
            track::Entity::find()
                .filter(track::Column::Id.is_in(head.clone()))
                .find_also_related(album::Entity)
                .all(self.db.get_connection())
                .await
                .map_err(|e| tonic::Status::internal(e.to_string()))?
                .into_iter()
                .map(|(mut track, album)| {
                    track.album = album.unwrap_or_default();
                    track
                })
                .collect::<Vec<_>>()
        };
        // `is_in` returns table order; the preview has to show the order the
        // filter actually produced.
        tracks.sort_by_key(|track| {
            head.iter()
                .position(|id| *id == track.id)
                .unwrap_or(usize::MAX)
        });

        Ok(tonic::Response::new(PreviewSmartPlaylistResponse {
            count,
            tracks: tracks.into_iter().map(Into::into).collect(),
            error: String::new(),
        }))
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
