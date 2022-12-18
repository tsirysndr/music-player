use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};

use async_graphql::*;

use futures_util::Stream;

use crate::simple_broker::SimpleBroker;

use music_player_types::types::{self, Connected};

use super::objects::device::{App, Device};

#[derive(Default)]
pub struct DevicesQuery;

#[Object]
impl DevicesQuery {
    async fn connected_device(&self, ctx: &Context<'_>) -> Result<Device, Error> {
        let connected_device = ctx
            .data::<Arc<Mutex<HashMap<String, types::Device>>>>()
            .unwrap();
        let connected_device = connected_device.lock().unwrap().clone();
        match connected_device.get("current_device") {
            Some(device) => Ok(device.clone().into()),
            None => Err(Error::new("No device connected")),
        }
    }
    async fn list_devices(
        &self,
        ctx: &Context<'_>,
        filter: Option<App>,
    ) -> Result<Vec<Device>, Error> {
        let connected_device = ctx
            .data::<Arc<Mutex<HashMap<String, types::Device>>>>()
            .unwrap();
        let devices = ctx.data::<Arc<Mutex<Vec<types::Device>>>>().unwrap();
        let devices = devices.lock().unwrap().clone();
        let connected_device = connected_device.lock().unwrap().clone();
        let current_device = connected_device.get("current_device");

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
            .map(|srv| types::Device::from(srv.clone()).is_connected(current_device))
            .map(Into::into)
            .collect();
        Ok(devices)
    }
}

#[derive(Default)]
pub struct DevicesMutation;

#[Object]
impl DevicesMutation {
    async fn connect_to_device(&self, ctx: &Context<'_>, id: ID) -> Result<Device, Error> {
        let devices = ctx.data::<Arc<Mutex<Vec<types::Device>>>>().unwrap();
        let devices = devices.lock().unwrap().clone();
        let connected_device = ctx
            .data::<Arc<Mutex<HashMap<String, types::Device>>>>()
            .unwrap();
        match devices
            .into_iter()
            .find(|device| device.id == id.to_string())
        {
            Some(device) => {
                connected_device.lock().unwrap().insert(
                    "current_device".to_string(),
                    types::Device::from(device.clone()).is_connected(Some(&device.clone())),
                );
                Ok(types::Device::from(device.clone())
                    .is_connected(Some(&device.clone()))
                    .into())
            }
            None => Err(Error::new("Device not found")),
        }
    }

    async fn disconnect_from_device(&self, ctx: &Context<'_>) -> Result<Option<Device>, Error> {
        let connected_device = ctx
            .data::<Arc<Mutex<HashMap<String, types::Device>>>>()
            .unwrap();
        match connected_device.lock().unwrap().remove("current_device") {
            Some(device) => Ok(Some(device.clone().into())),
            None => Ok(None),
        }
    }
}

#[derive(Default)]
pub struct DevicesSubscription;

#[Subscription]
impl DevicesSubscription {
    async fn on_new_device(&self, ctx: &Context<'_>) -> impl Stream<Item = Device> {
        let connected_device = ctx
            .data::<Arc<Mutex<HashMap<String, types::Device>>>>()
            .unwrap();
        let devices = ctx.data::<Arc<Mutex<Vec<types::Device>>>>().unwrap();
        let devices = devices.lock().unwrap().clone();
        let connected_device = connected_device.lock().unwrap().clone();

        thread::spawn(move || {
            let current_device = connected_device.get("current_device");

            thread::sleep(std::time::Duration::from_secs(1));
            devices.into_iter().for_each(|device| {
                SimpleBroker::<Device>::publish(device.is_connected(current_device).into());
            });
        });
        SimpleBroker::<Device>::subscribe()
    }
}
