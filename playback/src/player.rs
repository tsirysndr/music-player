use crate::{
    managed::{DesiredState, Phase},
    source_playback::{SourceCheckpoint, SourcePlayback},
    source_resolver::SourceResolver,
};
use async_trait::async_trait;
use music_player_entity::track::Model as Track;
use music_player_settings::{get_application_directory, read_settings, AudioSettings, Settings};
use music_player_tracklist::{validate_entries, PlaybackState, QueueEntry, Tracklist};
use music_player_types::audio::AudioSelection;
use music_player_types::source::{normalize_source_track, ResourceKind, SourceHandle};
use rockbox_playback::{
    CrossfadeMode, CrossfadeSettings, EqBand, Equalizer, MixMode, OutputConfig,
    PlaybackState as EngineState, Player as Engine, PlayerConfig, ReplayGainMode, EQ_BANDS,
    EQ_BAND_FREQUENCIES,
};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tracing::error;

pub type PlayerResult = Result<(), anyhow::Error>;

/// Work out a local track's key and tempo, if they are not known yet.
///
/// Playing a track is the strongest possible signal that its row is worth
/// filling in — it is the one the user is looking at. Detached and serialised
/// behind the same single analysis slot as everything else, so this never
/// competes with the audio it was triggered by.
///
/// Local files only. A remote track is not a row in the `track` table, so there
/// is nothing to write these to.
fn analyse_in_background(track: &Track) {
    if SourceHandle::is_handle(&track.id)
        || SourceHandle::is_handle(&track.uri)
        || track.uri.starts_with("http://")
        || track.uri.starts_with("https://")
    {
        return;
    }
    if track.key.is_some() && track.bpm.is_some() {
        return;
    }

    let reference = music_player_storage::track_analysis::TrackRef {
        id: track.id.clone(),
        uri: track.uri.clone(),
        artist: track.artist.clone(),
        title: track.title.clone(),
    };
    tokio::spawn(async move {
        let db = music_player_storage::shared().await;
        if let Err(cause) =
            music_player_storage::track_analysis::ensure(db.get_connection(), "", &reference).await
        {
            tracing::debug!(track = %reference.title, %cause, "could not analyse");
        }
    });
}

/// Cache a finite remote track, in the background.
///
/// Detached, and serialised by the cache itself: several downloads at once
/// would compete with the playing stream for the same connection, which is the
/// stutter this whole mechanism exists to remove.
fn cache_in_background(uri: &str) {
    if !music_player_storage::track_cache::enabled()
        || !music_player_storage::track_cache::is_cacheable(uri)
    {
        return;
    }
    let uri = uri.to_string();
    tokio::spawn(async move {
        // "already downloading" and "already cached" both land here; neither
        // is worth a warning.
        if let Err(e) = music_player_storage::track_cache::store(&uri).await {
            tracing::debug!("could not cache: {e}");
        }
    });
}

/// Minimum change in position before a `TrackTimePosition` event is broadcast.
const POSITION_BROADCAST_STEP_MS: u32 = 250;

/// How long a loaded track may stay silent before the open is called failed.
///
/// Generous on purpose: opening a remote track is a probe plus a header fetch
/// over someone else's connection, and the engine reports `Stopped` for the
/// whole of it. Only silence well past that is a failure rather than a slow
/// server.
const OPEN_TIMEOUT: Duration = Duration::from_secs(45);

/// How many times a track that will not open is loaded again before it is let
/// go. Two retries cover a blip; more would be a track that is simply broken.
const MAX_OPEN_ATTEMPTS: u32 = 2;

/// Track-id prefix every internet-radio entry carries. It is what marks a
/// queue entry as a live stream, so it is also what arms ICY metadata.
pub const RADIO_ID_PREFIX: &str = "radio:";

/// What a live stream announced last, as it came off the wire.
///
/// The tracklist overlay cannot answer this: a `StreamTitle` with no " - " is
/// folded in with the station's name in the artist slot — which reads well in
/// a now-playing bar but is not an artist — and a station that announces
/// nothing at all leaves the station entry's own fields in place. Anything
/// that must tell a real song from those stand-ins (the scrobbler) needs the
/// unfolded parse, so it is published here.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IcyNowPlaying {
    /// The artist half of `Artist - Title`. Empty when the station sent a bare
    /// title, or sent nothing.
    pub artist: String,
    /// The title half, or the whole `StreamTitle` when it does not split.
    /// Empty until the first metadata block arrives.
    pub title: String,
    /// `icy-name`, falling back to the station entry's own name.
    pub station: String,
}

/// Last [`IcyNowPlaying`], or `None` when what is playing is not radio.
static ICY_NOW_PLAYING: Mutex<Option<IcyNowPlaying>> = Mutex::new(None);

/// What the live stream that is playing announced last. `None` whenever the
/// current track is not internet radio.
pub fn icy_now_playing() -> Option<IcyNowPlaying> {
    ICY_NOW_PLAYING.lock().unwrap().clone()
}

