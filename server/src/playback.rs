use std::sync::Arc;

use music_player_playback::player::{Player, PlayerEngine};
use tokio::sync::Mutex;

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
    player: Arc<Mutex<Player>>,
}

impl Playback {
    pub fn new(player: Arc<Mutex<Player>>) -> Self {
        Self { player }
    }
}

#[tonic::async_trait]
impl PlaybackService for Playback {
    async fn get_currently_playing_song(
        &self,
        _request: tonic::Request<GetCurrentlyPlayingSongRequest>,
    ) -> Result<tonic::Response<GetCurrentlyPlayingSongResponse>, tonic::Status> {
        let player = self.player.lock().await;
        let track = player.get_current_track().await;

        if track.is_none() {
            let response = GetCurrentlyPlayingSongResponse {
                track: None,
                is_playing: false,
            };
            return Ok(tonic::Response::new(response));
        }

        let (track, is_playing) = track.unwrap();
        if track.is_none() {
            let response = GetCurrentlyPlayingSongResponse {
                track: None,
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
                ..Default::default()
            }),
            is_playing,
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
        self.player.lock().await.next();
        let response = NextResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn previous(
        &self,
        _request: tonic::Request<PreviousRequest>,
    ) -> Result<tonic::Response<PreviousResponse>, tonic::Status> {
        self.player.lock().await.previous();
        let response = PreviousResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn play(
        &self,
        _request: tonic::Request<PlayRequest>,
    ) -> Result<tonic::Response<PlayResponse>, tonic::Status> {
        self.player.lock().await.play();
        let response = PlayResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn pause(
        &self,
        _request: tonic::Request<PauseRequest>,
    ) -> Result<tonic::Response<PauseResponse>, tonic::Status> {
        self.player.lock().await.pause();
        let response = PauseResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn stop(
        &self,
        _request: tonic::Request<StopRequest>,
    ) -> Result<tonic::Response<StopResponse>, tonic::Status> {
        self.player.lock().await.stop();
        let response = StopResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn seek(
        &self,
        _request: tonic::Request<SeekRequest>,
    ) -> Result<tonic::Response<SeekResponse>, tonic::Status> {
        self.player.lock().await.seek(12);
        let response = SeekResponse {};
        Ok(tonic::Response::new(response))
    }
}
