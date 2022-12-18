pub mod datpiff;
pub mod deezer;
pub mod genius;
pub mod kodi;
pub mod local;
pub mod myvazo;
pub mod tononkira;

use anyhow::Error;
use async_trait::async_trait;
use music_player_types::types::{Album, Artist, Track};

pub trait Addon {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn author(&self) -> &str;
    fn description(&self) -> &str;
    fn enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
}

pub trait StreamingAddon {
    fn stream(&self, url: &str) -> Result<(), Error>;
}

pub trait LyricsAddon {
    fn get_lyrics(&self, artist: &str, title: &str) -> Option<String>;
}

#[async_trait]
pub trait Browseable {
    async fn albums(&mut self) -> Result<Vec<Album>, Error>;
    async fn artists(&mut self) -> Result<Vec<Artist>, Error>;
    async fn tracks(&mut self) -> Result<Vec<Track>, Error>;
    async fn album(&mut self, id: &str) -> Result<Album, Error>;
    async fn artist(&mut self, id: &str) -> Result<Artist, Error>;
    async fn track(&mut self, id: &str) -> Result<Track, Error>;
}

#[async_trait]
pub trait Player {
    async fn play(&mut self) -> Result<(), Error>;
    async fn pause(&mut self) -> Result<(), Error>;
    async fn stop(&mut self) -> Result<(), Error>;
    async fn next(&mut self) -> Result<(), Error>;
    async fn previous(&mut self) -> Result<(), Error>;
    async fn seek(&mut self, position: u32) -> Result<(), Error>;
}
