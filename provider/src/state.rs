//! Which source the library screens are currently reading from.
//!
//! # The invariant
//!
//! **A provider is not a sink.** This type holds no player command channel, no
//! tracklist and no receiver, and [`MusicProvider`] has no `play`, `seek` or
//! `load_tracks`. Nothing reachable from [`ProviderState::connect`] or
//! [`ProviderState::disconnect`] can therefore interrupt playback — switching
//! servers changes where the *screens* read from and nothing else.
//!
//! That holds for what is already queued, too: a queued track carries an
//! absolute uri (an authenticated Subsonic stream url, or
//! `http://peer:5053/tracks/<id>` for a music-player peer), so it keeps
//! playing after the source it came from has been swapped out. Nothing in the
//! queue points back at the source object.

use crate::{MusicProvider, ProviderConfig, ProviderError, ProviderRegistry};
use std::sync::Arc;
use tokio::sync::RwLock;

/// A live connection and the config it was built from.
#[derive(Clone)]
pub struct ConnectedProvider {
    pub config: ProviderConfig,
    pub provider: Arc<dyn MusicProvider>,
}

// Hand-written: `dyn MusicProvider` is not `Debug`, and the config is the part
// worth seeing in a panic message anyway.
impl std::fmt::Debug for ConnectedProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectedProvider")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

pub struct ProviderState {
    registry: Arc<ProviderRegistry>,
    current: RwLock<Option<ConnectedProvider>>,
    /// Host/port pairs that are this daemon itself. Connecting to one would
    /// make every library read call back into us and recurse.
    own_addresses: RwLock<Vec<(String, u16)>>,
}

impl ProviderState {
    pub fn new(registry: Arc<ProviderRegistry>) -> Self {
        Self {
            registry,
            current: RwLock::new(None),
            own_addresses: RwLock::new(Vec::new()),
        }
    }

    pub fn registry(&self) -> &ProviderRegistry {
        &self.registry
    }

    /// Register an address this daemon answers on, so it refuses to use itself
    /// as a source. Called once at startup for each bound port.
    pub async fn add_own_address(&self, host: impl Into<String>, port: u16) {
        let host = host.into();
        let mut own = self.own_addresses.write().await;
        if !own.iter().any(|(h, p)| *h == host && *p == port) {
            own.push((host, port));
        }
    }

    /// `None` means the local library. Cheap enough to call per resolver: a
    /// read lock, two `Arc` clones, released before any request goes out.
    pub async fn current(&self) -> Option<ConnectedProvider> {
        self.current.read().await.clone()
    }

    pub async fn config(&self) -> Option<ProviderConfig> {
        self.current.read().await.as_ref().map(|c| c.config.clone())
    }

    pub async fn is_connected(&self) -> bool {
        self.current.read().await.is_some()
    }

    /// Connect, then swap.
    ///
    /// The order matters: the new source is built and proven *before* the old
    /// one is replaced, so a server that is down or misconfigured leaves the
    /// previous one live and whatever is playing untouched. The write lock is
    /// held only for the pointer swap, never across a network request, so
    /// switching servers cannot block a read that is already in flight.
    pub async fn connect(
        &self,
        config: ProviderConfig,
    ) -> Result<ConnectedProvider, ProviderError> {
        self.reject_self(&config).await?;

        let source = self.registry.connect(&config).await?;
        source.ping().await?;

        let connected = ConnectedProvider {
            config,
            provider: source,
        };
        *self.current.write().await = Some(connected.clone());
        tracing::info!(
            kind = connected.config.kind,
            url = connected.config.url,
            "connected to provider"
        );
        Ok(connected)
    }

    /// Returns what was disconnected, or `None` if nothing was.
    pub async fn disconnect(&self) -> Option<ProviderConfig> {
        let previous = self.current.write().await.take();
        if let Some(previous) = &previous {
            tracing::info!(url = previous.config.url, "disconnected from provider");
        }
        previous.map(|connected| connected.config)
    }

    /// Refuse to point the daemon at itself.
    ///
    /// Once the gRPC library service reads through the source, a self-connect
    /// makes `get_albums` open a client back to this same daemon, which calls
    /// `get_albums`… until something runs out. Cheaper to refuse than to
    /// detect the recursion later.
    async fn reject_self(&self, config: &ProviderConfig) -> Result<(), ProviderError> {
        let (host, port) = config.host_port();
        let own = self.own_addresses.read().await;
        let loopback = matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1" | "[::1]");
        if own
            .iter()
            .any(|(h, p)| *p == port && (h == &host || (loopback && is_loopback(h))))
        {
            return Err(ProviderError::Other(
                "that is this server — pick a different one".into(),
            ));
        }
        Ok(())
    }
}

