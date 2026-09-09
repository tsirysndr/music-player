use std::sync::Arc;

use music_player_entity::{album, artist, track};
use music_player_playback::player::PlayerCommand;
use music_player_storage::Database;
use music_player_tracklist::Tracklist as TracklistState;
use sea_orm::EntityTrait;
use tokio::sync::mpsc::UnboundedSender;

use crate::api::{
    metadata::v1alpha1::Track,
    music::v1alpha1::{
        tracklist_service_server::TracklistService, AddTrackRequest, AddTrackResponse,
        AddTracksRequest, AddTracksResponse, ClearTracklistRequest, ClearTracklistResponse,
        FilterTracklistRequest, FilterTracklistResponse, GetNextTrackRequest, GetNextTrackResponse,
        GetPreviousTrackRequest, GetPreviousTrackResponse, GetRandomRequest, GetRandomResponse,
        GetRepeatRequest, GetRepeatResponse, GetSingleRequest, GetSingleResponse,
        GetTracklistTracksRequest, GetTracklistTracksResponse, LoadTracksRequest,
        LoadTracksResponse, PlayNextRequest, PlayNextResponse, PlayTrackAtRequest,
        PlayTrackAtResponse, RemoveTrackAtRequest, RemoveTrackAtResponse, SetRepeatRequest,
        SetRepeatResponse, ShuffleRequest, ShuffleResponse,
    },
};

pub struct Tracklist {
    state: Arc<std::sync::Mutex<TracklistState>>,
    cmd_tx: Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>,
    db: Database,
}

impl Tracklist {
    pub fn new(
        state: Arc<std::sync::Mutex<TracklistState>>,
        cmd_tx: Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>,
        db: Database,
    ) -> Self {
        Self { state, cmd_tx, db }
    }
}

#[tonic::async_trait]
impl TracklistService for Tracklist {
    async fn add_track(
        &self,
        request: tonic::Request<AddTrackRequest>,
    ) -> Result<tonic::Response<AddTrackResponse>, tonic::Status> {
        let song = request.get_ref().track.as_ref().unwrap();
        let id = song.clone().id;

        let result: Vec<(track::Model, Vec<artist::Model>)> = track::Entity::find_by_id(id.clone())
            .find_with_related(artist::Entity)
            .all(self.db.get_connection())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        if result.is_empty() {
            return Err(tonic::Status::not_found("Track not found"));
        }

        let (mut track, artists) = result.into_iter().next().unwrap();
        track.artists = artists;

        let result: Vec<(track::Model, Option<album::Model>)> =
            track::Entity::find_by_id(id.clone())
                .find_also_related(album::Entity)
                .all(self.db.get_connection())
                .await
                .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let (_, album) = result.into_iter().next().unwrap();
        track.album = album.unwrap();

        self.cmd_tx
            .lock()
            .unwrap()
            .send(PlayerCommand::LoadTracklist {
                tracks: vec![track],
                // Appending one track: wherever playback is, it stays.
                start_index: None,
            })
            .unwrap();
        let response = AddTrackResponse {};
        Ok(tonic::Response::new(response))
    }

    /// Append tracks to the queue.
    ///
    /// Takes whole tracks rather than ids, unlike `AddTrack`. Ids would have to
    /// be looked up here, against a local table that holds nothing when a
    /// remote provider is connected — so the caller sends what it already has
    /// from the listing it is adding from, and this works whatever the library
    /// is.
    async fn add_tracks(
        &self,
        request: tonic::Request<AddTracksRequest>,
    ) -> Result<tonic::Response<AddTracksResponse>, tonic::Status> {
        let tracks = request
            .into_inner()
            .tracks
            .into_iter()
            .map(Into::into)
            .collect::<Vec<track::Model>>();

        if !tracks.is_empty() {
            self.cmd_tx
                .lock()
                .unwrap()
                .send(PlayerCommand::LoadTracklist {
                    tracks,
                    // Appending: wherever playback is, it stays there.
                    start_index: None,
                })
                .map_err(|e| tonic::Status::internal(e.to_string()))?;
        }

        Ok(tonic::Response::new(AddTracksResponse {}))
    }

