//! Stable configured plugin accounts. Configuration values never determine IDs.
use anyhow::{bail, Error};
use music_player_entity::extension_source_account as account;
use music_player_types::source::{RemoteIdentity, SourceRef};
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbBackend, EntityTrait, Set, Statement};
use std::collections::BTreeMap;

pub use account::Model as SourceAccount;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountProtocol {
    V1,
    V2,
}
impl AccountProtocol {
    pub fn version(self) -> i32 {
        match self {
            Self::V1 => 1,
            Self::V2 => 2,
        }
    }
}
pub fn protocol(snapshot: &SourceAccount) -> Result<AccountProtocol, Error> {
    match snapshot.source_api_version {
        1 => Ok(AccountProtocol::V1),
        2 => Ok(AccountProtocol::V2),
        _ => Err(failed()),
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum ConfigChange {
    /// An empty string is an explicit value. Omitted keys keep their value.
    Set(String),
    /// Remove the override so the manifest default applies on next load.
    Remove,
}
impl std::fmt::Debug for ConfigChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Set(_) => "Set([redacted])",
            Self::Remove => "Remove",
        })
    }
}

fn failed() -> Error {
    Error::msg("extension source account operation failed")
}
fn changed() -> Error {
    Error::msg("extension source account changed or was removed")
}

pub fn config(snapshot: &SourceAccount) -> Result<BTreeMap<String, String>, Error> {
    serde_json::from_str(&snapshot.config_json).map_err(|_| failed())
}

pub async fn get(db: &DatabaseConnection, id: &str) -> Result<Option<SourceAccount>, Error> {
    account::Entity::find_by_id(id.to_owned())
        .one(db)
        .await
        .map_err(|_| failed())
}

pub async fn list(db: &DatabaseConnection) -> Result<Vec<SourceAccount>, Error> {
    let mut rows = account::Entity::find()
        .all(db)
        .await
        .map_err(|_| failed())?;
    rows.sort_by(|a, b| (&a.name, &a.id).cmp(&(&b.name, &b.id)));
    Ok(rows)
}

/// The host validates keys against the installed manifest before saving.
/// Each creation is a distinct account even when plugin and config match.
pub async fn create(
    db: &DatabaseConnection,
    extension_id: &str,
    name: &str,
    overrides: &BTreeMap<String, String>,
    now: &str,
) -> Result<SourceAccount, Error> {
    create_for_protocol(db, extension_id, name, overrides, now, AccountProtocol::V2).await
}

/// Protocol is fixed with the account. Upgrades require an explicitly created
/// account, since v1 local IDs cannot be asserted equivalent to a v2 identity.
pub async fn create_for_protocol(
    db: &DatabaseConnection,
    extension_id: &str,
    name: &str,
    overrides: &BTreeMap<String, String>,
    now: &str,
    protocol: AccountProtocol,
) -> Result<SourceAccount, Error> {
    SourceRef::extension_resolver(extension_id).map_err(|_| failed())?;
    if name.trim().is_empty() {
        bail!("source account name is empty");
    }
    account::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        extension_id: Set(extension_id.to_owned()),
        source_api_version: Set(protocol.version()),
        name: Set(name.to_owned()),
        config_json: Set(serde_json::to_string(overrides).map_err(|_| failed())?),
        config_revision: Set(1),
        remote_server_id: Set(None),
        remote_user_id: Set(None),
        local_bound_revision: Set(None),
        local_config_json: Set(None),
        created_at: Set(now.to_owned()),
        updated_at: Set(now.to_owned()),
    }
    .insert(db)
    .await
    .map_err(|_| failed())
}

/// Compare-and-set prevents concurrent configuration changes from being lost.
/// Neither name nor credential edits clear the existing remote identity pair.
/// Name is compared here to avoid overwriting a concurrent rename. It remains
/// excluded from authentication binding, which only cares about configuration.
pub async fn update(
    db: &DatabaseConnection,
    snapshot: &SourceAccount,
    name: &str,
    changes: &BTreeMap<String, ConfigChange>,
    now: &str,
) -> Result<SourceAccount, Error> {
    if name.trim().is_empty() {
        bail!("source account name is empty");
    }
    let mut overrides = config(snapshot)?;
    for (key, change) in changes {
        match change {
            ConfigChange::Set(value) => {
                overrides.insert(key.clone(), value.clone());
            }
            ConfigChange::Remove => {
                overrides.remove(key);
            }
        }
    }
    let json = serde_json::to_string(&overrides).map_err(|_| failed())?;
    let revision = if json == snapshot.config_json {
        snapshot.config_revision
    } else {
        snapshot.config_revision.checked_add(1).ok_or_else(failed)?
    };
    let statement = Statement::from_sql_and_values(DbBackend::Sqlite,
        "UPDATE extension_source_account SET name = ?, config_json = ?, config_revision = ?, updated_at = ?,
         local_bound_revision = CASE WHEN source_api_version = 1 AND config_revision != ?
                                    THEN NULL ELSE local_bound_revision END,
         local_config_json = CASE WHEN source_api_version = 1 AND config_revision != ?
                                 THEN NULL ELSE local_config_json END
         WHERE id = ? AND extension_id = ? AND source_api_version = ?
         AND config_revision = ? AND config_json = ? AND created_at = ? AND name = ?
         RETURNING *",
        vec![name.to_owned().into(), json.into(), revision.into(), now.to_owned().into(), revision.into(), revision.into(),
             snapshot.id.clone().into(), snapshot.extension_id.clone().into(),
             snapshot.source_api_version.into(),
             snapshot.config_revision.into(), snapshot.config_json.clone().into(), snapshot.created_at.clone().into(),
             snapshot.name.clone().into()]);
    account::Entity::find()
        .from_raw_sql(statement)
        .one(db)
        .await
        .map_err(|_| failed())?
        .ok_or_else(changed)
}

