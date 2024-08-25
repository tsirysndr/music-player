use extism_pdk::*;

use crate::types::{Track, Tracklist};

#[host_fn]
extern "ExtismHost" {
    fn open_media(url: String);
    fn register_addon(name: String);
    fn currently_playing_song();
    fn next();
    fn previous();
    fn pause();
    fn play();
    fn stop();
    fn seek(time: u32);
    fn position_ms() -> u32;
    fn load_tracklist(tracks: Json<Vec<Track>>);
    fn play_next();
    fn load(track: Json<Track>);
    fn get_current_track();
    fn get_current_tracklist() -> Json<Tracklist>;
    fn play_track_at(index: u32);
    fn remove_track(index: u32);
}

pub fn player() -> Player {
    Player {}
}

pub struct Player {}

impl Player {
    pub fn play(&self) -> Result<(), Error> {
        unsafe { play()? };
        Ok(())
    }

    pub fn pause(&self) -> Result<(), Error> {
        unsafe { pause()? };
        Ok(())
    }

    pub fn stop(&self) -> Result<(), Error> {
        unsafe { stop()? };
        Ok(())
    }

    pub fn next(&self) -> Result<(), Error> {
        unsafe { next()? };
        Ok(())
    }

    pub fn previous(&self) -> Result<(), Error> {
        unsafe { previous()? };
        Ok(())
    }

    pub fn seek(&self, time: u32) -> Result<(), Error> {
        unsafe { seek(time)? };
        Ok(())
    }

    pub fn position_ms(&self) -> Result<u32, Error> {
        unsafe { position_ms() }
    }

    pub fn currently_playing_song(&self) -> Result<(), Error> {
        unsafe { currently_playing_song() }
    }

    pub fn open_media(&self, url: String) -> Result<(), Error> {
        unsafe { open_media(url) }
    }

    pub fn load_tracks(&self, tracks: Json<Vec<Track>>) -> Result<(), Error> {
        unsafe { load_tracklist(tracks) }
    }

    pub fn play_next(&self) -> Result<(), Error> {
        unsafe { play_next() }
    }

    pub fn load(&self, track: Json<Track>) -> Result<(), Error> {
        unsafe { load(track) }
    }

    pub fn get_current_playback(&self) -> Result<(), Error> {
        unsafe { get_current_track() }
    }

    pub fn get_current_tracklist(&self) -> Result<Json<Tracklist>, Error> {
        unsafe { get_current_tracklist() }
    }

    pub fn play_track_at(&self, index: u32) -> Result<(), Error> {
        unsafe { play_track_at(index) }
    }

    pub fn remove_track_at(&self, index: u32) -> Result<(), Error> {
        unsafe { remove_track(index) }
    }
}
