use music_player_playback::player::PlayerCommand;
use music_player_tracklist::Tracklist as TracklistState;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    api::v1alpha1::{
        playback_service_server::PlaybackService, GetCurrentlyPlayingSongRequest,
        GetCurrentlyPlayingSongResponse, GetPlaybackStateRequest, GetPlaybackStateResponse,
        GetTimePositionRequest, GetTimePositionResponse, NextRequest, NextResponse, PauseRequest,
        PauseResponse, PlayRequest, PlayResponse, PreviousRequest, PreviousResponse, SeekRequest,
        SeekResponse, StopRequest, StopResponse,
    },
    metadata::v1alpha1::{Album, Artist, Track},
};

pub struct Playback {
    tracklist: Arc<std::sync::Mutex<TracklistState>>,
    cmd_tx: UnboundedSender<PlayerCommand>,
}

impl Playback {
    pub fn new(
        tracklist: Arc<std::sync::Mutex<TracklistState>>,
        cmd_tx: UnboundedSender<PlayerCommand>,
    ) -> Self {
        Self { tracklist, cmd_tx }
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
            track: Some(Track {
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
            }),
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
        self.cmd_tx.send(PlayerCommand::Next).unwrap();
        let response = NextResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn previous(
        &self,
        _request: tonic::Request<PreviousRequest>,
    ) -> Result<tonic::Response<PreviousResponse>, tonic::Status> {
        self.cmd_tx.send(PlayerCommand::Previous).unwrap();
        let response = PreviousResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn play(
        &self,
        _request: tonic::Request<PlayRequest>,
    ) -> Result<tonic::Response<PlayResponse>, tonic::Status> {
        self.cmd_tx.send(PlayerCommand::Play).unwrap();
        let response = PlayResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn pause(
        &self,
        _request: tonic::Request<PauseRequest>,
    ) -> Result<tonic::Response<PauseResponse>, tonic::Status> {
        self.cmd_tx.send(PlayerCommand::Pause).unwrap();
        let response = PauseResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn stop(
        &self,
        _request: tonic::Request<StopRequest>,
    ) -> Result<tonic::Response<StopResponse>, tonic::Status> {
        self.cmd_tx.send(PlayerCommand::Stop);
        let response = StopResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn seek(
        &self,
        _request: tonic::Request<SeekRequest>,
    ) -> Result<tonic::Response<SeekResponse>, tonic::Status> {
        self.cmd_tx.send(PlayerCommand::Seek(12)).unwrap();
        let response = SeekResponse {};
        Ok(tonic::Response::new(response))
    }
}
