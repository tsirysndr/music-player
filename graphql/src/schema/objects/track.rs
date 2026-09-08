use async_graphql::*;
use music_player_entity::{select_result, track::Model};
use music_player_types::types::{self, RemoteTrackUrl};
use music_player_types::types::{RemoteCoverUrl, SimplifiedSong as TrackType};
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
    /// The server this came from — its saved name — or `null` for this
    /// machine's own library.
    ///
    /// Search is federated, so one result list holds rows from both. Without
    /// this a client cannot tell them apart, and the actions that make sense
    /// differ: a local track cannot be added to a remote server's playlist.
    pub source: Option<String>,
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

impl RemoteTrackUrl for Track {
    fn with_remote_track_url(&self, base_url: &str) -> Self {
        // Streaming sources (Subsonic, Jellyfin, ...) already provide an
        // authenticated absolute stream url; keep it as is.
        if self.uri.starts_with("http://") || self.uri.starts_with("https://") {
            return self.clone();
        }
        Self {
            // Set by the search resolver; everything else is local.
            source: None,
            uri: format!("{}/tracks/{}", base_url, *self.id),
            ..self.clone()
        }
    }
}

impl RemoteCoverUrl for Track {
    fn with_remote_cover_url(&self, base_url: &str) -> Self {
        Self {
            // Set by the search resolver; everything else is local.
            source: None,
            album: Album {
                cover: match self.album.cover {
                    Some(ref cover) => match cover.starts_with("http") {
                        true => Some(cover.to_owned()),
                        false => Some(format!("{}/covers/{}", base_url, cover)),
                    },
                    None => None,
                },
                ..self.album.clone()
            },
            ..self.clone()
        }
    }
}

impl From<Model> for Track {
    fn from(model: Model) -> Self {
        Self {
            // Set by the search resolver; everything else is local.
            source: None,
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

impl From<TrackInput> for Model {
    fn from(val: TrackInput) -> Self {
        Model {
            id: val.id.0,
            title: val.title,
            uri: val.uri,
            duration: val.duration,
            track: val.track_number,
            ..Default::default()
        }
    }
}

impl From<TrackType> for Track {
    fn from(song: TrackType) -> Self {
        Self {
            // Set by the search resolver; everything else is local.
            source: None,
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

impl From<types::Track> for Track {
    fn from(track: types::Track) -> Self {
        Self {
            // Set by the search resolver; everything else is local.
            source: None,
            id: ID(track.id),
            title: track.title,
            uri: track.uri,
            duration: track.duration,
            track_number: track.track_number,
            // Carried, not defaulted: a remote source is the only thing that
            // knows an album spans two discs — the local schema has no column
            // for it — and dropping it here is what left the web client unable
            // to group by disc at all.
            disc_number: track.disc_number,
            artist: track.artist,
            album: match track.album.clone() {
                Some(album) => album.into(),
                None => Default::default(),
            },
            artists: track
                .artists
                .clone()
                .into_iter()
                .map(|artist| artist.into())
                .collect(),
            album_title: match track.album.clone() {
                Some(album) => album.title,
                None => String::new(),
            },
            album_id: match track.album.clone() {
                Some(album) => album.id,
                None => String::new(),
            },
            artist_id: match track.artists.clone().first() {
                Some(artist) => artist.id.clone(),
                None => String::new(),
            },
            cover: match track.album {
                Some(album) => album.cover,
                None => None,
            },
        }
    }
}

impl From<select_result::PlaylistTrack> for Track {
    fn from(result: select_result::PlaylistTrack) -> Self {
        Self {
            // Set by the search resolver; everything else is local.
            source: None,
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
