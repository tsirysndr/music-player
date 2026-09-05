use std::{env, sync::Arc};

use async_graphql::Schema;
use music_player_addons::{CurrentDevice, CurrentReceiverDevice, CurrentSourceDevice};
use music_player_playback::player::PlayerCommand;
use music_player_storage::{searcher::Searcher, Database};
use music_player_tracklist::Tracklist;
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};

use crate::{
    scan_devices,
    schema::{Mutation, Query, Subscription},
    MusicPlayerSchema,
};

pub mod library;
pub mod objects;
pub mod playback;
pub mod playlist;
pub mod tracklist;

pub async fn setup_schema() -> (
    MusicPlayerSchema,
    Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>,
    Arc<std::sync::Mutex<UnboundedReceiver<PlayerCommand>>>,
    Arc<std::sync::Mutex<Tracklist>>,
) {
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cmd_tx = Arc::new(std::sync::Mutex::new(cmd_tx));
    let cmd_rx = Arc::new(std::sync::Mutex::new(cmd_rx));
    let tracklist = Arc::new(std::sync::Mutex::new(Tracklist::new_empty()));
    let devices = scan_devices().await.unwrap();
    let current_device = Arc::new(Mutex::new(CurrentDevice::new()));
    let source_device = Arc::new(Mutex::new(CurrentSourceDevice::new()));
    let receiver_device = Arc::new(Mutex::new(CurrentReceiverDevice::new()));

    env::set_var("MUSIC_PLAYER_APPLICATION_DIRECTORY", "/tmp");
    env::set_var("MUSIC_PLAYER_MUSIC_DIRECTORY", "/tmp/audio");
    env::set_var(
        "MUSIC_PLAYER_DATABASE_URL",
        "sqlite:///tmp/music-player.sqlite3",
    );

    ensure_test_library().await;

    let db = Database::new().await;
    let searcher = Arc::new(Searcher::new(db.get_connection().clone()));
    (
        Schema::build(
            Query::default(),
            Mutation::default(),
            Subscription::default(),
        )
        .data(db)
        .data(Arc::clone(&cmd_tx))
        .data(Arc::clone(&tracklist))
        .data(Arc::clone(&devices))
        .data(Arc::clone(&current_device))
        .data(Arc::clone(&source_device))
        .data(Arc::clone(&receiver_device))
        .data(Arc::clone(&searcher))
        .finish(),
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    )
}

pub async fn new_playlist(schema: MusicPlayerSchema) -> String {
    let resp = schema
        .execute(
            r#"
        mutation {
            createPlaylist(name: "New Playlist") {
                id
            }
        }"#,
        )
        .await;
    resp.data.into_json().unwrap()["createPlaylist"]["id"].to_string()
}

pub async fn new_folder(schema: MusicPlayerSchema) -> String {
    let resp = schema
        .execute(
            r#"
        mutation {
            createFolder(name: "New Folder") {
                id
            }
        }"#,
        )
        .await;
    resp.data.into_json().unwrap()["createFolder"]["id"].to_string()
}

pub async fn play_album(schema: MusicPlayerSchema) {
    schema
        .execute(
            r#"
              mutation PlayAlbum {
                playAlbum(id: "216ccc791352fbbffc11268b984db19a", shuffle: false) 
              }
            "#,
        )
        .await;
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
