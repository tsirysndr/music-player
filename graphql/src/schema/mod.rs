use std::collections::HashMap;

use anyhow::Error;
use async_graphql::{Enum, MergedObject, MergedSubscription};
use music_player_addons::{
    airplay::Airplay, chromecast::Chromecast, kodi::Kodi, local::Local, Browseable, Player,
};
use music_player_types::types::Device;

use self::{
    devices::{DevicesMutation, DevicesQuery, DevicesSubscription},
    library::{LibraryMutation, LibraryQuery},
    mixer::{MixerMutation, MixerQuery},
    playback::{PlaybackMutation, PlaybackQuery, PlaybackSubscription},
    playlist::{PlaylistMutation, PlaylistQuery, PlaylistSubscription},
    tracklist::{TracklistMutation, TracklistQuery, TracklistSubscription},
};

pub mod addons;
pub mod core;
pub mod devices;
pub mod history;
pub mod library;
pub mod mixer;
pub mod objects;
pub mod playback;
pub mod playlist;
pub mod tracklist;

#[derive(MergedObject, Default)]
pub struct Query(
    DevicesQuery,
    LibraryQuery,
    MixerQuery,
    PlaybackQuery,
    PlaylistQuery,
    TracklistQuery,
);

#[derive(MergedObject, Default)]
pub struct Mutation(
    DevicesMutation,
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

pub async fn connect_to(device: Device) -> Result<Option<Box<dyn Browseable + Send>>, Error> {
    let mut local: Local = device.clone().into();
    local.connect().await?;
    Ok(Some(Box::new(local)))
}

pub enum PlayerType {
    MusicPlayer,
    Chromecast,
    Airplay,
    Kodi,
}

pub async fn connect_to_cast_device(
    device: Device,
    player_type: PlayerType,
) -> Result<Option<Box<dyn Player + Send>>, Error> {
    match player_type {
        PlayerType::MusicPlayer => Local::new().connect_to_player(device).await,
        PlayerType::Chromecast => Chromecast::connect(device),
        PlayerType::Airplay => Airplay::new().connect(device),
        PlayerType::Kodi => Kodi::new().connect_to_player(device),
    }
}
