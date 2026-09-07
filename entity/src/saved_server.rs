use sea_orm::entity::prelude::*;

/// A remote music server the user has saved: Subsonic/Navidrome, Jellyfin,
/// another music-player daemon, Kodi.
///
/// This lives in the daemon rather than in a client's own file so every client
/// sees the same list. It used to be three separate stores — the desktop's
/// `desktop_servers.json`, the `subsonic_*`/`jellyfin_*` keys in
/// `settings.toml` (which had room for exactly one of each), and whatever the
/// web client happened to be connected to — which meant adding a server in one
/// place left the others unaware of it.
///
/// `kind` is the source-registry key, so a new kind of server needs no schema
/// change: it is a string here and a factory there.
#[derive(Clone, Debug, Default, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "saved_server")]
pub struct Model {
    /// `md5(kind + "\0" + url)`, so adding the same server twice is one row.
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// `subsonic`, `jellyfin`, `music-player`, `kodi`, …
    pub kind: String,
    pub name: String,
    /// Absolute, scheme included, no trailing slash.
    pub url: String,
    pub username: Option<String>,
    /// Stored as given. Never returned over the API — callers are told only
    /// whether one is set.
    pub password: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
