use std::sync::Arc;

use music_player_entity::track;
use music_player_playback::player::{Player, PlayerEngine};
use music_player_storage::Database;
use tokio::sync::Mutex;

use crate::{
    api::v1alpha1::{
        tracklist_service_server::TracklistService, AddTrackRequest, AddTrackResponse,
        ClearTracklistRequest, ClearTracklistResponse, FilterTracklistRequest,
        FilterTracklistResponse, GetNextTrackRequest, GetNextTrackResponse,
        GetPreviousTrackRequest, GetPreviousTrackResponse, GetRandomRequest, GetRandomResponse,
        GetRepeatRequest, GetRepeatResponse, GetSingleRequest, GetSingleResponse,
        GetTracklistTracksRequest, GetTracklistTracksResponse, PlayNextRequest, PlayNextResponse,
        RemoveTrackRequest, RemoveTrackResponse, SetRepeatRequest, SetRepeatResponse,
        ShuffleRequest, ShuffleResponse,
    },
    metadata::v1alpha1::{Album, Artist, Track},
};

pub struct Tracklist {
    player: Arc<Mutex<Player>>,
    db: Arc<Database>,
}

impl Tracklist {
    pub fn new(player: Arc<Mutex<Player>>, db: Arc<Database>) -> Self {
        Self { player, db }
    }
}

#[tonic::async_trait]
impl TracklistService for Tracklist {
    async fn add_track(
        &self,
        request: tonic::Request<AddTrackRequest>,
    ) -> Result<tonic::Response<AddTrackResponse>, tonic::Status> {
        let song = request.get_ref().track.as_ref().unwrap();
        let artist = match song.artists.len() {
            0 => "Unknown Artist".to_string(),
            _ => song.artists[0].name.clone(),
        };
        self.player.lock().await.load_tracklist(vec![track::Model {
            id: song.id.clone(),
            uri: song.uri.clone(),
            title: song.title.clone(),
            album: song.album.clone().unwrap_or_default().title,
            artist,
            ..Default::default()
        }]);
        let response = AddTrackResponse {};
        Ok(tonic::Response::new(response))
    }

    async fn clear_tracklist(
        &self,
        _request: tonic::Request<ClearTracklistRequest>,
    ) -> Result<tonic::Response<ClearTracklistResponse>, tonic::Status> {
        self.player.lock().await.clear();
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
        let (previous_tracks, next_tracks) = self.player.lock().await.get_tracks().await;

        let response = GetTracklistTracksResponse {
            next_tracks: next_tracks
                .into_iter()
                .map(|track| Track {
                    id: track.id,
                    title: track.title,
                    uri: track.uri,
                    disc_number: format!("{}", track.track.unwrap_or(0)).parse().unwrap(),
                    artists: vec![Artist {
                        name: track.artist,
                        ..Default::default()
                    }],
                    album: Some(Album {
                        // id: track.album_id.unwrap(),
                        title: track.album,
                        year: format!("{}", track.year.unwrap_or(0)).parse().unwrap(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .collect(),
            previous_tracks: previous_tracks
                .into_iter()
                .map(|track| Track {
                    id: track.id,
                    title: track.title,
                    uri: track.uri,
                    disc_number: format!("{}", track.track.unwrap_or(0)).parse().unwrap(),
                    artists: vec![Artist {
                        name: track.artist,
                        ..Default::default()
                    }],
                    album: Some(Album {
                        // id: track.album_id.unwrap(),
                        title: track.album,
                        year: format!("{}", track.year.unwrap_or(0)).parse().unwrap(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .collect(),
        };
        Ok(tonic::Response::new(response))
    }

    async fn play_next(
        &self,
        _request: tonic::Request<PlayNextRequest>,
    ) -> Result<tonic::Response<PlayNextResponse>, tonic::Status> {
        let response = PlayNextResponse {};
        Ok(tonic::Response::new(response))
    }
}
