//! Saved remote accounts, shared by every client of the daemon.
//! IDs survive configuration edits; credentials never determine identity.

use anyhow::{bail, Error};
use music_player_entity::saved_server;
use music_player_types::source::RemoteIdentity;
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait,
    QueryFilter, QueryOrder, Set, Statement, TransactionTrait,
};
use uuid::Uuid;

pub use music_player_entity::saved_server::Model as SavedServer;

/// Explicit updates preserve the difference between an empty and absent password.
#[derive(Clone, Default, PartialEq, Eq)]
pub enum PasswordUpdate {
    #[default]
    Keep,
    Set(String),
    Clear,
}

impl std::fmt::Debug for PasswordUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Keep => "Keep",
            Self::Set(_) => "Set([redacted])",
            Self::Clear => "Clear",
        })
    }
}

impl PasswordUpdate {
    /// Shared gRPC/GraphQL compatibility rule. Legacy empty input means Keep.
    pub fn from_fields(
        legacy: Option<String>,
        value: Option<String>,
        clear: bool,
    ) -> Result<Self, Error> {
        let legacy = legacy.filter(|password| !password.is_empty());
        if (clear && value.is_some()) || (legacy.is_some() && (clear || value.is_some())) {
            bail!("conflicting password updates");
        }
        Ok(if clear {
            Self::Clear
        } else if let Some(value) = value.or(legacy) {
            Self::Set(value)
        } else {
            Self::Keep
        })
    }

    fn value(&self) -> Option<String> {
        match self {
            Self::Set(value) => Some(value.clone()),
            Self::Keep | Self::Clear => None,
        }
    }
}

/// Without an ID, save by account tuple. With an ID, edit that exact account.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NewServer {
    pub id: Option<String>,
    pub kind: String,
    pub name: String,
    pub url: String,
    pub username: Option<String>,
    pub password_update: PasswordUpdate,
}

impl NewServer {
    pub fn new(kind: impl Into<String>, name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            name: name.into(),
            url: normalize(&url.into()),
            ..Self::default()
        }
    }

    /// Compatibility with older forms: an empty password leaves storage unchanged.
    pub fn with_credentials(mut self, username: Option<String>, password: Option<String>) -> Self {
        self.username = normalize_username(username);
        self.password_update = password
            .filter(|password| !password.is_empty())
            .map(PasswordUpdate::Set)
            .unwrap_or_default();
        self
    }

    pub fn with_id(mut self, id: Option<String>) -> Self {
        self.id = id;
        self
    }

    pub fn with_password_update(mut self, update: PasswordUpdate) -> Self {
        self.password_update = update;
        self
    }
}

fn normalize(url: &str) -> String {
    url.trim().trim_end_matches('/').to_owned()
}

fn normalize_username(username: Option<String>) -> Option<String> {
    username.filter(|value| !value.trim().is_empty())
}

pub async fn list(db: &DatabaseConnection) -> Result<Vec<SavedServer>, Error> {
    Ok(saved_server::Entity::find()
        .order_by_asc(saved_server::Column::CreatedAt)
        .order_by_asc(saved_server::Column::Name)
        .all(db)
        .await?)
}

pub async fn get(db: &DatabaseConnection, id: &str) -> Result<Option<SavedServer>, Error> {
    Ok(saved_server::Entity::find_by_id(id.to_owned())
        .one(db)
        .await?)
}