fn is_loopback(host: &str) -> bool {
    matches!(host, "localhost" | "127.0.0.1" | "::1" | "[::1]")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{registry::ProviderFactory, Album, Artist, Page, Track};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Stub(&'static str);

    #[async_trait::async_trait]
    impl MusicProvider for Stub {
        fn kind(&self) -> &'static str {
            "stub"
        }
        fn base_url(&self) -> &str {
            self.0
        }
        fn host(&self) -> &str {
            "stub.lan"
        }
        async fn albums(&self, _: Option<&str>, _: Page) -> Result<Vec<Album>, ProviderError> {
            Ok(vec![])
        }
        async fn artists(&self, _: Option<&str>, _: Page) -> Result<Vec<Artist>, ProviderError> {
            Ok(vec![])
        }
        async fn tracks(&self, _: Option<&str>, _: Page) -> Result<Vec<Track>, ProviderError> {
            Ok(vec![])
        }
        async fn album(&self, _: &str) -> Result<Album, ProviderError> {
            Err(ProviderError::NotFound("album".into()))
        }
        async fn artist(&self, _: &str) -> Result<Artist, ProviderError> {
            Err(ProviderError::NotFound("artist".into()))
        }
        async fn track(&self, _: &str) -> Result<Track, ProviderError> {
            Err(ProviderError::NotFound("track".into()))
        }
    }

    /// Fails every connect after the first `ok_for` calls.
    struct FlakyFactory {
        ok_for: usize,
        calls: AtomicUsize,
    }

    #[async_trait::async_trait]
    impl ProviderFactory for FlakyFactory {
        fn kind(&self) -> &'static str {
            "stub"
        }
        fn display_name(&self) -> &'static str {
            "Stub"
        }
        fn default_port(&self) -> u16 {
            80
        }
        async fn connect(
            &self,
            config: &ProviderConfig,
        ) -> Result<Arc<dyn MusicProvider>, ProviderError> {
            let n = self.calls.fetch_add(1, Ordering::SeqCst);
            if n >= self.ok_for {
                return Err(ProviderError::Transport("connection refused".into()));
            }
            // Leaked so the stub can hand back a `&'static str` base url; only
            // ever a handful per test.
            Ok(Arc::new(Stub(Box::leak(
                config.url.clone().into_boxed_str(),
            ))))
        }
    }

    fn state(ok_for: usize) -> ProviderState {
        let mut registry = ProviderRegistry::new();
        registry.register(FlakyFactory {
            ok_for,
            calls: AtomicUsize::new(0),
        });
        ProviderState::new(Arc::new(registry))
    }

    #[tokio::test]
    async fn starts_on_the_local_library() {
        assert!(state(1).current().await.is_none());
    }

    #[tokio::test]
    async fn connecting_makes_a_source_current() {
        let state = state(1);
        let config = ProviderConfig::new("stub", "One", "http://one.lan:80");
        state.connect(config.clone()).await.unwrap();

        let current = state.current().await.unwrap();
        assert_eq!(current.config.url, "http://one.lan:80");
        assert!(state.is_connected().await);
    }

    /// The point of build-then-swap: a server that is down must not take the
    /// working one with it.
    #[tokio::test]
    async fn a_failed_connect_leaves_the_previous_source_live() {
        let state = state(1);
        state
            .connect(ProviderConfig::new("stub", "Good", "http://good.lan:80"))
            .await
            .unwrap();

        let error = state
            .connect(ProviderConfig::new("stub", "Bad", "http://bad.lan:80"))
            .await
            .unwrap_err();
        assert!(error.to_string().contains("connection refused"));

        let current = state.current().await.expect("still connected");
        assert_eq!(current.config.name, "Good");
    }

    #[tokio::test]
    async fn disconnecting_returns_to_the_local_library() {
        let state = state(1);
        state
            .connect(ProviderConfig::new("stub", "One", "http://one.lan:80"))
            .await
            .unwrap();

        let previous = state.disconnect().await.expect("something was connected");
        assert_eq!(previous.name, "One");
        assert!(state.current().await.is_none());
        // Twice is a no-op, not an error.
        assert!(state.disconnect().await.is_none());
    }

    #[tokio::test]
    async fn refuses_to_connect_to_itself() {
        let state = state(9);
        state.add_own_address("192.168.1.10", 5053).await;

        let error = state
            .connect(ProviderConfig::new(
                "stub",
                "Me",
                "http://192.168.1.10:5053",
            ))
            .await
            .unwrap_err();
        assert!(error.to_string().contains("this server"));

        // A different port on the same host is a different server.
        assert!(state
            .connect(ProviderConfig::new(
                "stub",
                "Peer",
                "http://192.168.1.10:5055"
            ))
            .await
            .is_ok());
    }

    /// `localhost` and `127.0.0.1` name the same daemon as any loopback
    /// address we advertise on.
    #[tokio::test]
    async fn recognises_itself_through_loopback_aliases() {
        let state = state(9);
        state.add_own_address("127.0.0.1", 5053).await;
        assert!(state
            .connect(ProviderConfig::new("stub", "Me", "http://localhost:5053"))
            .await
            .is_err());
    }
}