fn publish_icy(now_playing: Option<IcyNowPlaying>) {
    *ICY_NOW_PLAYING.lock().unwrap() = now_playing;
}

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
        Self::new_inner(event_broadcaster, cmd_tx, cmd_rx, tracklist, None)
    }

    /// Daemon entry points supply the saved-account resolver; short-lived local
    /// players retain the existing constructor and reject managed handles.
    pub fn with_source_resolver<G>(
        event_broadcaster: G,
        cmd_tx: Arc<std::sync::Mutex<mpsc::UnboundedSender<PlayerCommand>>>,
        cmd_rx: Arc<std::sync::Mutex<mpsc::UnboundedReceiver<PlayerCommand>>>,
        tracklist: Arc<std::sync::Mutex<Tracklist>>,
        resolver: SourceResolver,
    ) -> (Player, PlayerEventChannel)
    where
        G: Fn(PlayerEvent) + Send + 'static,
    {
        Self::new_inner(event_broadcaster, cmd_tx, cmd_rx, tracklist, Some(resolver))
    }

    fn new_inner<G>(
        event_broadcaster: G,
        cmd_tx: Arc<std::sync::Mutex<mpsc::UnboundedSender<PlayerCommand>>>,
        cmd_rx: Arc<std::sync::Mutex<mpsc::UnboundedReceiver<PlayerCommand>>>,
        tracklist: Arc<std::sync::Mutex<Tracklist>>,
        resolver: Option<SourceResolver>,
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
                source_playback: resolver.map(SourcePlayback::new),
                source_phase: None,
                music_crossfade: read_settings()
                    .ok()
                    .and_then(|c| c.try_deserialize::<Settings>().ok())
                    .map(|c| {
                        crossfade_settings(
                            c.audio.crossfade,
                            c.audio.fade_in_delay,
                            c.audio.fade_in_duration,
                            c.audio.fade_out_delay,
                            c.audio.fade_out_duration,
                            c.audio.fade_out_mixmode,
                        )
                    })
                    .unwrap_or_default(),
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
                loaded_at: Instant::now(),
                expect_playback: false,
                open_attempts: 0,
                open_attempts_uri: None,
                prefetched: None,
                prefetch_landed: Arc::new(AtomicBool::new(false)),
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
            if matches!(event, PlayerEvent::EndOfTrack { .. } | PlayerEvent::Stopped) {
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
        self.command(PlayerCommand::LoadTracklist {
            tracks,
            start_index: None,
        });
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
    source_playback: Option<SourcePlayback>,
    source_phase: Option<Phase>,
    music_crossfade: CrossfadeSettings,
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
    /// The uri the prefetch was last started for, so a tick every 100ms does
    /// not start the same download ten times a second.
    prefetched: Option<String>,
    /// Set by a finished prefetch. The engine's lookahead is queued when the
    /// *current* track starts — long before the next one has been downloaded —
    /// so without re-syncing it the engine still opens the network stream for
    /// a track that is by then sitting on disk, and the cut stays audible.
    prefetch_landed: Arc<AtomicBool>,
    /// Consecutive status ticks spent in `Stopped` mid-track; a backstop so a
    /// decode failure still ends the track instead of wedging the queue.
    stopped_ticks: u32,
    /// When the current track was handed to the engine, and whether that was
    /// meant to start playback. Together they bound the wait for a track that
    /// never opens: the engine reports `Stopped` while it probes a remote url,
    /// so only a long silence means the open failed.
    loaded_at: Instant,
    expect_playback: bool,
    /// Failed opens counted for [`Self::open_attempts_uri`], so a track that
    /// cannot be opened at all is retried a few times and then let go rather
    /// than retried forever.
    open_attempts: u32,
    open_attempts_uri: Option<String>,
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
    fn has_managed_source(&self) -> bool {
        self.source_playback
            .as_ref()
            .is_some_and(SourcePlayback::is_loaded)
    }

    fn load_source(&mut self, source: SourceHandle, target: Duration, playing: bool) {
        let entry = self
            .tracklist
            .lock()
            .unwrap()
            .current_entry()
            .filter(|entry| entry.track.uri == source.to_handle());
        let Some(playback) = self.source_playback.as_mut() else {
            error!("saved-account playback is unavailable in this player");
            return;
        };
        let result = match entry {
            Some(entry) => playback.load_occurrence(
                source,
                entry.occurrence_id.clone(),
                entry.effective_selection(),
                target,
                playing,
            ),
            None => playback.load(source, target, playing),
        };
        if let Err(error) = result {
            error!(%error, "source playback rejected");
            return;
        }
        self.engine.stop();
        self.engine.clear_queue();
        self.engine.set_crossfade(CrossfadeSettings::default());
        self.track_loaded = true;
        self.engine_started = false;
        self.engine_index = 0;
        self.source_phase = None;
        self.icy_station = None;
        self.icy_last = None;
        publish_icy(None);
        self.poll_source_playback();
        self.save_queue();
    }

    fn poll_source_playback(&mut self) {
        let Some(playback) = self.source_playback.as_mut() else {
            return;
        };
        playback.poll(&self.engine);
        if let Some((occurrence_id, pin)) = playback.accepted_pin() {
            let mut queue = self.tracklist.lock().unwrap();
            if queue.current_entry().is_some_and(|entry| {
                entry.occurrence_id == occurrence_id && entry.pin.as_ref() != Some(pin)
            }) {
                if let Err(error) = queue.accept_current_pin(occurrence_id, pin.clone()) {
                    error!(error, "could not retain the accepted audio choice");
                }
            }
        }
        let snapshot = playback.snapshot();
        let error = playback.error().map(str::to_owned);
        let position = snapshot
            .position
            .filter(|position| Some(position.generation) == snapshot.generation)
            .map(|position| position.absolute)
            .unwrap_or(snapshot.target);
        // The legacy event surface is u32. The managed checkpoint keeps u64;
        // larger positions must not wrap in legacy clients.
        self.position_ms = u32::try_from(position.as_millis()).unwrap_or(u32::MAX);
        let is_playing =
            snapshot.desired == DesiredState::Playing && snapshot.phase == Phase::Streaming;
        self.tracklist
            .lock()
            .unwrap()
            .set_playback_state(PlaybackState {
                position_ms: self.position_ms,
                is_playing,
            });
        if self.source_phase != Some(snapshot.phase) {
            self.source_phase = Some(snapshot.phase);
            if snapshot.phase == Phase::Failed {
                error!(
                    reason = error.as_deref().unwrap_or("audio stream failed"),
                    "source playback failed"
                );
            }
            if is_playing {
                self.send_event(PlayerEvent::Playing);
            } else if snapshot.phase == Phase::Paused {
                self.send_event(PlayerEvent::Paused);
            }
            let (track, position) = self.tracklist.lock().unwrap().current_track();
            let event = PlayerEvent::CurrentTrack {
                track,
                position,
                position_ms: self.position_ms,
                is_playing,
            };
            (self.event_broadcaster)(event.clone());
            self.send_event(event);
        }
        if self.position_ms.abs_diff(self.last_broadcast_position_ms) >= POSITION_BROADCAST_STEP_MS
        {
            self.last_broadcast_position_ms = self.position_ms;
            (self.event_broadcaster)(PlayerEvent::TrackTimePosition {
                position_ms: self.position_ms,
            });
        }
        if self.last_queue_save.elapsed() >= QUEUE_SAVE_INTERVAL {
            self.save_queue();
        }
        // EndUnconfirmed and Failed retain this queue entry. Neither triggers
        // the legacy near-end/retry/automatic-next heuristics.
    }

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
    /// Download the next track once the current one is half played.
    ///
    /// Half, rather than near the end: a track takes time to fetch, and
    /// starting at 90% leaves no margin on a slow connection — which is the
    /// short cut at every track change this exists to remove. Half is late
    /// enough that a skipped track has usually been skipped by then, so the
    /// bandwidth is rarely wasted.
    ///
    /// Detached, and at most one at a time: the cache refuses a second
    /// download of the same uri, and this returns without waiting either way.
    /// Nothing here may block the tick that drives playback.
    fn maybe_prefetch_next(&mut self, status: &rockbox_playback::Status) {
        let duration = status.duration.as_millis() as u32;
        let position = status.position.as_millis() as u32;
        if duration == 0 || position * 2 < duration {
            return;
        }

        let Some(next) = self.tracklist.lock().unwrap().tracks().1.first().cloned() else {
            return;
        };
        if self.prefetched.as_deref() == Some(next.uri.as_str()) {
            return;
        }
        if !music_player_storage::track_cache::enabled()
            || !music_player_storage::track_cache::is_cacheable(&next.uri)
        {
            return;
        }

        self.prefetched = Some(next.uri.clone());
        let landed = Arc::clone(&self.prefetch_landed);
        tokio::spawn(async move {
            match music_player_storage::track_cache::store(&next.uri).await {
                Ok(_) => {
                    tracing::debug!(track = %next.title, "prefetched the next track");
                    landed.store(true, Ordering::Relaxed);
                }
                // "already downloading" lands here too, which is why this is
                // debug rather than a warning.
                Err(e) => tracing::debug!("could not prefetch: {e}"),
            }
        });
    }

    /// Mirror the playback modes onto the tracklist, which is what the API
    /// layers read. Without this a client could set them but never see them,
    /// so every launch showed "off" whatever the session had been.
    fn publish_modes(&self) {
        self.tracklist
            .lock()
            .unwrap()
            .set_modes(self.shuffle, self.repeat_mode);
    }

    fn save_queue(&mut self) {
        if !self.resume {
            return;
        }
        let (played, tracks, current) = {
            let queue = self.tracklist.lock().unwrap();
            let (played, tracks) = queue.entries();
            (played, tracks, queue.current_entry())
        };
        let path = queue_file();
        if played.is_empty() && tracks.is_empty() {
            let _ = std::fs::remove_file(&path);
            return;
        }
        // A removed current item may continue sounding, but its position
        // must not become the checkpoint of the last retained history item.
        let position_ms = if current.as_ref().is_some_and(|current| {
            played
                .last()
                .is_none_or(|last| last.occurrence_id != current.occurrence_id)
        }) {
            0
        } else {
            self.position_ms
        };
        let saved = SavedQueue {
            version: 2,
            source_checkpoint: self
                .source_playback
                .as_ref()
                .and_then(SourcePlayback::checkpoint)
                .filter(|checkpoint| {
                    played
                        .last()
                        .is_some_and(|entry| entry.occurrence_id == checkpoint.occurrence_id)
                }),
            played,
            tracks,
            position_ms,
            shuffle: self.shuffle,
            repeat_mode: self.repeat_mode,
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
        let Ok(mut saved) = serde_json::from_str::<SavedQueue>(&raw) else {
            return;
        };
        if let Err(error) = saved.prepare() {
            error!(
                error,
                "saved queue has invalid source references or audio choices"
            );
            return;
        }
        if saved.played.is_empty() && saved.tracks.is_empty() {
            return;
        }
        let managed_source = saved
            .played
            .last()
            .filter(|entry| SourceHandle::is_handle(&entry.track.uri))
            .and_then(|entry| SourceHandle::parse(&entry.track.uri).ok());
        if let Some(source) = managed_source {
            let Some(playback) = self.source_playback.as_mut() else {
                error!("saved queue needs an account resolver");
                return;
            };
            let restored = match saved.source_checkpoint.take() {
                Some(checkpoint) => playback.restore(source, checkpoint),
                None if saved.position_ms == 0 => {
                    let entry = saved
                        .played
                        .last()
                        .expect("managed source came from the current entry");
                    playback.load_occurrence(
                        source,
                        entry.occurrence_id.clone(),
                        entry.effective_selection(),
                        Duration::ZERO,
                        false,
                    )
                }
                None => Err(music_player_provider::ProviderError::Other(
                    "saved audio position has no pinned selection".into(),
                )),
            };
            if let Err(error) = restored {
                error!(%error, "could not restore audio checkpoint");
                return;
            }
            self.engine.stop();
            self.engine.clear_queue();
            self.engine.set_crossfade(CrossfadeSettings::default());
            self.shuffle = saved.shuffle;
            self.repeat_mode = saved.repeat_mode.clamp(0, 2);
            self.publish_modes();
            self.tracklist
                .lock()
                .unwrap()
                .restore_entries(saved.played, saved.tracks, saved.position_ms)
                .expect("saved queue was validated before playback restore");
            self.track_loaded = true;
            self.engine_started = false;
            self.source_phase = None;
            self.poll_source_playback();
            return;
        }
        // Restored before the tracklist, so the modes are already in force
        // when the queue lands rather than being applied on the next command.
        if let Some(source) = self.source_playback.as_mut() {
            source.clear();
        }
        self.source_phase = None;
        self.engine.set_crossfade(self.music_crossfade);
        self.shuffle = saved.shuffle;
        self.repeat_mode = saved.repeat_mode.clamp(0, 2);
        self.publish_modes();

        let current_uri = saved.played.last().map(|entry| entry.track.uri.clone());
        self.tracklist
            .lock()
            .unwrap()
            .restore_entries(saved.played, saved.tracks, saved.position_ms)
            .expect("saved queue was validated before playback restore");
        let Some(uri) = current_uri else { return };
        let uri = music_player_storage::track_cache::resolve(&uri);
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
        // Cued, not played: a restored queue waits for the user, so a track
        // that fails to open here must not be retried into playing itself.
        self.loaded_at = Instant::now();
        self.expect_playback = false;
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
        if self.has_managed_source() {
            self.poll_source_playback();
            return;
        }
        let status = self.engine.status();
        // Published every tick so a meter has something current to read; the
        // engine measures the PCM it actually hands to the output.
        self.maybe_prefetch_next(&status);
        // A prefetch finished: re-point the lookahead at the file it just
        // wrote, replacing the network url queued when this track started.
        if self.prefetch_landed.swap(false, Ordering::Relaxed) {
            self.resync_engine_next();
        }
        self.tracklist
            .lock()
            .unwrap()
            .set_levels(music_player_tracklist::Levels {
                left: status.levels.left,
                right: status.levels.right,
                low_left: status.levels.low_left,
                low_right: status.levels.low_right,
                bands: status.levels.bands.to_vec(),
            });
        match status.state {
            EngineState::Playing | EngineState::Paused => {
                self.engine_started = true;
                self.stopped_ticks = 0;
                // The engine advanced into its lookahead track on its own
                // (crossfade / gapless transition) — catch the tracklist up
                // and queue the next lookahead.
                if let Some(index) = status.index {
                    // ...unless the track it left never played a single
                    // sample. Then this is not a transition at all: the open
                    // failed and the engine fell forward into the lookahead,
                    // which is what a track "skipping immediately" is. Give
                    // the track another go before letting it past.
                    if index > self.engine_index && self.never_played() && self.retry_current() {
                        return;
                    }
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
                // Audio is coming out: whatever went wrong before did not
                // stop this track, so the next failure starts from zero.
                if position_ms > 0 {
                    self.open_attempts = 0;
                    self.open_attempts_uri = None;
                }
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
                // Loaded, meant to play, and still silent long after the
                // engine had time to open it: the open failed and there was
                // no lookahead to fall into, so nothing else will ever move
                // this queue on. (The engine also reports `Stopped` while it
                // probes a remote url, which is why this waits.)
                if !self.engine_started
                    && self.track_loaded
                    && self.expect_playback
                    && self.loaded_at.elapsed() >= OPEN_TIMEOUT
                {
                    if !self.retry_current() {
                        self.track_loaded = false;
                        let is_last_track = self.tracklist.lock().unwrap().is_empty();
                        self.send_event(PlayerEvent::EndOfTrack {
                            is_last_track: is_last_track && self.repeat_mode == 0,
                        });
                        self.handle_next();
                    }
                    return;
                }
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

    /// Whether the track the engine holds has produced any audio at all.
    ///
    /// Both counters are reset by every load and only written while the engine
    /// is playing, so a track that never opened leaves them at zero.
    fn never_played(&self) -> bool {
        self.position_ms == 0 && self.last_duration_ms == 0
    }

    /// The engine could not open the current track. Load it again, or report
    /// that it is out of retries (`false`) so the caller can move on.
    ///
    /// Worth retrying because the usual cause is transient and remote: a
    /// Subsonic server that answers one stream request out of many with
    /// `Wrong username or password`, a connection dropped mid-probe, a server
    /// that was still waking up. An unhandled failure here is invisible —
    /// the track flashes past and the next one starts — so this is also where
    /// the reason gets logged.
    fn retry_current(&mut self) -> bool {
        let Some(track) = self.tracklist.lock().unwrap().current_track().0 else {
            return false;
        };
        if self.open_attempts_uri.as_deref() != Some(track.uri.as_str()) {
            self.open_attempts_uri = Some(track.uri.clone());
            self.open_attempts = 0;
        }
        if self.open_attempts >= MAX_OPEN_ATTEMPTS {
            error!(
                "could not play {} after {} attempts, skipping it: {}",
                track.title, MAX_OPEN_ATTEMPTS, track.uri
            );
            return false;
        }
        self.open_attempts += 1;
        // A cached copy that cannot be opened cannot be opened on the retry
        // either — it is a fixed set of bytes. Dropping it sends the retry to
        // the network, which is also the only way a poisoned entry (an error
        // page saved as audio) ever leaves the cache.
        if music_player_storage::track_cache::invalidate(&track.uri) {
            tracing::warn!("discarded the cached copy of {}", track.title);
        }
        tracing::warn!(
            "could not open {}, retrying ({}/{})",
            track.title,
            self.open_attempts,
            MAX_OPEN_ATTEMPTS
        );
        self.handle_command_load(&track.uri);
        true
    }

    /// Latch the pristine station entry when the track that just started is
    /// internet radio, so [`Self::refresh_icy`] has a base to fold onto.
    /// Clears the overlay for anything else — a local file carries its own
    /// tags and must never be rewritten from the engine.
    fn arm_icy(&mut self) {
        let (track, _) = self.tracklist.lock().unwrap().current_track();
        self.icy_last = None;
        self.icy_station = track.filter(|t| t.id.starts_with(RADIO_ID_PREFIX));
        // Nothing has been announced on this station yet — and on anything
        // that is not radio there is nothing to announce at all.
        publish_icy(self.icy_station.as_ref().map(|station| IcyNowPlaying {
            station: station.title.clone(),
            ..Default::default()
        }));
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
        // Published before the fold, so consumers see which halves the station
        // actually sent rather than the ones filled in below.
        publish_icy(Some(IcyNowPlaying {
            artist: snapshot.artist.clone(),
            title: snapshot.title.clone(),
            station: station_name.clone(),
        }));
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
            PlayerCommand::LoadTracklist {
                tracks,
                start_index,
            } => self.handle_command_load_tracklist(tracks, start_index),
            PlayerCommand::LoadSelectedTracks {
                tracks,
                start_index,
                reply,
            } => {
                let result = self
                    .handle_load_selected_tracks(tracks, start_index)
                    .map_err(str::to_owned);
                let _ = reply.send(result);
            }
            PlayerCommand::SelectAudio {
                occurrence_id,
                selection,
                reply,
            } => {
                let result = self
                    .handle_select_audio(&occurrence_id, selection)
                    .map_err(str::to_owned);
                let _ = reply.send(result);
            }
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
            PlayerCommand::PlaySelectedMedia {
                track,
                selection,
                next,
                reply,
            } => {
                let result = self
                    .handle_play_selected_media(track, selection, next)
                    .map_err(str::to_owned);
                let _ = reply.send(result);
            }
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
            } => {
                self.music_crossfade = crossfade_settings(
                    mode,
                    fade_in_delay,
                    fade_in_duration,
                    fade_out_delay,
                    fade_out_duration,
                    mix_mode,
                );
                if !self.has_managed_source() {
                    self.engine.set_crossfade(self.music_crossfade);
                }
            }
            PlayerCommand::SetDither(enabled) => self.engine.set_dither(enabled),
            // The queue lives in the tracklist (the engine only holds the
            // current track + one lookahead), so shuffle/repeat act here.
            PlayerCommand::SetShuffle(enabled) => {
                self.shuffle = enabled;
                self.publish_modes();
                if enabled {
                    self.tracklist.lock().unwrap().shuffle();
                }
                self.resync_engine_next();
            }
            PlayerCommand::SetRepeat(mode) => {
                self.repeat_mode = mode.clamp(0, 2);
                self.publish_modes();
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
        if self.repeat_mode == 2 || self.has_managed_source() {
            return;
        }
        if let Some(next) = self.tracklist.lock().unwrap().peek_next() {
            if SourceHandle::is_handle(&next.id) || SourceHandle::is_handle(&next.uri) {
                return;
            }
            // Through the cache, like every other uri handed to the engine.
            // Missing it here is what left the cut audible: the prefetch had
            // the file, and the engine still opened the network stream for
            // the lookahead — which is the transition the cache exists for.
            self.engine
                .insert_last(music_player_storage::track_cache::resolve(&next.uri));
        }
    }

    /// Drop the engine's lookahead (if any) and re-queue the CURRENT
    /// upcoming track. Call after anything that changes what comes next:
    /// play-next inserts, queue removals, appends, shuffle, repeat changes.
    fn resync_engine_next(&mut self) {
        if !self.track_loaded || self.has_managed_source() {
            return;
        }
        self.engine.remove(self.engine_index + 1);
        self.queue_next_into_engine();
    }

    fn handle_command_load(&mut self, uri: &str) {
        if SourceHandle::is_handle(uri) {
            match SourceHandle::parse(uri) {
                Ok(source) if source.kind() == ResourceKind::Item => {
                    self.load_source(source, Duration::ZERO, true);
                }
                _ => error!("invalid playable source handle"),
            }
            return;
        }
        if let Some(source) = self.source_playback.as_mut() {
            source.clear();
        }
        self.engine.set_crossfade(self.music_crossfade);
        self.source_phase = None;
        // The cached copy when there is one. Every play goes through this, so
        // a track the prefetch already fetched starts from disk without the
        // caller knowing whether it did.
        let uri = music_player_storage::track_cache::resolve(uri);
        let uri = uri.as_str();
        self.engine.stop();
        self.engine.set_queue(vec![uri.to_string()]);
        self.engine.play();
        self.track_loaded = true;
        self.engine_started = false;
        self.engine_index = 0;
        self.stopped_ticks = 0;
        self.loaded_at = Instant::now();
        self.expect_playback = true;
        self.last_duration_ms = 0;
        self.position_ms = 0;
        self.last_broadcast_position_ms = 0;
        self.queue_next_into_engine();
        self.arm_icy();

        self.send_event(PlayerEvent::Playing {});
        let (track, position) = self.tracklist.lock().unwrap().current_track();
        // The track being played is the one worth knowing the key of.
        if let Some(track) = track.as_ref() {
            analyse_in_background(track);
        }
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

    fn handle_command_load_tracklist(
        &mut self,
        mut tracks: Vec<Track>,
        start_index: Option<usize>,
    ) {
        if let Err(error) = normalize_queue(&mut tracks) {
            error!(%error, "queue input rejected");
            return;
        }
        // Appending one track — "add to queue" — is worth caching for the same
        // reason as play-next. A whole tracklist is not: that is a library's
        // worth of downloads for tracks that may never be reached, and the
        // half-way prefetch will get them as they approach.
        if let [single] = tracks.as_slice() {
            cache_in_background(&single.uri);
        }
        self.tracklist.lock().unwrap().queue(tracks);
        if self.shuffle {
            self.tracklist.lock().unwrap().shuffle();
        }
        let (current_track, _) = self.tracklist.lock().unwrap().current_track();
        if current_track.is_some() {
            // Appended while playing — the lookahead may have been empty.
            self.resync_engine_next();
            return;
        }
        match start_index.filter(|index| *index > 0) {
            // Straight to the wanted track: one stream opened, not two.
            Some(index) => self.handle_play_track_at(index),
            None => self.handle_next(),
        }
    }

    fn handle_load_selected_tracks(
        &mut self,
        mut tracks: Vec<(Track, AudioSelection)>,
        start_index: Option<usize>,
    ) -> Result<Vec<String>, &'static str> {
        if tracks.is_empty() {
            return if start_index.is_some() {
                Err("an empty batch has no playback start index")
            } else {
                Ok(Vec::new())
            };
        }
        if let Some(index) = start_index {
            let queue = self.tracklist.lock().unwrap();
            let (_, played_count) = queue.current_track();
            let total = played_count
                .checked_add(queue.len())
                .and_then(|len| len.checked_add(tracks.len()))
                .ok_or("queue size exceeds supported bounds")?;
            if index >= total {
                return Err("playback start index is outside the resulting queue");
            }
        }
        for (track, _) in &mut tracks {
            normalize_queue(std::slice::from_mut(track)).map_err(|_| "invalid queue source")?;
        }
        let ids = self
            .tracklist
            .lock()
            .unwrap()
            .queue_with_selection(tracks)?;
        if self.shuffle {
            self.tracklist.lock().unwrap().shuffle();
        }
        if self.tracklist.lock().unwrap().current_entry().is_some() {
            self.resync_engine_next();
        } else {
            match start_index.filter(|index| *index > 0) {
                Some(index) => self.handle_play_track_at(index),
                None => self.handle_next(),
            }
        }
        self.save_queue();
        Ok(ids)
    }

    fn handle_select_audio(
        &mut self,
        occurrence_id: &str,
        selection: AudioSelection,
    ) -> Result<(), &'static str> {
        // Check before mutation: changing a current selection needs this player
        // to be able to cancel and replace its managed playback.
        if self.source_playback.is_none() {
            return Err("saved-account playback is unavailable in this player");
        }
        let is_current = self
            .tracklist
            .lock()
            .unwrap()
            .select_audio(occurrence_id, selection)?;
        if is_current {
            let entry = self
                .tracklist
                .lock()
                .unwrap()
                .current_entry()
                .ok_or("current queue occurrence is missing")?;
            let source =
                SourceHandle::parse(&entry.track.uri).map_err(|_| "invalid current source")?;
            let playing = self
                .source_playback
                .as_ref()
                .is_some_and(|playback| playback.snapshot().desired == DesiredState::Playing);
            // A deliberate new choice starts at zero. Retry/resume still keeps
            // the existing pin and checkpoint through the normal commands.
            self.load_source(source, Duration::ZERO, playing);
        }
        self.save_queue();
        Ok(())
    }

    fn handle_play(&mut self) {
        if self.has_managed_source() {
            if let Err(error) = self.source_playback.as_mut().unwrap().play() {
                error!(%error, "source play rejected");
                return;
            }
            self.track_loaded = true;
            self.poll_source_playback();
            return;
        }
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
        if self.has_managed_source() {
            self.source_playback.as_mut().unwrap().pause();
            self.engine.stop();
            self.poll_source_playback();
            self.save_queue();
            return;
        }
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
        if self.has_managed_source() {
            self.source_playback.as_mut().unwrap().stop();
            self.engine.stop();
            self.engine.clear_queue();
            self.poll_source_playback();
            // Keep the loaded occurrence and checkpoint so Play can resume it.
            self.track_loaded = false;
            self.engine_started = false;
            self.send_event(PlayerEvent::Stopped);
            self.save_queue();
            return;
        }
        if let Some(source) = self.source_playback.as_mut() {
            source.stop();
        }
        self.engine.stop();
        self.engine.clear_queue();
        self.track_loaded = false;
        self.engine_started = false;
        self.engine_index = 0;
        self.icy_station = None;
        self.icy_last = None;
        publish_icy(None);
        self.tracklist.lock().unwrap().stop();
        self.save_queue();
    }

    fn handle_command_seek(&mut self, position_ms: u32) {
        if self.has_managed_source() {
            if let Err(error) = self
                .source_playback
                .as_mut()
                .unwrap()
                .seek(Duration::from_millis(position_ms as u64))
            {
                error!(%error, "source seek rejected");
                return;
            }
            self.engine.stop();
            self.poll_source_playback();
            self.save_queue();
            return;
        }
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
        } else if self.has_managed_source() {
            self.handle_player_stop();
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
        if let Some(current_track) = current_track {
            self.handle_command_load(&current_track.uri);
        }
    }

    fn handle_clear(&mut self) {
        if self.has_managed_source() {
            self.handle_player_stop();
            self.source_playback.as_mut().unwrap().clear();
            self.source_phase = None;
            self.position_ms = 0;
            self.last_broadcast_position_ms = 0;
            let mut queue = self.tracklist.lock().unwrap();
            queue.stop();
            queue.set_playback_state(PlaybackState::default());
        }
        self.tracklist.lock().unwrap().clear();
        if self.resume {
            let _ = std::fs::remove_file(queue_file());
        }
    }

    fn handle_get_tracks(&mut self) {
        let tracks = self.tracklist.lock().unwrap().tracks();
        self.send_event(PlayerEvent::TracklistUpdated { tracks });
    }

    fn handle_play_selected_media(
        &mut self,
        mut track: Track,
        selection: AudioSelection,
        next: bool,
    ) -> Result<String, &'static str> {
        normalize_queue(std::slice::from_mut(&mut track)).map_err(|_| "invalid queue source")?;
        if SourceHandle::is_handle(&track.id) && self.source_playback.is_none() {
            return Err("saved-account playback is unavailable in this player");
        }
        let id = self
            .tracklist
            .lock()
            .unwrap()
            .insert_next_with_selection(track, selection)?;
        // One command owns insertion and playback: no client-side queue index
        // race, no intermediate stream, and shuffle cannot displace the choice.
        if next {
            self.resync_engine_next();
        } else {
            self.handle_next();
        }
        self.save_queue();
        Ok(id)
    }

    fn handle_play_next(&mut self, mut track: Track) {
        if let Err(error) = normalize_queue(std::slice::from_mut(&mut track)) {
            error!(%error, "queue input rejected");
            return;
        }
        // Inserting a track is a statement that it will be played soon, so it
        // is worth having on disk before then — the same reason the next track
        // is prefetched. Queued behind any download already running.
        cache_in_background(&track.uri);
        self.tracklist.lock().unwrap().insert_next(track);
        self.resync_engine_next();
    }

    fn handle_remove_track(&mut self, index: usize) {
        self.tracklist.lock().unwrap().remove_track_at(index);
        self.resync_engine_next();
    }

    fn handle_get_current_track(&mut self) {
        let (track, position) = self.tracklist.lock().unwrap().current_track();
        let is_playing = if self.has_managed_source() {
            self.tracklist.lock().unwrap().playback_state().is_playing
        } else {
            self.track_loaded && self.engine.status().state == EngineState::Playing
        };
        self.send_event(PlayerEvent::CurrentTrack {
            track,
            position,
            position_ms: self.position_ms,
            is_playing,
        });
    }
}

#[derive(Debug)]
// One variant carries a decoded track and dwarfs the rest. Boxing it would
// shrink the channel's element, but every construction and match site across
// the workspace would have to change for a message that is not sent in bulk —
// deliberately left as-is.
#[allow(clippy::large_enum_variant)]
pub enum PlayerCommand {
    PlaySelectedMedia {
        track: Track,
        selection: AudioSelection,
        next: bool,
        reply: tokio::sync::oneshot::Sender<Result<String, String>>,
    },
    LoadSelectedTracks {
        tracks: Vec<(Track, AudioSelection)>,
        start_index: Option<usize>,
        reply: tokio::sync::oneshot::Sender<Result<Vec<String>, String>>,
    },
    SelectAudio {
        occurrence_id: String,
        selection: AudioSelection,
        reply: tokio::sync::oneshot::Sender<Result<(), String>>,
    },
    Load {
        track_id: String,
    },
    LoadTracklist {
        tracks: Vec<Track>,
        /// Which track to start on.
        ///
        /// `None` starts at the beginning. It exists so a caller that wants a
        /// particular track does not have to follow this with `PlayTrackAt`:
        /// that opened the first track's stream and immediately replaced it,
        /// which against a remote server is a wasted connection and a real
        /// delay before anything is heard.
        start_index: Option<usize>,
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

/// Validate an entire incoming batch before mutating the queue or stopping its
/// current source. API entry points use the same rule before sending commands.
pub fn normalize_queue(
    tracks: &mut [Track],
) -> Result<(), music_player_types::source::SourceError> {
    for track in tracks {
        if let Some(source) = normalize_source_track(&mut track.id, &mut track.uri)? {
            if source.kind() != ResourceKind::Item {
                return Err(music_player_types::source::SourceError::InvalidKind);
            }
        }
    }
    Ok(())
}

/// Persisted queue snapshot: the exact played/upcoming split plus the
/// position within the current track, so a restart comes back cued paused
/// where it left off.
#[derive(Serialize, Deserialize, Default)]
struct SavedQueue {
    #[serde(default)]
    version: u32,
    #[serde(default)]
    source_checkpoint: Option<SourceCheckpoint>,
    played: Vec<QueueEntry>,
    tracks: Vec<QueueEntry>,
    position_ms: u32,
    /// Shuffle and repeat, so a restart resumes the session as it was rather
    /// than silently reverting to off. Defaulted so a queue written by an
    /// older build still loads.
    #[serde(default)]
    shuffle: bool,
    #[serde(default)]
    repeat_mode: i32,
}

impl SavedQueue {
    fn prepare(&mut self) -> Result<(), &'static str> {
        if self.version != 0 && self.version != 2 {
            return Err("unsupported queue snapshot version");
        }
        for entry in self.played.iter_mut().chain(self.tracks.iter_mut()) {
            normalize_queue(std::slice::from_mut(&mut entry.track))
                .map_err(|_| "invalid saved queue source")?;
        }
        if let Some(checkpoint) = &self.source_checkpoint {
            let current = self
                .played
                .last_mut()
                .ok_or("audio checkpoint has no queue occurrence")?;
            if checkpoint.source != current.track.uri {
                return Err("audio checkpoint belongs to a different source");
            }
            let source =
                SourceHandle::parse(&current.track.uri).map_err(|_| "invalid checkpoint source")?;
            checkpoint.capabilities_for(&source)?;
            if self.version == 0 {
                // Old snapshots stored a choice only for the current source.
                current.occurrence_id = checkpoint.occurrence_id.clone();
                match &checkpoint.selection {
                    AudioSelection::Pinned(pin) => {
                        current.selection = AudioSelection::Auto;
                        current.pin = Some(pin.clone());
                    }
                    requested => {
                        current.selection = requested.clone();
                        current.pin = None;
                    }
                }
            }
            if checkpoint.occurrence_id != current.occurrence_id
                || checkpoint.selection != current.effective_selection()
            {
                return Err("audio checkpoint belongs to a different occurrence or choice");
            }
        }
        validate_entries(self.played.iter().chain(self.tracks.iter()))?;
        self.version = 2;
        Ok(())
    }
}

fn queue_file() -> PathBuf {
    PathBuf::from(get_application_directory())
        .join("cache")
        .join("queue.json")
}

#[cfg(test)]
mod queue_snapshot_tests {
    use super::*;
    use music_player_types::source::RemoteIdentity;

    // Use an isolated TCP output, without a sound device or user settings.
    // These tests exercise host command handling; they do not play audio.
    fn host(queue: Tracklist) -> (PlayerInternal, std::net::TcpListener) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let output = format!("tcp-connect:{}", listener.local_addr().unwrap())
            .parse::<OutputConfig>()
            .unwrap();
        let engine = PlayerConfig::builder().output(output).open().unwrap();
        let (_, commands) = mpsc::unbounded_channel();
        (
            PlayerInternal {
                source_playback: None,
                source_phase: None,
                music_crossfade: CrossfadeSettings::default(),
                commands: Arc::new(std::sync::Mutex::new(commands)),
                engine,
                event_senders: Vec::new(),
                tracklist: Arc::new(std::sync::Mutex::new(queue)),
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
                prefetched: None,
                prefetch_landed: Arc::new(AtomicBool::new(false)),
                stopped_ticks: 0,
                loaded_at: Instant::now(),
                expect_playback: false,
                open_attempts: 0,
                open_attempts_uri: None,
                icy_station: None,
                icy_last: None,
                event_broadcaster: Box::new(|_| {}),
            },
            listener,
        )
    }

    #[test]
    fn selected_empty_and_out_of_bounds_batches_leave_the_host_queue_unchanged() {
        let mut queue = Tracklist::new(vec![entry().track, entry().track, entry().track]);
        queue.next_track();
        let expected = queue.entries();
        let current = queue.current_entry();
        let (mut player, _listener) = host(queue);
        player.shuffle = true;
        assert!(player
            .handle_load_selected_tracks(Vec::new(), None)
            .unwrap()
            .is_empty());
        assert!(player
            .handle_load_selected_tracks(Vec::new(), Some(0))
            .is_err());
        // Three existing occurrences plus one new occurrence: index 4 is invalid.
        assert!(player
            .handle_load_selected_tracks(vec![(entry().track, AudioSelection::Auto)], Some(4))
            .is_err());
        assert!(player
            .handle_load_selected_tracks(
                vec![(entry().track, AudioSelection::Auto)],
                Some(usize::MAX)
            )
            .is_err());
        assert_eq!(player.tracklist.lock().unwrap().entries(), expected);
        assert_eq!(player.tracklist.lock().unwrap().current_entry(), current);
        assert!(!player.track_loaded);
    }

    #[tokio::test]
    async fn managed_clear_removes_current_occurrence_and_resets_position() {
        let mut queue = Tracklist::new(vec![entry().track, entry().track]);
        queue.next_track();
        let current = queue.current_entry().unwrap();
        queue.set_playback_state(PlaybackState {
            position_ms: 7200123,
            is_playing: false,
        });
        let (mut player, _listener) = host(queue);
        let db = music_player_storage::Database {
            connection: sea_orm::Database::connect("sqlite::memory:").await.unwrap(),
        };
        let mut source = SourcePlayback::new(SourceResolver::new(db, "ci-host".into(), true));
        // A paused occurrence has no resolver task or network session.
        source
            .load_occurrence(
                SourceHandle::parse(&current.track.uri).unwrap(),
                current.occurrence_id,
                AudioSelection::Auto,
                Duration::from_millis(7200123),
                false,
            )
            .unwrap();
        player.source_playback = Some(source);
        player.position_ms = 7200123;
        player.last_broadcast_position_ms = 7200123;
        player.track_loaded = true;
        player.handle_clear();
        assert!(!player.has_managed_source());
        assert_eq!(player.position_ms, 0);
        assert_eq!(player.last_broadcast_position_ms, 0);
        assert!(!player.track_loaded);
        let queue = player.tracklist.lock().unwrap();
        assert!(queue.current_entry().is_none());
        assert_eq!(queue.entries(), (Vec::new(), Vec::new()));
        assert_eq!(queue.playback_state(), PlaybackState::default());
    }

    #[tokio::test]
    async fn audio_command_edits_one_occurrence_and_keeps_current_selection_paused() {
        let mut queue = Tracklist::new(vec![entry().track, entry().track]);
        queue.next_track();
        let current = queue.current_entry().unwrap();
        let upcoming = queue.entries().1[0].clone();
        let (mut player, _listener) = host(queue);
        let db = music_player_storage::Database {
            connection: sea_orm::Database::connect("sqlite::memory:").await.unwrap(),
        };
        let mut source = SourcePlayback::new(SourceResolver::new(db, "ci-host".into(), true));
        source
            .load_occurrence(
                SourceHandle::parse(&current.track.uri).unwrap(),
                current.occurrence_id.clone(),
                AudioSelection::Auto,
                Duration::from_millis(7200123),
                false,
            )
            .unwrap();
        player.source_playback = Some(source);
        player.position_ms = 7200123;
        let choice = AudioSelection::Explicit {
            media_source_id: "version-b".into(),
            audio_stream_index: 0,
        };

        let (reply, received) = tokio::sync::oneshot::channel();
        player
            .handle_command(PlayerCommand::SelectAudio {
                occurrence_id: upcoming.occurrence_id.clone(),
                selection: choice.clone(),
                reply,
            })
            .unwrap();
        assert_eq!(received.await.unwrap(), Ok(()));
        let checkpoint = player
            .source_playback
            .as_ref()
            .unwrap()
            .checkpoint()
            .unwrap();
        assert_eq!(checkpoint.occurrence_id, current.occurrence_id);
        assert_eq!(checkpoint.offset_ms, 7200123);
        assert_eq!(checkpoint.selection, AudioSelection::Auto);
        assert_eq!(player.position_ms, 7200123);
        let expected_upcoming = player.tracklist.lock().unwrap().entries().1;
        assert_eq!(expected_upcoming[0].selection, choice);

        let (reply, received) = tokio::sync::oneshot::channel();
        player
            .handle_command(PlayerCommand::SelectAudio {
                occurrence_id: current.occurrence_id.clone(),
                selection: choice.clone(),
                reply,
            })
            .unwrap();
        assert_eq!(received.await.unwrap(), Ok(()));
        let playback = player.source_playback.as_ref().unwrap();
        let checkpoint = playback.checkpoint().unwrap();
        assert_eq!(checkpoint.occurrence_id, current.occurrence_id);
        assert_eq!(checkpoint.offset_ms, 0);
        assert_eq!(checkpoint.selection, choice);
        assert_eq!(playback.snapshot().desired, DesiredState::Paused);
        assert!(playback.accepted_pin().is_none());
        assert_eq!(player.position_ms, 0);
        let queue = player.tracklist.lock().unwrap();
        assert_eq!(queue.entries().1, expected_upcoming);
        assert_eq!(
            queue.current_entry().unwrap().occurrence_id,
            current.occurrence_id
        );
        assert_eq!(queue.current_entry().unwrap().selection, choice);
        assert!(!queue.playback_state().is_playing);
    }

    #[tokio::test]
    async fn full_metadata_play_is_atomic_and_next_preserves_paused_occurrence() {
        let mut queue = Tracklist::new(vec![entry().track, entry().track]);
        queue.next_track();
        let current = queue.current_entry().unwrap();
        let original_upcoming = queue.entries().1;
        let (mut player, _listener) = host(queue);
        let choice = AudioSelection::Explicit {
            media_source_id: "version-b".into(),
            audio_stream_index: 0,
        };
        // Capability rejection happens before changing the queue.
        assert!(player
            .handle_play_selected_media(entry().track, choice.clone(), false)
            .is_err());
        assert_eq!(
            player.tracklist.lock().unwrap().current_entry(),
            Some(current.clone())
        );
        assert_eq!(
            player.tracklist.lock().unwrap().entries().1,
            original_upcoming
        );
        let db = music_player_storage::Database {
            connection: sea_orm::Database::connect("sqlite::memory:").await.unwrap(),
        };
        let mut source = SourcePlayback::new(SourceResolver::new(db, "ci-host".into(), true));
        source
            .load_occurrence(
                SourceHandle::parse(&current.track.uri).unwrap(),
                current.occurrence_id.clone(),
                AudioSelection::Auto,
                Duration::from_millis(7200123),
                false,
            )
            .unwrap();
        player.source_playback = Some(source);
        player.shuffle = true;
        for next in [true, false] {
            let mut track = entry().track;
            track.title = "Full browse metadata".into();
            let (reply, received) = tokio::sync::oneshot::channel();
            player
                .handle_command(PlayerCommand::PlaySelectedMedia {
                    track,
                    selection: choice.clone(),
                    next,
                    reply,
                })
                .unwrap();
            let id = received.await.unwrap().unwrap();
            let checkpoint = player
                .source_playback
                .as_ref()
                .unwrap()
                .checkpoint()
                .unwrap();
            let snapshot = player.source_playback.as_ref().unwrap().snapshot();
            let queue = player.tracklist.lock().unwrap();
            if next {
                assert_eq!(checkpoint.occurrence_id, current.occurrence_id);
                assert_eq!(checkpoint.offset_ms, 7200123);
                assert_eq!(snapshot.desired, DesiredState::Paused);
                assert_eq!(queue.entries().1[0].occurrence_id, id);
                assert_eq!(&queue.entries().1[1..], original_upcoming.as_slice());
            } else {
                let playing = queue.current_entry().unwrap();
                assert_eq!(playing.occurrence_id, id);
                assert_eq!(playing.track.title, "Full browse metadata");
                assert_eq!(playing.selection, choice);
                assert_eq!(checkpoint.occurrence_id, id);
                assert_eq!(checkpoint.offset_ms, 0);
                assert_eq!(snapshot.desired, DesiredState::Playing);
                // The earlier Next request remains queued, with its own ID.
                assert_ne!(queue.entries().1[0].occurrence_id, id);
            }
        }
        player.source_playback.as_mut().unwrap().clear();
    }

    fn entry() -> QueueEntry {
        let handle = music_player_types::source::SourceRef {
            resolver: "emby".into(),
            account_id: "saved-family".into(),
            remote: RemoteIdentity {
                server_id: "server-a".into(),
                user_id: "family".into(),
            },
            kind: ResourceKind::Item,
            item_id: "55508".into(),
        }
        .to_handle();
        QueueEntry::new(Track {
            id: handle.clone(),
            uri: handle,
            ..Default::default()
        })
    }

    #[test]
    fn legacy_track_snapshot_assigns_distinct_occurrences() {
        let track = entry().track;
        let raw = serde_json::json!({ "played": [track.clone()], "tracks": [track], "position_ms": 1234 });
        let mut saved: SavedQueue = serde_json::from_value(raw).unwrap();
        saved.prepare().unwrap();
        assert_eq!(saved.version, 2);
        assert_ne!(saved.played[0].occurrence_id, saved.tracks[0].occurrence_id);
        assert_eq!(saved.played[0].effective_selection(), AudioSelection::Auto);
        assert_eq!(saved.position_ms, 1234);
    }

    #[test]
    fn legacy_checkpoint_migrates_pin_and_keeps_long_position() {
        let current = entry();
        let pin = music_player_types::audio::AudioPin {
            media_source_id: "version-a".into(),
            audio_stream_index: 0,
            runtime_ticks: Some(191429666670),
            etag: Some("revision-a".into()),
            codec: Some("aac".into()),
            channels: Some(2),
            sample_rate: Some(44100),
        };
        let checkpoint = SourceCheckpoint {
            version: 1,
            capabilities: None,
            occurrence_id: current.occurrence_id.clone(),
            source: current.track.uri.clone(),
            offset_ms: 7200123,
            selection: AudioSelection::Pinned(pin.clone()),
        };
        let raw = serde_json::json!({ "played": [current.track.clone()], "tracks": [current.track],
            "position_ms": 7200123, "source_checkpoint": checkpoint });
        let mut saved: SavedQueue = serde_json::from_value(raw).unwrap();
        saved.prepare().unwrap();
        assert_eq!(saved.played[0].occurrence_id, current.occurrence_id);
        assert_eq!(saved.played[0].selection, AudioSelection::Auto);
        assert_eq!(saved.played[0].pin, Some(pin));
        assert_eq!(saved.tracks[0].pin, None);
        let bytes = serde_json::to_vec(&saved).unwrap();
        let mut restored: SavedQueue = serde_json::from_slice(&bytes).unwrap();
        restored.prepare().unwrap();
        assert_eq!(restored.played, saved.played);
        assert_eq!(restored.tracks, saved.tracks);
        assert_eq!(restored.source_checkpoint.unwrap().offset_ms, 7200123);
    }

    #[test]
    fn checkpoint_for_same_item_but_other_occurrence_or_choice_is_rejected() {
        let current = entry();
        let mut saved = SavedQueue {
            version: 2,
            played: vec![current.clone()],
            source_checkpoint: Some(SourceCheckpoint {
                version: 1,
                capabilities: None,
                occurrence_id: uuid::Uuid::new_v4().to_string(),
                source: current.track.uri.clone(),
                offset_ms: 1234,
                selection: AudioSelection::Auto,
            }),
            ..Default::default()
        };
        assert!(saved.prepare().is_err());
        saved.source_checkpoint.as_mut().unwrap().occurrence_id = current.occurrence_id;
        saved.source_checkpoint.as_mut().unwrap().selection = AudioSelection::Explicit {
            media_source_id: "version-b".into(),
            audio_stream_index: 3,
        };
        assert!(saved.prepare().is_err());
        saved.source_checkpoint.as_mut().unwrap().selection = AudioSelection::Auto;
        saved.prepare().unwrap();
    }
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
// One variant carries a decoded track and dwarfs the rest. Boxing it would
// shrink the channel's element, but every construction and match site across
// the workspace would have to change for a message that is not sent in bulk —
// deliberately left as-is.
#[allow(clippy::large_enum_variant)]
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
