#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Addon {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub description: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub icon: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub url: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub version: ::prost::alloc::string::String,
    #[prost(string, tag = "7")]
    pub author: ::prost::alloc::string::String,
    #[prost(bool, tag = "8")]
    pub enabled: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Playlist {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub description: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "4")]
    pub tracks: ::prost::alloc::vec::Vec<super::super::metadata::v1alpha1::Track>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Tracklist {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub tracks: ::prost::alloc::vec::Vec<super::super::metadata::v1alpha1::Track>,
}
