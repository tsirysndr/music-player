use std::{env, sync::Arc};

use async_graphql::Schema;
use futures_util::FutureExt;
use music_player_entity::{album, artist, artist_tracks, track};
use music_player_playback::{
    audio_backend::{self, rodio::RodioSink, Sink},
    config::AudioFormat,
    player::PlayerCommand,
};
use music_player_scanner::scan_directory;
use music_player_storage::Database;
use music_player_tracklist::Tracklist;
use sea_orm::ActiveModelTrait;
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};

use crate::{
    schema::{Mutation, Query, Subscription},
    MusicPlayerSchema,
};

pub mod library;
pub mod playback;
pub mod playlist;
pub mod tracklist;

#[cfg(test)]
#[ctor::ctor]
fn initialize() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(setup_database());
}

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

    let db = Arc::new(Mutex::new(Database::new().await));
    (
        Schema::build(
            Query::default(),
            Mutation::default(),
            Subscription::default(),
        )
        .data(db)
        .data(Arc::clone(&cmd_tx))
        .data(Arc::clone(&tracklist))
        .finish(),
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
        backend,
        audio_format,
    )
}

async fn setup_database() {
    env::set_var("MUSIC_PLAYER_APPLICATION_DIRECTORY", "/tmp");
    env::set_var("MUSIC_PLAYER_MUSIC_DIRECTORY", "/tmp/audio");
    env::set_var(
        "MUSIC_PLAYER_DATABASE_URL",
        "sqlite:///tmp/music-player.sqlite3",
    );
    env::set_var("DATABASE_URL", "sqlite:///tmp/music-player.sqlite3");

    migration::run().await;
    scan_directory(move |song, db| {
        async move {
            let item: artist::ActiveModel = song.try_into().unwrap();
            match item.insert(db.get_connection()).await {
                Ok(_) => (),
                Err(_) => (),
            }

            let item: album::ActiveModel = song.try_into().unwrap();
            match item.insert(db.get_connection()).await {
                Ok(_) => (),
                Err(_) => (),
            }

            let item: track::ActiveModel = song.try_into().unwrap();

            match item.insert(db.get_connection()).await {
                Ok(_) => (),
                Err(_) => (),
            }

            let item: artist_tracks::ActiveModel = song.try_into().unwrap();
            match item.insert(db.get_connection()).await {
                Ok(_) => (),
                Err(_) => (),
            }
        }
        .boxed()
    })
    .await
    .unwrap_or_default();
}
