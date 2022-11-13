use async_graphql::*;
use music_player_entity::album::Model;
use music_player_types::types::Album as AlbumType;

use super::track::Track;

#[derive(Default, Clone)]
pub struct Album {
    pub id: ID,
    pub title: String,
    pub cover: Option<String>,
    pub release_date: String,
    pub artist: String,
    pub year: Option<u32>,
    pub genres: Vec<String>,
    pub tracks: Vec<Track>,
}

#[Object]
impl Album {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn title(&self) -> &str {
        &self.title
    }

    async fn cover(&self) -> &Option<String> {
        &self.cover
    }

    async fn release_date(&self) -> &str {
        &self.release_date
    }

    async fn artist(&self) -> &str {
        &self.artist
    }

    async fn year(&self) -> Option<u32> {
        self.year
    }

    async fn genres(&self) -> Vec<String> {
        self.genres.clone()
    }

    async fn tracks(&self) -> Vec<Track> {
        self.tracks.clone()
    }
}

impl From<Model> for Album {
    fn from(model: Model) -> Self {
        Self {
            id: ID(model.id),
            title: model.title,
            cover: model.cover,
            artist: model.artist,
            year: model.year,
            tracks: model.tracks.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

impl From<AlbumType> for Album {
    fn from(album: AlbumType) -> Self {
        Self {
            id: ID(album.id),
            title: album.title,
            cover: album.cover,
            artist: album.artist,
            ..Default::default()
        }
    }
}
