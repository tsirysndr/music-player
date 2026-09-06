use async_trait::async_trait;
use music_player_entity::track::Model as Track;
use music_player_settings::{get_application_directory, read_settings, AudioSettings, Settings};
use music_player_tracklist::{PlaybackState, Tracklist};
use rockbox_playback::{
    CrossfadeMode, CrossfadeSettings, EqBand, Equalizer, MixMode, OutputConfig,
    PlaybackState as EngineState, Player as Engine, PlayerConfig, ReplayGainMode, EQ_BANDS,
    EQ_BAND_FREQUENCIES,
};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tracing::error;

pub type PlayerResult = Result<(), anyhow::Error>;

/// Minimum change in position before a `TrackTimePosition` event is broadcast.
const POSITION_BROADCAST_STEP_MS: u32 = 250;

/// Track-id prefix every internet-radio entry carries. It is what marks a
/// queue entry as a live stream, so it is also what arms ICY metadata.
pub const RADIO_ID_PREFIX: &str = "radio:";

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
            // Restore the persisted [audio] settings (EQ, tone, replaygain,
            // crossfade, dithering) from settings.toml.
            if let Some(settings) = read_settings()
                .ok()
                .and_then(|config| config.try_deserialize::<Settings>().ok())
            {
                apply_audio_settings(&engine, &settings.audio);
            }
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
                shuffle: false,
                repeat_mode: 0,
                engine_index: 0,
                resume: false,
                last_queue_save: Instant::now(),
                stopped_ticks: 0,
                icy_station: None,
                icy_last: None,
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
    /// Queue-level shuffle flag (the queue itself lives in the tracklist).
    shuffle: bool,
    /// Queue-level repeat mode: 0 off, 1 all, 2 one.
    repeat_mode: i32,
    /// The engine's queue index last observed. The engine holds the current
    /// track plus ONE lookahead (so crossfade/gapless transitions happen
    /// inside the engine); when its index moves past this, the engine
    /// advanced on its own and the tracklist has to catch up.
    engine_index: usize,
    /// Queue persistence armed. Off until `PlayerCommand::RestoreQueue`
    /// arrives (sent by daemon boot paths only), so short-lived players —
    /// tests, `music-player open` — neither restore nor overwrite the
    /// daemon's persisted queue.
    resume: bool,
    /// Last time the queue snapshot was written while playing.
    last_queue_save: Instant,
    /// Consecutive status ticks spent in `Stopped` mid-track; a backstop so a
    /// decode failure still ends the track instead of wedging the queue.
    stopped_ticks: u32,
    /// The pristine station entry of the live stream that is playing, kept so
    /// every ICY refresh folds onto the original instead of onto the previous
    /// song. `None` for anything that is not internet radio.
    icy_station: Option<Track>,
    /// Last ICY snapshot folded into the tracklist, so an unchanged
    /// `StreamTitle` costs nothing.
    icy_last: Option<IcySnapshot>,
    event_broadcaster: Box<dyn Fn(PlayerEvent) + Send + 'static>,
}

/// The parts of the engine's metadata that a live stream actually moves:
/// the ICY `StreamTitle` (split into artist/title), the `icy-name` station,
/// and the format numbers that are only known once decoding starts.
#[derive(Clone, Default, PartialEq)]
struct IcySnapshot {
    title: String,
    artist: String,
    station: String,
    genre: String,
    bitrate: u32,
    sample_rate: u32,
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

