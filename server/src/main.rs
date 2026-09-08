use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{self, Arc},
};

use futures_channel::mpsc::UnboundedSender;
use music_player_playback::player::Player;
use music_player_server::server::MusicPlayerServer;
use music_player_storage::Database;
use music_player_tracklist::Tracklist;
use tungstenite::Message;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<sync::Mutex<HashMap<SocketAddr, Tx>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let peer_map: PeerMap = Arc::new(sync::Mutex::new(HashMap::new()));

    let tracklist = Arc::new(std::sync::Mutex::new(Tracklist::new_empty()));
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cmd_tx = Arc::new(std::sync::Mutex::new(cmd_tx));
    let cmd_rx = Arc::new(std::sync::Mutex::new(cmd_rx));
    let db = Database::new().await;

    let (_, _) = Player::new(
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    // One provider registry per process, shared by the gRPC server and the
    // GraphQL/web server — otherwise the two disagree about which server the
    // library screens are reading from, which is exactly what left the Slint
    // desktop showing local data after a switch.
    let providers = {
        let mut registry = music_player_provider::ProviderRegistry::new();
        music_player_provider::register_builtin(&mut registry);
        std::sync::Arc::new(music_player_provider::ProviderState::new(
            std::sync::Arc::new(registry),
        ))
    };
    MusicPlayerServer::new(
        tracklist,
        Arc::clone(&cmd_tx),
        Arc::clone(&peer_map),
        db,
        providers,
    )
        .start()
        .await?;

    Ok(())
}
