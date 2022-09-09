use music_player_server::api::v1alpha1::{
    tracklist_service_server::TracklistService, AddTrackRequest, AddTrackResponse,
    ClearTracklistRequest, ClearTracklistResponse, FilterTracklistRequest, FilterTracklistResponse,
    GetNextTrackRequest, GetNextTrackResponse, GetPreviousTrackRequest, GetPreviousTrackResponse,
    GetRandomRequest, GetRandomResponse, GetRepeatRequest, GetRepeatResponse, GetSingleRequest,
    GetSingleResponse, GetTracklistTracksRequest, GetTracklistTracksResponse, RemoveTrackRequest,
    RemoveTrackResponse, SetRepeatRequest, SetRepeatResponse, ShuffleRequest, ShuffleResponse,
};

#[derive(Debug, Default)]
pub struct Tracklist {}

#[tonic::async_trait]
impl TracklistService for Tracklist {
    async fn add_track(
        &self,
        _request: tonic::Request<AddTrackRequest>,
    ) -> Result<tonic::Response<AddTrackResponse>, tonic::Status> {
        let response = AddTrackResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn clear_tracklist(
        &self,
        _request: tonic::Request<ClearTracklistRequest>,
    ) -> Result<tonic::Response<ClearTracklistResponse>, tonic::Status> {
        let response = ClearTracklistResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn filter_tracklist(
        &self,
        _request: tonic::Request<FilterTracklistRequest>,
    ) -> Result<tonic::Response<FilterTracklistResponse>, tonic::Status> {
        let response = FilterTracklistResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn get_random(
        &self,
        _request: tonic::Request<GetRandomRequest>,
    ) -> Result<tonic::Response<GetRandomResponse>, tonic::Status> {
        let response = GetRandomResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn get_repeat(
        &self,
        _request: tonic::Request<GetRepeatRequest>,
    ) -> Result<tonic::Response<GetRepeatResponse>, tonic::Status> {
        let response = GetRepeatResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn get_single(
        &self,
        _request: tonic::Request<GetSingleRequest>,
    ) -> Result<tonic::Response<GetSingleResponse>, tonic::Status> {
        let response = GetSingleResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn get_next_track(
        &self,
        _request: tonic::Request<GetNextTrackRequest>,
    ) -> Result<tonic::Response<GetNextTrackResponse>, tonic::Status> {
        let response = GetNextTrackResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn get_previous_track(
        &self,
        _request: tonic::Request<GetPreviousTrackRequest>,
    ) -> Result<tonic::Response<GetPreviousTrackResponse>, tonic::Status> {
        let response = GetPreviousTrackResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn remove_track(
        &self,
        _request: tonic::Request<RemoveTrackRequest>,
    ) -> Result<tonic::Response<RemoveTrackResponse>, tonic::Status> {
        let response = RemoveTrackResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn shuffle(
        &self,
        _request: tonic::Request<ShuffleRequest>,
    ) -> Result<tonic::Response<ShuffleResponse>, tonic::Status> {
        let response = ShuffleResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn set_repeat(
        &self,
        _request: tonic::Request<SetRepeatRequest>,
    ) -> Result<tonic::Response<SetRepeatResponse>, tonic::Status> {
        let response = SetRepeatResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn get_tracklist_tracks(
        &self,
        _request: tonic::Request<GetTracklistTracksRequest>,
    ) -> Result<tonic::Response<GetTracklistTracksResponse>, tonic::Status> {
        let response = GetTracklistTracksResponse {};
        Ok(tonic::Response::new(response))
    }
}
