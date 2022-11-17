use async_graphql::*;
use music_player_entity::folder::Model;

use super::playlist::Playlist;

#[derive(Default, Clone)]
pub struct Folder {
    pub id: ID,
    pub name: String,
    pub playlists: Vec<Playlist>,
}

#[Object]
impl Folder {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn playlists(&self) -> Vec<Playlist> {
        self.playlists.clone()
    }
}

impl From<Model> for Folder {
    fn from(model: Model) -> Self {
        Self {
            id: ID(model.id),
            name: model.name,
            playlists: model.playlists.into_iter().map(Playlist::from).collect(),
        }
    }
}
