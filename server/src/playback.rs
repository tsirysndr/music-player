use music_player_playback::player::PlayerCommand;
use music_player_tracklist::Tracklist as TracklistState;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

use crate::api::music::v1alpha1::{
    playback_service_server::PlaybackService, GetCurrentlyPlayingSongRequest,
    GetCurrentlyPlayingSongResponse, GetPlaybackStateRequest, GetPlaybackStateResponse,
    GetTimePositionRequest, GetTimePositionResponse, NextRequest, NextResponse, PauseRequest,
    PauseResponse, PlayRequest, PlayResponse, PreviousRequest, PreviousResponse, SeekRequest,
    Levels, SeekResponse, StopRequest, StopResponse, StreamLevelsRequest,
};

pub struct Playback {
    tracklist: Arc<std::sync::Mutex<TracklistState>>,
    cmd_tx: Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>,
}

impl Playback {
    pub fn new(
        tracklist: Arc<std::sync::Mutex<TracklistState>>,
        cmd_tx: Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>,
    ) -> Self {
        Self { tracklist, cmd_tx }
    }
}

#[tonic::async_trait]
impl PlaybackService for Playback {
    type StreamLevelsStream = std::pin::Pin<
        Box<dyn futures_util::Stream<Item = Result<Levels, tonic::Status>> + Send + 'static>,
    >;

    /// Push output levels for a meter.
    ///
    /// A meter wants tens of updates a second, which is no way to poll — and
    /// the now-playing poll this would otherwise ride on runs once a second.
    /// The stream ends when the client drops it.
    async fn stream_levels(
        &self,
        _request: tonic::Request<StreamLevelsRequest>,
    ) -> Result<tonic::Response<Self::StreamLevelsStream>, tonic::Status> {
        // 20 Hz: fast enough to read as a meter, slow enough that it is not
        // the most expensive thing the daemon does.
        const INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);
        let tracklist = Arc::clone(&self.tracklist);

        let stream = async_stream::stream! {
            let mut tick = tokio::time::interval(INTERVAL);
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tick.tick().await;
                let levels = tracklist.lock().unwrap().levels();
                yield Ok(Levels {
                    left: levels.left,
                    right: levels.right,
                    low_left: levels.low_left,
                    low_right: levels.low_right,
                });
            }
        };
        Ok(tonic::Response::new(Box::pin(stream)))
    }

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
        self.cmd_tx
            .lock()
            .unwrap()
            .send(PlayerCommand::Next)
            .unwrap();
        let response = NextResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn previous(
        &self,
        _request: tonic::Request<PreviousRequest>,
    ) -> Result<tonic::Response<PreviousResponse>, tonic::Status> {
        self.cmd_tx
            .lock()
            .unwrap()
            .send(PlayerCommand::Previous)
            .unwrap();
        let response = PreviousResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn play(
        &self,
        _request: tonic::Request<PlayRequest>,
    ) -> Result<tonic::Response<PlayResponse>, tonic::Status> {
        self.cmd_tx
            .lock()
            .unwrap()
            .send(PlayerCommand::Play)
            .unwrap();
        let response = PlayResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn pause(
        &self,
        _request: tonic::Request<PauseRequest>,
    ) -> Result<tonic::Response<PauseResponse>, tonic::Status> {
        self.cmd_tx
            .lock()
            .unwrap()
            .send(PlayerCommand::Pause)
            .unwrap();
        let response = PauseResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn stop(
        &self,
        _request: tonic::Request<StopRequest>,
    ) -> Result<tonic::Response<StopResponse>, tonic::Status> {
        self.cmd_tx
            .lock()
            .unwrap()
            .send(PlayerCommand::Stop)
            .unwrap();
        let response = StopResponse {};
        Ok(tonic::Response::new(response))
    }
    async fn seek(
        &self,
        request: tonic::Request<SeekRequest>,
    ) -> Result<tonic::Response<SeekResponse>, tonic::Status> {
        self.cmd_tx
            .lock()
            .unwrap()
            .send(PlayerCommand::Seek(request.into_inner().position_ms))
            .unwrap();
        let response = SeekResponse {};
        Ok(tonic::Response::new(response))
    }
}
