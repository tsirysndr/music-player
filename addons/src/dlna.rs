use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{Addon, Browseable, Player};
use anyhow::Error;
use async_trait::async_trait;
use music_player_tracklist::Tracklist;
use music_player_types::types::{Album, Artist, Device, Playback, Playlist, Track};
use upnp_client::{
    device_client::DeviceClient,
    media_renderer::MediaRendererClient,
    media_server::MediaServerClient,
    types::{LoadOptions, Metadata, ObjectClass},
};

pub struct Dlna {
    name: String,
    version: String,
    author: String,
    description: String,
    enabled: bool,
    client: Option<MediaRendererClient>,
    location: Option<String>,
    media_server_client: Option<MediaServerClient>,
    tracklist: Arc<Mutex<Tracklist>>,
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
            media_server_client: None,
            location: None,
            tracklist: Arc::new(Mutex::new(Tracklist::new(vec![]))),
        }
    }

    pub fn connect_to_media_renderer(
        device: Device,
    ) -> Result<Option<Box<dyn Player + Send>>, Error> {
        let mut player: Self = device.clone().into();
        let location = player.location.clone().unwrap();
        let device_client = futures::executor::block_on(DeviceClient::new(&location).connect())?;
        player.client = Some(MediaRendererClient::new(device_client));
        Ok(Some(Box::new(player)))
    }

    pub fn connect_to_media_server(
        device: Device,
    ) -> Result<Option<Box<dyn Browseable + Send>>, Error> {
        let mut player: Self = device.clone().into();
        let location = player.location.clone().unwrap();
        let device_client = futures::executor::block_on(DeviceClient::new(&location).connect())?;
        player.media_server_client = Some(MediaServerClient::new(device_client));
        Ok(Some(Box::new(player)))
    }

    async fn preload_next_track(&mut self) -> Result<(), Error> {
        if let Some(client) = &self.client {
            let (_, next_tracks) = self.tracklist.lock().unwrap().tracks();
            if let Some(next_track) = next_tracks.first() {
                let content_type = get_content_type(&next_track.uri).await;
                let options = LoadOptions {
                    dlna_features: Some(
                        "DLNA.ORG_OP=01;DLNA.ORG_CI=0;DLNA.ORG_FLAGS=01700000000000000000000000000000"
                            .to_string(),
                    ),
                    content_type,
                    metadata: Some(next_track.clone().into()),
                    object_class: Some(ObjectClass::Audio),
                    ..Default::default()
                };
                client.set_next(&next_track.uri, options).await?;
            }
            return Ok(());
        }
        Err(Error::msg("No device connected"))
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
impl Browseable for Dlna {
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

    fn device_ip(&self) -> String {
        // self.ip.clone()
        todo!()
    }
}

#[async_trait]
impl Player for Dlna {
    async fn play(&mut self) -> Result<(), Error> {
        if let Some(client) = &self.client {
            client.play().await?;
            return Ok(());
        }
        Err(Error::msg("No device connected"))
    }

    async fn pause(&mut self) -> Result<(), Error> {
        if let Some(client) = &self.client {
            client.pause().await?;
            return Ok(());
        }
        Err(Error::msg("No device connected"))
    }

    async fn stop(&mut self) -> Result<(), Error> {
        if let Some(client) = &self.client {
            client.stop().await?;
            return Ok(());
        }
        Err(Error::msg("No device connected"))
    }

    async fn next(&mut self) -> Result<(), Error> {
        if let Some(client) = &self.client {
            if self.tracklist.lock().unwrap().next_track().is_some() {
                client.next().await?;
                tokio::time::sleep(Duration::from_secs(1)).await;
                self.preload_next_track().await?;
            }
            return Ok(());
        }
        Err(Error::msg("No device connected"))
    }

    async fn previous(&mut self) -> Result<(), Error> {
        if let Some(client) = &self.client {
            if self.tracklist.lock().unwrap().previous_track().is_some() {
                let (current_track, _) = self.tracklist.lock().unwrap().current_track();
                let current_track = current_track.unwrap();
                let options = build_load_options(current_track.clone(), &current_track.uri).await;
                client.load(&current_track.uri, options).await?;
                tokio::time::sleep(Duration::from_secs(1)).await;
                self.preload_next_track().await?;
            }
            return Ok(());
        }
        Err(Error::msg("No device connected"))
    }

    async fn seek(&mut self, position: u32) -> Result<(), Error> {
        if let Some(client) = &self.client {
            client.seek(position as u64).await?;
            return Ok(());
        }
        Err(Error::msg("No device connected"))
    }

    async fn load_tracks(
        &mut self,
        tracks: Vec<Track>,
        start_index: Option<i32>,
    ) -> Result<(), Error> {
        self.tracklist = Arc::new(Mutex::new(Tracklist::new(
            tracks.clone().into_iter().map(Into::into).collect(),
        )));

        if let Some(client) = &self.client {
            let start_index = start_index.unwrap_or(0);
            let (current_track, _) = self
                .tracklist
                .lock()
                .unwrap()
                .play_track_at(start_index as usize);
            let current_track = current_track.unwrap();

            let options = build_load_options(current_track.clone(), &current_track.uri).await;

            client.load(&current_track.uri, options).await?;
            // sleep to wait for the track to be loaded
            tokio::time::sleep(Duration::from_secs(3)).await;
            self.preload_next_track().await?;
            return Ok(());
        }
        Err(Error::msg("No device connected"))
    }

    async fn play_next(&mut self, track: Track) -> Result<(), Error> {
        if let Some(client) = &self.client {
            let content_type = get_content_type(&track.uri).await;
            let options = LoadOptions {
                dlna_features: Some(
                    "DLNA.ORG_OP=01;DLNA.ORG_CI=0;DLNA.ORG_FLAGS=01700000000000000000000000000000"
                        .to_string(),
                ),
                content_type,
                metadata: Some(track.clone().into()),
                object_class: Some(ObjectClass::Audio),
                ..Default::default()
            };
            client.set_next(&track.uri, options).await?;
            self.tracklist
                .lock()
                .unwrap()
                .insert_next(track.clone().into());
            return Ok(());
        }
        Err(Error::msg("No device connected"))
    }

    async fn load(&mut self, track: Track) -> Result<(), Error> {
        self.tracklist = Arc::new(Mutex::new(Tracklist::new(vec![track.clone().into()])));
        let options = build_load_options(track.clone(), &track.uri).await;
        if let Some(client) = &self.client {
            self.tracklist.lock().unwrap().play_track_at(0);
            let (current_track, _) = self.tracklist.lock().unwrap().current_track();
            let current_track = current_track.unwrap();
            client.load(&current_track.uri, options).await?;
            return Ok(());
        }
        Err(Error::msg("No device connected"))
    }

    async fn get_current_playback(&mut self) -> Result<Playback, Error> {
        if let Some(client) = &self.client {
            let position = client.get_position().await?;
            let duration = client.get_duration().await?;
            let transport_info = client.get_transport_info().await?;
            if duration != 0 && position >= (duration - 10) && position < (duration - 5) {
                self.preload_next_track().await?;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            if duration != 0 && position >= (duration - 2) {
                self.next().await?;
            }
            let (previous_tracks, next_tracks) = self.tracklist.lock().unwrap().tracks();
            let tracks: Vec<Track> = previous_tracks
                .iter()
                .map(|t| t.clone().into())
                .chain(next_tracks.iter().map(|t| t.clone().into()))
                .collect();
            let items: Vec<(Track, i32)> = tracks
                .iter()
                .enumerate()
                .map(|(i, t)| (t.clone(), (i + 1) as i32))
                .collect();
            let (current_track, index) = self.tracklist.lock().unwrap().current_track();
            return match current_track {
                Some(track) => Ok(Playback {
                    current_track: Some(track.clone().into()),
                    index: index as u32,
                    position_ms: position * 1000 as u32,
                    is_playing: transport_info.current_transport_state == "PLAYING",
                    current_item_id: Some(index as i32),
                    items,
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
        Err(Error::msg("device not connected"))
    }

    fn device_type(&self) -> String {
        "dlna".to_string()
    }

    fn disconnect(&mut self) -> Result<(), Error> {
        self.client = None;
        Ok(())
    }
}

impl From<Device> for Dlna {
    fn from(device: Device) -> Self {
        Self {
            location: device.base_url,
            ..Dlna::new()
        }
    }
}

async fn get_content_type(url: &str) -> Option<String> {
    let req = hyper::Request::builder()
        .method("GET")
        .header("Range", "bytes=0-1000")
        .uri(url)
        .body(hyper::Body::empty())
        .unwrap();
    let client = hyper::Client::new();
    let response = client.request(req).await.unwrap();

    response
        .headers()
        .get("Content-Type")
        .map(|content_type| content_type.to_str().unwrap().to_owned())
}

async fn build_load_options<T>(track: T, url: &str) -> LoadOptions
where
    T: Into<Metadata>,
{
    let content_type = get_content_type(url).await;
    LoadOptions {
        dlna_features: Some(
            "DLNA.ORG_OP=01;DLNA.ORG_CI=0;DLNA.ORG_FLAGS=01700000000000000000000000000000"
                .to_string(),
        ),
        metadata: Some(track.into()),
        content_type,
        object_class: Some(ObjectClass::Audio),
        autoplay: true,
        ..Default::default()
    }
}
