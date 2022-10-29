use std::sync::Arc;

use async_graphql::*;
use music_player_entity::{album as album_entity, artist as artist_entity, track as track_entity};
use music_player_playback::player::PlayerCommand;
use music_player_storage::Database;
use music_player_tracklist::Tracklist as TracklistState;
use sea_orm::EntityTrait;
use tokio::sync::{mpsc::UnboundedSender, Mutex};

use super::objects::{
    track::{Track, TrackInput},
    tracklist::Tracklist,
};

#[derive(Default)]
pub struct TracklistQuery;

#[Object]
impl TracklistQuery {
    async fn tracklist_tracks(&self, ctx: &Context<'_>) -> Result<Tracklist, Error> {
        let state = ctx.data::<Arc<std::sync::Mutex<TracklistState>>>().unwrap();
        let (previous_tracks, next_tracks) = state.lock().unwrap().tracks();

        let response = Tracklist {
            next_tracks: next_tracks.into_iter().map(Into::into).collect(),
            previous_tracks: previous_tracks.into_iter().map(Into::into).collect(),
        };

        Ok(response)
    }
    async fn get_repeat(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        todo!()
    }
    async fn get_random(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        todo!()
    }
    async fn get_next_track(&self, ctx: &Context<'_>) -> Result<Option<Track>, Error> {
        todo!()
    }

    async fn get_previous_track(&self, ctx: &Context<'_>) -> Result<Option<Track>, Error> {
        todo!()
    }
}

#[derive(Default)]
pub struct TracklistMutation;

#[Object]
impl TracklistMutation {
    async fn add_track(&self, ctx: &Context<'_>, track: TrackInput) -> Result<Vec<Track>, Error> {
        let player_cmd = ctx
            .data::<Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();

        let id = track.id.to_string();

        let result: Vec<(track_entity::Model, Vec<artist_entity::Model>)> =
            track_entity::Entity::find_by_id(id.clone())
                .find_with_related(artist_entity::Entity)
                .all(db.lock().await.get_connection())
                .await?;

        if result.len() == 0 {
            return Err(Error::new("Track not found"));
        }

        let (mut track, artists) = result.into_iter().next().unwrap();
        track.artists = artists;

        let result: Vec<(track_entity::Model, Option<album_entity::Model>)> =
            track_entity::Entity::find_by_id(id.clone())
                .find_also_related(album_entity::Entity)
                .all(db.lock().await.get_connection())
                .await?;
        let (_, album) = result.into_iter().next().unwrap();
        track.album = album.unwrap();

        player_cmd
            .lock()
            .unwrap()
            .send(PlayerCommand::LoadTracklist {
                tracks: vec![track],
            })
            .unwrap();
        Ok(vec![])
    }

    async fn add_tracks(&self, ctx: &Context<'_>, tracks: Vec<TrackInput>) -> Result<bool, Error> {
        let player_cmd = ctx.data::<UnboundedSender<PlayerCommand>>().unwrap();
        todo!()
    }

    async fn clear_tracklist(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        let player_cmd = ctx.data::<UnboundedSender<PlayerCommand>>().unwrap();
        player_cmd.send(PlayerCommand::Clear).unwrap();
        Ok(true)
    }

    async fn remove_track(&self, ctx: &Context<'_>, position: u32) -> Result<bool, Error> {
        let player_cmd = ctx.data::<UnboundedSender<PlayerCommand>>().unwrap();
        todo!()
    }

    async fn play_track_at(&self, ctx: &Context<'_>, position: u32) -> Result<bool, Error> {
        let player_cmd = ctx.data::<UnboundedSender<PlayerCommand>>().unwrap();
        todo!()
    }

    async fn shuffle(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        let player_cmd = ctx.data::<UnboundedSender<PlayerCommand>>().unwrap();
        todo!()
    }

    async fn play_next(&self, ctx: &Context<'_>, id: ID) -> Result<bool, Error> {
        let player_cmd = ctx.data::<UnboundedSender<PlayerCommand>>().unwrap();
        todo!()
    }
}
