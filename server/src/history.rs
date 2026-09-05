use music_player_storage::Database;

use crate::api::music::v1alpha1::{
    history_service_server::HistoryService, GetHistoryRequest, GetHistoryResponse,
};

pub struct History {
    _db: Database,
}

impl History {
    pub fn new(db: Database) -> Self {
        Self { _db: db }
    }
}

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
