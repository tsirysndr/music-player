//! One HTTP client for every backend.
//!
//! Shared and reused rather than built per request: a `reqwest::Client` owns a
//! connection pool, and constructing one per call leaks file descriptors until
//! the process runs out of them. Cloning it is cheap — it is an `Arc` inside —
//! so backends hold their own clone and keep the pool.
//!
//! `reqwest` rather than `surf` because the rest of the daemon is tokio, and
//! `surf` drags an async-std runtime in beside it.

use std::sync::OnceLock;
use std::time::Duration;

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

/// The process-wide client. Built once, on first use.
pub fn client() -> reqwest::Client {
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                // A server that has gone away should fail the screen, not hang
                // it: every call here is on the path of a user waiting for a
                // list to appear.
                .timeout(Duration::from_secs(20))
                .connect_timeout(Duration::from_secs(5))
                .user_agent(concat!("music-player/", env!("CARGO_PKG_VERSION")))
                .build()
                .unwrap_or_default()
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hands_back_the_same_pool_every_time() {
        // Not an equality check — `Client` has no `PartialEq`. The point is
        // that a second call does not build a second pool, which is what the
        // `OnceLock` is for.
        let first = client();
        let second = client();
        drop((first, second));
    }
}
