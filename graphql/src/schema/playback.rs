use std::sync::Arc;

use async_graphql::*;
use music_player_playback::player::Player;
use tokio::sync::Mutex;

use super::objects::track::Track;

#[derive(Default)]
pub struct PlaybackQuery;

#[Object]
impl PlaybackQuery {
    async fn currently_playing_song(&self, ctx: &Context<'_>) -> Option<Track> {
        let player = ctx.data::<Arc<Mutex<Player>>>().unwrap();
        let player = player.lock().await;
        None
    }

    async fn get_playback_state(&self, ctx: &Context<'_>) -> bool {
        let player = ctx.data::<Arc<Mutex<Player>>>().unwrap();
        let player = player.lock().await;
        false
    }
}

#[derive(Default)]
pub struct PlaybackMutation;

#[Object]
impl PlaybackMutation {
    async fn next(&self, ctx: &Context<'_>) -> bool {
        let player = ctx.data::<Arc<Mutex<Player>>>().unwrap();
        let player = player.lock().await;
        false
    }

    async fn play(&self, ctx: &Context<'_>) -> bool {
        let player = ctx.data::<Arc<Mutex<Player>>>().unwrap();
        let player = player.lock().await;
        false
    }

    async fn pause(&self, ctx: &Context<'_>) -> bool {
        let player = ctx.data::<Arc<Mutex<Player>>>().unwrap();
        let player = player.lock().await;
        false
    }

    async fn previous(&self, ctx: &Context<'_>) -> bool {
        let player = ctx.data::<Arc<Mutex<Player>>>().unwrap();
        let player = player.lock().await;
        false
    }

    async fn seek(&self, ctx: &Context<'_>, position: u32) -> bool {
        let player = ctx.data::<Arc<Mutex<Player>>>().unwrap();
        let player = player.lock().await;
        false
    }

    async fn stop(&self, ctx: &Context<'_>) -> bool {
        let player = ctx.data::<Arc<Mutex<Player>>>().unwrap();
        let player = player.lock().await;
        false
    }
}
