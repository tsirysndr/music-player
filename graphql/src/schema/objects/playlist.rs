use async_graphql::*;

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
