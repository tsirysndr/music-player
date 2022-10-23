use std::sync::Arc;

use async_graphql::*;
use music_player_playback::player::PlayerCommand;
use music_player_tracklist::Tracklist;
use tokio::sync::{mpsc::UnboundedSender, Mutex};

use super::objects::{
    album::Album, artist::Artist, current_track::CurrentlyPlayingSong, track::Track,
};

#[derive(Default)]
pub struct PlaybackQuery;

#[Object]
impl PlaybackQuery {
    async fn currently_playing_song(
        &self,
        ctx: &Context<'_>,
    ) -> Result<CurrentlyPlayingSong, Error> {
        let tracklist = ctx.data::<Arc<std::sync::Mutex<Tracklist>>>().unwrap();
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
            track: Some(Track {
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
            }),
            index: index as u32,
            position_ms: playback_state.position_ms,
            is_playing: playback_state.is_playing,
        })
    }

    async fn get_playback_state(&self, ctx: &Context<'_>) -> bool {
        let _tracklist = ctx.data::<Arc<Mutex<Tracklist>>>().unwrap();
        todo!()
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
