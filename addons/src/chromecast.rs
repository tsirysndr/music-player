use crate::{Addon, Player};
use anyhow::Error;
use async_trait::async_trait;
use music_player_types::types::{Device, Track};

pub struct Chromecast {
    name: String,
    version: String,
    author: String,
    description: String,
    enabled: bool,
}

impl Chromecast {
    pub fn new() -> Self {
        Self {
            name: "Chromecast".to_string(),
            version: "0.1.0".to_string(),
            author: "Tsiry Sandratraina".to_string(),
            description: "Chromecast addon".to_string(),
            enabled: true,
        }
    }

    pub fn connect(&mut self, device: Device) -> Result<Option<Box<dyn Player + Send>>, Error> {
        let player: Self = device.clone().into();
        Ok(Some(Box::new(player)))
    }
}

impl Addon for Chromecast {
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
impl Player for Chromecast {
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

    async fn load_tracks(&mut self, tracks: Vec<Track>) -> Result<(), Error> {
        todo!()
    }
}

impl From<Device> for Chromecast {
    fn from(device: Device) -> Self {
        Self {
            ..Chromecast::new()
        }
    }
}
