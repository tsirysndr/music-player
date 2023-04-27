use std::{env, sync::Arc};

use async_graphql::Schema;
use music_player_addons::{CurrentDevice, CurrentReceiverDevice, CurrentSourceDevice};
use music_player_playback::{
    audio_backend::{self, rodio::RodioSink, Sink},
    config::AudioFormat,
    player::PlayerCommand,
};
use music_player_storage::{searcher::Searcher, Database};
use music_player_tracklist::Tracklist;
use music_player_types::types::Device;
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
    fn(Option<String>, AudioFormat) -> Box<dyn Sink>,
    AudioFormat,
) {
    let audio_format = AudioFormat::default();
    let backend = audio_backend::find(Some(RodioSink::NAME.to_string())).unwrap();
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cmd_tx = Arc::new(std::sync::Mutex::new(cmd_tx));
    let cmd_rx = Arc::new(std::sync::Mutex::new(cmd_rx));
    let tracklist = Arc::new(std::sync::Mutex::new(Tracklist::new_empty()));
    let devices = scan_devices().await.unwrap();
    let current_device = Arc::new(Mutex::new(CurrentDevice::new()));
    let source_device = Arc::new(Mutex::new(CurrentSourceDevice::new()));
    let receiver_device = Arc::new(Mutex::new(CurrentReceiverDevice::new()));
    let searcher = Arc::new(Mutex::new(Searcher::new()));

    env::set_var("MUSIC_PLAYER_APPLICATION_DIRECTORY", "/tmp");
    env::set_var("MUSIC_PLAYER_MUSIC_DIRECTORY", "/tmp/audio");
    env::set_var(
        "MUSIC_PLAYER_DATABASE_URL",
        "sqlite:///tmp/music-player.sqlite3",
    );

    let db = Database::new().await;
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
        backend,
        audio_format,
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
