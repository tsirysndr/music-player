use std::time::Duration;

use local_ip_addr::get_local_ip_address;
use mdns_sd::ServiceInfo;
use music_player_discovery::SERVICE_NAME;
use music_player_settings::{read_settings, Settings};
use rockbox_metadata::Metadata as AudioMetadata;
use upnp_client::types::Metadata;
use url::Url;

pub const CHROMECAST_SERVICE_NAME: &str = "_googlecast._tcp.local.";

pub const CHROMECAST_DEVICE: &str = "Chromecast";
pub const MUSIC_PLAYER_DEVICE: &str = "MusicPlayer";
pub const UPNP_DLNA_DEVICE: &str = "UPnP/DLNA";
pub const SUBSONIC_DEVICE: &str = "Subsonic";
pub const JELLYFIN_DEVICE: &str = "Jellyfin";

#[derive(Debug, Clone, Default)]
pub struct Playback {
    pub current_track: Option<Track>,
    pub index: u32,
    pub current_item_id: Option<i32>,
    pub position_ms: u32,
    pub is_playing: bool,
    pub items: Vec<(Track, i32)>,
}

pub struct CurrentPlayback {
    pub current: Option<Playback>,
}

impl CurrentPlayback {
    pub fn new() -> Self {
        Self { current: None }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Song {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub year: Option<u32>,
    pub track: Option<u32>,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u8>,
    pub channels: Option<u8>,
    pub duration: Duration,
    pub uri: Option<String>,
    pub cover: Option<String>,
    pub album_artist: String,
}

#[derive(Debug, Clone, Default)]
pub struct SimplifiedSong {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub duration: Duration,
    pub cover: Option<String>,
    pub artist_id: String,
    pub album_id: String,
}

#[derive(Debug, Clone, Default)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub artist_id: Option<String>,
    pub year: Option<u32>,
    pub cover: Option<String>,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, Default)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub picture: Option<String>,
    pub albums: Vec<Album>,
    pub songs: Vec<Track>,
}

fn tag_or_none(value: &str) -> String {
    if value.is_empty() {
        "None".to_string()
    } else {
        value.to_string()
    }
}

fn album_artist_of(meta: &AudioMetadata) -> String {
    if !meta.albumartist.is_empty() {
        meta.albumartist.clone()
    } else if !meta.artist.is_empty() {
        meta.artist.clone()
    } else {
        "None".to_string()
    }
}

impl From<&AudioMetadata> for Song {
    fn from(meta: &AudioMetadata) -> Self {
        Self {
            title: tag_or_none(&meta.title),
            artist: tag_or_none(&meta.artist),
            album: tag_or_none(&meta.album),
            genre: tag_or_none(&meta.genre),
            year: meta.year,
            track: meta.track_number,
            bitrate: (meta.bitrate > 0).then(|| meta.bitrate),
            sample_rate: (meta.sample_rate > 0).then(|| meta.sample_rate),
            bit_depth: None,
            channels: None,
            duration: meta.duration,
            album_artist: album_artist_of(meta),
            ..Default::default()
        }
    }
}

impl From<&AudioMetadata> for Artist {
    fn from(meta: &AudioMetadata) -> Self {
        let name = album_artist_of(meta);
        let id = format!("{:x}", md5::compute(&name));
        Self {
            id,
            name,
            ..Default::default()
        }
    }
}

impl From<&AudioMetadata> for Album {
    fn from(meta: &AudioMetadata) -> Self {
        let title = tag_or_none(&meta.album);
        let artist = album_artist_of(meta);
        let id = format!("{:x}", md5::compute(&title));
        let artist_id = Some(format!("{:x}", md5::compute(&artist)));
        Self {
            id,
            title,
            artist,
            year: meta.year,
            artist_id,
            ..Default::default()
        }
    }
}

#[derive(Default, Clone)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub host: String,
    pub ip: String,
    pub port: u16,
    pub service: String,
    pub app: String,
    pub is_connected: bool,
    pub base_url: Option<String>,
    pub is_cast_device: bool,
    pub is_source_device: bool,
    pub is_current_device: bool,
}

impl Device {
    pub fn with_base_url(&mut self, base_url: Option<String>) -> Self {
        self.base_url = base_url;
        self.clone()
    }

