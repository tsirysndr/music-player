use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::simple_broker::SimpleBroker;
use async_graphql::*;
use futures_util::Stream;
use music_player_playback::player::PlayerCommand;
use music_player_renderer::CurrentReceiverDevice;
use music_player_tracklist::Tracklist;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex as TokioMutex;

use super::objects::{
    current_track::CurrentlyPlayingSong, player_state::PlayerState, track::Track,
};

#[derive(Default)]
pub struct PlaybackQuery;

#[Object]
impl PlaybackQuery {
    async fn currently_playing_song(
        &self,
        ctx: &Context<'_>,
    ) -> Result<CurrentlyPlayingSong, Error> {
        let current_device = ctx
            .data::<Arc<TokioMutex<CurrentReceiverDevice>>>()
            .unwrap();
        let mut device = current_device.lock().await;

        if device.client.is_some() {
            let receiver = device.client.as_mut().unwrap();
            let playback = receiver.get_current_playback().await?;
            return Ok(playback.into());
        }

        let tracklist = ctx.data::<Arc<Mutex<Tracklist>>>().unwrap();
        let (track, index) = tracklist.lock().unwrap().current_track();
        let playback_state = tracklist.lock().unwrap().playback_state();

        if track.is_none() {
            let response = CurrentlyPlayingSong {
                track: None,
                index: 0,
                position_ms: 0,
                is_playing: false,
            };
            return Ok(response);
        }

        let track = track.unwrap();

        Ok(CurrentlyPlayingSong {
            track: Some(track.into()),
            index: index as u32,
            position_ms: playback_state.position_ms,
            is_playing: playback_state.is_playing,
        })
    }

    async fn get_player_state(&self, ctx: &Context<'_>) -> PlayerState {
        let tracklist = ctx.data::<Arc<Mutex<Tracklist>>>().unwrap();
        let tracklist = tracklist.lock().unwrap();
        let (_, index) = tracklist.current_track();
        let playback_state = tracklist.playback_state();
        PlayerState {
            index: index as u32,
            position_ms: playback_state.position_ms,
            is_playing: playback_state.is_playing,
        }
    }
}

#[derive(Default)]
pub struct PlaybackMutation;

#[Object]
impl PlaybackMutation {
    async fn next(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        let current_device = ctx
            .data::<Arc<TokioMutex<CurrentReceiverDevice>>>()
            .unwrap();
        let mut device = current_device.lock().await;

        if device.client.is_some() {
            let receiver = device.client.as_mut().unwrap();
            receiver.next().await?;
            return Ok(true);
        }

        let player_cmd = ctx
            .data::<Arc<Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        player_cmd
            .lock()
            .unwrap()
            .send(PlayerCommand::Next)
            .unwrap();
        Ok(true)
    }

    async fn play(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        let current_device = ctx
            .data::<Arc<TokioMutex<CurrentReceiverDevice>>>()
            .unwrap();
        let mut device = current_device.lock().await;

        if device.client.is_some() {
            let receiver = device.client.as_mut().unwrap();
            receiver.play().await?;
            return Ok(true);
        }

        let player_cmd = ctx
            .data::<Arc<Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        player_cmd
            .lock()
            .unwrap()
            .send(PlayerCommand::Play)
            .unwrap();
        Ok(true)
    }

    async fn pause(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        let current_device = ctx
            .data::<Arc<TokioMutex<CurrentReceiverDevice>>>()
            .unwrap();
        let mut device = current_device.lock().await;

        if device.client.is_some() {
            let receiver = device.client.as_mut().unwrap();
            receiver.pause().await?;
            return Ok(true);
        }

        let player_cmd = ctx
            .data::<Arc<Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        player_cmd
            .lock()
            .unwrap()
            .send(PlayerCommand::Pause)
            .unwrap();
        Ok(true)
    }

    async fn previous(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        let current_device = ctx
            .data::<Arc<TokioMutex<CurrentReceiverDevice>>>()
            .unwrap();
        let mut device = current_device.lock().await;

        if device.client.is_some() {
            let receiver = device.client.as_mut().unwrap();
            receiver.previous().await?;
            return Ok(true);
        }

        let player_cmd = ctx
            .data::<Arc<Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        player_cmd
            .lock()
            .unwrap()
            .send(PlayerCommand::Previous)
            .unwrap();
        Ok(true)
    }

    async fn seek(&self, ctx: &Context<'_>, position: u32) -> Result<bool, Error> {
        let current_device = ctx
            .data::<Arc<TokioMutex<CurrentReceiverDevice>>>()
            .unwrap();
        let mut device = current_device.lock().await;

        if device.client.is_some() {
            let receiver = device.client.as_mut().unwrap();
            receiver.seek(position).await?;
            return Ok(true);
        }

        let player_cmd = ctx
            .data::<Arc<Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        player_cmd
            .lock()
            .unwrap()
            .send(PlayerCommand::Seek(position))
            .unwrap();
        Ok(true)
    }

    async fn stop(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        let current_device = ctx
            .data::<Arc<TokioMutex<CurrentReceiverDevice>>>()
            .unwrap();
        let mut device = current_device.lock().await;

        if device.client.is_some() {
            let receiver = device.client.as_mut().unwrap();
            receiver.stop().await?;
            return Ok(true);
        }

        let player_cmd = ctx
            .data::<Arc<Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        player_cmd
            .lock()
            .unwrap()
            .send(PlayerCommand::Stop)
            .unwrap();
        Ok(true)
    }
}

#[derive(Clone)]
pub struct PositionMilliseconds {
    pub position_ms: u32,
}

#[Object]
impl PositionMilliseconds {
    async fn position_ms(&self) -> u32 {
        self.position_ms
    }
}

#[derive(Default)]
pub struct PlaybackSubscription;

#[Subscription]
impl PlaybackSubscription {
    async fn player_state(&self) -> impl Stream<Item = PlayerState> {
        SimpleBroker::<PlayerState>::subscribe()
    }

    async fn currently_playing_song(&self) -> impl Stream<Item = Track> {
        SimpleBroker::<Track>::subscribe()
    }

    async fn track_time_position(&self) -> impl Stream<Item = PositionMilliseconds> {
        SimpleBroker::<PositionMilliseconds>::subscribe()
    }

    /// Output levels for a meter, pushed at 20 Hz.
    ///
    /// A stream rather than a field: a meter wants tens of updates a second,
    /// and the poll a client would otherwise ride on is far slower. Read from
    /// the tracklist, which the player writes each tick, so this adds no work
    /// to the audio path.
    async fn levels(&self, ctx: &Context<'_>) -> impl Stream<Item = Levels> {
        const INTERVAL: Duration = Duration::from_millis(50);
        let tracklist = Arc::clone(
            ctx.data::<Arc<std::sync::Mutex<Tracklist>>>()
                .expect("the tracklist is registered on the schema"),
        );
        async_stream::stream! {
            let mut tick = tokio::time::interval(INTERVAL);
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tick.tick().await;
                let levels = tracklist.lock().unwrap().levels();
                yield Levels {
                    left: levels.left,
                    right: levels.right,
                    low_left: levels.low_left,
                    low_right: levels.low_right,
                };
            }
        }
    }
}

/// Output levels for a meter, measured on the PCM leaving the device.
#[derive(SimpleObject, Clone, Copy, Default)]
pub struct Levels {
    /// 0..1 RMS over one output buffer.
    pub left: f32,
    pub right: f32,
    /// The same signal below roughly 200 Hz, which is what makes a meter move
    /// with the bass rather than with whatever is loudest.
    pub low_left: f32,
    pub low_right: f32,
}
