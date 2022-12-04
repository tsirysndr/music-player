use super::{album::Album, track::Track};
use async_graphql::*;
use music_player_entity::artist::Model;
use music_player_types::types::Artist as ArtistType;
use serde::Serialize;

#[derive(Default, Clone, Serialize)]
pub struct Artist {
    pub id: ID,
    pub name: String,
    pub picture: String,
    pub bio: String,
    pub website: String,
    pub genres: Vec<String>,
    pub images: Vec<String>,
    pub albums: Vec<Album>,
    pub songs: Vec<Track>,
}

#[Object]
impl Artist {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn picture(&self) -> &str {
        &self.picture
    }

    async fn bio(&self) -> &str {
        &self.bio
    }

    async fn website(&self) -> &str {
        &self.website
    }

    async fn genres(&self) -> Vec<String> {
        self.genres.clone()
    }

    async fn images(&self) -> Vec<String> {
        self.images.clone()
    }

    async fn albums(&self) -> Vec<Album> {
        self.albums.clone()
    }

    async fn songs(&self) -> Vec<Track> {
        self.songs.clone()
    }
}

impl From<Model> for Artist {
    fn from(model: Model) -> Self {
        Self {
            id: ID(model.id),
            name: model.name,
            albums: model.albums.into_iter().map(Into::into).collect(),
            songs: model.tracks.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}
impl From<ArtistType> for Artist {
    fn from(artist: ArtistType) -> Self {
        Self {
            id: ID(artist.id),
            name: artist.name,
            ..Default::default()
        }
    }
}