    /// Builds a statically-configured source device pointing at a music
    /// streaming server (Subsonic/Navidrome, Jellyfin, ...).
    ///
    /// `app` is the addon identifier ("subsonic" or "jellyfin") and
    /// `base_url` the root url of the server (e.g. "https://music.example.com").
    /// Returns `None` when the url is empty or cannot be parsed.
    pub fn from_streaming_server(app: &str, base_url: &str) -> Option<Self> {
        let base_url = base_url.trim().trim_end_matches('/').to_string();
        if base_url.is_empty() {
            return None;
        }
        let url = Url::parse(&base_url).ok()?;
        let host = url.host_str()?.to_string();
        let port = url.port_or_known_default().unwrap_or(80);
        Some(Self {
            id: format!("{:x}", md5::compute(&base_url)),
            name: host.clone(),
            host: host.clone(),
            ip: host,
            port,
            service: app.to_owned(),
            app: app.to_owned(),
            is_connected: false,
            base_url: Some(base_url),
            is_cast_device: false,
            is_source_device: true,
            is_current_device: false,
        })
    }
}

impl From<ServiceInfo> for Device {
    fn from(srv: ServiceInfo) -> Self {
        if srv.get_fullname().contains(SERVICE_NAME) {
            let device_id = srv
                .get_fullname()
                .replace(SERVICE_NAME, "")
                .split("-")
                .collect::<Vec<&str>>()[1]
                .replace(".", "")
                .to_owned();

            let config = read_settings().unwrap();
            let settings = config.try_deserialize::<Settings>().unwrap();

            let is_current_device = device_id == settings.device_id
                && srv.get_fullname().split("-").collect::<Vec<&str>>()[0].to_owned() == "http";

            let mut addresses = srv.get_addresses().iter();
            let mut ip = addresses.next().unwrap().to_string();

            if is_current_device {
                ip = get_local_ip_address().unwrap();
            }

            return Self {
                id: device_id.clone(),
                name: srv
                    .get_properties()
                    .get("device_name")
                    .unwrap_or(&device_id.clone())
                    .to_owned(),
                host: srv
                    .get_hostname()
                    .split_at(srv.get_hostname().len() - 1)
                    .0
                    .to_owned(),
                ip,
                port: srv.get_port(),
                service: srv.get_fullname().split("-").collect::<Vec<&str>>()[0].to_owned(),
                app: "music-player".to_owned(),
                is_connected: false,
                base_url: None,
                is_cast_device: true,
                is_source_device: true,
                is_current_device,
            };
        }

        if srv.get_fullname().contains(CHROMECAST_SERVICE_NAME) {
            return Self {
                id: srv.get_properties().get("id").unwrap().to_owned(),
                name: srv.get_properties().get("fn").unwrap().to_owned(),
                host: srv
                    .get_hostname()
                    .split_at(srv.get_hostname().len() - 1)
                    .0
                    .to_owned(),
                ip: srv.get_addresses().iter().next().unwrap().to_string(),
                port: srv.get_port(),
                service: srv.get_fullname().to_owned(),
                app: "chromecast".to_owned(),
                is_connected: false,
                base_url: None,
                is_cast_device: true,
                is_source_device: false,
                is_current_device: false,
            };
        }

        Self {
            ..Default::default()
        }
    }
}

impl From<upnp_client::types::Device> for Device {
    fn from(device: upnp_client::types::Device) -> Self {
        let (host, port) = Url::parse(&device.location)
            .map(|url| {
                let host = url.host_str().unwrap();
                let port = url.port().unwrap();
                (host.to_string(), port)
            })
            .unwrap();
        let is_cast_device = device
            .device_type
            .contains("urn:schemas-upnp-org:device:MediaRenderer");
        let is_source_device = device
            .device_type
            .contains("urn:schemas-upnp-org:device:MediaServer");

        Self {
            id: device.udn,
            name: device.friendly_name,
            host: host.clone(),
            ip: host.clone(),
            port,
            service: device.device_type,
            app: "dlna".to_owned(),
            is_connected: false,
            base_url: Some(device.location),
            is_cast_device,
            is_source_device,
            is_current_device: false,
        }
    }
}

