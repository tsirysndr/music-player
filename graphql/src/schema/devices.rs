use std::{
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
    connect_to, connect_to_cast_device,
    objects::device::{App, ConnectedDevice, Device, DisconnectedDevice},
    PlayerType,
};

#[derive(Default)]
pub struct DevicesQuery;

#[Object]
impl DevicesQuery {
    async fn connected_device(&self, ctx: &Context<'_>) -> Result<Device, Error> {
        let current_device = ctx.data::<Arc<TokioMutex<CurrentDevice>>>().unwrap();
        let device = current_device.lock().await;
        match &device.source_device {
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
        let current_device = ctx.data::<Arc<TokioMutex<CurrentDevice>>>().unwrap();
        let device = current_device.lock().await;
        let devices = ctx.data::<Arc<Mutex<Vec<types::Device>>>>().unwrap();
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
            .filter(|device| device.is_source_device)
            .map(|srv| types::Device::from(srv.clone()).is_connected(device.source_device.as_ref()))
            .map(Into::into)
            .collect();
        Ok(devices)
    }

    async fn list_cast_devices(&self, ctx: &Context<'_>) -> Result<Vec<Device>, Error> {
        let devices = ctx.data::<Arc<Mutex<Vec<types::Device>>>>().unwrap();
        let devices = devices.lock().unwrap().clone();
        Ok(devices
            .into_iter()
            .filter(|device| device.is_cast_device)
            .map(Into::into)
            .collect())
    }

    async fn connected_cast_device(&self, ctx: &Context<'_>) -> Result<Device, Error> {
        let current_device = ctx.data::<Arc<TokioMutex<CurrentDevice>>>().unwrap();
        let device = current_device.lock().await;
        match &device.receiver_device {
            Some(device) => Ok(Device {
                is_connected: true,
                ..device.clone().into()
            }),
            None => Err(Error::new("No device connected")),
        }
    }
}

#[derive(Default)]
pub struct DevicesMutation;

#[Object]
impl DevicesMutation {
    async fn connect_to_device(&self, ctx: &Context<'_>, id: ID) -> Result<Device, Error> {
        let devices = ctx.data::<Arc<Mutex<Vec<types::Device>>>>().unwrap();
        let devices = devices.lock().unwrap().clone();
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
                io_device.set_source_device(current_device.clone());

                let source = connect_to(
                    types::Device::from(device.clone()).is_connected(Some(&device.clone())),
                )
                .await?;

                match source {
                    Some(source) => io_device.set_source(source),
                    None => return Err(Error::new("No source found")),
                }

                SimpleBroker::<ConnectedDevice>::publish(device.clone().into());

                Ok(types::Device::from(device.clone())
                    .is_connected(Some(&device.clone()))
                    .into())
            }
            None => Err(Error::new("Device not found")),
        }
    }

    async fn disconnect_from_device(&self, ctx: &Context<'_>) -> Result<Option<Device>, Error> {
        let io_device = ctx.data::<Arc<TokioMutex<CurrentDevice>>>().unwrap();
        let mut io_device = io_device.lock().await;
        match io_device.clear_source() {
            Some(device) => {
                SimpleBroker::<DisconnectedDevice>::publish(device.clone().into());
                Ok(Some(device.clone().into()))
            }
            None => Ok(None),
        }
    }

    async fn connect_to_cast_device(&self, ctx: &Context<'_>, id: ID) -> Result<Device, Error> {
        let devices = ctx.data::<Arc<Mutex<Vec<types::Device>>>>().unwrap();
        let devices = devices.lock().unwrap().clone();
        let io_device = ctx.data::<Arc<TokioMutex<CurrentDevice>>>().unwrap();
        let mut io_device = io_device.lock().await;

        match devices.into_iter().find(|device| {
            device.id == id.to_string()
                && (device.service == "grpc"
                    || device.app == "xbmc"
                    || device.app == "chromecast"
                    || device.app == "airplay")
        }) {
            Some(device) => {
                let current_device =
                    types::Device::from(device.clone()).is_connected(Some(&device.clone()));
                io_device.set_receiver_device(current_device.clone());

                let player_type = match device.app.as_str() {
                    "chromecast" => PlayerType::Chromecast,
                    "airplay" => PlayerType::Airplay,
                    "xbmc" => PlayerType::Kodi,
                    _ => PlayerType::MusicPlayer,
                };

                let receiver = connect_to_cast_device(
                    types::Device::from(device.clone()).is_connected(Some(&device.clone())),
                    player_type,
                )
                .await?;

                match receiver {
                    Some(receiver) => io_device.set_receiver(receiver),
                    None => return Err(Error::new("No source found")),
                }

                SimpleBroker::<ConnectedDevice>::publish(device.clone().into());

                Ok(types::Device::from(device.clone())
                    .is_connected(Some(&device.clone()))
                    .into())
            }
            None => Err(Error::new("Device not found")),
        }
    }

    async fn disconnect_from_cast_device(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<Device>, Error> {
        let io_device = ctx.data::<Arc<TokioMutex<CurrentDevice>>>().unwrap();
        let mut io_device = io_device.lock().await;
        if let Some(receiver) = io_device.receiver.as_mut() {
            receiver.disconnect()?;
        }
        match io_device.clear_receiver() {
            Some(device) => {
                SimpleBroker::<DisconnectedDevice>::publish(device.clone().into());
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
        let devices = ctx.data::<Arc<Mutex<Vec<types::Device>>>>().unwrap();
        let devices = devices.lock().unwrap().clone();
        let current_device = ctx.data::<Arc<TokioMutex<CurrentDevice>>>().unwrap();
        let current_device = current_device.lock().await;

        let current_device = match &current_device.source_device {
            Some(device) => Some(types::Device {
                id: device.id.clone(),
                ..Default::default()
            }),
            None => None,
        };

        thread::spawn(move || {
            thread::sleep(std::time::Duration::from_secs(1));
            devices.into_iter().for_each(|device| {
                SimpleBroker::<Device>::publish(
                    device.is_connected(current_device.as_ref()).into(),
                );
            });
        });
        SimpleBroker::<Device>::subscribe()
    }

    async fn on_connected(&self, ctx: &Context<'_>) -> impl Stream<Item = ConnectedDevice> {
        SimpleBroker::<ConnectedDevice>::subscribe()
    }

    async fn on_disconnected(&self, ctx: &Context<'_>) -> impl Stream<Item = DisconnectedDevice> {
        SimpleBroker::<DisconnectedDevice>::subscribe()
    }
}
