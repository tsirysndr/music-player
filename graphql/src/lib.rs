#[cfg(test)]
mod tests;
use crate::simple_broker::SimpleBroker;
use async_graphql::Schema;
use futures_util::StreamExt;
use music_player_discovery::{discover, SERVICE_NAME, XBMC_SERVICE_NAME};
use music_player_entity::track as track_entity;
use music_player_playback::player::PlayerCommand;
use music_player_types::types::Device;
use rand::seq::SliceRandom;
use schema::{Mutation, Query, Subscription};
use std::{
    sync::{Arc, Mutex},
    thread,
};
use tokio::sync::mpsc::UnboundedSender;

pub mod schema;
pub mod simple_broker;

pub type MusicPlayerSchema = Schema<Query, Mutation, Subscription>;

pub async fn scan_devices() -> Result<Arc<std::sync::Mutex<Vec<Device>>>, Box<dyn std::error::Error>>
{
    let devices: Arc<std::sync::Mutex<Vec<Device>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
    let mp_devices = Arc::clone(&devices);
    let xbmc_devices = Arc::clone(&devices);
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

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

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

    Ok(devices)
}

pub fn load_tracks(
    player_cmd: &Arc<Mutex<UnboundedSender<PlayerCommand>>>,
    mut tracks: Vec<track_entity::Model>,
    position: Option<u32>,
    shuffle: bool,
) {
    if shuffle {
        tracks.shuffle(&mut rand::thread_rng());
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
}
