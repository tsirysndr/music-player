use music_player_entity::track::Model as Track;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackEvent {
    pub track: Option<Track>,
    pub index: u32,
    pub is_playing: bool,
    pub position_ms: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub event_type: String,
    pub data: String,
}
