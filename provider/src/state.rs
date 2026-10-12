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
use music_player_types::source::RemoteIdentity;
use std::future::Future;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
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
    connect_generation: AtomicU64,
    /// Host/port pairs that are this daemon itself. Connecting to one would
    /// make every library read call back into us and recurse.
    own_addresses: RwLock<Vec<(String, u16)>>,
}

impl ProviderState {
    pub fn new(registry: Arc<ProviderRegistry>) -> Self {
        Self {
            registry,
            current: RwLock::new(None),
            connect_generation: AtomicU64::new(0),
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
        self.connect_checked(config, |identity| async move {
            if identity.is_some() {
                return Err(ProviderError::Other(
                    "this source requires saved-account identity verification".into(),
                ));
            }
            Ok(())
        })
        .await
    }

    /// Authenticate and verify the saved account before replacing the current
    /// source. The verifier owns the storage snapshot captured BEFORE this call.
    /// An error or cancellation leaves the previous connection available.
    pub async fn connect_checked<F, V>(
        &self,
        config: ProviderConfig,
        verify: V,
    ) -> Result<ConnectedProvider, ProviderError>
    where
        V: FnOnce(Option<RemoteIdentity>) -> F + Send,
        F: Future<Output = Result<(), ProviderError>> + Send,
    {
        let generation = self
            .connect_generation
            .fetch_add(1, Ordering::SeqCst)
            .wrapping_add(1);
        self.reject_self(&config).await?;

        let source = self.registry.connect(&config).await?;
        source.ping().await?;
        verify(source.remote_identity()).await?;

        let connected = ConnectedProvider {
            config,
            provider: source,
        };
        let mut current = self.current.write().await;
        if self.connect_generation.load(Ordering::SeqCst) != generation {
            return Err(ProviderError::Other(
                "connection request was superseded".into(),
            ));
        }
        *current = Some(connected.clone());
        drop(current);
        tracing::info!(
            kind = connected.config.kind,
            account_id = connected.config.id,
            "connected to provider"
        );
        Ok(connected)
    }

    /// Returns what was disconnected, or `None` if nothing was or a newer
    /// request superseded this disconnect while it waited for the write lock.
    pub async fn disconnect(&self) -> Option<ProviderConfig> {
        let generation = self
            .connect_generation
            .fetch_add(1, Ordering::SeqCst)
            .wrapping_add(1);
        let mut current = self.current.write().await;
        if self.connect_generation.load(Ordering::SeqCst) != generation {
            return None;
        }
        let previous = current.take();
        drop(current);
        if let Some(previous) = &previous {
            tracing::info!(
                account_id = previous.config.id,
                "disconnected from provider"
            );
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

    struct Stub(&'static str, Option<RemoteIdentity>);

    #[async_trait::async_trait]
    impl MusicProvider for Stub {
        fn kind(&self) -> &'static str {
            "stub"
        }
        fn remote_identity(&self) -> Option<RemoteIdentity> {
            self.1.clone()
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
        identity: Option<RemoteIdentity>,
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
            Ok(Arc::new(Stub(
                Box::leak(config.url.clone().into_boxed_str()),
                self.identity.clone(),
            )))
        }
    }

    fn state(ok_for: usize) -> ProviderState {
        identified_state(ok_for, None)
    }

    fn identified_state(ok_for: usize, identity: Option<RemoteIdentity>) -> ProviderState {
        let mut registry = ProviderRegistry::new();
        registry.register(FlakyFactory {
            ok_for,
            calls: AtomicUsize::new(0),
            identity,
        });
        ProviderState::new(Arc::new(registry))
    }

    #[tokio::test]
    async fn starts_on_the_local_library() {
        assert!(state(1).current().await.is_none());
    }

    #[tokio::test]
    async fn actual_identity_must_be_verified_before_it_becomes_current() {
        let identity = RemoteIdentity {
            server_id: "server-a".into(),
            user_id: "family".into(),
        };
        let state = identified_state(4, Some(identity.clone()));
        let config = ProviderConfig::new("stub", "Verified", "http://one.lan");
        assert!(state.connect(config.clone()).await.is_err());
        assert!(state.current().await.is_none());
        state
            .connect_checked(config, move |actual| async move {
                assert_eq!(actual, Some(identity));
                Ok(())
            })
            .await
            .unwrap();
        let rejected = state
            .connect_checked(
                ProviderConfig::new("stub", "Mismatch", "http://two.lan"),
                |_| async { Err(ProviderError::Other("identity mismatch".into())) },
            )
            .await;
        assert!(rejected.is_err());
        assert_eq!(state.current().await.unwrap().config.name, "Verified");
    }

    #[tokio::test]
    async fn cancelled_verification_does_not_publish_or_hold_the_current_lock() {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let state = Arc::new(state(2));
            state
                .connect(ProviderConfig::new("stub", "Original", "http://one.lan"))
                .await
                .unwrap();
            let entered = Arc::new(tokio::sync::Notify::new());
            let worker = {
                let state = state.clone();
                let entered = entered.clone();
                tokio::spawn(async move {
                    state
                        .connect_checked(
                            ProviderConfig::new("stub", "Pending", "http://two.lan"),
                            |_| async move {
                                entered.notify_one();
                                std::future::pending::<Result<(), ProviderError>>().await
                            },
                        )
                        .await
                })
            };
            entered.notified().await;
            assert_eq!(state.current().await.unwrap().config.name, "Original");
            worker.abort();
            assert!(worker.await.unwrap_err().is_cancelled());
            assert_eq!(state.current().await.unwrap().config.name, "Original");
        })
        .await
        .expect("cancelled-verification fixture timed out");
    }

    #[tokio::test]
    async fn disconnect_waiting_for_lock_preserves_a_later_connect_intent() {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let state = Arc::new(state(2));
            state
                .connect(ProviderConfig::new("stub", "Original", "http://one.lan"))
                .await
                .unwrap();

            // Poll the real disconnect with the write lock held: it issues its
            // ticket and must stop at lock acquisition, without a timing race.
            let locked = state.current.write().await;
            let before = state.connect_generation.load(Ordering::SeqCst);
            let disconnect = state.disconnect();
            tokio::pin!(disconnect);
            std::future::poll_fn(|cx| {
                assert!(disconnect.as_mut().poll(cx).is_pending());
                std::task::Poll::Ready(())
            })
            .await;
            assert_eq!(
                state.connect_generation.load(Ordering::SeqCst),
                before.wrapping_add(1)
            );

            let entered = Arc::new(tokio::sync::Notify::new());
            let release = Arc::new(tokio::sync::Notify::new());
            let latest = {
                let state = state.clone();
                let entered = entered.clone();
                let release = release.clone();
                tokio::spawn(async move {
                    state
                        .connect_checked(
                            ProviderConfig::new("stub", "Latest", "http://latest.lan"),
                            |_| async move {
                                entered.notify_one();
                                release.notified().await;
                                Ok(())
                            },
                        )
                        .await
                })
            };
            entered.notified().await;
            drop(locked);

            // The newer connect is still verifying. The stale disconnect must
            // leave the original available until the latest request can swap it.
            assert!(disconnect.await.is_none());
            assert_eq!(state.current().await.unwrap().config.name, "Original");
            release.notify_one();
            assert_eq!(latest.await.unwrap().unwrap().config.name, "Latest");
            assert_eq!(state.current().await.unwrap().config.name, "Latest");
        })
        .await
        .expect("disconnect-order fixture timed out");
    }

    #[tokio::test]
    async fn late_verification_cannot_override_a_later_connect_or_disconnect() {
        for disconnect in [false, true] {
            tokio::time::timeout(std::time::Duration::from_secs(5), async {
                let state = Arc::new(state(3));
                state
                    .connect(ProviderConfig::new("stub", "Original", "http://one.lan"))
                    .await
                    .unwrap();
                let entered = Arc::new(tokio::sync::Notify::new());
                let release = Arc::new(tokio::sync::Notify::new());
                let pending = {
                    let state = state.clone();
                    let entered = entered.clone();
                    let release = release.clone();
                    tokio::spawn(async move {
                        state
                            .connect_checked(
                                ProviderConfig::new("stub", "Late", "http://late.lan"),
                                |_| async move {
                                    entered.notify_one();
                                    release.notified().await;
                                    Ok(())
                                },
                            )
                            .await
                    })
                };
                entered.notified().await;
                if disconnect {
                    state.disconnect().await;
                } else {
                    state
                        .connect(ProviderConfig::new("stub", "Latest", "http://latest.lan"))
                        .await
                        .unwrap();
                }
                release.notify_one();
                assert!(
                    matches!(pending.await.unwrap(), Err(ProviderError::Other(message))
                    if message == "connection request was superseded")
                );
                if disconnect {
                    assert!(state.current().await.is_none());
                } else {
                    assert_eq!(state.current().await.unwrap().config.name, "Latest");
                }
            })
            .await
            .expect("connection-order fixture timed out");
        }
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
