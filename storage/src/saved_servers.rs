//! The remote servers the user has saved.
//!
//! One list, in the daemon, so every client sees the same servers. It replaces
//! three stores that could not see each other: the Slint desktop's
//! `desktop_servers.json`, the single `subsonic_*`/`jellyfin_*` pair in
//! `settings.toml`, and whatever the web client happened to be connected to.
//! Both of the older stores are imported once, on first run, by
//! [`import_legacy_once`].
//!
//! `kind` is the source-registry key rather than an enum, so adding a new kind
//! of server is a factory in `addons` and nothing here.

use anyhow::Error;
use music_player_entity::saved_server;
use sea_orm::{
    ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};

pub use music_player_entity::saved_server::Model as SavedServer;

/// A server about to be written. `id` is derived, not supplied.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NewServer {
    pub kind: String,
    pub name: String,
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl NewServer {
    pub fn new(kind: impl Into<String>, name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            name: name.into(),
            url: normalize(&url.into()),
            username: None,
            password: None,
        }
    }

    pub fn with_credentials(mut self, username: Option<String>, password: Option<String>) -> Self {
        self.username = username.filter(|value| !value.trim().is_empty());
        self.password = password.filter(|value| !value.is_empty());
        self
    }

    /// The row id this server will have. Deterministic, so the caller can look
    /// one up before deciding whether to add it.
    pub fn id(&self) -> String {
        id_for(&self.kind, &self.url)
    }
}

/// `md5(kind + "\0" + url)` — the same rule `SourceConfig::id_for` uses, so an
/// id survives a round trip through either.
pub fn id_for(kind: &str, url: &str) -> String {
    format!("{:x}", md5::compute(format!("{kind}\0{}", normalize(url))))
}

fn normalize(url: &str) -> String {
    url.trim().trim_end_matches('/').to_string()
}

/// Every saved server, oldest first, so the list does not reshuffle itself
/// between reads.
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

/// Add a server, or update the one already saved at that url.
///
/// A `None` password on an *existing* row keeps whatever was stored: the edit
/// form never receives the password back (it is not exposed over the API), so
/// leaving the field blank has to mean "unchanged" rather than "clear it".
pub async fn upsert(
    db: &DatabaseConnection,
    server: &NewServer,
    now: &str,
) -> Result<SavedServer, Error> {
    let id = server.id();
    let existing = get(db, &id).await?;

    let password = match (&server.password, &existing) {
        (Some(password), _) => Some(password.clone()),
        (None, Some(row)) => row.password.clone(),
        (None, None) => None,
    };

    let row = saved_server::ActiveModel {
        id: Set(id.clone()),
        kind: Set(server.kind.clone()),
        name: Set(server.name.clone()),
        url: Set(server.url.clone()),
        username: Set(server.username.clone()),
        password: Set(password),
        created_at: match &existing {
            Some(row) => ActiveValue::Unchanged(row.created_at.clone()),
            None => Set(Some(now.to_owned())),
        },
        updated_at: Set(Some(now.to_owned())),
    };

    match existing {
        Some(_) => saved_server::Entity::update(row).exec(db).await?,
        None => {
            saved_server::Entity::insert(row)
                .exec_with_returning(db)
                .await?
        }
    };

    get(db, &id)
        .await?
        .ok_or_else(|| Error::msg("the server disappeared right after it was saved"))
}

/// Returns whether a row was actually removed.
pub async fn delete(db: &DatabaseConnection, id: &str) -> Result<bool, Error> {
    let result = saved_server::Entity::delete_many()
        .filter(saved_server::Column::Id.eq(id))
        .exec(db)
        .await?;
    Ok(result.rows_affected > 0)
}

/// Bring across servers from the two stores that predate this table.
///
/// Runs on every start and is idempotent twice over: the derived id makes an
/// upsert of the same server a no-op, and the JSON file is renamed once it has
/// been read. Existing rows are never overwritten — a server edited here
/// should not be reverted by a stale file.
///
/// Returns how many rows it added.
pub async fn import_legacy_once(db: &DatabaseConnection, now: &str) -> usize {
    let mut imported = 0;
    for server in legacy_json_servers()
        .into_iter()
        .chain(legacy_settings_servers())
    {
        match get(db, &server.id()).await {
            Ok(Some(_)) => continue,
            Ok(None) => {}
            Err(e) => {
                tracing::warn!("could not check for an existing server: {e}");
                continue;
            }
        }
        match upsert(db, &server, now).await {
            Ok(_) => imported += 1,
            Err(e) => tracing::warn!(url = %server.url, "could not import a saved server: {e}"),
        }
    }
    retire_legacy_json();
    imported
}

fn legacy_json_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|dir| dir.join("music-player").join("desktop_servers.json"))
}

