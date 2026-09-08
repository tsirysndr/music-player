#[cfg(test)]
mod tests;
use crate::simple_broker::SimpleBroker;
use anyhow::Error;
use async_graphql::Schema;
use futures_util::StreamExt;
use music_player_discovery::{discover, SERVICE_NAME};
use music_player_entity::track as track_entity;
use music_player_playback::player::PlayerCommand;
use music_player_renderer::Player;
use music_player_settings::{read_settings, Settings};
use music_player_types::types::RemoteCoverUrl;
use music_player_types::types::RemoteTrackUrl;
use music_player_types::types::{Device, CHROMECAST_SERVICE_NAME};
use rand::seq::SliceRandom;
use schema::{Mutation, Query, Subscription};
use std::{
    sync::{Arc, Mutex},
    thread,
};
use tokio::sync::mpsc::UnboundedSender;
use upnp_client::discovery::discover_pnp_locations;
use url::Url;

pub mod schema;
pub mod simple_broker;

pub type MusicPlayerSchema = Schema<Query, Mutation, Subscription>;

const MEDIA_RENDERER: &str = "urn:schemas-upnp-org:device:MediaRenderer";
const MEDIA_SERVER: &str = "urn:schemas-upnp-org:device:MediaServer";

fn scan_mp_devices(mp_devices: Arc<Mutex<Vec<Device>>>) {
    thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let services = discover(SERVICE_NAME);
            tokio::pin!(services);
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            while let Some(info) = services.next().await {
                let device = Device::from(info.clone());
                if insert_unique(&mp_devices, device.clone()) {
                    SimpleBroker::<Device>::publish(device);
                }
            }
        });
    });
}

/// Add a discovered device, unless it is already listed.
///
/// mDNS re-announces periodically and SSDP answers more than once, so every
/// discovery loop sees the same device repeatedly — without this the picker
/// fills up with copies of one Chromecast. Keyed on id *and* service because a
/// music-player peer legitimately advertises two records (gRPC and HTTP) under
/// one id.
///
/// Returns whether it was new, so the caller only broadcasts a device the
/// clients have not already been told about.
fn insert_unique(devices: &Arc<Mutex<Vec<Device>>>, device: Device) -> bool {
    let mut devices = devices.lock().unwrap();
    if devices
        .iter()
        .any(|existing| existing.id == device.id && existing.service == device.service)
    {
        return false;
    }
    devices.push(device);
    true
}

fn scan_chromecast_devices(devices: Arc<Mutex<Vec<Device>>>) {
    thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let services = discover(CHROMECAST_SERVICE_NAME);
            tokio::pin!(services);
            while let Some(info) = services.next().await {
                let device = Device::from(info.clone());
                if insert_unique(&devices, device.clone()) {
                    SimpleBroker::<Device>::publish(device);
                }
            }
        });
    });
}

fn add_configured_streaming_devices(devices: Arc<Mutex<Vec<Device>>>) {
    let settings = match read_settings() {
        Ok(config) => match config.try_deserialize::<Settings>() {
            Ok(settings) => settings,
            Err(_) => return,
        },
        Err(_) => return,
    };

    let servers = [
        ("subsonic", settings.subsonic_url),
        ("jellyfin", settings.jellyfin_url),
    ];

    for (app, url) in servers {
        if let Some(url) = url {
            if let Some(device) = Device::from_streaming_server(app, &url) {
                if insert_unique(&devices, device.clone()) {
                    SimpleBroker::<Device>::publish(device);
                }
            }
        }
    }
}

fn scan_upnp_dlna_devices(devices: Arc<Mutex<Vec<Device>>>) {
    thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            if let Ok(upnp_devices) = discover_pnp_locations().await {
                tokio::pin!(upnp_devices);

                while let Some(device) = upnp_devices.next().await {
                    if device.device_type.contains(MEDIA_RENDERER)
                        || device.device_type.contains(MEDIA_SERVER)
                    {
                        let device = Device::from(device.clone());
                        if insert_unique(&devices, device.clone()) {
                            SimpleBroker::<Device>::publish(device);
                        }
                    }
                }
            }
        });
    });
}

pub async fn scan_devices() -> Result<Arc<std::sync::Mutex<Vec<Device>>>, Box<dyn std::error::Error>>
{
    let devices: Arc<std::sync::Mutex<Vec<Device>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
    let mp_devices = Arc::clone(&devices);
    let chromecast_devices = Arc::clone(&devices);
    let dlna_devices = Arc::clone(&devices);

    scan_mp_devices(mp_devices);

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    scan_chromecast_devices(chromecast_devices);

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    scan_upnp_dlna_devices(dlna_devices);

    add_configured_streaming_devices(Arc::clone(&devices));

    Ok(devices)
}

