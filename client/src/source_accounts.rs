//! Extension accounts and browsing through the daemon's shared resolver.
//! Mutations are sent once. A missing reply does not prove they were rejected.
use anyhow::Error;
use music_player_server::api::music::v1alpha1::{
    source_accounts_service_client::SourceAccountsServiceClient, BrowseSourceMediaRequest,
    BrowseSourceMediaResponse, ConnectSourceAccountRequest, CreateSourceAccountRequest,
    DeleteSourceAccountRequest, ExtensionSourceAccount, ExtensionSourceDefinition,
    GetSourceAccountRequest, ListExtensionSourceDefinitionsRequest, ListSourceAccountsRequest,
    UpdateSourceAccountRequest,
};
use tonic::transport::Channel;

pub struct SourceAccountsClient {
    client: SourceAccountsServiceClient<Channel>,
}
impl SourceAccountsClient {
    pub async fn new(host: String, port: u16) -> Result<Self, Error> {
        let channel = Channel::from_shared(format!("http://{host}:{port}"))?
            .connect()
            .await?;
        Ok(Self::from_channel(channel))
    }
    pub fn from_channel(channel: Channel) -> Self {
        Self {
            client: SourceAccountsServiceClient::new(channel)
                .max_decoding_message_size(music_player_server::LIBRARY_MESSAGE_LIMIT)
                .max_encoding_message_size(music_player_server::LIBRARY_MESSAGE_LIMIT),
        }
    }
    pub async fn definitions(&mut self) -> Result<Vec<ExtensionSourceDefinition>, Error> {
        Ok(self
            .client
            .list_extension_source_definitions(ListExtensionSourceDefinitionsRequest {})
            .await?
            .into_inner()
            .definitions)
    }
    pub async fn list(&mut self) -> Result<Vec<ExtensionSourceAccount>, Error> {
        Ok(self
            .client
            .list_source_accounts(ListSourceAccountsRequest {})
            .await?
            .into_inner()
            .accounts)
    }
    pub async fn get(&mut self, id: &str) -> Result<ExtensionSourceAccount, Error> {
        account(
            self.client
                .get_source_account(GetSourceAccountRequest { id: id.into() })
                .await?
                .into_inner()
                .account,
        )
    }
    pub async fn create(
        &mut self,
        request: CreateSourceAccountRequest,
    ) -> Result<ExtensionSourceAccount, Error> {
        account(
            self.client
                .create_source_account(request)
                .await?
                .into_inner()
                .account,
        )
    }
    pub async fn update(
        &mut self,
        request: UpdateSourceAccountRequest,
    ) -> Result<ExtensionSourceAccount, Error> {
        account(
            self.client
                .update_source_account(request)
                .await?
                .into_inner()
                .account,
        )
    }
    pub async fn connect(&mut self, id: &str) -> Result<ExtensionSourceAccount, Error> {
        account(
            self.client
                .connect_source_account(ConnectSourceAccountRequest { id: id.into() })
                .await?
                .into_inner()
                .account,
        )
    }
    pub async fn delete(&mut self, id: &str) -> Result<bool, Error> {
        Ok(self
            .client
            .delete_source_account(DeleteSourceAccountRequest { id: id.into() })
            .await?
            .into_inner()
            .deleted)
    }
    pub async fn browse(
        &mut self,
        request: BrowseSourceMediaRequest,
    ) -> Result<BrowseSourceMediaResponse, Error> {
        Ok(self.client.browse_source_media(request).await?.into_inner())
    }
}
fn account(value: Option<ExtensionSourceAccount>) -> Result<ExtensionSourceAccount, Error> {
    value.ok_or_else(|| Error::msg("The daemon returned no source account"))
}
