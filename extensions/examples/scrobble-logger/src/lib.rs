//! Posts every play to a webhook — the shape a scrobbler takes.
//!
//! The smallest useful `events` extension. What to copy:
//!
//! * handling only the events you care about (this one ignores likes and scans),
//! * reading configuration the manifest declared,
//! * failing softly — an event handler that errors is logged and skipped, so a
//!   webhook being down must not stop playback.
//!
//! Point it at ListenBrainz, Maloja, a Discord webhook, or your own endpoint.
//!
//! Build with `cargo build --release --target wasm32-wasip1`.

mod pdk;

use extism_pdk::*;
use pdk::types;

/// A play, as posted to the webhook. Deliberately close to the ListenBrainz
/// "listen" shape so an endpoint written for that needs little translation.
#[derive(serde::Serialize)]
struct Scrobble<'a> {
    artist: &'a str,
    track: &'a str,
    album: &'a str,
    /// Unix seconds when the play started.
    played_at: i64,
    /// How many times this track has now been played locally.
    play_count: i32,
}

pub(crate) fn on_track_played(input: types::PlayEvent) -> Result<(), Error> {
    let endpoint = pdk::get_config("webhookUrl".to_string()).unwrap_or_default();
    if endpoint.trim().is_empty() {
        // Not configured yet. Say so once per play rather than failing: the
        // user has installed this but not finished setting it up.
        pdk::log("no webhookUrl configured, skipping".to_string())?;
        return Ok(());
    }

    let scrobble = Scrobble {
        artist: &input.track.artist,
        track: &input.track.title,
        album: &input.track.album,
        played_at: input.played_at,
        play_count: input.play_count,
    };
    let body = serde_json::to_vec(&scrobble)?;

    let mut request = HttpRequest::new(&endpoint)
        .with_method("POST")
        .with_header("Content-Type", "application/json");
    // An optional bearer token, for endpoints that want one.
    let token = pdk::get_config("token".to_string()).unwrap_or_default();
    if !token.trim().is_empty() {
        request = request.with_header("Authorization", format!("Bearer {token}"));
    }

    match http::request::<Vec<u8>>(&request, Some(body)) {
        Ok(response) if (200..300).contains(&response.status_code()) => {
            pdk::log(format!(
                "scrobbled {} — {}",
                input.track.artist, input.track.title
            ))?;
        }
        Ok(response) => {
            // A rejected scrobble is worth reporting, but not worth failing
            // over: the next play will try again.
            pdk::log(format!(
                "the webhook answered {} for {} — {}",
                response.status_code(),
                input.track.artist,
                input.track.title
            ))?;
        }
        Err(e) => pdk::log(format!("could not reach the webhook: {e}"))?,
    }
    Ok(())
}

/// Skips are worth knowing about too, and cost nothing to log.
pub(crate) fn on_track_skipped(input: types::SkipEvent) -> Result<(), Error> {
    pdk::log(format!(
        "skipped {} — {}",
        input.track.artist, input.track.title
    ))?;
    Ok(())
}

/// An extension with the `events` capability need not handle every event — a
/// missing export is "not implemented", not an error. These three are left
/// empty to show that.
pub(crate) fn on_track_liked(_input: types::LikeEvent) -> Result<(), Error> {
    Ok(())
}

pub(crate) fn on_playlist_created(_input: types::PlaylistEvent) -> Result<(), Error> {
    Ok(())
}

pub(crate) fn on_scan_completed(_input: types::ScanEvent) -> Result<(), Error> {
    Ok(())
}

// ── Unused capabilities ─────────────────────────────────────────────────────
//
// `pdk.rs` is generated from the whole schema, so the module exports every
// function in it. That is harmless: the host calls an export only when the
// manifest declares the matching capability. These stubs satisfy the linker.

pub(crate) fn get_metadata(
    _input: types::MetadataRequest,
) -> Result<types::MetadataResponse, Error> {
    Ok(types::MetadataResponse {
        lyrics: None,
        artwork_url: None,
        bio: None,
        genres: Vec::new(),
    })
}

pub(crate) fn commands() -> Result<Vec<types::CommandSpec>, Error> {
    Ok(Vec::new())
}

pub(crate) fn run_command(_input: types::CommandRequest) -> Result<types::CommandResponse, Error> {
    Ok(types::CommandResponse {
        ok: false,
        message: String::new(),
        data: String::new(),
    })
}

pub(crate) fn predicates() -> Result<Vec<types::PredicateSpec>, Error> {
    Ok(Vec::new())
}

pub(crate) fn evaluate(_input: types::PredicateRequest) -> Result<types::PredicateResponse, Error> {
    Ok(types::PredicateResponse { matches: false })
}

pub(crate) fn source_info() -> Result<types::SourceInfo, Error> {
    Ok(types::SourceInfo {
        name: String::new(),
        description: String::new(),
        requires_auth: false,
        supports_search: false,
    })
}

pub(crate) fn list_albums(_input: types::BrowseRequest) -> Result<Vec<types::SourceAlbum>, Error> {
    Ok(Vec::new())
}

pub(crate) fn list_artists(_input: types::BrowseRequest) -> Result<Vec<types::SourceArtist>, Error> {
    Ok(Vec::new())
}

pub(crate) fn list_tracks(_input: types::BrowseRequest) -> Result<Vec<types::Track>, Error> {
    Ok(Vec::new())
}

pub(crate) fn list_playlists(
    _input: types::BrowseRequest,
) -> Result<Vec<types::SourcePlaylist>, Error> {
    Ok(Vec::new())
}

pub(crate) fn get_stream_url(_input: types::StreamRequest) -> Result<types::StreamResponse, Error> {
    Ok(types::StreamResponse {
        url: String::new(),
        mime_type: String::new(),
        headers: String::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_scrobble_serializes_to_the_expected_shape() {
        let scrobble = Scrobble {
            artist: "Radiohead",
            track: "Airbag",
            album: "OK Computer",
            played_at: 1_700_000_000,
            play_count: 3,
        };
        let json: serde_json::Value = serde_json::to_value(&scrobble).unwrap();
        assert_eq!(json["artist"], "Radiohead");
        assert_eq!(json["playedAt"], serde_json::Value::Null);
        // Field names are snake_case as written — an endpoint reads these
        // exactly, so the test pins them.
        assert_eq!(json["played_at"], 1_700_000_000_i64);
        assert_eq!(json["play_count"], 3);
    }
}
