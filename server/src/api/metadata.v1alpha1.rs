#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SongArtist {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Song {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub title: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "4")]
    pub artists: ::prost::alloc::vec::Vec<SongArtist>,
    #[prost(float, tag = "5")]
    pub duration: f32,
    #[prost(int32, tag = "6")]
    pub disc_number: i32,
    #[prost(int32, tag = "7")]
    pub track_number: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Album {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub cover: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub release_date: ::prost::alloc::string::String,
    #[prost(int32, tag = "5")]
    pub year: i32,
    #[prost(string, tag = "6")]
    pub artist: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "7")]
    pub genres: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "8")]
    pub tracks: ::prost::alloc::vec::Vec<Song>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ArtistSong {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub artist: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "4")]
    pub artists: ::prost::alloc::vec::Vec<Artist>,
    #[prost(float, tag = "5")]
    pub duration: f32,
    #[prost(int32, tag = "6")]
    pub disc_number: i32,
    #[prost(int32, tag = "7")]
    pub track_number: i32,
    #[prost(message, optional, tag = "8")]
    pub album: ::core::option::Option<Album>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Artist {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub picture: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub bio: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub website: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "6")]
    pub genres: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "7")]
    pub images: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "8")]
    pub albums: ::prost::alloc::vec::Vec<Album>,
    #[prost(message, repeated, tag = "9")]
    pub songs: ::prost::alloc::vec::Vec<ArtistSong>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Lyrics {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub content: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Track {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub title: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub album: ::core::option::Option<Album>,
    #[prost(message, repeated, tag = "4")]
    pub artists: ::prost::alloc::vec::Vec<Artist>,
    #[prost(float, tag = "5")]
    pub duration: f32,
    #[prost(int32, tag = "6")]
    pub disc_number: i32,
    #[prost(int32, tag = "7")]
    pub track_number: i32,
    #[prost(message, optional, tag = "8")]
    pub lyrics: ::core::option::Option<Lyrics>,
    #[prost(string, tag = "9")]
    pub uri: ::prost::alloc::string::String,
    #[prost(string, tag = "10")]
    pub artist: ::prost::alloc::string::String,
}
