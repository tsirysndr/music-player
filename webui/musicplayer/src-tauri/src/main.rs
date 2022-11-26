#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::{sync::Arc};

use async_graphql::{Response, Request, Schema};
use music_player_graphql::{
  schema::{Mutation, Query, Subscription},
  MusicPlayerSchema,
};
use music_player_playback::{
    audio_backend::{self, rodio::RodioSink},
    config::AudioFormat,
    player::Player,
};
use music_player_tracklist::Tracklist;
use music_player_storage::Database;
use tokio::sync::{mpsc, Mutex};

#[tauri::command]
async fn execute_graphql(request: Request, schema: tauri::State<'_, MusicPlayerSchema>) -> Result<Response, ()> {
  Ok(schema.execute(request).await)
}

#[tokio::main]
async fn main() {
  let audio_format = AudioFormat::default();
  let backend = audio_backend::find(Some(RodioSink::NAME.to_string())).unwrap();
  let tracklist = Arc::new(std::sync::Mutex::new(Tracklist::new_empty()));
  let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
  let cmd_tx = Arc::new(std::sync::Mutex::new(cmd_tx));
  let cmd_rx = Arc::new(std::sync::Mutex::new(cmd_rx));
  let (_, _) = Player::new(
      move || backend(None, audio_format),
      move |_| {},
      Arc::clone(&cmd_tx),
      Arc::clone(&cmd_rx),
      tracklist.clone(),
  );
  let db = Arc::new(Mutex::new(Database::new().await));
  let schema: MusicPlayerSchema = Schema::build(
      Query::default(),
      Mutation::default(),
      Subscription::default(),
  )
    .data(db)
    .data(cmd_tx)
    .data(tracklist)
    .finish();

  tauri::async_runtime::set(tokio::runtime::Handle::current());
  tauri::Builder::default()
    .manage(schema)
    .invoke_handler(tauri::generate_handler![execute_graphql])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
