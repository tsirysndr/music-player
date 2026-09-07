use async_graphql::*;
use music_player_extensions::catalog::Installed;

/// An installed WebAssembly extension, as its manifest declares it.
#[derive(Default, Clone, Debug, SimpleObject)]
pub struct Extension {
    /// Reverse-DNS id, e.g. `com.example.lyrics`. Stable across versions.
    pub id: ID,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub homepage: String,
    pub repository: String,
    pub license: String,
    /// A URL, or a file name relative to the extension's directory.
    pub logo: String,
    /// Free-form tags, e.g. `["lyrics", "offline"]`.
    pub topics: Vec<String>,
    /// What it plugs into: `events`, `metadata`, `commands`, `predicates`,
    /// `source`. An extension only ever receives calls for what it declared.
    pub capabilities: Vec<String>,
    /// Hosts it may reach over HTTP. Empty means no network at all.
    pub allowed_hosts: Vec<String>,
    /// Whether it may query the library through the host functions.
    pub library_read: bool,
    /// `enabled` or `disabled`.
    ///
    /// This is a manifest scan, so it cannot report whether an enabled module
    /// actually loads — that needs the module. The daemon logs a load failure
    /// at startup.
    pub status: String,
    /// The directory it was read from, so a user can find it on disk.
    pub path: String,
}

impl From<Installed> for Extension {
    fn from(installed: Installed) -> Self {
        let manifest = installed.manifest;
        Self {
            id: ID(manifest.id),
            name: manifest.name,
            version: manifest.version,
            author: manifest.author,
            description: manifest.description,
            homepage: manifest.homepage,
            repository: manifest.repository,
            license: manifest.license,
            logo: manifest.logo,
            topics: manifest.topics,
            capabilities: manifest
                .capabilities
                .iter()
                .map(|capability| capability.as_str().to_owned())
                .collect(),
            allowed_hosts: manifest.permissions.allowed_hosts,
            library_read: manifest.permissions.library_read,
            status: installed.status.as_str().to_owned(),
            path: installed.dir.display().to_string(),
        }
    }
}
