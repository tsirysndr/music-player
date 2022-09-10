use crate::api::v1alpha1::{
    library_service_server::LibraryService, GetAlbumDetailsRequest, GetAlbumDetailsResponse,
    GetAlbumsRequest, GetAlbumsResponse, GetArtistDetailsRequest, GetArtistDetailsResponse,
    GetArtistsRequest, GetArtistsResponse, GetTrackDetailsRequest, GetTrackDetailsResponse,
    GetTracksRequest, GetTracksResponse, ScanRequest, ScanResponse, SearchRequest, SearchResponse,
};
use crate::metadata::v1alpha1::{Album, Artist, Track};

#[derive(Debug, Default)]
pub struct Library {}

#[tonic::async_trait]
impl LibraryService for Library {
    async fn scan(
        &self,
        _request: tonic::Request<ScanRequest>,
    ) -> Result<tonic::Response<ScanResponse>, tonic::Status> {
        let response = ScanResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn search(
        &self,
        _request: tonic::Request<SearchRequest>,
    ) -> Result<tonic::Response<SearchResponse>, tonic::Status> {
        let response = SearchResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn get_artists(
        &self,
        _request: tonic::Request<GetArtistsRequest>,
    ) -> Result<tonic::Response<GetArtistsResponse>, tonic::Status> {
        let response = GetArtistsResponse { artists: vec![] };
        Ok(tonic::Response::new(response))
    }
    async fn get_albums(
        &self,
        _request: tonic::Request<GetAlbumsRequest>,
    ) -> Result<tonic::Response<GetAlbumsResponse>, tonic::Status> {
        let response = GetAlbumsResponse { albums: vec![] };
        Ok(tonic::Response::new(response))
    }
    async fn get_tracks(
        &self,
        _request: tonic::Request<GetTracksRequest>,
    ) -> Result<tonic::Response<GetTracksResponse>, tonic::Status> {
        let response = GetTracksResponse { tracks: vec![] };
        Ok(tonic::Response::new(response))
    }
    async fn get_track_details(
        &self,
        _request: tonic::Request<GetTrackDetailsRequest>,
    ) -> Result<tonic::Response<GetTrackDetailsResponse>, tonic::Status> {
        let response = GetTrackDetailsResponse {
            track: Some(Track::default()),
        };
        Ok(tonic::Response::new(response))
    }
    async fn get_album_details(
        &self,
        _request: tonic::Request<GetAlbumDetailsRequest>,
    ) -> Result<tonic::Response<GetAlbumDetailsResponse>, tonic::Status> {
        let response = GetAlbumDetailsResponse {
            album: Some(Album::default()),
        };
        Ok(tonic::Response::new(response))
    }
    async fn get_artist_details(
        &self,
        _request: tonic::Request<GetArtistDetailsRequest>,
    ) -> Result<tonic::Response<GetArtistDetailsResponse>, tonic::Status> {
        let response = GetArtistDetailsResponse {
            artist: Some(Artist::default()),
        };
        Ok(tonic::Response::new(response))
    }
}
