use async_graphql::*;
use music_player_discovery::{SERVICE_NAME, XBMC_SERVICE_NAME};
use music_player_types::types;
use serde::Serialize;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum App {
    MusicPlayer,
    XBMC,
}

#[derive(Default, Clone, Serialize)]
pub struct Device {
    pub id: ID,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub service: String,
    pub app: String,
    pub is_connected: bool,
}

#[Object]
impl Device {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn host(&self) -> &str {
        &self.host
    }

    async fn port(&self) -> u16 {
        self.port
    }

    async fn service(&self) -> &str {
        &self.service
    }

    async fn app(&self) -> &str {
        &self.app
    }

    async fn is_connected(&self) -> bool {
        self.is_connected
    }
}

impl From<types::Device> for Device {
    fn from(device: types::Device) -> Self {
        Self {
            id: ID::from(device.id),
            name: device.name,
            host: device.host,
            port: device.port,
            service: device.service,
            app: device.app,
            is_connected: false,
        }
    }
}

#[derive(Default, Clone, Serialize)]
pub struct ConnectedDevice {
    pub id: ID,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub service: String,
    pub app: String,
    pub is_connected: bool,
}

#[Object]
impl ConnectedDevice {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn host(&self) -> &str {
        &self.host
    }

    async fn port(&self) -> u16 {
        self.port
    }

    async fn service(&self) -> &str {
        &self.service
    }

    async fn app(&self) -> &str {
        &self.app
    }

    async fn is_connected(&self) -> bool {
        self.is_connected
    }
}

impl From<types::Device> for ConnectedDevice {
    fn from(device: types::Device) -> Self {
        Self {
            id: ID::from(device.id),
            name: device.name,
            host: device.host,
            port: device.port,
            service: device.service,
            app: device.app,
            is_connected: true,
        }
    }
}

#[derive(Default, Clone, Serialize)]
pub struct DisconnectedDevice {
    pub id: ID,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub service: String,
    pub app: String,
    pub is_connected: bool,
}

#[Object]
impl DisconnectedDevice {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn host(&self) -> &str {
        &self.host
    }

    async fn port(&self) -> u16 {
        self.port
    }

    async fn service(&self) -> &str {
        &self.service
    }

    async fn app(&self) -> &str {
        &self.app
    }

    async fn is_connected(&self) -> bool {
        self.is_connected
    }
}

impl From<types::Device> for DisconnectedDevice {
    fn from(device: types::Device) -> Self {
        Self {
            id: ID::from(device.id),
            name: device.name,
            host: device.host,
            port: device.port,
            service: device.service,
            app: device.app,
            is_connected: false,
        }
    }
}
