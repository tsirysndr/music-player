use super::{album::Album, track::Track};
use async_graphql::*;
use music_player_entity::artist::Model;
use music_player_types::types::{Artist as ArtistType, RemoteTrackUrl};
use serde::Serialize;

#[derive(Default, Clone, Serialize)]
pub struct Artist {
    /// The server this came from — its saved name — or `null` for this
    /// machine's own library.
    ///
    /// Search is federated, so one result list holds rows from both. Without
    /// this a client cannot tell them apart, and the actions that make sense
    /// differ: a local track cannot be added to a remote server's playlist.
    pub source: Option<String>,
    pub id: ID,
    pub name: String,
    pub picture: String,
    pub bio: String,
    pub website: String,
    pub genres: Vec<String>,
    pub images: Vec<String>,
    pub albums: Vec<Album>,
    pub songs: Vec<Track>,
}

#[Object]
impl Artist {
    /// Which library this row came from; `null` is this machine.
    async fn source(&self) -> &Option<String> {
        &self.source
    }

    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn picture(&self) -> &str {
        &self.picture
    }

    async fn bio(&self) -> &str {
        &self.bio
    }

    async fn website(&self) -> &str {
        &self.website
    }

    async fn genres(&self) -> Vec<String> {
        self.genres.clone()
    }

    async fn images(&self) -> Vec<String> {
        self.images.clone()
    }

    async fn albums(&self) -> Vec<Album> {
        self.albums.clone()
    }

    async fn songs(&self) -> Vec<Track> {
        self.songs.clone()
    }
}

impl From<Model> for Artist {
    fn from(model: Model) -> Self {
        Self {
            // Set by the search resolver; everything else is local.
            source: None,
            id: ID(model.id),
            name: model.name,
            picture: model.picture.unwrap_or_default(),
            albums: model.albums.into_iter().map(Into::into).collect(),
            songs: model.tracks.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

impl From<ArtistType> for Artist {
    fn from(artist: ArtistType) -> Self {
        Self {
            // Set by the search resolver; everything else is local.
            source: None,
            id: ID(artist.id),
            name: artist.name,
            picture: artist.picture.unwrap_or_default(),
            albums: artist.albums.into_iter().map(Into::into).collect(),
            songs: artist.songs.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

impl RemoteTrackUrl for Artist {
    fn with_remote_track_url(&self, base_url: &str) -> Self {
        Self {
            // Set by the search resolver; everything else is local.
            source: None,
            songs: self
                .songs
                .iter()
                .map(|track| track.with_remote_track_url(base_url))
                .collect(),
            ..self.clone()
        }
    }
}
