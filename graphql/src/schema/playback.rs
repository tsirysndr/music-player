use std::sync::Arc;

use async_graphql::*;
use futures_util::{lock::Mutex, Stream};
use music_player_playback::player::PlayerCommand;
use music_player_tracklist::{PlaybackState, Tracklist};
use tokio::sync::mpsc::UnboundedSender;

use crate::simple_broker::SimpleBroker;

use super::objects::{
    current_track::CurrentlyPlayingSong, player_state::PlayerState, track::Track,
};

#[derive(Default)]
pub struct PlaybackQuery;

#[Object]
impl PlaybackQuery {
    async fn currently_playing_song(&self, ctx: &Context<'_>) -> Result<Option<Track>, Error> {
        let tracklist = ctx.data::<Arc<std::sync::Mutex<Tracklist>>>().unwrap();
        let (track, _) = tracklist.lock().unwrap().current_track();
        match track {
            Some(track) => Ok(Some(Track::from(track))),
            None => Ok(None),
        }
    }

    async fn get_player_state(&self, ctx: &Context<'_>) -> Result<PlayerState, Error> {
        let tracklist = ctx.data::<Arc<std::sync::Mutex<Tracklist>>>().unwrap();
        let (_, index) = tracklist.lock().unwrap().current_track();
        let playback_state = tracklist.lock().unwrap().playback_state();

        Ok(PlayerState {
            index: index as u32,
            position_ms: playback_state.position_ms,
            is_playing: playback_state.is_playing,
        })
    }

    async fn playback_progress(&self, ctx: &Context<'_>) -> Result<u32, Error> {
        let tracklist = ctx.data::<Arc<std::sync::Mutex<Tracklist>>>().unwrap();
        let playback_state = tracklist.lock().unwrap().playback_state();

        Ok(playback_state.position_ms)
    }
}

#[derive(Default)]
pub struct PlaybackMutation;

#[Object]
impl PlaybackMutation {
    async fn next(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        let player_cmd = ctx
            .data::<Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        player_cmd
            .lock()
            .unwrap()
            .send(PlayerCommand::Next)
            .unwrap();
        Ok(true)
    }

    async fn play(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        let player_cmd = ctx
            .data::<Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        player_cmd
            .lock()
            .unwrap()
            .send(PlayerCommand::Play)
            .unwrap();
        Ok(true)
    }

    async fn pause(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        let player_cmd = ctx
            .data::<Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        player_cmd
            .lock()
            .unwrap()
            .send(PlayerCommand::Pause)
            .unwrap();
        Ok(true)
    }

    async fn previous(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        let player_cmd = ctx
            .data::<Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        player_cmd
            .lock()
            .unwrap()
            .send(PlayerCommand::Previous)
            .unwrap();
        Ok(true)
    }

    async fn seek(&self, ctx: &Context<'_>, position: u32) -> Result<bool, Error> {
        let player_cmd = ctx
            .data::<Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        player_cmd
            .lock()
            .unwrap()
            .send(PlayerCommand::Seek(position))
            .unwrap();
        Ok(true)
    }

    async fn stop(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        let player_cmd = ctx
            .data::<Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>>()
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
