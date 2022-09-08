use futures_util::{future::FusedFuture, Future};
use log::{error, trace};
use parking_lot::Mutex;
use std::{
    collections::HashMap,
    fs::File,
    path::Path,
    pin::Pin,
    process::exit,
    sync::Arc,
    task::{Context, Poll},
    thread,
};
use symphonia::core::{errors::Error, io::MediaSourceStream, probe::Hint};
use tokio::sync::mpsc;

use crate::{
    audio_backend::Sink,
    convert::Converter,
    decoder::{symphonia_decoder::SymphoniaDecoder, AudioDecoder},
    dither::{mk_ditherer, TriangularDitherer},
    metadata::audio::AudioFileFormat,
};

const PRELOAD_NEXT_TRACK_BEFORE_END: u64 = 30000;

pub type PlayerResult = Result<(), Error>;

pub struct Player {
    commands: Option<mpsc::UnboundedSender<PlayerCommand>>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl Player {
    pub fn new<F>(sink_builder: F) -> (Player, PlayerEventChannel)
    where
        F: FnOnce() -> Box<dyn Sink> + Send + 'static,
    {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let handle = thread::spawn(move || {
            let internal = PlayerInternal {
                commands: cmd_rx,
                load_handles: Arc::new(Mutex::new(HashMap::new())),
                sink: sink_builder(),
                state: PlayerState::Stopped,
                sink_status: SinkStatus::Closed,
                sink_event_callback: None,
                event_senders: [event_sender].to_vec(),
            };
            let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            runtime.block_on(internal);
        });
        (
            Player {
                commands: Some(cmd_tx),
                thread_handle: Some(handle),
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

    pub fn load(&mut self, track_id: &str, start_playing: bool, position_ms: u32) {
        self.command(PlayerCommand::Load {
            track_id: track_id.to_string(),
        });
    }

    pub fn preload(&self, track_id: &str) {
        self.command(PlayerCommand::Preload);
    }

    pub fn play(&self) {
        self.command(PlayerCommand::Play)
    }

    pub fn pause(&self) {
        self.command(PlayerCommand::Pause)
    }

    pub fn stop(&self) {
        self.command(PlayerCommand::Stop)
    }

    pub fn seek(&self, position_ms: u32) {
        self.command(PlayerCommand::Seek(position_ms));
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

            if let PlayerState::Playing {
                ref mut decoder,
                track_id: _,
            } = self.state
            {
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
                            self.send_event(PlayerEvent::EndOfTrack);
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
            PlayerCommand::Load { track_id } => {
                self.handle_command_load(&track_id);
            }
            PlayerCommand::Preload => {
                self.handle_command_preload();
            }
            PlayerCommand::Play => {
                self.handle_play();
            }
            PlayerCommand::Pause => {
                self.handle_pause();
            }
            PlayerCommand::Stop => {
                self.handle_player_stop();
            }
            PlayerCommand::Seek(position_ms) => {
                self.handle_command_seek();
            }
            PlayerCommand::AddEventSender(sender) => self.event_senders.push(sender),
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
        self.send_event(PlayerEvent::Playing {
            track_id: track_id.to_string(),
        });

        self.state = PlayerState::Playing {
            track_id: track_id.to_string(),
            decoder: loaded_track.decoder,
        };
    }

    fn send_event(&mut self, event: PlayerEvent) {
        self.event_senders
            .retain(|sender| sender.send(event.clone()).is_ok());
    }

    fn handle_command_load(&mut self, track_id: &str) {
        println!("load track {}", track_id);
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

    fn handle_command_preload(&self) {
        todo!()
    }

    fn handle_play(&self) {
        todo!()
    }

    fn handle_player_stop(&mut self) {
        self.ensure_sink_stopped(false);
    }

    fn handle_pause(&self) {
        todo!()
    }

    fn handle_command_seek(&self) {
        todo!()
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
        track_id: String,
        loader: Pin<Box<dyn FusedFuture<Output = Result<PlayerLoadedTrackData, ()>> + Send>>,
    },
    Paused {
        decoder: Decoder,
    },
    Playing {
        decoder: Decoder,
        track_id: String,
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
    Preload,
    Play,
    Pause,
    Stop,
    Seek(u32),
    AddEventSender(mpsc::UnboundedSender<PlayerEvent>),
}

#[derive(Debug, Clone)]
pub enum PlayerEvent {
    Stopped,
    Started,
    Loading { track_id: String },
    Preloading,
    Playing { track_id: String },
    Paused,
    TimeToPreloadNextTrack,
    EndOfTrack,
    VolumeSet { volume: u16 },
    Error { track_id: String, error: String },
}

pub type PlayerEventChannel = mpsc::UnboundedReceiver<PlayerEvent>;
