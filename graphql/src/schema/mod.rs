use async_graphql::{Enum, MergedObject, MergedSubscription};

use self::{
    library::{LibraryMutation, LibraryQuery},
    mixer::{MixerMutation, MixerQuery},
    playback::{PlaybackMutation, PlaybackQuery, PlaybackSubscription},
    playlist::{PlaylistMutation, PlaylistQuery, PlaylistSubscription},
    tracklist::{TracklistMutation, TracklistQuery, TracklistSubscription},
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

#[derive(MergedSubscription, Default)]
pub struct Subscription(
    PlaybackSubscription,
    PlaylistSubscription,
    TracklistSubscription,
);

#[derive(Enum, Eq, PartialEq, Copy, Clone)]
pub enum MutationType {
    Created,
    Cleared,
    Deleted,
    Renamed,
    Moved,
    Updated,
}
