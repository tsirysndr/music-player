//! The list of backends, and how a new one gets added.
//!
//! **Adding a kind of server is one file and one line.** Write a
//! [`MusicProvider`] plus a [`ProviderFactory`] in `addons/src/sources/`, then add
//! one `.register(...)` to `register_builtin`. Nothing else in the codebase
//! needs to know it exists: the clients build their add-server form from
//! [`ProviderRegistry::describe`], so a new backend shows up in both UIs on its
//! own.

use crate::{MusicProvider, ProviderConfig, ProviderError};
use std::sync::Arc;

/// What a client needs to render one entry of the "add a server" form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderKindInfo {
    pub kind: &'static str,
    pub display_name: &'static str,
    pub needs_credentials: bool,
    pub default_port: u16,
    /// A backend that always talks to one address. Clients hide the url field
    /// for these rather than asking for something that would be ignored.
    pub fixed_url: Option<&'static str>,
}

/// How to build one kind of source.
#[async_trait::async_trait]
pub trait ProviderFactory: Send + Sync + 'static {
    /// Stored verbatim in the saved-server row, and matched against a
    /// device's `app`.
    fn kind(&self) -> &'static str;

    /// The one address this backend talks to, if it only talks to one.
    ///
    /// A hosted service has no url to ask for; saving one uses this instead of
    /// whatever the form sent, so the field can be hidden rather than being a
    /// thing to get wrong.
    fn fixed_url(&self) -> Option<&'static str> {
        None
    }

    /// Other `app` values that mean this backend. Kodi advertises itself as
    /// `xbmc`, which is the string already flowing through discovery.
    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    fn display_name(&self) -> &'static str;

    /// False for backends with no login, so the form can hide the fields
    /// rather than asking for something that will be ignored.
    fn needs_credentials(&self) -> bool {
        true
    }

    fn default_port(&self) -> u16;

    /// Open a connection and prove it works. Returning `Ok` means the server
    /// answered — [`crate::ProviderState::connect`] will make it current only
    /// after this succeeds.
    async fn connect(
        &self,
        config: &ProviderConfig,
    ) -> Result<Arc<dyn MusicProvider>, ProviderError>;
}

#[derive(Default)]
pub struct ProviderRegistry {
    factories: Vec<Arc<dyn ProviderFactory>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Chainable, so `register_builtin` reads as a list.
    pub fn register<F: ProviderFactory>(&mut self, factory: F) -> &mut Self {
        self.factories.push(Arc::new(factory));
        self
    }

    /// The factory for a kind, by its own name or any of its aliases.
    pub fn get(&self, kind: &str) -> Option<Arc<dyn ProviderFactory>> {
        self.factories
            .iter()
            .find(|factory| factory.kind() == kind || factory.aliases().contains(&kind))
            .cloned()
    }

    pub fn kinds(&self) -> Vec<&'static str> {
        self.factories.iter().map(|f| f.kind()).collect()
    }

    /// What the clients render their add-server form from.
    pub fn describe(&self) -> Vec<ProviderKindInfo> {
        self.factories
            .iter()
            .map(|factory| ProviderKindInfo {
                kind: factory.kind(),
                display_name: factory.display_name(),
                needs_credentials: factory.needs_credentials(),
                default_port: factory.default_port(),
                fixed_url: factory.fixed_url(),
            })
            .collect()
    }

    /// Connect using whichever factory claims the config's kind.
    ///
    /// An unknown kind is an error rather than a fallback. The old code fell
    /// through to a gRPC client for anything unrecognised, which quietly
    /// opened a music-player connection to DLNA renderers.
    pub async fn connect(
        &self,
        config: &ProviderConfig,
    ) -> Result<Arc<dyn MusicProvider>, ProviderError> {
        let factory = self.get(&config.kind).ok_or_else(|| {
            ProviderError::Other(format!("no provider backend for '{}'", config.kind))
        })?;
        if config.url.is_empty() {
            return Err(ProviderError::Other(
                "a server needs a url before it can be connected to".into(),
            ));
        }
        factory.connect(config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Album, Artist, Page, Track};

    struct Stub;

    #[async_trait::async_trait]
    impl MusicProvider for Stub {
        fn kind(&self) -> &'static str {
            "kodi"
        }
        fn base_url(&self) -> &str {
            "http://kodi.lan:8080"
        }
        fn host(&self) -> &str {
            "kodi.lan"
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

    struct StubFactory;

    #[async_trait::async_trait]
    impl ProviderFactory for StubFactory {
        fn kind(&self) -> &'static str {
            "kodi"
        }
        fn aliases(&self) -> &'static [&'static str] {
            &["xbmc"]
        }
        fn display_name(&self) -> &'static str {
            "Kodi"
        }
        fn default_port(&self) -> u16 {
            8080
        }
        async fn connect(
            &self,
            _: &ProviderConfig,
        ) -> Result<Arc<dyn MusicProvider>, ProviderError> {
            Ok(Arc::new(Stub))
        }
    }

    fn registry() -> ProviderRegistry {
        let mut registry = ProviderRegistry::new();
        registry.register(StubFactory);
        registry
    }

    #[test]
    fn finds_a_backend_by_its_alias() {
        let registry = registry();
        assert!(registry.get("kodi").is_some());
        // Discovery reports Kodi as `xbmc`; both have to land on one backend.
        assert!(registry.get("xbmc").is_some());
        assert!(registry.get("plex").is_none());
    }

    #[test]
    fn describes_itself_for_the_add_server_form() {
        let described = registry().describe();
        assert_eq!(described.len(), 1);
        assert_eq!(described[0].display_name, "Kodi");
        assert_eq!(described[0].default_port, 8080);
    }

    #[tokio::test]
    async fn an_unknown_kind_is_an_error_not_a_fallback() {
        let registry = registry();
        let config = ProviderConfig::new("plex", "Plex", "http://plex.lan:32400");
        // `Arc<dyn MusicProvider>` is not `Debug`, so drop the Ok value first.
        let error = registry.connect(&config).await.map(|_| ()).unwrap_err();
        assert!(error.to_string().contains("no provider backend"));
    }

    #[tokio::test]
    async fn refuses_a_config_with_no_url() {
        let registry = registry();
        let mut config = ProviderConfig::new("kodi", "Kodi", "http://kodi.lan:8080");
        config.url = String::new();
        assert!(registry.connect(&config).await.is_err());
    }
}
