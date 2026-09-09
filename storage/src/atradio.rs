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
use std::sync::{Arc, OnceLock, RwLock};

use anyhow::{anyhow, Error};
use atradio_sdk::{AtradioAgent, StationDraft, StationInfo};
use music_player_entity::saved_radio;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};
use serde::Deserialize;

use crate::{atproto, repo_sync};

const COLLECTION: &str = "fm.atradio.favorite";
/// The user's own stations — the ones typed in by hand rather than found in a
/// directory. Separate from a favorite: a station is *authored*, a favorite is
/// a bookmark of one (a custom station is normally both).
const STATION_COLLECTION: &str = "fm.atradio.station";

/// Provider label a hand-entered station carries in `saved_radio.source`.
pub const CUSTOM_SOURCE: &str = "Custom";

// ── Identity ────────────────────────────────────────────────────────────────

/// The session file shared with the `atradio` CLI, so logging in there is
/// enough to enable write-through here.
fn session_path() -> PathBuf {
    directories::ProjectDirs::from("fm", "atradio", "atradio")
        .map(|dirs| dirs.config_dir().join("session.json"))
        .unwrap_or_else(|| PathBuf::from(".atradio-session.json"))
}

/// The agent is cached rather than rebuilt per call: it owns the lock that
/// keeps refresh-token rotations ordered, so every write must go through the
/// same one. It is swapped out only when the session is thrown away.
static AGENT: OnceLock<RwLock<Arc<AtradioAgent>>> = OnceLock::new();

fn agent_cell() -> &'static RwLock<Arc<AtradioAgent>> {
    AGENT.get_or_init(|| RwLock::new(Arc::new(AtradioAgent::new(session_path()))))
}

fn agent() -> Arc<AtradioAgent> {
    agent_cell().read().unwrap().clone()
}

/// Rebuild the agent around the (now empty) session store.
fn reset_agent() {
    *agent_cell().write().unwrap() = Arc::new(AtradioAgent::new(session_path()));
}

/// The profile of the session on disk, if there is one.
pub fn profile() -> Option<atradio_sdk::Profile> {
    agent().profile()
}

/// Establish a session from a handle and an app password.
///
/// Writes the shared session file, so this is also what turns on scrobbling
/// and station sync — there is one session, not one per feature.
pub async fn sign_in(
    identifier: &str,
    password: &str,
) -> Result<atradio_sdk::Profile, Error> {
    // Any existing session is dropped first: `login_password` resumes one when
    // it can, which would silently sign in as whoever was already there.
    agent().logout();
    reset_agent();
    agent()
        .login_password(identifier, password)
        .await
        .map_err(|e| Error::msg(sign_in_message(&e)))
}

/// Forget the session, on disk and in memory.
pub fn sign_out() {
    agent().logout();
    reset_agent();
}

/// What went wrong, in words a person can act on.
///
/// The SDK erases jacquard's own errors to a string, so the useful cases are
/// recognised by name; anything else is passed through rather than replaced
/// with something vaguer.
fn sign_in_message(e: &atradio_sdk::SdkError) -> String {
    let raw = e.to_string();
    if raw.contains("AuthenticationRequired") || raw.contains("Invalid identifier or password") {
        return "that handle and app password did not match".to_string();
    }
    if raw.contains("AccountTakedown") {
        return "that account has been taken down".to_string();
    }
    if raw.contains("RateLimit") {
        return "too many attempts — wait a minute and try again".to_string();
    }
    raw
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
    // Both the daemon's startup check and each sync task resolve the identity;
    // saying so once is enough.
    fn announce(source: &str, did: &str) {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| tracing::info!(%did, "atproto identity resolved from the {source}"));
    }
    if let Some(did) = atproto::token_did() {
        announce("Rocksky token", &did);
        return Some(did);
    }
    if let Some(profile) = agent().profile() {
        announce("stored session", &profile.did);
        return Some(profile.did);
    }
    let (identifier, _) = atproto::env_credentials()?;
    if identifier.starts_with("did:") {
        announce("environment", &identifier);
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
    // A stored password session is usually already past its access-token
    // lifetime by the time the daemon starts, so refresh before the first
    // write rather than letting that write fail and retry.
    if agent().is_logged_in() {
        reauthenticate().await;
    }
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
        "custom" => CUSTOM_SOURCE.to_owned(),
        "" => "atradio".to_owned(),
        other => other.to_owned(),
    }
}

