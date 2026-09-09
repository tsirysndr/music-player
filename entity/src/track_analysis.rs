use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// A stored analysis of one track.
///
/// Every measured field is nullable, and separately so. Extraction fails one
/// feature at a time — a track whose tempo cannot be found still has a
/// waveform, a loudness and a mood worth keeping — and a row that had to be
/// all-or-nothing would mean re-decoding the whole track to retry the one part
/// that failed.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, Default)]
#[sea_orm(table_name = "track_analysis")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub track_id: String,
    /// Copied from the library when the track was analysed, so matching has an
    /// artist and a title without a lookup per candidate.
    pub artist: Option<String>,
    pub title: Option<String>,
    /// Which library the track id belongs to. Empty is the local one.
    pub source: String,
    /// Peak per display bar, 0–255.
    pub waveform: Option<Vec<u8>>,
    pub bpm: Option<f32>,
    pub bpm_confidence: Option<f32>,
    pub valence: Option<f32>,
    pub arousal: Option<f32>,
    /// A json array of `[label, confidence]`.
    pub moods: Option<String>,
    pub lufs: Option<f32>,
    pub true_peak_db: Option<f32>,
    pub duration: Option<f32>,
    pub analyzed_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// The row id for a track in a library.
///
/// Derived rather than random so that analysing the same track twice replaces
/// the row instead of adding a second one — and so a caller can look a row up
/// without first searching for it.
///
/// The separator matters: without it, source `"ab"` + id `"c"` and source
/// `"a"` + id `"bc"` would hash the same, and a track from one server could
/// show another's waveform.
pub fn id_for(source: &str, track_id: &str) -> String {
    format!("{:x}", md5::compute(format!("{source}\0{track_id}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_id_is_stable_for_the_same_track() {
        assert_eq!(id_for("", "abc"), id_for("", "abc"));
    }

    /// The same track id on two servers is two different tracks.
    #[test]
    fn the_id_distinguishes_libraries() {
        assert_ne!(id_for("", "abc"), id_for("navidrome", "abc"));
    }

    /// The separator is what stops the two halves running together.
    #[test]
    fn the_boundary_between_source_and_track_is_unambiguous() {
        assert_ne!(id_for("ab", "c"), id_for("a", "bc"));
    }
}
