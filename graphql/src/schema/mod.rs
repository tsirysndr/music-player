use async_graphql::MergedObject;

pub mod addons;
pub mod core;
pub mod history;
pub mod library;
pub mod mixer;
pub mod objects;
pub mod playback;
pub mod playlist;
pub mod tracklist;

#[derive(MergedObject, Default)]
pub struct Query(playback::PlaybackQuery, tracklist::TracklistQuery);

#[derive(MergedObject, Default)]
pub struct Mutation(playback::PlaybackMutation, tracklist::TracklistMutation);
