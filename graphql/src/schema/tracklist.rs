use async_graphql::*;
use futures_util::Stream;
use music_player_addons::CurrentDevice;
use music_player_entity::{
    album as album_entity, artist as artist_entity, playlist as playlist_entity,
    playlist_tracks as playlist_tracks_entity, select_result, track as track_entity,
};
use music_player_playback::player::PlayerCommand;
use music_player_storage::repo::album::AlbumRepository;
use music_player_storage::repo::artist::ArtistRepository;
use music_player_storage::repo::playlist::PlaylistRepository;
use music_player_storage::repo::track::TrackRepository;
use music_player_storage::Database;
use music_player_tracklist::Tracklist as TracklistState;
use music_player_types::types::{self, RemoteCoverUrl, RemoteTrackUrl};
use rand::seq::SliceRandom;
use sea_orm::{
    ColumnTrait, EntityTrait, JoinType, ModelTrait, QueryFilter, QueryOrder, QuerySelect,
    RelationTrait,
};
use std::sync::Mutex as StdMutex;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc::UnboundedSender, Mutex};

use crate::load_tracks;
use crate::simple_broker::SimpleBroker;

use super::objects::album::Album;
use super::{
    objects::{
        track::{Track, TrackInput},
        tracklist::Tracklist,
    },
    MutationType,
};

#[derive(Default)]
pub struct TracklistQuery;

