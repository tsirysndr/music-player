use async_graphql::*;

use super::objects::track::Track;

#[derive(Default)]
pub struct PlaybackQuery;

#[Object]
impl PlaybackQuery {
    async fn currently_playing_song(&self, ctx: &Context<'_>) -> Option<Track> {
        None
    }

    async fn get_playback_state(&self, ctx: &Context<'_>) -> bool {
        false
    }
}

#[derive(Default)]
pub struct PlaybackMutation;

#[Object]
impl PlaybackMutation {
    async fn next(&self, ctx: &Context<'_>) -> bool {
        false
    }

    async fn play(&self, ctx: &Context<'_>) -> bool {
        false
    }

    async fn pause(&self, ctx: &Context<'_>) -> bool {
        false
    }

    async fn previous(&self, ctx: &Context<'_>) -> bool {
        false
    }

    async fn seek(&self, ctx: &Context<'_>, position: u32) -> bool {
        false
    }

    async fn stop(&self, ctx: &Context<'_>) -> bool {
        false
    }
}
