use crate::api::v1alpha1::{
    mixer_service_server::MixerService, GetMuteRequest, GetMuteResponse, GetVolumeRequest,
    GetVolumeResponse, SetMuteRequest, SetMuteResponse, SetVolumeRequest, SetVolumeResponse,
};

#[derive(Debug, Default)]
pub struct Mixer {}

#[tonic::async_trait]
impl MixerService for Mixer {
    async fn get_mute(
        &self,
        _request: tonic::Request<GetMuteRequest>,
    ) -> Result<tonic::Response<GetMuteResponse>, tonic::Status> {
        let response = GetMuteResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn set_mute(
        &self,
        _request: tonic::Request<SetMuteRequest>,
    ) -> Result<tonic::Response<SetMuteResponse>, tonic::Status> {
        let response = SetMuteResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn get_volume(
        &self,
        _request: tonic::Request<GetVolumeRequest>,
    ) -> Result<tonic::Response<GetVolumeResponse>, tonic::Status> {
        let response = GetVolumeResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn set_volume(
        &self,
        _request: tonic::Request<SetVolumeRequest>,
    ) -> Result<tonic::Response<SetVolumeResponse>, tonic::Status> {
        let response = SetVolumeResponse {};
        Ok(tonic::Response::new(response))
    }
}
