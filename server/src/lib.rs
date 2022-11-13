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
    pub mod v1alpha1 {
        tonic::include_proto!("music.v1alpha1");
    }
}

pub mod objects {
    pub mod v1alpha1 {
        tonic::include_proto!("objects.v1alpha1");
    }
}

pub mod metadata {
    pub mod v1alpha1 {
        use music_player_entity::{album, artist, track};
        tonic::include_proto!("metadata.v1alpha1");

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
