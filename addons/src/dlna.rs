use anyhow::Error;
use async_trait::async_trait;
use music_player_types::types::{Playback, Track};
use upnp_client::media_renderer::MediaRendererClient;

use crate::{Addon, Player};

pub struct Dlna {
    name: String,
    version: String,
    author: String,
    description: String,
    enabled: bool,
    client: Option<MediaRendererClient>,
}

impl Dlna {
    pub fn new() -> Self {
        Self {
            name: "DLNA".to_string(),
            version: "0.1.0".to_string(),
            author: "Tsiry Sandratraina".to_string(),
            description: "UPnP/DLNA addon".to_string(),
            enabled: true,
            client: None,
        }
    }
}

impl Addon for Dlna {
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

#[async_trait]
impl Player for Dlna {
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

    fn device_type(&self) -> String {
        "xbmc".to_string()
    }

    fn disconnect(&mut self) -> Result<(), Error> {
        self.client = None;
        Ok(())
    }
}
