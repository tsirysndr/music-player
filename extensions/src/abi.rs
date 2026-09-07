//! The contract between the host and an extension.
//!
//! Everything crossing the boundary is JSON. That costs a serialization step
//! per call, and buys a boundary that any Extism PDK can speak — an extension
//! can be written in Rust, Go, JavaScript, Python or C without the host caring.
//!
//! These types are the whole ABI. Adding a field is compatible (older
//! extensions ignore what they do not read, `#[serde(default)]` fills what they
//! do not send); removing or renaming one is not.

use serde::{Deserialize, Serialize};

/// A track as an extension sees it — the tags, not the internals.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackInfo {
    pub id: String,
    pub title: String,
    pub artist: String,
    #[serde(default)]
    pub album: String,
    #[serde(default)]
    pub album_artist: String,
    #[serde(default)]
    pub genre: String,
    #[serde(default)]
    pub year: Option<u32>,
    #[serde(default)]
    pub track_number: Option<u32>,
    /// Seconds.
    #[serde(default)]
    pub duration: f32,
    #[serde(default)]
    pub uri: String,
}

impl From<&music_player_entity::track::Model> for TrackInfo {
    fn from(track: &music_player_entity::track::Model) -> Self {
        Self {
            id: track.id.clone(),
            title: track.title.clone(),
            artist: track.artist.clone(),
            album: track.album.title.clone(),
            album_artist: track.album.artist.clone(),
            genre: track.genre.clone(),
            year: track.year,
            track_number: track.track,
            duration: track.duration.unwrap_or_default(),
            uri: track.uri.clone(),
        }
    }
}

// ── Events ──────────────────────────────────────────────────────────────────

/// Something that happened, delivered to extensions with the `events`
/// capability. The variant name is the exported function the host calls, so
/// `TrackPlayed` looks for `on_track_played`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    /// A play crossed the scrobble threshold.
    TrackPlayed {
        track: TrackInfo,
        /// Unix seconds when the play started.
        played_at: i64,
        play_count: i32,
    },
    /// A track was passed over before that threshold.
    TrackSkipped { track: TrackInfo, skipped_at: i64 },
    /// A track was liked or unliked.
    TrackLiked { track: TrackInfo, liked: bool },
    /// A playlist was created.
    PlaylistCreated {
        id: String,
        name: String,
        is_smart: bool,
    },
    /// A library scan finished.
    ScanCompleted {
        tracks_added: u32,
        total_tracks: u32,
    },
}

impl Event {
    /// The function name the host invokes for this event.
    pub fn handler(&self) -> &'static str {
        match self {
            Event::TrackPlayed { .. } => "on_track_played",
            Event::TrackSkipped { .. } => "on_track_skipped",
            Event::TrackLiked { .. } => "on_track_liked",
            Event::PlaylistCreated { .. } => "on_playlist_created",
            Event::ScanCompleted { .. } => "on_scan_completed",
        }
    }
}

// ── Metadata ────────────────────────────────────────────────────────────────

/// What the host asks a metadata provider for.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataRequest {
    pub track: TrackInfo,
}

/// A provider's answer. Every field is optional: a provider fills in what it
/// knows and leaves the rest, and the host takes the first non-empty answer
/// across the enabled providers.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataResponse {
    /// Plain-text or LRC lyrics.
    #[serde(default)]
    pub lyrics: Option<String>,
    /// URL of cover art.
    #[serde(default)]
    pub artwork_url: Option<String>,
    /// Artist biography.
    #[serde(default)]
    pub bio: Option<String>,
    /// Genre tags this provider would apply.
    #[serde(default)]
    pub genres: Vec<String>,
}

impl MetadataResponse {
    /// Whether this answer carries anything at all.
    pub fn is_empty(&self) -> bool {
        self.lyrics.is_none()
            && self.artwork_url.is_none()
            && self.bio.is_none()
            && self.genres.is_empty()
    }
}

// ── Commands ────────────────────────────────────────────────────────────────

/// An action an extension offers, as listed by its `commands` export.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandSpec {
    /// Identifier passed back to `run_command`.
    pub name: String,
    /// What the UI shows.
    pub title: String,
    #[serde(default)]
    pub description: String,
    /// Whether the command wants the current track passed in.
    #[serde(default)]
    pub needs_track: bool,
}

/// The call into `run_command`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandRequest {
    pub name: String,
    /// The current track, when the command asked for one.
    #[serde(default)]
    pub track: Option<TrackInfo>,
    /// Free-form arguments from the caller, as a JSON string. A string rather
    /// than a nested object because the XTP schema — which generates the
    /// bindings for every guest language — has no "any JSON" type.
    #[serde(default)]
    pub args: String,
}

/// What a command did, for the UI to report.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResponse {
    #[serde(default)]
    pub ok: bool,
    /// A line to show the user.
    #[serde(default)]
    pub message: String,
    /// Anything structured the extension wants to hand back, as a JSON string.
    #[serde(default)]
    pub data: String,
}

// ── Smart-playlist predicates ───────────────────────────────────────────────

/// A predicate an extension adds to the filter vocabulary, as listed by its
/// `predicates` export.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PredicateSpec {
    /// The name used in a filter, e.g. `mood` for `ext:mood==chill`.
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// Example values, shown in a picker.
    #[serde(default)]
    pub values: Vec<String>,
}

/// The call into `evaluate`: does this track satisfy `name op value`?
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PredicateRequest {
    pub name: String,
    /// The comparison written in the filter: `==`, `!=`, `>`, `>=`, `<`, `<=`.
    #[serde(default)]
    pub op: String,
    #[serde(default)]
    pub value: String,
    pub track: TrackInfo,
}