    async fn clear_tracklist(
        &self,
        _request: tonic::Request<ClearTracklistRequest>,
    ) -> Result<tonic::Response<ClearTracklistResponse>, tonic::Status> {
        self.cmd_tx.lock().unwrap().send(PlayerCommand::Clear).ok();
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

    async fn remove_track_at(
        &self,
        request: tonic::Request<RemoveTrackAtRequest>,
    ) -> Result<tonic::Response<RemoveTrackAtResponse>, tonic::Status> {
        let request = request.into_inner();
        self.cmd_tx
            .lock()
            .unwrap()
            .send(PlayerCommand::RemoveTrack(request.position as usize))
            .unwrap();
        let response = RemoveTrackAtResponse {};
        Ok(tonic::Response::new(response))
    }

    async fn shuffle(
        &self,
        request: tonic::Request<ShuffleRequest>,
    ) -> Result<tonic::Response<ShuffleResponse>, tonic::Status> {
        let enabled = request.into_inner().enabled;
        self.cmd_tx
            .lock()
            .unwrap()
            .send(PlayerCommand::SetShuffle(enabled))
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let response = ShuffleResponse {};
        Ok(tonic::Response::new(response))
    }

    async fn set_repeat(
        &self,
        request: tonic::Request<SetRepeatRequest>,
    ) -> Result<tonic::Response<SetRepeatResponse>, tonic::Status> {
        let mode = request.into_inner().mode;
        self.cmd_tx
            .lock()
            .unwrap()
            .send(PlayerCommand::SetRepeat(mode))
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let response = SetRepeatResponse {};
        Ok(tonic::Response::new(response))
    }

    async fn get_tracklist_tracks(
        &self,
        _request: tonic::Request<GetTracklistTracksRequest>,
    ) -> Result<tonic::Response<GetTracklistTracksResponse>, tonic::Status> {
        let (previous_tracks, next_tracks) = self.state.lock().unwrap().tracks();

        let response = GetTracklistTracksResponse {
            next_tracks: next_tracks.into_iter().map(Into::into).collect(),
            previous_tracks: previous_tracks.into_iter().map(Into::into).collect(),
        };
        Ok(tonic::Response::new(response))
    }

    async fn play_next(
        &self,
        request: tonic::Request<PlayNextRequest>,
    ) -> Result<tonic::Response<PlayNextResponse>, tonic::Status> {
        let track = request.into_inner().track;
        if track.is_none() {
            return Err(tonic::Status::invalid_argument("Track is required"));
        }

        let track = track.unwrap();

        if track.uri.is_empty() {
            return Err(tonic::Status::invalid_argument("Track URI is required"));
        }

        let track = track.into();

        self.cmd_tx
            .lock()
            .unwrap()
            .send(PlayerCommand::PlayNext(track))
            .unwrap();
        let response = PlayNextResponse {};
        Ok(tonic::Response::new(response))
    }

    async fn play_track_at(
        &self,
        request: tonic::Request<PlayTrackAtRequest>,
    ) -> Result<tonic::Response<PlayTrackAtResponse>, tonic::Status> {
        let request = request.into_inner();
        self.cmd_tx
            .lock()
            .unwrap()
            .send(PlayerCommand::PlayTrackAt(request.index as usize))
            .unwrap();
        let response = PlayTrackAtResponse {};
        Ok(tonic::Response::new(response))
    }

    async fn load_tracks(
        &self,
        request: tonic::Request<LoadTracksRequest>,
    ) -> Result<tonic::Response<LoadTracksResponse>, tonic::Status> {
        let request = request.into_inner();
        let tracks = request
            .tracks
            .into_iter()
            .map(Into::into)
            .collect::<Vec<track::Model>>();
        let start_index = request.start_index as usize;

        self.cmd_tx
            .lock()
            .unwrap()
            .send(PlayerCommand::Stop)
            .unwrap();
        self.cmd_tx
            .lock()
            .unwrap()
            .send(PlayerCommand::Clear)
            .unwrap();
        // One command, so the engine opens the wanted track's stream once.
        // This used to load the tracklist — which starts the first track — and
        // then ask for the wanted index, opening two remote streams for every
        // play.
        self.cmd_tx
            .lock()
            .unwrap()
            .send(PlayerCommand::LoadTracklist {
                tracks,
                start_index: Some(start_index),
            })
            .unwrap();
        let response = LoadTracksResponse {};
        Ok(tonic::Response::new(response))
    }
}
