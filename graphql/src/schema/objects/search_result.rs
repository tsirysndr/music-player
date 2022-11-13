use async_graphql::*;

use super::{album::Album, artist::Artist, track::Track};

#[derive(Default, Clone)]
pub struct SearchResult {
    pub artists: Vec<Artist>,
    pub tracks: Vec<Track>,
    pub albums: Vec<Album>,
}

#[Object]
impl SearchResult {
    async fn artists(&self) -> Vec<Artist> {
        self.artists.clone()
    }

    async fn tracks(&self) -> Vec<Track> {
        self.tracks.clone()
    }

    async fn albums(&self) -> Vec<Album> {
        self.albums.clone()
    }
}
