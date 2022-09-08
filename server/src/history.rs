use music_player_server::api::{self, history_service_server::HistoryService};

#[derive(Debug, Default)]
pub struct History {}

#[tonic::async_trait]
impl HistoryService for History {
    async fn get_history(
        &self,
        _request: tonic::Request<api::GetHistoryRequest>,
    ) -> Result<tonic::Response<api::GetHistoryResponse>, tonic::Status> {
        let response = api::GetHistoryResponse {};
        Ok(tonic::Response::new(response))
    }
}
