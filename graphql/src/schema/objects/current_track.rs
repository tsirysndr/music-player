use async_graphql::Object;
use music_player_types::types;

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

impl From<types::Playback> for CurrentlyPlayingSong {
    fn from(playback: types::Playback) -> Self {
        CurrentlyPlayingSong {
            track: playback.current_track.map(|track| track.into()),
            index: playback.index,
            position_ms: playback.position_ms,
            is_playing: playback.is_playing,
        }
    }
}
