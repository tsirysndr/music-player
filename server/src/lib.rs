#[macro_use]
extern crate log;

#[cfg(test)]
mod tests;

pub mod addons;
pub mod analysis;
pub mod atproto_sync;
pub mod core;
pub mod event;
pub mod history;
pub mod library;
pub mod media_controls;
pub mod mixer;
pub mod play_stats;
pub mod playback;
pub mod playlist;
pub mod remote;
pub mod scrobbler;
pub mod server;
pub mod servers;
pub mod tracklist;
pub mod api {
    #[path = ""]
    pub mod music {
        use music_player_entity::folder;
        use music_player_types::types::Playlist;

        use self::v1alpha1::{GetFolderDetailsResponse, GetPlaylistDetailsResponse};

        #[path = "music.v1alpha1.rs"]
        pub mod v1alpha1;

        impl From<folder::Model> for GetFolderDetailsResponse {
            fn from(model: folder::Model) -> Self {
                Self {
                    id: model.id,
                    name: model.name,
                    playlists: model.playlists.into_iter().map(Into::into).collect(),
                }
            }
        }

        impl From<GetPlaylistDetailsResponse> for Playlist {
            fn from(val: GetPlaylistDetailsResponse) -> Self {
                Playlist {
                    id: val.id,
                    name: val.name,
                    description: Some(val.description),
                    tracks: val.tracks.into_iter().map(Into::into).collect(),
                    track_count: Some(val.track_count).filter(|count| *count > 0),
                }
            }
        }

        impl From<Playlist> for GetPlaylistDetailsResponse {
            fn from(playlist: Playlist) -> Self {
                Self {
                    track_count: playlist.len(),
                    id: playlist.id,
                    name: playlist.name,
                    description: playlist.description.unwrap_or_default(),
                    tracks: playlist.tracks.into_iter().map(Into::into).collect(),
                }
            }
        }
    }

    #[path = ""]
    pub mod objects {
        use self::v1alpha1::Playlist;
        use music_player_entity::playlist;
        use music_player_types::types;

        #[path = "objects.v1alpha1.rs"]
        pub mod v1alpha1;

        impl From<playlist::Model> for Playlist {
            fn from(model: playlist::Model) -> Self {
                Self {
                    track_count: model.tracks.len() as u32,
                    id: model.id,
                    name: model.name,
                    description: model.description.unwrap_or_default(),
                    tracks: model.tracks.into_iter().map(Into::into).collect(),
                }
            }
        }

        impl From<Playlist> for types::Playlist {
            fn from(val: Playlist) -> Self {
                types::Playlist {
                    id: val.id,
                    name: val.name,
                    description: Some(val.description),
                    tracks: val.tracks.into_iter().map(Into::into).collect(),
                    track_count: Some(val.track_count).filter(|count| *count > 0),
                }
            }
        }
    }

    #[path = ""]
    pub mod metadata {
        use music_player_entity::{album, artist, track};
        use music_player_types::types;

        use self::v1alpha1::{Album, Artist, ArtistSong, Song, SongArtist, Track};

        #[path = "metadata.v1alpha1.rs"]
        pub mod v1alpha1;

        impl From<artist::Model> for Artist {
            fn from(model: artist::Model) -> Self {
                Self {
                    id: model.id,
                    name: model.name,
                    picture: model.picture.unwrap_or_default(),
                    songs: model.tracks.into_iter().map(Into::into).collect(),
                    albums: model.albums.into_iter().map(Into::into).collect(),
                    ..Default::default()
                }
            }
        }

        impl From<album::Model> for Album {
            fn from(model: album::Model) -> Self {
                Self {
                    id: model.id,
                    title: model.title,
                    cover: model.cover.unwrap_or_default(),
                    artist: model.artist,
                    year: i32::try_from(model.year.unwrap_or_default()).unwrap_or_default(),
                    tracks: model.tracks.into_iter().map(Into::into).collect(),
                    ..Default::default()
                }
            }
        }

