//! All extension account operations share the resolver used by playback.
use music_player_playback::{
    extension_accounts::{AccountError, ExtensionAccounts},
    source_resolver::{LegacySourceList, SourceResolver},
};
use music_player_storage::extension_source_accounts::{
    self as accounts, AccountProtocol, ConfigChange, SourceAccount,
};
use music_player_types::source::SourceHandle;
use std::collections::BTreeMap;
use tonic::{Request, Response, Status};

use crate::api::music::v1alpha1::{
    source_accounts_service_server::SourceAccountsService, source_config_change::Change,
    BrowseSourceMediaRequest, BrowseSourceMediaResponse, ConnectSourceAccountRequest,
    ConnectSourceAccountResponse, CreateSourceAccountRequest, CreateSourceAccountResponse,
    DeleteSourceAccountRequest, DeleteSourceAccountResponse, ExtensionSourceAccount,
    ExtensionSourceDefinition, GetSourceAccountRequest, GetSourceAccountResponse,
    ListExtensionSourceDefinitionsRequest, ListExtensionSourceDefinitionsResponse,
    ListSourceAccountsRequest, ListSourceAccountsResponse, MediaEntry, SourceConfigChange,
    SourceConfigValue, UpdateSourceAccountRequest, UpdateSourceAccountResponse,
};

pub struct SourceAccounts {
    sources: SourceResolver,
}
impl SourceAccounts {
    pub fn new(sources: SourceResolver) -> Self {
        Self { sources }
    }
    fn manager(&self) -> Result<&ExtensionAccounts, Status> {
        self.sources.extension_accounts().map_err(precondition)
    }
}
fn precondition(error: impl std::fmt::Display) -> Status {
    Status::failed_precondition(error.to_string())
}
fn account_error(error: AccountError) -> Status {
    let message = error.to_string();
    match error {
        AccountError::Busy => Status::resource_exhausted(message),
        AccountError::Missing => Status::not_found(message),
        AccountError::InvalidSource => Status::invalid_argument(message),
        AccountError::Changed => Status::aborted(message),
        AccountError::Storage => Status::internal(message),
        _ => Status::failed_precondition(message),
    }
}
fn account(row: SourceAccount) -> Result<ExtensionSourceAccount, Status> {
    let protocol = accounts::protocol(&row).map_err(precondition)?;
    let identity_bound = match protocol {
        AccountProtocol::V1 => {
            row.local_bound_revision == Some(row.config_revision) && row.local_config_json.is_some()
        }
        AccountProtocol::V2 => row.remote_server_id.is_some() && row.remote_user_id.is_some(),
    };
    let overridden_config_keys = accounts::config(&row)
        .map_err(|_| Status::internal("Source account configuration could not be read"))?
        .into_keys()
        .collect();
    let config_revision = u64::try_from(row.config_revision)
        .ok()
        .filter(|revision| *revision > 0)
        .ok_or_else(|| Status::internal("Invalid source account revision"))?;
    Ok(ExtensionSourceAccount {
        id: row.id,
        extension_id: row.extension_id,
        name: row.name,
        source_api_version: protocol.version() as u32,
        config_revision,
        identity_bound,
        overridden_config_keys,
    })
}
fn config(values: Vec<SourceConfigValue>) -> Result<BTreeMap<String, String>, Status> {
    let mut result = BTreeMap::new();
    for value in values {
        if result.insert(value.key, value.value).is_some() {
            return Err(Status::invalid_argument(
                "A configuration key was supplied more than once",
            ));
        }
    }
    Ok(result)
}
fn changes(values: Vec<SourceConfigChange>) -> Result<BTreeMap<String, ConfigChange>, Status> {
    let mut result = BTreeMap::new();
    for value in values {
        let change = match value.change {
            Some(Change::SetValue(value)) => ConfigChange::Set(value),
            Some(Change::Remove(true)) => ConfigChange::Remove,
            _ => {
                return Err(Status::invalid_argument(
                    "A configuration change needs a value or remove=true",
                ))
            }
        };
        if result.insert(value.key, change).is_some() {
            return Err(Status::invalid_argument(
                "A configuration key was supplied more than once",
            ));
        }
    }
    Ok(result)
}

