use crate::api::v1alpha1::{
    playback_service_server::PlaybackService, GetCurrentlyPlayingSongRequest,
    GetCurrentlyPlayingSongResponse, GetPlaybackStateRequest, GetPlaybackStateResponse,
    GetTimePositionRequest, GetTimePositionResponse, NextRequest, NextResponse, PauseRequest,
    PauseResponse, PlayRequest, PlayResponse, PreviousRequest, PreviousResponse, SeekRequest,
    SeekResponse, StopRequest, StopResponse,
};

#[derive(Debug, Default)]
pub struct Playback {}

#[tonic::async_trait]
impl PlaybackService for Playback {
    async fn get_currently_playing_song(
        &self,
        _request: tonic::Request<GetCurrentlyPlayingSongRequest>,
    ) -> Result<tonic::Response<GetCurrentlyPlayingSongResponse>, tonic::Status> {
        let response = GetCurrentlyPlayingSongResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn get_playback_state(
        &self,
        _request: tonic::Request<GetPlaybackStateRequest>,
    ) -> Result<tonic::Response<GetPlaybackStateResponse>, tonic::Status> {
        let response = GetPlaybackStateResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn get_time_position(
        &self,
        _request: tonic::Request<GetTimePositionRequest>,
    ) -> Result<tonic::Response<GetTimePositionResponse>, tonic::Status> {
        let response = GetTimePositionResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn next(
        &self,
        _request: tonic::Request<NextRequest>,
    ) -> Result<tonic::Response<NextResponse>, tonic::Status> {
        let response = NextResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn previous(
        &self,
        _request: tonic::Request<PreviousRequest>,
    ) -> Result<tonic::Response<PreviousResponse>, tonic::Status> {
        let response = PreviousResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn play(
        &self,
        _request: tonic::Request<PlayRequest>,
    ) -> Result<tonic::Response<PlayResponse>, tonic::Status> {
        let response = PlayResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn pause(
        &self,
        _request: tonic::Request<PauseRequest>,
    ) -> Result<tonic::Response<PauseResponse>, tonic::Status> {
        let response = PauseResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn stop(
        &self,
        _request: tonic::Request<StopRequest>,
    ) -> Result<tonic::Response<StopResponse>, tonic::Status> {
        let response = StopResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn seek(
        &self,
        _request: tonic::Request<SeekRequest>,
    ) -> Result<tonic::Response<SeekResponse>, tonic::Status> {
        let response = SeekResponse {};
        Ok(tonic::Response::new(response))
    }
}
