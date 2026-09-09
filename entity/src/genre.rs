use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// One genre in the library.
///
/// The id is `md5` of the normalised name, so the same genre found in two
/// places is one row and a rescan does not duplicate it.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, Default)]
#[sea_orm(table_name = "genre")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(unique)]
    pub name: String,
    /// How many tracks reach this genre, by either route. Not a column —
    /// counted per query, because it changes with every scan.
    #[sea_orm(ignore)]
    pub track_count: u32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// `md5` of the name, lowercased and trimmed: "Hip Hop", "hip hop" and
/// " Hip  Hop " are the same genre, and tags disagree about all three.
pub fn id_for(name: &str) -> String {
    format!("{:x}", md5::compute(normalize(name)))
}

/// The comparison form of a genre name.
pub fn normalize(name: &str) -> String {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Split a free-text genre tag into individual genres.
///
/// Tags carry more than one in a single string, and no two taggers agree on
/// the separator: `;`, `/`, and `,` all appear in the wild.
pub fn split_tag(tag: &str) -> Vec<String> {
    tag.split([';', '/', ','])
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .map(|part| part.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_genre_written_differently_is_one_id() {
        let canonical = id_for("Hip Hop");
        for variant in ["hip hop", "HIP HOP", "  Hip   Hop  "] {
            assert_eq!(id_for(variant), canonical, "{variant} differed");
        }
        assert_ne!(id_for("Hip Hop"), id_for("Trip Hop"));
    }

    /// A tag is one string holding several genres, and taggers disagree about
    /// the separator.
    #[test]
    fn a_tag_splits_into_its_genres() {
        assert_eq!(split_tag("Rock"), vec!["Rock"]);
        assert_eq!(split_tag("Rock; Metal"), vec!["Rock", "Metal"]);
        assert_eq!(split_tag("Rock/Metal"), vec!["Rock", "Metal"]);
        assert_eq!(split_tag("Rock, Metal"), vec!["Rock", "Metal"]);
    }

    /// An empty or separator-only tag contributes nothing rather than a blank
    /// genre that would show as an unnamed row.
    #[test]
    fn an_empty_tag_yields_nothing() {
        assert!(split_tag("").is_empty());
        assert!(split_tag("  ").is_empty());
        assert!(split_tag(" ; / , ").is_empty());
    }
}
