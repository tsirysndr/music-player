#[cfg(test)]
mod tests;

pub mod addons;
pub mod core;
pub mod event;
pub mod history;
pub mod library;
pub mod mixer;
pub mod playback;
pub mod playlist;
pub mod server;
pub mod tracklist;
pub mod api {
    #[path = ""]
    pub mod music {
        use music_player_entity::folder;

        use self::v1alpha1::GetFolderDetailsResponse;

        #[path = "music.v1alpha1.rs"]
        pub mod v1alpha1;

        impl From<folder::Model> for GetFolderDetailsResponse {
            fn from(model: folder::Model) -> Self {
                Self {
                    id: model.id,
                    name: model.name,
                    playlists: model.playlists.into_iter().map(Into::into).collect(),
                    ..Default::default()
                }
            }
        }
    }

    #[path = ""]
    pub mod objects {
        use self::v1alpha1::Playlist;
        use music_player_entity::playlist;

        #[path = "objects.v1alpha1.rs"]
        pub mod v1alpha1;

        impl From<playlist::Model> for Playlist {
            fn from(model: playlist::Model) -> Self {
                Self {
                    id: model.id,
                    name: model.name,
                    description: model.description.unwrap_or_default(),
                    tracks: model.tracks.into_iter().map(Into::into).collect(),
                    ..Default::default()
                }
            }
        }
    }

    #[path = ""]
    pub mod metadata {
        use music_player_entity::{album, artist, track};

        use self::v1alpha1::{Album, Artist, ArtistSong, Song, SongArtist, Track};

        #[path = "metadata.v1alpha1.rs"]
        pub mod v1alpha1;

        impl From<artist::Model> for Artist {
            fn from(model: artist::Model) -> Self {
                Self {
                    id: model.id,
                    name: model.name,
                    songs: model.tracks.into_iter().map(Into::into).collect(),
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
                    artists: model.artists.into_iter().map(Into::into).collect(),
                    ..Default::default()
                }
            }
        }

        impl From<artist::Model> for SongArtist {
            fn from(model: artist::Model) -> Self {
                Self {
                    id: model.id,
                    name: model.name,
                    ..Default::default()
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
                    ..Default::default()
                }
            }
        }
    }
}
