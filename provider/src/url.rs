//! Making a remote item's uris reachable from here.
//!
//! Backends return either an absolute, already-authenticated url (Subsonic and
//! Jellyfin sign their stream links) or something relative to the server
//! (`/tracks/<id>` on a music-player peer). The existing
//! [`RemoteTrackUrl`]/[`RemoteCoverUrl`] impls in `music-player-types` know the
//! difference and leave absolute uris alone; this is the one place that calls
//! them, so the gRPC and GraphQL paths cannot decorate differently.

use crate::ProviderConfig;
use music_player_types::types::{RemoteCoverUrl, RemoteTrackUrl};

/// Absolutise one item's track and cover uris against its source.
pub fn decorate<T>(item: T, config: &ProviderConfig) -> T
where
    T: RemoteTrackUrl + RemoteCoverUrl,
{
    item.with_remote_track_url(&config.url)
        .with_remote_cover_url(&config.url)
}

pub fn decorate_all<T>(items: Vec<T>, config: &ProviderConfig) -> Vec<T>
where
    T: RemoteTrackUrl + RemoteCoverUrl,
{
    items
        .into_iter()
        .map(|item| decorate(item, config))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use music_player_types::types::{Album, Track};

    fn config() -> ProviderConfig {
        ProviderConfig::new("music-player", "peer", "http://peer.lan:5053")
    }

    #[test]
    fn a_relative_uri_is_resolved_against_the_server() {
        let track = Track {
            id: "abc".into(),
            uri: "abc".into(),
            ..Default::default()
        };
        assert_eq!(
            decorate(track, &config()).uri,
            "http://peer.lan:5053/tracks/abc"
        );
    }

    /// Subsonic and Jellyfin sign their own stream urls; rewriting one would
    /// strip the token and make it unplayable.
    #[test]
    fn an_absolute_uri_is_left_alone() {
        let signed = "http://nas.lan:4533/rest/stream?id=7&t=abc&s=xyz";
        let track = Track {
            id: "7".into(),
            uri: signed.into(),
            ..Default::default()
        };
        assert_eq!(decorate(track, &config()).uri, signed);
    }

    #[test]
    fn covers_are_resolved_too() {
        let album = Album {
            id: "a1".into(),
            cover: Some("a1.jpg".into()),
            ..Default::default()
        };
        assert_eq!(
            decorate(album, &config()).cover.as_deref(),
            Some("http://peer.lan:5053/covers/a1.jpg")
        );
    }

    #[test]
    fn decorates_a_whole_listing() {
        let tracks = vec![
            Track {
                id: "1".into(),
                uri: "1".into(),
                ..Default::default()
            },
            Track {
                id: "2".into(),
                uri: "2".into(),
                ..Default::default()
            },
        ];
        let decorated = decorate_all(tracks, &config());
        assert_eq!(decorated[1].uri, "http://peer.lan:5053/tracks/2");
    }
}
