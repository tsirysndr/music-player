use std::{
    sync::{Arc, Mutex},
    thread,
};

use async_graphql::*;

use futures_util::StreamExt;
use mdns_sd::ServiceInfo;
use music_player_discovery::{discover, SERVICE_NAME};

use super::objects::device::Device;

#[derive(Default)]
pub struct DevicesQuery;

#[Object]
impl DevicesQuery {
    async fn list_devices(&self, ctx: &Context<'_>) -> Result<Vec<Device>, Error> {
        let discovered: Mutex<Vec<ServiceInfo>> = Mutex::new(vec![]);
        let discovered: Arc<Mutex<Vec<ServiceInfo>>> = Arc::new(discovered);
        let cloned_discovered = Arc::clone(&discovered);
        thread::spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let services = discover(SERVICE_NAME);
                    tokio::pin!(services);
                    while let Some(info) = services.next().await {
                        discovered.lock().unwrap().push(info);
                    }
                });
        });

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let devices = cloned_discovered
            .lock()
            .unwrap()
            .iter()
            .map(|srv| Device::from(srv.clone()))
            .collect();
        Ok(devices)
    }
}

#[derive(Default)]
pub struct DevicesMutation;

#[Object]
impl DevicesMutation {
    async fn connect_to_device(&self, ctx: &Context<'_>, id: ID) -> Result<Device, Error> {
        todo!()
    }

    async fn disconnect_from_device(&self, ctx: &Context<'_>, id: ID) -> Result<Device, Error> {
        todo!()
    }
}
