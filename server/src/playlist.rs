use music_player_server::api::v1alpha1::{
    playlist_service_server::PlaylistService, AddItemRequest, AddItemResponse, CreateRequest,
    CreateResponse, DeleteRequest, DeleteResponse, GetItemsRequest, GetItemsResponse,
    RemoveItemRequest, RemoveItemResponse, RenameRequest, RenameResponse, SaveRequest,
    SaveResponse,
};

#[derive(Debug, Default)]
pub struct Playlist {}

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
}
