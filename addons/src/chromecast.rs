extern crate chromecast as rust_cast;

use crate::{Addon, Player};
use anyhow::Error;
use async_trait::async_trait;
use futures_util::Future;
use music_player_types::types::{Album, Artist, Device, Playback, Track, CHROMECAST_DEVICE, CurrentPlayback};
use rust_cast::{
    channels::{
        media::{Image, Media, Metadata, MusicTrackMediaMetadata, StatusEntry, StreamType},
        receiver::CastDeviceApp,
    },
    CastDevice,
};
use std::{
    pin::Pin,
    str::FromStr,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    thread::{self},
    time::Duration,
};
use tokio::sync::mpsc;

const DEFAULT_DESTINATION_ID: &str = "receiver-0";

// const DEFAULT_APP_ID: &str = "CF09BEBE";
const DEFAULT_APP_ID: &str = "34164A08";

pub struct Chromecast<'a> {
    name: String,
    version: String,
    author: String,
    description: String,
    enabled: bool,
    host: Option<String>,
    port: Option<u16>,
    client: Option<CastDevice<'a>>,
    transport_id: Option<String>,
    session_id: Option<String>,
    current_playback: Arc<Mutex<CurrentPlayback>>,
    cast_player: Option<CastPlayer>,
    cmd_tx: Option<mpsc::UnboundedSender<CastPlayerCommand>>,
}

impl<'a> Chromecast<'a> {
    pub fn new() -> Self {
        Self {
            name: "Chromecast".to_string(),
            version: "0.1.0".to_string(),
            author: "Tsiry Sandratraina".to_string(),
            description: "Chromecast addon".to_string(),
            enabled: true,
            host: None,
            port: None,
            client: None,
            transport_id: None,
            session_id: None,
            current_playback: Arc::new(Mutex::new(CurrentPlayback::new())),
            cast_player: None,
            cmd_tx: None,
        }
    }

    pub fn connect(device: Device) -> Result<Option<Box<dyn Player + Send + 'a>>, Error> {
        let mut player: Self = device.clone().into();

        let cast_device = match CastDevice::connect_without_host_verification(
            player.host.clone().unwrap(),
            player.port.unwrap(),
        ) {
            Ok(cast_device) => cast_device,
            Err(err) => panic!("Could not establish connection with Cast Device: {:?}", err),
        };

        cast_device
            .connection
            .connect(DEFAULT_DESTINATION_ID.to_string())?;
        cast_device.heartbeat.ping()?;

        let app_to_run = CastDeviceApp::from_str(DEFAULT_APP_ID).unwrap();
        let app = cast_device.receiver.launch_app(&app_to_run)?;

        cast_device
            .connection
            .connect(app.transport_id.as_str())
            .unwrap();

        player.client = Some(cast_device);
        player.transport_id = Some(app.transport_id);
        player.session_id = Some(app.session_id);

        let current_playback = Arc::new(Mutex::new(CurrentPlayback::new()));
        player.current_playback = current_playback.clone();
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<CastPlayerCommand>();

        player.cmd_tx = Some(cmd_tx.clone());
        player.cast_player = Some(CastPlayer::new(
            player.host.clone().unwrap(),
            player.port.unwrap(),
            current_playback.clone(),
            Arc::new(Mutex::new(cmd_rx)),
        ));

        Ok(Some(Box::new(player)))
    }

    fn reconnect(&mut self) -> Result<(CastDevice, String), Error> {
        let cast_device = match CastDevice::connect_without_host_verification(
            self.host.clone().unwrap(),
            self.port.unwrap(),
        ) {
            Ok(cast_device) => cast_device,
            Err(err) => panic!("Could not establish connection with Cast Device: {:?}", err),
        };

        cast_device
            .connection
            .connect(DEFAULT_DESTINATION_ID.to_string())?;
        cast_device.heartbeat.ping()?;

        let app_to_run = CastDeviceApp::from_str(DEFAULT_APP_ID).unwrap();
        let app = cast_device.receiver.launch_app(&app_to_run)?;

        cast_device
            .connection
            .connect(app.transport_id.as_str())
            .unwrap();

        Ok((cast_device, app.transport_id))
    }

    fn current_app_session(&mut self) -> Result<(CastDevice, String, i32, String), Error> {
        let app_to_manage = CastDeviceApp::from_str(DEFAULT_APP_ID).unwrap();

        let (cast_device, _) = self.reconnect()?;

        cast_device
            .connection
            .connect(DEFAULT_DESTINATION_ID.to_string())
            .unwrap();
        cast_device.heartbeat.ping().unwrap();

        let status = cast_device.receiver.get_status().unwrap();

        let app = status
            .applications
            .iter()
            .find(|app| CastDeviceApp::from_str(app.app_id.as_str()).unwrap() == app_to_manage);

        match app {
            Some(app) => {
                cast_device
                    .connection
                    .connect(app.transport_id.as_str())
                    .unwrap();

                let status = cast_device
                    .media
                    .get_status(app.transport_id.as_str(), None)
                    .unwrap();
                let status = status.entries.first().unwrap();
                let media_session_id = status.media_session_id;
                let transport_id = app.transport_id.as_str();
                Ok((
                    cast_device,
                    transport_id.to_string(),
                    media_session_id,
                    "".to_string(),
                ))
            }
            None => Err(Error::msg(format!("{:?} is not running", app_to_manage))),
        }
    }
}

