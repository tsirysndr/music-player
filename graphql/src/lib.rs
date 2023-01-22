#[cfg(test)]
mod tests;
use crate::simple_broker::SimpleBroker;
use anyhow::Error;
use async_graphql::{Context, Schema};
use futures_util::StreamExt;
use music_player_addons::Browseable;
use music_player_addons::{CurrentDevice, Player};
use music_player_discovery::{discover, SERVICE_NAME, XBMC_SERVICE_NAME};
use music_player_entity::album as album_entity;
use music_player_entity::track as track_entity;
use music_player_playback::player::PlayerCommand;
use music_player_types::types::RemoteCoverUrl;
use music_player_types::types::RemoteTrackUrl;
use music_player_types::types::{Device, Track, AIRPLAY_SERVICE_NAME, CHROMECAST_SERVICE_NAME};
use rand::seq::SliceRandom;
use schema::{Mutation, Query, Subscription};
use std::{
    sync::{Arc, Mutex},
    thread,
};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex as TokioMutex;
use url::Url;

pub mod schema;
pub mod simple_broker;

pub type MusicPlayerSchema = Schema<Query, Mutation, Subscription>;

fn scan_mp_devices(mp_devices: Arc<Mutex<Vec<Device>>>) {
    thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let services = discover(SERVICE_NAME);
            tokio::pin!(services);
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            while let Some(info) = services.next().await {
                let device = Device::from(info.clone());
                let mut mp_devices = mp_devices.lock().unwrap();
                if mp_devices
                    .iter()
                    .find(|d| d.id == device.id && d.service == device.service)
                    .is_none()
                {
                    mp_devices.push(device.clone());
                    SimpleBroker::<Device>::publish(device.clone());
                }
            }
        });
    });
}

fn scan_xbmc_devices(xbmc_devices: Arc<Mutex<Vec<Device>>>) {
    thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let services = discover(XBMC_SERVICE_NAME);
            tokio::pin!(services);
            while let Some(info) = services.next().await {
                xbmc_devices
                    .lock()
                    .unwrap()
                    .push(Device::from(info.clone()));
                SimpleBroker::<Device>::publish(Device::from(info.clone()));
            }
        });
    });
}

fn scan_chromecast_devices(devices: Arc<Mutex<Vec<Device>>>) {
    thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let services = discover(CHROMECAST_SERVICE_NAME);
            tokio::pin!(services);
            while let Some(info) = services.next().await {
                devices.lock().unwrap().push(Device::from(info.clone()));
                SimpleBroker::<Device>::publish(Device::from(info.clone()));
            }
        });
    });
}

fn scan_airplay_devices(devices: Arc<Mutex<Vec<Device>>>) {
    thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let services = discover(AIRPLAY_SERVICE_NAME);
            tokio::pin!(services);
            while let Some(info) = services.next().await {
                devices.lock().unwrap().push(Device::from(info.clone()));
                SimpleBroker::<Device>::publish(Device::from(info.clone()));
            }
        });
    });
}

pub async fn scan_devices() -> Result<Arc<std::sync::Mutex<Vec<Device>>>, Box<dyn std::error::Error>>
{
    let devices: Arc<std::sync::Mutex<Vec<Device>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
    let mp_devices = Arc::clone(&devices);
    let xbmc_devices = Arc::clone(&devices);
    let chromecast_devices = Arc::clone(&devices);
    let airplay_devices = Arc::clone(&devices);

    scan_mp_devices(mp_devices);

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    scan_xbmc_devices(xbmc_devices);

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    scan_chromecast_devices(chromecast_devices);

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    scan_airplay_devices(airplay_devices);

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
                        track
                    })
                    .collect();
            }
        }
        player
            .load_tracks(tracks.clone().into_iter().map(Into::into).collect())
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

pub fn update_track_url<T: RemoteTrackUrl>(devices: Vec<Device>, result: T) -> Result<T, Error> {
    let base_url = match devices
        .clone()
        .into_iter()
        .find(|device| device.is_current_device)
    {
        Some(device) => Some(format!("http://{}:{}", device.host, device.port)),
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
