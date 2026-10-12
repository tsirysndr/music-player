// The schema's combined resolver future sits right at rustc's default type
// recursion limit; one more captured future in a mutation tips it over.
#![recursion_limit = "256"]

#[cfg(test)]
mod tests;
use crate::simple_broker::SimpleBroker;
use anyhow::Error;
use async_graphql::Schema;
use futures_util::StreamExt;
use music_player_discovery::{discover, SERVICE_NAME};
use music_player_entity::track as track_entity;
use music_player_playback::{
    player::{normalize_queue, PlayerCommand},
    source_resolver::SourceResolver,
};
use music_player_renderer::Player;
use music_player_settings::{read_settings, Settings};
use music_player_storage::Database;
use music_player_types::source::SourceHandle;
use music_player_types::types::RemoteCoverUrl;
use music_player_types::types::RemoteTrackUrl;
use music_player_types::types::{Device, CHROMECAST_SERVICE_NAME};
use rand::seq::SliceRandom;
use schema::{Mutation, Query, Subscription};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    thread,
};
use tokio::sync::mpsc::UnboundedSender;
use upnp_client::discovery::discover_pnp_locations;
use url::Url;

pub mod schema;
pub mod simple_broker;

pub type MusicPlayerSchema = Schema<Query, Mutation, Subscription>;

/// Schema builders must inject the daemon's shared SourceResolver with .data().
/// Missing configuration is an error; never create an account manager per request.
pub(crate) fn source_resolver<'a>(
    ctx: &'a async_graphql::Context<'_>,
) -> async_graphql::Result<&'a SourceResolver> {
    ctx.data::<SourceResolver>()
        .map_err(|_| async_graphql::Error::new("Shared source resolver is not configured"))
}

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

/// Queue metadata may contain handles from several saved accounts. Validate
/// all syntax first, then check each account/identity once without networking.
pub(crate) async fn validate_queue(
    resolver: &SourceResolver,
    tracks: &mut [track_entity::Model],
) -> Result<(), Error> {
    normalize_queue(tracks)?;
    let mut checked = HashSet::new();
    for track in tracks {
        if SourceHandle::is_handle(&track.id) {
            let source = SourceHandle::parse(&track.id)?;
            let identity = match &source {
                SourceHandle::Remote(source) => (Some(source.remote.clone()), None),
                SourceHandle::Local(source) => (None, Some(source.config_revision)),
            };
            if checked.insert((
                source.resolver().to_owned(),
                source.account_id().to_owned(),
                identity,
            )) {
                resolver.validate_handle(&source).await?;
            }
        }
    }
    Ok(())
}

pub(crate) fn reject_source_receiver(
    tracks: &[track_entity::Model],
    remote: bool,
) -> Result<(), Error> {
    if remote
        && tracks
            .iter()
            .any(|track| SourceHandle::is_handle(&track.id) || SourceHandle::is_handle(&track.uri))
    {
        return Err(Error::msg("Saved sources are unsupported on remote receivers: URL-only playback and remote account mapping are unavailable"));
    }
    Ok(())
}

pub async fn load_tracks(
    db: &Database,
    player_cmd: &Arc<Mutex<UnboundedSender<PlayerCommand>>>,
    player: Option<&mut Box<dyn Player + Send>>,
    source_ip: Option<String>,
    tracks: Vec<track_entity::Model>,
    position: Option<u32>,
    shuffle: bool,
) -> Result<(), Error> {
    // Compatibility for external native callers. Schema operations must use
    // load_tracks_with_resolver with their injected daemon resolver.
    let resolver = SourceResolver::from_settings(db.clone());
    load_tracks_with_resolver(
        &resolver, player_cmd, player, source_ip, tracks, position, shuffle,
    )
    .await
}

pub async fn load_tracks_with_resolver(
    resolver: &SourceResolver,
    player_cmd: &Arc<Mutex<UnboundedSender<PlayerCommand>>>,
    player: Option<&mut Box<dyn Player + Send>>,
    source_ip: Option<String>,
    mut tracks: Vec<track_entity::Model>,
    position: Option<u32>,
    shuffle: bool,
) -> Result<(), Error> {
    normalize_queue(&mut tracks)?;
    reject_source_receiver(&tracks, player.is_some())?;
    validate_queue(resolver, &mut tracks).await?;
    let start_index = position.unwrap_or(0) as usize;
    if !tracks.is_empty() && start_index >= tracks.len() {
        return Err(Error::msg("Invalid start index"));
    }
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
                Some(i32::try_from(start_index).map_err(|_| Error::msg("Invalid start index"))?),
            )
            .await?;
        return Ok(());
    }
    let player_cmd_tx = player_cmd.lock().unwrap();
    player_cmd_tx
        .send(PlayerCommand::Stop)
        .map_err(|_| Error::msg("Player command channel closed"))?;
    player_cmd_tx
        .send(PlayerCommand::Clear)
        .map_err(|_| Error::msg("Player command channel closed"))?;
    // One command: loading the tracklist and *then* asking for an index opened
    // the first track's stream and immediately replaced it, which against a
    // remote server is a wasted connection and a real delay before anything
    // is heard.
    player_cmd_tx
        .send(PlayerCommand::LoadTracklist {
            tracks,
            start_index: Some(start_index),
        })
        .map_err(|_| Error::msg("Player command channel closed"))?;
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
