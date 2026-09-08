//! The servers this build knows how to read a library from.
//!
//! # Adding one
//!
//! 1. A new file here implementing [`MusicProvider`](crate::MusicProvider) and
//!    [`ProviderFactory`](crate::ProviderFactory).
//! 2. One `.register(...)` line in [`register_builtin`].
//!
//! That is the whole contract. The clients build their add-server form from
//! the registry, so a new backend turns up in the Slint desktop and the web UI
//! without either of them being told about it.
//!
//! Every backend here talks HTTP and owns nothing but its own client, which is
//! what keeps this crate below the server crate in the dependency graph — the
//! gRPC layer needs to route reads through a source, and it could not if a
//! source needed the gRPC client.

pub mod jellyfin;
pub mod kodi;
pub mod music_player;
pub mod plex;
pub mod rocksky;
pub mod subsonic;

use crate::ProviderRegistry;

/// Every backend compiled into this build.
///
/// Called once at startup by whichever binary owns the daemon; the resulting
/// registry is shared by the gRPC server and the GraphQL schema, so the two
/// cannot disagree about what a server is.
pub fn register_builtin(registry: &mut ProviderRegistry) {
    registry
        .register(subsonic::SubsonicFactory)
        .register(jellyfin::JellyfinFactory)
        .register(music_player::MusicPlayerFactory)
        .register(kodi::KodiFactory)
        .register(plex::PlexFactory)
        .register(rocksky::RockskyFactory);
}

/// A registry with the built-ins already in it.
pub fn builtin_registry() -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();
    register_builtin(&mut registry);
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_builtin_is_reachable_by_kind() {
        let registry = builtin_registry();
        for kind in [
            "subsonic",
            "jellyfin",
            "music-player",
            "kodi",
            "plex",
            "rocksky",
        ] {
            assert!(registry.get(kind).is_some(), "{kind} is not registered");
        }
    }

    #[test]
    fn a_peer_daemon_needs_no_credentials() {
        let registry = builtin_registry();
        assert!(!registry.get("music-player").unwrap().needs_credentials());
        assert!(registry.get("subsonic").unwrap().needs_credentials());
    }

    /// Kodi still calls itself xbmc over mDNS, and that string already flows
    /// through discovery.
    #[test]
    fn xbmc_is_kodi() {
        assert_eq!(
            builtin_registry().get("xbmc").map(|f| f.kind()),
            Some("kodi")
        );
    }

    /// Navidrome speaks the Subsonic API under its own name.
    #[test]
    fn navidrome_is_a_subsonic_server() {
        assert_eq!(
            builtin_registry().get("navidrome").map(|f| f.kind()),
            Some("subsonic")
        );
    }
}
