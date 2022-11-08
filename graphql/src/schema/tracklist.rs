use std::sync::Arc;

use async_graphql::*;
use music_player_entity::{album as album_entity, artist as artist_entity, track as track_entity};
use music_player_playback::player::PlayerCommand;
use music_player_storage::Database;
use music_player_tracklist::Tracklist as TracklistState;
use rand::seq::SliceRandom;
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QueryOrder};
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
        let _player_cmd = ctx
            .data::<Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        todo!()
    }

    async fn clear_tracklist(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        let player_cmd = ctx
            .data::<Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        player_cmd
            .lock()
            .unwrap()
            .send(PlayerCommand::Clear)
            .unwrap();
        Ok(true)
    }

    async fn remove_track(&self, ctx: &Context<'_>, position: u32) -> Result<bool, Error> {
        let player_cmd = ctx
            .data::<Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        player_cmd
            .lock()
            .unwrap()
            .send(PlayerCommand::RemoveTrack(position as usize))
            .unwrap();
        Ok(true)
    }

    async fn play_track_at(&self, ctx: &Context<'_>, position: u32) -> Result<bool, Error> {
        let player_cmd = ctx
            .data::<Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        player_cmd
            .lock()
            .unwrap()
            .send(PlayerCommand::PlayTrackAt(position as usize))
            .unwrap();
        Ok(true)
    }

    async fn shuffle(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        let _player_cmd = ctx
            .data::<Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        todo!()
    }

    async fn play_next(&self, ctx: &Context<'_>, id: ID) -> Result<bool, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let id = id.to_string();
        let results: Vec<(track_entity::Model, Vec<artist_entity::Model>)> =
            track_entity::Entity::find()
                .filter(track_entity::Column::Id.eq(id.clone()))
                .find_with_related(artist_entity::Entity)
                .all(db.lock().await.get_connection())
                .await?;
        if results.len() == 0 {
            return Err(Error::new("Track not found"));
        }
        let track = results[0].0.clone();
        let album =
            album_entity::Entity::find_by_id(track.album_id.unwrap_or_default().to_string())
                .one(db.lock().await.get_connection())
                .await?;
        let track = track_entity::Model {
            artists: results[0].1.clone(),
            album: album.unwrap(),
            id: track.id,
            title: track.title,
            duration: track.duration,
            uri: track.uri,
            ..Default::default()
        };
        let player_cmd = ctx
            .data::<Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        player_cmd
            .lock()
            .unwrap()
            .send(PlayerCommand::PlayNext(track_entity::Model { ..track }))
            .unwrap();
        Ok(true)
    }

    async fn play_album(
        &self,
        ctx: &Context<'_>,
        id: ID,
        position: Option<u32>,
        shuffle: bool,
    ) -> Result<bool, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let result = album_entity::Entity::find_by_id(id.to_string())
            .one(db.lock().await.get_connection())
            .await?;
        if result.is_none() {
            return Err(Error::new("Album not found"));
        }
        let album = result.unwrap();
        let mut tracks = album
            .find_related(track_entity::Entity)
            .order_by_asc(track_entity::Column::Track)
            .all(db.lock().await.get_connection())
            .await?;
        for track in &mut tracks {
            track.artists = track
                .find_related(artist_entity::Entity)
                .all(db.lock().await.get_connection())
                .await?;
            track.album = album.clone();
        }
        let player_cmd = ctx
            .data::<Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        let player_cmd_tx = player_cmd.lock().unwrap();
        player_cmd_tx.send(PlayerCommand::Stop).unwrap();
        player_cmd_tx.send(PlayerCommand::Clear).unwrap();

        if shuffle {
            tracks.shuffle(&mut rand::thread_rng());
        }

        player_cmd_tx
            .send(PlayerCommand::LoadTracklist { tracks })
            .unwrap();
        player_cmd_tx
            .send(PlayerCommand::PlayTrackAt(position.unwrap_or(0) as usize))
            .unwrap();
        Ok(true)
    }

    async fn play_artist_tracks(
        &self,
        ctx: &Context<'_>,
        id: ID,
        position: Option<u32>,
        shuffle: bool,
    ) -> Result<bool, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let id = id.to_string();
        let result = artist_entity::Entity::find_by_id(id.clone())
            .one(db.lock().await.get_connection())
            .await?;

        if result.is_none() {
            return Err(Error::new("Artist not found"));
        }

        let mut artist = result.unwrap();
        let results: Vec<(track_entity::Model, Option<album_entity::Model>)> =
            track_entity::Entity::find()
                .filter(track_entity::Column::ArtistId.eq(id.clone()))
                .order_by_asc(track_entity::Column::Title)
                .find_also_related(album_entity::Entity)
                .all(db.lock().await.get_connection())
                .await?;

        artist.tracks = results
            .into_iter()
            .map(|(track, album)| {
                let mut track = track;
                track.artists = vec![artist.clone()];
                track.album = album.unwrap();
                track
            })
            .collect();

        let player_cmd = ctx
            .data::<Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        let player_cmd_tx = player_cmd.lock().unwrap();
        player_cmd_tx.send(PlayerCommand::Stop).unwrap();
        player_cmd_tx.send(PlayerCommand::Clear).unwrap();

        if shuffle {
            artist.tracks.shuffle(&mut rand::thread_rng());
        }

        player_cmd_tx
            .send(PlayerCommand::LoadTracklist {
                tracks: artist.tracks,
            })
            .unwrap();
        player_cmd_tx
            .send(PlayerCommand::PlayTrackAt(position.unwrap_or(0) as usize))
            .unwrap();

        Ok(true)
    }

    async fn play_playlist(
        &self,
        ctx: &Context<'_>,
        id: ID,
        position: Option<u32>,
        shuffle: bool,
    ) -> Result<bool, Error> {
        let player_cmd = ctx
            .data::<Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        let player_cmd_tx = player_cmd.lock().unwrap();
        player_cmd_tx.send(PlayerCommand::Stop).unwrap();
        player_cmd_tx.send(PlayerCommand::Clear).unwrap();
        player_cmd_tx
            .send(PlayerCommand::LoadTracklist { tracks: vec![] })
            .unwrap();
        todo!()
    }
}
