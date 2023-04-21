use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{Addon, Browseable, Player};
use anyhow::Error;
use async_trait::async_trait;
use music_player_tracklist::Tracklist;
use music_player_types::types::{
    Album, Artist, CurrentPlayback, Device, Playback, Playlist, Track, UPNP_DLNA_DEVICE,
};
use tokio::sync::mpsc;
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
    dlna_player: Option<DlnaPlayer>,
    location: Option<String>,
    media_server_client: Option<MediaServerClient>,
    cmd_tx: Option<mpsc::UnboundedSender<DlnaPlayerCommand>>,
    tracklist: Arc<Mutex<Tracklist>>,
    current_playback: Arc<Mutex<CurrentPlayback>>,
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
            dlna_player: None,
            media_server_client: None,
            cmd_tx: None,
            location: None,
            tracklist: Arc::new(Mutex::new(Tracklist::new(vec![]))),
            current_playback: Arc::new(Mutex::new(CurrentPlayback::new())),
        }
    }

    pub fn connect_to_media_renderer(
        device: Device,
    ) -> Result<Option<Box<dyn Player + Send>>, Error> {
        let mut player: Self = device.clone().into();
        let location = player.location.clone().unwrap();
        let device_client = futures::executor::block_on(DeviceClient::new(&location)?.connect())?;
        player.client = Some(MediaRendererClient::new(device_client));

        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<DlnaPlayerCommand>();
        let tracklist = Arc::new(Mutex::new(Tracklist::new(vec![])));

        let current_playback = Arc::new(Mutex::new(CurrentPlayback::new()));
        player.current_playback = current_playback.clone();
        player.cmd_tx = Some(cmd_tx.clone());
        player.dlna_player = Some(DlnaPlayer::new(
            location,
            tracklist,
            current_playback,
            cmd_rx,
        ));
        Ok(Some(Box::new(player)))
    }

    pub fn connect_to_media_server(
        device: Device,
    ) -> Result<Option<Box<dyn Browseable + Send>>, Error> {
        let mut player: Self = device.clone().into();
        let location = player.location.clone().unwrap();
        let device_client = futures::executor::block_on(DeviceClient::new(&location)?.connect())?;
        player.media_server_client = Some(MediaServerClient::new(device_client));
        Ok(Some(Box::new(player)))
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
    async fn albums(
        &mut self,
        filter: Option<String>,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<Album>, Error> {
        if let Some(client) = &self.media_server_client {
            client
                .browse("musicdb://albums", "BrowseDirectChildren")
                .await?;
            let mut result = vec![];
            return Ok(result);
        }
        Err(Error::msg("No device connected"))
    }

    async fn artists(
        &mut self,
        filter: Option<String>,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<Artist>, Error> {
        if let Some(client) = &self.media_server_client {
            client
                .browse("musicdb://artists", "BrowseDirectChildren")
                .await?;
            let result = vec![];
            return Ok(result);
        }
        Err(Error::msg("No device connected"))
    }

    async fn tracks(
        &mut self,
        filter: Option<String>,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<Track>, Error> {
        if let Some(client) = &self.media_server_client {
            client
                .browse("musicdb://songs", "BrowseDirectChildren")
                .await?;
            let result = vec![];
            return Ok(result);
        }
        Err(Error::msg("No device connected"))
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
        if self.client.is_some() {
            self.cmd_tx
                .as_ref()
                .unwrap()
                .send(DlnaPlayerCommand::Play)
                .map_err(|_| Error::msg("Failed to send command"))?;
            return Ok(());
        }
        Err(Error::msg("No device connected"))
    }

    async fn pause(&mut self) -> Result<(), Error> {
        if self.client.is_some() {
            self.cmd_tx
                .as_ref()
                .unwrap()
                .send(DlnaPlayerCommand::Pause)
                .map_err(|_| Error::msg("Failed to send command"))?;
            return Ok(());
        }
        Err(Error::msg("No device connected"))
    }

    async fn stop(&mut self) -> Result<(), Error> {
        if self.client.is_some() {
            self.cmd_tx
                .as_ref()
                .unwrap()
                .send(DlnaPlayerCommand::Stop)
                .map_err(|_| Error::msg("Failed to send command"))?;
            return Ok(());
        }
        Err(Error::msg("No device connected"))
    }

    async fn next(&mut self) -> Result<(), Error> {
        if self.client.is_some() {
            self.cmd_tx
                .as_ref()
                .unwrap()
                .send(DlnaPlayerCommand::Next)
                .map_err(|_| Error::msg("Failed to send command"))?;
            return Ok(());
        }
        Err(Error::msg("No device connected"))
    }

    async fn previous(&mut self) -> Result<(), Error> {
        if self.client.is_some() {
            self.cmd_tx
                .as_ref()
                .unwrap()
                .send(DlnaPlayerCommand::Previous)
                .map_err(|_| Error::msg("Failed to send command"))?;
            return Ok(());
        }
        Err(Error::msg("No device connected"))
    }

    async fn seek(&mut self, position: u32) -> Result<(), Error> {
        if self.client.is_some() {
            self.cmd_tx
                .as_ref()
                .unwrap()
                .send(DlnaPlayerCommand::Seek(position))
                .map_err(|_| Error::msg("Failed to send command"))?;
            return Ok(());
        }
        Err(Error::msg("No device connected"))
    }

    async fn load_tracks(
        &mut self,
        tracks: Vec<Track>,
        start_index: Option<i32>,
    ) -> Result<(), Error> {
        if self.client.is_some() {
            self.cmd_tx
                .as_ref()
                .unwrap()
                .send(DlnaPlayerCommand::LoadTracks(tracks, start_index))
                .map_err(|_| Error::msg("Failed to send command"))?;
            return Ok(());
        }
        Err(Error::msg("No device connected"))
    }

    async fn play_next(&mut self, track: Track) -> Result<(), Error> {
        if self.client.is_some() {
            self.cmd_tx
                .as_ref()
                .unwrap()
                .send(DlnaPlayerCommand::PlayNext(track))
                .map_err(|_| Error::msg("Failed to send command"))?;
            return Ok(());
        }
        Err(Error::msg("No device connected"))
    }

    async fn load(&mut self, track: Track) -> Result<(), Error> {
        if self.client.is_some() {
            self.cmd_tx
                .as_ref()
                .unwrap()
                .send(DlnaPlayerCommand::Load(track))
                .map_err(|_| Error::msg("Failed to send command"))?;
            return Ok(());
        }
        Err(Error::msg("No device connected"))
    }

    async fn get_current_playback(&mut self) -> Result<Playback, Error> {
        if self.client.is_some() {
            let current_playback = self.current_playback.lock().unwrap();
            let playback = current_playback.current.clone();
            return Ok(playback.unwrap_or_default());
        }
        Err(Error::msg("device not connected"))
    }

    async fn get_current_tracklist(&mut self) -> Result<(Vec<Track>, Vec<Track>), Error> {
        if self.client.is_some() {
            let tracklist = self.tracklist.lock().unwrap();
            let (previous_tracks, next_tracks) = tracklist.tracks();
            return Ok((
                previous_tracks.into_iter().map(Into::into).collect(),
                next_tracks.into_iter().map(Into::into).collect(),
            ));
        }
        Err(Error::msg("device not connected"))
    }

    async fn play_track_at(&mut self, position: u32) -> Result<(), Error> {
        if self.client.is_some() {
            self.cmd_tx
                .as_ref()
                .unwrap()
                .send(DlnaPlayerCommand::PlayTrackAt(position))
                .map_err(|_| Error::msg("Failed to send command"))?;
            return Ok(());
        }
        Err(Error::msg("No device connected"))
    }

    async fn remove_track_at(&mut self, position: u32) -> Result<(), Error> {
        if self.client.is_some() {
            self.cmd_tx
                .as_ref()
                .unwrap()
                .send(DlnaPlayerCommand::RemoveTrackAt(position))
                .map_err(|_| Error::msg("Failed to send command"))?;
            return Ok(());
        }
        Err(Error::msg("No device connected"))
    }

    fn device_type(&self) -> String {
        String::from(UPNP_DLNA_DEVICE)
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

#[derive(Debug)]
enum DlnaPlayerCommand {
    Play,
    Pause,
    Stop,
    Next,
    Previous,
    Seek(u32),
    Load(Track),
    LoadTracks(Vec<Track>, Option<i32>),
    PlayNext(Track),
    PlayTrackAt(u32),
    RemoveTrackAt(u32),
}

struct DlnaPlayer {}

impl DlnaPlayer {
    pub fn new(
        location: String,
        tracklist: Arc<Mutex<Tracklist>>,
        current_playback: Arc<Mutex<CurrentPlayback>>,
        mut cmd_rx: mpsc::UnboundedReceiver<DlnaPlayerCommand>,
    ) -> Self {
        let device_client =
            futures::executor::block_on(DeviceClient::new(location.as_str()).unwrap().connect())
                .unwrap();
        let player_internal = Arc::new(Mutex::new(DlnaPlayerInternal {
            client: MediaRendererClient::new(device_client),
            tracklist,
            current_playback,
        }));
        let player_internal_clone = Arc::clone(&player_internal);
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                while let Some(cmd) = cmd_rx.recv().await {
                    player_internal_clone
                        .lock()
                        .unwrap()
                        .handle_command(cmd)
                        .await
                        .unwrap();
                }
            });
        });

        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                loop {
                    player_internal
                        .lock()
                        .unwrap()
                        .update_current_playback()
                        .await;
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
            });
        });

        DlnaPlayer {}
    }
}

