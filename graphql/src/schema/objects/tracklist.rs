use async_graphql::Object;

use super::track::Track;

#[derive(Default, Clone)]
pub struct Tracklist {
    pub next_tracks: Vec<Track>,
    pub previous_tracks: Vec<Track>,
}

#[Object]
impl Tracklist {
    async fn next_tracks(&self) -> Vec<Track> {
        self.next_tracks.clone()
    }

    async fn previous_tracks(&self) -> Vec<Track> {
        self.previous_tracks.clone()
    }
}
