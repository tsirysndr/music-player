use async_graphql::*;

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
}
