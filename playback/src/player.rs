use async_trait::async_trait;
use music_player_entity::track::Model as Track;
use music_player_settings::read_settings;
use music_player_tracklist::{PlaybackState, Tracklist};
use rockbox_playback::{
    OutputConfig, PlaybackState as EngineState, Player as Engine, PlayerConfig,
};
use std::{sync::Arc, thread, time::Duration};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tracing::error;

pub type PlayerResult = Result<(), anyhow::Error>;

/// Minimum change in position before a `TrackTimePosition` event is broadcast.
const POSITION_BROADCAST_STEP_MS: u32 = 250;

pub enum RepeatState {
    Off,
    One,
    All,
}

#[async_trait]
pub trait PlayerEngine: Send + Sync {
    fn load(&mut self, track_id: &str, _start_playing: bool, _position_ms: u32);
    fn load_tracklist(&mut self, tracks: Vec<Track>);
    fn play(&self);
    fn pause(&self);
    fn stop(&self);
    fn seek(&self, position_ms: u32);
    fn next(&self);
    fn previous(&self);
    fn play_track_at(&self, index: usize);
    fn clear(&self);
    async fn get_tracks(&self) -> (Vec<Track>, Vec<Track>);
    async fn wait_for_tracklist(
        mut event: UnboundedReceiver<PlayerEvent>,
    ) -> (Vec<Track>, Vec<Track>);
    async fn get_current_track(&self) -> Option<(Option<Track>, usize, u32, bool)>;
    async fn wait_for_current_track(
        mut channel: UnboundedReceiver<PlayerEvent>,
    ) -> Option<(Option<Track>, usize, u32, bool)>;
}

#[derive(Clone)]
pub struct Player {
    commands: Option<Arc<std::sync::Mutex<mpsc::UnboundedSender<PlayerCommand>>>>,
}

impl Player {
    pub fn new<G>(
        event_broadcaster: G,
        cmd_tx: Arc<std::sync::Mutex<mpsc::UnboundedSender<PlayerCommand>>>,
        cmd_rx: Arc<std::sync::Mutex<mpsc::UnboundedReceiver<PlayerCommand>>>,
        tracklist: Arc<std::sync::Mutex<Tracklist>>,
    ) -> (Player, PlayerEventChannel)
    where
        G: Fn(PlayerEvent) + Send + 'static,
    {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let start = move || {
            // `audio_output` in settings.toml selects the output backend:
            // "cpal" (default), "stdout", "fifo:PATH", "unix:PATH" or "tcp:ADDR".
            let output = read_settings()
                .ok()
                .and_then(|config| config.get_string("audio_output").ok())
                .map(|spec| {
                    spec.parse::<OutputConfig>().unwrap_or_else(|e| {
                        error!("Invalid audio_output setting {:?}: {}", spec, e);
                        OutputConfig::Cpal
                    })
                })
                .unwrap_or(OutputConfig::Cpal);
            let engine = match PlayerConfig::builder().output(output).open() {
                Ok(engine) => engine,
                Err(e) => {
                    error!("Failed to open audio engine: {}", e);
                    return None;
                }
            };
            Some(PlayerInternal {
                commands: cmd_rx,
                engine,
                event_senders: [event_sender].to_vec(),
                tracklist,
                event_broadcaster: Box::new(event_broadcaster),
                position_ms: 0,
                last_broadcast_position_ms: 0,
                track_loaded: false,
                engine_started: false,
                last_duration_ms: 0,
                stopped_ticks: 0,
            })
        };

        // The engine handle is not Send (it wraps native state), so the async
        // player task runs on a small dedicated current-thread runtime instead
        // of the caller's work-stealing runtime.
        thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime");
            if let Some(internal) = start() {
                runtime.block_on(internal.run());
            }
        });
        (
            Player {
                commands: Some(cmd_tx),
            },
            event_receiver,
        )
    }

    fn command(&self, cmd: PlayerCommand) {
        if let Some(commands) = self.commands.as_ref() {
            if let Err(e) = commands.lock().unwrap().send(cmd) {
                error!("Player Commands Error: {}", e);
            }
        }
    }

    pub fn get_player_event_channel(&self) -> PlayerEventChannel {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        self.command(PlayerCommand::AddEventSender(event_sender));
        event_receiver
    }

    pub async fn await_end_of_track(&self) {
        let mut channel = self.get_player_event_channel();
        while let Some(event) = channel.recv().await {
            if matches!(
                event,
                PlayerEvent::EndOfTrack { .. } | PlayerEvent::Stopped { .. }
            ) {
                return;
            }
        }
    }

    pub async fn await_end_of_tracklist(&self) {
        let mut channel = self.get_player_event_channel();
        while let Some(event) = channel.recv().await {
            if matches!(event, PlayerEvent::EndOfTrack { .. })
                && event.get_is_last_track().unwrap_or(false)
            {
                return;
            }
        }
    }

    pub fn set_volume(&self, volume: u16) {
        self.command(PlayerCommand::SetVolume(volume));
    }
}

