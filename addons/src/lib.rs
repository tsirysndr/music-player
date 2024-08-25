pub mod airplay;
pub mod chromecast;
pub mod datpiff;
pub mod deezer;
pub mod dlna;
pub mod genius;
pub mod kodi;

use std::fs;

use anyhow::Error;
use async_trait::async_trait;
use extism::{Manifest, Plugin, PluginBuilder, UserData, Wasm, PTR};
use music_player_host_fn::{
    call, chromecast::*, get_addons, get_settings, player::*, register_addon, state::State,
    tracklist::*, upnp::*, with_capabilities,
};
use music_player_settings::get_application_directory;
use music_player_types::types::{Album, Artist, Device, Playback, Playlist, Track};

pub trait Addon {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn author(&self) -> &str;
    fn description(&self) -> &str;
    fn enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
}

pub trait StreamingAddon {
    fn stream(&self, url: &str) -> Result<(), Error>;
}

pub trait LyricsAddon {
    fn get_lyrics(&self, artist: &str, title: &str) -> Option<String>;
}

#[async_trait]
pub trait Browsable {
    async fn albums(
        &mut self,
        filter: Option<String>,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<Album>, Error>;
    async fn artists(
        &mut self,
        filter: Option<String>,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<Artist>, Error>;
    async fn tracks(
        &mut self,
        filter: Option<String>,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<Track>, Error>;
    async fn playlists(&mut self, offset: i32, limit: i32) -> Result<Vec<Playlist>, Error>;
    async fn album(&mut self, id: &str) -> Result<Album, Error>;
    async fn artist(&mut self, id: &str) -> Result<Artist, Error>;
    async fn track(&mut self, id: &str) -> Result<Track, Error>;
    async fn playlist(&mut self, id: &str) -> Result<Playlist, Error>;
    fn device_ip(&self) -> String;
}

#[async_trait]
pub trait Player {
    async fn play(&mut self) -> Result<(), Error>;
    async fn pause(&mut self) -> Result<(), Error>;
    async fn stop(&mut self) -> Result<(), Error>;
    async fn next(&mut self) -> Result<(), Error>;
    async fn previous(&mut self) -> Result<(), Error>;
    async fn seek(&mut self, position: u32) -> Result<(), Error>;
    async fn load_tracks(
        &mut self,
        tracks: Vec<Track>,
        start_index: Option<i32>,
    ) -> Result<(), Error>;
    async fn play_next(&mut self, track: Track) -> Result<(), Error>;
    async fn load(&mut self, track: Track) -> Result<(), Error>;
    async fn get_current_playback(&mut self) -> Result<Playback, Error>;
    async fn get_current_tracklist(&mut self) -> Result<(Vec<Track>, Vec<Track>), Error>;
    async fn play_track_at(&mut self, position: u32) -> Result<(), Error>;
    async fn remove_track_at(&mut self, position: u32) -> Result<(), Error>;
    fn device_type(&self) -> String;
    fn disconnect(&mut self) -> Result<(), Error>;
}

pub struct CurrentSourceDevice {
    pub client: Option<Box<dyn Browsable + Send>>,
    pub source_device: Option<Device>,
}

impl CurrentSourceDevice {
    pub fn new() -> Self {
        Self {
            client: None,
            source_device: None,
        }
    }

    pub fn set_client(&mut self, client: Box<dyn Browsable + Send>) {
        self.client = Some(client);
    }

    pub fn set_source_device(&mut self, device: Device) {
        self.source_device = Some(device);
    }

    pub fn clear_client(&mut self) -> Option<Device> {
        self.client = None;
        match self.source_device.take() {
            Some(device) => Some(device),
            None => None,
        }
    }

    pub fn get_source_device(&self) -> Option<Device> {
        match &self.source_device {
            Some(device) => Some(device.clone()),
            None => None,
        }
    }
}

pub struct CurrentReceiverDevice {
    pub client: Option<Box<dyn Player + Send>>,
    pub receiver_device: Option<Device>,
}

impl CurrentReceiverDevice {
    pub fn new() -> Self {
        Self {
            client: None,
            receiver_device: None,
        }
    }

    pub fn set_client(&mut self, client: Box<dyn Player + Send>) {
        self.client = Some(client);
    }

    pub fn set_receiver_device(&mut self, device: Device) {
        self.receiver_device = Some(device);
    }

    pub fn clear_client(&mut self) -> Option<Device> {
        self.client = None;
        match self.receiver_device.take() {
            Some(device) => Some(device),
            None => None,
        }
    }

    pub fn get_receiver_device(&self) -> Option<Device> {
        match &self.receiver_device {
            Some(device) => Some(device.clone()),
            None => None,
        }
    }
}

pub struct CurrentDevice {
    pub source: Option<Box<dyn Browsable + Send>>,
    pub receiver: Option<Box<dyn Player + Send>>,
    pub source_device: Option<Device>,
    pub receiver_device: Option<Device>,
}

impl CurrentDevice {
    pub fn new() -> Self {
        Self {
            source: None,
            receiver: None,
            source_device: None,
            receiver_device: None,
        }
    }

    pub fn set_source(&mut self, source: Box<dyn Browsable + Send>) {
        self.source = Some(source);
    }

    pub fn set_source_device(&mut self, device: Device) {
        self.source_device = Some(device);
    }

    pub fn clear_source(&mut self) -> Option<Device> {
        self.source = None;
        match self.source_device.take() {
            Some(device) => Some(device),
            None => None,
        }
    }

    pub fn set_receiver(&mut self, receiver: Box<dyn Player + Send>) {
        self.receiver = Some(receiver);
    }

    pub fn set_receiver_device(&mut self, device: Device) {
        self.receiver_device = Some(device);
    }

    pub fn clear_receiver(&mut self) -> Option<Device> {
        self.receiver = None;
        match self.receiver_device.take() {
            Some(device) => Some(device),
            None => None,
        }
    }

    pub fn get_source_device(&self) -> Option<Device> {
        match &self.source_device {
            Some(device) => Some(device.clone()),
            None => None,
        }
    }
}

pub fn load_plugin(module: &str, user_data: &UserData<State>) -> Result<Plugin, Error> {
    let app_dir = get_application_directory();
    let module = match fs::metadata(module.clone()).is_ok() || module.starts_with("http") {
        true => module,
        false => &format!("{}/addons/{}.wasm", app_dir, module),
    };

    let module = match module.starts_with("http") {
        true => Wasm::url(module),
        false => Wasm::file(module),
    };

    let manifest = Manifest::new([module]);
    let plugin = PluginBuilder::new(manifest.clone())
        .with_wasi(true)
        .with_function(
            "register_addon",
            [PTR],
            [],
            user_data.clone(),
            register_addon,
        )
        .with_function(
            "with_capabilities",
            [PTR],
            [],
            user_data.clone(),
            with_capabilities,
        )
        .with_function(
            "connect_to_chromecast",
            [PTR],
            [],
            user_data.clone(),
            connect_to_chromecast,
        )
        .with_function(
            "reconnect_to_chromecast",
            [],
            [],
            user_data.clone(),
            reconnect_to_chromecast,
        )
        .with_function(
            "send_command_to_chromecast",
            [PTR],
            [],
            user_data.clone(),
            send_command_to_chromecast,
        )
        .with_function(
            "chromecast_queue_load",
            [PTR],
            [],
            user_data.clone(),
            chromecast_queue_load,
        )
        .with_function(
            "load_track_to_chromecast",
            [PTR],
            [],
            user_data.clone(),
            load_track_to_chromecast,
        )
        .with_function(
            "get_chromecast_current_playback",
            [],
            [PTR],
            user_data.clone(),
            get_chromecast_current_playback,
        )
        .with_function(
            "disconnect_from_chromecast",
            [],
            [],
            user_data.clone(),
            disconnect_from_chromecast,
        )
        .with_function("load", [PTR], [], user_data.clone(), load)
        .with_function(
            "load_tracklist",
            [PTR],
            [],
            user_data.clone(),
            load_tracklist,
        )
        .with_function("preload", [PTR], [], user_data.clone(), preload)
        .with_function("play", [], [], user_data.clone(), play)
        .with_function("pause", [], [], user_data.clone(), pause)
        .with_function("stop", [], [], user_data.clone(), stop)
        .with_function("next", [], [], user_data.clone(), next)
        .with_function("previous", [], [], user_data.clone(), previous)
        .with_function("seek", [PTR], [], user_data.clone(), seek)
        .with_function("play_track_at", [PTR], [], user_data.clone(), play_track_at)
        .with_function("clear", [], [], user_data.clone(), clear)
        .with_function(
            "get_current_tracklist",
            [],
            [PTR],
            user_data.clone(),
            get_current_tracklist,
        )
        .with_function(
            "get_current_track",
            [],
            [PTR],
            user_data.clone(),
            get_current_track,
        )
        .with_function("play_next", [], [], user_data.clone(), play_next)
        .with_function("remove_track", [PTR], [], user_data.clone(), remove_track)
        .with_function(
            "connect_to_upnp_media_renderer",
            [],
            [],
            user_data.clone(),
            connect_to_upnp_media_renderer,
        )
        .with_function(
            "connect_to_upnp_media_server",
            [],
            [],
            user_data.clone(),
            connect_to_upnp_media_server,
        )
        .with_function(
            "browse_upnp_media_server",
            [],
            [],
            user_data.clone(),
            browse_upnp_media_server,
        )
        .with_function(
            "send_command_to_upnp_player",
            [PTR],
            [],
            user_data.clone(),
            send_command_to_upnp_player,
        )
        .with_function("get_settings", [], [PTR], user_data.clone(), get_settings)
        .with_function("get_addons", [], [PTR], user_data.clone(), get_addons)
        .with_function("call", [PTR], [], user_data.clone(), call)
        .build()?;
    Ok(plugin)
}