/// The Slint desktop's own list, which only it could see.
fn legacy_json_servers() -> Vec<NewServer> {
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

    let Some(path) = legacy_json_path() else {
        return vec![];
    };
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return vec![];
    };
    let rows: Vec<Legacy> = serde_json::from_str(&raw).unwrap_or_default();
    rows.into_iter()
        .filter(|row| !row.url.trim().is_empty())
        .map(|row| {
            NewServer::new(row.kind, row.name, row.url)
                .with_credentials(Some(row.username), Some(row.password))
        })
        .collect()
}

fn retire_legacy_json() {
    if let Some(path) = legacy_json_path() {
        if path.exists() {
            // Renamed, not deleted: it is the only copy of those credentials
            // and the import is the kind of thing worth being able to check.
            let _ = std::fs::rename(&path, path.with_extension("json.migrated"));
        }
    }
}

/// The one Subsonic and one Jellyfin server `settings.toml` had room for.
fn legacy_settings_servers() -> Vec<NewServer> {
    use music_player_settings::{read_settings, Settings};

    let Ok(config) = read_settings() else {
        return vec![];
    };
    let Ok(settings) = config.try_deserialize::<Settings>() else {
        return vec![];
    };

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
    servers
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database, DbBackend, Schema};

    async fn db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let schema = Schema::new(DbBackend::Sqlite);
        db.execute(
            db.get_database_backend()
                .build(&schema.create_table_from_entity(saved_server::Entity)),
        )
        .await
        .unwrap();
        db
    }

    const NOW: &str = "2026-09-08T10:00:00Z";
    const LATER: &str = "2026-09-08T11:00:00Z";

    fn navidrome() -> NewServer {
        NewServer::new("subsonic", "NAS", "http://nas.lan:4533")
            .with_credentials(Some("tsiry".into()), Some("hunter2".into()))
    }

    #[tokio::test]
    async fn starts_empty() {
        assert!(list(&db().await).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn saves_and_reads_back_a_server() {
        let db = db().await;
        let saved = upsert(&db, &navidrome(), NOW).await.unwrap();

        assert_eq!(saved.kind, "subsonic");
        assert_eq!(saved.url, "http://nas.lan:4533");
        assert_eq!(saved.username.as_deref(), Some("tsiry"));
        assert_eq!(list(&db).await.unwrap().len(), 1);
        assert_eq!(get(&db, &saved.id).await.unwrap().unwrap().name, "NAS");
    }

    /// The same server added twice is one row — the id is derived from it.
    #[tokio::test]
    async fn re_adding_updates_rather_than_duplicates() {
        let db = db().await;
        upsert(&db, &navidrome(), NOW).await.unwrap();

        let renamed = NewServer::new("subsonic", "Living room", "http://nas.lan:4533/")
            .with_credentials(Some("tsiry".into()), Some("hunter2".into()));
        let saved = upsert(&db, &renamed, LATER).await.unwrap();

        assert_eq!(list(&db).await.unwrap().len(), 1);
        assert_eq!(saved.name, "Living room");
        // The first save is when it was added; that does not move.
        assert_eq!(saved.created_at.as_deref(), Some(NOW));
        assert_eq!(saved.updated_at.as_deref(), Some(LATER));
    }

    /// The API never hands the password back, so a blank field on the edit
    /// form means "leave it alone", not "clear it".
    #[tokio::test]
    async fn editing_without_a_password_keeps_the_stored_one() {
        let db = db().await;
        upsert(&db, &navidrome(), NOW).await.unwrap();

        let edited = NewServer::new("subsonic", "NAS", "http://nas.lan:4533")
            .with_credentials(Some("tsiry".into()), None);
        let saved = upsert(&db, &edited, LATER).await.unwrap();

        assert_eq!(saved.password.as_deref(), Some("hunter2"));
    }

    #[tokio::test]
    async fn kind_is_part_of_the_identity() {
        let db = db().await;
        upsert(&db, &NewServer::new("subsonic", "A", "http://h:8096"), NOW)
            .await
            .unwrap();
        upsert(&db, &NewServer::new("jellyfin", "B", "http://h:8096"), NOW)
            .await
            .unwrap();
        assert_eq!(list(&db).await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn deletes_and_says_whether_it_did() {
        let db = db().await;
        let saved = upsert(&db, &navidrome(), NOW).await.unwrap();

        assert!(delete(&db, &saved.id).await.unwrap());
        assert!(list(&db).await.unwrap().is_empty());
        // Deleting it again is not an error, it just did nothing.
        assert!(!delete(&db, &saved.id).await.unwrap());
    }

    #[tokio::test]
    async fn empty_credentials_are_stored_as_absent() {
        let db = db().await;
        let anonymous = NewServer::new("music-player", "peer", "http://peer.lan:5053")
            .with_credentials(Some("  ".into()), Some(String::new()));
        let saved = upsert(&db, &anonymous, NOW).await.unwrap();
        assert_eq!(saved.username, None);
        assert_eq!(saved.password, None);
    }
}
