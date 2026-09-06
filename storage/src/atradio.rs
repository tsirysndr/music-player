//! atradio.fm favorite ⇄ local radio-bookmark sync.
//!
//! Bookmarks in the `saved_radio` table mirror `fm.atradio.favorite` records in
//! the signed-in user's atproto repo:
//!
//! * **import** — favorites already in the repo are pulled into sqlite on
//!   startup, from the repo's CAR archive (falling back to `listRecords`).
//! * **write-through** — bookmarking a station also writes the record to the
//!   user's PDS, and unbookmarking deletes it. This half needs a session:
//!   either the one the `atradio` CLI writes (`atradio login`), or password
//!   credentials in `ATPROTO_IDENTIFIER` / `ATPROTO_APP_PASSWORD`, which are
//!   signed in on demand.
//! * **status** — while a station plays, it is published as the account's
//!   `fm.atradio.actor.status` record, and deleted when playback stops.
//! * **live sync** — a Jetstream subscription applies favorites added or
//!   removed on atradio.fm or another device while the daemon runs.
//!
//! None of this is required to use music-player: with no Rocksky token and no
//! atproto credentials, [`resolve_did`] returns `None` and every entry point
//! here returns without touching the network.

use std::path::PathBuf;
use std::sync::OnceLock;

use anyhow::{anyhow, Error};
use atradio_sdk::{AtradioAgent, StationInfo};
use music_player_entity::saved_radio;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};
use serde::Deserialize;

use crate::{atproto, repo_sync};

const COLLECTION: &str = "fm.atradio.favorite";

// ── Identity ────────────────────────────────────────────────────────────────

/// The session file shared with the `atradio` CLI, so logging in there is
/// enough to enable write-through here.
fn session_path() -> PathBuf {
    directories::ProjectDirs::from("fm", "atradio", "atradio")
        .map(|dirs| dirs.config_dir().join("session.json"))
        .unwrap_or_else(|| PathBuf::from(".atradio-session.json"))
}

fn agent() -> &'static AtradioAgent {
    static AGENT: OnceLock<AtradioAgent> = OnceLock::new();
    AGENT.get_or_init(|| AtradioAgent::new(session_path()))
}

/// True when a session file exists or password credentials are in the
/// environment — i.e. write-through is configured. It does not prove the
/// credentials work; [`ensure_session`] is what actually establishes a session.
pub fn is_configured() -> bool {
    agent().is_logged_in() || atproto::env_credentials().is_some()
}

/// Resolve a usable session, signing in with the environment credentials when
/// no stored session exists yet. That keeps a headless daemon working without
/// an interactive `atradio login`. Returns whether writes can reach the PDS.
///
/// The session the SDK writes on success is the one the `atradio` CLI reads, so
/// this signs in once and later runs resume it.
pub async fn ensure_session() -> bool {
    if agent().is_logged_in() {
        return true;
    }
    let Some((identifier, password)) = atproto::env_credentials() else {
        return false;
    };
    match agent().login_password(&identifier, &password).await {
        Ok(profile) => {
            tracing::info!(
                user = %profile.handle,
                "signed in to atradio.fm with the environment credentials"
            );
            true
        }
        Err(e) => {
            tracing::warn!("atradio sign-in with the environment credentials failed: {e}");
            false
        }
    }
}

/// Whose repo to sync with, resolved in order of confidence:
///
/// 1. the `rocksky login` token,
/// 2. an atradio session already on disk,
/// 3. the password-auth identifier in the environment — used as-is when it is
///    already a DID, otherwise resolved by signing in (which also covers an
///    email identifier) or by resolving the handle.
///
/// `None` means nothing identifies a user, and the whole integration stays off.
pub async fn resolve_did() -> Option<String> {
    if let Some(did) = atproto::token_did() {
        tracing::info!(%did, "atproto identity resolved from the Rocksky token");
        return Some(did);
    }
    if let Some(profile) = agent().profile() {
        tracing::info!(did = %profile.did, "atproto identity resolved from the stored session");
        return Some(profile.did);
    }
    let (identifier, _) = atproto::env_credentials()?;
    if identifier.starts_with("did:") {
        tracing::info!(did = %identifier, "atproto identity taken from the environment");
        return Some(identifier);
    }
    if ensure_session().await {
        if let Some(profile) = agent().profile() {
            tracing::info!(
                did = %profile.did,
                "atproto identity resolved by signing in as {identifier}"
            );
            return Some(profile.did);
        }
    }
    let did = atproto::resolve_handle(&identifier).await?;
    tracing::info!(%did, "atproto identity resolved from the handle {identifier}");
    Some(did)
}

