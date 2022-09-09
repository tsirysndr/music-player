use music_player_server::api::v1alpha1::{
    history_service_server::HistoryService, GetHistoryRequest, GetHistoryResponse,
};

#[derive(Debug, Default)]
pub struct History {}

#[tonic::async_trait]
impl HistoryService for History {
    async fn get_history(
        &self,
        _request: tonic::Request<GetHistoryRequest>,
    ) -> Result<tonic::Response<GetHistoryResponse>, tonic::Status> {
        let response = GetHistoryResponse {};
        Ok(tonic::Response::new(response))
    }
}
