//! When the repo CAR archive is worth downloading again.
//!
//! The archive is the whole atproto repo in one request, and both repo
//! integrations ([`crate::atradio`] and [`crate::rocksky_likes`]) read from
//! it. Two things keep that to a single download:
//!
//! * within a run — [`crate::atproto::cached_repo_car`] hands the archive
//!   already in memory to the second reader;
//! * across runs — the timestamp of the last successful download is kept in
//!   `atproto_repo_sync`, and a repo younger than [`max_age`] is not pulled
//!   again. What the last import wrote is already in the local tables, and
//!   Jetstream has been applying changes since.
//!
//! A download can always be forced, for when the local copy is suspect:
//!
//! * `atproto_force_car_sync = true` in settings.toml, or
//! * `music-player --force-car-sync` (this process only).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Error;
use chrono::{DateTime, Duration, Utc};
use music_player_entity::atproto_repo_sync;
use music_player_settings::read_settings;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};

use crate::atproto;

/// How old the last download may be before the repo is pulled again, unless
/// `atproto_car_max_age_hours` says otherwise.
const DEFAULT_MAX_AGE_HOURS: i64 = 24;

/// Set by `--force-car-sync` for the lifetime of the process.
static FORCED: AtomicBool = AtomicBool::new(false);

/// Force the next download regardless of when the last one happened — the
/// `--force-car-sync` flag.
pub fn force_download() {
    FORCED.store(true, Ordering::Relaxed);
}

/// Whether a download is being forced, by the flag or by settings.toml.
pub fn forced() -> bool {
    FORCED.load(Ordering::Relaxed) || setting_bool("atproto_force_car_sync")
}

fn setting_bool(key: &str) -> bool {
    read_settings()
        .ok()
        .and_then(|config| config.get_bool(key).ok())
        .unwrap_or(false)
}

/// How long a download stays good for, from `atproto_car_max_age_hours`.
/// `0` means "always download".
fn max_age() -> Duration {
    let hours = read_settings()
        .ok()
        .and_then(|config| config.get_int("atproto_car_max_age_hours").ok())
        .unwrap_or(DEFAULT_MAX_AGE_HOURS);
    Duration::hours(hours.max(0))
}

/// When `did`'s repo was last downloaded in full, as recorded locally.
pub async fn last_download(
    conn: &DatabaseConnection,
    did: &str,
) -> Result<Option<DateTime<Utc>>, Error> {
    Ok(atproto_repo_sync::Entity::find_by_id(did.to_owned())
        .one(conn)
        .await?
        .and_then(|row| DateTime::parse_from_rfc3339(&row.last_downloaded_at).ok())
        .map(|at| at.with_timezone(&Utc)))
}

async fn record_download(conn: &DatabaseConnection, did: &str, bytes: usize) -> Result<(), Error> {
    let model = atproto_repo_sync::ActiveModel {
        did: ActiveValue::Set(did.to_owned()),
        last_downloaded_at: ActiveValue::Set(Utc::now().to_rfc3339()),
        bytes: ActiveValue::Set(bytes as i64),
    };
    // An explicit upsert, not `save()`: the DID is a primary key we assign, so
    // sea-orm always sees it Set and issues an UPDATE — which matches nothing
    // on the first run and fails with RecordNotFound.
    atproto_repo_sync::Entity::insert(model)
        .on_conflict(
            OnConflict::column(atproto_repo_sync::Column::Did)
                .update_columns([
                    atproto_repo_sync::Column::LastDownloadedAt,
                    atproto_repo_sync::Column::Bytes,
                ])
                .to_owned(),
        )
        .exec(conn)
        .await?;
    Ok(())
}

/// `did`'s repo as a CAR archive, or `None` when the last download is still
/// recent enough that pulling the whole repo again would buy nothing.
///
/// `None` means "keep what is already in the local tables" — it is not an
/// error, and callers should skip their import rather than fall back to
/// `listRecords`.
pub async fn repo_car(conn: &DatabaseConnection, did: &str) -> Result<Option<Arc<Vec<u8>>>, Error> {
    // Already downloaded in this run: the other integration got there first.
    if let Some(car) = atproto::cached_repo_car(did).await {
        return Ok(Some(car));
    }

    if forced() {
        tracing::info!(%did, "forcing a repo CAR download");
    } else if let Some(at) = last_download(conn, did).await? {
        let age = Utc::now().signed_duration_since(at);
        if age < max_age() {
            tracing::info!(
                %did,
                last_downloaded_at = %at.to_rfc3339(),
                age_hours = age.num_hours(),
                "skipping the repo CAR download: the local copy is still current \
                 (force with --force-car-sync or atproto_force_car_sync = true)"
            );
            return Ok(None);
        }
    }

    let car = atproto::get_repo_car(did).await?;
    if let Err(e) = record_download(conn, did, car.len()).await {
        // The archive is in hand; failing to remember it only costs a
        // download next time.
        tracing::warn!(%did, "could not record the repo CAR download: {e}");
    }
    Ok(Some(car))
}
