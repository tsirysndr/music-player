//! The connection to the daemon.
//!
//! Clients are made on first use and kept: an agent's session is a long run of
//! small calls, and a fresh gRPC channel per call would spend more time on
//! handshakes than on the request. Tonic's channel reconnects by itself, so a
//! daemon that restarts mid-session is picked up again without anything here
//! noticing.
//!
//! Connecting *lazily* also matters for a different reason: the host spawns
//! this process at startup, quite possibly before the daemon is running. A
//! server that refused to start without one would look broken to the user in a
//! way that has nothing to do with the daemon being down.

use anyhow::Error;
use music_player_client::{
    analysis::AnalysisClient, library::LibraryClient, playback::PlaybackClient,
    playlist::PlaylistClient, servers::ServersClient, tracklist::TracklistClient,
};

/// The daemon, as the tools see it.
pub struct Session {
    host: String,
    port: u16,
    playback: Option<PlaybackClient>,
    tracklist: Option<TracklistClient>,
    library: Option<LibraryClient>,
    playlist: Option<PlaylistClient>,
    servers: Option<ServersClient>,
    analysis: Option<AnalysisClient>,
}

impl Session {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            playback: None,
            tracklist: None,
            library: None,
            playlist: None,
            servers: None,
            analysis: None,
        }
    }

    pub async fn analysis(&mut self) -> Result<&mut AnalysisClient, Error> {
        if self.analysis.is_none() {
            self.analysis = Some(self.connect(AnalysisClient::new).await?);
        }
        Ok(self.analysis.as_mut().unwrap())
    }

    /// Where the daemon is, for error messages that would otherwise leave the
    /// user guessing which one failed.
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub async fn playback(&mut self) -> Result<&mut PlaybackClient, Error> {
        if self.playback.is_none() {
            self.playback = Some(self.connect(PlaybackClient::new).await?);
        }
        Ok(self.playback.as_mut().unwrap())
    }

    pub async fn tracklist(&mut self) -> Result<&mut TracklistClient, Error> {
        if self.tracklist.is_none() {
            self.tracklist = Some(self.connect(TracklistClient::new).await?);
        }
        Ok(self.tracklist.as_mut().unwrap())
    }

    pub async fn library(&mut self) -> Result<&mut LibraryClient, Error> {
        if self.library.is_none() {
            self.library = Some(self.connect(LibraryClient::new).await?);
        }
        Ok(self.library.as_mut().unwrap())
    }

    pub async fn playlist(&mut self) -> Result<&mut PlaylistClient, Error> {
        if self.playlist.is_none() {
            self.playlist = Some(self.connect(PlaylistClient::new).await?);
        }
        Ok(self.playlist.as_mut().unwrap())
    }

    pub async fn servers(&mut self) -> Result<&mut ServersClient, Error> {
        if self.servers.is_none() {
            self.servers = Some(self.connect(ServersClient::new).await?);
        }
        Ok(self.servers.as_mut().unwrap())
    }

    /// Connect, and say what to do about it if that fails.
    ///
    /// "transport error" on its own tells an agent nothing it can act on; being
    /// told the daemon is not running tells it to say so, or to offer to start
    /// it. Nothing is cached on failure, so the next call tries again.
    async fn connect<T, F, Fut>(&self, make: F) -> Result<T, Error>
    where
        F: FnOnce(String, u16) -> Fut,
        Fut: std::future::Future<Output = Result<T, Error>>,
    {
        make(self.host.clone(), self.port).await.map_err(|_| {
            Error::msg(format!(
                "No music-player daemon at {}. Start one with `music-player` and try again.",
                self.address()
            ))
        })
    }
}
