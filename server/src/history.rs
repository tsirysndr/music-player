use std::sync::Arc;

use music_player_storage::Database;
use tokio::sync::Mutex;

use crate::api::music::v1alpha1::{
    history_service_server::HistoryService, GetHistoryRequest, GetHistoryResponse,
};

pub struct History {
    db: Arc<Mutex<Database>>,
}

impl History {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self { db }
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