impl<'a> Addon for Chromecast<'a> {
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
impl<'a> Player for Chromecast<'a> {
    async fn play(&mut self) -> Result<(), Error> {
        self.cmd_tx
            .as_ref()
            .unwrap()
            .send(CastPlayerCommand::Play)
            .unwrap();
        Ok(())
    }

    async fn pause(&mut self) -> Result<(), Error> {
        self.cmd_tx
            .as_ref()
            .unwrap()
            .send(CastPlayerCommand::Pause)
            .unwrap();
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Error> {
        self.cmd_tx
            .as_ref()
            .unwrap()
            .send(CastPlayerCommand::Stop)
            .unwrap();
        Ok(())
    }

    async fn next(&mut self) -> Result<(), Error> {
        self.cmd_tx
            .as_ref()
            .unwrap()
            .send(CastPlayerCommand::Next)
            .unwrap();
        Ok(())
    }

    async fn previous(&mut self) -> Result<(), Error> {
        self.cmd_tx
            .as_ref()
            .unwrap()
            .send(CastPlayerCommand::Previous)
            .unwrap();
        Ok(())
    }

    async fn seek(&mut self, _position: u32) -> Result<(), Error> {
        self.current_app_session()?;
        todo!()
    }

    async fn load_tracks(
        &mut self,
        tracks: Vec<Track>,
        start_index: Option<i32>,
    ) -> Result<(), Error> {
        let media = tracks
            .iter()
            .map(|track| Media {
                content_id: track.uri.clone(),
                content_type: "".to_string(),
                stream_type: StreamType::None,
                metadata: Some(Metadata::MusicTrack(MusicTrackMediaMetadata {
                    title: Some(track.title.clone()),
                    artist: Some(track.artists.first().unwrap().name.clone()),
                    album_name: Some(track.album.as_ref().unwrap().title.clone()),
                    album_artist: Some(track.artists.first().unwrap().name.clone()),
                    track_number: track.track_number,
                    disc_number: Some(track.disc_number),
                    images: match &track.album.as_ref().unwrap().cover {
                        Some(cover) => vec![Image {
                            url: cover.clone(),
                            dimensions: None,
                        }],
                        None => vec![],
                    },
                    release_date: None,
                    composer: None,
                })),
                duration: None,
            })
            .collect::<Vec<Media>>();

        let transport_id = self.transport_id.as_ref().map(|id| id.as_str()).unwrap();

        if let Some(cast_device) = &self.client {
            cast_device.media.queue_load(
                transport_id,
                media.clone(),
                Some(start_index.unwrap_or(0)),
                None,
            )?;
            println!("[chromecast] Tracks loaded");
            println!("[chromecast] Playing track {:#?}", media[0]);
            return Ok(());
        }

        return Err(Error::msg("device not connected"));
    }

    async fn play_next(&mut self, track: Track) -> Result<(), Error> {
        if let Some(cmd_tx) = &self.cmd_tx {
            cmd_tx.send(CastPlayerCommand::PlayNext(track)).unwrap();
            return Ok(());
        }
        Err(Error::msg("Cast device is not connected"))
    }

    async fn load(&mut self, track: Track) -> Result<(), Error> {
        let (cast_device, transport_id, _, session_id) = self.current_app_session()?;

        cast_device.media.load(
            transport_id.as_str(),
            session_id.as_str(),
            &Media {
                content_id: track.uri,
                content_type: "".to_string(),
                stream_type: StreamType::Buffered,
                duration: None,
                metadata: None,
            },
        )?;

        Ok(())
    }

    async fn get_current_playback(&mut self) -> Result<Playback, Error> {
        if self.host.is_none() || self.port.is_none() {
            return Err(Error::msg("No device connected"));
        }

        if self.client.is_none() {
            return Err(Error::msg("Cast device is not connected"));
        }

        let current_playback = self.current_playback.lock().unwrap();
        match &current_playback.current {
            Some(playback) => return Ok(playback.clone()),
            None => return Ok(Playback::default()),
        }
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
        String::from(CHROMECAST_DEVICE)
    }

    fn disconnect(&mut self) -> Result<(), Error> {
        if self.host.is_none() || self.port.is_none() {
            return Err(Error::msg("No device connected"));
        }

        if let Some(cmd_tx) = &self.cmd_tx {
            cmd_tx.send(CastPlayerCommand::Disconnect)?;
            return Ok(());
        }
        Err(Error::msg("Cast device is not connected"))
    }
}

impl<'a> From<Device> for Chromecast<'a> {
    fn from(device: Device) -> Self {
        Self {
            host: Some(device.host),
            port: Some(device.port),
            ..Chromecast::new()
        }
    }
}

#[derive(Debug)]
pub enum CastPlayerCommand {
    Play,
    Pause,
    Stop,
    Next,
    Previous,
    PlayNext(Track),
    Disconnect,
}

struct CastPlayer {}

impl CastPlayer {
    pub fn new(
        host: String,
        port: u16,
        current_playback: Arc<Mutex<CurrentPlayback>>,
        cmd_rx: Arc<Mutex<mpsc::UnboundedReceiver<CastPlayerCommand>>>,
    ) -> CastPlayer {
        thread::spawn(move || {
            let cast_device = match CastDevice::connect_without_host_verification(host, port) {
                Ok(cast_device) => cast_device,
                Err(err) => panic!("Could not establish connection with Cast Device: {:?}", err),
            };

            cast_device
                .connection
                .connect(DEFAULT_DESTINATION_ID.to_string())
                .unwrap();
            cast_device.heartbeat.ping().unwrap();

            let app_to_run = CastDeviceApp::from_str(DEFAULT_APP_ID).unwrap();
            let app = cast_device.receiver.launch_app(&app_to_run).unwrap();

            cast_device
                .connection
                .connect(app.transport_id.as_str())
                .unwrap();

            let internal = CastPlayerInternal {
                cast_device,
                current_playback,
                commands: cmd_rx,
            };
            futures::executor::block_on(internal);
        });
        CastPlayer {}
    }
}

struct CastPlayerInternal<'a> {
    cast_device: CastDevice<'a>,
    current_playback: Arc<Mutex<CurrentPlayback>>,
    commands: Arc<Mutex<mpsc::UnboundedReceiver<CastPlayerCommand>>>,
}

impl<'a> CastPlayerInternal<'a> {
    pub fn get_current_playback(&self) -> Result<Playback, Error> {
        let app_to_manage = CastDeviceApp::from_str(DEFAULT_APP_ID).unwrap();
        let status = self.cast_device.receiver.get_status()?;
        let app = status
            .applications
            .iter()
            .find(|app| CastDeviceApp::from_str(app.app_id.as_str()).unwrap() == app_to_manage);

        if let Some(app) = app {
            self.cast_device
                .connection
                .connect(app.transport_id.as_str())
                .unwrap();

            if let Ok(status) = self
                .cast_device
                .media
                .get_status(app.transport_id.as_str(), None)
            {
                match status.entries.first() {
                    Some(status) => {
                        return self.parse_status(status);
                    }
                    None => {
                        return Ok(Playback {
                            current_track: None,
                            index: 0,
                            position_ms: 0,
                            is_playing: false,
                            current_item_id: None,
                            items: vec![],
                        });
                    }
                }
            }
        }
        return Ok(Playback {
            current_track: None,
            index: 0,
            position_ms: 0,
            is_playing: false,
            current_item_id: None,
            items: vec![],
        });
    }

