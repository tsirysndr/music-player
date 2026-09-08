use async_graphql::*;
use music_player_provider::ProviderKindInfo;
use music_player_storage::saved_servers::SavedServer;

/// A remote server the user has saved.
///
/// Note the absence of `password`. It is stored, and it is sent when a server
/// is added or edited, but it is never returned — so an edit form has to
/// re-prompt for it rather than round-tripping a secret through a client.
#[derive(SimpleObject, Clone, Debug)]
pub struct Server {
    pub id: ID,
    /// The provider-registry key: `subsonic`, `jellyfin`, `music-player`, …
    pub kind: String,
    pub name: String,
    pub url: String,
    pub username: Option<String>,
    /// Whether a password is stored, since the password itself is not exposed.
    pub has_password: bool,
    /// Whether this is the server the library screens are currently reading
    /// from.
    pub connected: bool,
}

impl Server {
    pub fn from_row(row: SavedServer, connected_id: Option<&str>) -> Self {
        let connected = connected_id == Some(row.id.as_str());
        Self {
            id: ID(row.id),
            kind: row.kind,
            name: row.name,
            url: row.url,
            username: row.username,
            has_password: row.password.is_some_and(|password| !password.is_empty()),
            connected,
        }
    }
}

/// What a client needs to offer one kind of server in its add-server form.
///
/// Comes straight from the provider registry, so a newly registered backend
/// appears in every client without either of them being changed.
#[derive(SimpleObject, Clone, Debug)]
pub struct SourceKind {
    pub kind: String,
    pub display_name: String,
    /// False for backends with no login — the form hides those fields rather
    /// than asking for something that will be ignored.
    pub needs_credentials: bool,
    pub default_port: i32,
    /// Set for a backend that always talks to one address, so the form can
    /// drop the url field instead of asking for something it ignores.
    pub fixed_url: Option<String>,
}

impl From<ProviderKindInfo> for SourceKind {
    fn from(info: ProviderKindInfo) -> Self {
        Self {
            kind: info.kind.to_string(),
            display_name: info.display_name.to_string(),
            needs_credentials: info.needs_credentials,
            default_port: info.default_port as i32,
            fixed_url: info.fixed_url.map(str::to_owned),
        }
    }
}

#[derive(InputObject, Clone, Debug)]
pub struct ServerInput {
    pub kind: String,
    pub name: String,
    pub url: String,
    pub username: Option<String>,
    /// Omit to keep whatever is already stored — the edit form never receives
    /// the current one, so a blank field cannot mean "clear it".
    pub password: Option<String>,
}
