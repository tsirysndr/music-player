use async_trait::async_trait;
use futures_util::{future::FusedFuture, Future};
use log::{error, trace};
use music_player_entity::track::Model as Track;
use music_player_tracklist::Tracklist;
use parking_lot::Mutex;
use std::{
    collections::HashMap,
    fs::File,
    mem,
    path::Path,
    pin::Pin,
    process::exit,
    sync::Arc,
    task::{Context, Poll},
    thread,
};
use symphonia::core::{errors::Error, io::MediaSourceStream, probe::Hint};
use tokio::{
    runtime::Runtime,
    sync::mpsc::{self, UnboundedReceiver},
};

use crate::{
    audio_backend::Sink,
    convert::Converter,
    decoder::{symphonia_decoder::SymphoniaDecoder, AudioDecoder},
    dither::{mk_ditherer, TriangularDitherer},
    formatter,
    metadata::audio::AudioFileFormat,
};

const PRELOAD_NEXT_TRACK_BEFORE_END: u64 = 30000;

pub type PlayerResult = Result<(), Error>;

#[async_trait]
pub trait PlayerEngine: Send + Sync {
    fn load(&mut self, track_id: &str, _start_playing: bool, _position_ms: u32);
    fn load_tracklist(&mut self, tracks: Vec<Track>);
    fn preload(&self, _track_id: &str);
    fn play(&self);
    fn pause(&self);
    fn stop(&self);
    fn seek(&self, position_ms: u32);
    fn next(&self);
    fn previous(&self);
    fn clear(&self);
    async fn get_tracks(&self) -> (Vec<Track>, Vec<Track>);
    async fn wait_for_tracklist(
        mut event: UnboundedReceiver<PlayerEvent>,
    ) -> (Vec<Track>, Vec<Track>);
    async fn get_current_track(&self) -> Option<Track>;
    async fn wait_for_current_track(mut channel: UnboundedReceiver<PlayerEvent>) -> Option<Track>;
}

#[derive(Clone)]
pub struct Player {
    commands: Option<mpsc::UnboundedSender<PlayerCommand>>,
}