    fn parse_status(&self, status: &StatusEntry) -> Result<Playback, Error> {
        let media = status.media.as_ref().unwrap();
        let metadata = media.metadata.as_ref().unwrap();
        let items = status.items.as_ref().unwrap();
        let items = items
            .iter()
            .map(|item| {
                let media = item.media.as_ref().unwrap();
                let metadata = media.metadata.as_ref().unwrap();
                let cover = metadata.images.first().map(|x| x.url.clone());
                (
                    Track {
                        id: media
                            .content_id
                            .clone()
                            .split("/")
                            .last()
                            .unwrap()
                            .to_string(),
                        uri: media.content_id.clone(),
                        title: metadata.title.clone().unwrap(),
                        artists: vec![Artist {
                            id: format!("{:x}", md5::compute(metadata.artist.clone().unwrap())),
                            name: metadata.artist.clone().unwrap(),
                            ..Default::default()
                        }],
                        album: Some(Album {
                            id: cover
                                .clone()
                                .map(|x| {
                                    x.split("/")
                                        .last()
                                        .map(|x| x.split(".").next().unwrap())
                                        .unwrap()
                                        .to_string()
                                })
                                .unwrap_or_default(),
                            title: metadata.album_name.clone().unwrap(),
                            cover,
                            ..Default::default()
                        }),
                        track_number: metadata.track_number,
                        disc_number: metadata.disc_number.unwrap_or(0),
                        duration: media.duration,
                        ..Default::default()
                    },
                    item.item_id,
                )
            })
            .collect::<Vec<(Track, i32)>>();

        match metadata {
            Metadata::MusicTrack(metadata) => {
                let cover = metadata.images.first().map(|x| x.url.clone());
                let track = Track {
                    id: media
                        .content_id
                        .clone()
                        .split("/")
                        .last()
                        .unwrap()
                        .to_string(),
                    uri: media.content_id.clone(),
                    title: metadata.title.clone().unwrap(),
                    artists: vec![Artist {
                        id: format!("{:x}", md5::compute(metadata.artist.clone().unwrap())),
                        name: metadata.artist.clone().unwrap(),
                        ..Default::default()
                    }],
                    album: Some(Album {
                        id: cover
                            .clone()
                            .map(|x| {
                                x.split("/")
                                    .last()
                                    .map(|x| x.split(".").next().unwrap())
                                    .unwrap()
                                    .to_string()
                            })
                            .unwrap_or_default(),
                        title: metadata.album_name.clone().unwrap(),
                        cover,
                        ..Default::default()
                    }),
                    track_number: metadata.track_number,
                    disc_number: metadata.disc_number.unwrap_or(0),
                    duration: media.duration,
                    ..Default::default()
                };
                return Ok(Playback {
                    current_track: Some(track),
                    index: 0,
                    position_ms: status
                        .current_time
                        .map(|x| (x * 1000.0) as u32)
                        .unwrap_or(0),
                    is_playing: true,
                    items,
                    current_item_id: status.current_item_id,
                });
            }
            _ => {}
        }

        return Ok(Playback {
            current_track: Some(Track {
                uri: status
                    .media
                    .as_ref()
                    .map(|x| x.content_id.clone().split("/").last().unwrap().to_string())
                    .unwrap_or("".to_string()),
                ..Default::default()
            }),
            index: 0,
            position_ms: status.current_time.map(|x| x as u32).unwrap_or(0),
            is_playing: true,
            current_item_id: status.current_item_id,
            items,
        });
    }

