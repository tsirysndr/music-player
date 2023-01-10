use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};

use async_graphql::*;

use futures_util::Stream;
use music_player_addons::CurrentDevice;
use tokio::sync::Mutex as TokioMutex;

use crate::simple_broker::SimpleBroker;

use music_player_types::types::{self, Connected};

use super::{
    connect_to, connect_to_current_device,
    objects::device::{App, Device},
};

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
            Some(device) => Ok(Device {
                is_connected: true,
                ..device.clone().into()
            }),
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
        let io_device = ctx.data::<Arc<TokioMutex<CurrentDevice>>>().unwrap();
        let mut io_device = io_device.lock().await;

        let base_url = match devices.clone().into_iter().find(|device| {
            device.id == id.to_string() && (device.service == "http" || device.app == "xbmc")
        }) {
            Some(device) => Some(format!("http://{}:{}", device.host, device.port)),
            None => None,
        };

        match devices.into_iter().find(|device| {
            device.id == id.to_string() && (device.service == "grpc" || device.app == "xbmc")
        }) {
            Some(device) => {
                let current_device = types::Device::from(device.clone())
                    .is_connected(Some(&device.clone()))
                    .with_base_url(base_url);
                connected_device
                    .lock()
                    .unwrap()
                    .insert("current_device".to_string(), current_device.clone());

                let source = connect_to(
                    types::Device::from(device.clone()).is_connected(Some(&device.clone())),
                )
                .await?;

                match source {
                    Some(source) => io_device.set_source(source),
                    None => return Err(Error::new("No source found")),
                }

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
        let io_device = ctx.data::<Arc<TokioMutex<CurrentDevice>>>().unwrap();
        let mut io_device = io_device.lock().await;
        let mut connected_device = connected_device.lock().unwrap();
        match connected_device.remove("current_device") {
            Some(device) => {
                io_device.clear_source();
                Ok(Some(device.clone().into()))
            }
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
