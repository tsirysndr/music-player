//! Catalog and runtime invalidation on the selected daemon. No mutation retry.
use anyhow::Error;
use music_player_server::api::music::v1alpha1::{
    extensions_service_client::ExtensionsServiceClient, InstalledExtension, ListExtensionsRequest,
    RescanExtensionsRequest, SetExtensionEnabledRequest,
};
use std::time::Duration;
use tonic::transport::Channel;

pub struct ExtensionsClient {
    client: ExtensionsServiceClient<Channel>,
}
impl ExtensionsClient {
    pub async fn new(host: String, port: u16) -> Result<Self, Error> {
        let channel = Channel::from_shared(format!("http://{host}:{port}"))?
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(30))
            .connect()
            .await?;
        Ok(Self::from_channel(channel))
    }
    pub fn from_channel(channel: Channel) -> Self {
        Self {
            client: ExtensionsServiceClient::new(channel)
                .max_decoding_message_size(music_player_server::LIBRARY_MESSAGE_LIMIT)
                .max_encoding_message_size(music_player_server::LIBRARY_MESSAGE_LIMIT),
        }
    }
    pub async fn list(&mut self) -> Result<Vec<InstalledExtension>, Error> {
        Ok(self
            .client
            .list_extensions(ListExtensionsRequest {})
            .await?
            .into_inner()
            .extensions)
    }
    pub async fn set_enabled(
        &mut self,
        id: &str,
        enabled: bool,
    ) -> Result<InstalledExtension, Error> {
        self.client
            .set_extension_enabled(SetExtensionEnabledRequest {
                id: id.into(),
                enabled,
            })
            .await?
            .into_inner()
            .extension
            .ok_or_else(|| Error::msg("The daemon returned no extension"))
    }
    pub async fn rescan(&mut self) -> Result<Vec<InstalledExtension>, Error> {
        Ok(self
            .client
            .rescan_extensions(RescanExtensionsRequest {})
            .await?
            .into_inner()
            .extensions)
    }
}
