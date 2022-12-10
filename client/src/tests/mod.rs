use std::{
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use music_player_playback::{
    audio_backend::{self, rodio::RodioSink, Sink},
    config::AudioFormat,
    player::PlayerCommand,
};
use music_player_settings::{read_settings, Settings};
use music_player_storage::Database;
use music_player_tracklist::Tracklist;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub mod library;
pub mod playback;
pub mod playlist;
pub mod tracklist;

pub async fn setup_new_params(
    port: u16,
) -> (
    fn(Option<String>, AudioFormat) -> Box<dyn Sink>,
    AudioFormat,
    Arc<Mutex<UnboundedSender<PlayerCommand>>>,
    Arc<Mutex<UnboundedReceiver<PlayerCommand>>>,
    Arc<Mutex<Tracklist>>,
    Arc<tokio::sync::Mutex<Database>>,
    SocketAddr,
    String,
) {
    let audio_format = AudioFormat::default();
    let backend = audio_backend::find(Some(RodioSink::NAME.to_string())).unwrap();
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cmd_tx = Arc::new(Mutex::new(cmd_tx));
    let cmd_rx = Arc::new(Mutex::new(cmd_rx));
    let tracklist = Arc::new(Mutex::new(Tracklist::new_empty()));

    env::set_var("MUSIC_PLAYER_APPLICATION_DIRECTORY", "/tmp");
    env::set_var("MUSIC_PLAYER_MUSIC_DIRECTORY", "/tmp/audio");
    env::set_var(
        "MUSIC_PLAYER_DATABASE_URL",
        "sqlite:///tmp/music-player.sqlite3",
    );

    let config = read_settings().unwrap();
    let settings = config.try_deserialize::<Settings>().unwrap();
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    let url = format!("http://{}:{}", settings.host, port);

    let db = Arc::new(tokio::sync::Mutex::new(Database::new().await));

    return (
        backend,
        audio_format,
        cmd_tx,
        cmd_rx,
        tracklist,
        db,
        addr,
        url,
    );
}
