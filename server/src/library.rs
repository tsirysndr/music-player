use futures::future::FutureExt;
use music_player_entity::{album, artist, track};
use music_player_scanner::scan_directory;
use music_player_storage::Database;
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};
use std::sync::Arc;

use crate::api::v1alpha1::{
    library_service_server::LibraryService, GetAlbumDetailsRequest, GetAlbumDetailsResponse,
    GetAlbumsRequest, GetAlbumsResponse, GetArtistDetailsRequest, GetArtistDetailsResponse,
    GetArtistsRequest, GetArtistsResponse, GetTrackDetailsRequest, GetTrackDetailsResponse,
    GetTracksRequest, GetTracksResponse, ScanRequest, ScanResponse, SearchRequest, SearchResponse,
};
use crate::metadata::v1alpha1::{Album, Artist, Track};

pub struct Library {
    db: Arc<Database>,
}

impl Library {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

#[tonic::async_trait]
impl LibraryService for Library {
    async fn scan(
        &self,
        _request: tonic::Request<ScanRequest>,
    ) -> Result<tonic::Response<ScanResponse>, tonic::Status> {
        scan_directory(move |song, db| {
            async move {
                let id = format!("{:x}", md5::compute(song.uri.as_ref().unwrap()));
                let item = track::ActiveModel {
                    id: ActiveValue::set(id),
                    title: ActiveValue::Set(song.title.clone()),
                    artist: ActiveValue::Set(song.artist.clone()),
                    album: ActiveValue::Set(song.album.clone()),
                    genre: ActiveValue::Set(song.genre.clone()),
                    year: ActiveValue::Set(song.year),
                    track: ActiveValue::Set(song.track),
                    bitrate: ActiveValue::Set(song.bitrate),
                    sample_rate: ActiveValue::Set(song.sample_rate),
                    bit_depth: ActiveValue::Set(song.bit_depth),
                    channels: ActiveValue::Set(song.channels),
                    duration: ActiveValue::Set(Some(song.duration.as_secs())),
                    uri: ActiveValue::Set(song.uri.clone()),
                };

                match item.insert(db.get_connection()).await {
                    Ok(_) => (),
                    Err(_) => (),
                }

                let id = format!("{:x}", md5::compute(song.artist.to_string()));
                let item = artist::ActiveModel {
                    id: ActiveValue::set(id),
                    name: ActiveValue::Set(song.artist.clone()),
                };
                match item.insert(db.get_connection()).await {
                    Ok(_) => (),
                    Err(_) => (),
                }

                let id = format!("{:x}", md5::compute(song.album.to_string()));
                let item = album::ActiveModel {
                    id: ActiveValue::set(id),
                    title: ActiveValue::Set(song.album.clone()),
                    artist: ActiveValue::Set(song.artist.clone()),
                };
                match item.insert(db.get_connection()).await {
                    Ok(_) => (),
                    Err(_) => (),
                }
            }
            .boxed()
        })
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
        _request: tonic::Request<GetArtistsRequest>,
    ) -> Result<tonic::Response<GetArtistsResponse>, tonic::Status> {
        artist::Entity::find()
            .all(self.db.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let response = GetArtistsResponse { artists: vec![] };
        Ok(tonic::Response::new(response))
    }

    async fn get_albums(
        &self,
        _request: tonic::Request<GetAlbumsRequest>,
    ) -> Result<tonic::Response<GetAlbumsResponse>, tonic::Status> {
        album::Entity::find()
            .all(self.db.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let response = GetAlbumsResponse { albums: vec![] };
        Ok(tonic::Response::new(response))
    }

    async fn get_tracks(
        &self,
        _request: tonic::Request<GetTracksRequest>,
    ) -> Result<tonic::Response<GetTracksResponse>, tonic::Status> {
        track::Entity::find()
            .all(self.db.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let response = GetTracksResponse { tracks: vec![] };
        Ok(tonic::Response::new(response))
    }

    async fn get_track_details(
        &self,
        _request: tonic::Request<GetTrackDetailsRequest>,
    ) -> Result<tonic::Response<GetTrackDetailsResponse>, tonic::Status> {
        track::Entity::find()
            .one(self.db.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let response = GetTrackDetailsResponse {
            track: Some(Track::default()),
        };
        Ok(tonic::Response::new(response))
    }

    async fn get_album_details(
        &self,
        _request: tonic::Request<GetAlbumDetailsRequest>,
    ) -> Result<tonic::Response<GetAlbumDetailsResponse>, tonic::Status> {
        album::Entity::find()
            .one(self.db.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let response = GetAlbumDetailsResponse {
            album: Some(Album::default()),
        };
        Ok(tonic::Response::new(response))
    }

    async fn get_artist_details(
        &self,
        _request: tonic::Request<GetArtistDetailsRequest>,
    ) -> Result<tonic::Response<GetArtistDetailsResponse>, tonic::Status> {
        artist::Entity::find()
            .one(self.db.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let response = GetArtistDetailsResponse {
            artist: Some(Artist::default()),
        };
        Ok(tonic::Response::new(response))
    }
}
