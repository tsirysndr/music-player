use async_graphql::*;
use music_player_entity::playlist::Model;

use super::track::Track;

#[derive(Default, Clone)]
pub struct Playlist {
    pub id: ID,
    pub name: String,
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

    async fn tracks(&self) -> &Vec<Track> {
        &self.tracks
    }
}

impl From<Model> for Playlist {
    fn from(model: Model) -> Self {
        Self {
            id: ID(model.id),
            name: model.name,
            tracks: model.tracks.into_iter().map(Track::from).collect(),
        }
    }
}
