use music_player_types::types::Song;
use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};

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
    #[sea_orm(ignore)]
    pub artists: Vec<artist::Model>,
    #[sea_orm(ignore)]
    pub album: album::Model,
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
            album_id: ActiveValue::Set(Some(format!(
                "{:x}",
                md5::compute(format!("{}", song.album))
            ))),
            artist_id: ActiveValue::Set(Some(format!(
                "{:x}",
                md5::compute(song.album_artist.to_owned())
            ))),
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
                artist: playlist_track.artist_name.clone(),
                ..Default::default()
            },
            artists: vec![artist::Model {
                id: playlist_track.artist_id,
                name: playlist_track.artist_name.clone(),
                ..Default::default()
            }],
            ..Default::default()
        }
    }
}
