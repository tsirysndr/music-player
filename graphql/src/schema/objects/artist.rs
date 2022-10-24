use async_graphql::*;

use super::{album::Album, track::Track};

#[derive(Default, Clone)]
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
