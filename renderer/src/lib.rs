//! Where the audio comes out.
//!
//! A *renderer* is a sink: a Chromecast, a UPnP/DLNA device, or another
//! music-player daemon told to play something. It is deliberately the opposite
//! half of [`music_player_provider`], which is where the library is *read*
//! from — and keeping the two apart is what lets a user switch servers without
//! interrupting playback, because nothing on the provider side can reach a
//! renderer.
//!
//! This crate depends on `music-player-client` (and so on the server crate),
//! which is exactly why it cannot be merged back into `provider`: a provider
//! has to sit *below* the server so the gRPC layer can route library reads
//! through one.

pub mod chromecast;
pub mod dlna;
pub mod local;

use anyhow::Error;
use async_trait::async_trait;
use music_player_types::types::{Device, Playback, Track};

/// Something that can be handed tracks and told to play them.
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

/// The renderer currently playing, if it is not this machine.
///
/// `None` means the local engine, which is the ordinary case. Note what is
/// absent: nothing here knows about a library or a provider. Handing playback
/// to a speaker and pointing the screens at a different server are separate
/// acts, and neither can disturb the other.
pub struct CurrentReceiverDevice {
    pub client: Option<Box<dyn Player + Send>>,
    pub receiver_device: Option<Device>,
}

impl Default for CurrentReceiverDevice {
    fn default() -> Self {
        Self::new()
    }
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

    /// Drops the connection and returns what it was, so the caller can say
    /// what stopped.
    pub fn clear_client(&mut self) -> Option<Device> {
        self.client = None;
        self.receiver_device.take()
    }

    pub fn get_receiver_device(&self) -> Option<Device> {
        self.receiver_device.clone()
    }
}
