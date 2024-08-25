use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub genre: String,
    pub year: Option<u32>,
    pub track: Option<u32>,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u8>,
    pub channels: Option<u8>,
    pub duration: Option<f32>,
    pub uri: String,
    pub album_id: Option<String>,
    pub artist_id: Option<String>,
    pub artists: Vec<Artist>,
    pub album: Album,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Artist {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub artist_id: Option<String>,
    pub year: Option<u32>,
    pub cover: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Tracklist {
    pub next_tracks: Vec<Track>,
    pub previous_tracks: Vec<Track>,
}
