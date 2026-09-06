//! Rocksky like/unlike sync (mirrors rockbox-zig's crates/rocksky client —
//! the SDK only reads loved songs, so the REST endpoints are used directly).
//!
//! Lives here so both API layers (gRPC in `music-player-server`, GraphQL in
//! `music-player-graphql`) reach it through their shared storage dependency.
//! Requires the access token written by `rocksky login` to
//! `~/.rocksky/token.json`; without it every call is a silent no-op.

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Error;
use music_player_entity::track::Model as Track;
use sha2::{Digest, Sha256};

fn token_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/"))
        .join(".rocksky")
        .join("token.json")
}

fn read_token() -> Option<String> {
    let raw = std::fs::read_to_string(token_path()).ok()?;
    let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
    value
        .get("token")
        .and_then(|t| t.as_str())
        .filter(|t| !t.trim().is_empty())
        .map(String::from)
}

fn api_url() -> String {
    std::env::var("ROCKSKY_API_URL").unwrap_or_else(|_| "https://api.rocksky.app".to_string())
}

fn http() -> Result<reqwest::Client, Error> {
    Ok(reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()?)
}

/// Rocksky identifies a song by sha256 of "title - artist - album",
/// lowercased.
fn song_hash(track: &Track) -> String {
    let key = format!("{} - {} - {}", track.title, track.artist, track.album.title).to_lowercase();
    format!("{:x}", Sha256::digest(key.as_bytes()))
}

/// Like `track` on Rocksky (`POST /likes`). No-op without a login token.
pub async fn like(track: &Track) -> Result<(), Error> {
    let Some(token) = read_token() else {
        return Ok(());
    };
    let response = http()?
        .post(format!("{}/likes", api_url()))
        .bearer_auth(token)
        .json(&serde_json::json!({
            "title": track.title,
            "album": track.album.title,
            "artist": track.artist,
            "albumArtist": track.album.artist,
            "duration": (track.duration.unwrap_or(0.0) * 1000.0) as u64,
            "trackNumber": track.track.unwrap_or_default(),
            "year": track.album.year,
            "discNumber": 1,
        }))
        .send()
        .await?;
    if !response.status().is_success() {
        tracing::warn!(
            title = %track.title,
            status = %response.status(),
            "rocksky like failed"
        );
    } else {
        tracing::info!(title = %track.title, "liked on Rocksky");
    }
    Ok(())
}

/// Remove the like on Rocksky (`DELETE /likes/<sha256>`). No-op without a
/// login token.
pub async fn unlike(track: &Track) -> Result<(), Error> {
    let Some(token) = read_token() else {
        return Ok(());
    };
    let hash = song_hash(track);
    let response = http()?
        .delete(format!("{}/likes/{hash}", api_url()))
        .bearer_auth(token)
        .send()
        .await?;
    if !response.status().is_success() {
        tracing::warn!(
            title = %track.title,
            status = %response.status(),
            "rocksky unlike failed"
        );
    } else {
        tracing::info!(title = %track.title, "unliked on Rocksky");
    }
    Ok(())
}

/// Fire-and-forget like/unlike: looks the track up (with its album) and
/// syncs Rocksky in a spawned task, so API handlers never block on the
/// network. Errors are logged, never surfaced.
pub fn sync_like(db: &crate::Database, track_id: String, liked: bool) {
    let repo = crate::repo::track::TrackRepository::new(db.get_connection());
    tokio::spawn(async move {
        let track = match repo.find(&track_id).await {
            Ok(track) => track,
            Err(e) => {
                tracing::warn!("rocksky like: track {track_id} not found: {e}");
                return;
            }
        };
        let result = if liked {
            like(&track).await
        } else {
            unlike(&track).await
        };
        if let Err(e) = result {
            tracing::warn!("rocksky like sync failed: {e}");
        }
    });
}