#[tonic::async_trait]
impl SourceAccountsService for SourceAccounts {
    async fn list_extension_source_definitions(
        &self,
        _: Request<ListExtensionSourceDefinitionsRequest>,
    ) -> Result<Response<ListExtensionSourceDefinitionsResponse>, Status> {
        let definitions = self
            .manager()?
            .definitions()
            .await
            .map_err(account_error)?
            .into_iter()
            .map(|item| ExtensionSourceDefinition {
                extension_id: item.extension_id,
                name: item.name,
                enabled: item.enabled,
                source_api_version: item.protocol.version() as u32,
                config_keys: item.config_keys,
            })
            .collect();
        Ok(Response::new(ListExtensionSourceDefinitionsResponse {
            definitions,
        }))
    }
    async fn list_source_accounts(
        &self,
        _: Request<ListSourceAccountsRequest>,
    ) -> Result<Response<ListSourceAccountsResponse>, Status> {
        let accounts = self
            .manager()?
            .list()
            .await
            .map_err(account_error)?
            .into_iter()
            .map(account)
            .collect::<Result<_, _>>()?;
        Ok(Response::new(ListSourceAccountsResponse { accounts }))
    }
    async fn get_source_account(
        &self,
        request: Request<GetSourceAccountRequest>,
    ) -> Result<Response<GetSourceAccountResponse>, Status> {
        let row = self
            .manager()?
            .get(&request.into_inner().id)
            .await
            .map_err(account_error)?;
        Ok(Response::new(GetSourceAccountResponse {
            account: Some(account(row)?),
        }))
    }
    async fn create_source_account(
        &self,
        request: Request<CreateSourceAccountRequest>,
    ) -> Result<Response<CreateSourceAccountResponse>, Status> {
        let request = request.into_inner();
        let values = config(request.config)?;
        let row = self
            .manager()?
            .create(
                &request.extension_id,
                &request.name,
                values,
                &chrono::Utc::now().to_rfc3339(),
            )
            .await
            .map_err(account_error)?;
        Ok(Response::new(CreateSourceAccountResponse {
            account: Some(account(row)?),
        }))
    }
    async fn update_source_account(
        &self,
        request: Request<UpdateSourceAccountRequest>,
    ) -> Result<Response<UpdateSourceAccountResponse>, Status> {
        let request = request.into_inner();
        let changes = changes(request.changes)?;
        let manager = self.manager()?;
        let snapshot = manager.get(&request.id).await.map_err(account_error)?;
        if u64::try_from(snapshot.config_revision).ok() != Some(request.expected_revision) {
            return Err(Status::aborted(
                "Source account changed; refresh before editing",
            ));
        }
        let name = request.name.unwrap_or_else(|| snapshot.name.clone());
        let row = manager
            .update(&snapshot, &name, changes, &chrono::Utc::now().to_rfc3339())
            .await
            .map_err(account_error)?;
        Ok(Response::new(UpdateSourceAccountResponse {
            account: Some(account(row)?),
        }))
    }
    async fn connect_source_account(
        &self,
        request: Request<ConnectSourceAccountRequest>,
    ) -> Result<Response<ConnectSourceAccountResponse>, Status> {
        let row = self
            .manager()?
            .connect(&request.into_inner().id)
            .await
            .map_err(account_error)?;
        Ok(Response::new(ConnectSourceAccountResponse {
            account: Some(account(row)?),
        }))
    }
    async fn delete_source_account(
        &self,
        request: Request<DeleteSourceAccountRequest>,
    ) -> Result<Response<DeleteSourceAccountResponse>, Status> {
        self.manager()?
            .remove(&request.into_inner().id)
            .await
            .map_err(account_error)?;
        Ok(Response::new(DeleteSourceAccountResponse { deleted: true }))
    }
    async fn browse_source_media(
        &self,
        request: Request<BrowseSourceMediaRequest>,
    ) -> Result<Response<BrowseSourceMediaResponse>, Status> {
        let request = request.into_inner();
        let limit = request.limit.unwrap_or(100);
        if !(1..=500).contains(&limit) {
            return Err(Status::invalid_argument(
                "Source pages need a limit from 1 to 500",
            ));
        }
        let parent = request
            .parent
            .as_deref()
            .map(SourceHandle::parse)
            .transpose()
            .map_err(precondition)?;
        let row = self
            .manager()?
            .get(&request.account_id)
            .await
            .map_err(account_error)?;
        let page = match accounts::protocol(&row).map_err(precondition)? {
            AccountProtocol::V1 => {
                let list = match request.list {
                    1 => LegacySourceList::Tracks,
                    2 => LegacySourceList::Albums,
                    3 => LegacySourceList::Artists,
                    4 => LegacySourceList::Playlists,
                    _ => {
                        return Err(Status::invalid_argument(
                            "Choose a list for this legacy source",
                        ))
                    }
                };
                let parent = match parent.as_ref() {
                    None => None,
                    Some(SourceHandle::Local(parent)) => Some(parent),
                    _ => {
                        return Err(Status::invalid_argument(
                            "Parent does not belong to this source protocol",
                        ))
                    }
                };
                self.sources
                    .extension_legacy_media(
                        &request.account_id,
                        list,
                        parent,
                        request.query,
                        request.offset,
                        limit,
                    )
                    .await
            }
            AccountProtocol::V2 => {
                if request.list != 0 {
                    return Err(Status::invalid_argument(
                        "This source uses root and container navigation",
                    ));
                }
                let parent = match parent.as_ref() {
                    None => None,
                    Some(SourceHandle::Remote(parent)) => Some(parent),
                    _ => {
                        return Err(Status::invalid_argument(
                            "Parent does not belong to this source protocol",
                        ))
                    }
                };
                self.sources
                    .extension_media(
                        &request.account_id,
                        parent,
                        request.query,
                        request.offset,
                        limit,
                    )
                    .await
            }
        }
        .map_err(precondition)?;
        Ok(Response::new(BrowseSourceMediaResponse {
            account_id: request.account_id,
            next_offset: page.next_offset,
            entries: page
                .items
                .into_iter()
                .map(|item| MediaEntry {
                    id: item.id,
                    title: item.title,
                    item_type: item.item_type,
                    media_type: item.media_type,
                    is_container: item.is_container,
                    season_number: item.season_number,
                    episode_number: item.episode_number,
                    track: item.track.map(Into::into),
                })
                .collect(),
        }))
    }
}

#[cfg(test)]
mod tests;