impl Player {
    pub fn new<F>(sink_builder: F) -> (Player, PlayerEventChannel)
    where
        F: FnOnce() -> Box<dyn Sink> + Send + 'static,
    {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        thread::spawn(move || {
            let internal = PlayerInternal {
                commands: cmd_rx,
                load_handles: Arc::new(Mutex::new(HashMap::new())),
                sink: sink_builder(),
                state: PlayerState::Stopped,
                sink_status: SinkStatus::Closed,
                sink_event_callback: None,
                event_senders: [event_sender].to_vec(),
                tracklist: Tracklist::new_empty(),
            };
            let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            runtime.block_on(internal);
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
            if let Err(e) = commands.send(cmd) {
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

    fn preload(&self, _track_id: &str) {
        self.command(PlayerCommand::Preload);
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

    fn clear(&self) {
        self.command(PlayerCommand::Clear);
    }

    async fn get_tracks(&self) -> (Vec<Track>, Vec<Track>) {
        let channel = self.get_player_event_channel();
        let handle = thread::spawn(move || {
            Runtime::new()
                .unwrap()
                .block_on(Self::wait_for_tracklist(channel))
        });
        self.command(PlayerCommand::GetTracks);
        handle.join().unwrap()
    }

    async fn get_current_track(&self) -> Option<Track> {
        let channel = self.get_player_event_channel();
        let handle = thread::spawn(move || {
            Runtime::new()
                .unwrap()
                .block_on(Self::wait_for_current_track(channel))
        });
        self.command(PlayerCommand::GetCurrentTrack);
        handle.join().unwrap()
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

    async fn wait_for_current_track(mut channel: UnboundedReceiver<PlayerEvent>) -> Option<Track> {
        while let Some(event) = channel.recv().await {
            if matches!(event, PlayerEvent::CurrentTrack { .. }) {
                return event.get_current_track().unwrap();
            }
        }
        None
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum SinkStatus {
    Running,
    Closed,
    TemporarilyClosed,
}

pub type SinkEventCallback = Box<dyn Fn(SinkStatus) + Send>;

struct PlayerInternal {
    commands: mpsc::UnboundedReceiver<PlayerCommand>,
    load_handles: Arc<Mutex<HashMap<thread::ThreadId, thread::JoinHandle<()>>>>,

    state: PlayerState,
    sink: Box<dyn Sink>,
    sink_status: SinkStatus,
    sink_event_callback: Option<SinkEventCallback>,
    event_senders: Vec<mpsc::UnboundedSender<PlayerEvent>>,
    tracklist: Tracklist,
}

impl Future for PlayerInternal {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        loop {
            let mut all_futures_completed_or_not_ready = true;

            // process commands that were sent to us
            let cmd = match self.commands.poll_recv(cx) {
                Poll::Ready(None) => return Poll::Ready(()), // client has disconnected - shut down.
                Poll::Ready(Some(cmd)) => {
                    all_futures_completed_or_not_ready = false;
                    Some(cmd)
                }
                _ => None,
            };

            if let Some(cmd) = cmd {
                if let Err(e) = self.handle_command(cmd) {
                    // error!("Error handling command: {}", e);
                }
            }

            if let PlayerState::Playing { ref mut decoder } = self.state {
                match decoder.next_packet() {
                    Ok(result) => {
                        if let Some((ref _packet_position, packet, channels, sample_rate)) = result
                        {
                            match packet.samples() {
                                Ok(_) => {
                                    let mut converter =
                                        Converter::new(Some(mk_ditherer::<TriangularDitherer>));
                                    if let Err(e) = self.sink.write(
                                        packet,
                                        channels,
                                        sample_rate,
                                        &mut converter,
                                    ) {
                                        error!("Error writing to sink: {}", e);
                                        exit(1)
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to decode packet: {}", e);
                                }
                            }
                        } else {
                            // end of track
                            self.state = PlayerState::Stopped;
                            let tracklist = self.tracklist.clone();
                            self.send_event(PlayerEvent::EndOfTrack {
                                is_last_track: tracklist.is_empty(),
                            });
                            self.handle_next();
                        }
                    }
                    Err(e) => {
                        error!("Failed to decode packet: {}", e);
                    }
                };
            }
        }
    }
}

impl PlayerInternal {
    fn ensure_sink_running(&mut self) {
        if self.sink_status != SinkStatus::Running {
            trace!("== Starting sink ==");
            if let Some(callback) = &mut self.sink_event_callback {
                callback(SinkStatus::Running);
            }
            match self.sink.start() {
                Ok(()) => self.sink_status = SinkStatus::Running,
                Err(e) => {
                    error!("{}", e);
                    exit(1);
                }
            }
        }
    }

    fn ensure_sink_stopped(&mut self, temporarily: bool) {
        match self.sink_status {
            SinkStatus::Running => {
                trace!("== Stopping sink ==");
                match self.sink.stop() {
                    Ok(()) => {
                        self.sink_status = if temporarily {
                            SinkStatus::TemporarilyClosed
                        } else {
                            SinkStatus::Closed
                        };
                        if let Some(callback) = &mut self.sink_event_callback {
                            callback(self.sink_status);
                        }
                    }
                    Err(e) => {
                        error!("{}", e);
                        exit(1);
                    }
                }
            }
            SinkStatus::TemporarilyClosed => {
                if !temporarily {
                    self.sink_status = SinkStatus::Closed;
                    if let Some(callback) = &mut self.sink_event_callback {
                        callback(SinkStatus::Closed);
                    }
                }
            }
            SinkStatus::Closed => (),
        }
    }

    fn handle_command(&mut self, cmd: PlayerCommand) -> PlayerResult {
        match cmd {
            PlayerCommand::Load { track_id } => self.handle_command_load(&track_id),
            PlayerCommand::LoadTracklist { tracks } => self.handle_command_load_tracklist(tracks),
            PlayerCommand::Preload => self.handle_command_preload(),
            PlayerCommand::Play => self.handle_play(),
            PlayerCommand::Pause => self.handle_pause(),
            PlayerCommand::Stop => self.handle_player_stop(),
            PlayerCommand::Seek(position_ms) => self.handle_command_seek(),
            PlayerCommand::AddEventSender(sender) => self.event_senders.push(sender),
            PlayerCommand::Next => self.handle_next(),
            PlayerCommand::Previous => self.handle_previous(),
            PlayerCommand::Clear => self.handle_clear(),
            PlayerCommand::GetTracks => self.handle_get_tracks(),
            PlayerCommand::GetCurrentTrack => self.handle_get_current_track(),
        }
        Ok(())
    }

    fn load_track(&self, song: &str) -> Option<PlayerLoadedTrackData> {
        // Create a hint to help the format registry guess what format reader is appropriate.
        let mut hint = Hint::new();

        let path = Path::new(song);

        // Provide the file extension as a hint.
        if let Some(extension) = path.extension() {
            if let Some(extension_str) = extension.to_str() {
                hint.with_extension(extension_str);
            }
        }

        let source = Box::new(File::open(path).unwrap());

        // Create the media source stream using the boxed media source from above.
        let mss = MediaSourceStream::new(source, Default::default());

        let symphonia_decoder = |mss: MediaSourceStream, hint| {
            SymphoniaDecoder::new(mss, hint).map(|mut decoder| {
                // For formats other that Vorbis, we'll try getting normalisation data from
                // ReplayGain metadata fields, if present.
                Box::new(decoder) as Decoder
            })
        };

        let decoder_type = symphonia_decoder(mss, hint);

        let decoder = match decoder_type {
            Ok(decoder) => decoder,
            Err(e) => {
                panic!("Failed to create decoder: {}", e);
            }
        };
        return Some(PlayerLoadedTrackData {
            decoder,
            bytes_per_second: 0,
            duration_ms: 0,
            stream_position_ms: 0,
            is_explicit: false,
        });
    }

    fn start_playback(&mut self, track_id: &str, loaded_track: PlayerLoadedTrackData) {
        self.ensure_sink_running();
        self.send_event(PlayerEvent::Playing {});

        self.state = PlayerState::Playing {
            decoder: loaded_track.decoder,
        };
    }

    fn send_event(&mut self, event: PlayerEvent) {
        self.event_senders
            .retain(|sender| sender.send(event.clone()).is_ok());
    }

    fn handle_command_load(&mut self, track_id: &str) {
        formatter::print_format(track_id);
        let loaded_track = self.load_track(track_id);
        match loaded_track {
            Some(loaded_track) => {
                self.start_playback(track_id, loaded_track);
            }
            None => {
                self.send_event(PlayerEvent::Error {
                    track_id: track_id.to_string(),
                    error: "Failed to load track".to_string(),
                });
            }
        }
    }

    fn handle_command_load_tracklist(&mut self, tracks: Vec<Track>) {
        self.tracklist.queue(tracks);
        if self.tracklist.current_track().is_none() {
            self.handle_next();
        }
    }

    fn handle_command_preload(&self) {
        todo!()
    }

    fn handle_play(&mut self) {
        if let PlayerState::Paused { .. } = self.state {
            self.state.paused_to_playing();
            self.send_event(PlayerEvent::Playing);
            self.ensure_sink_running();
        } else {
            error!("Player::play called from invalid state");
        }
    }

    fn handle_player_stop(&mut self) {
        self.ensure_sink_stopped(false);
        self.state = PlayerState::Stopped;
    }

    fn handle_pause(&mut self) {
        if let PlayerState::Playing { .. } = self.state {
            self.state.playing_to_paused();
            self.send_event(PlayerEvent::Paused);
        } else {
            error!("Player::pause called from invalid state");
        }
    }

    fn handle_command_seek(&self) {
        todo!()
    }

    fn handle_next(&mut self) {
        if self.tracklist.next_track().is_some() {
            self.handle_command_load(&self.tracklist.current_track().unwrap().uri);
        }
    }

    fn handle_previous(&mut self) {
        if self.tracklist.previous_track().is_some() {
            self.handle_command_load(&self.tracklist.current_track().unwrap().uri);
        }
    }

    fn handle_clear(&mut self) {
        self.tracklist.clear();
    }

    fn handle_get_tracks(&mut self) {
        let tracks = self.tracklist.tracks();
        self.send_event(PlayerEvent::TracklistUpdated { tracks });
    }

    fn handle_get_current_track(&mut self) {
        let track = self.tracklist.current_track();
        self.send_event(PlayerEvent::CurrentTrack { track });
    }
}

struct PlayerLoadedTrackData {
    decoder: Decoder,
    bytes_per_second: usize,
    duration_ms: u32,
    stream_position_ms: u32,
    is_explicit: bool,
}

type Decoder = Box<dyn AudioDecoder + Send>;

enum PlayerState {
    Stopped,
    Loading {
        loader: Pin<Box<dyn FusedFuture<Output = Result<PlayerLoadedTrackData, ()>> + Send>>,
    },
    Paused {
        decoder: Decoder,
    },
    Playing {
        decoder: Decoder,
    },
    EndOfTrack {
        loaded_track: PlayerLoadedTrackData,
    },
    Invalid,
}

impl PlayerState {
    fn is_playing(&self) -> bool {
        use self::PlayerState::*;
        match *self {
            Stopped | EndOfTrack { .. } | Paused { .. } | Loading { .. } => false,
            Playing { .. } => true,
            Invalid => {
                // "PlayerState::is_playing in invalid state"
                exit(1);
            }
        }
    }

    #[allow(dead_code)]
    fn is_stopped(&self) -> bool {
        use self::PlayerState::*;
        matches!(self, Stopped)
    }

    #[allow(dead_code)]
    fn is_loading(&self) -> bool {
        use self::PlayerState::*;
        matches!(self, Loading { .. })
    }

    fn decoder(&mut self) -> Option<&mut Decoder> {
        use self::PlayerState::*;
        match *self {
            Stopped | EndOfTrack { .. } | Loading { .. } => None,
            Paused {
                ref mut decoder, ..
            }
            | Playing {
                ref mut decoder, ..
            } => Some(decoder),
            Invalid => {
                // error!("PlayerState::decoder in invalid state");
                exit(1);
            }
        }
    }

    fn playing_to_paused(&mut self) {
        use self::PlayerState::*;
        let new_state = mem::replace(self, Invalid);
        match new_state {
            Playing { decoder } => {
                *self = Paused { decoder };
            }
            _ => {
                error!("PlayerState::playing_to_paused in invalid state");
                exit(1);
            }
        }
    }

    fn paused_to_playing(&mut self) {
        use self::PlayerState::*;
        let new_state = mem::replace(self, Invalid);
        match new_state {
            Paused { decoder } => {
                *self = Playing { decoder };
            }
            _ => {
                error!("PlayerState::paused_to_playing in invalid state");
                exit(1);
            }
        }
    }
}

pub struct PlayerTrackLoader {}

impl PlayerTrackLoader {
    fn stream_data_rate(&self, format: AudioFileFormat) -> usize {
        let kbps = match format {
            AudioFileFormat::OGG_VORBIS_96 => 12,
            AudioFileFormat::OGG_VORBIS_160 => 20,
            AudioFileFormat::OGG_VORBIS_320 => 40,
            AudioFileFormat::MP3_256 => 32,
            AudioFileFormat::MP3_320 => 40,
            AudioFileFormat::MP3_160 => 20,
            AudioFileFormat::MP3_96 => 12,
            AudioFileFormat::MP3_160_ENC => 20,
            AudioFileFormat::MP4_128_DUAL => todo!(),
            AudioFileFormat::OTHER3 => todo!(),
            AudioFileFormat::AAC_160 => todo!(),
            AudioFileFormat::AAC_320 => todo!(),
            AudioFileFormat::MP4_128 => todo!(),
            AudioFileFormat::OTHER5 => todo!(),
        };
        kbps * 1024
    }
}

enum PlayerCommand {
    Load { track_id: String },
    LoadTracklist { tracks: Vec<Track> },
    Preload,
    Play,
    Pause,
    Stop,
    Seek(u32),
    Next,
    Previous,
    AddEventSender(mpsc::UnboundedSender<PlayerEvent>),
    Clear,
    GetTracks,
    GetCurrentTrack,
}

#[derive(Debug, Clone)]
pub enum PlayerEvent {
    Stopped,
    Started,
    Loading,
    Preloading,
    Playing,
    Paused,
    TimeToPreloadNextTrack,
    EndOfTrack { is_last_track: bool },
    VolumeSet { volume: u16 },
    Error { track_id: String, error: String },
    TracklistUpdated { tracks: (Vec<Track>, Vec<Track>) },
    CurrentTrack { track: Option<Track> },
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

    pub fn get_current_track(&self) -> Option<Option<Track>> {
        use PlayerEvent::*;
        match self {
            CurrentTrack { track, .. } => Some(track.clone()),
            _ => None,
        }
    }
}

pub type PlayerEventChannel = mpsc::UnboundedReceiver<PlayerEvent>;
