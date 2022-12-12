use async_graphql::*;
use serde::Serialize;

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
