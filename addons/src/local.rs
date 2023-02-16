use super::{Addon, Browseable, Player, StreamingAddon};
use anyhow::{Error, Ok};
use async_trait::async_trait;
use music_player_client::{
    library::LibraryClient, playback::PlaybackClient, playlist::PlaylistClient,
    tracklist::TracklistClient,
};
use music_player_types::types::{Album, Artist, Device, Playback, Playlist, Track};

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
    ip: String,
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
            ip: "".to_string(),
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

    fn device_ip(&self) -> String {
        self.ip.clone()
    }
}

#[async_trait]
impl Player for Local {
    async fn play(&mut self) -> Result<(), Error> {
        self.client.as_mut().unwrap().playback.play().await?;
        Ok(())
    }

    async fn pause(&mut self) -> Result<(), Error> {
        self.client.as_mut().unwrap().playback.pause().await?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Error> {
        self.client.as_mut().unwrap().playback.stop().await?;
        Ok(())
    }

    async fn next(&mut self) -> Result<(), Error> {
        self.client.as_mut().unwrap().playback.next().await?;
        Ok(())
    }

    async fn previous(&mut self) -> Result<(), Error> {
        self.client.as_mut().unwrap().playback.prev().await?;
        Ok(())
    }

    async fn seek(&mut self, position: u32) -> Result<(), Error> {
        todo!()
    }

    async fn load_tracks(
        &mut self,
        tracks: Vec<Track>,
        start_index: Option<i32>,
    ) -> Result<(), Error> {
        let ids: Vec<String> = tracks.into_iter().map(|t| t.id).collect();
        self.client
            .as_mut()
            .unwrap()
            .tracklist
            .load_tracks(ids, start_index.unwrap_or(0))
            .await?;
        Ok(())
    }

    async fn play_next(&mut self, track: Track) -> Result<(), Error> {
        self.client
            .as_mut()
            .unwrap()
            .tracklist
            .play_next(&track.id)
            .await?;
        Ok(())
    }

    async fn load(&mut self, track: Track) -> Result<(), Error> {
        self.client
            .as_mut()
            .unwrap()
            .tracklist
            .add(&track.id)
            .await?;
        Ok(())
    }

    async fn get_current_playback(&mut self) -> Result<Playback, Error> {
        let (current_track, index, position_ms, is_playing) =
            self.client.as_mut().unwrap().playback.current().await?;
        return match current_track {
            Some(track) => Ok(Playback {
                current_track: Some(track.into()),
                index,
                position_ms,
                is_playing,
                current_item_id: None,
                items: vec![],
            }),
            None => Ok(Playback {
                current_track: None,
                index: 0,
                position_ms: 0,
                is_playing: false,
                current_item_id: None,
                items: vec![],
            }),
        };
    }

    fn device_type(&self) -> String {
        "music-player".to_string()
    }

    fn disconnect(&mut self) -> Result<(), Error> {
        self.client = None;
        Ok(())
    }
}

impl From<Device> for Local {
    fn from(device: Device) -> Self {
        Self {
            host: device.host,
            port: device.port,
            ip: device.ip,
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
