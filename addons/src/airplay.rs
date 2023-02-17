use crate::{Addon, Player};
use anyhow::Error;
use async_trait::async_trait;
use music_player_types::types::{Device, Playback, Track, AIRPLAY_DEVICE};

pub struct Airplay {
    name: String,
    version: String,
    author: String,
    description: String,
    enabled: bool,
}

impl Airplay {
    pub fn new() -> Self {
        Self {
            name: "Airplay".to_string(),
            version: "0.1.0".to_string(),
            author: "Tsiry Sandratraina".to_string(),
            description: "Airplay addon".to_string(),
            enabled: true,
        }
    }

    pub fn connect(&mut self, device: Device) -> Result<Option<Box<dyn Player + Send>>, Error> {
        let player: Self = device.clone().into();
        Ok(Some(Box::new(player)))
    }
}

impl Addon for Airplay {
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
impl Player for Airplay {
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

    async fn load_tracks(&mut self, tracks: Vec<Track>, start_index: Option<i32>) -> Result<(), Error> {
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
        String::from(AIRPLAY_DEVICE)
    }

    fn disconnect(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl From<Device> for Airplay {
    fn from(device: Device) -> Self {
        Self { ..Airplay::new() }
    }
}
