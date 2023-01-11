use std::collections::HashMap;

use anyhow::Error;
use async_graphql::{Enum, MergedObject, MergedSubscription};
use music_player_addons::{local::Local, Browseable};
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

pub async fn connect_to_current_device(
    devices: HashMap<String, Device>,
) -> Result<Option<Box<dyn Browseable + Send>>, Error> {
    match devices.get("current_device") {
        Some(current_device) => {
            let mut local: Local = current_device.clone().into();
            local.connect().await?;
            Ok(Some(Box::new(local)))
        }
        None => Ok(None),
    }
}

pub async fn connect_to(device: Device) -> Result<Option<Box<dyn Browseable + Send>>, Error> {
    let mut local: Local = device.clone().into();
    local.connect().await?;
    Ok(Some(Box::new(local)))
}
