//! Configured extension accounts use the daemon's shared manager. Browsing an
//! account never changes the native browsing provider or the active player.
use async_graphql::{Context, Enum, Error, InputObject, Object, SimpleObject, ID};
use music_player_playback::{
    extension_accounts::{ExtensionAccounts, SourceDefinition},
    source_resolver::LegacySourceList,
};
use music_player_storage::extension_source_accounts::{
    self as accounts, AccountProtocol, ConfigChange, SourceAccount,
};
use music_player_types::source::SourceHandle;
use std::collections::BTreeMap;

use super::media::MediaEntry;

#[cfg(test)]
mod tests;

#[derive(SimpleObject)]
pub struct ExtensionSourceDefinition {
    pub extension_id: ID,
    pub name: String,
    pub enabled: bool,
    pub source_api_version: i32,
    /// All declared keys; values are write-only because manifests do not label secrets.
    pub config_keys: Vec<String>,
}
impl From<SourceDefinition> for ExtensionSourceDefinition {
    fn from(value: SourceDefinition) -> Self {
        Self {
            extension_id: ID(value.extension_id),
            name: value.name,
            enabled: value.enabled,
            source_api_version: value.protocol.version(),
            config_keys: value.config_keys,
        }
    }
}

#[derive(SimpleObject)]
pub struct ExtensionSourceAccount {
    pub id: ID,
    pub extension_id: ID,
    pub name: String,
    pub source_api_version: i32,
    /// Decimal text preserves revisions beyond JavaScript's exact integer range.
    pub config_revision: String,
    /// A stored identity binding, not a claim that a remote server is reachable.
    pub identity_bound: bool,
    pub overridden_config_keys: Vec<String>,
}
impl ExtensionSourceAccount {
    fn from_row(row: SourceAccount) -> Result<Self, Error> {
        let overridden_config_keys = accounts::config(&row)
            .map_err(|_| Error::new("Source account configuration could not be read"))?
            .into_keys()
            .collect();
        let identity_bound = match accounts::protocol(&row)
            .map_err(|e| Error::new(e.to_string()))?
        {
            AccountProtocol::V1 => {
                row.local_bound_revision == Some(row.config_revision)
                    && row.local_config_json.is_some()
            }
            AccountProtocol::V2 => row.remote_server_id.is_some() && row.remote_user_id.is_some(),
        };
        Ok(Self {
            id: ID(row.id),
            extension_id: ID(row.extension_id),
            name: row.name,
            source_api_version: row.source_api_version,
            config_revision: row.config_revision.to_string(),
            identity_bound,
            overridden_config_keys,
        })
    }
}

#[derive(InputObject)]
pub struct SourceConfigValueInput {
    pub key: String,
    /// Empty string is an explicit value (for example an empty password).
    pub value: String,
}
#[derive(Enum, Clone, Copy, PartialEq, Eq)]
pub enum SourceConfigOperation {
    Set,
    Remove,
}
#[derive(InputObject)]
pub struct SourceConfigChangeInput {
    pub key: String,
    pub operation: SourceConfigOperation,
    pub value: Option<String>,
}

fn config_values(inputs: Vec<SourceConfigValueInput>) -> Result<BTreeMap<String, String>, Error> {
    let mut values = BTreeMap::new();
    for input in inputs {
        if values.insert(input.key, input.value).is_some() {
            return Err(Error::new(
                "A configuration key was supplied more than once",
            ));
        }
    }
    Ok(values)
}
fn config_changes(
    inputs: Vec<SourceConfigChangeInput>,
) -> Result<BTreeMap<String, ConfigChange>, Error> {
    let mut changes = BTreeMap::new();
    for input in inputs {
        let change = match (input.operation, input.value) {
            (SourceConfigOperation::Set, Some(value)) => ConfigChange::Set(value),
            (SourceConfigOperation::Remove, None) => ConfigChange::Remove,
            _ => {
                return Err(Error::new(
                    "SET requires a value; REMOVE must omit the value",
                ))
            }
        };
        if changes.insert(input.key, change).is_some() {
            return Err(Error::new(
                "A configuration key was supplied more than once",
            ));
        }
    }
    Ok(changes)
}

fn manager<'a>(ctx: &'a Context<'_>) -> Result<&'a ExtensionAccounts, Error> {
    Ok(crate::source_resolver(ctx)?.extension_accounts()?)
}

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
pub enum SourceBrowseList {
    Tracks,
    Albums,
    Artists,
    Playlists,
}
impl From<SourceBrowseList> for LegacySourceList {
    fn from(value: SourceBrowseList) -> Self {
        match value {
            SourceBrowseList::Tracks => Self::Tracks,
            SourceBrowseList::Albums => Self::Albums,
            SourceBrowseList::Artists => Self::Artists,
            SourceBrowseList::Playlists => Self::Playlists,
        }
    }
}
#[derive(SimpleObject)]
pub struct SourceMediaPage {
    pub account_id: ID,
    pub entries: Vec<MediaEntry>,
    pub next_offset: Option<String>,
}

