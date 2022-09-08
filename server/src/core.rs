use music_player_server::api::{self, core_service_server::CoreService};

#[derive(Debug, Default)]
pub struct Core {}

#[tonic::async_trait]
impl CoreService for Core {
    async fn get_version(
        &self,
        _request: tonic::Request<api::GetVersionRequest>,
    ) -> Result<tonic::Response<api::GetVersionResponse>, tonic::Status> {
        let response = api::GetVersionResponse {
            version: "0.1.0".to_string(),
        };
        Ok(tonic::Response::new(response))
    }
}
