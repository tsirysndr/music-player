//! Library reports, as commands in the UI.
//!
//! Three commands that read the user's library and hand back a summary. What
//! to copy from this example:
//!
//! * declaring commands in `commands()` so the UI can list them before any of
//!   them runs,
//! * reading the library through the host functions, with RSQL filters,
//! * returning both a human line (`message`) and structured JSON (`data`), so
//!   a caller can display either.
//!
//! This is also the example to read for the library API: it touches tracks,
//! albums, playlists and saved radios.
//!
//! Build with `cargo build --release --target wasm32-wasip1`.

mod pdk;

use std::collections::BTreeMap;

use extism_pdk::*;
use pdk::types;

/// The commands this extension offers, as (name, title, description).
///
/// A single list so `commands()` and `run_command` cannot drift apart — a
/// command listed but not handled would show up in the UI and do nothing.
const COMMANDS: &[(&str, &str, &str)] = &[
    (
        "library_summary",
        "Library summary",
        "Counts of tracks, albums, playlists and saved radios",
    ),
    (
        "decades",
        "Tracks by decade",
        "How the library spreads across decades",
    ),
    (
        "unplayed",
        "Never played",
        "Tracks that have never been played",
    ),
];

pub(crate) fn commands() -> Result<Vec<types::CommandSpec>, Error> {
    Ok(COMMANDS
        .iter()
        .map(|(name, title, description)| types::CommandSpec {
            name: (*name).to_string(),
            title: (*title).to_string(),
            description: (*description).to_string(),
            needs_track: false,
        })
        .collect())
}

/// Whether `run_command` handles a name. Kept beside the dispatch so the two
/// are read together, and testable without a host to call into — the handlers
/// themselves call host functions, which only exist inside a WebAssembly host.
#[cfg_attr(not(test), allow(dead_code))]
fn handles(name: &str) -> bool {
    matches!(name, "library_summary" | "decades" | "unplayed")
}

pub(crate) fn run_command(input: types::CommandRequest) -> Result<types::CommandResponse, Error> {
    match input.name.as_str() {
        "library_summary" => library_summary(),
        "decades" => decades(),
        "unplayed" => unplayed(),
        // A command the UI asked for but this build does not know. Report it
        // rather than silently succeeding.
        other => Ok(types::CommandResponse {
            ok: false,
            message: format!("unknown command '{other}'"),
            data: String::new(),
        }),
    }
}

/// Counts across every part of the library.
fn library_summary() -> Result<types::CommandResponse, Error> {
    // An empty filter matches everything; limit 0 is no cap.
    let query = |filter: &str| types::LibraryQuery {
        filter: filter.to_string(),
        limit: 0,
    };

    let tracks = pdk::query_library(query(""))?.len();
    let albums = pdk::query_albums(query(""))?.len();
    let artists = pdk::query_artists(query(""))?.len();
    let playlists = pdk::query_playlists(query(""))?;
    let smart = playlists.iter().filter(|p| p.is_smart).count();
    let radios = pdk::get_saved_radios()?.len();

    let data = serde_json::json!({
        "tracks": tracks,
        "albums": albums,
        "artists": artists,
        "playlists": playlists.len(),
        "smartPlaylists": smart,
        "savedRadios": radios,
    });
    Ok(types::CommandResponse {
        ok: true,
        message: format!(
            "{tracks} tracks · {albums} albums · {artists} artists · \
             {} playlists ({smart} smart) · {radios} radios",
            playlists.len()
        ),
        data: data.to_string(),
    })
}

/// How the library spreads across decades.
fn decades() -> Result<types::CommandResponse, Error> {
    // `year=notnull=` skips the untagged, which would otherwise all land in a
    // meaningless "0s" bucket.
    let tracks = pdk::query_library(types::LibraryQuery {
        filter: "year=notnull=".to_string(),
        limit: 0,
    })?;

    let mut counts: BTreeMap<i32, usize> = BTreeMap::new();
    for track in &tracks {
        if let Some(year) = track.year.filter(|y| *y > 0) {
            *counts.entry(year - year % 10).or_default() += 1;
        }
    }

    let line = counts
        .iter()
        .map(|(decade, count)| format!("{decade}s: {count}"))
        .collect::<Vec<_>>()
        .join(" · ");
    let data: serde_json::Map<String, serde_json::Value> = counts
        .iter()
        .map(|(decade, count)| (format!("{decade}s"), serde_json::json!(count)))
        .collect();

    Ok(types::CommandResponse {
        ok: true,
        message: if line.is_empty() {
            "no tracks carry a year".to_string()
        } else {
            line
        },
        data: serde_json::Value::Object(data).to_string(),
    })
}

/// Tracks that have never been played — the corners of a library nobody visits.
fn unplayed() -> Result<types::CommandResponse, Error> {
    // A track with no `track_stats` row still matches `playcount==0`: the host
    // coalesces the missing row to zero.
    let tracks = pdk::query_library(types::LibraryQuery {
        filter: "playcount==0".to_string(),
        limit: 0,
    })?;

    let sample: Vec<String> = tracks
        .iter()
        .take(10)
        .map(|track| format!("{} — {}", track.artist, track.title))
        .collect();
    Ok(types::CommandResponse {
        ok: true,
        message: format!("{} tracks have never been played", tracks.len()),
        data: serde_json::json!({
            "count": tracks.len(),
            "sample": sample,
        })
        .to_string(),
    })
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

    /// A command listed but not handled would appear in the UI and do nothing,
    /// which is worse than not offering it.
    ///
    /// The handlers themselves call host functions, which only exist inside a
    /// WebAssembly host — so this checks the dispatch table rather than
    /// invoking them.
    #[test]
    fn every_listed_command_is_handled() {
        for spec in commands().unwrap() {
            assert!(handles(&spec.name), "{} is listed but not handled", spec.name);
        }
    }

    #[test]
    fn an_unhandled_name_is_rejected() {
        assert!(!handles("nope"));
        assert!(!handles(""));
    }

    /// Every command needs a title: it is what the UI shows.
    #[test]
    fn every_command_has_a_title() {
        for spec in commands().unwrap() {
            assert!(!spec.title.trim().is_empty(), "{} has no title", spec.name);
            assert!(!spec.name.trim().is_empty());
        }
    }
}
