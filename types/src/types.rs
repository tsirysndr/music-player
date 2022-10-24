use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Song {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub year: Option<u32>,
    pub track: Option<u32>,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u8>,
    pub channels: Option<u8>,
    pub duration: Duration,
    pub uri: Option<String>,
    pub cover: Option<String>,
    pub album_artist: String,
}