        impl From<track::Model> for Track {
            fn from(model: track::Model) -> Self {
                Self {
                    id: model.id,
                    title: model.title,
                    uri: model.uri,
                    duration: model.duration.unwrap_or(0.0),
                    track_number: i32::try_from(model.track.unwrap_or_default()).unwrap(),
                    bitrate: model.bitrate.unwrap_or_default(),
                    sample_rate: model.sample_rate.unwrap_or_default(),
                    liked: model.liked,
                    artists: model.artists.into_iter().map(Into::into).collect(),
                    album: Some(model.album.into()),
                    artist: model.artist,
                    ..Default::default()
                }
            }
        }

        impl From<track::Model> for Song {
            fn from(model: track::Model) -> Self {
                Self {
                    id: model.id,
                    title: model.title,
                    duration: model.duration.unwrap_or_default(),
                    track_number: i32::try_from(model.track.unwrap_or_default()).unwrap(),
                    // The local track table has no disc column; multi-disc
                    // information reaches the app by the scanner's own path.
                    disc_number: 0,
                    uri: model.uri,
                    album: model.album.title.clone(),
                    artist: model.artist,
                    artists: model.artists.into_iter().map(Into::into).collect(),
                }
            }
        }

        impl From<types::Track> for Song {
            fn from(track: types::Track) -> Self {
                Self {
                    id: track.id,
                    title: track.title,
                    duration: track.duration.unwrap_or_default(),
                    track_number: track.track_number.unwrap_or_default() as i32,
                    disc_number: track.disc_number as i32,
                    uri: track.uri,
                    album: track
                        .album
                        .as_ref()
                        .map(|a| a.title.clone())
                        .unwrap_or_default(),
                    artist: track.artist,
                    artists: track.artists.into_iter().map(Into::into).collect(),
                }
            }
        }

        impl From<artist::Model> for SongArtist {
            fn from(model: artist::Model) -> Self {
                Self {
                    id: model.id,
                    name: model.name,
                }
            }
        }

        impl From<types::Artist> for SongArtist {
            fn from(artist: types::Artist) -> Self {
                Self {
                    id: artist.id,
                    name: artist.name,
                }
            }
        }

        impl From<track::Model> for ArtistSong {
            fn from(model: track::Model) -> Self {
                Self {
                    id: model.id,
                    title: model.title,
                    duration: model.duration.unwrap_or_default(),
                    track_number: i32::try_from(model.track.unwrap_or_default()).unwrap(),
                    artists: model.artists.into_iter().map(Into::into).collect(),
                    album: Some(model.album.into()),
                    artist: model.artist,
                    ..Default::default()
                }
            }
        }

        impl From<ArtistSong> for types::Track {
            fn from(val: ArtistSong) -> Self {
                types::Track {
                    id: val.id,
                    title: val.title,
                    duration: Some(val.duration),
                    track_number: Some(u32::try_from(val.track_number).unwrap_or_default()),
                    disc_number: u32::try_from(val.disc_number).unwrap_or_default(),
                    artists: val.artists.into_iter().map(Into::into).collect(),
                    album: val.album.map(|album| album.into()),
                    artist: val.artist,
                    ..Default::default()
                }
            }
        }

        impl From<Track> for types::Track {
            fn from(val: Track) -> Self {
                types::Track {
                    id: val.id,
                    title: val.title,
                    uri: val.uri,
                    duration: Some(val.duration),
                    track_number: Some(u32::try_from(val.track_number).unwrap_or_default()),
                    disc_number: u32::try_from(val.disc_number).unwrap_or_default(),
                    artists: val.artists.into_iter().map(Into::into).collect(),
                    artist: val.artist,
                    album: val.album.map(|album| album.into()),
                    // Zero on the wire means unknown.
                    bitrate: Some(val.bitrate).filter(|rate| *rate > 0),
                    sample_rate: Some(val.sample_rate).filter(|rate| *rate > 0),
                    liked: val.liked,
                }
            }
        }

