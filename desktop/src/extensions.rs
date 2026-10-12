//! Extension data comes from the selected daemon, including embedded mode.
//! A catalog query does not load WASM; mutations invalidate its shared manager.

use anyhow::Error;
use music_player_server::api::music::v1alpha1::{
    extensions_service_client::ExtensionsServiceClient, InstalledExtension, ListExtensionsRequest,
    RescanExtensionsRequest, SetExtensionEnabledRequest,
};
use tonic::transport::Channel;

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

fn client(channel: Channel) -> ExtensionsServiceClient<Channel> {
    ExtensionsServiceClient::new(channel)
        .max_decoding_message_size(music_player_server::LIBRARY_MESSAGE_LIMIT)
        .max_encoding_message_size(music_player_server::LIBRARY_MESSAGE_LIMIT)
}

fn rows(installed: Vec<InstalledExtension>) -> Vec<Extension> {
    installed
        .into_iter()
        .map(|manifest| {
            let capabilities = manifest.capabilities.join(", ");
            let hosts = manifest.allowed_hosts.join(", ");
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
                library_read: manifest.library_read,
                enabled: manifest.enabled,
                error: String::new(),
                haystack,
            }
        })
        .collect()
}

pub async fn load(channel: Channel) -> Result<Vec<Extension>, Error> {
    Ok(rows(
        client(channel)
            .list_extensions(ListExtensionsRequest {})
            .await?
            .into_inner()
            .extensions,
    ))
}
pub async fn set_enabled(
    channel: Channel,
    id: &str,
    enabled: bool,
) -> Result<Vec<Extension>, Error> {
    let mut client = client(channel);
    // Send once: a lost reply does not prove the mutation failed.
    client
        .set_extension_enabled(SetExtensionEnabledRequest {
            id: id.into(),
            enabled,
        })
        .await?;
    Ok(rows(
        client
            .list_extensions(ListExtensionsRequest {})
            .await?
            .into_inner()
            .extensions,
    ))
}
pub async fn rescan(channel: Channel) -> Result<Vec<Extension>, Error> {
    Ok(rows(
        client(channel)
            .rescan_extensions(RescanExtensionsRequest {})
            .await?
            .into_inner()
            .extensions,
    ))
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
