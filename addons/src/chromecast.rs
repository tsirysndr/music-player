use crate::{Addon, Player};
use anyhow::Error;
use async_trait::async_trait;
use music_player_types::types::{Device, Track};
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

pub struct Chromecast {
    name: String,
    version: String,
    author: String,
    description: String,
    enabled: bool,
    host: Option<String>,
    port: Option<u16>,
}

impl Chromecast {
    pub fn new() -> Self {
        Self {
            name: "Chromecast".to_string(),
            version: "0.1.0".to_string(),
            author: "Tsiry Sandratraina".to_string(),
            description: "Chromecast addon".to_string(),
            enabled: true,
            host: None,
            port: None,
        }
    }

    pub fn connect(&mut self, device: Device) -> Result<Option<Box<dyn Player + Send>>, Error> {
        let player: Self = device.clone().into();
        Ok(Some(Box::new(player)))
    }

    fn connect_without_host_verification(
        &mut self,
        app_to_run: Option<String>,
    ) -> Result<(CastDevice, String, i32, String), Error> {
        let cast_device = match CastDevice::connect_without_host_verification(
            self.host.as_ref().unwrap(),
            self.port.unwrap(),
        ) {
            Ok(cast_device) => cast_device,
            Err(err) => panic!("Could not establish connection with Cast Device: {:?}", err),
        };

        cast_device
            .connection
            .connect(DEFAULT_DESTINATION_ID.to_string())
            .unwrap();
        cast_device.heartbeat.ping().unwrap();

        if let Some(app_to_run) = app_to_run {
            let app_to_run = CastDeviceApp::from_str(app_to_run.as_str()).unwrap();
            let app = cast_device.receiver.launch_app(&app_to_run)?;

            cast_device
                .connection
                .connect(app.transport_id.as_str())
                .unwrap();

            return Ok((cast_device, app.transport_id, 0, app.session_id));
        }

        let app_to_manage = CastDeviceApp::from_str(DEFAULT_APP_ID).unwrap();
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
                let media_session_id = status.to_owned().media_session_id;
                let transport_id = app.to_owned().transport_id;
                Ok((cast_device, transport_id, media_session_id, "".to_string()))
            }
            None => Err(Error::msg(format!("{:?} is not running", app_to_manage))),
        }
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
        let (cast_device, transport_id, media_session_id, _) =
            self.connect_without_host_verification(None)?;
        cast_device
            .media
            .play(transport_id.as_str(), media_session_id)?;
        Ok(())
    }

    async fn pause(&mut self) -> Result<(), Error> {
        let (cast_device, transport_id, media_session_id, _) =
            self.connect_without_host_verification(None)?;
        cast_device
            .media
            .pause(transport_id.as_str(), media_session_id)?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Error> {
        let (cast_device, transport_id, media_session_id, _) =
            self.connect_without_host_verification(None)?;
        cast_device
            .media
            .stop(transport_id.as_str(), media_session_id)?;
        Ok(())
    }

    async fn next(&mut self) -> Result<(), Error> {
        let (cast_device, transport_id, media_session_id, _) =
            self.connect_without_host_verification(None)?;
        cast_device
            .media
            .next(transport_id.as_str(), media_session_id)?;
        Ok(())
    }

    async fn previous(&mut self) -> Result<(), Error> {
        let (cast_device, transport_id, media_session_id, _) =
            self.connect_without_host_verification(None)?;
        cast_device
            .media
            .previous(transport_id.as_str(), media_session_id)?;
        Ok(())
    }

    async fn seek(&mut self, _position: u32) -> Result<(), Error> {
        self.connect_without_host_verification(None)?;
        todo!()
    }

    async fn load_tracks(&mut self, tracks: Vec<Track>) -> Result<(), Error> {
        let (cast_device, transport_id, _, _) =
            self.connect_without_host_verification(Some(DEFAULT_APP_ID.to_owned()))?;

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

        cast_device
            .media
            .queue_load(transport_id.as_str(), medias, Some(0), None)?;

        Ok(())
    }

    async fn play_next(&mut self, track: Track) -> Result<(), Error> {
        self.connect_without_host_verification(None)?;
        todo!()
    }

    async fn load(&mut self, track: Track) -> Result<(), Error> {
        let (cast_device, transport_id, _, session_id) =
            self.connect_without_host_verification(Some(DEFAULT_APP_ID.to_owned()))?;

        println!("Loading track: {}", track.uri);

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

    fn device_type(&self) -> String {
        "chromecast".to_string()
    }
}

impl From<Device> for Chromecast {
    fn from(device: Device) -> Self {
        Self {
            host: Some(device.host),
            port: Some(device.port),
            ..Chromecast::new()
        }
    }
}
