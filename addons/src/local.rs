use super::{Addon, Browseable, Player, StreamingAddon};
use anyhow::{Error, Ok};
use async_trait::async_trait;
use music_player_client::{
    library::LibraryClient, playback::PlaybackClient, playlist::PlaylistClient,
    tracklist::TracklistClient,
};
use music_player_types::types::{Album, Artist, Device, Playlist, Track};

pub struct Client {
    pub library: LibraryClient,
    pub playback: PlaybackClient,
    pub playlist: PlaylistClient,
    pub tracklist: TracklistClient,
}

pub struct Local {
    name: String,
    version: String,
    author: String,
    description: String,
    enabled: bool,
    client: Option<Client>,
    host: String,
    port: u16,
}

impl Local {
    pub fn new() -> Self {
        Self {
            name: "Local".to_string(),
            version: "0.1.0".to_string(),
            author: "Tsiry Sandratraina".to_string(),
            description: "Local addon".to_string(),
            enabled: true,
            client: None,
            host: "localhost".to_string(),
            port: 5051,
        }
    }
}

impl Addon for Local {
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

impl StreamingAddon for Local {
    fn stream(&self, url: &str) -> Result<(), Error> {
        todo!("Implement Local::stream");
    }
}

#[async_trait]
impl Browseable for Local {
    async fn albums(&mut self, offset: i32, limit: i32) -> Result<Vec<Album>, Error> {
        let response = self
            .client
            .as_mut()
            .unwrap()
            .library
            .albums(offset, limit)
            .await?;
        Ok(response.into_iter().map(Into::into).collect())
    }

    async fn artists(&mut self, offset: i32, limit: i32) -> Result<Vec<Artist>, Error> {
        let response = self
            .client
            .as_mut()
            .unwrap()
            .library
            .artists(offset, limit)
            .await?;
        Ok(response.into_iter().map(Into::into).collect())
    }

    async fn tracks(&mut self, offset: i32, limit: i32) -> Result<Vec<Track>, Error> {
        let response = self
            .client
            .as_mut()
            .unwrap()
            .library
            .songs(offset, limit)
            .await?;
        Ok(response.into_iter().map(Into::into).collect())
    }

    async fn playlists(&mut self, offset: i32, limit: i32) -> Result<Vec<Playlist>, Error> {
        let response = self.client.as_mut().unwrap().playlist.list_all().await?;
        Ok(response)
    }

    async fn album(&mut self, id: &str) -> Result<Album, Error> {
        let response = self.client.as_mut().unwrap().library.album(id).await?;
        match response {
            Some(album) => Ok(album.into()),
            None => Err(Error::msg("Album not found")),
        }
    }

    async fn artist(&mut self, id: &str) -> Result<Artist, Error> {
        let response = self.client.as_mut().unwrap().library.artist(id).await?;
        match response {
            Some(artist) => Ok(artist.into()),
            None => Err(Error::msg("Artist not found")),
        }
    }

    async fn track(&mut self, id: &str) -> Result<Track, Error> {
        let response = self.client.as_mut().unwrap().library.song(id).await?;
        match response {
            Some(track) => Ok(track.into()),
            None => Err(Error::msg("Track not found")),
        }
    }

    async fn playlist(&mut self, id: &str) -> Result<Playlist, Error> {
        let response = self.client.as_mut().unwrap().playlist.find(id).await?;
        Ok(response)
    }
}

#[async_trait]
impl Player for Local {
    async fn play(&mut self) -> Result<(), Error> {
        self.client.as_mut().unwrap().playback.play().await?;
        todo!()
    }

    async fn pause(&mut self) -> Result<(), Error> {
        self.client.as_mut().unwrap().playback.pause().await?;
        todo!()
    }

    async fn stop(&mut self) -> Result<(), Error> {
        self.client.as_mut().unwrap().playback.stop().await?;
        todo!()
    }

    async fn next(&mut self) -> Result<(), Error> {
        self.client.as_mut().unwrap().playback.next().await?;
        todo!()
    }

    async fn previous(&mut self) -> Result<(), Error> {
        self.client.as_mut().unwrap().playback.prev().await?;
        todo!()
    }

    async fn seek(&mut self, position: u32) -> Result<(), Error> {
        todo!()
    }

    async fn load_tracks(&mut self, tracks: Vec<Track>) -> Result<(), Error> {
        todo!()
    }
}

impl From<Device> for Local {
    fn from(device: Device) -> Self {
        Self {
            host: device.host,
            port: device.port,
            ..Local::new()
        }
    }
}

impl Local {
    pub async fn connect(&mut self) -> Result<(), Error> {
        let client = Client {
            library: LibraryClient::new(self.host.clone(), self.port).await?,
            playback: PlaybackClient::new(self.host.clone(), self.port).await?,
            playlist: PlaylistClient::new(self.host.clone(), self.port).await?,
            tracklist: TracklistClient::new(self.host.clone(), self.port).await?,
        };

        self.client = Some(client);

        Ok(())
    }

    pub async fn connect_to_player(
        &mut self,
        device: Device,
    ) -> Result<Option<Box<dyn Player + Send>>, Error> {
        let mut player: Self = device.clone().into();
        player.connect().await?;
        Ok(Some(Box::new(player)))
    }
}
