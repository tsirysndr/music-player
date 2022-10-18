use async_graphql::MergedObject;

use self::{
    library::{LibraryMutation, LibraryQuery},
    mixer::{MixerMutation, MixerQuery},
    playback::{PlaybackMutation, PlaybackQuery},
    playlist::{PlaylistMutation, PlaylistQuery},
    tracklist::{TracklistMutation, TracklistQuery},
};

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
pub struct Query(
    LibraryQuery,
    MixerQuery,
    PlaybackQuery,
    PlaylistQuery,
    TracklistQuery,
);

#[derive(MergedObject, Default)]
pub struct Mutation(
    LibraryMutation,
    MixerMutation,
    PlaybackMutation,
    PlaylistMutation,
    TracklistMutation,
);