#[Object]
impl TracklistQuery {
    async fn tracklist_tracks(&self, ctx: &Context<'_>) -> Result<Tracklist, Error> {
        let state = ctx.data::<Arc<StdMutex<TracklistState>>>().unwrap();
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
        let state = ctx.data::<Arc<StdMutex<TracklistState>>>().unwrap();
        let player_cmd = ctx
            .data::<Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let connected_device = ctx
            .data::<Arc<StdMutex<HashMap<String, types::Device>>>>()
            .unwrap();
        let current_device = ctx.data::<Arc<Mutex<CurrentDevice>>>().unwrap();
        let mut device = current_device.lock().await;

        let id = track.id.to_string();

        if device.source.is_some() {
            let source = device.source.as_mut().unwrap();
            // TODO: call grpc to add track to tracklist
        }

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
                tracks: vec![track.clone()],
            })
            .unwrap();

        let (previous_tracks, next_tracks) = state.lock().unwrap().tracks();

        SimpleBroker::publish(TracklistChanged {
            tracklist: Tracklist {
                next_tracks: next_tracks.into_iter().map(Into::into).collect(),
                previous_tracks: previous_tracks.into_iter().map(Into::into).collect(),
            },
            mutation_type: MutationType::Updated,
            track: Some(track.clone().into()),
        });
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
            .data::<Arc<StdMutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        player_cmd
            .lock()
            .unwrap()
            .send(PlayerCommand::Clear)
            .unwrap();
        SimpleBroker::publish(TracklistChanged {
            tracklist: Tracklist {
                next_tracks: vec![],
                previous_tracks: vec![],
            },
            mutation_type: MutationType::Cleared,
            track: None,
        });
        Ok(true)
    }

    async fn remove_track(&self, ctx: &Context<'_>, position: u32) -> Result<bool, Error> {
        let state = ctx.data::<Arc<StdMutex<TracklistState>>>().unwrap();
        let player_cmd = ctx
            .data::<Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        player_cmd
            .lock()
            .unwrap()
            .send(PlayerCommand::RemoveTrack(position as usize))
            .unwrap();

        let (previous_tracks, next_tracks) = state.lock().unwrap().tracks();
        SimpleBroker::publish(TracklistChanged {
            tracklist: Tracklist {
                next_tracks: next_tracks.into_iter().map(Into::into).collect(),
                previous_tracks: previous_tracks.into_iter().map(Into::into).collect(),
            },
            mutation_type: MutationType::Updated,
            track: None,
        });
        Ok(true)
    }

    async fn play_track_at(&self, ctx: &Context<'_>, position: u32) -> Result<bool, Error> {
        let player_cmd = ctx
            .data::<Arc<StdMutex<UnboundedSender<PlayerCommand>>>>()
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
            .data::<Arc<StdMutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        todo!()
    }

    async fn play_next(&self, ctx: &Context<'_>, id: ID) -> Result<bool, Error> {
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let connected_device = ctx
            .data::<Arc<StdMutex<HashMap<String, types::Device>>>>()
            .unwrap();
        let current_device = ctx.data::<Arc<Mutex<CurrentDevice>>>().unwrap();
        let mut device = current_device.lock().await;

        let id = id.to_string();

        let track: track_entity::Model;

        if device.source.is_some() {
            let source = device.source.as_mut().unwrap();
            let result = source.track(&id).await?;

            let device = connected_device.lock().unwrap();
            let device = device.get("current_device").unwrap();
            let base_url = device.base_url.as_ref().unwrap();

            track = result
                .with_remote_track_url(base_url.as_str())
                .with_remote_cover_url(base_url.as_str())
                .into();
        } else {
            track = TrackRepository::new(db.lock().await.get_connection())
                .find(&id)
                .await?;
        }

        let player_cmd = ctx
            .data::<Arc<StdMutex<UnboundedSender<PlayerCommand>>>>()
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
        let player_cmd = ctx
            .data::<Arc<StdMutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let connected_device = ctx
            .data::<Arc<StdMutex<HashMap<String, types::Device>>>>()
            .unwrap();
        let current_device = ctx.data::<Arc<Mutex<CurrentDevice>>>().unwrap();
        let mut device = current_device.lock().await;

        let id = id.to_string();

        if device.source.is_some() {
            let source = device.source.as_mut().unwrap();
            let album = source.album(&id).await?;

            let device = connected_device.lock().unwrap();
            let device = device.get("current_device").unwrap();
            let base_url = device.base_url.as_ref().unwrap();

            let album: album_entity::Model = album
                .with_remote_cover_url(base_url.as_str())
                .with_remote_track_url(base_url.as_str())
                .into();
            let tracks = album.tracks;
            load_tracks(player_cmd, tracks, position, shuffle);
            return Ok(true);
        }

        let result = AlbumRepository::new(db.lock().await.get_connection())
            .find(&id)
            .await?;
        load_tracks(player_cmd, result.tracks, position, shuffle);
        Ok(true)
    }

    async fn play_artist_tracks(
        &self,
        ctx: &Context<'_>,
        id: ID,
        position: Option<u32>,
        shuffle: bool,
    ) -> Result<bool, Error> {
        let player_cmd = ctx
            .data::<Arc<StdMutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let connected_device = ctx
            .data::<Arc<StdMutex<HashMap<String, types::Device>>>>()
            .unwrap();
        let current_device = ctx.data::<Arc<Mutex<CurrentDevice>>>().unwrap();
        let mut device = current_device.lock().await;
        let id = id.to_string();

        if device.source.is_some() {
            let source = device.source.as_mut().unwrap();
            let artist = source.artist(&id).await?;

            let device = connected_device.lock().unwrap();
            let device = device.get("current_device").unwrap();
            let base_url = device.base_url.as_ref().unwrap();

            let artist: artist_entity::Model =
                artist.with_remote_track_url(base_url.as_str()).into();
            load_tracks(player_cmd, artist.tracks, position, shuffle);
            return Ok(true);
        }

        let artist = ArtistRepository::new(db.lock().await.get_connection())
            .find(&id)
            .await?;

        load_tracks(player_cmd, artist.tracks, position, shuffle);
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
        let db = ctx.data::<Arc<Mutex<Database>>>().unwrap();
        let db = db.lock().await;
        let connected_device = ctx
            .data::<Arc<StdMutex<HashMap<String, types::Device>>>>()
            .unwrap();
        let current_device = ctx.data::<Arc<Mutex<CurrentDevice>>>().unwrap();
        let mut device = current_device.lock().await;

        let id = id.to_string();

        if device.source.is_some() {
            let source = device.source.as_mut().unwrap();
            let result = source.playlist(&id).await?;

            let device = connected_device.lock().unwrap();
            let device = device.get("current_device").unwrap();
            let base_url = device.base_url.as_ref().unwrap();

            let tracks = result.with_remote_track_url(base_url.as_str()).tracks;
            let tracks: Vec<track_entity::Model> = tracks.into_iter().map(Into::into).collect();

            load_tracks(player_cmd, tracks, position, shuffle);
            return Ok(true);
        }

        let playlist = PlaylistRepository::new(db.get_connection())
            .find(id.as_str())
            .await?;

        let tracks: Vec<track_entity::Model> =
            playlist.tracks.into_iter().map(Into::into).collect();

        load_tracks(player_cmd, tracks, position, shuffle);

        Ok(true)
    }
}

#[derive(Clone)]
struct TracklistChanged {
    mutation_type: MutationType,
    tracklist: Tracklist,
    track: Option<Track>,
}

#[Object]
impl TracklistChanged {
    async fn mutation_type(&self) -> MutationType {
        self.mutation_type
    }

    async fn tracklist(&self) -> &Tracklist {
        &self.tracklist
    }

    async fn track(&self) -> Option<&Track> {
        self.track.as_ref()
    }
}

#[derive(Default)]
pub struct TracklistSubscription;

#[Subscription]
impl TracklistSubscription {
    async fn tracklist(&self, _id: ID) -> impl Stream<Item = TracklistChanged> {
        SimpleBroker::<TracklistChanged>::subscribe()
    }
}
