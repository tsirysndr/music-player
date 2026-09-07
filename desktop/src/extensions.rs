//! The extensions view's data.
//!
//! Reads the manifests in the configured search paths and pairs each with the
//! enabled flag stored in the `extension` table, exactly as the daemon's
//! `extensions` GraphQL query does — the two clients show the same list from
//! the same scan rather than from two different notions of what is installed.
//!
//! Nothing here loads a module: instantiating WebAssembly to draw a list would
//! cost the user a visible pause on a tab they may only be browsing. The one
//! thing that costs is whether an enabled module *would* load, which is why
//! [`Extension::error`] is only ever filled in by the daemon's own startup log,
//! not by this.

use anyhow::Error;
use music_player_extensions::{catalog, search_paths};
use music_player_settings::{get_application_directory, read_settings, Settings};
use music_player_storage::extension_state;

/// One installed extension, flattened into what the row displays.
#[derive(Clone, Debug, Default)]
pub struct Extension {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    /// Capabilities joined for display, e.g. `"metadata, source"`.
    pub capabilities: String,
    /// Allowed hosts joined; empty means no network at all.
    pub hosts: String,
    pub library_read: bool,
    pub enabled: bool,
    pub error: String,
    /// Everything searchable about it, lowercased once at scan time so the
    /// filter is a substring test rather than a rebuild per keystroke.
    haystack: String,
}

impl Extension {
    /// Whether this matches a (already lowercased) search term.
    pub fn matches(&self, needle: &str) -> bool {
        needle.is_empty() || self.haystack.contains(needle)
    }
}

/// The configured extension paths, or none when settings cannot be read —
/// which only means falling back to the default location.
fn extension_paths() -> Vec<String> {
    read_settings()
        .ok()
        .and_then(|config| config.try_deserialize::<Settings>().ok())
        .map(|settings| settings.extension_paths)
        .unwrap_or_default()
}

/// Scan every search path and return what is installed, in id order.
pub async fn load() -> Vec<Extension> {
    let paths = search_paths(&get_application_directory(), &extension_paths());
    let db = music_player_storage::shared().await;
    let enabled = extension_state::enabled_map(db.get_connection()).await;

    catalog::installed(&paths, &enabled)
        .into_iter()
        .map(|installed| {
            // Read the flag before the manifest moves out of `installed`.
            let enabled = installed.enabled();
            let manifest = installed.manifest;
            let capabilities = manifest
                .capabilities
                .iter()
                .map(|capability| capability.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            let hosts = manifest.permissions.allowed_hosts.join(", ");
            let haystack = [
                manifest.id.as_str(),
                manifest.name.as_str(),
                manifest.description.as_str(),
                manifest.author.as_str(),
                capabilities.as_str(),
                manifest.topics.join(" ").as_str(),
            ]
            .join(" ")
            .to_lowercase();

            Extension {
                id: manifest.id,
                name: manifest.name,
                version: manifest.version,
                author: manifest.author,
                description: manifest.description,
                capabilities,
                hosts,
                library_read: manifest.permissions.library_read,
                enabled,
                error: String::new(),
                haystack,
            }
        })
        .collect()
}

/// Switch an extension on or off, and hand back the refreshed list.
///
/// The flag is stored immediately, but a module the daemon already loaded
/// keeps running until it next starts — unloading WebAssembly mid-session
/// would pull the ground out from under whatever is calling into it.
pub async fn set_enabled(id: &str, enabled: bool) -> Result<Vec<Extension>, Error> {
    let db = music_player_storage::shared().await;
    extension_state::set_enabled(
        db.get_connection(),
        id,
        enabled,
        &chrono::Utc::now().to_rfc3339(),
    )
    .await?;
    Ok(load().await)
}

/// Re-read the directories, dropping the stored flags for anything that is no
/// longer installed.
///
/// A leftover row is harmless while it matches nothing, but it would switch an
/// extension straight back off if it were ever reinstalled, which is not what
/// deleting it meant.
pub async fn rescan() -> Vec<Extension> {
    let extensions = load().await;
    let installed: Vec<String> = extensions
        .iter()
        .map(|extension| extension.id.clone())
        .collect();

    let db = music_player_storage::shared().await;
    if let Err(e) = extension_state::prune(db.get_connection(), &installed).await {
        // Not fatal: the list is still right, there is just a stale row left.
        tracing::warn!("could not prune the extension state: {e}");
    }
    extensions
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extension(name: &str, description: &str, capabilities: &str) -> Extension {
        Extension {
            name: name.into(),
            description: description.into(),
            capabilities: capabilities.into(),
            haystack: format!("{name} {description} {capabilities}").to_lowercase(),
            ..Default::default()
        }
    }

    /// The filter searches more than the name — people look for what an
    /// extension does at least as often as for what it is called.
    #[test]
    fn the_filter_searches_every_field() {
        let lyrics = extension("Lyrics Provider", "Fetches lyrics", "metadata");
        assert!(lyrics.matches("lyric"));
        assert!(lyrics.matches("fetches"));
        assert!(lyrics.matches("metadata"));
        assert!(!lyrics.matches("radio"));
    }

    /// An empty search shows everything rather than nothing.
    #[test]
    fn an_empty_filter_matches_everything() {
        assert!(extension("Anything", "", "events").matches(""));
    }

    /// The haystack is lowercased at scan time, so a capitalised search still
    /// has to hit — the caller lowercases the needle.
    #[test]
    fn the_search_is_case_insensitive() {
        let extension = extension("Radio Browser", "Stations", "source");
        assert!(extension.matches("radio"));
        assert!(extension.matches("stations"));
    }
}
