use anyhow::Error;
use async_graphql::{Enum, MergedObject, MergedSubscription};
use music_player_addons::{
    chromecast::Chromecast, dlna::Dlna, jellyfin::Jellyfin, local::Local, subsonic::Subsonic,
    Browsable, Player,
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

pub async fn connect_to(device: Device) -> Result<Option<Box<dyn Browsable + Send>>, Error> {
    match device.app.as_str() {
        "subsonic" => {
            let mut subsonic: Subsonic = device.clone().into();
            subsonic.connect().await?;
            Ok(Some(Box::new(subsonic)))
        }
        "jellyfin" => {
            let mut jellyfin: Jellyfin = device.clone().into();
            jellyfin.connect().await?;
            Ok(Some(Box::new(jellyfin)))
        }
        _ => {
            let mut local: Local = device.clone().into();
            local.connect().await?;
            Ok(Some(Box::new(local)))
        }
    }
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