/// Pin an authenticated identity only while the saved authentication config matches.
/// Pass the saved row read BEFORE authentication, not one refreshed afterwards.
/// Name and updated_at are deliberately excluded: a name-only edit is allowed and
/// the returned row includes that edit. Binding itself changes neither timestamp.
/// This compares configuration values, not edit history (there is no revision).
pub async fn bind_remote_identity(
    db: &DatabaseConnection,
    auth_start_snapshot: &SavedServer,
    remote: &RemoteIdentity,
) -> Result<SavedServer, Error> {
    if remote.server_id.trim().is_empty() || remote.user_id.trim().is_empty() {
        bail!("remote identity requires both server and user IDs");
    }
    match (
        auth_start_snapshot.remote_server_id.as_deref(),
        auth_start_snapshot.remote_user_id.as_deref(),
    ) {
        (None, None) => {}
        (Some(server), Some(user)) if server == remote.server_id && user == remote.user_id => {}
        _ => bail!("authentication snapshot has a different or incomplete remote identity"),
    }

    // One UPDATE checks both config and binding while holding SQLite's write lock.
    // IS compares nullable values exactly: NULL and an explicit empty password
    // are different. Partial bindings cannot be silently repaired or overwritten.
    let statement = Statement::from_sql_and_values(
        DbBackend::Sqlite,
        "UPDATE saved_server SET remote_server_id = ?, remote_user_id = ?
        WHERE id = ? AND kind = ? AND url = ?
          AND username IS ? AND password IS ? AND created_at IS ?
          AND ((remote_server_id IS NULL AND remote_user_id IS NULL)
            OR (remote_server_id = ? AND remote_user_id = ?))
        RETURNING *",
        vec![
            remote.server_id.clone().into(),
            remote.user_id.clone().into(),
            auth_start_snapshot.id.clone().into(),
            auth_start_snapshot.kind.clone().into(),
            auth_start_snapshot.url.clone().into(),
            auth_start_snapshot.username.clone().into(),
            auth_start_snapshot.password.clone().into(),
            auth_start_snapshot.created_at.clone().into(),
            remote.server_id.clone().into(),
            remote.user_id.clone().into(),
        ],
    );
    saved_server::Entity::find()
        .from_raw_sql(statement)
        .one(db)
        .await
        // Do not expose database diagnostics that may contain bound credentials.
        .map_err(|_| Error::msg("remote identity binding database operation failed"))?
        .ok_or_else(|| {
            Error::msg("saved account changed, was removed, or has a different remote identity")
        })
}

/// Save atomically by (kind, normalized URL, normalized username), or edit an ID.
/// Saving does not reconnect a provider: active connections retain their snapshot.
/// Neither edit path writes the remote identity pair; only binding may set it.
pub async fn upsert(
    db: &DatabaseConnection,
    server: &NewServer,
    now: &str,
) -> Result<SavedServer, Error> {
    let username = normalize_username(server.username.clone());
    if let Some(id) = &server.id {
        let existing = get(db, id)
            .await?
            .ok_or_else(|| Error::msg("no such saved account"))?;
        if existing.kind != server.kind || existing.username != username {
            bail!("changing kind or username requires a new saved account");
        }
        // NotSet is essential: Keep must not write back a stale password read above.
        let password = match &server.password_update {
            PasswordUpdate::Keep => ActiveValue::NotSet,
            update => Set(update.value()),
        };
        // A tuple collision is an UPDATE error, never an upsert into another ID.
        // Leave the remote identity NotSet, including if authentication just bound it.
        return Ok(saved_server::Entity::update(saved_server::ActiveModel {
            id: Set(id.clone()),
            name: Set(server.name.clone()),
            url: Set(normalize(&server.url)),
            password,
            updated_at: Set(Some(now.to_owned())),
            ..Default::default()
        })
        .exec(db)
        .await?);
    }

    // The expression conflict target exactly matches the migration's unique index.
    // A single SQLite statement also preserves Keep during concurrent saves.
    let statement = insert_statement(server, now, true);
    saved_server::Entity::find()
        .from_raw_sql(statement)
        .one(db)
        .await?
        .ok_or_else(|| Error::msg("saving the account returned no row"))
}

fn insert_statement(server: &NewServer, now: &str, update: bool) -> Statement {
    let conflict = if update {
        "DO UPDATE SET name = excluded.name, updated_at = excluded.updated_at,
        password = CASE WHEN ? THEN saved_server.password ELSE excluded.password END RETURNING *"
    } else {
        "DO NOTHING"
    };
    let mut values = vec![
        Uuid::new_v4().to_string().into(),
        server.kind.clone().into(),
        server.name.clone().into(),
        normalize(&server.url).into(),
        normalize_username(server.username.clone()).into(),
        server.password_update.value().into(),
        now.to_owned().into(),
        now.to_owned().into(),
    ];
    if update {
        values.push(matches!(server.password_update, PasswordUpdate::Keep).into());
    }
    Statement::from_sql_and_values(
        DbBackend::Sqlite,
        format!("INSERT INTO saved_server (id, kind, name, url, username, password, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(kind, url, COALESCE(username, '')) {conflict}"),
        values,
    )
}

