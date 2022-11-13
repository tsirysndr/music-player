use futures::future::FutureExt;
use music_player_entity::{album, artist, artist_tracks, track};
use music_player_scanner::scan_directory;
use music_player_storage::Database;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::api::v1alpha1::{
    library_service_server::LibraryService, GetAlbumDetailsRequest, GetAlbumDetailsResponse,
    GetAlbumsRequest, GetAlbumsResponse, GetArtistDetailsRequest, GetArtistDetailsResponse,
    GetArtistsRequest, GetArtistsResponse, GetTrackDetailsRequest, GetTrackDetailsResponse,
    GetTracksRequest, GetTracksResponse, ScanRequest, ScanResponse, SearchRequest, SearchResponse,
};
use crate::metadata::v1alpha1::{Album, Artist, ArtistSong, Song, SongArtist, Track};

pub struct Library {
    db: Arc<Mutex<Database>>,
}

impl Library {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
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
        let results = artist::Entity::find()
            .order_by_asc(artist::Column::Name)
            .all(self.db.lock().await.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let response = GetArtistsResponse {
            artists: results.into_iter().map(Into::into).collect(),
        };
        Ok(tonic::Response::new(response))
    }

    async fn get_albums(
        &self,
        _request: tonic::Request<GetAlbumsRequest>,
    ) -> Result<tonic::Response<GetAlbumsResponse>, tonic::Status> {
        let results = album::Entity::find()
            .order_by_asc(album::Column::Title)
            .all(self.db.lock().await.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let response = GetAlbumsResponse {
            albums: results.into_iter().map(Into::into).collect(),
        };
        Ok(tonic::Response::new(response))
    }

    async fn get_tracks(
        &self,
        _request: tonic::Request<GetTracksRequest>,
    ) -> Result<tonic::Response<GetTracksResponse>, tonic::Status> {
        let results: Vec<(track::Model, Vec<artist::Model>)> = track::Entity::find()
            .order_by_asc(track::Column::Title)
            .find_with_related(artist::Entity)
            .all(self.db.lock().await.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let albums: Vec<(track::Model, Option<album::Model>)> = track::Entity::find()
            .order_by_asc(track::Column::Title)
            .find_also_related(album::Entity)
            .all(self.db.lock().await.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let albums: Vec<Option<album::Model>> = albums
            .into_iter()
            .map(|(_track, album)| album.clone())
            .collect();
        let mut albums = albums.into_iter();

        let response = GetTracksResponse {
            tracks: results
                .into_iter()
                .map(|(track, artists)| {
                    let album = albums.next().unwrap().unwrap();
                    Track {
                        artists: artists.into_iter().map(Into::into).collect(),
                        album: Some(album.into()),
                        ..track.into()
                    }
                })
                .collect(),
        };
        Ok(tonic::Response::new(response))
    }

    async fn get_track_details(
        &self,
        request: tonic::Request<GetTrackDetailsRequest>,
    ) -> Result<tonic::Response<GetTrackDetailsResponse>, tonic::Status> {
        let id = request.into_inner().id;
        let result: Vec<(track::Model, Vec<artist::Model>)> = track::Entity::find_by_id(id.clone())
            .find_with_related(artist::Entity)
            .all(self.db.lock().await.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        if result.len() == 0 {
            return Err(tonic::Status::not_found("Track not found"));
        }

        let (mut track, artists) = result.into_iter().next().unwrap();
        track.artists = artists;

        let result: Vec<(track::Model, Option<album::Model>)> =
            track::Entity::find_by_id(id.clone())
                .find_also_related(album::Entity)
                .all(self.db.lock().await.get_connection())
                .await
                .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let (_, album) = result.into_iter().next().unwrap();
        track.album = album.unwrap();

        Ok(tonic::Response::new(GetTrackDetailsResponse {
            track: Some(track.into()),
        }))
    }

    async fn get_album_details(
        &self,
        request: tonic::Request<GetAlbumDetailsRequest>,
    ) -> Result<tonic::Response<GetAlbumDetailsResponse>, tonic::Status> {
        let result: Vec<(album::Model, Vec<track::Model>)> =
            album::Entity::find_by_id(request.into_inner().id)
                .find_with_related(track::Entity)
                .order_by_asc(track::Column::Track)
                .all(self.db.lock().await.get_connection())
                .await
                .map_err(|e| tonic::Status::internal(e.to_string()))?;
        if result.len() == 0 {
            return Err(tonic::Status::not_found("Album not found"));
        }
        let (mut album, mut tracks) = result.into_iter().next().unwrap();

        for track in &mut tracks {
            track.artists = track
                .find_related(artist::Entity)
                .all(self.db.lock().await.get_connection())
                .await
                .map_err(|e| tonic::Status::internal(e.to_string()))?;
        }

        album.tracks = tracks;

        Ok(tonic::Response::new(GetAlbumDetailsResponse {
            album: Some(album.into()),
        }))
    }

    async fn get_artist_details(
        &self,
        request: tonic::Request<GetArtistDetailsRequest>,
    ) -> Result<tonic::Response<GetArtistDetailsResponse>, tonic::Status> {
        let id = request.into_inner().id;
        let result = artist::Entity::find_by_id(id.clone())
            .one(self.db.lock().await.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        if result.is_none() {
            return Err(tonic::Status::not_found("Artist not found"));
        }

        let mut artist = result.unwrap();
        let results: Vec<(track::Model, Option<album::Model>)> = track::Entity::find()
            .filter(track::Column::ArtistId.eq(id.to_owned()))
            .order_by_asc(track::Column::Title)
            .find_also_related(album::Entity)
            .all(self.db.lock().await.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        artist.tracks = results
            .into_iter()
            .map(|(track, album)| {
                let mut track = track;
                track.artists = vec![artist.clone()];
                track.album = album.unwrap();
                track
            })
            .collect();

        artist.albums = album::Entity::find()
            .filter(album::Column::ArtistId.eq(id.clone()))
            .order_by_asc(album::Column::Title)
            .all(self.db.lock().await.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let response = GetArtistDetailsResponse {
            artist: Some(artist.into()),
        };
        Ok(tonic::Response::new(response))
    }
}
