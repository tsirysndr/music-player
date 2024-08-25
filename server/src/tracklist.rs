use std::sync::Arc;

use extism::{convert::Json, UserData};
use music_player_addons::load_plugin;
use music_player_entity::{album, artist, track};
use music_player_host_fn::state::State;
use music_player_playback::player::PlayerCommand;
use music_player_storage::{repo::track::TrackRepository, Database};
use music_player_tracklist::Tracklist as TracklistState;
use sea_orm::EntityTrait;
use tokio::sync::{mpsc::UnboundedSender, Mutex};

use crate::{
    api::{
        metadata::v1alpha1::Track,
        music::v1alpha1::{
            tracklist_service_server::TracklistService, AddTrackRequest, AddTrackResponse,
            AddTracksRequest, AddTracksResponse, ClearTracklistRequest, ClearTracklistResponse,
            FilterTracklistRequest, FilterTracklistResponse, GetNextTrackRequest,
            GetNextTrackResponse, GetPreviousTrackRequest, GetPreviousTrackResponse,
            GetRandomRequest, GetRandomResponse, GetRepeatRequest, GetRepeatResponse,
            GetSingleRequest, GetSingleResponse, GetTracklistTracksRequest,
            GetTracklistTracksResponse, LoadTracksRequest, LoadTracksResponse, PlayNextRequest,
            PlayNextResponse, PlayTrackAtRequest, PlayTrackAtResponse, RemoveTrackAtRequest,
            RemoveTrackAtResponse, SetRepeatRequest, SetRepeatResponse, ShuffleRequest,
            ShuffleResponse,
        },
    },
    into_tonic_status,
};

pub struct Tracklist {
    state: Arc<std::sync::Mutex<TracklistState>>,
    cmd_tx: Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>,
    db: Database,
    user_data: UserData<State>,
}

impl Tracklist {
    pub fn new(
        state: Arc<std::sync::Mutex<TracklistState>>,
        cmd_tx: Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>,
        db: Database,
        user_data: UserData<State>,
    ) -> Self {
        Self {
            state,
            cmd_tx,
            db,
            user_data,
        }
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
        if result.len() == 0 {
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

        let mut plugin = load_plugin("local", &self.user_data).map_err(into_tonic_status)?;
        plugin
            .call::<Json<Vec<music_player_pdk::types::Track>>, ()>(
                "load_tracks",
                Json(vec![track.into()]),
            )
            .map_err(into_tonic_status)?;

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
        let mut plugin = load_plugin("local", &self.user_data).map_err(into_tonic_status)?;
        plugin
            .call::<&str, ()>("clear", "")
            .map_err(into_tonic_status)?;
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
        let mut plugin = load_plugin("local", &self.user_data).map_err(into_tonic_status)?;
        plugin
            .call::<u32, ()>("remove_track", request.position)
            .map_err(into_tonic_status)?;
        let response = RemoveTrackAtResponse {};
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

        let track: music_player_entity::track::Model = track.into();
        let mut plugin = load_plugin("local", &self.user_data).map_err(into_tonic_status)?;
        plugin
            .call::<Json<music_player_pdk::types::Track>, ()>("play_next", Json(track.into()))
            .map_err(into_tonic_status)?;

        let response = PlayNextResponse {};
        Ok(tonic::Response::new(response))
    }

    async fn play_track_at(
        &self,
        request: tonic::Request<PlayTrackAtRequest>,
    ) -> Result<tonic::Response<PlayTrackAtResponse>, tonic::Status> {
        let request = request.into_inner();

        let mut plugin = load_plugin("local", &self.user_data).map_err(into_tonic_status)?;
        plugin
            .call::<u32, ()>("play_track_at", request.index)
            .map_err(into_tonic_status)?;

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

        let mut plugin = load_plugin("local", &self.user_data).map_err(into_tonic_status)?;
        plugin
            .call::<&str, ()>("stop", "")
            .map_err(into_tonic_status)?;
        plugin
            .call::<&str, ()>("clear", "")
            .map_err(into_tonic_status)?;
        plugin
            .call::<Json<Vec<music_player_pdk::types::Track>>, ()>(
                "load_tracks",
                Json(tracks.into_iter().map(Into::into).collect()),
            )
            .map_err(into_tonic_status)?;
        plugin
            .call::<u32, ()>("play_track_at", start_index as u32)
            .map_err(into_tonic_status)?;

        let response = LoadTracksResponse {};
        Ok(tonic::Response::new(response))
    }
}
