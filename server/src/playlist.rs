use std::sync::Arc;

use music_player_entity::playlist;
use music_player_storage::Database;
use sea_orm::EntityTrait;

use crate::api::v1alpha1::{
    playlist_service_server::PlaylistService, AddItemRequest, AddItemResponse, CreateRequest,
    CreateResponse, DeleteRequest, DeleteResponse, FindAllRequest, FindAllResponse,
    GetItemsRequest, GetItemsResponse, GetPlaylistDetailsRequest, GetPlaylistDetailsResponse,
    RemoveItemRequest, RemoveItemResponse, RenameRequest, RenameResponse, SaveRequest,
    SaveResponse,
};

pub struct Playlist {
    db: Arc<Database>,
}

impl Playlist {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

#[tonic::async_trait]
impl PlaylistService for Playlist {
    async fn create(
        &self,
        _request: tonic::Request<CreateRequest>,
    ) -> Result<tonic::Response<CreateResponse>, tonic::Status> {
        let response = CreateResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn delete(
        &self,
        _request: tonic::Request<DeleteRequest>,
    ) -> Result<tonic::Response<DeleteResponse>, tonic::Status> {
        let response = DeleteResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn get_items(
        &self,
        _request: tonic::Request<GetItemsRequest>,
    ) -> Result<tonic::Response<GetItemsResponse>, tonic::Status> {
        let response = GetItemsResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn save(
        &self,
        _request: tonic::Request<SaveRequest>,
    ) -> Result<tonic::Response<SaveResponse>, tonic::Status> {
        let response = SaveResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn rename(
        &self,
        _request: tonic::Request<RenameRequest>,
    ) -> Result<tonic::Response<RenameResponse>, tonic::Status> {
        let response = RenameResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn remove_item(
        &self,
        _request: tonic::Request<RemoveItemRequest>,
    ) -> Result<tonic::Response<RemoveItemResponse>, tonic::Status> {
        let response = RemoveItemResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn add_item(
        &self,
        _request: tonic::Request<AddItemRequest>,
    ) -> Result<tonic::Response<AddItemResponse>, tonic::Status> {
        let response = AddItemResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn find_all(
        &self,
        _request: tonic::Request<FindAllRequest>,
    ) -> Result<tonic::Response<FindAllResponse>, tonic::Status> {
        playlist::Entity::find()
            .all(self.db.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let response = FindAllResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn get_playlist_details(
        &self,
        _request: tonic::Request<GetPlaylistDetailsRequest>,
    ) -> Result<tonic::Response<GetPlaylistDetailsResponse>, tonic::Status> {
        playlist::Entity::find()
            .all(self.db.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let response = GetPlaylistDetailsResponse {};
        Ok(tonic::Response::new(response))
    }
}