    fn current_app_session(&self) -> Result<(String, i32, String), Error> {
        let app_to_manage = CastDeviceApp::from_str(DEFAULT_APP_ID).unwrap();
        self.cast_device
            .connection
            .connect(DEFAULT_DESTINATION_ID.to_string())
            .unwrap();
        self.cast_device.heartbeat.ping().unwrap();

        let status = self.cast_device.receiver.get_status().unwrap();

        let app = status
            .applications
            .iter()
            .find(|app| CastDeviceApp::from_str(app.app_id.as_str()).unwrap() == app_to_manage);

        match app {
            Some(app) => {
                self.cast_device
                    .connection
                    .connect(app.transport_id.as_str())
                    .unwrap();

                let status = self
                    .cast_device
                    .media
                    .get_status(app.transport_id.as_str(), None)
                    .unwrap();
                let status = status.entries.first().unwrap();
                let media_session_id = status.media_session_id;
                let transport_id = app.transport_id.as_str();
                Ok((transport_id.to_string(), media_session_id, "".to_string()))
            }
            None => Err(Error::msg(format!("{:?} is not running", app_to_manage))),
        }
    }

    fn handle_play(&self) -> Result<(), Error> {
        let (transport_id, media_session_id, _) = self.current_app_session()?;
        self.cast_device
            .media
            .play(transport_id.as_str(), media_session_id)?;
        Ok(())
    }

