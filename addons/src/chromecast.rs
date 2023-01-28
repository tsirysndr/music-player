use crate::{Addon, Player};
use anyhow::Error;
use async_trait::async_trait;
use music_player_types::types::{Album, Artist, Device, Playback, Track};
use rust_cast::{
    channels::{
        media::{Image, Media, Metadata, MusicTrackMediaMetadata, StreamType},
        receiver::CastDeviceApp,
    },
    CastDevice,
};
use std::str::FromStr;

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
        }
    }

    pub fn connect(device: Device) -> Result<Option<Box<dyn Player + Send + 'a>>, Error> {
        let mut player: Self = device.clone().into();
        // player.connect_without_host_verification(Some(DEFAULT_APP_ID.to_owned()))?;

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

        Ok(Some(Box::new(player)))
    }

    fn current_app_session(&mut self) -> Result<(&CastDevice, String, i32, String), Error> {
        let app_to_manage = CastDeviceApp::from_str(DEFAULT_APP_ID).unwrap();
        if let Some(cast_device) = &self.client {
            let status = cast_device.receiver.get_status().unwrap();

            let app = status
                .applications
                .iter()
                .find(|app| CastDeviceApp::from_str(app.app_id.as_str()).unwrap() == app_to_manage);

            match app {
                Some(app) => {
                    cast_device
                        .clone()
                        .connection
                        .connect(app.transport_id.as_str())
                        .unwrap();

                    let status = cast_device
                        .media
                        .get_status(app.transport_id.as_str(), None)
                        .unwrap();
                    let status = status.entries.first().unwrap();
                    let media_session_id = status.to_owned().media_session_id;
                    let transport_id = app.to_owned().transport_id;
                    Ok((cast_device, transport_id, media_session_id, "".to_string()))
                }
                None => Err(Error::msg(format!("{:?} is not running", app_to_manage))),
            }
        } else {
            Err(Error::msg("Cast device is not connected"))
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
        let (cast_device, transport_id, media_session_id, _) = self.current_app_session()?;
        cast_device
            .media
            .play(transport_id.as_str(), media_session_id)?;
        Ok(())
    }

    async fn pause(&mut self) -> Result<(), Error> {
        let (cast_device, transport_id, media_session_id, _) = self.current_app_session()?;
        cast_device
            .media
            .pause(transport_id.as_str(), media_session_id)?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Error> {
        let (cast_device, transport_id, media_session_id, _) = self.current_app_session()?;
        cast_device
            .media
            .stop(transport_id.as_str(), media_session_id)?;
        Ok(())
    }

    async fn next(&mut self) -> Result<(), Error> {
        let (cast_device, transport_id, media_session_id, _) = self.current_app_session()?;
        cast_device
            .media
            .next(transport_id.as_str(), media_session_id)?;
        Ok(())
    }

    async fn previous(&mut self) -> Result<(), Error> {
        let (cast_device, transport_id, media_session_id, _) = self.current_app_session()?;
        cast_device
            .media
            .previous(transport_id.as_str(), media_session_id)?;
        Ok(())
    }

    async fn seek(&mut self, _position: u32) -> Result<(), Error> {
        self.current_app_session()?;
        todo!()
    }

    async fn load_tracks(&mut self, tracks: Vec<Track>) -> Result<(), Error> {
        if let Some(cast_device) = &self.client {
            let medias = tracks
                .iter()
                .map(|track| Media {
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
                })
                .collect::<Vec<Media>>();

            cast_device.media.queue_load(
                self.transport_id.as_ref().unwrap(),
                medias,
                Some(0),
                None,
            )?;

            return Ok(());
        }
        Err(Error::msg("Cast device is not connected"))
    }

    async fn play_next(&mut self, track: Track) -> Result<(), Error> {
        self.current_app_session()?;
        todo!()
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

        if let Some(cast_device) = &self.client {
            let app_to_manage = CastDeviceApp::from_str(DEFAULT_APP_ID).unwrap();
            let status = cast_device.receiver.get_status().unwrap();
            let app = status
                .applications
                .iter()
                .find(|app| CastDeviceApp::from_str(app.app_id.as_str()).unwrap() == app_to_manage);

            if let Some(app) = app {
                cast_device
                    .connection
                    .connect(app.transport_id.as_str())
                    .unwrap();

                let status = cast_device
                    .media
                    .get_status(app.transport_id.as_str(), None)
                    .unwrap();
                match status.entries.first() {
                    Some(status) => {
                        let media = status.media.as_ref().unwrap();
                        let metadata = media.metadata.as_ref().unwrap();

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
                                        id: format!(
                                            "{:x}",
                                            md5::compute(metadata.artist.clone().unwrap())
                                        ),
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
                                });
                            }
                            _ => {}
                        }

                        return Ok(Playback {
                            current_track: Some(Track {
                                uri: status
                                    .media
                                    .as_ref()
                                    .map(|x| {
                                        x.content_id.clone().split("/").last().unwrap().to_string()
                                    })
                                    .unwrap_or("".to_string()),
                                ..Default::default()
                            }),
                            index: 0,
                            position_ms: status.current_time.map(|x| x as u32).unwrap_or(0),
                            is_playing: true,
                        });
                    }
                    None => {
                        return Ok(Playback {
                            current_track: None,
                            index: 0,
                            position_ms: 0,
                            is_playing: false,
                        });
                    }
                }
            }
        }
        Err(Error::msg("Cast device is not connected"))
    }

    fn device_type(&self) -> String {
        "chromecast".to_string()
    }

    fn disconnect(&mut self) -> Result<(), Error> {
        if self.host.is_none() || self.port.is_none() {
            return Err(Error::msg("No device connected"));
        }

        if let Some(cast_device) = &self.client {
            let status = cast_device.receiver.get_status().unwrap();

            let current_app = &CastDeviceApp::from_str(DEFAULT_APP_ID).unwrap();

            let app = status
                .applications
                .iter()
                .find(|app| &CastDeviceApp::from_str(app.app_id.as_str()).unwrap() == current_app);
            if let Some(app) = app {
                cast_device
                    .receiver
                    .stop_app(app.session_id.as_str())
                    .unwrap();
            }
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
