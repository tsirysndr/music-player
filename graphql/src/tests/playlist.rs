use music_player_playback::player::Player;

use super::setup_schema;
use std::sync::Arc;

#[tokio::test]
async fn playlist() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}

#[tokio::test]
async fn playlists() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}

#[tokio::test]
async fn folder() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}

#[tokio::test]
async fn folders() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}

#[tokio::test]
async fn create_playlist() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}

#[tokio::test]
async fn delete_playlist() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}

#[tokio::test]
async fn add_track_to_playlist() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}

#[tokio::test]
async fn remove_track_from_playlist() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}

#[tokio::test]
async fn rename_playlist() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}

#[tokio::test]
async fn create_folder() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}

#[tokio::test]
async fn delete_folder() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}

#[tokio::test]
async fn rename_folder() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}

#[tokio::test]
async fn move_playlist_to_folder() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}

#[tokio::test]
async fn move_playlists_to_folder() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}