/// A predicate's verdict for one track.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PredicateResponse {
    pub matches: bool,
}

// ── The user's library ──────────────────────────────────────────────────────

/// An album in the user's own library, as an extension sees it.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryAlbum {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub year: Option<u32>,
    /// Cover file name, resolvable through the player's cover route.
    #[serde(default)]
    pub cover: String,
}

impl From<&music_player_entity::album::Model> for LibraryAlbum {
    fn from(album: &music_player_entity::album::Model) -> Self {
        Self {
            id: album.id.clone(),
            title: album.title.clone(),
            artist: album.artist.clone(),
            year: album.year,
            cover: album.cover.clone().unwrap_or_default(),
        }
    }
}

/// An artist in the user's own library.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryArtist {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub image: String,
}

impl From<&music_player_entity::artist::Model> for LibraryArtist {
    fn from(artist: &music_player_entity::artist::Model) -> Self {
        Self {
            id: artist.id.clone(),
            name: artist.name.clone(),
            image: artist.picture.clone().unwrap_or_default(),
        }
    }
}

/// A playlist in the user's own library.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryPlaylist {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub is_smart: bool,
    #[serde(default)]
    pub track_count: u32,
}

impl From<&music_player_entity::playlist::Model> for LibraryPlaylist {
    fn from(playlist: &music_player_entity::playlist::Model) -> Self {
        Self {
            id: playlist.id.clone(),
            name: playlist.name.clone(),
            description: playlist.description.clone().unwrap_or_default(),
            is_smart: playlist.is_smart,
            track_count: playlist.tracks.len() as u32,
        }
    }
}

/// A radio station the user has bookmarked.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedRadio {
    pub id: String,
    pub name: String,
    pub stream_url: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub genre: String,
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub logo: String,
    #[serde(default)]
    pub bitrate: u32,
}

impl From<&music_player_entity::saved_radio::Model> for SavedRadio {
    fn from(radio: &music_player_entity::saved_radio::Model) -> Self {
        Self {
            id: radio.id.clone(),
            name: radio.name.clone(),
            stream_url: radio.stream_url.clone(),
            source: radio.source.clone(),
            genre: radio.genre.clone(),
            country: radio.country.clone(),
            logo: radio.logo.clone(),
            bitrate: radio.bitrate,
        }
    }
}

// ── Media sources ───────────────────────────────────────────────────────────

/// What kind of source an extension provides, from its `source_info` export.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfo {
    /// Shown in the source picker, e.g. "Bandcamp".
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// Whether the source needs credentials, which it declares as config keys.
    #[serde(default)]
    pub requires_auth: bool,
    /// Whether `BrowseRequest::query` is honoured.
    #[serde(default)]
    pub supports_search: bool,
}

/// One page of a browse or search.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseRequest {
    /// List the children of this album, artist or playlist. Empty is the top
    /// level.
    #[serde(default)]
    pub parent_id: String,
    /// Free-text search. Empty browses rather than searches.
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub offset: u32,
    /// 0 means the host's default.
    #[serde(default)]
    pub limit: u32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceAlbum {
    /// The source's own id, passed back as `parent_id`.
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub year: Option<u32>,
    #[serde(default)]
    pub cover_url: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceArtist {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub image_url: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcePlaylist {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub track_count: u32,
}

/// Ask a source to resolve one of its track ids to something playable.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamRequest {
    pub track_id: String,
}

/// Where a source's track actually lives.
///
/// Resolved at play time rather than while browsing, so a signed or expiring
/// url is still valid when the decoder opens it.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamResponse {
    /// A direct http(s) url the player can decode.
    pub url: String,
    #[serde(default)]
    pub mime_type: String,
    /// Extra request headers as a JSON object, for a source that needs an
    /// `Authorization` header. Empty when none are needed.
    #[serde(default)]
    pub headers: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_name_their_handler() {
        let event = Event::ScanCompleted {
            tracks_added: 3,
            total_tracks: 40,
        };
        assert_eq!(event.handler(), "on_scan_completed");
    }

    /// The tag is what a non-Rust PDK switches on, so it has to be stable.
    #[test]
    fn events_serialize_with_a_type_tag() {
        let event = Event::TrackLiked {
            track: TrackInfo {
                id: "t1".into(),
                title: "Airbag".into(),
                artist: "Radiohead".into(),
                ..Default::default()
            },
            liked: true,
        };
        let json: serde_json::Value = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "track_liked");
        assert_eq!(json["track"]["title"], "Airbag");
        assert_eq!(json["liked"], true);
    }

    /// An extension may send back only the fields it knows about; the rest has
    /// to default rather than fail the whole call.
    #[test]
    fn partial_metadata_answers_deserialize() {
        let response: MetadataResponse = serde_json::from_str(r#"{"lyrics":"la la la"}"#).unwrap();
        assert_eq!(response.lyrics.as_deref(), Some("la la la"));
        assert!(response.artwork_url.is_none());
        assert!(!response.is_empty());

        assert!(serde_json::from_str::<MetadataResponse>("{}")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn track_info_comes_from_the_entity() {
        let mut track = music_player_entity::track::Model {
            id: "t1".into(),
            title: "Airbag".into(),
            artist: "Radiohead".into(),
            duration: Some(284.0),
            ..Default::default()
        };
        track.album.title = "OK Computer".into();
        let info = TrackInfo::from(&track);
        assert_eq!(info.album, "OK Computer");
        assert_eq!(info.duration, 284.0);
    }
}
