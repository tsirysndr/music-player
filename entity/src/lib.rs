pub mod addon;
pub mod album;
pub mod artist;
pub mod artist_tracks;
pub mod folder;
pub mod playlist;
pub mod playlist_tracks;
pub mod track;

pub mod select_result {
    use sea_orm::FromQueryResult;

    #[derive(Debug, FromQueryResult)]
    pub struct PlaylistTrack {
        pub id: String,
        pub name: String,
        pub description: Option<String>,
        pub album_id: String,
        pub album_title: String,
        pub album_artist: String,
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
}
