use extism::UserData;
use music_player_addons::load_plugin;
use music_player_host_fn::state::State;
use music_player_playback::player::PlayerCommand;
use music_player_tracklist::Tracklist as TracklistState;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    api::{
        metadata::v1alpha1::{Album, Artist, Track},
        music::v1alpha1::{
            playback_service_server::PlaybackService, GetCurrentlyPlayingSongRequest,
            GetCurrentlyPlayingSongResponse, GetPlaybackStateRequest, GetPlaybackStateResponse,
            GetTimePositionRequest, GetTimePositionResponse, NextRequest, NextResponse,
            PauseRequest, PauseResponse, PlayRequest, PlayResponse, PreviousRequest,
            PreviousResponse, SeekRequest, SeekResponse, StopRequest, StopResponse,
        },
    },
    into_tonic_status,
};

pub struct Playback {
    tracklist: Arc<std::sync::Mutex<TracklistState>>,
    cmd_tx: Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>,
    user_data: UserData<State>,
}

impl Playback {
    pub fn new(
        tracklist: Arc<std::sync::Mutex<TracklistState>>,
        cmd_tx: Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>,
        user_data: UserData<State>,
    ) -> Self {
        Self {
            tracklist,
            cmd_tx,
            user_data,
        }
    }
}

#[tonic::async_trait]
impl PlaybackService for Playback {
    async fn get_currently_playing_song(
        &self,
        _request: tonic::Request<GetCurrentlyPlayingSongRequest>,
    ) -> Result<tonic::Response<GetCurrentlyPlayingSongResponse>, tonic::Status> {
        let (track, index) = self.tracklist.lock().unwrap().current_track();
        let playback_state = self.tracklist.lock().unwrap().playback_state();

        if track.is_none() {
            let response = GetCurrentlyPlayingSongResponse {
                track: None,
                index: 0,
                position_ms: 0,
                is_playing: false,
            };
            return Ok(tonic::Response::new(response));
        }

        if track.is_none() {
            let response = GetCurrentlyPlayingSongResponse {
                track: None,
                index: 0,
                position_ms: 0,
                is_playing: false,
            };
            return Ok(tonic::Response::new(response));
        }

        let track = track.unwrap();
        let response = GetCurrentlyPlayingSongResponse {
            track: Some(track.into()),
            index: index as u32,
            position_ms: playback_state.position_ms,
            is_playing: playback_state.is_playing,
        };
        Ok(tonic::Response::new(response))
    }
    async fn get_playback_state(
        &self,
        _request: tonic::Request<GetPlaybackStateRequest>,
    ) -> Result<tonic::Response<GetPlaybackStateResponse>, tonic::Status> {
        let response = GetPlaybackStateResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn get_time_position(
        &self,
        _request: tonic::Request<GetTimePositionRequest>,
    ) -> Result<tonic::Response<GetTimePositionResponse>, tonic::Status> {
        let response = GetTimePositionResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn next(
        &self,
        _request: tonic::Request<NextRequest>,
    ) -> Result<tonic::Response<NextResponse>, tonic::Status> {
        let mut plugin = load_plugin("local", &self.user_data).map_err(into_tonic_status)?;
        plugin
            .call::<&str, ()>("next", "")
            .map_err(into_tonic_status)?;
        let response = NextResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn previous(
        &self,
        _request: tonic::Request<PreviousRequest>,
    ) -> Result<tonic::Response<PreviousResponse>, tonic::Status> {
        let mut plugin = load_plugin("local", &self.user_data).map_err(into_tonic_status)?;
        plugin
            .call::<&str, ()>("previous", "")
            .map_err(into_tonic_status)?;
        let response = PreviousResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn play(
        &self,
        _request: tonic::Request<PlayRequest>,
    ) -> Result<tonic::Response<PlayResponse>, tonic::Status> {
        let mut plugin = load_plugin("local", &self.user_data).map_err(into_tonic_status)?;
        plugin
            .call::<&str, ()>("play", "")
            .map_err(into_tonic_status)?;
        let response = PlayResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn pause(
        &self,
        _request: tonic::Request<PauseRequest>,
    ) -> Result<tonic::Response<PauseResponse>, tonic::Status> {
        let mut plugin = load_plugin("local", &self.user_data).map_err(into_tonic_status)?;
        plugin
            .call::<&str, ()>("pause", "")
            .map_err(into_tonic_status)?;
        let response = PauseResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn stop(
        &self,
        _request: tonic::Request<StopRequest>,
    ) -> Result<tonic::Response<StopResponse>, tonic::Status> {
        let mut plugin = load_plugin("local", &self.user_data).map_err(into_tonic_status)?;
        plugin
            .call::<&str, ()>("stop", "")
            .map_err(into_tonic_status)?;
        let response = StopResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn seek(
        &self,
        request: tonic::Request<SeekRequest>,
    ) -> Result<tonic::Response<SeekResponse>, tonic::Status> {
        let mut plugin = load_plugin("local", &self.user_data).map_err(into_tonic_status)?;
        plugin
            .call::<u32, ()>("seek", 0)
            .map_err(into_tonic_status)?;
        let response = SeekResponse {};
        Ok(tonic::Response::new(response))
    }
}
