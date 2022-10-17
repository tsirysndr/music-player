use async_graphql::*;

use super::objects::track::{Track, TrackInput};

#[derive(Default)]
pub struct TracklistQuery;

#[Object]
impl TracklistQuery {
    async fn tracks(&self, ctx: &Context<'_>) -> Vec<Track> {
        vec![]
    }
    async fn get_repeat(&self, ctx: &Context<'_>) -> bool {
        false
    }
    async fn get_random(&self, ctx: &Context<'_>) -> bool {
        false
    }
    async fn get_next_track(&self, ctx: &Context<'_>) -> Option<Track> {
        None
    }

    async fn get_previous_track(&self, ctx: &Context<'_>) -> Option<Track> {
        None
    }
}

#[derive(Default)]
pub struct TracklistMutation;

#[Object]
impl TracklistMutation {
    async fn add_track(&self, ctx: &Context<'_>, track: TrackInput) -> bool {
        false
    }

    async fn add_tracks(&self, ctx: &Context<'_>, tracks: Vec<TrackInput>) -> bool {
        false
    }

    async fn clear_tracklist(&self, ctx: &Context<'_>) -> bool {
        false
    }

    async fn remove_track(&self, ctx: &Context<'_>, position: u32) -> bool {
        false
    }

    async fn play_track_at(&self, ctx: &Context<'_>, position: u32) -> bool {
        false
    }

    async fn shuffle(&self, ctx: &Context<'_>) -> bool {
        false
    }
}
