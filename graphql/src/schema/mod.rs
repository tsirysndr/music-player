use anyhow::Error;
use async_graphql::{Enum, MergedObject, MergedSubscription};
use music_player_renderer::{chromecast::Chromecast, dlna::Dlna, local::Local, Player};
use music_player_types::types::Device;

use self::{
    devices::{DevicesMutation, DevicesQuery, DevicesSubscription},
    extensions::{ExtensionsMutation, ExtensionsQuery},
    library::{LibraryMutation, LibraryQuery},
    mixer::{MixerMutation, MixerQuery},
    playback::{PlaybackMutation, PlaybackQuery, PlaybackSubscription},
    playlist::{PlaylistMutation, PlaylistQuery, PlaylistSubscription},
    radio::{RadioMutation, RadioQuery},
    servers::{ServersMutation, ServersQuery},
    tracklist::{TracklistMutation, TracklistQuery, TracklistSubscription},
};

pub mod addons;
pub mod core;
pub mod devices;
pub mod extensions;
pub mod history;
pub mod library;
pub mod mixer;
pub mod objects;
pub mod playback;
pub mod provider;
pub mod playlist;
pub mod radio;
pub mod servers;
pub mod tracklist;

#[derive(MergedObject, Default)]
pub struct Query(
    DevicesQuery,
    LibraryQuery,
    MixerQuery,
    PlaybackQuery,
    PlaylistQuery,
    TracklistQuery,
    RadioQuery,
    ExtensionsQuery,
    ServersQuery,
);

#[derive(MergedObject, Default)]
pub struct Mutation(
    DevicesMutation,
    LibraryMutation,
    MixerMutation,
    PlaybackMutation,
    PlaylistMutation,
    TracklistMutation,
    RadioMutation,
    ExtensionsMutation,
    ServersMutation,
);

#[derive(MergedSubscription, Default)]
pub struct Subscription(
    PlaybackSubscription,
    PlaylistSubscription,
    TracklistSubscription,
    DevicesSubscription,
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

pub enum PlayerType {
    MusicPlayer,
    Chromecast,
    Dlna,
}

pub async fn connect_to_cast_device(
    device: Device,
    player_type: PlayerType,
) -> Result<Option<Box<dyn Player + Send>>, Error> {
    match player_type {
        PlayerType::MusicPlayer => Local::new().connect_to_player(device).await,
        PlayerType::Chromecast => Chromecast::connect(device),
        PlayerType::Dlna => Dlna::connect_to_media_renderer(device),
    }
}
