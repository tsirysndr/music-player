use futures::future::FutureExt;
use music_player_entity::{album, artist, track};
use music_player_scanner::scan_directory;
use music_player_storage::Database;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QueryOrder,
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
                let id = format!("{:x}", md5::compute(song.artist.to_string()));
                let item = artist::ActiveModel {
                    id: ActiveValue::set(id),
                    name: ActiveValue::Set(song.artist.clone()),
                };
                match item.insert(db.get_connection()).await {
                    Ok(_) => (),
                    Err(_) => (),
                }

                let id = format!(
                    "{:x}",
                    md5::compute(format!("{}{}", song.album, song.artist))
                );
                let item = album::ActiveModel {
                    id: ActiveValue::set(id),
                    title: ActiveValue::Set(song.album.clone()),
                    artist: ActiveValue::Set(song.artist.clone()),
                    artist_id: ActiveValue::Set(Some(format!(
                        "{:x}",
                        md5::compute(song.artist.to_string())
                    ))),
                    year: ActiveValue::Set(song.year),
                };
                match item.insert(db.get_connection()).await {
                    Ok(_) => (),
                    Err(_) => (),
                }
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
                    duration: ActiveValue::Set(Some(song.duration.as_secs_f32())),
                    uri: ActiveValue::Set(song.uri.clone().unwrap_or_default()),
                    album_id: ActiveValue::Set(Some(format!(
                        "{:x}",
                        md5::compute(format!("{}{}", song.album, song.artist))
                    ))),
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
        let results = artist::Entity::find()
            .order_by_asc(artist::Column::Name)
            .all(self.db.lock().await.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let response = GetArtistsResponse {
            artists: results
                .into_iter()
                .map(|artist| Artist {
                    id: artist.id,
                    name: artist.name,
                    ..Default::default()
                })
                .collect(),
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
            albums: results
                .into_iter()
                .map(|album| Album {
                    id: album.id,
                    title: album.title,
                    artist: album.artist,
                    year: album.year.unwrap_or_default() as i32,
                    ..Default::default()
                })
                .collect(),
        };
        Ok(tonic::Response::new(response))
    }

    async fn get_tracks(
        &self,
        _request: tonic::Request<GetTracksRequest>,
    ) -> Result<tonic::Response<GetTracksResponse>, tonic::Status> {
        let results = track::Entity::find()
            .order_by_asc(track::Column::Title)
            .all(self.db.lock().await.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let response = GetTracksResponse {
            tracks: results
                .into_iter()
                .map(|track| Track {
                    id: track.id,
                    title: track.title,
                    uri: track.uri,
                    duration: track.duration.unwrap_or_default(),
                    track_number: i32::try_from(track.track.unwrap_or_default()).unwrap(),
                    artists: vec![Artist {
                        name: track.artist,
                        ..Default::default()
                    }],
                    album: Some(Album {
                        id: track.album_id.unwrap(),
                        title: track.album,
                        year: i32::try_from(track.year.unwrap_or(0)).unwrap(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .collect(),
        };
        Ok(tonic::Response::new(response))
    }

    async fn get_track_details(
        &self,
        request: tonic::Request<GetTrackDetailsRequest>,
    ) -> Result<tonic::Response<GetTrackDetailsResponse>, tonic::Status> {
        let result = track::Entity::find_by_id(request.into_inner().id)
            .one(self.db.lock().await.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        if result.is_none() {
            return Err(tonic::Status::not_found("Track not found"));
        }
        let track = result.unwrap();
        let response = GetTrackDetailsResponse {
            track: Some(Track {
                id: track.id,
                title: track.title,
                uri: track.uri,
                duration: track.duration.unwrap_or_default(),
                track_number: i32::try_from(track.track.unwrap_or_default()).unwrap(),
                artists: vec![Artist {
                    name: track.artist,
                    ..Default::default()
                }],
                album: Some(Album {
                    id: track.album_id.unwrap(),
                    title: track.album,
                    year: i32::try_from(track.year.unwrap_or(0)).unwrap(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        };
        Ok(tonic::Response::new(response))
    }

    async fn get_album_details(
        &self,
        request: tonic::Request<GetAlbumDetailsRequest>,
    ) -> Result<tonic::Response<GetAlbumDetailsResponse>, tonic::Status> {
        let result = album::Entity::find_by_id(request.into_inner().id)
            .one(self.db.lock().await.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        if result.is_none() {
            return Err(tonic::Status::not_found("Album not found"));
        }
        let album = result.unwrap();
        let tracks = album
            .find_related(track::Entity)
            .order_by_asc(track::Column::Track)
            .all(self.db.lock().await.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let album = Some(Album {
            id: album.id,
            title: album.title,
            tracks: tracks
                .into_iter()
                .map(|track| Song {
                    id: track.id,
                    title: track.title,
                    track_number: i32::try_from(track.track.unwrap_or_default()).unwrap(),
                    duration: track.duration.unwrap_or_default(),
                    artists: vec![SongArtist {
                        name: track.artist,
                        ..Default::default()
                    }],
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        });
        let response = GetAlbumDetailsResponse { album };
        Ok(tonic::Response::new(response))
    }

    async fn get_artist_details(
        &self,
        request: tonic::Request<GetArtistDetailsRequest>,
    ) -> Result<tonic::Response<GetArtistDetailsResponse>, tonic::Status> {
        let result = artist::Entity::find_by_id(request.into_inner().id)
            .one(self.db.lock().await.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        if result.is_none() {
            return Err(tonic::Status::not_found("Artist not found"));
        }

        let artist = result.unwrap();
        let name = artist.name.clone();
        let tracks = track::Entity::find()
            .filter(track::Column::Artist.eq(artist.name.to_owned()))
            .order_by_asc(track::Column::Title)
            .all(self.db.lock().await.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let response = GetArtistDetailsResponse {
            artist: Some(Artist {
                id: artist.id,
                name: artist.name.to_owned(),
                songs: tracks
                    .into_iter()
                    .map(|track| ArtistSong {
                        id: track.id,
                        title: track.title,
                        track_number: i32::try_from(track.track.unwrap_or_default()).unwrap(),
                        duration: track.duration.unwrap_or_default(),
                        artists: vec![Artist {
                            name: name.to_owned(),
                            ..Default::default()
                        }],
                        album: Some(Album {
                            id: track.album_id.unwrap(),
                            title: track.album,
                            year: i32::try_from(track.year.unwrap_or(0)).unwrap(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            }),
        };
        Ok(tonic::Response::new(response))
    }
}
