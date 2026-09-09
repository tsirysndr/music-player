use music_player_types::types::{album_id, RemoteTrackUrl, Song, Track as TrackType};
use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};
use upnp_client::types::Metadata;

use crate::{album, artist, select_result};

#[derive(Clone, Debug, Default, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "track")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
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
    /// `at://` uri of the matching `app.rocksky.song` record, when the track
    /// has been linked to the user's atproto repo. Optional: unmatched tracks
    /// and libraries with no linked account leave it null.
    pub aturi: Option<String>,
    /// When the scanner first saw this file. Null for tracks that were already
    /// in the library before the column existed.
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[sea_orm(ignore)]
    pub artists: Vec<artist::Model>,
    #[sea_orm(ignore)]
    pub album: album::Model,
    /// Whether the source that produced this track has it liked.
    ///
    /// Not a column: a local like lives in the user's atproto repo, and a
    /// remote one on that server. It rides along so a queued track still knows
    /// its own state — without it the flag died here and the heart fell back
    /// to a snapshot list, which is why some tracks lit and some did not.
    #[sea_orm(ignore)]
    pub liked: Option<bool>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::album::Entity",
        from = "Column::AlbumId",
        to = "super::album::Column::Id"
    )]
    Album,
    #[sea_orm(
        belongs_to = "super::artist::Entity",
        from = "Column::ArtistId",
        to = "super::artist::Column::Id"
    )]
    Artist,
}

impl Related<super::album::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Album.def()
    }
}

impl Related<super::playlist::Entity> for Entity {
    fn to() -> RelationDef {
        super::playlist_tracks::Relation::Playlist.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::playlist_tracks::Relation::Track.def().rev())
    }
}

impl Related<super::artist::Entity> for Entity {
    fn to() -> RelationDef {
        super::artist_tracks::Relation::Artist.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::artist_tracks::Relation::Track.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug)]
pub struct TrackToAlbum;

impl Linked for TrackToAlbum {
    type FromEntity = super::track::Entity;
    type ToEntity = super::album::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            super::album::Relation::Track.def().rev(),
            super::track::Relation::Album.def(),
        ]
    }
}

#[derive(Debug)]
pub struct TrackToArtist;

impl Linked for TrackToArtist {
    type FromEntity = super::track::Entity;
    type ToEntity = super::artist::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            super::artist_tracks::Relation::Track.def().rev(),
            super::artist_tracks::Relation::Artist.def(),
        ]
    }
}

#[derive(Debug)]
pub struct TrackToPlaylist;

impl Linked for TrackToPlaylist {
    type FromEntity = super::track::Entity;
    type ToEntity = super::playlist::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            super::playlist_tracks::Relation::Track.def().rev(),
            super::playlist_tracks::Relation::Playlist.def(),
        ]
    }
}

impl From<&Song> for ActiveModel {
    fn from(song: &Song) -> Self {
        let id = format!("{:x}", md5::compute(song.uri.as_ref().unwrap()));
        Self {
            id: ActiveValue::set(id),
            artist: ActiveValue::Set(song.artist.clone()),
            title: ActiveValue::Set(song.title.clone()),
            genre: ActiveValue::Set(song.genre.clone()),
            year: ActiveValue::Set(song.year),
            track: ActiveValue::Set(song.track),
            bitrate: ActiveValue::Set(song.bitrate),
            sample_rate: ActiveValue::Set(song.sample_rate),
            bit_depth: ActiveValue::Set(song.bit_depth),
            channels: ActiveValue::Set(song.channels),
            duration: ActiveValue::Set(Some(song.duration.as_secs_f32())),
            uri: ActiveValue::Set(song.uri.clone().unwrap_or_default()),
            album_id: ActiveValue::Set(Some(album_id(&song.album, &song.album_artist))),
            artist_id: ActiveValue::Set(Some(format!("{:x}", md5::compute(&song.album_artist)))),
            // Preserved across rescans: the atproto link is set by the likes
            // importer, not by the scanner.
            aturi: ActiveValue::NotSet,
            // Stamped once, when the scanner first inserts the row. A rescan
            // must not move it, or "recently added" would list the whole
            // library after every scan.
            created_at: ActiveValue::Set(Some(chrono::Utc::now())),
        }
    }
}

impl From<select_result::PlaylistTrack> for Model {
    fn from(playlist_track: select_result::PlaylistTrack) -> Self {
        Self {
            id: playlist_track.track_id,
            title: playlist_track.track_title,
            artist: playlist_track.track_artist,
            uri: playlist_track.track_uri,
            album_id: Some(playlist_track.album_id.clone()),
            artist_id: Some(playlist_track.artist_id.clone()),
            duration: Some(playlist_track.track_duration),
            album: album::Model {
                id: playlist_track.album_id,
                title: playlist_track.album_title,
                cover: playlist_track.album_cover,
                ..Default::default()
            },
            artists: vec![artist::Model {
                id: playlist_track.artist_id,
                name: playlist_track.artist_name,
                ..Default::default()
            }],
            ..Default::default()
        }
    }
}

impl From<TrackType> for Model {
    fn from(track: TrackType) -> Self {
        let track_album = track.album.unwrap();
        Self {
            id: track.id,
            title: track.title,
            artist: track.artist,
            uri: track.uri,
            album_id: Some(track_album.id.clone()),
            artist_id: if track.artists.is_empty() {
                None
            } else {
                Some(track.artists[0].id.clone())
            },
            duration: track.duration,
            album: album::Model {
                id: track_album.id,
                title: track_album.title,
                cover: track_album.cover,
                year: track_album.year,
                ..Default::default()
            },
            artists: track.artists.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

impl From<Model> for TrackType {
    fn from(val: Model) -> Self {
        TrackType {
            id: val.id,
            title: val.title,
            artist: val.artist,
            uri: val.uri,
            duration: val.duration,
            album: Some(val.album.into()),
            artists: val.artists.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

impl From<Model> for Metadata {
    fn from(val: Model) -> Self {
        Metadata {
            title: val.title,
            artist: Some(val.artist),
            album: Some(val.album.title),
            album_art_uri: val.album.cover,
            ..Default::default()
        }
    }
}

impl RemoteTrackUrl for Model {
    fn with_remote_track_url(&self, base_url: &str) -> Self {
        Self {
            uri: format!("{}/tracks/{}", base_url, self.id),
            ..self.clone()
        }
    }
}
