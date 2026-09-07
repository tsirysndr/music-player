//! Adds `ext:mood` and `ext:era` to the smart-playlist vocabulary.
//!
//! Filters can then say things the built-in schema cannot:
//!
//! ```text
//! ext:mood==energetic;year>2015
//! ext:era==nineties,ext:era==eighties
//! ```
//!
//! What to copy from this example:
//!
//! * declaring predicates in `predicates()`, with example values so a picker
//!   can offer them,
//! * honouring the operator, not just the value — `!=` has to work too,
//! * keeping `evaluate` cheap. It runs once per candidate track, so this one
//!   derives its answer from the track it is handed and makes no request at
//!   all. A predicate that called an API per track would be unusable on a
//!   library of any size; cache aggressively if you must.
//!
//! The mood here is derived from genre keywords — a stand-in for the audio
//! analysis or service lookup a real one would do.
//!
//! Build with `cargo build --release --target wasm32-wasip1`.

mod pdk;

use extism_pdk::*;
use pdk::types;

/// Genre keywords that suggest each mood. First match wins, so the more
/// specific moods are listed before the broader ones.
const MOODS: &[(&str, &[&str])] = &[
    (
        "energetic",
        &["punk", "metal", "hardcore", "drum and bass", "techno", "dance", "house"],
    ),
    (
        "calm",
        &["ambient", "classical", "chill", "lofi", "acoustic", "folk"],
    ),
    ("melancholy", &["blues", "sad", "shoegaze", "slowcore", "doom"]),
    ("upbeat", &["pop", "disco", "funk", "ska", "soul"]),
    ("focused", &["instrumental", "post-rock", "minimal", "study"]),
];

/// The decade names `ext:era` accepts.
const ERAS: &[(&str, u32, u32)] = &[
    ("sixties", 1960, 1969),
    ("seventies", 1970, 1979),
    ("eighties", 1980, 1989),
    ("nineties", 1990, 1999),
    ("twothousands", 2000, 2009),
    ("twentytens", 2010, 2019),
    ("twentytwenties", 2020, 2029),
];

pub(crate) fn predicates() -> Result<Vec<types::PredicateSpec>, Error> {
    Ok(vec![
        types::PredicateSpec {
            name: "mood".to_string(),
            description: "The feel of a track, derived from its genre".to_string(),
            values: MOODS.iter().map(|(mood, _)| mood.to_string()).collect(),
        },
        types::PredicateSpec {
            name: "era".to_string(),
            description: "The decade a track was released in".to_string(),
            values: ERAS.iter().map(|(era, _, _)| era.to_string()).collect(),
        },
    ])
}

pub(crate) fn evaluate(input: types::PredicateRequest) -> Result<types::PredicateResponse, Error> {
    let holds = match input.name.as_str() {
        "mood" => mood_of(&input.track.genre)
            .map(|mood| mood == input.value.to_ascii_lowercase())
            .unwrap_or(false),
        "era" => era_of(input.track.year)
            .map(|era| era == input.value.to_ascii_lowercase())
            .unwrap_or(false),
        // A predicate this build does not know matches nothing, rather than
        // matching everything and quietly widening the playlist.
        _ => false,
    };

    // The operator matters: `ext:mood!=calm` has to be the complement of
    // `ext:mood==calm`, or half the vocabulary silently does the wrong thing.
    let matches = match input.op.as_str() {
        "!=" => !holds,
        // `==` and anything else (a comparison that makes no sense for a
        // categorical value) read as equality.
        _ => holds,
    };
    Ok(types::PredicateResponse { matches })
}

/// The mood a genre suggests, if any.
fn mood_of(genre: &str) -> Option<&'static str> {
    let genre = genre.to_ascii_lowercase();
    if genre.trim().is_empty() {
        return None;
    }
    MOODS
        .iter()
        .find(|(_, keywords)| keywords.iter().any(|keyword| genre.contains(keyword)))
        .map(|(mood, _)| *mood)
}

/// The decade name for a year.
fn era_of(year: Option<i32>) -> Option<&'static str> {
    let year = u32::try_from(year?).ok()?;
    ERAS.iter()
        .find(|(_, from, to)| year >= *from && year <= *to)
        .map(|(era, _, _)| *era)
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

    fn track(genre: &str, year: Option<i32>) -> types::Track {
        types::Track {
            id: "t1".into(),
            title: "Test".into(),
            artist: "Someone".into(),
            album: String::new(),
            album_artist: String::new(),
            genre: genre.into(),
            year,
            track_number: None,
            duration: 0.0,
            uri: String::new(),
        }
    }

    fn ask(name: &str, op: &str, value: &str, track: types::Track) -> bool {
        evaluate(types::PredicateRequest {
            name: name.into(),
            op: op.into(),
            value: value.into(),
            track,
        })
        .unwrap()
        .matches
    }

    #[test]
    fn derives_a_mood_from_the_genre() {
        assert_eq!(mood_of("Ambient"), Some("calm"));
        assert_eq!(mood_of("drum and bass"), Some("energetic"));
        // Substring, so "post-punk" still reads as punk.
        assert_eq!(mood_of("post-punk revival"), Some("energetic"));
        assert_eq!(mood_of(""), None);
        assert_eq!(mood_of("polka"), None);
    }

    #[test]
    fn maps_years_onto_decades() {
        assert_eq!(era_of(Some(1994)), Some("nineties"));
        assert_eq!(era_of(Some(2020)), Some("twentytwenties"));
        assert_eq!(era_of(None), None);
        // Outside the table, and not a year at all.
        assert_eq!(era_of(Some(1850)), None);
        assert_eq!(era_of(Some(-5)), None);
    }

    /// `!=` has to be the complement of `==`, or half the vocabulary silently
    /// does the wrong thing.
    #[test]
    fn the_operator_is_honoured() {
        let calm = track("ambient", None);
        assert!(ask("mood", "==", "calm", calm.clone()));
        assert!(!ask("mood", "!=", "calm", calm.clone()));
        assert!(!ask("mood", "==", "energetic", calm.clone()));
        assert!(ask("mood", "!=", "energetic", calm));
    }

    /// A track with no genre has no mood — and must not match `==`.
    #[test]
    fn an_unknown_value_does_not_match() {
        assert!(!ask("mood", "==", "calm", track("", None)));
        assert!(!ask("era", "==", "nineties", track("rock", None)));
        // Nor does a predicate this build does not provide.
        assert!(!ask("bpm", "==", "120", track("rock", Some(1994))));
    }

    /// Every value offered in the picker has to be one `evaluate` can return,
    /// or the UI would suggest filters that match nothing.
    #[test]
    fn every_advertised_value_is_reachable() {
        for spec in predicates().unwrap() {
            for value in &spec.values {
                let reachable = match spec.name.as_str() {
                    "mood" => MOODS.iter().any(|(mood, _)| mood == value),
                    "era" => ERAS.iter().any(|(era, _, _)| era == value),
                    _ => false,
                };
                assert!(reachable, "{}=={value} is offered but unreachable", spec.name);
            }
        }
    }
}
