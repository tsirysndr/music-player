#[cfg(test)]
mod tests;

pub mod addon;
pub mod album;
pub mod artist;
pub mod artist_tracks;
pub mod folder;
pub mod playlist;
pub mod playlist_tracks;
pub mod track;

pub mod select_result {
    use music_player_types::types::{Album, Artist, Track};
    use sea_orm::FromQueryResult;

    #[derive(Debug, FromQueryResult, Clone)]
    pub struct PlaylistTrack {
        pub id: String,
        pub name: String,
        pub description: Option<String>,
        pub album_id: String,
        pub album_title: String,
        pub album_cover: Option<String>,
        pub album_year: Option<u32>,
        pub artist_id: String,
        pub artist_name: String,
        pub track_id: String,
        pub track_title: String,
        pub track_duration: f32,
        pub track_number: Option<u32>,
        pub track_artist: String,
        pub track_genre: Option<String>,
        pub track_uri: String,
    }

    impl Into<Track> for PlaylistTrack {
        fn into(self) -> Track {
            Track {
                id: self.track_id,
                title: self.track_title,
                duration: Some(self.track_duration),
                track_number: self.track_number,
                uri: self.track_uri,
                artists: vec![Artist {
                    id: self.artist_id,
                    name: self.artist_name,
                    ..Default::default()
                }],
                album: Some(Album {
                    id: self.album_id,
                    title: self.album_title,
                    cover: self.album_cover,
                    year: self.album_year,
                    ..Default::default()
                }),
                artist: self.track_artist,
                ..Default::default()
            }
        }
    }
}
