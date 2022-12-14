use anyhow::Error;

use async_trait::async_trait;
use music_player_types::types::{Album, Artist, Device, Playlist, Track};

use super::{Addon, Browseable, Player, StreamingAddon};

pub struct Client {
    pub host: String,
    pub port: u16,
}

pub struct Kodi {
    name: String,
    version: String,
    author: String,
    description: String,
    enabled: bool,
    client: Option<Client>,
}

impl Kodi {
    pub fn new() -> Self {
        Self {
            name: "Kodi".to_string(),
            version: "0.1.0".to_string(),
            author: "Tsiry Sandratraina".to_string(),
            description: "Kodi addon".to_string(),
            enabled: true,
            client: None,
        }
    }
}

impl Addon for Kodi {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn author(&self) -> &str {
        &self.author
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl StreamingAddon for Kodi {
    fn stream(&self, url: &str) -> Result<(), Error> {
        todo!("Implement Kodi::stream");
    }
}

#[async_trait]
impl Browseable for Kodi {
    async fn albums(&mut self, offset: i32, limit: i32) -> Result<Vec<Album>, Error> {
        todo!()
    }

    async fn artists(&mut self, offset: i32, limit: i32) -> Result<Vec<Artist>, Error> {
        todo!()
    }

    async fn tracks(&mut self, offset: i32, limit: i32) -> Result<Vec<Track>, Error> {
        todo!()
    }

    async fn playlists(&mut self, offset: i32, limit: i32) -> Result<Vec<Playlist>, Error> {
        todo!()
    }

    async fn album(&mut self, id: &str) -> Result<Album, Error> {
        todo!()
    }

    async fn artist(&mut self, id: &str) -> Result<Artist, Error> {
        todo!()
    }

    async fn track(&mut self, id: &str) -> Result<Track, Error> {
        todo!()
    }

    async fn playlist(&mut self, id: &str) -> Result<Playlist, Error> {
        todo!()
    }
}

#[async_trait]
impl Player for Kodi {
    async fn play(&mut self) -> Result<(), Error> {
        todo!()
    }

    async fn pause(&mut self) -> Result<(), Error> {
        todo!()
    }

    async fn stop(&mut self) -> Result<(), Error> {
        todo!()
    }

    async fn next(&mut self) -> Result<(), Error> {
        todo!()
    }

    async fn previous(&mut self) -> Result<(), Error> {
        todo!()
    }

    async fn seek(&mut self, position: u32) -> Result<(), Error> {
        todo!()
    }
}

impl From<Device> for Kodi {
    fn from(device: Device) -> Self {
        Self { ..Kodi::new() }
    }
}
