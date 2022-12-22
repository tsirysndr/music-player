use crate::api::music::v1alpha1::{
    core_service_server::CoreService, GetVersionRequest, GetVersionResponse,
};

#[derive(Debug, Default)]
pub struct Core {}

#[tonic::async_trait]
impl CoreService for Core {
    async fn get_version(
        &self,
        _request: tonic::Request<GetVersionRequest>,
    ) -> Result<tonic::Response<GetVersionResponse>, tonic::Status> {
        let response = GetVersionResponse {
            version: "0.1.0".to_string(),
        };
        Ok(tonic::Response::new(response))
    }
}
