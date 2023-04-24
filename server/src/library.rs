use futures::future::FutureExt;
use music_player_entity::{album, artist, artist_tracks, track};
use music_player_scanner::scan_directory;
use music_player_storage::repo::artist::ArtistRepository;
use music_player_storage::repo::track::TrackRepository;
use music_player_storage::Database;
use music_player_storage::{repo::album::AlbumRepository, searcher::Searcher};
use sea_orm::ActiveModelTrait;

use crate::api::music::v1alpha1::{
    library_service_server::LibraryService, GetAlbumDetailsRequest, GetAlbumDetailsResponse,
    GetAlbumsRequest, GetAlbumsResponse, GetArtistDetailsRequest, GetArtistDetailsResponse,
    GetArtistsRequest, GetArtistsResponse, GetTrackDetailsRequest, GetTrackDetailsResponse,
    GetTracksRequest, GetTracksResponse, ScanRequest, ScanResponse, SearchRequest, SearchResponse,
};

pub struct Library {
    db: Database,
}

impl Library {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[tonic::async_trait]
impl LibraryService for Library {
    async fn scan(
        &self,
        _request: tonic::Request<ScanRequest>,
    ) -> Result<tonic::Response<ScanResponse>, tonic::Status> {
        scan_directory(
            move |song, db| {
                async move {
                    let item: artist::ActiveModel = song.try_into().unwrap();
                    match item.insert(db.get_connection()).await {
                        Ok(_) => (),
                        Err(_) => (),
                    }

                    let item: album::ActiveModel = song.try_into().unwrap();
                    match item.insert(db.get_connection()).await {
                        Ok(_) => (),
                        Err(_) => (),
                    }

                    let item: track::ActiveModel = song.try_into().unwrap();

                    match item.insert(db.get_connection()).await {
                        Ok(_) => (),
                        Err(_) => (),
                    }

                    let item: artist_tracks::ActiveModel = song.try_into().unwrap();
                    match item.insert(db.get_connection()).await {
                        Ok(_) => (),
                        Err(_) => (),
                    }
                }
                .boxed()
            },
            &Database::new().await,
            &Searcher::new(),
        )
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?;

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
        request: tonic::Request<GetArtistsRequest>,
    ) -> Result<tonic::Response<GetArtistsResponse>, tonic::Status> {
        let request = request.into_inner();
        let filter = match request.filter.as_str() {
            "" => None,
            _ => Some(request.filter),
        };
        let offset = request.offset;
        let limit = request.limit;
        let results = ArtistRepository::new(&self.db.get_connection())
            .find_all(filter, Some(offset as u64), Some(limit as u64))
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let response = GetArtistsResponse {
            artists: results.into_iter().map(Into::into).collect(),
        };
        Ok(tonic::Response::new(response))
    }

    async fn get_albums(
        &self,
        request: tonic::Request<GetAlbumsRequest>,
    ) -> Result<tonic::Response<GetAlbumsResponse>, tonic::Status> {
        let request = request.into_inner();
        let filter = match request.filter.as_str() {
            "" => None,
            _ => Some(request.filter),
        };
        let offset = request.offset;
        let limit = request.limit;
        let results = AlbumRepository::new(&self.db.get_connection())
            .find_all(filter, Some(offset as u64), Some(limit as u64))
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let response = GetAlbumsResponse {
            albums: results.into_iter().map(Into::into).collect(),
        };
        Ok(tonic::Response::new(response))
    }

    async fn get_tracks(
        &self,
        request: tonic::Request<GetTracksRequest>,
    ) -> Result<tonic::Response<GetTracksResponse>, tonic::Status> {
        let request = request.into_inner();
        let filter = match request.filter.as_str() {
            "" => None,
            _ => Some(request.filter),
        };
        let offset = request.offset;
        let limit = match request.limit {
            0 => 100,
            _ => request.limit,
        };
        let tracks = TrackRepository::new(&self.db.get_connection())
            .find_all(filter, Some(offset as u64), limit as u64)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let response = GetTracksResponse {
            tracks: tracks.into_iter().map(Into::into).collect(),
        };
        Ok(tonic::Response::new(response))
    }

    async fn get_track_details(
        &self,
        request: tonic::Request<GetTrackDetailsRequest>,
    ) -> Result<tonic::Response<GetTrackDetailsResponse>, tonic::Status> {
        let id = request.into_inner().id;

        let track = TrackRepository::new(&self.db.get_connection())
            .find(&id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(GetTrackDetailsResponse {
            track: Some(track.into()),
        }))
    }

    async fn get_album_details(
        &self,
        request: tonic::Request<GetAlbumDetailsRequest>,
    ) -> Result<tonic::Response<GetAlbumDetailsResponse>, tonic::Status> {
        let id = request.into_inner().id;
        let album = AlbumRepository::new(&self.db.get_connection())
            .find(&id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(GetAlbumDetailsResponse {
            album: Some(album.into()),
        }))
    }

    async fn get_artist_details(
        &self,
        request: tonic::Request<GetArtistDetailsRequest>,
    ) -> Result<tonic::Response<GetArtistDetailsResponse>, tonic::Status> {
        let id = request.into_inner().id;

        let artist = ArtistRepository::new(&self.db.get_connection())
            .find(&id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let response = GetArtistDetailsResponse {
            artist: Some(artist.into()),
        };
        Ok(tonic::Response::new(response))
    }
}