    /// Write the queue snapshot (or remove it when the queue is empty).
    /// No-op until persistence was armed by `RestoreQueue`.
    fn save_queue(&mut self) {
        if !self.resume {
            return;
        }
        let (played, tracks) = self.tracklist.lock().unwrap().tracks();
        let path = queue_file();
        if played.is_empty() && tracks.is_empty() {
            let _ = std::fs::remove_file(&path);
            return;
        }
        let saved = SavedQueue {
            played,
            tracks,
            position_ms: self.position_ms,
        };
        match serde_json::to_string(&saved) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    error!("failed to persist queue: {}", e);
                }
            }
            Err(e) => error!("failed to serialize queue: {}", e),
        }
        self.last_queue_save = Instant::now();
    }

    /// Restore the persisted queue on boot: rebuild the tracklist split and
    /// cue the current track paused at the saved position — a daemon that
    /// started blaring music on boot would be a surprise.
    fn restore_queue(&mut self) {
        let Ok(raw) = std::fs::read_to_string(queue_file()) else {
            return;
        };
        let Ok(saved) = serde_json::from_str::<SavedQueue>(&raw) else {
            return;
        };
        if saved.played.is_empty() && saved.tracks.is_empty() {
            return;
        }
        let current_uri = saved.played.last().map(|t| t.uri.clone());
        self.tracklist
            .lock()
            .unwrap()
            .restore(saved.played, saved.tracks, saved.position_ms);
        let Some(uri) = current_uri else { return };
        self.engine.stop();
        self.engine.set_queue(vec![uri]);
        self.engine.play();
        self.engine.pause();
        if saved.position_ms > 0 {
            self.engine
                .seek(Duration::from_millis(saved.position_ms as u64));
        }
        self.track_loaded = true;
        self.engine_started = false;
        self.engine_index = 0;
        self.position_ms = saved.position_ms;
        self.last_broadcast_position_ms = saved.position_ms;
        self.queue_next_into_engine();
        self.arm_icy();
        let (track, position) = self.tracklist.lock().unwrap().current_track();
        (self.event_broadcaster)(PlayerEvent::CurrentTrack {
            track: track.clone(),
            position,
            position_ms: saved.position_ms,
            is_playing: false,
        });
        self.send_event(PlayerEvent::CurrentTrack {
            track,
            position,
            position_ms: saved.position_ms,
            is_playing: false,
        });
    }

    /// Reconcile the engine's status with the tracklist state and emit events.
    fn poll_engine(&mut self) {
        let status = self.engine.status();
        match status.state {
            EngineState::Playing | EngineState::Paused => {
                self.engine_started = true;
                self.stopped_ticks = 0;
                // The engine advanced into its lookahead track on its own
                // (crossfade / gapless transition) — catch the tracklist up
                // and queue the next lookahead.
                if let Some(index) = status.index {
                    let mut advanced = false;
                    while index > self.engine_index {
                        self.engine_index += 1;
                        advanced = true;
                        self.send_event(PlayerEvent::EndOfTrack {
                            is_last_track: false,
                        });
                        if self.tracklist.lock().unwrap().next_track().is_some() {
                            let (track, position) = self.tracklist.lock().unwrap().current_track();
                            self.send_event(PlayerEvent::Playing {});
                            (self.event_broadcaster)(PlayerEvent::CurrentTrack {
                                track,
                                position,
                                position_ms: 0,
                                is_playing: status.state == EngineState::Playing,
                            });
                        }
                    }
                    if advanced {
                        self.queue_next_into_engine();
                        self.arm_icy();
                        self.save_queue();
                    }
                }
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
                if self.last_queue_save.elapsed() >= QUEUE_SAVE_INTERVAL {
                    self.save_queue();
                }
                self.refresh_icy(&status);
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
                    // Repeat one: reload the finished track and stay put.
                    if self.repeat_mode == 2 {
                        self.send_event(PlayerEvent::EndOfTrack {
                            is_last_track: false,
                        });
                        let (current_track, _) = self.tracklist.lock().unwrap().current_track();
                        if let Some(track) = current_track {
                            self.handle_command_load(&track.uri);
                        }
                        return;
                    }
                    let is_last_track = self.tracklist.lock().unwrap().is_empty();
                    self.send_event(PlayerEvent::EndOfTrack {
                        is_last_track: is_last_track && self.repeat_mode == 0,
                    });
                    if is_last_track && self.repeat_mode == 1 {
                        // Repeat all: wrap back to the start of the queue.
                        self.handle_play_track_at(0);
                        return;
                    }
                    self.handle_next();
                }
            }
        }
    }

    /// Latch the pristine station entry when the track that just started is
    /// internet radio, so [`Self::refresh_icy`] has a base to fold onto.
    /// Clears the overlay for anything else — a local file carries its own
    /// tags and must never be rewritten from the engine.
    fn arm_icy(&mut self) {
        let (track, _) = self.tracklist.lock().unwrap().current_track();
        self.icy_last = None;
        self.icy_station = track.filter(|t| t.id.starts_with(RADIO_ID_PREFIX));
    }

    /// Fold the live stream's ICY metadata onto the station entry so every
    /// consumer of the tracklist — the gRPC/GraphQL now-playing, the desktop
    /// bar, the web UI — shows the song that is on the air rather than the
    /// station name forever. A no-op for anything but internet radio, and
    /// cheap while the `StreamTitle` holds still.
    fn refresh_icy(&mut self, status: &rockbox_playback::Status) {
        let Some(station) = self.icy_station.clone() else {
            return;
        };
        let snapshot = match status.metadata.as_ref() {
            Some(meta) => IcySnapshot {
                title: meta.title.trim().to_string(),
                artist: meta.artist.trim().to_string(),
                station: meta.album.trim().to_string(),
                genre: meta.genre.trim().to_string(),
                bitrate: meta.bitrate,
                sample_rate: meta.sample_rate,
            },
            None => IcySnapshot::default(),
        };
        if self.icy_last.as_ref() == Some(&snapshot) {
            return;
        }
        self.icy_last = Some(snapshot.clone());

        // `icy-name` is the station's own name; fall back to the directory's
        // when the server does not send one.
        let station_name = if snapshot.station.is_empty() {
            station.title.clone()
        } else {
            snapshot.station.clone()
        };
        let mut track = station.clone();
        track.album.title = station_name.clone();
        if !snapshot.title.is_empty() {
            track.title = snapshot.title.clone();
            // A bare `StreamTitle` with no " - " leaves the artist slot free;
            // the station reads better there than the directory's source name.
            track.artist = if snapshot.artist.is_empty() {
                station_name
            } else {
                snapshot.artist.clone()
            };
        }
        if !snapshot.genre.is_empty() {
            track.genre = snapshot.genre.clone();
        }
        if snapshot.bitrate > 0 {
            track.bitrate = Some(snapshot.bitrate);
        }
        if snapshot.sample_rate > 0 {
            track.sample_rate = Some(snapshot.sample_rate);
        }

        let position = {
            let mut tracklist = self.tracklist.lock().unwrap();
            tracklist.update_current_track(track.clone());
            tracklist.current_track().1
        };
        let is_playing = status.state == EngineState::Playing;
        (self.event_broadcaster)(PlayerEvent::CurrentTrack {
            track: Some(track.clone()),
            position,
            position_ms: self.position_ms,
            is_playing,
        });
        self.send_event(PlayerEvent::CurrentTrack {
            track: Some(track),
            position,
            position_ms: self.position_ms,
            is_playing,
        });
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
            PlayerCommand::RestoreQueue => {
                self.resume = true;
                self.restore_queue();
            }
            PlayerCommand::SetEqEnabled(enabled) => self.engine.set_eq_enabled(enabled),
            PlayerCommand::SetEqBandGain { band, gain_db } => {
                if band < EQ_BANDS {
                    self.engine.set_eq_band(
                        band,
                        EqBand {
                            cutoff_hz: EQ_BAND_FREQUENCIES[band],
                            q: 1.0,
                            gain_db,
                        },
                    );
                }
            }
            PlayerCommand::SetEqPrecut(db) => self.engine.set_eq_precut(db),
            PlayerCommand::SetBass(db) => self.engine.set_bass(db),
            PlayerCommand::SetTreble(db) => self.engine.set_treble(db),
            PlayerCommand::SetBalance(balance) => self.engine.set_balance(balance),
            PlayerCommand::SetReplaygain {
                mode,
                preamp_db,
                prevent_clipping,
            } => self
                .engine
                .set_replaygain(replaygain_mode(mode), preamp_db, prevent_clipping),
            PlayerCommand::SetCrossfade {
                mode,
                fade_in_delay,
                fade_in_duration,
                fade_out_delay,
                fade_out_duration,
                mix_mode,
            } => self.engine.set_crossfade(crossfade_settings(
                mode,
                fade_in_delay,
                fade_in_duration,
                fade_out_delay,
                fade_out_duration,
                mix_mode,
            )),
            PlayerCommand::SetDither(enabled) => self.engine.set_dither(enabled),
            // The queue lives in the tracklist (the engine only holds the
            // current track + one lookahead), so shuffle/repeat act here.
            PlayerCommand::SetShuffle(enabled) => {
                self.shuffle = enabled;
                if enabled {
                    self.tracklist.lock().unwrap().shuffle();
                }
                self.resync_engine_next();
            }
            PlayerCommand::SetRepeat(mode) => {
                self.repeat_mode = mode.clamp(0, 2);
                // Repeat-one must drop the lookahead (the engine has to stop
                // at track end); leaving it re-queues the lookahead.
                self.resync_engine_next();
            }
        }
        Ok(())
    }

    fn send_event(&mut self, event: PlayerEvent) {
        self.event_senders
            .retain(|sender| sender.send(event.clone()).is_ok());
    }

    /// Queue the tracklist's upcoming track into the engine as lookahead,
    /// so the engine performs the transition itself (crossfade / gapless).
    /// Repeat-one skips the lookahead — the engine must stop at track end so
    /// the Stopped handler can reload the same track.
    fn queue_next_into_engine(&mut self) {
        if self.repeat_mode == 2 {
            return;
        }
        if let Some(next) = self.tracklist.lock().unwrap().peek_next() {
            self.engine.insert_last(next.uri);
        }
    }

    /// Drop the engine's lookahead (if any) and re-queue the CURRENT
    /// upcoming track. Call after anything that changes what comes next:
    /// play-next inserts, queue removals, appends, shuffle, repeat changes.
    fn resync_engine_next(&mut self) {
        if !self.track_loaded {
            return;
        }
        self.engine.remove(self.engine_index + 1);
        self.queue_next_into_engine();
    }

    fn handle_command_load(&mut self, uri: &str) {
        self.engine.stop();
        self.engine.set_queue(vec![uri.to_string()]);
        self.engine.play();
        self.track_loaded = true;
        self.engine_started = false;
        self.engine_index = 0;
        self.stopped_ticks = 0;
        self.last_duration_ms = 0;
        self.position_ms = 0;
        self.last_broadcast_position_ms = 0;
        self.queue_next_into_engine();
        self.arm_icy();

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
        self.save_queue();
    }

    fn handle_command_load_tracklist(&mut self, tracks: Vec<Track>) {
        self.tracklist.lock().unwrap().queue(tracks);
        if self.shuffle {
            self.tracklist.lock().unwrap().shuffle();
        }
        let (current_track, _) = self.tracklist.lock().unwrap().current_track();
        if current_track.is_none() {
            self.handle_next();
        } else {
            // Appended while playing — the lookahead may have been empty.
            self.resync_engine_next();
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
        self.engine_index = 0;
        self.icy_station = None;
        self.icy_last = None;
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
        if self.resume {
            let _ = std::fs::remove_file(queue_file());
        }
    }

    fn handle_get_tracks(&mut self) {
        let tracks = self.tracklist.lock().unwrap().tracks();
        self.send_event(PlayerEvent::TracklistUpdated { tracks });
    }

    fn handle_play_next(&mut self, track: Track) {
        self.tracklist.lock().unwrap().insert_next(track);
        self.resync_engine_next();
    }

    fn handle_remove_track(&mut self, index: usize) {
        self.tracklist.lock().unwrap().remove_track_at(index);
        self.resync_engine_next();
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
    Load {
        track_id: String,
    },
    LoadTracklist {
        tracks: Vec<Track>,
    },
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
    /// Arm queue persistence and restore the persisted queue (cued paused at
    /// the saved position). Sent once at boot by daemon entry points only.
    RestoreQueue,
    // Audio/DSP settings (integer enums follow the Rockbox firmware
    // conventions documented on music_player_settings::AudioSettings).
    SetEqEnabled(bool),
    SetEqBandGain {
        band: usize,
        gain_db: f32,
    },
    SetEqPrecut(f32),
    SetBass(i32),
    SetTreble(i32),
    SetBalance(i32),
    SetReplaygain {
        mode: i32,
        preamp_db: f32,
        prevent_clipping: bool,
    },
    SetCrossfade {
        mode: i32,
        fade_in_delay: u64,
        fade_in_duration: u64,
        fade_out_delay: u64,
        fade_out_duration: u64,
        mix_mode: i32,
    },
    SetDither(bool),
    SetShuffle(bool),
    SetRepeat(i32),
}

pub fn replaygain_mode(mode: i32) -> ReplayGainMode {
    match mode {
        0 | 2 => ReplayGainMode::Track, // 2 = "track (shuffle)" in the UI
        1 => ReplayGainMode::Album,
        _ => ReplayGainMode::Off,
    }
}

pub fn crossfade_settings(
    mode: i32,
    fade_in_delay: u64,
    fade_in_duration: u64,
    fade_out_delay: u64,
    fade_out_duration: u64,
    mix_mode: i32,
) -> CrossfadeSettings {
    CrossfadeSettings {
        mode: match mode {
            1 => CrossfadeMode::AutoSkip,
            2 => CrossfadeMode::ManualSkip,
            3 => CrossfadeMode::Shuffle,
            4 => CrossfadeMode::ShuffleOrManualSkip,
            5 => CrossfadeMode::Always,
            _ => CrossfadeMode::Off,
        },
        fade_in_delay: Duration::from_secs(fade_in_delay.min(7)),
        fade_in_duration: Duration::from_secs(fade_in_duration.min(15)),
        fade_out_delay: Duration::from_secs(fade_out_delay.min(7)),
        fade_out_duration: Duration::from_secs(fade_out_duration.min(15)),
        mix_mode: if mix_mode == 2 {
            MixMode::Mix
        } else {
            MixMode::Crossfade
        },
    }
}

/// Persisted queue snapshot: the exact played/upcoming split plus the
/// position within the current track, so a restart comes back cued paused
/// where it left off.
#[derive(Serialize, Deserialize, Default)]
struct SavedQueue {
    played: Vec<Track>,
    tracks: Vec<Track>,
    position_ms: u32,
}

fn queue_file() -> PathBuf {
    PathBuf::from(get_application_directory())
        .join("cache")
        .join("queue.json")
}

/// How often the queue snapshot is refreshed while playing (track changes
/// save immediately).
const QUEUE_SAVE_INTERVAL: Duration = Duration::from_secs(5);

/// Push the persisted `[audio]` settings into a freshly opened engine.
pub fn apply_audio_settings(engine: &Engine, audio: &AudioSettings) {
    let bands = EQ_BAND_FREQUENCIES
        .iter()
        .enumerate()
        .map(|(i, &cutoff_hz)| EqBand {
            cutoff_hz,
            q: 1.0,
            gain_db: audio.eq_band_gains.get(i).copied().unwrap_or(0.0),
        })
        .collect();
    engine.set_equalizer(Equalizer {
        enabled: audio.eq_enabled,
        precut_db: audio.eq_precut,
        bands,
    });
    engine.set_bass(audio.bass);
    engine.set_treble(audio.treble);
    engine.set_balance(audio.balance);
    engine.set_replaygain(
        replaygain_mode(audio.replaygain_mode),
        audio.replaygain_preamp,
        audio.replaygain_noclip,
    );
    engine.set_crossfade(crossfade_settings(
        audio.crossfade,
        audio.fade_in_delay,
        audio.fade_in_duration,
        audio.fade_out_delay,
        audio.fade_out_duration,
        audio.fade_out_mixmode,
    ));
    engine.set_dither(audio.dithering);
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
