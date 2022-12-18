#[cfg(test)]
mod tests;
use async_graphql::Schema;
use futures_util::StreamExt;
use music_player_discovery::{discover, SERVICE_NAME, XBMC_SERVICE_NAME};
use music_player_types::types::Device;
use schema::{Mutation, Query, Subscription};
use std::{sync::Arc, thread};

use crate::simple_broker::SimpleBroker;

pub mod schema;
pub mod simple_broker;

pub type MusicPlayerSchema = Schema<Query, Mutation, Subscription>;

pub async fn scan_devices() -> Result<Arc<std::sync::Mutex<Vec<Device>>>, Box<dyn std::error::Error>>
{
    let devices = Arc::new(std::sync::Mutex::new(Vec::new()));
    let mp_devices = Arc::clone(&devices);
    let xbmc_devices = Arc::clone(&devices);
    thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let services = discover(SERVICE_NAME);
            tokio::pin!(services);
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            while let Some(info) = services.next().await {
                mp_devices.lock().unwrap().push(Device::from(info.clone()));
                SimpleBroker::<Device>::publish(Device::from(info.clone()));
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
