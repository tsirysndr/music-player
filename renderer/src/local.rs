use super::Player;
use anyhow::{Error, Ok};
use async_trait::async_trait;
use music_player_client::{
    library::LibraryClient, playback::PlaybackClient, playlist::PlaylistClient,
    tracklist::TracklistClient,
};
use music_player_types::types::{Device, Playback, Track, MUSIC_PLAYER_DEVICE};

pub struct Client {
    pub library: LibraryClient,
    pub playback: PlaybackClient,
    pub playlist: PlaylistClient,
    pub tracklist: TracklistClient,
}

pub struct Local {
    client: Option<Client>,
    host: String,
    ip: String,
    port: u16,
}

impl Default for Local {
    fn default() -> Self {
        Self::new()
    }
}

impl Local {
    pub fn new() -> Self {
        Self {
            client: None,
            host: "localhost".to_string(),
            ip: "".to_string(),
            port: 5051,
        }
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
        self.client
            .as_mut()
            .unwrap()
            .playback
            .seek(position)
            .await?;
        Ok(())
    }

    async fn load_tracks(
        &mut self,
        tracks: Vec<Track>,
        start_index: Option<i32>,
    ) -> Result<(), Error> {
        self.client
            .as_mut()
            .unwrap()
            .tracklist
            .load_tracks(tracks, start_index.unwrap_or(0))
            .await?;
        Ok(())
    }

    async fn play_next(&mut self, track: Track) -> Result<(), Error> {
        self.client
            .as_mut()
            .unwrap()
            .tracklist
            .play_next(track)
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

    async fn get_current_tracklist(&mut self) -> Result<(Vec<Track>, Vec<Track>), Error> {
        let (previous, next) = self.client.as_mut().unwrap().tracklist.list().await?;
        Ok((
            previous.into_iter().map(Into::into).collect(),
            next.into_iter().map(Into::into).collect(),
        ))
    }

    async fn play_track_at(&mut self, position: u32) -> Result<(), Error> {
        self.client
            .as_mut()
            .unwrap()
            .tracklist
            .play_track_at(position as usize)
            .await?;
        Ok(())
    }

    async fn remove_track_at(&mut self, position: u32) -> Result<(), Error> {
        self.client
            .as_mut()
            .unwrap()
            .tracklist
            .remove(position)
            .await?;
        Ok(())
    }

    fn device_type(&self) -> String {
        String::from(MUSIC_PLAYER_DEVICE)
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
    /// Point at a peer daemon's gRPC endpoint before connecting.
    pub fn set_endpoint(&mut self, host: &str, port: u16) {
        self.host = host.to_owned();
        self.ip = host.to_owned();
        self.port = port;
    }

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
