use music_player_types::{
    source::SourceRef,
    types::{album_id, RemoteTrackUrl, Song, Track as TrackType},
};
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
    /// Which disc of a set. Null for a single-disc release, which is most of
    /// them — the clients only group when an album spans more than one.
    pub disc: Option<u32>,
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
    /// Musical key in Camelot notation, e.g. "8A". Null until the track has
    /// been analysed — which is not the same as a track with no clear key, but
    /// both display as nothing, because a guess is worse than a blank.
    pub key: Option<String>,
    /// Tempo. Null until analysed.
    pub bpm: Option<f32>,
    #[sea_orm(ignore)]
    pub artists: Vec<artist::Model>,
    #[sea_orm(ignore)]
    // Existing consumers require a value here. A track without an album has
    // album_id=None and a default value; conversions restore Option::None.
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
            disc: ActiveValue::Set(song.disc),
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
            // Left alone by the scanner: analysis costs a decode per track and
            // runs separately. `NotSet` rather than `Set(None)` so a rescan
            // does not throw away a key that took a minute to work out.
            key: ActiveValue::NotSet,
            bpm: ActiveValue::NotSet,
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
        let album_id = track
            .album
            .as_ref()
            .filter(|album| !album.id.is_empty())
            .map(|album| album.id.clone());
        // Keep this metadata shallow: Album's conversion attaches the album
        // back to its tracks, so invoking it here would recursively duplicate
        // the same album/track tree.
        let album = track
            .album
            .map(|album| album::Model {
                id: album.id,
                title: album.title,
                artist: album.artist,
                artist_id: album.artist_id,
                cover: album.cover,
                year: album.year,
                ..Default::default()
            })
            .unwrap_or_default();
        Self {
            id: track.id,
            title: track.title,
            artist: track.artist,
            uri: track.uri,
            album_id,
            artist_id: if track.artists.is_empty() {
                None
            } else {
                Some(track.artists[0].id.clone())
            },
            duration: track.duration,
            disc: Some(track.disc_number).filter(|disc| *disc > 0),
            track: track.track_number,
            bitrate: track.bitrate,
            sample_rate: track.sample_rate,
            liked: track.liked,
            album,
            artists: track.artists.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

impl From<Model> for TrackType {
    fn from(val: Model) -> Self {
        // Some existing wire adapters supply album metadata without album_id.
        let has_album = val.album_id.is_some() || val.album != album::Model::default();
        TrackType {
            id: val.id,
            title: val.title,
            artist: val.artist,
            uri: val.uri,
            duration: val.duration,
            disc_number: val.disc.unwrap_or_default(),
            track_number: val.track,
            bitrate: val.bitrate,
            sample_rate: val.sample_rate,
            liked: val.liked,
            album: has_album.then(|| val.album.into()),
            artists: val.artists.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Model> for Metadata {
    fn from(val: Model) -> Self {
        let has_album = val.album_id.is_some() || val.album != album::Model::default();
        Metadata {
            title: val.title,
            artist: Some(val.artist),
            album: has_album.then_some(val.album.title),
            album_art_uri: val.album.cover,
            ..Default::default()
        }
    }
}

impl RemoteTrackUrl for Model {
    fn with_remote_track_url(&self, base_url: &str) -> Self {
        if SourceRef::is_handle(&self.id) || SourceRef::is_handle(&self.uri) {
            return self.clone();
        }
        Self {
            uri: format!("{}/tracks/{}", base_url, self.id),
            ..self.clone()
        }
    }
}

#[cfg(test)]
mod source_tests {
    use super::*;
    use music_player_types::{
        source::{RemoteIdentity, ResourceKind},
        types::Album,
    };

    fn movie() -> TrackType {
        let handle = SourceRef {
            resolver: "emby".into(),
            account_id: "saved-account".into(),
            remote: RemoteIdentity {
                server_id: "server".into(),
                user_id: "user".into(),
            },
            kind: ResourceKind::Item,
            item_id: "movie-1".into(),
        }
        .to_handle();
        TrackType {
            id: handle.clone(),
            uri: handle,
            title: "Movie without a music album".into(),
            duration: Some(19142.97),
            bitrate: Some(192),
            sample_rate: Some(44100),
            liked: Some(true),
            ..Default::default()
        }
    }

    #[test]
    fn movie_round_trip_preserves_absent_album_identity_and_source_metadata() {
        let original = movie();
        let entity = Model::from(original.clone());
        assert_eq!(entity.album_id, None);
        assert_eq!(entity.album, album::Model::default());
        let metadata = Metadata::from(entity.clone());
        assert_eq!(metadata.album, None);
        assert_eq!(metadata.album_art_uri, None);
        let restored = TrackType::from(entity);
        assert!(restored.album.is_none());
        assert_eq!(restored.id, original.id);
        assert_eq!(restored.uri, original.uri);
        assert_eq!(restored.duration, original.duration);
        assert_eq!(restored.bitrate, original.bitrate);
        assert_eq!(restored.sample_rate, original.sample_rate);
        assert_eq!(restored.liked, Some(true));
    }

    #[test]
    fn album_track_conversion_keeps_handles_and_music_numbering() {
        let track = TrackType {
            disc_number: 2,
            track_number: Some(3),
            ..movie()
        };
        let handle = track.id.clone();
        let album = Album {
            id: "album-1".into(),
            title: "Album".into(),
            tracks: vec![track],
            ..Default::default()
        };
        let entity = album::Model::from(album);
        let restored = Album::from(entity);
        let track = &restored.tracks[0];
        assert_eq!(track.id, handle);
        assert_eq!(track.uri, handle);
        assert_eq!(track.disc_number, 2);
        assert_eq!(track.track_number, Some(3));
        assert_eq!(track.album.as_ref().unwrap().id, "album-1");
        assert!(track.album.as_ref().unwrap().tracks.is_empty());
    }

    #[test]
    fn attached_album_metadata_without_foreign_key_is_not_discarded() {
        let entity = Model {
            album: album::Model {
                title: "Legacy radio label".into(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(
            Metadata::from(entity.clone()).album.as_deref(),
            Some("Legacy radio label")
        );
        assert_eq!(
            TrackType::from(entity).album.unwrap().title,
            "Legacy radio label"
        );
    }

    #[test]
    fn entity_decoration_keeps_reserved_namespace_visible_for_validation() {
        for (id, uri) in [
            (movie().id, String::new()),
            ("55508".into(), movie().uri),
            ("mp-source:v2?".into(), "/local/path".into()),
        ] {
            let entity = Model {
                id: id.clone(),
                uri: uri.clone(),
                ..Default::default()
            };
            let decorated = entity.with_remote_track_url("http://other-account.invalid");
            assert_eq!(decorated.id, id);
            assert_eq!(decorated.uri, uri);
        }
    }
}
