use anyhow::Error;

use async_trait::async_trait;
use music_player_types::types::{Album, Artist, Device, Playback, Playlist, Track, XBMC_DEVICE};

use super::{Addon, Browsable, Player, StreamingAddon};

pub struct Client {
    pub host: String,
    pub ip: String,
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

    pub fn connect_to_player(
        &mut self,
        device: Device,
    ) -> Result<Option<Box<dyn Player + Send>>, Error> {
        let player: Self = device.clone().into();
        Ok(Some(Box::new(player)))
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
impl Browsable for Kodi {
    async fn albums(
        &mut self,
        filter: Option<String>,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<Album>, Error> {
        todo!()
    }

    async fn artists(
        &mut self,
        filter: Option<String>,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<Artist>, Error> {
        todo!()
    }

    async fn tracks(
        &mut self,
        filter: Option<String>,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<Track>, Error> {
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

    fn device_ip(&self) -> String {
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

    async fn load_tracks(
        &mut self,
        tracks: Vec<Track>,
        start_index: Option<i32>,
    ) -> Result<(), Error> {
        todo!()
    }

    async fn play_next(&mut self, track: Track) -> Result<(), Error> {
        todo!()
    }

    async fn load(&mut self, track: Track) -> Result<(), Error> {
        todo!()
    }

    async fn get_current_playback(&mut self) -> Result<Playback, Error> {
        todo!()
    }

    async fn get_current_tracklist(&mut self) -> Result<(Vec<Track>, Vec<Track>), Error> {
        todo!()
    }

    async fn play_track_at(&mut self, position: u32) -> Result<(), Error> {
        todo!()
    }

    async fn remove_track_at(&mut self, position: u32) -> Result<(), Error> {
        todo!()
    }

    fn device_type(&self) -> String {
        String::from(XBMC_DEVICE)
    }

    fn disconnect(&mut self) -> Result<(), Error> {
        self.client = None;
        Ok(())
    }
}

impl From<Device> for Kodi {
    fn from(device: Device) -> Self {
        Self { ..Kodi::new() }
    }
}