pub trait Connected {
    fn is_connected(&self, current: Option<&Device>) -> Self;
}

impl Connected for Device {
    fn is_connected(&self, current: Option<&Device>) -> Self {
        match current {
            Some(current) => Self {
                is_connected: self.id == current.id,
                ..self.clone()
            },
            None => Self {
                is_connected: false,
                ..self.clone()
            },
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub duration: Option<f32>,
    pub disc_number: u32,
    pub track_number: Option<u32>,
    pub uri: String,
    pub artists: Vec<Artist>,
    pub album: Option<Album>,
    pub artist: String,
}

#[derive(Default, Clone)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub tracks: Vec<Track>,
}

#[derive(Default, Clone)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub playlists: Vec<Playlist>,
}

impl Into<Metadata> for Track {
    fn into(self) -> Metadata {
        Metadata {
            title: self.title,
            artist: Some(self.artist),
            album: self.album.clone().map(|a| a.title),
            album_art_uri: self.album.map(|a| a.cover.unwrap()),
            ..Default::default()
        }
    }
}

pub trait RemoteTrackUrl {
    fn with_remote_track_url(&self, base_url: &str) -> Self;
}

pub trait RemoteCoverUrl {
    fn with_remote_cover_url(&self, base_url: &str) -> Self;
}

impl RemoteTrackUrl for Track {
    fn with_remote_track_url(&self, base_url: &str) -> Self {
        // Tracks coming from streaming servers (Subsonic, Jellyfin, ...)
        // already carry an authenticated absolute stream url; keep it as is.
        if self.uri.starts_with("http://") || self.uri.starts_with("https://") {
            return self.clone();
        }
        Self {
            uri: format!("{}/tracks/{}", base_url, self.id),
            ..self.clone()
        }
    }
}

impl RemoteCoverUrl for Track {
    fn with_remote_cover_url(&self, base_url: &str) -> Self {
        Self {
            album: match self.album {
                Some(ref album) => Some(album.with_remote_cover_url(base_url)),
                None => None,
            },
            ..self.clone()
        }
    }
}

impl RemoteCoverUrl for Album {
    fn with_remote_cover_url(&self, base_url: &str) -> Self {
        let cover_url = match self.cover {
            Some(ref cover) => match cover.starts_with("http") {
                true => Some(cover.to_owned()),
                false => Some(format!("{}/covers/{}", base_url, cover)),
            },
            None => None,
        };
        Self {
            cover: cover_url,
            tracks: self
                .tracks
                .iter()
                .map(|track| track.with_remote_cover_url(base_url))
                .collect(),
            ..self.clone()
        }
    }
}

impl RemoteTrackUrl for Album {
    fn with_remote_track_url(&self, base_url: &str) -> Self {
        Self {
            tracks: self
                .tracks
                .iter()
                .map(|track| track.with_remote_track_url(base_url))
                .collect(),
            ..self.clone()
        }
    }
}

impl RemoteCoverUrl for Artist {
    fn with_remote_cover_url(&self, base_url: &str) -> Self {
        Self {
            albums: self
                .albums
                .iter()
                .map(|album| album.with_remote_cover_url(base_url))
                .collect(),
            songs: self
                .songs
                .iter()
                .map(|track| track.with_remote_cover_url(base_url))
                .collect(),
            ..self.clone()
        }
    }
}

impl RemoteTrackUrl for Artist {
    fn with_remote_track_url(&self, base_url: &str) -> Self {
        Self {
            songs: self
                .songs
                .iter()
                .map(|track| track.with_remote_track_url(base_url))
                .collect(),
            ..self.clone()
        }
    }
}

impl RemoteTrackUrl for Playlist {
    fn with_remote_track_url(&self, base_url: &str) -> Self {
        Self {
            tracks: self
                .tracks
                .iter()
                .map(|track| track.with_remote_track_url(base_url))
                .collect(),
            ..self.clone()
        }
    }
}

impl RemoteCoverUrl for Playlist {
    fn with_remote_cover_url(&self, base_url: &str) -> Self {
        Self {
            tracks: self
                .tracks
                .iter()
                .map(|track| track.with_remote_cover_url(base_url))
                .collect(),
            ..self.clone()
        }
    }
}