fn source_lexicon(source: &str) -> String {
    match source {
        "Radio Browser" => "radio-browser".to_owned(),
        "TuneIn" => "tunein".to_owned(),
        CUSTOM_SOURCE | "" => "custom".to_owned(),
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

fn station_draft(row: &saved_radio::Model) -> StationDraft {
    StationDraft {
        name: row.name.clone(),
        stream_url: row.stream_url.clone(),
        genre: Some(row.genre.clone()).filter(|value| !value.is_empty()),
        homepage: None,
        logo: Some(row.logo.clone()).filter(|value| !value.is_empty()),
    }
}

/// Turn a station snapshot from the AppView into a bookmark row.
fn row_of(info: StationInfo) -> saved_radio::Model {
    saved_radio::Model {
        id: info.station_id,
        name: info.name,
        stream_url: info.stream_url,
        source: source_label(&info.source),
        genre: info.genre.unwrap_or_default(),
        country: info.country.unwrap_or_default(),
        logo: info.logo.unwrap_or_default(),
        bitrate: info.bitrate.unwrap_or_default(),
    }
}

/// atradio keys a user's own station by its record rkey, so the id the AppView
/// lists it under is derivable from the uri `create_station` hands back.
fn custom_id(rkey: &str) -> String {
    format!("custom:{rkey}")
}

fn rkey_of(uri: &str) -> Option<&str> {
    uri.rsplit('/').next().filter(|rkey| !rkey.is_empty())
}

/// The id a hand-entered station gets before it reaches a PDS. Derived from the
/// stream url so adding the same station twice updates one row instead of
/// piling up duplicates.
pub fn local_station_id(stream_url: &str) -> String {
    custom_id(&format!("local-{:x}", md5::compute(stream_url.trim())))
}

/// True for an id this device minted, i.e. a station that has not been
/// published yet.
fn is_local_id(id: &str) -> bool {
    id.starts_with("custom:local-")
}

/// The `fm.atradio.station` record shape, for the Jetstream feed. The station
/// id is not in the record — atradio derives it from the record's rkey.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StationRecord {
    #[serde(default)]
    name: String,
    #[serde(default)]
    stream_url: String,
    #[serde(default)]
    genre: Option<String>,
    #[serde(default)]
    logo: Option<String>,
}

impl StationRecord {
    fn into_row(self, rkey: &str) -> saved_radio::Model {
        saved_radio::Model {
            id: custom_id(rkey),
            name: self.name,
            stream_url: self.stream_url,
            source: CUSTOM_SOURCE.to_owned(),
            genre: self.genre.unwrap_or_default(),
            country: String::new(),
            logo: self.logo.unwrap_or_default(),
            bitrate: 0,
        }
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
    // `save()` would issue an UPDATE here: these primary keys are assigned by
    // us, never auto-incremented, so sea-orm always sees one that is Set and
    // takes a new station for an existing row — an update matching nothing,
    // which comes back as RecordNotFound. An explicit upsert is what this
    // wants.
    saved_radio::Entity::insert(model)
        .on_conflict(
            OnConflict::column(saved_radio::Column::Id)
                .update_columns([
                    saved_radio::Column::Name,
                    saved_radio::Column::StreamUrl,
                    saved_radio::Column::Source,
                    saved_radio::Column::Genre,
                    saved_radio::Column::Country,
                    saved_radio::Column::Logo,
                    saved_radio::Column::Bitrate,
                ])
                .to_owned(),
        )
        .exec(conn)
        .await?;
    Ok(())
}

/// Re-establish a session after the PDS rejected the current one.
///
/// A password session's access token is short-lived, and the stored one is
/// often already stale by the time the daemon starts. Refreshing is tried
/// first; when the refresh token is dead too, the environment credentials mint
/// a completely new session.
async fn reauthenticate() -> bool {
    if agent().refresh_session().await.is_ok() {
        tracing::info!("refreshed the atradio session");
        return true;
    }
    let Some((identifier, password)) = atproto::env_credentials() else {
        tracing::warn!(
            "the atradio session expired and cannot be refreshed; run `atradio login`, or set \
             ATPROTO_IDENTIFIER + ATPROTO_APP_PASSWORD"
        );
        return false;
    };
    // `login_password` resumes a stored session when it finds one, so with the
    // dead session still on disk it would hand back the same expired token and
    // report success. Dropping it first forces a real login.
    agent().logout();
    reset_agent();
    match agent().login_password(&identifier, &password).await {
        Ok(profile) => {
            tracing::info!(
                user = %profile.handle,
                "the atradio session expired; signed in again with the environment credentials"
            );
            true
        }
        Err(e) => {
            tracing::warn!("could not re-authenticate with atradio.fm: {e}");
            false
        }
    }
}

/// True for the errors that mean "this session is no longer good", as opposed
/// to a network blip or a genuine rejection.
fn is_auth_error(e: &atradio_sdk::SdkError) -> bool {
    match e {
        atradio_sdk::SdkError::NotAuthenticated | atradio_sdk::SdkError::SessionExpired => true,
        // jacquard's own errors are erased to a string by the SDK, so the
        // expired-token case can only be recognised by name.
        atradio_sdk::SdkError::Auth(message) => {
            message.contains("TokenExpired") || message.contains("ExpiredToken")
        }
        _ => false,
    }
}

/// Run a PDS write, re-authenticating and retrying once if the session had
/// expired. Every write here is an idempotent put or delete, so a retry cannot
/// duplicate anything.
async fn write<T, F, Fut>(what: &str, operation: F) -> Result<T, Error>
where
    F: Fn(Arc<AtradioAgent>) -> Fut,
    Fut: std::future::Future<Output = atradio_sdk::Result<T>>,
{
    match operation(agent()).await {
        Ok(value) => Ok(value),
        Err(e) if is_auth_error(&e) => {
            if !reauthenticate().await {
                return Err(anyhow!("{what} failed: {e}"));
            }
            // Deliberately re-read the agent: re-authenticating replaces it.
            operation(agent())
                .await
                .map_err(|e| anyhow!("{what} failed after re-authenticating: {e}"))
        }
        Err(e) => Err(anyhow!("{what} failed: {e}")),
    }
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
    let info = &station_info(station);
    write("atradio favorite", move |agent| async move {
        agent.favorite(info).await
    })
    .await?;
    tracing::info!(station = %station.id, "favorited on atradio.fm");
    Ok(())
}

/// Delete the `fm.atradio.favorite` record for `station`. No-op when signed out.
pub async fn unfavorite(station: &saved_radio::Model) -> Result<(), Error> {
    if !ensure_session().await {
        return Ok(());
    }
    let info = &station_info(station);
    write("atradio unfavorite", move |agent| async move {
        agent.unfavorite(info).await
    })
    .await?;
    tracing::info!(station = %station.id, "unfavorited on atradio.fm");
    Ok(())
}

// ── The user's own stations ─────────────────────────────────────────────────

/// Save a hand-entered station and, when signed in, publish it to the user's
/// repo as `fm.atradio.station` so it shows up on atradio.fm and on their other
/// devices. Returns the stored row.
///
/// Publishing re-keys the row from the local id to the `custom:<rkey>` the
/// AppView will list it under, so a later [`import_stations`] recognises it as
/// the same station instead of importing a second copy. Signed out, the station
/// stays local with its local id and is published by the next sync.
pub async fn add_station(
    conn: &DatabaseConnection,
    mut row: saved_radio::Model,
) -> Result<saved_radio::Model, Error> {
    row.source = CUSTOM_SOURCE.to_owned();
    if row.id.is_empty() {
        row.id = local_station_id(&row.stream_url);
    }
    upsert(conn, row.clone()).await?;
    if let Some(published) = publish_station(&row).await? {
        row = replace_id(conn, &row, published).await?;
    }
    // A station the user added is one they want in their list of stations, so
    // it is favorited too — that is what puts it in the bookmarks everywhere.
    if let Err(e) = favorite(&row).await {
        tracing::warn!(station = %row.id, "could not mirror the new station to atradio.fm: {e}");
    }
    Ok(row)
}

/// Move a row to a new primary key. sea-orm cannot update a primary key in
/// place, so the new row is written first and the old one dropped after.
async fn replace_id(
    conn: &DatabaseConnection,
    row: &saved_radio::Model,
    id: String,
) -> Result<saved_radio::Model, Error> {
    if row.id == id {
        return Ok(row.clone());
    }
    let moved = saved_radio::Model { id, ..row.clone() };
    upsert(conn, moved.clone()).await?;
    saved_radio::Entity::delete_by_id(row.id.clone())
        .exec(conn)
        .await?;
    Ok(moved)
}

/// Publish `station` as a `fm.atradio.station` record and return the
/// `custom:<rkey>` id the AppView will list it under. `Ok(None)` when signed
/// out — nothing is written and the caller keeps the local id.
pub async fn publish_station(station: &saved_radio::Model) -> Result<Option<String>, Error> {
    if !ensure_session().await {
        return Ok(None);
    }
    let draft = &station_draft(station);
    let uri = write("atradio station", move |agent| async move {
        agent.create_station(draft).await
    })
    .await?;
    tracing::info!(station = %station.id, %uri, "published a station to atradio.fm");
    Ok(rkey_of(&uri).map(custom_id))
}

/// Pull the account's own `fm.atradio.station` records into `saved_radio`, then
/// publish any hand-entered station that has never reached the PDS.
///
/// Runs before the favorite import so a station published here is favorited
/// under its final `custom:<rkey>` id rather than the local one.
pub async fn import_stations(conn: &DatabaseConnection) -> Result<usize, Error> {
    let Some(did) = resolve_did().await else {
        return Ok(0);
    };
    let remote = agent()
        .appview()
        .stations(&did, 100)
        .await
        .map_err(|e| anyhow!("could not list the stations on atradio.fm: {e}"))?;

    let imported = remote.items.len();
    for view in remote.items {
        let row = row_of(view.station);
        if row.id.is_empty() || row.stream_url.is_empty() {
            continue;
        }
        tracing::info!(station = %row.id, name = %row.name, "importing an atradio station");
        upsert(conn, row).await?;
    }

    // Anything still carrying a local id was added while signed out (or before
    // the account was linked); this is its first chance to go up.
    let unpublished: Vec<saved_radio::Model> = saved_radio::Entity::find()
        .all(conn)
        .await?
        .into_iter()
        .filter(|row| is_local_id(&row.id))
        .collect();
    let mut published = 0;
    for row in unpublished {
        match publish_station(&row).await {
            Ok(Some(id)) => {
                replace_id(conn, &row, id).await?;
                published += 1;
            }
            Ok(None) => break, // signed out; the rest would fail the same way
            Err(e) => tracing::warn!(station = %row.id, "could not publish the station: {e}"),
        }
    }
    tracing::info!(imported, published, "synced stations with atradio.fm");
    Ok(imported)
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
    let info = &station_info(station);
    write("atradio status update", move |agent| async move {
        agent.set_play_status(info).await
    })
    .await?;
    tracing::info!(station = %station.id, "published listening status to atradio.fm");
    Ok(())
}

/// Delete the account's now-listening status record from the PDS, i.e. when
/// radio playback stops. No-op when signed out.
pub async fn clear_status() -> Result<(), Error> {
    if !ensure_session().await {
        return Ok(());
    }
    write("atradio status delete", move |agent| async move {
        agent.delete_play_status().await
    })
    .await?;
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
    if commit.collection == STATION_COLLECTION {
        return apply_station_commit(conn, commit).await;
    }
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

/// A station created or removed on atradio.fm (or another device) while the
/// daemon runs. The record has no station id of its own — atradio keys it by
/// rkey — so both halves re-derive `custom:<rkey>`.
async fn apply_station_commit(conn: &DatabaseConnection, commit: atproto::JetstreamCommit) {
    match commit.operation.as_str() {
        "create" | "update" => {
            let Some(record) = commit
                .record
                .and_then(|value| serde_json::from_value::<StationRecord>(value).ok())
            else {
                return;
            };
            let row = record.into_row(&commit.rkey);
            if row.name.is_empty() || row.stream_url.is_empty() {
                return;
            }
            tracing::info!(station = %row.id, "station added from atradio.fm");
            if let Err(e) = upsert(conn, row).await {
                tracing::warn!("could not store a station from Jetstream: {e}");
            }
        }
        "delete" => {
            let id = custom_id(&commit.rkey);
            tracing::info!(station = %id, "station removed from atradio.fm");
            let _ = saved_radio::Entity::delete_by_id(id).exec(conn).await;
        }
        _ => {}
    }
}

/// Import the account's stations and favorites, then follow its repo for
/// changes until the process exits. Returns immediately when no user is signed
/// in.
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
    // Stations first: publishing one re-keys its local row, and the favorite
    // import pushes local-only bookmarks up — in the other order it would
    // favorite the throwaway local id.
    if let Err(e) = import_stations(&conn).await {
        tracing::warn!("could not import atradio stations: {e}");
    }
    if let Err(e) = import_favorites(&conn).await {
        tracing::warn!("could not import atradio favorites: {e}");
    }

    let applier = conn.clone();
    atproto::subscribe(&did, &[COLLECTION, STATION_COLLECTION], |commit| {
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

    /// The expired-token case only reaches us as a string inside
    /// `SdkError::Auth`, and getting this wrong means writes stop retrying.
    #[test]
    fn recognises_expired_sessions() {
        use atradio_sdk::SdkError;
        assert!(is_auth_error(&SdkError::SessionExpired));
        assert!(is_auth_error(&SdkError::NotAuthenticated));
        assert!(is_auth_error(&SdkError::Auth(
            "update play status: AgentError { kind: Auth(TokenExpired), source: None }".into()
        )));
        assert!(!is_auth_error(&SdkError::Auth(
            "create favorite: 400".into()
        )));
        assert!(!is_auth_error(&SdkError::RecordNotFound));
    }

    #[test]
    fn source_names_round_trip() {
        for label in ["Radio Browser", "TuneIn", CUSTOM_SOURCE] {
            assert_eq!(source_label(&source_lexicon(label)), label);
        }
    }

    /// A station added while signed out gets a local id derived from its
    /// stream url; publishing swaps it for the `custom:<rkey>` the AppView
    /// lists. Both halves have to agree or every sync imports a duplicate.
    #[test]
    fn custom_station_ids() {
        let id = local_station_id("https://example.com/stream");
        assert_eq!(id, local_station_id("  https://example.com/stream  "));
        assert!(is_local_id(&id));

        let published = rkey_of("at://did:plc:abc/fm.atradio.station/3labcd234").map(custom_id);
        assert_eq!(published.as_deref(), Some("custom:3labcd234"));
        assert!(!is_local_id(published.as_deref().unwrap()));
    }

    /// A Jetstream station commit carries no station id — it has to come from
    /// the rkey, or the row lands under a key nothing else can find.
    #[test]
    fn station_records_are_keyed_by_rkey() {
        let record: StationRecord = serde_json::from_str(
            r#"{"name":"Night Drive","streamUrl":"https://example.com/nd","genre":"Synthwave"}"#,
        )
        .unwrap();
        let row = record.into_row("3lxyz789");
        assert_eq!(row.id, "custom:3lxyz789");
        assert_eq!(row.source, CUSTOM_SOURCE);
        assert_eq!(row.genre, "Synthwave");
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

    async fn temp_db() -> (tempfile::TempDir, DatabaseConnection) {
        use migration::{Migrator, MigratorTrait};
        let dir = tempfile::tempdir().unwrap();
        let url = format!(
            "sqlite://{}?mode=rwc",
            dir.path().join("music-player.sqlite3").display()
        );
        let conn = sea_orm::Database::connect(&url).await.unwrap();
        Migrator::up(&conn, None).await.unwrap();
        (dir, conn)
    }

    fn station(name: &str) -> saved_radio::Model {
        saved_radio::Model {
            id: "rb:test".into(),
            name: name.into(),
            stream_url: "https://example.com/stream".into(),
            source: "Radio Browser".into(),
            genre: "lofi".into(),
            country: "Ghana".into(),
            logo: String::new(),
            bitrate: 128,
        }
    }

    /// Importing the same favorite twice must insert then update. `save()` used
    /// to issue an UPDATE for the insert too, which matched no rows and failed
    /// the whole import with RecordNotFound.
    #[tokio::test]
    async fn importing_a_favorite_twice_inserts_then_updates() {
        let (_dir, conn) = temp_db().await;

        upsert(&conn, station("Lofi 24/7")).await.expect("insert");
        upsert(&conn, station("Lofi Renamed"))
            .await
            .expect("update");

        let rows = saved_radio::Entity::find().all(&conn).await.unwrap();
        assert_eq!(rows.len(), 1, "the second import must not add a row");
        assert_eq!(rows[0].name, "Lofi Renamed");
        assert_eq!(rows[0].bitrate, 128);
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
