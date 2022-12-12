use async_graphql::*;
use mdns_sd::ServiceInfo;
use music_player_discovery::{SERVICE_NAME, XBMC_SERVICE_NAME};
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

impl From<ServiceInfo> for Device {
    fn from(srv: ServiceInfo) -> Self {
        if srv.get_fullname().contains("xbmc") {
            return Self {
                id: ID::from(srv.get_fullname().to_owned()),
                name: srv
                    .get_fullname()
                    .replace(XBMC_SERVICE_NAME, "")
                    .replace(".", "")
                    .to_owned(),
                host: srv
                    .get_hostname()
                    .split_at(srv.get_hostname().len() - 1)
                    .0
                    .to_owned(),
                port: srv.get_port(),
                service: srv.get_fullname().to_owned(),
                app: "xbmc".to_owned(),
                is_connected: false,
            };
        }

        if srv.get_fullname().contains(SERVICE_NAME) {
            return Self {
                id: ID::from(
                    srv.get_fullname()
                        .replace(SERVICE_NAME, "")
                        .split("-")
                        .collect::<Vec<&str>>()[1]
                        .replace(".", "")
                        .to_owned(),
                ),
                name: srv
                    .get_fullname()
                    .replace(SERVICE_NAME, "")
                    .replace(".", "")
                    .to_owned(),
                host: srv
                    .get_hostname()
                    .split_at(srv.get_hostname().len() - 1)
                    .0
                    .to_owned(),
                port: srv.get_port(),
                service: srv.get_fullname().split("-").collect::<Vec<&str>>()[0].to_owned(),
                app: "music-player".to_owned(),
                is_connected: false,
            };
        }

        Self {
            ..Default::default()
        }
    }
}
