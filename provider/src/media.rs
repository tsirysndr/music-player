//! Metadata for navigation, separate from a playable queue entry.

use crate::Track;

#[derive(Clone, Debug)]
pub struct MediaEntry {
    /// Stable, token-free source handle. Containers use ResourceKind::Container;
    /// leaves use ResourceKind::Item. Only `track: Some` offers queue metadata.
    pub id: String,
    pub title: String,
    /// Original server values, including types this client does not recognize.
    pub item_type: Option<String>,
    pub media_type: Option<String>,
    pub is_container: bool,
    /// The original Season.IndexNumber or Episode.ParentIndexNumber.
    pub season_number: Option<u64>,
    pub episode_number: Option<u64>,
    /// None for containers and non-Audio/Video leaves. Some is metadata only;
    /// audio stream availability is checked by the separate playback resolver.
    pub track: Option<Track>,
}
