#[cfg(test)]
mod tests;

pub mod addon;
pub mod album;
pub mod artist;
pub mod artist_tracks;
pub mod atproto_repo_sync;
pub mod extension;
pub mod folder;
pub mod playlist;
pub mod playlist_tracks;
pub mod rocksky_like;
pub mod saved_radio;
pub mod saved_server;
pub mod track;
pub mod track_stats;

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

    impl From<PlaylistTrack> for Track {
        fn from(val: PlaylistTrack) -> Self {
            Track {
                id: val.track_id,
                title: val.track_title,
                duration: Some(val.track_duration),
                track_number: val.track_number,
                uri: val.track_uri,
                artists: vec![Artist {
                    id: val.artist_id,
                    name: val.artist_name,
                    ..Default::default()
                }],
                album: Some(Album {
                    id: val.album_id,
                    title: val.album_title,
                    cover: val.album_cover,
                    year: val.album_year,
                    ..Default::default()
                }),
                artist: val.track_artist,
                ..Default::default()
            }
        }
    }
}
