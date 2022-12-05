use std::sync::{Arc, Mutex};

use music_player_playback::{
    audio_backend::{self, rodio::RodioSink, Sink},
    config::AudioFormat,
    player::PlayerCommand,
};
use music_player_tracklist::Tracklist;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub mod library;
pub mod playback;
pub mod tracklist;

pub fn setup_new_params() -> (
    fn(Option<String>, AudioFormat) -> Box<dyn Sink>,
    AudioFormat,
    Arc<Mutex<UnboundedSender<PlayerCommand>>>,
    Arc<Mutex<UnboundedReceiver<PlayerCommand>>>,
    Arc<Mutex<Tracklist>>,
) {
    let audio_format = AudioFormat::default();
    let backend = audio_backend::find(Some(RodioSink::NAME.to_string())).unwrap();
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cmd_tx = Arc::new(Mutex::new(cmd_tx));
    let cmd_rx = Arc::new(Mutex::new(cmd_rx));
    let tracklist = Arc::new(Mutex::new(Tracklist::new_empty()));
    return (backend, audio_format, cmd_tx, cmd_rx, tracklist);
}