#[async_trait]
impl PlayerEngine for Player {
    fn load(&mut self, track_id: &str, _start_playing: bool, _position_ms: u32) {
        self.command(PlayerCommand::Load {
            track_id: track_id.to_string(),
        });
    }

    fn load_tracklist(&mut self, tracks: Vec<Track>) {
        self.command(PlayerCommand::LoadTracklist { tracks });
    }

    fn play(&self) {
        self.command(PlayerCommand::Play)
    }

    fn pause(&self) {
        self.command(PlayerCommand::Pause)
    }

    fn stop(&self) {
        self.command(PlayerCommand::Stop)
    }

    fn seek(&self, position_ms: u32) {
        self.command(PlayerCommand::Seek(position_ms));
    }

    fn next(&self) {
        self.command(PlayerCommand::Next);
    }

    fn previous(&self) {
        self.command(PlayerCommand::Previous);
    }

    fn play_track_at(&self, index: usize) {
        self.command(PlayerCommand::PlayTrackAt(index));
    }

    fn clear(&self) {
        self.command(PlayerCommand::Clear);
    }

    async fn get_tracks(&self) -> (Vec<Track>, Vec<Track>) {
        let channel = self.get_player_event_channel();
        self.command(PlayerCommand::GetTracks);
        Self::wait_for_tracklist(channel).await
    }

    async fn get_current_track(&self) -> Option<(Option<Track>, usize, u32, bool)> {
        let channel = self.get_player_event_channel();
        self.command(PlayerCommand::GetCurrentTrack);
        Self::wait_for_current_track(channel).await
    }

    async fn wait_for_tracklist(
        mut channel: UnboundedReceiver<PlayerEvent>,
    ) -> (Vec<Track>, Vec<Track>) {
        while let Some(event) = channel.recv().await {
            if matches!(event, PlayerEvent::TracklistUpdated { .. }) {
                return event.get_tracks().unwrap();
            }
        }
        (vec![], vec![])
    }

    async fn wait_for_current_track(
        mut channel: UnboundedReceiver<PlayerEvent>,
    ) -> Option<(Option<Track>, usize, u32, bool)> {
        while let Some(event) = channel.recv().await {
            if matches!(event, PlayerEvent::CurrentTrack { .. }) {
                return event.get_current_track();
            }
        }
        None
    }
}

struct PlayerInternal {
    commands: Arc<std::sync::Mutex<mpsc::UnboundedReceiver<PlayerCommand>>>,
    engine: Engine,
    event_senders: Vec<mpsc::UnboundedSender<PlayerEvent>>,
    tracklist: Arc<std::sync::Mutex<Tracklist>>,
    position_ms: u32,
    last_broadcast_position_ms: u32,
    /// A track has been handed to the engine and has not finished yet.
    track_loaded: bool,
    /// The engine has reported `Playing` since the last load; used to tell a
    /// finished track apart from one that is still buffering/probing.
    engine_started: bool,
    /// Duration of the current track as last reported while playing.
    last_duration_ms: u32,
    /// Consecutive status ticks spent in `Stopped` mid-track; a backstop so a
    /// decode failure still ends the track instead of wedging the queue.
    stopped_ticks: u32,
    event_broadcaster: Box<dyn Fn(PlayerEvent) + Send + 'static>,
}