fn page_offset(offset: Option<&str>) -> Result<u64, Error> {
    let text = offset.unwrap_or("0");
    let value = text
        .parse::<u64>()
        .map_err(|_| Error::new("Invalid source page offset"))?;
    if value.to_string() != text {
        return Err(Error::new("Invalid source page offset"));
    }
    Ok(value)
}

#[derive(Default)]
pub struct SourceAccountsQuery;
#[Object]
impl SourceAccountsQuery {
    async fn extension_source_definitions(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<ExtensionSourceDefinition>, Error> {
        Ok(manager(ctx)?
            .definitions()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }
    async fn source_accounts(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<ExtensionSourceAccount>, Error> {
        manager(ctx)?
            .list()
            .await?
            .into_iter()
            .map(ExtensionSourceAccount::from_row)
            .collect()
    }
    async fn source_account(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<ExtensionSourceAccount, Error> {
        ExtensionSourceAccount::from_row(manager(ctx)?.get(&id).await?)
    }
    /// V1 requires a list choice; v2 uses its root/container browse contract.
    /// nextOffset is explicit and never imposes a fixed total-library cap.
    async fn browse_source_media(
        &self,
        ctx: &Context<'_>,
        account_id: ID,
        parent: Option<ID>,
        list: Option<SourceBrowseList>,
        query: Option<String>,
        offset: Option<String>,
        limit: Option<i32>,
    ) -> Result<SourceMediaPage, Error> {
        let offset = page_offset(offset.as_deref())?;
        let limit = limit.unwrap_or(100);
        if !(1..=500).contains(&limit) {
            return Err(Error::new("Source pages need a limit from 1 to 500"));
        }
        let parent = parent
            .map(|id| SourceHandle::parse(&id))
            .transpose()
            .map_err(|e| Error::new(e.to_string()))?;
        let resolver = crate::source_resolver(ctx)?;
        let row = resolver.extension_accounts()?.get(&account_id).await?;
        let page = match accounts::protocol(&row).map_err(|e| Error::new(e.to_string()))? {
            AccountProtocol::V1 => {
                let list =
                    list.ok_or_else(|| Error::new("Choose a list for this legacy source"))?;
                let parent = match parent.as_ref() {
                    None => None,
                    Some(SourceHandle::Local(parent)) => Some(parent),
                    _ => return Err(Error::new("Parent does not belong to this source protocol")),
                };
                resolver
                    .extension_legacy_media(
                        &account_id,
                        list.into(),
                        parent,
                        query,
                        offset,
                        limit as u32,
                    )
                    .await?
            }
            AccountProtocol::V2 => {
                if list.is_some() {
                    return Err(Error::new("This source uses root and container navigation"));
                }
                let parent = match parent.as_ref() {
                    None => None,
                    Some(SourceHandle::Remote(parent)) => Some(parent),
                    _ => return Err(Error::new("Parent does not belong to this source protocol")),
                };
                resolver
                    .extension_media(&account_id, parent, query, offset, limit as u32)
                    .await?
            }
        };
        Ok(SourceMediaPage {
            account_id,
            entries: page.items.into_iter().map(Into::into).collect(),
            next_offset: page.next_offset.map(|offset| offset.to_string()),
        })
    }
}

#[derive(Default)]
pub struct SourceAccountsMutation;
#[Object]
impl SourceAccountsMutation {
    /// Saving does not connect or start playback. A lost reply may still have
    /// committed: clients must not automatically retry account creation.
    async fn create_source_account(
        &self,
        ctx: &Context<'_>,
        extension_id: ID,
        name: String,
        config: Option<Vec<SourceConfigValueInput>>,
    ) -> Result<ExtensionSourceAccount, Error> {
        let values = config_values(config.unwrap_or_default())?;
        let row = manager(ctx)?
            .create(
                &extension_id,
                &name,
                values,
                &chrono::Utc::now().to_rfc3339(),
            )
            .await?;
        ExtensionSourceAccount::from_row(row)
    }
    /// Omitted keys are unchanged, SET accepts empty strings, REMOVE restores
    /// the manifest default. An edit never silently binds new configuration.
    async fn update_source_account(
        &self,
        ctx: &Context<'_>,
        id: ID,
        expected_revision: String,
        name: Option<String>,
        changes: Option<Vec<SourceConfigChangeInput>>,
    ) -> Result<ExtensionSourceAccount, Error> {
        let changes = config_changes(changes.unwrap_or_default())?;
        let manager = manager(ctx)?;
        let row = manager.get(&id).await?;
        if row.config_revision.to_string() != expected_revision {
            return Err(Error::new("Source account changed; refresh before editing"));
        }
        let name = name.unwrap_or_else(|| row.name.clone());
        let row = manager
            .update(&row, &name, changes, &chrono::Utc::now().to_rfc3339())
            .await?;
        ExtensionSourceAccount::from_row(row)
    }
    /// Explicitly establish/confirm this account's binding. No player command.
    async fn connect_source_account(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<ExtensionSourceAccount, Error> {
        ExtensionSourceAccount::from_row(manager(ctx)?.connect(&id).await?)
    }
    async fn delete_source_account(&self, ctx: &Context<'_>, id: ID) -> Result<bool, Error> {
        manager(ctx)?.remove(&id).await?;
        Ok(true)
    }
}
