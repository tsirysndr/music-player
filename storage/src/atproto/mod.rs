//! Shared atproto plumbing, split by concern:
//!
//! * [`identity`] — who the user is (Rocksky token, environment credentials,
//!   handle resolution).
//! * [`repo`] — public repo reads: PDS discovery, `listRecords`, `getRecord`.
//! * [`car`] — repo CAR archives and MST traversal.
//! * [`jetstream`] — the de-duplicated multi-endpoint firehose subscription.
//!
//! Both repo integrations sit on top of this — [`crate::atradio`] for radio
//! bookmarks and [`crate::rocksky_likes`] for liked songs — so the CAR reader
//! and the firehose client exist once.

use std::time::Duration;

use anyhow::Error;

pub mod car;
pub mod identity;
pub mod jetstream;
pub mod repo;

pub use car::{cached_repo_car, car_records, get_repo_car, records_from_car};
pub use identity::{env_credentials, resolve_did, resolve_handle, token_did};
pub use jetstream::{subscribe, JetstreamCommit, JetstreamEvent};
pub use repo::{get_record, list_records, pds_endpoint, split_at_uri};

pub(crate) fn http() -> Result<reqwest::Client, Error> {
    Ok(reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?)
}