impl PlayerInternal {
    /// The player task: reacts to commands as they arrive and reconciles the
    /// engine's status on a fixed tick. Ends when every command sender is gone.
    async fn run(mut self) {
        let mut tick = tokio::time::interval(Duration::from_millis(100));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            let commands = Arc::clone(&self.commands);
            tokio::select! {
                cmd = futures_util::future::poll_fn(move |cx| commands.lock().unwrap().poll_recv(cx)) => {
                    match cmd {
                        // client has disconnected - shut down.
                        None => return,
                        Some(cmd) => {
                            if let Err(e) = self.handle_command(cmd) {
                                error!("Error handling command: {}", e);
                            }
                        }
                    }
                }
                _ = tick.tick() => {
                    if self.track_loaded {
                        self.poll_engine();
                    }
                }
            }
        }
    }

    /// Reconcile the engine's status with the tracklist state and emit events.
    fn poll_engine(&mut self) {
        let status = self.engine.status();
        match status.state {
            EngineState::Playing | EngineState::Paused => {
                self.engine_started = true;
                self.stopped_ticks = 0;
                self.last_duration_ms = status.duration.as_millis() as u32;
                let position_ms = status.position.as_millis() as u32;
                self.position_ms = position_ms;
                let playback_state = self.tracklist.lock().unwrap().playback_state();
                self.tracklist
                    .lock()
                    .unwrap()
                    .set_playback_state(PlaybackState {
                        position_ms,
                        ..playback_state
                    });
                if position_ms.abs_diff(self.last_broadcast_position_ms)
                    >= POSITION_BROADCAST_STEP_MS
                {
                    self.last_broadcast_position_ms = position_ms;
                    (self.event_broadcaster)(PlayerEvent::TrackTimePosition { position_ms });
                }
            }
            EngineState::Stopped => {
                if self.engine_started {
                    // A reload (stop + set_queue + play) passes through a brief
                    // Stopped probe gap that a status tick can land in; a track
                    // that really ran to completion stops with its position at
                    // the end. Anything else is that gap — re-latch on the next
                    // Playing tick instead of advancing the queue.
                    self.stopped_ticks += 1;
                    let near_end = self.last_duration_ms == 0
                        || self.position_ms + 3000 >= self.last_duration_ms;
                    if !near_end && self.stopped_ticks < 30 {
                        return;
                    }
                    // the loaded track ran to completion
                    self.track_loaded = false;
                    self.engine_started = false;
                    let playback_state = self.tracklist.lock().unwrap().playback_state();
                    self.tracklist
                        .lock()
                        .unwrap()
                        .set_playback_state(PlaybackState {
                            is_playing: false,
                            ..playback_state
                        });
                    let is_last_track = self.tracklist.lock().unwrap().is_empty();
                    self.send_event(PlayerEvent::EndOfTrack { is_last_track });
                    self.handle_next();
                }
            }
        }
    }

    fn handle_command(&mut self, cmd: PlayerCommand) -> PlayerResult {
        match cmd {
            PlayerCommand::Load { track_id } => self.handle_command_load(&track_id),
            PlayerCommand::LoadTracklist { tracks } => self.handle_command_load_tracklist(tracks),
            PlayerCommand::Play => self.handle_play(),
            PlayerCommand::Pause => self.handle_pause(),
            PlayerCommand::Stop => self.handle_player_stop(),
            PlayerCommand::Seek(position_ms) => self.handle_command_seek(position_ms),
            PlayerCommand::AddEventSender(sender) => self.event_senders.push(sender),
            PlayerCommand::Next => self.handle_next(),
            PlayerCommand::Previous => self.handle_previous(),
            PlayerCommand::PlayTrackAt(index) => self.handle_play_track_at(index),
            PlayerCommand::Clear => self.handle_clear(),
            PlayerCommand::GetTracks => self.handle_get_tracks(),
            PlayerCommand::GetCurrentTrack => self.handle_get_current_track(),
            PlayerCommand::PlayNext(track) => self.handle_play_next(track),
            PlayerCommand::RemoveTrack(index) => self.handle_remove_track(index),
            PlayerCommand::SetVolume(volume) => self.handle_set_volume(volume),
        }
        Ok(())
    }

    fn send_event(&mut self, event: PlayerEvent) {
        self.event_senders
            .retain(|sender| sender.send(event.clone()).is_ok());
    }

    fn handle_command_load(&mut self, uri: &str) {
        self.engine.stop();
        self.engine.set_queue(vec![uri.to_string()]);
        self.engine.play();
        self.track_loaded = true;
        self.engine_started = false;
        self.stopped_ticks = 0;
        self.last_duration_ms = 0;
        self.position_ms = 0;
        self.last_broadcast_position_ms = 0;

        self.send_event(PlayerEvent::Playing {});
        let (track, position) = self.tracklist.lock().unwrap().current_track();
        self.tracklist
            .lock()
            .unwrap()
            .set_playback_state(PlaybackState {
                is_playing: true,
                position_ms: 0,
            });
        (self.event_broadcaster)(PlayerEvent::CurrentTrack {
            track,
            position,
            position_ms: 0,
            is_playing: true,
        });
    }

    fn handle_command_load_tracklist(&mut self, tracks: Vec<Track>) {
        self.tracklist.lock().unwrap().queue(tracks);
        let (current_track, _) = self.tracklist.lock().unwrap().current_track();
        if current_track.is_none() {
            self.handle_next();
        }
    }

    fn handle_play(&mut self) {
        self.engine.play();
        let playback_state = self.tracklist.lock().unwrap().playback_state();
        self.tracklist
            .lock()
            .unwrap()
            .set_playback_state(PlaybackState {
                is_playing: true,
                ..playback_state
            });
        self.send_event(PlayerEvent::Playing);
        let (track, position) = self.tracklist.lock().unwrap().current_track();
        (self.event_broadcaster)(PlayerEvent::CurrentTrack {
            track,
            position,
            position_ms: self.position_ms,
            is_playing: true,
        });
    }

    fn handle_pause(&mut self) {
        self.engine.pause();
        let playback_state = self.tracklist.lock().unwrap().playback_state();
        self.tracklist
            .lock()
            .unwrap()
            .set_playback_state(PlaybackState {
                is_playing: false,
                ..playback_state
            });
        self.send_event(PlayerEvent::Paused);
        let (track, position) = self.tracklist.lock().unwrap().current_track();
        (self.event_broadcaster)(PlayerEvent::CurrentTrack {
            track,
            position,
            position_ms: self.position_ms,
            is_playing: false,
        });
    }

    fn handle_player_stop(&mut self) {
        self.engine.stop();
        self.engine.clear_queue();
        self.track_loaded = false;
        self.engine_started = false;
        self.tracklist.lock().unwrap().stop();
    }

    fn handle_command_seek(&mut self, position_ms: u32) {
        self.engine.seek(Duration::from_millis(position_ms as u64));
        self.position_ms = position_ms;
        self.last_broadcast_position_ms = position_ms;
        let playback_state = self.tracklist.lock().unwrap().playback_state();
        self.tracklist
            .lock()
            .unwrap()
            .set_playback_state(PlaybackState {
                position_ms,
                ..playback_state
            });
        (self.event_broadcaster)(PlayerEvent::TrackTimePosition { position_ms });
    }

    fn handle_set_volume(&mut self, volume: u16) {
        let volume = volume.min(100);
        self.engine.set_volume(volume as f32 / 100.0);
        self.send_event(PlayerEvent::VolumeSet { volume });
    }

    fn handle_next(&mut self) {
        if self.tracklist.lock().unwrap().next_track().is_some() {
            let (current_track, _) = self.tracklist.lock().unwrap().current_track();
            self.handle_command_load(&current_track.unwrap().uri);
        }
    }

    fn handle_previous(&mut self) {
        if self.tracklist.lock().unwrap().previous_track().is_some() {
            let (current_track, _) = self.tracklist.lock().unwrap().current_track();
            self.handle_command_load(&current_track.unwrap().uri);
        }
    }

    fn handle_play_track_at(&mut self, index: usize) {
        let (current_track, _) = self.tracklist.lock().unwrap().play_track_at(index);
        if current_track.is_some() {
            self.handle_command_load(&current_track.unwrap().uri);
        }
    }

    fn handle_clear(&mut self) {
        self.tracklist.lock().unwrap().clear();
    }

    fn handle_get_tracks(&mut self) {
        let tracks = self.tracklist.lock().unwrap().tracks();
        self.send_event(PlayerEvent::TracklistUpdated { tracks });
    }

    fn handle_play_next(&mut self, track: Track) {
        self.tracklist.lock().unwrap().insert_next(track);
    }

    fn handle_remove_track(&mut self, index: usize) {
        self.tracklist.lock().unwrap().remove_track_at(index);
    }

    fn handle_get_current_track(&mut self) {
        let (track, position) = self.tracklist.lock().unwrap().current_track();
        let is_playing = self.track_loaded && self.engine.status().state == EngineState::Playing;
        self.send_event(PlayerEvent::CurrentTrack {
            track,
            position,
            position_ms: self.position_ms,
            is_playing,
        });
    }
}

