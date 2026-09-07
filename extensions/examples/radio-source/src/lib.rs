//! A media source backed by the Radio Browser directory.
//!
//! Browsing a directory of internet radio stations is a good fit for the
//! `source` capability: every station is one playable track, the catalogue is
//! far too large to import into the library, and the stream url is best
//! resolved at play time.
//!
//! What to copy from this example:
//!
//! * the `source_info` / `list_*` / `get_stream_url` shape a source implements,
//! * reading configuration the manifest declared, via `get_config`,
//! * making HTTP requests to a host the manifest allow-listed.
//!
//! Build with `cargo build --release --target wasm32-wasip1`.

mod pdk;

use extism_pdk::*;
use pdk::types;

/// Radio Browser mirror. Overridable through the manifest's config, so a user
/// can point this at a different mirror without a rebuild — and so this file
/// shows how to read config at all.
const DEFAULT_API: &str = "https://de1.api.radio-browser.info";

/// How many stations to ask for when the host does not say.
const DEFAULT_LIMIT: u32 = 50;

/// One station, as Radio Browser returns it.
#[derive(serde::Deserialize)]
struct Station {
    stationuuid: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    url_resolved: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    tags: String,
    #[serde(default)]
    codec: String,
    #[serde(default)]
    homepage: String,
}

impl Station {
    /// The playable url. `url_resolved` is the one Radio Browser has already
    /// followed through any redirects; `url` is what the submitter typed.
    fn stream(&self) -> &str {
        if self.url_resolved.is_empty() {
            &self.url
        } else {
            &self.url_resolved
        }
    }

    fn into_track(self) -> types::Track {
        let genre = self
            .tags
            .split(',')
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();
        types::Track {
            id: self.stationuuid,
            title: self.name.trim().to_string(),
            // A station has no artist; naming the source reads better in the
            // player than an empty line.
            artist: "Radio Browser".to_string(),
            album: String::new(),
            album_artist: String::new(),
            genre,
            year: None,
            track_number: None,
            // A live stream has no length. Zero is how the player knows.
            duration: 0.0,
            uri: self.url_resolved.clone(),
        }
    }
}

fn api_base() -> String {
    // An undeclared or unset key reads as empty, so fall back rather than
    // building a request against "".
    let configured = pdk::get_config("apiUrl".to_string()).unwrap_or_default();
    let base = if configured.trim().is_empty() {
        DEFAULT_API.to_string()
    } else {
        configured
    };
    base.trim_end_matches('/').to_string()
}

/// Fetch and decode a list of stations.
fn fetch(path: &str) -> Result<Vec<Station>, Error> {
    let url = format!("{}{path}", api_base());
    let request = HttpRequest::new(&url)
        .with_method("GET")
        // Radio Browser asks clients to identify themselves.
        .with_header("User-Agent", "music-player-radio-source/0.1");
    let response = http::request::<()>(&request, None)?;
    if response.status_code() != 200 {
        // Not an error worth failing the browse over — an empty page is a
        // better answer to the user than a broken source.
        pdk::log(format!(
            "radio-browser answered {} for {url}",
            response.status_code()
        ))?;
        return Ok(Vec::new());
    }
    Ok(serde_json::from_slice(&response.body())?)
}

fn limit_of(request: &types::BrowseRequest) -> u32 {
    if request.limit > 0 {
        request.limit as u32
    } else {
        DEFAULT_LIMIT
    }
}

fn offset_of(request: &types::BrowseRequest) -> u32 {
    request.offset.max(0) as u32
}

pub(crate) fn source_info() -> Result<types::SourceInfo, Error> {
    Ok(types::SourceInfo {
        name: "Radio Browser".to_string(),
        description: "Internet radio stations from the Radio Browser directory".to_string(),
        // The directory is public: no credentials to ask for.
        requires_auth: false,
        supports_search: true,
    })
}

/// Tracks are stations. A `query` searches by name; without one, the most
/// clicked stations are a far better landing page than an arbitrary slice.
pub(crate) fn list_tracks(input: types::BrowseRequest) -> Result<Vec<types::Track>, Error> {
    let limit = limit_of(&input);
    let offset = offset_of(&input);
    let query = input.query;

    let path = if query.trim().is_empty() {
        format!(
            "/json/stations/search?limit={limit}&offset={offset}\
             &hidebroken=true&order=clickcount&reverse=true"
        )
    } else {
        format!(
            "/json/stations/search?limit={limit}&offset={offset}\
             &hidebroken=true&order=clickcount&reverse=true&name={}",
            urlencode(&query)
        )
    };
    Ok(fetch(&path)?.into_iter().map(Station::into_track).collect())
}

