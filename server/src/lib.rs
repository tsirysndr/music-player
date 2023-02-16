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
                    ..Default::default()
                }
            }
        }

        impl Into<Playlist> for GetPlaylistDetailsResponse {
            fn into(self) -> Playlist {
                Playlist {
                    id: self.id,
                    name: self.name,
                    description: Some(self.description),
                    tracks: self.tracks.into_iter().map(Into::into).collect(),
                }
            }
        }

        impl From<Playlist> for GetPlaylistDetailsResponse {
            fn from(playlist: Playlist) -> Self {
                Self {
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
                    id: model.id,
                    name: model.name,
                    description: model.description.unwrap_or_default(),
                    tracks: model.tracks.into_iter().map(Into::into).collect(),
                    ..Default::default()
                }
            }
        }

        impl Into<types::Playlist> for Playlist {
            fn into(self) -> types::Playlist {
                types::Playlist {
                    id: self.id,
                    name: self.name,
                    description: Some(self.description),
                    tracks: self.tracks.into_iter().map(Into::into).collect(),
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

        impl From<types::Track> for Song {
            fn from(track: types::Track) -> Self {
                Self {
                    id: track.id,
                    title: track.title,
                    duration: track.duration.unwrap_or_default(),
                    track_number: track.track_number.unwrap_or_default() as i32,
                    artists: track.artists.into_iter().map(Into::into).collect(),
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

        impl From<types::Artist> for SongArtist {
            fn from(artist: types::Artist) -> Self {
                Self {
                    id: artist.id,
                    name: artist.name,
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
                    artist: model.artist,
                    ..Default::default()
                }
            }
        }

        impl Into<types::Track> for ArtistSong {
            fn into(self) -> types::Track {
                types::Track {
                    id: self.id,
                    title: self.title,
                    duration: Some(self.duration),
                    track_number: Some(u32::try_from(self.track_number).unwrap_or_default()),
                    disc_number: u32::try_from(self.disc_number).unwrap_or_default(),
                    artists: self.artists.into_iter().map(Into::into).collect(),
                    album: match self.album {
                        Some(album) => Some(album.into()),
                        None => None,
                    },
                    artist: self.artist,
                    ..Default::default()
                }
            }
        }

        impl Into<types::Track> for Track {
            fn into(self) -> types::Track {
                types::Track {
                    id: self.id,
                    title: self.title,
                    uri: self.uri,
                    duration: Some(self.duration),
                    track_number: Some(u32::try_from(self.track_number).unwrap_or_default()),
                    disc_number: u32::try_from(self.disc_number).unwrap_or_default(),
                    artists: self.artists.into_iter().map(Into::into).collect(),
                    artist: self.artist,
                    album: match self.album {
                        Some(album) => Some(album.into()),
                        None => None,
                    },
                }
            }
        }

        impl Into<track::Model> for Track {
            fn into(self) -> track::Model {
                track::Model {
                    id: self.id,
                    title: self.title,
                    uri: self.uri,
                    duration: Some(self.duration),
                    track: Some(u32::try_from(self.track_number).unwrap_or_default()),
                    artists: self.artists.into_iter().map(Into::into).collect(),
                    artist: self.artist,
                    album: self.album.unwrap().into(),
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
                    album: match track.album {
                        Some(album) => Some(album.into()),
                        None => None,
                    },
                    ..Default::default()
                }
            }
        }

        impl Into<types::Artist> for Artist {
            fn into(self) -> types::Artist {
                types::Artist {
                    id: self.id,
                    name: self.name,
                    picture: Some(self.picture),
                    albums: self.albums.into_iter().map(Into::into).collect(),
                    songs: self.songs.into_iter().map(Into::into).collect(),
                }
            }
        }

        impl Into<artist::Model> for Artist {
            fn into(self) -> artist::Model {
                artist::Model {
                    id: self.id,
                    name: self.name,
                    albums: self.albums.into_iter().map(Into::into).collect(),
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

        impl Into<types::Artist> for SongArtist {
            fn into(self) -> types::Artist {
                types::Artist {
                    id: self.id,
                    name: self.name,
                    ..Default::default()
                }
            }
        }

        impl Into<types::Track> for Song {
            fn into(self) -> types::Track {
                types::Track {
                    id: self.id,
                    title: self.title,
                    duration: Some(self.duration),
                    track_number: Some(u32::try_from(self.track_number).unwrap_or_default()),
                    disc_number: u32::try_from(self.disc_number).unwrap_or_default(),
                    artists: self.artists.into_iter().map(Into::into).collect(),
                    ..Default::default()
                }
            }
        }

        impl Into<types::Album> for Album {
            fn into(self) -> types::Album {
                types::Album {
                    id: self.id,
                    title: self.title,
                    cover: Some(self.cover),
                    artist: self.artist.clone(),
                    year: Some(u32::try_from(self.year).unwrap_or_default()),
                    artist_id: Some(format!("{:x}", md5::compute(self.artist.as_str()))),
                    tracks: self.tracks.into_iter().map(Into::into).collect(),
                }
            }
        }

        impl Into<album::Model> for Album {
            fn into(self) -> album::Model {
                album::Model {
                    id: self.id,
                    title: self.title,
                    cover: Some(self.cover),
                    artist: self.artist.clone(),
                    year: Some(u32::try_from(self.year).unwrap_or_default()),
                    artist_id: Some(format!("{:x}", md5::compute(self.artist.as_str()))),
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
