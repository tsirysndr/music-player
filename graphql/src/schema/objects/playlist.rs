use async_graphql::*;
use music_player_entity::{playlist::Model, select_result};
use music_player_types::types::{Playlist as PlaylistType, RemoteTrackUrl};

use super::track::Track;

#[derive(Default, Clone)]
pub struct Playlist {
    pub id: ID,
    pub name: String,
    pub description: Option<String>,
    pub tracks: Vec<Track>,
    /// A smart playlist refills itself from `rsql` rather than holding a
    /// hand-picked list. Everything else about it behaves the same.
    pub is_smart: bool,
    pub rsql: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub max_tracks: Option<u32>,
    /// How many tracks it has.
    ///
    /// A listing reports a count without sending the entries, so counting
    /// `tracks` gives zero there — which is what every row used to show.
    pub track_count: u32,
}

#[Object]
impl Playlist {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn description(&self) -> &Option<String> {
        &self.description
    }

    async fn tracks(&self) -> &Vec<Track> {
        &self.tracks
    }

    async fn is_smart(&self) -> bool {
        self.is_smart
    }

    /// The RSQL filter behind a smart playlist, e.g. `genre==rock;year>2000`.
    async fn rsql(&self) -> &Option<String> {
        &self.rsql
    }

    async fn sort_by(&self) -> &Option<String> {
        &self.sort_by
    }

    async fn sort_order(&self) -> &Option<String> {
        &self.sort_order
    }

    async fn max_tracks(&self) -> Option<u32> {
        self.max_tracks
    }

    /// Never zero for a playlist that has tracks, whether or not this
    /// response carried them.
    async fn track_count(&self) -> u32 {
        self.track_count.max(self.tracks.len() as u32)
    }
}

impl From<Model> for Playlist {
    fn from(model: Model) -> Self {
        Self {
            id: ID(model.id),
            name: model.name,
            description: model.description,
            track_count: model.tracks.len() as u32,
            tracks: model.tracks.into_iter().map(Track::from).collect(),
            is_smart: model.is_smart,
            rsql: model.rsql,
            sort_by: model.sort_by,
            sort_order: model.sort_order,
            max_tracks: model.max_tracks,
        }
    }
}

impl From<Vec<select_result::PlaylistTrack>> for Playlist {
    fn from(result: Vec<select_result::PlaylistTrack>) -> Self {
        if result.is_empty() {
            return Self::default();
        }
        Self {
            id: ID(result[0].id.clone()),
            name: result[0].name.clone(),
            description: result[0].description.clone(),
            tracks: result.into_iter().map(Track::from).collect(),
            // The joined row carries no playlist columns beyond these three;
            // callers that need the smart fields read the playlist row itself.
            ..Default::default()
        }
    }
}

impl From<PlaylistType> for Playlist {
    fn from(playlist: PlaylistType) -> Self {
        // Taken before the fields move out from under it.
        let track_count = playlist.len();
        Self {
            id: ID(playlist.id),
            name: playlist.name,
            description: playlist.description,
            track_count,
            tracks: playlist.tracks.into_iter().map(Into::into).collect(),
            // A playlist from a remote source (Subsonic, Jellyfin, DLNA) is
            // never smart — those servers have no notion of one.
            ..Default::default()
        }
    }
}

impl RemoteTrackUrl for Playlist {
    fn with_remote_track_url(&self, base_url: &str) -> Self {
        Self {
            tracks: self
                .tracks
                .clone()
                .into_iter()
                .map(|track| track.with_remote_track_url(base_url))
                .collect(),
            ..self.clone()
        }
    }
}