/// Genres stand in for albums: a station has no album, but "browse by tag" is
/// the shape a directory this size actually wants.
pub(crate) fn list_albums(input: types::BrowseRequest) -> Result<Vec<types::SourceAlbum>, Error> {
    // Listing a genre's stations is `list_tracks` with a parent; the album
    // list itself is the fixed set of tags worth offering.
    let _ = input;
    Ok([
        "jazz", "rock", "classical", "electronic", "ambient", "news", "pop", "reggae",
    ]
    .iter()
    .map(|tag| types::SourceAlbum {
        id: (*tag).to_string(),
        title: title_case(tag),
        artist: "Radio Browser".to_string(),
        year: None,
        cover_url: String::new(),
    })
    .collect())
}

/// A directory of stations has no artists to speak of.
pub(crate) fn list_artists(_input: types::BrowseRequest) -> Result<Vec<types::SourceArtist>, Error> {
    Ok(Vec::new())
}

/// Nor playlists.
pub(crate) fn list_playlists(
    _input: types::BrowseRequest,
) -> Result<Vec<types::SourcePlaylist>, Error> {
    Ok(Vec::new())
}

/// Resolve a station id to its stream, at play time — a station's url can
/// change, and looking it up now rather than at browse time keeps it fresh.
pub(crate) fn get_stream_url(input: types::StreamRequest) -> Result<types::StreamResponse, Error> {
    let stations = fetch(&format!("/json/stations/byuuid/{}", input.track_id))?;
    let Some(station) = stations.into_iter().next() else {
        return Ok(types::StreamResponse {
            url: String::new(),
            mime_type: String::new(),
            headers: String::new(),
        });
    };
    let mime = match station.codec.to_ascii_uppercase().as_str() {
        "MP3" => "audio/mpeg",
        "AAC" | "AAC+" => "audio/aac",
        "OGG" => "audio/ogg",
        "FLAC" => "audio/flac",
        _ => "",
    };
    let _ = station.homepage;
    Ok(types::StreamResponse {
        url: station.stream().to_string(),
        mime_type: mime.to_string(),
        headers: String::new(),
    })
}

// ── The other capabilities ──────────────────────────────────────────────────
//
// `pdk.rs` is generated from the whole schema, so the module exports every
// function in it. That is harmless: the host calls an export only when the
// manifest declares the matching capability, and `plugin.toml` here declares
// `source` alone. These stubs exist to satisfy the linker, not to be called.

pub(crate) fn commands() -> Result<Vec<types::CommandSpec>, Error> {
    Ok(Vec::new())
}

pub(crate) fn run_command(_input: types::CommandRequest) -> Result<types::CommandResponse, Error> {
    Ok(types::CommandResponse {
        ok: false,
        message: "this extension provides no commands".to_string(),
        data: String::new(),
    })
}

pub(crate) fn predicates() -> Result<Vec<types::PredicateSpec>, Error> {
    Ok(Vec::new())
}

pub(crate) fn evaluate(_input: types::PredicateRequest) -> Result<types::PredicateResponse, Error> {
    Ok(types::PredicateResponse { matches: false })
}

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

/// Percent-encode a query value. Small enough to write out rather than pull in
/// a crate for — a WebAssembly module pays for every dependency in bytes.
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

fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn station_prefers_the_resolved_url() {
        let station = Station {
            stationuuid: "abc".into(),
            name: "Test".into(),
            url_resolved: "https://resolved.example/stream".into(),
            url: "https://submitted.example/stream".into(),
            tags: String::new(),
            codec: String::new(),
            homepage: String::new(),
        };
        assert_eq!(station.stream(), "https://resolved.example/stream");
    }

    #[test]
    fn station_falls_back_to_the_submitted_url() {
        let station = Station {
            stationuuid: "abc".into(),
            name: "Test".into(),
            url_resolved: String::new(),
            url: "https://submitted.example/stream".into(),
            tags: String::new(),
            codec: String::new(),
            homepage: String::new(),
        };
        assert_eq!(station.stream(), "https://submitted.example/stream");
    }

    /// A live stream has no length, and the player reads a zero duration as
    /// exactly that.
    #[test]
    fn a_station_becomes_a_track_with_no_duration() {
        let station = Station {
            stationuuid: "abc".into(),
            name: "  Jazz FM  ".into(),
            url_resolved: "https://example/stream".into(),
            url: String::new(),
            tags: "jazz,smooth".into(),
            codec: "MP3".into(),
            homepage: String::new(),
        };
        let track = station.into_track();
        assert_eq!(track.title, "Jazz FM");
        assert_eq!(track.genre, "jazz");
        assert_eq!(track.duration, 0.0);
    }

    #[test]
    fn query_values_are_encoded() {
        assert_eq!(urlencode("smooth jazz"), "smooth+jazz");
        assert_eq!(urlencode("rock&roll"), "rock%26roll");
    }
}
