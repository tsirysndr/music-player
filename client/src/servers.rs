//! Saved servers, for clients that speak gRPC.
//!
//! A *server* here is where the library is read from. Connecting to one
//! repoints every library screen at it and leaves playback alone.

use anyhow::Error;
use music_player_server::api::music::v1alpha1::{
    servers_service_client::ServersServiceClient, AddServerRequest, ConnectServerRequest,
    DeleteServerRequest, DisconnectServerRequest, GetConnectedServerRequest, ListServersRequest,
    ListSourceKindsRequest, Server, SourceKind,
};
use tonic::transport::Channel;

pub struct ServersClient {
    client: ServersServiceClient<Channel>,
}

impl ServersClient {
    pub async fn new(host: String, port: u16) -> Result<Self, Error> {
        let client = ServersServiceClient::connect(format!("http://{}:{}", host, port)).await?;
        Ok(Self { client })
    }

    pub async fn list(&mut self) -> Result<Vec<Server>, Error> {
        let response = self
            .client
            .list_servers(tonic::Request::new(ListServersRequest {}))
            .await?;
        Ok(response.into_inner().servers)
    }

    /// The kinds this daemon can talk to, so a form need not hardcode them.
    pub async fn source_kinds(&mut self) -> Result<Vec<SourceKind>, Error> {
        let response = self
            .client
            .list_source_kinds(tonic::Request::new(ListSourceKindsRequest {}))
            .await?;
        Ok(response.into_inner().kinds)
    }

    /// `None` means the daemon's own library.
    pub async fn connected(&mut self) -> Result<Option<Server>, Error> {
        let response = self
            .client
            .get_connected_server(tonic::Request::new(GetConnectedServerRequest {}))
            .await?;
        Ok(response.into_inner().server)
    }

    pub async fn add(
        &mut self,
        kind: &str,
        name: &str,
        url: &str,
        username: &str,
        password: &str,
    ) -> Result<Server, Error> {
        let response = self
            .client
            .add_server(tonic::Request::new(AddServerRequest {
                kind: kind.to_owned(),
                name: name.to_owned(),
                url: url.to_owned(),
                username: username.to_owned(),
                password: password.to_owned(),
            }))
            .await?;
        response
            .into_inner()
            .server
            .ok_or_else(|| Error::msg("the daemon saved no server"))
    }

    pub async fn delete(&mut self, id: &str) -> Result<bool, Error> {
        let response = self
            .client
            .delete_server(tonic::Request::new(DeleteServerRequest {
                id: id.to_owned(),
            }))
            .await?;
        Ok(response.into_inner().deleted)
    }

    pub async fn connect(&mut self, id: &str) -> Result<Server, Error> {
        let response = self
            .client
            .connect_server(tonic::Request::new(ConnectServerRequest {
                id: id.to_owned(),
            }))
            .await?;
        response
            .into_inner()
            .server
            .ok_or_else(|| Error::msg("the daemon connected to nothing"))
    }

    /// Back to the daemon's own library. Returns what was disconnected.
    pub async fn disconnect(&mut self) -> Result<Option<Server>, Error> {
        let response = self
            .client
            .disconnect_server(tonic::Request::new(DisconnectServerRequest {}))
            .await?;
        Ok(response.into_inner().server)
    }
}
