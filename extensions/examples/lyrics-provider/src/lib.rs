//! Fetches synced lyrics from [LRCLIB](https://lrclib.net), a free, no-key
//! lyrics API.
//!
//! What to copy from this example:
//!
//! * answering only for what you know — the host merges answers across
//!   providers, taking the first non-empty value per field, so returning an
//!   empty response is a normal outcome rather than a failure,
//!   and leaving `artwork_url` unset lets an artwork provider fill it in,
//! * preferring synced (LRC) lyrics over plain, falling back cleanly,
//! * treating "not found" as an empty answer, not an error.
//!
//! Build with `cargo build --release --target wasm32-wasip1`.

mod pdk;

use extism_pdk::*;
use pdk::types;

const API: &str = "https://lrclib.net";

/// LRCLIB's response. Both lyric fields are nullable — a track can be in the
/// database with neither, which is how instrumentals are represented.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Lyrics {
    #[serde(default)]
    synced_lyrics: Option<String>,
    #[serde(default)]
    plain_lyrics: Option<String>,
    #[serde(default)]
    instrumental: bool,
}

impl Lyrics {
    /// The best lyrics available: synced if there are any, else plain. An
    /// instrumental has none by definition and must not come back as an empty
    /// string, or it would look like a failed lookup to the next provider.
    fn best(self) -> Option<String> {
        if self.instrumental {
            return None;
        }
        self.synced_lyrics
            .filter(|l| !l.trim().is_empty())
            .or(self.plain_lyrics)
            .filter(|l| !l.trim().is_empty())
    }
}

pub(crate) fn get_metadata(
    input: types::MetadataRequest,
) -> Result<types::MetadataResponse, Error> {
    let empty = types::MetadataResponse {
        lyrics: None,
        artwork_url: None,
        bio: None,
        genres: Vec::new(),
    };

    let track = &input.track;
    if track.artist.trim().is_empty() || track.title.trim().is_empty() {
        // Nothing to look up. Not worth a request.
        return Ok(empty);
    }

    // LRCLIB matches on artist + title + album + duration. Sending the
    // duration is what makes it pick the right version of a song that has
    // several; it is optional, so a track with no duration still matches.
    let mut url = format!(
        "{API}/api/get?artist_name={}&track_name={}",
        urlencode(&track.artist),
        urlencode(&track.title)
    );
    if !track.album.trim().is_empty() {
        url.push_str(&format!("&album_name={}", urlencode(&track.album)));
    }
    if track.duration > 0.0 {
        url.push_str(&format!("&duration={}", track.duration.round() as i64));
    }

    let request = HttpRequest::new(&url)
        .with_method("GET")
        .with_header("User-Agent", "music-player-lyrics-provider/0.1");
    let response = match http::request::<()>(&request, None) {
        Ok(response) => response,
        Err(e) => {
            // The network being down is not this track having no lyrics; say
            // so in the log and let another provider try.
            pdk::log(format!("lrclib request failed: {e}"))?;
            return Ok(empty);
        }
    };

    // 404 is the normal "we do not have this one" answer.
    if response.status_code() == 404 {
        return Ok(empty);
    }
    if response.status_code() != 200 {
        pdk::log(format!("lrclib answered {}", response.status_code()))?;
        return Ok(empty);
    }

    let found: Lyrics = match serde_json::from_slice(&response.body()) {
        Ok(found) => found,
        Err(e) => {
            pdk::log(format!("could not read the lrclib response: {e}"))?;
            return Ok(empty);
        }
    };

    Ok(types::MetadataResponse {
        lyrics: found.best(),
        // Left unset on purpose: this provider knows about lyrics and nothing
        // else, so an artwork or bio provider can still fill these in.
        artwork_url: None,
        bio: None,
        genres: Vec::new(),
    })
}

/// Percent-encode a query value. Written out rather than pulled in — a
/// WebAssembly module pays for every dependency in bytes.
fn urlencode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

// ── Unused capabilities ─────────────────────────────────────────────────────
//
// `pdk.rs` is generated from the whole schema, so the module exports every
// function in it. That is harmless: the host calls an export only when the
// manifest declares the matching capability. These stubs satisfy the linker.

pub(crate) fn on_track_played(_input: types::PlayEvent) -> Result<(), Error> {
    Ok(())
}

pub(crate) fn on_track_skipped(_input: types::SkipEvent) -> Result<(), Error> {
    Ok(())
}

pub(crate) fn on_track_liked(_input: types::LikeEvent) -> Result<(), Error> {
    Ok(())
}

pub(crate) fn on_playlist_created(_input: types::PlaylistEvent) -> Result<(), Error> {
    Ok(())
}

pub(crate) fn on_scan_completed(_input: types::ScanEvent) -> Result<(), Error> {
    Ok(())
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

    fn lyrics(synced: Option<&str>, plain: Option<&str>, instrumental: bool) -> Lyrics {
        Lyrics {
            synced_lyrics: synced.map(String::from),
            plain_lyrics: plain.map(String::from),
            instrumental,
        }
    }

    #[test]
    fn prefers_synced_over_plain() {
        assert_eq!(
            lyrics(Some("[00:01.00] la"), Some("la"), false).best(),
            Some("[00:01.00] la".to_string())
        );
    }

    #[test]
    fn falls_back_to_plain() {
        assert_eq!(
            lyrics(None, Some("la la la"), false).best(),
            Some("la la la".to_string())
        );
        // An empty synced field is not an answer either.
        assert_eq!(
            lyrics(Some("   "), Some("la"), false).best(),
            Some("la".to_string())
        );
    }

    /// An instrumental genuinely has no lyrics. Returning `Some("")` would look
    /// like a successful lookup and stop other providers from being asked.
    #[test]
    fn an_instrumental_has_no_lyrics() {
        assert_eq!(lyrics(Some("[00:01.00] la"), None, true).best(), None);
        assert_eq!(lyrics(None, None, false).best(), None);
    }

    #[test]
    fn query_values_are_encoded() {
        assert_eq!(urlencode("Sigur Rós"), "Sigur+R%C3%B3s");
        assert_eq!(urlencode("A&B"), "A%26B");
    }
}
