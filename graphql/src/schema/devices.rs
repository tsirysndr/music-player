use std::{
    sync::{Arc, Mutex},
    thread,
};

use async_graphql::*;

use futures_util::Stream;
use music_player_renderer::CurrentReceiverDevice;
use music_player_provider::ProviderConfig;
use tokio::sync::Mutex as TokioMutex;

use crate::simple_broker::SimpleBroker;

use super::provider;

use music_player_types::types::{self, Connected};

use super::{
    connect_to_cast_device,
    objects::device::{App, ConnectedDevice, Device, DisconnectedDevice},
    PlayerType,
};

#[derive(Default)]
pub struct DevicesQuery;

#[Object]
impl DevicesQuery {
    /// The device the library is being read from, if it is a discovered one.
    async fn connected_device(&self, ctx: &Context<'_>) -> Result<Device, Error> {
        match provider::state(ctx).config().await {
            Some(config) => Ok(Device {
                is_connected: true,
                ..config.to_device().into()
            }),
            None => Err(Error::new("No device connected")),
        }
    }
    async fn list_devices(
        &self,
        ctx: &Context<'_>,
        filter: Option<App>,
    ) -> Result<Vec<Device>, Error> {
        let connected = provider::state(ctx).config().await.map(|c| c.to_device());
        let devices = ctx.data::<Arc<Mutex<Vec<types::Device>>>>().unwrap();
        let devices = devices.lock().unwrap().clone();

        let devices = match filter {
            Some(App::MusicPlayer) => devices
                .into_iter()
                .filter(|device| device.app == "music-player")
                .collect(),
            Some(App::Subsonic) => devices
                .into_iter()
                .filter(|device| device.app == "subsonic")
                .collect(),
            Some(App::Jellyfin) => devices
                .into_iter()
                .filter(|device| device.app == "jellyfin")
                .collect(),
            None => devices,
        };

        let devices = devices
            .iter()
            .filter(|device| device.is_source_device)
            .map(|srv| types::Device::from(srv.clone()).is_connected(connected.as_ref()))
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
        let current_device = ctx
            .data::<Arc<TokioMutex<CurrentReceiverDevice>>>()
            .unwrap();
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
    /// Read the library from a *discovered* device — an mDNS peer, or a
    /// streaming server named in the settings.
    ///
    /// A saved server goes through `connectToServer` instead. Both end at the
    /// same `ProviderState::connect`, which is what stops the two paths from
    /// disagreeing about what "connected" means.
    async fn connect_to_device(&self, ctx: &Context<'_>, id: ID) -> Result<Device, Error> {
        let devices = ctx.data::<Arc<Mutex<Vec<types::Device>>>>().unwrap();
        let devices = devices.lock().unwrap().clone();

        // A music-player peer advertises gRPC and HTTP as two records; the
        // HTTP one is the address a provider actually reads from.
        let http_port = devices
            .iter()
            .find(|device| device.id == id.to_string() && device.service == "http")
            .map(|device| device.port);

        let device = devices
            .into_iter()
            .find(|device| device.id == id.to_string() && device.is_source_device)
            .ok_or_else(|| Error::new("Device not found"))?;

        let config = ProviderConfig::from_device(&device, http_port)
            .ok_or_else(|| Error::new("that device has no address to read from"))?;
        provider::state(ctx)
            .connect(config)
            .await
            .map_err(provider::err)?;

        SimpleBroker::<ConnectedDevice>::publish(device.clone().into());
        Ok(types::Device::from(device.clone())
            .is_connected(Some(&device))
            .into())
    }

    async fn disconnect_from_device(&self, ctx: &Context<'_>) -> Result<Option<Device>, Error> {
        match provider::state(ctx).disconnect().await {
            Some(config) => {
                let device = config.to_device();
                SimpleBroker::<DisconnectedDevice>::publish(device.clone().into());
                Ok(Some(device.into()))
            }
            None => Ok(None),
        }
    }

    async fn connect_to_cast_device(&self, ctx: &Context<'_>, id: ID) -> Result<Device, Error> {
        let devices = ctx.data::<Arc<Mutex<Vec<types::Device>>>>().unwrap();
        let devices = devices.lock().unwrap().clone();
        let io_device = ctx
            .data::<Arc<TokioMutex<CurrentReceiverDevice>>>()
            .unwrap();
        let mut io_device = io_device.lock().await;

        match devices.into_iter().find(|device| {
            device.id == id.to_string()
                && (device.service == "grpc" || device.app == "dlna" || device.app == "chromecast")
        }) {
            Some(device) => {
                let current_device =
                    types::Device::from(device.clone()).is_connected(Some(&device.clone()));
                io_device.set_receiver_device(current_device.clone());

                let player_type = match device.app.as_str() {
                    "chromecast" => PlayerType::Chromecast,
                    "dlna" => PlayerType::Dlna,
                    _ => PlayerType::MusicPlayer,
                };

                let receiver = connect_to_cast_device(
                    types::Device::from(device.clone()).is_connected(Some(&device.clone())),
                    player_type,
                )
                .await?;

                match receiver {
                    Some(receiver) => io_device.set_client(receiver),
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
        let io_device = ctx
            .data::<Arc<TokioMutex<CurrentReceiverDevice>>>()
            .unwrap();
        let mut io_device = io_device.lock().await;
        if let Some(receiver) = io_device.client.as_mut() {
            receiver.stop().await?;
            receiver.disconnect()?;
        }
        match io_device.clear_client() {
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
        // Marks whichever discovered device is the current provider, so a
        // late-arriving mDNS record does not appear unconnected when it is the
        // one being read from. `CurrentDevice` was read here before and was
        // never written to, so this was always `None`.
        let current_device = provider::state(ctx)
            .config()
            .await
            .map(|config| types::Device {
                id: config.id,
                ..Default::default()
            });

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

    async fn on_connected(&self, _ctx: &Context<'_>) -> impl Stream<Item = ConnectedDevice> {
        SimpleBroker::<ConnectedDevice>::subscribe()
    }

    async fn on_disconnected(&self, _ctx: &Context<'_>) -> impl Stream<Item = DisconnectedDevice> {
        SimpleBroker::<DisconnectedDevice>::subscribe()
    }
}
