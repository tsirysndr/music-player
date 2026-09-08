//! Saved servers over gRPC, so the TUI and the Slint desktop reach the same
//! provider state the GraphQL layer does.
//!
//! Connecting is deliberately dull: it makes a server the current provider and
//! nothing else. It cannot interrupt playback — a provider is where the
//! *screens* read from, and nothing reachable from here touches the player.

use music_player_provider::{ProviderConfig, ProviderState};
use music_player_storage::{
    saved_servers::{self, NewServer, SavedServer},
    Database,
};
use std::sync::Arc;

use crate::api::music::v1alpha1::{
    servers_service_server::ServersService, AddServerRequest, AddServerResponse,
    ConnectServerRequest, ConnectServerResponse, DeleteServerRequest, DeleteServerResponse,
    DisconnectServerRequest, DisconnectServerResponse, GetConnectedServerRequest,
    GetConnectedServerResponse, ListServersRequest, ListServersResponse, ListSourceKindsRequest,
    ListSourceKindsResponse, Server, SourceKind,
};

pub struct Servers {
    db: Database,
    providers: Arc<ProviderState>,
}

impl Servers {
    pub fn new(db: Database, providers: Arc<ProviderState>) -> Self {
        Self { db, providers }
    }

    async fn connected_id(&self) -> Option<String> {
        self.providers.config().await.map(|config| config.id)
    }
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn to_proto(row: SavedServer, connected_id: Option<&str>) -> Server {
    Server {
        connected: connected_id == Some(row.id.as_str()),
        id: row.id,
        kind: row.kind,
        name: row.name,
        url: row.url,
        username: row.username.unwrap_or_default(),
        has_password: row.password.is_some_and(|password| !password.is_empty()),
    }
}

fn internal(e: impl std::fmt::Display) -> tonic::Status {
    tonic::Status::internal(e.to_string())
}

#[tonic::async_trait]
impl ServersService for Servers {
    async fn list_servers(
        &self,
        _request: tonic::Request<ListServersRequest>,
    ) -> Result<tonic::Response<ListServersResponse>, tonic::Status> {
        let connected = self.connected_id().await;
        let rows = saved_servers::list(self.db.get_connection())
            .await
            .map_err(internal)?;
        Ok(tonic::Response::new(ListServersResponse {
            servers: rows
                .into_iter()
                .map(|row| to_proto(row, connected.as_deref()))
                .collect(),
        }))
    }

    async fn list_source_kinds(
        &self,
        _request: tonic::Request<ListSourceKindsRequest>,
    ) -> Result<tonic::Response<ListSourceKindsResponse>, tonic::Status> {
        Ok(tonic::Response::new(ListSourceKindsResponse {
            kinds: self
                .providers
                .registry()
                .describe()
                .into_iter()
                .map(|info| SourceKind {
                    kind: info.kind.to_string(),
                    display_name: info.display_name.to_string(),
                    needs_credentials: info.needs_credentials,
                    default_port: info.default_port as u32,
                    fixed_url: info.fixed_url.map(str::to_owned),
                })
                .collect(),
        }))
    }

    async fn get_connected_server(
        &self,
        _request: tonic::Request<GetConnectedServerRequest>,
    ) -> Result<tonic::Response<GetConnectedServerResponse>, tonic::Status> {
        let Some(config) = self.providers.config().await else {
            return Ok(tonic::Response::new(GetConnectedServerResponse {
                server: None,
            }));
        };
        // A discovered peer has no saved row, so fall back to the live config.
        let server = match saved_servers::get(self.db.get_connection(), &config.id).await {
            Ok(Some(row)) => to_proto(row, Some(config.id.as_str())),
            _ => Server {
                id: config.id.clone(),
                kind: config.kind.clone(),
                name: config.name.clone(),
                url: config.url.clone(),
                username: config.username.clone().unwrap_or_default(),
                has_password: config.password.is_some(),
                connected: true,
            },
        };
        Ok(tonic::Response::new(GetConnectedServerResponse {
            server: Some(server),
        }))
    }

    async fn add_server(
        &self,
        request: tonic::Request<AddServerRequest>,
    ) -> Result<tonic::Response<AddServerResponse>, tonic::Status> {
        let request = request.into_inner();
        let factory = self
            .providers
            .registry()
            .get(&request.kind)
            .ok_or_else(|| {
                tonic::Status::invalid_argument(format!(
                    "unknown kind of server: {}",
                    request.kind
                ))
            })?;

        // A hosted backend has one address, and it is the factory's.
        let url = match factory.fixed_url() {
            Some(fixed) => fixed.to_string(),
            None if request.url.trim().is_empty() => {
                return Err(tonic::Status::invalid_argument("a server needs a url"))
            }
            None => request.url.clone(),
        };

        let server = NewServer::new(request.kind, request.name, url)
            .with_credentials(Some(request.username), Some(request.password));
        let row = saved_servers::upsert(self.db.get_connection(), &server, &now())
            .await
            .map_err(internal)?;
        let connected = self.connected_id().await;
        Ok(tonic::Response::new(AddServerResponse {
            server: Some(to_proto(row, connected.as_deref())),
        }))
    }

    /// Disconnects first if it is the one in use, so the screens fall back to
    /// the local library rather than reading from something unlisted.
    async fn delete_server(
        &self,
        request: tonic::Request<DeleteServerRequest>,
    ) -> Result<tonic::Response<DeleteServerResponse>, tonic::Status> {
        let id = request.into_inner().id;
        if self.connected_id().await.as_deref() == Some(id.as_str()) {
            self.providers.disconnect().await;
        }
        let deleted = saved_servers::delete(self.db.get_connection(), &id)
            .await
            .map_err(internal)?;
        Ok(tonic::Response::new(DeleteServerResponse { deleted }))
    }

    async fn connect_server(
        &self,
        request: tonic::Request<ConnectServerRequest>,
    ) -> Result<tonic::Response<ConnectServerResponse>, tonic::Status> {
        let id = request.into_inner().id;
        let row = saved_servers::get(self.db.get_connection(), &id)
            .await
            .map_err(internal)?
            .ok_or_else(|| tonic::Status::not_found("no such server"))?;

        let config = ProviderConfig {
            id: row.id.clone(),
            kind: row.kind.clone(),
            name: row.name.clone(),
            url: row.url.clone(),
            username: row.username.clone(),
            password: row.password.clone(),
        };
        self.providers
            .connect(config)
            .await
            .map_err(crate::library::provider_status)?;

        Ok(tonic::Response::new(ConnectServerResponse {
            server: Some(to_proto(row, Some(id.as_str()))),
        }))
    }

    async fn disconnect_server(
        &self,
        _request: tonic::Request<DisconnectServerRequest>,
    ) -> Result<tonic::Response<DisconnectServerResponse>, tonic::Status> {
        let server = self.providers.disconnect().await.map(|config| Server {
            id: config.id,
            kind: config.kind,
            name: config.name,
            url: config.url,
            username: config.username.unwrap_or_default(),
            has_password: config.password.is_some(),
            connected: false,
        });
        Ok(tonic::Response::new(DisconnectServerResponse { server }))
    }
}
