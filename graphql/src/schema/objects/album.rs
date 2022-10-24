use async_graphql::*;

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
