use std::sync::Arc;

use music_player_entity::track;
use music_player_playback::player::{Player, PlayerEngine};
use music_player_storage::Database;
use sea_orm::EntityTrait;
use tokio::sync::Mutex;

use crate::{
    api::v1alpha1::{
        tracklist_service_server::TracklistService, AddTrackRequest, AddTrackResponse,
        AddTracksRequest, AddTracksResponse, ClearTracklistRequest, ClearTracklistResponse,
        FilterTracklistRequest, FilterTracklistResponse, GetNextTrackRequest, GetNextTrackResponse,
        GetPreviousTrackRequest, GetPreviousTrackResponse, GetRandomRequest, GetRandomResponse,
        GetRepeatRequest, GetRepeatResponse, GetSingleRequest, GetSingleResponse,
        GetTracklistTracksRequest, GetTracklistTracksResponse, PlayNextRequest, PlayNextResponse,
        PlayTrackAtRequest, PlayTrackAtResponse, RemoveTrackRequest, RemoveTrackResponse,
        SetRepeatRequest, SetRepeatResponse, ShuffleRequest, ShuffleResponse,
    },
    metadata::v1alpha1::{Album, Artist, Track},
};

pub struct Tracklist {
    player: Arc<Mutex<Player>>,
    db: Arc<Mutex<Database>>,
}

impl Tracklist {
    pub fn new(player: Arc<Mutex<Player>>, db: Arc<Mutex<Database>>) -> Self {
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
        let result = track::Entity::find_by_id(song.clone().id)
            .one(self.db.lock().await.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        if result.is_none() {
            return Err(tonic::Status::not_found("Track not found"));
        }
        let track = result.unwrap();
        self.player.lock().await.load_tracklist(vec![track]);
        let response = AddTrackResponse {};
        Ok(tonic::Response::new(response))
    }

    async fn add_tracks(
        &self,
        _req: tonic::Request<AddTracksRequest>,
    ) -> Result<tonic::Response<AddTracksResponse>, tonic::Status> {
        unimplemented!()
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
        let response = GetNextTrackResponse {
            track: Some(Track {
                ..Default::default()
            }),
        };
        Ok(tonic::Response::new(response))
    }

    async fn get_previous_track(
        &self,
        _request: tonic::Request<GetPreviousTrackRequest>,
    ) -> Result<tonic::Response<GetPreviousTrackResponse>, tonic::Status> {
        let response = GetPreviousTrackResponse {
            track: Some(Track {
                ..Default::default()
            }),
        };
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
                    disc_number: i32::try_from(track.track.unwrap_or(0)).unwrap(),
                    artists: vec![Artist {
                        name: track.artist,
                        ..Default::default()
                    }],
                    album: Some(Album {
                        // id: track.album_id.unwrap(),
                        title: track.album,
                        year: i32::try_from(track.year.unwrap_or(0)).unwrap(),
                        ..Default::default()
                    }),
                    duration: track.duration.unwrap_or(0.0),
                    ..Default::default()
                })
                .collect(),
            previous_tracks: previous_tracks
                .into_iter()
                .map(|track| Track {
                    id: track.id,
                    title: track.title,
                    uri: track.uri,
                    disc_number: i32::try_from(track.track.unwrap_or(0)).unwrap(),
                    artists: vec![Artist {
                        name: track.artist,
                        ..Default::default()
                    }],
                    album: Some(Album {
                        // id: track.album_id.unwrap(),
                        title: track.album,
                        year: i32::try_from(track.year.unwrap_or(0)).unwrap(),
                        ..Default::default()
                    }),
                    duration: track.duration.unwrap_or(0.0),
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

    async fn play_track_at(
        &self,
        request: tonic::Request<PlayTrackAtRequest>,
    ) -> Result<tonic::Response<PlayTrackAtResponse>, tonic::Status> {
        let request = request.into_inner();
        self.player
            .lock()
            .await
            .play_track_at(request.index as usize);
        let response = PlayTrackAtResponse {};
        Ok(tonic::Response::new(response))
    }
}