#[derive(Clone)]
struct DlnaPlayerInternal {
    client: MediaRendererClient,
    tracklist: Arc<Mutex<Tracklist>>,
    current_playback: Arc<Mutex<CurrentPlayback>>,
}

impl DlnaPlayerInternal {
    pub async fn update_current_playback(&mut self) {
        match self.get_current_playback().await {
            Ok(playback) => {
                let mut current_playback = self.current_playback.lock().unwrap();
                current_playback.current = Some(playback);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    pub async fn get_current_playback(&self) -> Result<Playback, Error> {
        let position = self.client.get_position().await?;
        let duration = self.client.get_duration().await?;
        let transport_info = self.client.get_transport_info().await?;
        if duration != 0 && position >= (duration - 2) {
            self.handle_next().await?;
        }
        let tracklist = self.tracklist.lock().unwrap();
        let (previous_tracks, next_tracks) = tracklist.tracks();
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
        let (current_track, index) = tracklist.current_track();
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

    async fn handle_play(&self) -> Result<(), Error> {
        self.client.play().await
    }

    async fn handle_pause(&self) -> Result<(), Error> {
        self.client.pause().await
    }

    async fn handle_stop(&self) -> Result<(), Error> {
        self.client.stop().await
    }

    async fn handle_next(&self) -> Result<(), Error> {
        let mut tracklist = self.tracklist.lock().unwrap();
        if tracklist.next_track().is_some() {
            drop(tracklist);
            self.client.next().await?;
            tokio::time::sleep(Duration::from_secs(1)).await;
            self.preload_next_track().await?;
        }
        Ok(())
    }

    async fn handle_previous(&self) -> Result<(), Error> {
        let mut tracklist = self.tracklist.lock().unwrap();
        if tracklist.previous_track().is_some() {
            let (current_track, _) = tracklist.current_track();
            let current_track = current_track.unwrap();
            let options = build_load_options(current_track.clone(), &current_track.uri).await;
            self.client.load(&current_track.uri, options).await?;
            drop(tracklist);
            tokio::time::sleep(Duration::from_secs(1)).await;
            self.preload_next_track().await?;
        }
        Ok(())
    }

    async fn handle_seek(&self, position: u32) -> Result<(), Error> {
        self.client.seek(position as u64).await
    }

    async fn handle_load(&mut self, track: Track) -> Result<(), Error> {
        let mut tracklist = self.tracklist.lock().unwrap();
        tracklist.load_tracks(vec![track.clone().into()]);
        let options = build_load_options(track.clone(), &track.uri).await;
        tracklist.play_track_at(0);
        let (current_track, _) = tracklist.current_track();
        let current_track = current_track.unwrap();
        self.client.load(&current_track.uri, options).await?;
        Ok(())
    }

    async fn preload_next_track(&self) -> Result<(), Error> {
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
            self.client.set_next(&next_track.uri, options).await?;
        }
        Ok(())
    }

    async fn handle_load_tracks(
        &self,
        tracks: Vec<Track>,
        start_index: Option<i32>,
    ) -> Result<(), Error> {
        let mut tracklist = self.tracklist.lock().unwrap();
        tracklist.load_tracks(tracks.into_iter().map(Into::into).collect());
        let start_index = start_index.unwrap_or(0);
        let (current_track, _) = tracklist.play_track_at(start_index as usize);
        let current_track = current_track.unwrap();

        let options = build_load_options(current_track.clone(), &current_track.uri).await;

        self.client.load(&current_track.uri, options).await?;
        // sleep to wait for the track to be loaded
        tokio::time::sleep(Duration::from_secs(3)).await;
        drop(tracklist);
        self.preload_next_track().await?;
        Ok(())
    }

    async fn handle_play_next(&self, track: Track) -> Result<(), Error> {
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
        self.client.set_next(&track.uri, options).await?;
        self.tracklist
            .lock()
            .unwrap()
            .insert_next(track.clone().into());
        Ok(())
    }

    async fn handle_play_track_at(&self, _position: u32) -> Result<(), Error> {
        todo!()
    }

    async fn handle_remove_track_at(&self, _position: u32) -> Result<(), Error> {
        todo!()
    }

    pub async fn handle_command(&mut self, cmd: DlnaPlayerCommand) -> Result<(), Error> {
        match cmd {
            DlnaPlayerCommand::Play => self.handle_play().await,
            DlnaPlayerCommand::Pause => self.handle_pause().await,
            DlnaPlayerCommand::Stop => self.handle_stop().await,
            DlnaPlayerCommand::Next => self.handle_next().await,
            DlnaPlayerCommand::Previous => self.handle_previous().await,
            DlnaPlayerCommand::Seek(position) => self.handle_seek(position).await,
            DlnaPlayerCommand::Load(track) => self.handle_load(track).await,
            DlnaPlayerCommand::LoadTracks(tracks, start_index) => {
                self.handle_load_tracks(tracks, start_index).await
            }
            DlnaPlayerCommand::PlayNext(track) => self.handle_play_next(track).await,
            DlnaPlayerCommand::PlayTrackAt(position) => self.handle_play_track_at(position).await,
            DlnaPlayerCommand::RemoveTrackAt(position) => {
                self.handle_remove_track_at(position).await
            }
        }
    }
}
