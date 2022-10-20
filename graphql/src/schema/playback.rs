use std::sync::Arc;

use async_graphql::*;
use music_player_playback::player::{Player, PlayerEngine};
use tokio::sync::Mutex;

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
        let player = ctx.data::<Arc<Mutex<Player>>>().unwrap();
        let player = player.lock().await;

        let track = player.get_current_track().await;

        if track.is_none() {
            let response = CurrentlyPlayingSong {
                track: None,
                index: 0,
                position_ms: 0,
                is_playing: false,
            };
            return Ok(response);
        }

        let (track, index, position_ms, is_playing) = track.unwrap();
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
            position_ms,
            is_playing,
        })
    }

    async fn get_playback_state(&self, ctx: &Context<'_>) -> bool {
        let player = ctx.data::<Arc<Mutex<Player>>>().unwrap();
        let player = player.lock().await;
        todo!()
    }
}

#[derive(Default)]
pub struct PlaybackMutation;

#[Object]
impl PlaybackMutation {
    async fn next(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        let player = ctx.data::<Arc<Mutex<Player>>>().unwrap();
        let player = player.lock().await;
        player.next();
        Ok(true)
    }

    async fn play(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        let player = ctx.data::<Arc<Mutex<Player>>>().unwrap();
        let player = player.lock().await;
        player.play();
        Ok(true)
    }

    async fn pause(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        let player = ctx.data::<Arc<Mutex<Player>>>().unwrap();
        let player = player.lock().await;
        player.pause();
        Ok(true)
    }

    async fn previous(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        let player = ctx.data::<Arc<Mutex<Player>>>().unwrap();
        let player = player.lock().await;
        player.previous();
        Ok(true)
    }

    async fn seek(&self, ctx: &Context<'_>, position: u32) -> Result<bool, Error> {
        let player = ctx.data::<Arc<Mutex<Player>>>().unwrap();
        let player = player.lock().await;
        player.seek(position);
        Ok(true)
    }

    async fn stop(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        let player = ctx.data::<Arc<Mutex<Player>>>().unwrap();
        let player = player.lock().await;
        player.stop();
        Ok(true)
    }
}
