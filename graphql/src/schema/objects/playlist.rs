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
}

impl From<Model> for Playlist {
    fn from(model: Model) -> Self {
        Self {
            id: ID(model.id),
            name: model.name,
            description: model.description,
            tracks: model.tracks.into_iter().map(Track::from).collect(),
        }
    }
}

impl From<Vec<select_result::PlaylistTrack>> for Playlist {
    fn from(result: Vec<select_result::PlaylistTrack>) -> Self {
        if result.len() == 0 {
            return Self::default();
        }
        Self {
            id: ID(result[0].id.clone()),
            name: result[0].name.clone(),
            description: result[0].description.clone(),
            tracks: result.into_iter().map(Track::from).collect(),
        }
    }
}

impl From<PlaylistType> for Playlist {
    fn from(playlist: PlaylistType) -> Self {
        Self {
            id: ID(playlist.id),
            name: playlist.name,
            description: playlist.description,
            tracks: playlist.tracks.into_iter().map(Into::into).collect(),
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
