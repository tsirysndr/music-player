use async_graphql::*;
use music_player_entity::album::Model;
use music_player_types::types::{Album as AlbumType, RemoteCoverUrl, RemoteTrackUrl};
use serde::Serialize;

use super::track::Track;

#[derive(Default, Clone, Serialize)]
pub struct Album {
    /// The server this came from — its saved name — or `null` for this
    /// machine's own library.
    ///
    /// Search is federated, so one result list holds rows from both. Without
    /// this a client cannot tell them apart, and the actions that make sense
    /// differ: a local track cannot be added to a remote server's playlist.
    pub source: Option<String>,
    pub id: ID,
    pub title: String,
    pub cover: Option<String>,
    pub release_date: String,
    pub artist: String,
    pub year: Option<u32>,
    pub genres: Vec<String>,
    pub tracks: Vec<Track>,
}

#[Object]
impl Album {
    /// Which library this row came from; `null` is this machine.
    async fn source(&self) -> &Option<String> {
        &self.source
    }

    async fn id(&self) -> &str {
        &self.id
    }

    async fn title(&self) -> &str {
        &self.title
    }

    async fn cover(&self) -> &Option<String> {
        &self.cover
    }

    async fn release_date(&self) -> &str {
        &self.release_date
    }

    async fn artist(&self) -> &str {
        &self.artist
    }

    async fn year(&self) -> Option<u32> {
        self.year
    }

    async fn genres(&self) -> Vec<String> {
        self.genres.clone()
    }

    async fn tracks(&self) -> Vec<Track> {
        self.tracks.clone()
    }
}

impl RemoteCoverUrl for Album {
    fn with_remote_cover_url(&self, base_url: &str) -> Self {
        Self {
            // Set by the search resolver; everything else is local.
            source: None,
            cover: self
                .cover
                .clone()
                .map(|cover| match cover.starts_with("http") {
                    true => cover,
                    false => format!("{}/covers/{}", base_url, cover),
                }),
            ..self.clone()
        }
    }
}

impl RemoteTrackUrl for Album {
    fn with_remote_track_url(&self, base_url: &str) -> Self {
        Self {
            // Set by the search resolver; everything else is local.
            source: None,
            tracks: self
                .tracks
                .iter()
                .map(|track| track.with_remote_track_url(base_url))
                .collect(),
            ..self.clone()
        }
    }
}

impl From<Model> for Album {
    fn from(model: Model) -> Self {
        Self {
            // Set by the search resolver; everything else is local.
            source: None,
            id: ID(model.id),
            title: model.title,
            cover: model.cover,
            artist: model.artist,
            year: model.year,
            tracks: model.tracks.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

impl From<AlbumType> for Album {
    fn from(album: AlbumType) -> Self {
        Self {
            // Set by the search resolver; everything else is local.
            source: None,
            id: ID(album.id),
            title: album.title,
            cover: album.cover,
            artist: album.artist,
            year: album.year,
            tracks: album.tracks.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}
