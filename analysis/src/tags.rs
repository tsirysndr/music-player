//! Key and tempo as the file itself states them.
//!
//! Preferred over detection whenever a file has them, because a tag is almost
//! always a *better* answer than a re-derivation: it was written by a tool that
//! analysed the file deliberately, or by a person who knows the track. Mixxx,
//! Rekordbox, Traktor and Serato all read the tag first for the same reason,
//! which is also why a library that has been through one of them agrees with
//! itself everywhere except here.
//!
//! It matters most for the *mode*. Correlation-based key detection picks the
//! tonic reliably and the major/minor far less so — the two profiles for one
//! tonic look alike — and confusing them is not a small error: F major and
//! F minor sit at opposite ends of the circle of fifths. A tag says which
//! outright.

use symphonia::core::meta::{MetadataRevision, StandardTagKey};

/// What a file says about itself.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tags {
    /// Traditional notation, e.g. `"Fm"`, whatever notation the tag used.
    pub key: Option<String>,
    pub bpm: Option<f32>,
}

/// Read key and tempo from a metadata revision.
pub fn read(revision: &MetadataRevision) -> Tags {
    let mut tags = Tags::default();

    for tag in revision.tags() {
        if tags.bpm.is_none() && matches!(tag.std_key, Some(StandardTagKey::Bpm)) {
            tags.bpm = parse_bpm(&tag.value.to_string());
            continue;
        }
        // Key has no standard variant in symphonia, so it is matched by the
        // name the format actually uses: `initialkey` in mp4 and Vorbis,
        // `TKEY` in id3.
        if tags.key.is_none() && is_key_tag(&tag.key) {
            tags.key = key_from_tag(&tag.value.to_string());
        }
    }

    tags
}

fn is_key_tag(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    // mp4 freeform atoms arrive namespaced — `com.apple.iTunes:initialkey` —
    // so the vendor prefix is dropped before matching. Without this the tag
    // every DJ tool writes into an m4a is invisible, which is precisely how a
    // track tagged `4A` came back detected as its parallel major.
    let name = name.rsplit(':').next().unwrap_or(&name);
    matches!(name, "initialkey" | "tkey" | "key" | "initial key")
}

/// A tempo tag, if it is a plausible one.
///
/// Zero is what a tool writes when it has not analysed a file, and reading it
/// as a tempo would put every unanalysed track at the bottom of a tempo sort.
fn parse_bpm(value: &str) -> Option<f32> {
    let bpm: f32 = value.trim().parse().ok()?;
    (bpm.is_finite() && (20.0..=300.0).contains(&bpm)).then_some(bpm)
}

/// A key tag, normalised to traditional notation.
///
/// Tags are written in both spellings and either is accepted: DJ tools write
/// Camelot (`4A`), taggers and people write musical (`Fm`, `F minor`, `Abm`,
/// or a bare `F` for the major). Both name the same key, so both are stored the
/// same way.
pub fn key_from_tag(value: &str) -> Option<String> {
    crate::key::Key::parse(value).map(|key| key.name())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Both spellings appear in real libraries and both name the same key, so
    /// both are stored the same way. These are the two tags that started this:
    /// one file says `4A`, another says `10B`, and the user's other tools show
    /// them as `Fm` and `D`.
    #[test]
    fn either_spelling_is_stored_traditionally() {
        assert_eq!(key_from_tag("4A"), Some("Fm".into()));
        assert_eq!(key_from_tag("10B"), Some("D".into()));
        assert_eq!(key_from_tag("Fm"), Some("Fm".into()));
        assert_eq!(key_from_tag("F minor"), Some("Fm".into()));
        // A bare letter is the major, and a long way round the wheel from its
        // parallel minor — which is why the mode has to be right.
        assert_eq!(key_from_tag("F"), Some("F".into()));
        assert_eq!(key_from_tag("Abm"), Some("G#m".into()));
    }

    /// Anything unrecognised is left alone rather than guessed at — a wrong key
    /// is worse than none, because it is acted on.
    #[test]
    fn nonsense_is_not_a_key() {
        for input in ["", "  ", "Hm", "42", "F lydian", "unknown", "13A"] {
            assert_eq!(key_from_tag(input), None, "{input:?}");
        }
    }

    /// Zero is what a tool writes when it has not analysed a file.
    #[test]
    fn an_implausible_tempo_is_no_tempo() {
        assert_eq!(parse_bpm("0"), None);
        assert_eq!(parse_bpm(""), None);
        assert_eq!(parse_bpm("banana"), None);
        assert_eq!(parse_bpm("9999"), None);

        assert_eq!(parse_bpm("162"), Some(162.0));
        assert_eq!(parse_bpm(" 128.5 "), Some(128.5));
    }

    #[test]
    fn key_tags_are_recognised_by_every_name_they_use() {
        for name in ["initialkey", "INITIALKEY", "TKEY", "key", "Initial Key"] {
            assert!(is_key_tag(name), "{name}");
        }
        // How an m4a actually presents it, and the case that was being missed.
        assert!(is_key_tag("com.apple.iTunes:initialkey"));
        assert!(is_key_tag("COM.APPLE.ITUNES:INITIALKEY"));

        assert!(!is_key_tag("keywords"));
        assert!(!is_key_tag("title"));
        assert!(!is_key_tag("com.apple.iTunes:UPC"));
    }
}
