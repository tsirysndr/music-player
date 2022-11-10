use async_graphql::*;
use music_player_entity::track::Model;

use super::{album::Album, artist::Artist};

#[derive(InputObject, Clone)]
pub struct TrackInput {
    pub id: ID,
    pub title: String,
    pub duration: Option<f32>,
    pub disc_number: u32,
    pub track_number: Option<u32>,
    pub uri: String,
}

#[derive(Default, Clone)]
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