pub async fn delete(db: &DatabaseConnection, id: &str) -> Result<bool, Error> {
    Ok(saved_server::Entity::delete_many()
        .filter(saved_server::Column::Id.eq(id))
        .exec(db)
        .await?
        .rows_affected
        > 0)
}

const LEGACY_IMPORT: &str = "saved-accounts-v1";

async fn legacy_imported(db: &DatabaseConnection) -> Result<bool, Error> {
    Ok(db
        .query_one_raw(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "SELECT name FROM config_import WHERE name = ?",
            [LEGACY_IMPORT.into()],
        ))
        .await?
        .is_some())
}

/// Import only after schema migration, before the daemon exposes its account list.
/// Failure keeps the original input and leaves the import retryable next startup.
pub async fn import_legacy_once(db: &DatabaseConnection, now: &str) -> Result<usize, Error> {
    if legacy_imported(db).await? {
        return Ok(0);
    }
    let path = dirs::config_dir().map(|dir| dir.join("music-player").join("desktop_servers.json"));
    let mut servers = legacy_json_servers(path.as_deref())?;
    servers.extend(legacy_settings_servers()?);
    let count = import_legacy_accounts(db, &servers, now).await?;
    if let Some(path) = path.filter(|path| path.exists()) {
        // Retirement is cosmetic: the committed marker prevents stale re-import
        // even if the process exits between commit and rename.
        if let Err(error) = std::fs::rename(&path, path.with_extension("json.migrated")) {
            tracing::warn!(%error, "saved accounts imported but legacy file could not be renamed");
        }
    }
    Ok(count)
}

async fn import_legacy_accounts(
    db: &DatabaseConnection,
    servers: &[NewServer],
    now: &str,
) -> Result<usize, Error> {
    let transaction = db.begin().await?;
    // Claiming the marker is the first write, serializing concurrent importers.
    // It commits together with all rows, or rolls back together on any failure.
    let claim = transaction.execute_raw(Statement::from_sql_and_values(
        DbBackend::Sqlite,
        "INSERT INTO config_import (name, completed_at) VALUES (?, ?) ON CONFLICT(name) DO NOTHING",
        [LEGACY_IMPORT.into(), now.into()],
    )).await?;
    let mut imported = 0;
    if claim.rows_affected() > 0 {
        for server in servers {
            imported += transaction
                .execute_raw(insert_statement(server, now, false))
                .await?
                .rows_affected() as usize;
        }
    }
    transaction.commit().await?;
    Ok(imported)
}

fn legacy_json_servers(path: Option<&std::path::Path>) -> Result<Vec<NewServer>, Error> {
    #[derive(serde::Deserialize)]
    struct Legacy {
        kind: String,
        name: String,
        url: String,
        #[serde(default)]
        username: String,
        #[serde(default)]
        password: String,
    }
    let Some(path) = path else {
        return Ok(vec![]);
    };
    let raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(error) => return Err(error.into()),
    };
    let rows: Vec<Legacy> = serde_json::from_str(&raw)?;
    Ok(rows
        .into_iter()
        .filter(|row| !row.url.trim().is_empty())
        .map(|row| {
            NewServer::new(row.kind, row.name, row.url)
                .with_credentials(Some(row.username), Some(row.password))
        })
        .collect())
}

fn legacy_settings_servers() -> Result<Vec<NewServer>, Error> {
    use music_player_settings::{read_settings, Settings};
    let settings = read_settings()?.try_deserialize::<Settings>()?;
    let mut servers = Vec::new();
    let mut push = |kind: &str, name: &str, url: Option<String>, user, password| {
        if let Some(url) = url.filter(|url| !url.trim().is_empty()) {
            servers.push(NewServer::new(kind, name, url).with_credentials(user, password));
        }
    };
    push(
        "subsonic",
        "Subsonic",
        settings.subsonic_url,
        settings.subsonic_username,
        settings.subsonic_password,
    );
    push(
        "jellyfin",
        "Jellyfin",
        settings.jellyfin_url,
        settings.jellyfin_username,
        settings.jellyfin_password,
    );
    Ok(servers)
}

#[cfg(test)]
mod tests;