/// Bind only the exact configuration used to authenticate. Subsequent binds may
/// confirm that same remote identity, but may never substitute another user.
pub async fn bind_remote_identity(
    db: &DatabaseConnection,
    snapshot: &SourceAccount,
    remote: &RemoteIdentity,
) -> Result<SourceAccount, Error> {
    if protocol(snapshot)? != AccountProtocol::V2
        || snapshot.local_bound_revision.is_some()
        || snapshot.local_config_json.is_some()
    {
        return Err(failed());
    }
    if remote.server_id.trim().is_empty() || remote.user_id.trim().is_empty() {
        bail!("source requires both remote server and user IDs");
    }
    match (&snapshot.remote_server_id, &snapshot.remote_user_id) {
        (None, None) => {}
        (Some(server), Some(user)) if server == &remote.server_id && user == &remote.user_id => {}
        _ => bail!("source account remote identity does not match"),
    }
    let statement = Statement::from_sql_and_values(DbBackend::Sqlite,
        "UPDATE extension_source_account SET remote_server_id = ?, remote_user_id = ?
         WHERE id = ? AND extension_id = ? AND config_revision = ? AND config_json = ? AND created_at = ?
         AND source_api_version = 2 AND local_bound_revision IS NULL AND local_config_json IS NULL
         AND ((remote_server_id IS NULL AND remote_user_id IS NULL)
              OR (remote_server_id = ? AND remote_user_id = ?)) RETURNING *",
        vec![remote.server_id.clone().into(), remote.user_id.clone().into(),
             snapshot.id.clone().into(), snapshot.extension_id.clone().into(),
             snapshot.config_revision.into(), snapshot.config_json.clone().into(), snapshot.created_at.clone().into(),
             remote.server_id.clone().into(), remote.user_id.clone().into()]);
    account::Entity::find()
        .from_raw_sql(statement)
        .one(db)
        .await
        .map_err(|_| failed())?
        .ok_or_else(changed)
}

/// Called after source_info succeeds on exactly this configuration. The value
/// is a local binding, never a server or user ID returned by the source.
pub async fn bind_local_configuration(
    db: &DatabaseConnection,
    snapshot: &SourceAccount,
    effective: &BTreeMap<String, String>,
) -> Result<SourceAccount, Error> {
    if protocol(snapshot)? != AccountProtocol::V1
        || snapshot.config_revision < 1
        || snapshot.remote_server_id.is_some()
        || snapshot.remote_user_id.is_some()
        || snapshot
            .local_bound_revision
            .is_some_and(|revision| revision != snapshot.config_revision)
        || snapshot.local_bound_revision.is_some() != snapshot.local_config_json.is_some()
    {
        return Err(failed());
    }
    let json = serde_json::to_string(effective).map_err(|_| failed())?;
    // Only explicit connect uses this operation. A changed manifest default
    // creates a new scope even when the user's override map did not change.
    let revision = if snapshot
        .local_config_json
        .as_ref()
        .is_some_and(|old| old != &json)
    {
        snapshot.config_revision.checked_add(1).ok_or_else(failed)?
    } else {
        snapshot.config_revision
    };
    let statement = Statement::from_sql_and_values(
        DbBackend::Sqlite,
        "UPDATE extension_source_account SET local_bound_revision = ?, local_config_json = ?, config_revision = ?
         WHERE id = ? AND extension_id = ? AND source_api_version = 1
         AND config_revision = ? AND config_json = ? AND created_at = ?
         AND remote_server_id IS NULL AND remote_user_id IS NULL
         AND local_bound_revision IS ? AND local_config_json IS ? RETURNING *",
        vec![
            revision.into(),
            json.into(),
            revision.into(),
            snapshot.id.clone().into(),
            snapshot.extension_id.clone().into(),
            snapshot.config_revision.into(),
            snapshot.config_json.clone().into(),
            snapshot.created_at.clone().into(),
            snapshot.local_bound_revision.into(),
            snapshot.local_config_json.clone().into(),
        ],
    );
    account::Entity::find()
        .from_raw_sql(statement)
        .one(db)
        .await
        .map_err(|_| failed())?
        .ok_or_else(changed)
}

pub async fn remove(db: &DatabaseConnection, id: &str) -> Result<(), Error> {
    account::Entity::delete_by_id(id.to_owned())
        .exec(db)
        .await
        .map_err(|_| failed())?;
    Ok(())
}

#[cfg(test)]
mod tests;