#[derive(Debug)]
pub enum PlayerCommand {
    Load { track_id: String },
    LoadTracklist { tracks: Vec<Track> },
    Play,
    Pause,
    Stop,
    Seek(u32),
    Next,
    Previous,
    PlayTrackAt(usize),
    AddEventSender(mpsc::UnboundedSender<PlayerEvent>),
    Clear,
    GetTracks,
    GetCurrentTrack,
    RemoveTrack(usize),
    PlayNext(Track),
    SetVolume(u16),
}

#[derive(Debug, Clone)]
pub enum PlayerEvent {
    Stopped,
    Started,
    Loading,
    Playing,
    Paused,
    EndOfTrack {
        is_last_track: bool,
    },
    VolumeSet {
        volume: u16,
    },
    Error {
        track_id: String,
        error: String,
    },
    TracklistUpdated {
        tracks: (Vec<Track>, Vec<Track>),
    },
    CurrentTrack {
        track: Option<Track>,
        position: usize,
        position_ms: u32,
        is_playing: bool,
    },
    TrackTimePosition {
        position_ms: u32,
    },
}

impl PlayerEvent {
    pub fn get_is_last_track(&self) -> Option<bool> {
        use PlayerEvent::*;
        match self {
            EndOfTrack { is_last_track, .. } => Some(*is_last_track),
            _ => None,
        }
    }

    pub fn get_tracks(&self) -> Option<(Vec<Track>, Vec<Track>)> {
        use PlayerEvent::*;
        match self {
            TracklistUpdated { tracks, .. } => Some(tracks.clone()),
            _ => None,
        }
    }

    pub fn get_current_track(&self) -> Option<(Option<Track>, usize, u32, bool)> {
        use PlayerEvent::*;
        match self {
            CurrentTrack {
                track,
                position,
                position_ms,
                is_playing,
            } => Some((track.clone(), *position, *position_ms, *is_playing)),
            _ => None,
        }
    }
}

pub type PlayerEventChannel = mpsc::UnboundedReceiver<PlayerEvent>;
