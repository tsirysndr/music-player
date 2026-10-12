//! Making a remote item's uris reachable from here.
//!
//! Backends return either an absolute, already-authenticated url (Subsonic and
//! Jellyfin sign their stream links) or something relative to the server
//! (`/tracks/<id>` on a music-player peer). The existing
//! [`RemoteTrackUrl`]/[`RemoteCoverUrl`] impls in `music-player-types` know the
//! difference and leave absolute uris and reserved SourceRef handles alone.
//! Source handles (including malformed ones) must reach source-aware API
//! validation, never be hidden inside a peer's /tracks or /covers URL.
//! This is the one place that calls
//! them, so the gRPC and GraphQL paths cannot decorate differently.

use crate::ProviderConfig;
use music_player_types::types::{RemoteCoverUrl, RemoteTrackUrl};

/// Absolutise legacy relative uris while preserving managed source identity.
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

    #[test]
    fn switching_browsing_account_does_not_rewrite_queued_source_identity() {
        use music_player_types::source::{RemoteIdentity, ResourceKind, SourceRef};
        use music_player_types::types::{Artist, Playlist};

        let make_track = |account: &str| {
            let handle = SourceRef {
                resolver: "emby".into(),
                account_id: account.into(),
                remote: RemoteIdentity {
                    server_id: "server".into(),
                    user_id: account.into(),
                },
                kind: ResourceKind::Item,
                item_id: "55508".into(),
            }
            .to_handle();
            Track {
                id: handle.clone(),
                uri: handle,
                album: None,
                ..Default::default()
            }
        };
        let a = make_track("account-a");
        let b = make_track("account-b");
        let playlist = Playlist {
            tracks: vec![a.clone(), b.clone()],
            ..Default::default()
        };
        let decorated = decorate(playlist, &config());
        assert_eq!(decorated.tracks[0].id, a.id);
        assert_eq!(decorated.tracks[0].uri, a.uri);
        assert_eq!(decorated.tracks[1].id, b.id);
        assert_eq!(decorated.tracks[1].uri, b.uri);
        assert_ne!(decorated.tracks[0].id, decorated.tracks[1].id);
        assert!(decorated.tracks.iter().all(|track| track.album.is_none()));

        let artist = Artist {
            albums: vec![Album {
                tracks: vec![a.clone()],
                cover: None,
                ..Default::default()
            }],
            songs: vec![b.clone()],
            ..Default::default()
        };
        let decorated = decorate(artist, &config());
        assert_eq!(decorated.albums[0].tracks[0].uri, a.uri);
        assert_eq!(decorated.albums[0].cover, None);
        assert_eq!(decorated.songs[0].uri, b.uri);
    }
}