    fn handle_pause(&self) -> Result<(), Error> {
        let (transport_id, media_session_id, _) = self.current_app_session()?;
        self.cast_device
            .media
            .pause(transport_id.as_str(), media_session_id)?;
        Ok(())
    }

    fn handle_stop(&self) -> Result<(), Error> {
        let (transport_id, media_session_id, _) = self.current_app_session()?;
        self.cast_device
            .media
            .stop(transport_id.as_str(), media_session_id)?;
        Ok(())
    }

    fn handle_next(&self) -> Result<(), Error> {
        let (transport_id, media_session_id, _) = self.current_app_session()?;
        self.cast_device
            .media
            .next(transport_id.as_str(), media_session_id)?;
        Ok(())
    }

    fn handle_previous(&self) -> Result<(), Error> {
        let (transport_id, media_session_id, _) = self.current_app_session()?;
        self.cast_device
            .media
            .previous(transport_id.as_str(), media_session_id)?;
        Ok(())
    }

    fn handle_play_next(&self, track: Track) -> Result<(), Error> {
        let items = vec![Media {
            content_id: track.uri.clone(),
            content_type: "".to_string(),
            stream_type: StreamType::Buffered,
            metadata: Some(Metadata::MusicTrack(MusicTrackMediaMetadata {
                title: Some(track.title.clone()),
                artist: Some(track.artists.first().unwrap().name.clone()),
                album_name: Some(track.album.as_ref().unwrap().title.clone()),
                album_artist: Some(track.artists.first().unwrap().name.clone()),
                track_number: track.track_number,
                disc_number: Some(track.disc_number),
                images: match &track.album.as_ref().unwrap().cover {
                    Some(cover) => vec![Image {
                        url: cover.clone(),
                        dimensions: None,
                    }],
                    None => vec![],
                },
                release_date: None,
                composer: None,
            })),
            duration: None,
        }];
        let playback = self.get_current_playback()?;

        let tracklist = playback.items;
        let mut tracklist = tracklist.iter();
        let mut before: Option<i32> = None;
        loop {
            let cursor = tracklist.next();
            if cursor.is_none() {
                break;
            }
            let (_, item_id) = cursor.unwrap();
            if *item_id == playback.current_item_id.unwrap() {
                let cursor = tracklist.next();
                before = cursor.map(|(_, item_id)| *item_id);
                break;
            }
        }

        let (transport_id, media_session_id, _) = self.current_app_session()?;
        self.cast_device.media.queue_insert(
            transport_id.as_str(),
            media_session_id,
            items,
            before,
        )?;
        return Ok(());
    }

    fn handle_disconnect(&self) -> Result<(), Error> {
        let status = self.cast_device.receiver.get_status().unwrap();

        let current_app = &CastDeviceApp::from_str(DEFAULT_APP_ID).unwrap();

        let app = status
            .applications
            .iter()
            .find(|app| &CastDeviceApp::from_str(app.app_id.as_str()).unwrap() == current_app);
        if let Some(app) = app {
            self.cast_device
                .receiver
                .stop_app(app.session_id.as_str())
                .unwrap();
        }
        Ok(())
    }

    pub fn handle_command(&self, cmd: CastPlayerCommand) -> Result<(), Error> {
        match cmd {
            CastPlayerCommand::Play => self.handle_play(),
            CastPlayerCommand::Pause => self.handle_pause(),
            CastPlayerCommand::Stop => self.handle_stop(),
            CastPlayerCommand::Next => self.handle_next(),
            CastPlayerCommand::Previous => self.handle_previous(),
            CastPlayerCommand::PlayNext(track) => self.handle_play_next(track),
            CastPlayerCommand::Disconnect => self.handle_disconnect(),
        }
    }
}

impl<'a> Future for CastPlayerInternal<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<()> {
        loop {
            // Process commands that have been sent to the player
            let cmd = match self.commands.lock().unwrap().poll_recv(ctx) {
                Poll::Ready(None) => return Poll::Ready(()), // client has disconnected - shut down.
                Poll::Ready(Some(cmd)) => Some(cmd),
                _ => None,
            };

            if let Some(cmd) = cmd {
                if let Err(e) = self.handle_command(cmd) {
                    println!("{:?}", e);
                }
            }

            match self.get_current_playback() {
                Ok(playback) => {
                    let mut current_playback = self.current_playback.lock().unwrap();
                    current_playback.current = Some(playback);
                }
                Err(e) => {
                    println!("{:?}", e);
                }
            }
            thread::sleep(Duration::from_millis(500));
        }
    }
}
