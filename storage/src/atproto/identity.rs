//! Who the local user is, without requiring any session: the Rocksky login
//! token, atproto password credentials from the environment, and handle
//! resolution.

use std::path::PathBuf;

use base64::Engine;
use serde::Deserialize;

use super::http;

fn token_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/"))
        .join(".rocksky")
        .join("token.json")
}

/// The user's DID as carried by the `rocksky login` token. The token is a JWT
/// whose payload holds the DID, so no network round-trip is needed.
pub fn token_did() -> Option<String> {
    let raw = std::fs::read_to_string(token_path()).ok()?;
    let token = serde_json::from_str::<serde_json::Value>(&raw)
        .ok()?
        .get("token")?
        .as_str()?
        .to_owned();
    let payload = token.split('.').nth(1)?;
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .ok()?;
    serde_json::from_slice::<serde_json::Value>(&decoded)
        .ok()?
        .get("did")?
        .as_str()
        .filter(|did| did.starts_with("did:"))
        .map(String::from)
}

fn env_nonempty(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.trim().is_empty())
}

/// atproto password credentials from the environment, reading the same
/// variables (and precedence) as the `atradio` CLI so one setup covers both.
pub fn env_credentials() -> Option<(String, String)> {
    let identifier = env_nonempty("ATPROTO_IDENTIFIER")
        .or_else(|| env_nonempty("ATRADIO_IDENTIFIER"))
        .or_else(|| env_nonempty("BLUESKY_IDENTIFIER"))?;
    let password = env_nonempty("ATPROTO_APP_PASSWORD")
        .or_else(|| env_nonempty("ATPROTO_PASSWORD"))
        .or_else(|| env_nonempty("ATRADIO_APP_PASSWORD"))
        .or_else(|| env_nonempty("BLUESKY_APP_PASSWORD"))?;
    Some((identifier, password))
}

/// Resolve a handle to its DID through the public identity service.
pub async fn resolve_handle(handle: &str) -> Option<String> {
    #[derive(Deserialize)]
    struct Resolved {
        did: String,
    }
    let resolved: Resolved = http()
        .ok()?
        .get("https://public.api.bsky.app/xrpc/com.atproto.identity.resolveHandle")
        .query(&[("handle", handle)])
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json()
        .await
        .ok()?;
    Some(resolved.did)
}

/// Whose repo to read, without needing any session: the `rocksky login` token,
/// else the password-auth identifier from the environment (used as-is when it
/// is already a DID, otherwise resolved as a handle).
pub async fn resolve_did() -> Option<String> {
    if let Some(did) = token_did() {
        return Some(did);
    }
    let (identifier, _) = env_credentials()?;
    if identifier.starts_with("did:") {
        return Some(identifier);
    }
    resolve_handle(&identifier).await
}
