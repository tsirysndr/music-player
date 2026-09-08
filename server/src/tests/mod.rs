use std::{
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use music_player_playback::player::PlayerCommand;
use music_player_settings::{read_settings, Settings};
use music_player_storage::Database;
use music_player_tracklist::Tracklist;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub mod library;
pub mod playback;
pub mod server;
pub mod tracklist;

pub async fn setup_new_params(
    port: u16,
) -> (
    Arc<Mutex<UnboundedSender<PlayerCommand>>>,
    Arc<Mutex<UnboundedReceiver<PlayerCommand>>>,
    Arc<Mutex<Tracklist>>,
    Database,
    SocketAddr,
    String,
) {
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

    ensure_test_library().await;

    let config = read_settings().unwrap();
    let settings = config.try_deserialize::<Settings>().unwrap();
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    let url = format!("http://{}:{}", settings.host, port);

    let db = Database::new().await;

    return (cmd_tx, cmd_rx, tracklist, db, addr, url);
}

static TEST_LIBRARY: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();

/// Create + migrate the test database and index the fixture library
/// (/tmp/audio) once per test process.
async fn ensure_test_library() {
    TEST_LIBRARY
        .get_or_init(|| async {
            migration::apply().await;
            music_player_scanner::scan_music_library(false, Database::new().await)
                .await
                .ok();
        })
        .await;
}

/// A provider state with nothing connected, so the services take their
/// local-library branch — which is what these tests exercise.
pub(crate) fn local_only() -> std::sync::Arc<music_player_provider::ProviderState> {
    let mut registry = music_player_provider::ProviderRegistry::new();
    music_player_provider::register_builtin(&mut registry);
    std::sync::Arc::new(music_player_provider::ProviderState::new(
        std::sync::Arc::new(registry),
    ))
}
