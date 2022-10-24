use async_graphql::Object;

use super::track::Track;

#[derive(Default, Clone)]
pub struct CurrentlyPlayingSong {
    pub track: Option<Track>,
    pub index: u32,
    pub position_ms: u32,
    pub is_playing: bool,
}

#[Object]
impl CurrentlyPlayingSong {
    async fn track(&self) -> Option<Track> {
        self.track.clone()
    }

    async fn index(&self) -> u32 {
        self.index
    }

    async fn position_ms(&self) -> u32 {
        self.position_ms
    }

    async fn is_playing(&self) -> bool {
        self.is_playing
    }
}