/// Log, once at startup, whether the account is signed in to atradio — and if
/// not, exactly what is missing. Silent failures here are the confusing kind:
/// bookmarks would keep working locally while nothing reached the PDS.
async fn announce_session(did: &str) {
    if ensure_session().await {
        match agent().profile() {
            Some(profile) => tracing::info!(
                did = %profile.did,
                method = %profile.method,
                "atradio.fm: authenticated as {} — bookmark writes and listening status are on",
                profile.label()
            ),
            None => tracing::info!(
                %did,
                "atradio.fm: authenticated — bookmark writes and listening status are on"
            ),
        }
    } else {
        tracing::info!(
            %did,
            "atradio.fm: not authenticated — favorites are imported read-only, and bookmark \
             writes and listening status are off. Run `atradio login`, or set \
             ATPROTO_IDENTIFIER + ATPROTO_APP_PASSWORD (session file: {})",
            session_path().display()
        );
    }
}

// ── Record ⇄ row mapping ────────────────────────────────────────────────────

/// atradio names its providers in lexicon form; the local rows carry the label
/// the UI shows. The station id prefix (`rb:` / `tunein:`) is identical on both
/// sides, so it stays the join key.
fn source_label(source: &str) -> String {
    match source {
        "radio-browser" => "Radio Browser".to_owned(),
        "tunein" => "TuneIn".to_owned(),
        "" => "atradio".to_owned(),
        other => other.to_owned(),
    }
}

fn source_lexicon(source: &str) -> String {
    match source {
        "Radio Browser" => "radio-browser".to_owned(),
        "TuneIn" => "tunein".to_owned(),
        "" => "custom".to_owned(),
        other => other.to_lowercase(),
    }
}

