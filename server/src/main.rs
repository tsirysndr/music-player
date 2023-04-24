use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{self, Arc},
};

use futures_channel::mpsc::UnboundedSender;
use music_player_playback::{
    audio_backend::{self, rodio::RodioSink},
    config::AudioFormat,
    player::Player,
};
use music_player_server::server::MusicPlayerServer;
use music_player_storage::Database;
use music_player_tracklist::Tracklist;
use tokio::sync::Mutex;
use tungstenite::Message;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<sync::Mutex<HashMap<SocketAddr, Tx>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let audio_format = AudioFormat::default();
    let backend = audio_backend::find(Some(RodioSink::NAME.to_string())).unwrap();
    let peer_map: PeerMap = Arc::new(sync::Mutex::new(HashMap::new()));

    let tracklist = Arc::new(std::sync::Mutex::new(Tracklist::new_empty()));
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cmd_tx = Arc::new(std::sync::Mutex::new(cmd_tx));
    let cmd_rx = Arc::new(std::sync::Mutex::new(cmd_rx));
    let db = Database::new().await;

    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    MusicPlayerServer::new(tracklist, Arc::clone(&cmd_tx), Arc::clone(&peer_map), db)
        .start()
        .await?;

    Ok(())
}
