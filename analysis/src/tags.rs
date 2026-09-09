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
//! tonic look alike — and confusing them is not a small error: F major is 7B
//! and F minor is 4A, which sit at opposite ends of the wheel. A track tagged
//! `4A` says so outright.

use symphonia::core::meta::{MetadataRevision, StandardTagKey};

/// What a file says about itself.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tags {
    /// Camelot notation, e.g. `"4A"`, whatever notation the tag used.
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
            tags.key = camelot_from_tag(&tag.value.to_string());
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

/// A key tag in Camelot notation, whatever notation it was written in.
///
/// Three spellings are common and all appear in real libraries: Camelot
/// (`4A`), which DJ tools write; musical (`Fm`, `F minor`, `Abmaj`); and the
/// plain letter for a major key (`F`). Anything else is left alone rather than
/// guessed at.
pub fn camelot_from_tag(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    // Already Camelot — the common case for a library that has been through a
    // DJ tool, and the one that must survive untouched.
    if crate::key_color::rgb_for(value).is_some() {
        return Some(value.to_uppercase());
    }

    let (root, rest) = parse_root(value)?;
    let rest = rest.trim().to_ascii_lowercase();
    // A bare letter means major, as every tool that writes one intends.
    let minor = match rest.as_str() {
        "" | "maj" | "major" | "m*" => false,
        "m" | "min" | "minor" => true,
        _ => return None,
    };

    Some(crate::features::camelot(root, !minor))
}

/// The pitch class a key name starts with, and what follows it.
fn parse_root(value: &str) -> Option<(u8, &str)> {
    let mut chars = value.chars();
    let letter = chars.next()?.to_ascii_uppercase();
    let natural = match letter {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };

    let rest = chars.as_str();
    // Accidentals, in both the spellings tags use.
    if let Some(rest) = rest.strip_prefix(['#', '♯']) {
        return Some(((natural + 1) % 12, rest));
    }
    if let Some(rest) = rest.strip_prefix(['b', '♭']) {
        // `b` is ambiguous: "Bb" is a flat, but "Bb" could also be read as B
        // followed by nothing. Treating it as a flat is right — no notation
        // writes a mode as a bare `b`.
        return Some(((natural + 11) % 12, rest));
    }
    Some((natural, rest))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Camelot passes through: a library tagged by a DJ tool is already saying
    /// exactly what we want to store.
    #[test]
    fn camelot_survives_untouched() {
        assert_eq!(camelot_from_tag("4A"), Some("4A".into()));
        assert_eq!(camelot_from_tag("11B"), Some("11B".into()));
        assert_eq!(camelot_from_tag("8a"), Some("8A".into()));
    }

    /// Musical notation converts. `Fm` is the case that started this: the
    /// detector called it F major, and the tag says minor.
    #[test]
    fn musical_notation_converts_to_camelot() {
        assert_eq!(camelot_from_tag("Fm"), Some("4A".into()));
        assert_eq!(camelot_from_tag("F minor"), Some("4A".into()));
        assert_eq!(camelot_from_tag("Fmin"), Some("4A".into()));
        // The parallel major is a different key, and a long way away on the
        // wheel — which is why getting the mode right matters.
        assert_eq!(camelot_from_tag("F"), Some("7B".into()));
        assert_eq!(camelot_from_tag("F major"), Some("7B".into()));
    }

    #[test]
    fn accidentals_are_understood() {
        // A minor is 8A, so A-flat minor is a fifth away.
        assert_eq!(camelot_from_tag("Abm"), Some("1A".into()));
        assert_eq!(camelot_from_tag("G#m"), Some("1A".into()));
        assert_eq!(camelot_from_tag("Bb"), Some("6B".into()));
        assert_eq!(camelot_from_tag("C#"), Some("3B".into()));
    }

    /// Anything unrecognised is left alone rather than guessed at — a wrong key
    /// is worse than none, because it is acted on.
    #[test]
    fn nonsense_is_not_a_key() {
        for input in ["", "  ", "Hm", "42", "F lydian", "unknown", "13A"] {
            assert_eq!(camelot_from_tag(input), None, "{input:?}");
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
