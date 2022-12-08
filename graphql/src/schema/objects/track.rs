use async_graphql::*;
use music_player_entity::{select_result, track::Model};
use music_player_types::types::SimplifiedSong as TrackType;
use serde::Serialize;

use super::{album::Album, artist::Artist};

#[derive(InputObject, Default, Clone)]
pub struct TrackInput {
    pub id: ID,
    pub title: String,
    pub duration: Option<f32>,
    pub disc_number: u32,
    pub track_number: Option<u32>,
    pub uri: String,
}

#[derive(Default, Clone, Serialize)]
pub struct Track {
    pub id: ID,
    pub title: String,
    pub duration: Option<f32>,
    pub disc_number: u32,
    pub track_number: Option<u32>,
    pub uri: String,
    pub artists: Vec<Artist>,
    pub album: Album,
    pub artist: String,
    pub cover: Option<String>,
    pub artist_id: String,
    pub album_id: String,
    pub album_title: String,
}

#[Object]
impl Track {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn title(&self) -> &str {
        &self.title
    }

    async fn duration(&self) -> Option<f32> {
        self.duration
    }

    async fn disc_number(&self) -> u32 {
        self.disc_number
    }

    async fn track_number(&self) -> Option<u32> {
        self.track_number
    }

    async fn uri(&self) -> &str {
        &self.uri
    }

    async fn artists(&self) -> Vec<Artist> {
        self.artists.clone()
    }

    async fn album(&self) -> Album {
        self.album.clone()
    }

    async fn artist(&self) -> &str {
        &self.artist
    }

    async fn cover(&self) -> Option<String> {
        self.cover.clone()
    }

    async fn artist_id(&self) -> &str {
        &self.artist_id
    }

    async fn album_id(&self) -> &str {
        &self.album_id
    }

    async fn album_title(&self) -> &str {
        &self.album_title
    }
}

impl From<Model> for Track {
    fn from(model: Model) -> Self {
        Self {
            id: ID(model.id),
            title: model.title,
            uri: model.uri,
            duration: model.duration,
            track_number: model.track,
            artists: model.artists.into_iter().map(Into::into).collect(),
            album: model.album.into(),
            artist: model.artist,
            ..Default::default()
        }
    }
}

impl Into<Model> for TrackInput {
    fn into(self) -> Model {
        Model {
            id: self.id.0,
            title: self.title,
            uri: self.uri,
            duration: self.duration,
            track: self.track_number,
            ..Default::default()
        }
    }
}

impl From<TrackType> for Track {
    fn from(song: TrackType) -> Self {
        Self {
            id: ID(song.id),
            title: song.title,
            artist: song.artist,
            duration: Some(song.duration.as_secs_f32()),
            cover: song.cover,
            artist_id: song.artist_id,
            album_id: song.album_id,
            album_title: song.album,
            ..Default::default()
        }
    }
}

impl From<select_result::PlaylistTrack> for Track {
    fn from(result: select_result::PlaylistTrack) -> Self {
        Self {
            id: ID(result.track_id),
            title: result.track_title,
            duration: Some(result.track_duration),
            track_number: result.track_number,
            artist: result.track_artist,
            album_title: result.album_title.clone(),
            album_id: result.album_id.clone(),
            artist_id: result.artist_id.clone(),
            cover: result.album_cover.clone(),
            album: Album {
                id: ID(result.album_id),
                title: result.album_title,
                cover: result.album_cover,
                year: result.album_year,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