        impl From<Track> for track::Model {
            fn from(val: Track) -> Self {
                track::Model {
                    id: val.id,
                    title: val.title,
                    uri: val.uri,
                    duration: Some(val.duration),
                    track: Some(u32::try_from(val.track_number).unwrap_or_default()),
                    artists: val.artists.into_iter().map(Into::into).collect(),
                    artist: val.artist,
                    // The tracklist holds these, and the now-playing readout
                    // reads them back — dropping them here is what made a
                    // remote track show no bitrate and probe the stream for it.
                    bitrate: Some(val.bitrate).filter(|rate| *rate > 0),
                    sample_rate: Some(val.sample_rate).filter(|rate| *rate > 0),
                    // Rides along so a queued track still knows whether the
                    // source has it liked; without it the heart falls back to
                    // a snapshot list, which can always be incomplete.
                    liked: val.liked,
                    album: val.album.map(Into::into).unwrap_or_default(),
                    ..Default::default()
                }
            }
        }

        impl From<types::Track> for Track {
            fn from(track: types::Track) -> Self {
                Self {
                    id: track.id,
                    title: track.title,
                    uri: track.uri,
                    duration: track.duration.unwrap_or_default(),
                    track_number: i32::try_from(track.track_number.unwrap_or_default()).unwrap(),
                    disc_number: i32::try_from(track.disc_number).unwrap(),
                    artists: track.artists.into_iter().map(Into::into).collect(),
                    artist: track.artist,
                    album: track.album.map(|album| album.into()),
                    bitrate: track.bitrate.unwrap_or_default(),
                    sample_rate: track.sample_rate.unwrap_or_default(),
                    liked: track.liked,
                    ..Default::default()
                }
            }
        }

        impl From<Artist> for types::Artist {
            fn from(val: Artist) -> Self {
                types::Artist {
                    id: val.id,
                    name: val.name,
                    picture: Some(val.picture),
                    albums: val.albums.into_iter().map(Into::into).collect(),
                    songs: val.songs.into_iter().map(Into::into).collect(),
                }
            }
        }

        impl From<Artist> for artist::Model {
            fn from(val: Artist) -> Self {
                artist::Model {
                    id: val.id,
                    name: val.name,
                    albums: val.albums.into_iter().map(Into::into).collect(),
                    ..Default::default()
                }
            }
        }

        impl From<types::Artist> for Artist {
            fn from(artist: types::Artist) -> Self {
                Self {
                    id: artist.id,
                    name: artist.name,
                    picture: artist.picture.unwrap_or_default(),
                    ..Default::default()
                }
            }
        }

        impl From<SongArtist> for types::Artist {
            fn from(val: SongArtist) -> Self {
                types::Artist {
                    id: val.id,
                    name: val.name,
                    ..Default::default()
                }
            }
        }

        impl From<Song> for types::Track {
            fn from(val: Song) -> Self {
                types::Track {
                    id: val.id,
                    title: val.title,
                    duration: Some(val.duration),
                    track_number: Some(u32::try_from(val.track_number).unwrap_or_default()),
                    disc_number: u32::try_from(val.disc_number).unwrap_or_default(),
                    artists: val.artists.into_iter().map(Into::into).collect(),
                    ..Default::default()
                }
            }
        }

        impl From<Album> for types::Album {
            fn from(val: Album) -> Self {
                types::Album {
                    id: val.id,
                    title: val.title,
                    cover: Some(val.cover),
                    artist: val.artist.clone(),
                    year: Some(u32::try_from(val.year).unwrap_or_default()),
                    artist_id: Some(format!("{:x}", md5::compute(val.artist.as_str()))),
                    tracks: val.tracks.into_iter().map(Into::into).collect(),
                }
            }
        }

        impl From<Album> for album::Model {
            fn from(val: Album) -> Self {
                album::Model {
                    id: val.id,
                    title: val.title,
                    cover: Some(val.cover),
                    artist: val.artist.clone(),
                    year: Some(u32::try_from(val.year).unwrap_or_default()),
                    artist_id: Some(format!("{:x}", md5::compute(val.artist.as_str()))),
                    ..Default::default()
                }
            }
        }

        impl From<types::Album> for Album {
            fn from(album: types::Album) -> Self {
                Self {
                    id: album.id,
                    title: album.title,
                    cover: album.cover.unwrap_or_default(),
                    artist: album.artist,
                    year: i32::try_from(album.year.unwrap_or_default()).unwrap_or_default(),
                    tracks: album.tracks.into_iter().map(Into::into).collect(),
                    ..Default::default()
                }
            }
        }
    }
}
