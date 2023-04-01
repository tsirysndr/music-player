use std::sync::{Arc, Mutex};

use crate::simple_broker::SimpleBroker;
use async_graphql::*;
use futures_util::Stream;
use music_player_addons::CurrentReceiverDevice;
use music_player_playback::player::PlayerCommand;
use music_player_tracklist::{PlaybackState, Tracklist};
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
        let _tracklist = ctx.data::<Arc<Mutex<Tracklist>>>().unwrap();
        todo!()
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
}
