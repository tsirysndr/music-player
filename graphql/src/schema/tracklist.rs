use std::sync::Arc;

use async_graphql::*;
use music_player_entity::track as track_entity;
use music_player_playback::player::PlayerCommand;
use music_player_storage::Database;
use music_player_tracklist::Tracklist as TracklistState;
use sea_orm::EntityTrait;
use tokio::sync::{mpsc::UnboundedSender, Mutex};

use super::objects::{
    album::Album,
    artist::Artist,
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
            next_tracks: next_tracks
                .into_iter()
                .map(|track| Track {
                    id: ID(track.id),
                    title: track.title,
                    uri: track.uri,
                    track_number: track.track,
                    artists: vec![Artist {
                        name: track.artist,
                        ..Default::default()
                    }],
                    album: Album {
                        // id: track.album_id.unwrap(),
                        title: track.album,
                        year: track.year,
                        ..Default::default()
                    },
                    duration: track.duration,
                    ..Default::default()
                })
                .collect(),
            previous_tracks: previous_tracks
                .into_iter()
                .map(|track| Track {
                    id: ID(track.id),
                    title: track.title,
                    uri: track.uri,
                    track_number: track.track,
                    artists: vec![Artist {
                        name: track.artist,
                        ..Default::default()
                    }],
                    album: Album {
                        // id: track.album_id.unwrap(),
                        title: track.album,
                        year: track.year,
                        ..Default::default()
                    },
                    duration: track.duration,
                    ..Default::default()
                })
                .collect(),
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
        let player_cmd = ctx.data::<Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>>().unwrap();
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();

        let result = track_entity::Entity::find_by_id(track.clone().id.to_string())
            .one(db.lock().await.get_connection())
            .await?;
        if result.is_none() {
            return Err(Error::new("Track not found"));
        }

        let track = result.unwrap();
        player_cmd.lock().unwrap().send(PlayerCommand::LoadTracklist {
            tracks: vec![track],
        });
        Ok(vec![])
    }

    async fn add_tracks(&self, ctx: &Context<'_>, tracks: Vec<TrackInput>) -> Result<bool, Error> {
        let player_cmd = ctx.data::<UnboundedSender<PlayerCommand>>().unwrap();
        todo!()
    }

    async fn clear_tracklist(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        let player_cmd = ctx.data::<UnboundedSender<PlayerCommand>>().unwrap();
        player_cmd.send(PlayerCommand::Clear);
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
