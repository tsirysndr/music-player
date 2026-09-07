use anyhow::Error;
use async_graphql::{Enum, MergedObject, MergedSubscription};
use music_player_addons::{chromecast::Chromecast, dlna::Dlna, local::Local, Browsable, Player};
use music_player_provider::backends::{jellyfin::Jellyfin, subsonic::Subsonic};
use music_player_settings::{read_settings, Settings};
use music_player_types::types::Device;

use self::{
    devices::{DevicesMutation, DevicesQuery, DevicesSubscription},
    extensions::{ExtensionsMutation, ExtensionsQuery},
    library::{LibraryMutation, LibraryQuery},
    mixer::{MixerMutation, MixerQuery},
    playback::{PlaybackMutation, PlaybackQuery, PlaybackSubscription},
    playlist::{PlaylistMutation, PlaylistQuery, PlaylistSubscription},
    radio::{RadioMutation, RadioQuery},
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
            // Credentials come from the saved-server row now, not from
            // `settings.toml`. This path is the last caller that still has
            // only a `Device`; it goes when `connect_to_device` moves onto
            // `ProviderState`.
            let base_url = device
                .base_url
                .clone()
                .unwrap_or_else(|| format!("http://{}:{}", device.host, device.port));
            let settings = read_settings()
                .ok()
                .and_then(|config| config.try_deserialize::<Settings>().ok());
            let (username, password) = settings
                .map(|settings| {
                    (
                        settings.subsonic_username.unwrap_or_default(),
                        settings.subsonic_password.unwrap_or_default(),
                    )
                })
                .unwrap_or_default();
            let mut subsonic = Subsonic::with_credentials(&base_url, &username, &password);
            subsonic.connect().await?;
            Ok(Some(Box::new(subsonic)))
        }
        "jellyfin" => {
            let base_url = device
                .base_url
                .clone()
                .unwrap_or_else(|| format!("http://{}:{}", device.host, device.port));
            let settings = read_settings()
                .ok()
                .and_then(|config| config.try_deserialize::<Settings>().ok());
            let (username, password) = settings
                .map(|settings| {
                    (
                        settings.jellyfin_username.unwrap_or_default(),
                        settings.jellyfin_password.unwrap_or_default(),
                    )
                })
                .unwrap_or_default();
            let mut jellyfin = Jellyfin::with_credentials(&base_url, &username, &password);
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
