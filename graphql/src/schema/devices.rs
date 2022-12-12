use std::{
    sync::{Arc, Mutex},
    thread,
};

use async_graphql::*;

use futures_util::{Stream, StreamExt};
use mdns_sd::ServiceInfo;
use music_player_discovery::{discover, SERVICE_NAME, XBMC_SERVICE_NAME};

use crate::simple_broker::SimpleBroker;

use super::objects::device::{App, Device};

#[derive(Default)]
pub struct DevicesQuery;

#[Object]
impl DevicesQuery {
    async fn list_devices(
        &self,
        ctx: &Context<'_>,
        filter: Option<App>,
    ) -> Result<Vec<Device>, Error> {
        let devices = ctx.data::<Arc<Mutex<Vec<Device>>>>().unwrap();
        let devices = devices.lock().unwrap().clone();

        let devices = match filter {
            Some(App::MusicPlayer) => devices
                .into_iter()
                .filter(|device| device.app == "music-player")
                .collect(),
            Some(App::XBMC) => devices
                .into_iter()
                .filter(|device| device.app == "xbmc")
                .collect(),
            None => devices,
        };

        let devices = devices
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

#[derive(Default)]
pub struct DevicesSubscription;

#[Subscription]
impl DevicesSubscription {
    async fn on_new_device(&self, ctx: &Context<'_>) -> impl Stream<Item = Device> {
        let devices = ctx.data::<Arc<Mutex<Vec<Device>>>>().unwrap();
        let devices = devices.lock().unwrap().clone();
        thread::spawn(move || {
            thread::sleep(std::time::Duration::from_secs(1));
            devices.into_iter().for_each(|device| {
                SimpleBroker::<Device>::publish(device);
            });
        });
        SimpleBroker::<Device>::subscribe()
    }
}
