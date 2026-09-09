use async_graphql::*;
use music_player_entity::genre::Model;
use music_player_types::types::Genre as GenreType;

/// A genre in the library.
#[derive(Default, Clone)]
pub struct Genre {
    pub id: ID,
    pub name: String,
    /// How many tracks reach it, by either route — the file's own tag or its
    /// artist's. Zero means unknown rather than empty when a remote server
    /// does not report one.
    pub track_count: u32,
}

#[Object]
impl Genre {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn name(&self) -> &String {
        &self.name
    }

    async fn track_count(&self) -> u32 {
        self.track_count
    }
}

impl From<Model> for Genre {
    fn from(model: Model) -> Self {
        Self {
            id: ID(model.id),
            name: model.name,
            track_count: model.track_count,
        }
    }
}

impl From<GenreType> for Genre {
    fn from(genre: GenreType) -> Self {
        Self {
            id: ID(genre.id),
            name: genre.name,
            track_count: genre.track_count,
        }
    }
}