fn station_info(row: &saved_radio::Model) -> StationInfo {
    StationInfo {
        station_id: row.id.clone(),
        name: row.name.clone(),
        stream_url: row.stream_url.clone(),
        source: source_lexicon(&row.source),
        description: None,
        genre: Some(row.genre.clone()).filter(|value| !value.is_empty()),
        homepage: None,
        logo: Some(row.logo.clone()).filter(|value| !value.is_empty()),
        country: Some(row.country.clone()).filter(|value| !value.is_empty()),
        language: None,
        bitrate: Some(row.bitrate).filter(|bitrate| *bitrate > 0),
        codec: None,
        tags: Vec::new(),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecordStation {
    station_id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    stream_url: String,
    #[serde(default)]
    source: String,
    #[serde(default)]
    genre: Option<String>,
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    logo: Option<String>,
    #[serde(default)]
    bitrate: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct FavoriteRecord {
    station: RecordStation,
}

impl RecordStation {
    fn into_row(self) -> saved_radio::Model {
        saved_radio::Model {
            id: self.station_id,
            name: self.name,
            stream_url: self.stream_url,
            source: source_label(&self.source),
            genre: self.genre.unwrap_or_default(),
            country: self.country.unwrap_or_default(),
            logo: self.logo.unwrap_or_default(),
            bitrate: self.bitrate.unwrap_or_default(),
        }
    }
}

fn rows_of(records: Vec<FavoriteRecord>) -> Vec<saved_radio::Model> {
    records
        .into_iter()
        .map(|record| record.station.into_row())
        .filter(|row| !row.id.is_empty() && !row.stream_url.is_empty())
        .collect()
}

// ── Repo reads ──────────────────────────────────────────────────────────────

/// Every favorite in `car`, the repo archive of `did`.
pub fn car_favorites(car: &[u8], did: &str) -> Result<Vec<saved_radio::Model>, Error> {
    Ok(rows_of(
        atproto::records_from_car::<FavoriteRecord>(car, did, COLLECTION)?
            .into_iter()
            .map(|(_, record)| record)
            .collect(),
    ))
}

/// Every favorite in `did`'s repo, paged through `listRecords`.
pub async fn repo_favorites(did: &str) -> Result<Vec<saved_radio::Model>, Error> {
    Ok(rows_of(
        atproto::list_records::<FavoriteRecord>(did, COLLECTION)
            .await?
            .into_iter()
            .map(|(_, record)| record)
            .collect(),
    ))
}

// ── Local table ─────────────────────────────────────────────────────────────

async fn upsert(conn: &DatabaseConnection, row: saved_radio::Model) -> Result<(), Error> {
    let model = saved_radio::ActiveModel {
        id: ActiveValue::Set(row.id),
        name: ActiveValue::Set(row.name),
        stream_url: ActiveValue::Set(row.stream_url),
        source: ActiveValue::Set(row.source),
        genre: ActiveValue::Set(row.genre),
        country: ActiveValue::Set(row.country),
        logo: ActiveValue::Set(row.logo),
        bitrate: ActiveValue::Set(row.bitrate),
    };
    // `save` inserts or updates depending on whether the primary key exists.
    model.save(conn).await?;
    Ok(())
}

// ── Public API ──────────────────────────────────────────────────────────────

/// Pull the account's atradio favorites into `saved_radio`, then (when signed
/// in to atradio) push up bookmarks that only exist locally.
///
/// The import is additive: a bookmark missing from the repo is left alone
/// rather than deleted, so bookmarks made before the account was linked
/// survive. Removals arrive through [`sync`] instead. Returns how many rows
/// were imported.
pub async fn import_favorites(conn: &DatabaseConnection) -> Result<usize, Error> {
    let Some(did) = resolve_did().await else {
        return Ok(0);
    };
    // The CAR archive is one request for the whole repo, and only downloaded
    // when the local copy has aged out; `listRecords` is the fallback for a PDS
    // that will not serve it.
    let remote = match repo_sync::repo_car(conn, &did).await {
        Ok(None) => {
            tracing::info!("keeping the radio bookmarks already imported from the repo");
            return Ok(0);
        }
        Ok(Some(car)) => match car_favorites(&car, &did) {
            Ok(stations) => stations,
            Err(e) => {
                tracing::warn!("could not read the repo CAR ({e}); falling back to listRecords");
                repo_favorites(&did).await?
            }
        },
        Err(e) => {
            tracing::warn!("could not download the repo CAR ({e}); falling back to listRecords");
            repo_favorites(&did).await?
        }
    };
    let local = saved_radio::Entity::find().all(conn).await?;

    let remote_ids: std::collections::HashSet<&str> =
        remote.iter().map(|row| row.id.as_str()).collect();
    let missing: Vec<saved_radio::Model> = local
        .into_iter()
        .filter(|row| !remote_ids.contains(row.id.as_str()))
        .collect();

    let imported = remote.len();
    for (i, row) in remote.into_iter().enumerate() {
        tracing::info!(
            progress = format!("{}/{imported}", i + 1),
            station = %row.id,
            name = %row.name,
            "importing atradio favorite"
        );
        upsert(conn, row).await?;
    }

    let authenticated = ensure_session().await;
    if authenticated {
        for row in &missing {
            if let Err(e) = favorite(row).await {
                tracing::warn!(station = %row.id, "could not push bookmark to the PDS: {e}");
            }
        }
    }
    tracing::info!(
        imported,
        pushed = if authenticated { missing.len() } else { 0 },
        "synced radio bookmarks with atradio.fm"
    );
    Ok(imported)
}

/// Write a `fm.atradio.favorite` record for `station` to the user's repo.
/// Silently does nothing when there is no atradio session.
pub async fn favorite(station: &saved_radio::Model) -> Result<(), Error> {
    if !ensure_session().await {
        return Ok(());
    }
    agent()
        .favorite(&station_info(station))
        .await
        .map_err(|e| anyhow!("atradio favorite failed: {e}"))?;
    tracing::info!(station = %station.id, "favorited on atradio.fm");
    Ok(())
}

/// Delete the `fm.atradio.favorite` record for `station`. No-op when signed out.
pub async fn unfavorite(station: &saved_radio::Model) -> Result<(), Error> {
    if !ensure_session().await {
        return Ok(());
    }
    agent()
        .unfavorite(&station_info(station))
        .await
        .map_err(|e| anyhow!("atradio unfavorite failed: {e}"))?;
    tracing::info!(station = %station.id, "unfavorited on atradio.fm");
    Ok(())
}

// ── Listening status ────────────────────────────────────────────────────────

/// Publish `station` as the account's now-listening status.
///
/// This is a `put_record` of the `fm.atradio.actor.status` singleton (rkey
/// `self`) straight into the user's repo on their PDS — no AppView in the path
/// — which is what puts them in atradio's recently-played and listener-count
/// feeds. No-op when signed out.
pub async fn set_status(station: &saved_radio::Model) -> Result<(), Error> {
    if !ensure_session().await {
        return Ok(());
    }
    agent()
        .set_play_status(&station_info(station))
        .await
        .map_err(|e| anyhow!("atradio status update failed: {e}"))?;
    tracing::info!(station = %station.id, "published listening status to atradio.fm");
    Ok(())
}

/// Delete the account's now-listening status record from the PDS, i.e. when
/// radio playback stops. No-op when signed out.
pub async fn clear_status() -> Result<(), Error> {
    if !ensure_session().await {
        return Ok(());
    }
    agent()
        .delete_play_status()
        .await
        .map_err(|e| anyhow!("atradio status delete failed: {e}"))?;
    tracing::info!("cleared listening status on atradio.fm");
    Ok(())
}

/// Fill in the metadata a bookmark row carries but a queued radio track does
/// not (genre, country), when the station happens to be bookmarked.
pub async fn enrich_from_bookmark(conn: &DatabaseConnection, station: &mut saved_radio::Model) {
    if let Ok(Some(saved)) = saved_radio::Entity::find_by_id(station.id.clone())
        .one(conn)
        .await
    {
        if station.genre.is_empty() {
            station.genre = saved.genre;
        }
        if station.country.is_empty() {
            station.country = saved.country;
        }
        if station.logo.is_empty() {
            station.logo = saved.logo;
        }
    }
}

// ── Live sync ───────────────────────────────────────────────────────────────

async fn apply_commit(conn: &DatabaseConnection, commit: atproto::JetstreamCommit) {
    match commit.operation.as_str() {
        "create" | "update" => {
            let Some(record) = commit
                .record
                .and_then(|value| serde_json::from_value::<FavoriteRecord>(value).ok())
            else {
                return;
            };
            let row = record.station.into_row();
            if row.id.is_empty() || row.stream_url.is_empty() {
                return;
            }
            tracing::info!(station = %row.id, "radio bookmark added from atradio.fm");
            if let Err(e) = upsert(conn, row).await {
                tracing::warn!("could not store a bookmark from Jetstream: {e}");
            }
        }
        "delete" => {
            // A delete event carries only the rkey. atradio derives that key
            // from the station id, so the matching local row is found by
            // re-deriving it rather than by keeping a uri↔row mapping.
            let Ok(rows) = saved_radio::Entity::find().all(conn).await else {
                return;
            };
            for row in rows {
                if atradio_sdk::agent::favorite_rkey(&row.id) == commit.rkey {
                    tracing::info!(station = %row.id, "radio bookmark removed from atradio.fm");
                    let _ = saved_radio::Entity::delete_by_id(row.id).exec(conn).await;
                    break;
                }
            }
        }
        _ => {}
    }
}

/// Import the account's favorites, then follow its repo for changes until the
/// process exits. Returns immediately when no user is signed in.
pub async fn sync(conn: DatabaseConnection) {
    let Some(did) = resolve_did().await else {
        tracing::info!(
            "atradio bookmark sync off: no Rocksky token and no atproto credentials in the \
             environment (run `rocksky login`, or set ATPROTO_IDENTIFIER + ATPROTO_APP_PASSWORD)"
        );
        return;
    };
    tracing::info!(%did, "linking radio bookmarks to atradio.fm");
    announce_session(&did).await;
    if let Err(e) = import_favorites(&conn).await {
        tracing::warn!("could not import atradio favorites: {e}");
    }

    let applier = conn.clone();
    atproto::subscribe(&did, &[COLLECTION], |commit| {
        let conn = applier.clone();
        async move { apply_commit(&conn, commit).await }
    })
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A record as `com.atproto.repo.listRecords` returns it, trimmed.
    const RECORD: &str = r#"{
        "$type": "fm.atradio.favorite",
        "createdAt": "2026-07-20T03:40:53.228Z",
        "station": {
            "stationId": "rb:a5213a32-d614-47bc-8d52-70a2b6eed8e1",
            "name": "Box Lofi Radio",
            "streamUrl": "https://stream.zeno.fm/tabzverz0fctv",
            "source": "radio-browser",
            "genre": "lofi study beats",
            "country": "Ghana",
            "logo": "https://zeno.fm/logo.webp",
            "codec": "MP3",
            "tags": ["lofi study beats"]
        }
    }"#;

    #[test]
    fn maps_a_favorite_record_onto_a_bookmark_row() {
        let record: FavoriteRecord = serde_json::from_str(RECORD).unwrap();
        let row = record.station.into_row();
        assert_eq!(row.id, "rb:a5213a32-d614-47bc-8d52-70a2b6eed8e1");
        assert_eq!(row.name, "Box Lofi Radio");
        assert_eq!(row.stream_url, "https://stream.zeno.fm/tabzverz0fctv");
        // Lexicon provider name becomes the label the local UI shows.
        assert_eq!(row.source, "Radio Browser");
        assert_eq!(row.country, "Ghana");
        // Absent in the record, and the column is not nullable.
        assert_eq!(row.bitrate, 0);
    }

    #[test]
    fn source_names_round_trip() {
        for label in ["Radio Browser", "TuneIn"] {
            assert_eq!(source_label(&source_lexicon(label)), label);
        }
    }

    /// A Jetstream delete carries only the rkey, so the local row is found by
    /// re-deriving atradio's key from the station id. Guards that assumption.
    #[test]
    fn favorite_rkey_matches_the_published_record_key() {
        assert_eq!(
            atradio_sdk::agent::favorite_rkey("rb:a5213a32-d614-47bc-8d52-70a2b6eed8e1"),
            "93218bda705528f6"
        );
    }

    /// Live check of the CAR import against the signed-in account's real repo.
    /// Ignored by default: it needs an atproto identity and the network. Run
    /// with `cargo test -p music-player-storage -- --ignored car_import`.
    #[tokio::test]
    #[ignore]
    async fn car_import_matches_list_records() {
        let did = resolve_did()
            .await
            .expect("a rocksky token or atproto credentials are required");
        let car = atproto::get_repo_car(&did).await.unwrap();
        let from_car = car_favorites(&car, &did).unwrap();
        let from_list = repo_favorites(&did).await.unwrap();
        let mut car_ids: Vec<&str> = from_car.iter().map(|row| row.id.as_str()).collect();
        let mut list_ids: Vec<&str> = from_list.iter().map(|row| row.id.as_str()).collect();
        car_ids.sort_unstable();
        list_ids.sort_unstable();
        assert_eq!(car_ids, list_ids, "CAR scan and listRecords disagree");
        assert!(!car_ids.is_empty(), "the account has no favorites to check");
    }
}
