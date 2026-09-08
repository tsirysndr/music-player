//! Rocksky — the hosted Navidrome at `navidrome.rocksky.app`.
//!
//! It is a Subsonic server, so it reuses that backend wholesale; what it adds
//! is that there is nothing to configure but the login. The url is the
//! factory's, not the user's, which is what `fixed_url` tells the clients so
//! they can drop the field from the form.

use crate::{
    backends::subsonic::Subsonic, MusicProvider, ProviderConfig, ProviderError, ProviderFactory,
};
use async_trait::async_trait;
use std::sync::Arc;

/// The only address this provider talks to.
pub const ROCKSKY_URL: &str = "https://navidrome.rocksky.app";

pub struct RockskyFactory;

#[async_trait]
impl ProviderFactory for RockskyFactory {
    fn kind(&self) -> &'static str {
        "rocksky"
    }

    fn display_name(&self) -> &'static str {
        "Rocksky"
    }

    fn fixed_url(&self) -> Option<&'static str> {
        Some(ROCKSKY_URL)
    }

    /// Https, though nothing asks: the url is not the user's to give.
    fn default_port(&self) -> u16 {
        443
    }

    async fn connect(
        &self,
        config: &ProviderConfig,
    ) -> Result<Arc<dyn MusicProvider>, ProviderError> {
        // Whatever url the config carries is ignored — a saved row from an
        // older build, or a client that sent one anyway, must not point this
        // somewhere else.
        let mut client = Subsonic::with_credentials(
            ROCKSKY_URL,
            config.username.as_deref().unwrap_or_default(),
            config.password.as_deref().unwrap_or_default(),
        );
        client.connect().await?;
        Ok(Arc::new(client))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtin_registry;

    #[test]
    fn it_is_registered_and_fixed_to_one_url() {
        let registry = builtin_registry();
        let factory = registry.get("rocksky").expect("rocksky is registered");
        assert_eq!(factory.fixed_url(), Some(ROCKSKY_URL));
        // It is a login, not an address, that the user supplies.
        assert!(factory.needs_credentials());
    }

    /// Every other backend leaves the url to the user.
    #[test]
    fn the_other_kinds_have_no_fixed_url() {
        for info in builtin_registry().describe() {
            if info.kind != "rocksky" {
                assert_eq!(info.fixed_url, None, "{} should take a url", info.kind);
            }
        }
    }
}