pub async fn load_tracks(
    player_cmd: &Arc<Mutex<UnboundedSender<PlayerCommand>>>,
    player: Option<&mut Box<dyn Player + Send>>,
    source_ip: Option<String>,
    mut tracks: Vec<track_entity::Model>,
    position: Option<u32>,
    shuffle: bool,
) -> Result<(), Error> {
    if shuffle {
        tracks.shuffle(&mut rand::thread_rng());
    }
    if let Some(player) = player {
        if player.device_type() == "chromecast" {
            if let Some(source_ip) = source_ip {
                tracks = tracks
                    .into_iter()
                    .map(|mut track| {
                        let url = Url::parse(track.uri.as_str()).unwrap();
                        let host = url.host_str().unwrap();
                        track.uri = track.uri.to_lowercase().replace(host, source_ip.as_str());
                        let cover = match track.clone().album.cover {
                            Some(cover) => Url::parse(cover.as_str()).ok().map(|url| {
                                let host = url.host_str().unwrap();
                                cover.to_lowercase().replace(host, source_ip.as_str())
                            }),
                            None => None,
                        };
                        track.album.cover = cover;
                        track
                    })
                    .collect();
            }
        }
        player
            .load_tracks(
                tracks.clone().into_iter().map(Into::into).collect(),
                Some(0),
            )
            .await?;
        return Ok(());
    }
    let player_cmd_tx = player_cmd.lock().unwrap();
    player_cmd_tx.send(PlayerCommand::Stop).unwrap();
    player_cmd_tx.send(PlayerCommand::Clear).unwrap();
    player_cmd_tx
        .send(PlayerCommand::LoadTracklist { tracks })
        .unwrap();
    player_cmd_tx
        .send(PlayerCommand::PlayTrackAt(position.unwrap_or(0) as usize))
        .unwrap();
    Ok(())
}

pub fn update_tracks_url<T: RemoteCoverUrl + RemoteTrackUrl>(
    devices: Vec<Device>,
    result: T,
    will_play_on_chromecast: bool,
) -> Result<T, Error> {
    let base_url = match devices
        .clone()
        .into_iter()
        .find(|device| device.is_current_device)
    {
        Some(device) => {
            let host = match will_play_on_chromecast {
                true => device.ip,
                false => device.host,
            };
            Some(format!("http://{}:{}", host, device.port))
        }
        None => None,
    };

    match base_url.is_some() {
        true => {
            let base_url = base_url.unwrap();
            Ok(result
                .with_remote_cover_url(&base_url)
                .with_remote_track_url(&base_url))
        }
        false => Err(Error::msg("Cannot find current device")),
    }
}

pub fn update_track_url<T: RemoteTrackUrl>(
    devices: Vec<Device>,
    result: T,
    will_play_on_chromecast: bool,
) -> Result<T, Error> {
    let base_url = match devices
        .clone()
        .into_iter()
        .find(|device| device.is_current_device)
    {
        Some(device) => {
            let host = match will_play_on_chromecast {
                true => device.ip,
                false => device.host,
            };
            Some(format!("http://{}:{}", host, device.port))
        }
        None => None,
    };

    match base_url.is_some() {
        true => {
            let base_url = base_url.unwrap();
            Ok(result.with_remote_track_url(&base_url))
        }
        false => Err(Error::msg("Cannot find current device")),
    }
}

pub fn update_cover_url<T: RemoteCoverUrl>(
    devices: Vec<Device>,
    result: T,
    will_play_on_chromecast: bool,
) -> Result<T, Error> {
    let base_url = match devices
        .clone()
        .into_iter()
        .find(|device| device.is_current_device)
    {
        Some(device) => {
            let host = match will_play_on_chromecast {
                true => device.ip,
                false => device.host,
            };
            Some(format!("http://{}:{}", host, device.port))
        }
        None => None,
    };

    match base_url.is_some() {
        true => {
            let base_url = base_url.unwrap();
            Ok(result.with_remote_cover_url(&base_url))
        }
        false => Err(Error::msg("Cannot find current device")),
    }
}

/// Swap the host in a url for another one, leaving the rest untouched.
///
/// A cast device fetches the audio itself, so a url that names the daemon is
/// no use to it — the server it came from is what it has to be told about.
/// A url that will not parse is handed back unchanged rather than dropped.
pub fn replace_host(url: &str, host: &str) -> String {
    match url::Url::parse(url) {
        Ok(parsed) => match parsed.host_str() {
            Some(original) => url.replace(original, host),
            None => url.to_string(),
        },
        Err(_) => url.to_string(),
    }
}
