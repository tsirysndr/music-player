use music_player_playback::player::Player;
use std::sync::Arc;

use super::setup_schema;

#[tokio::test]
async fn tracklist_tracks() {
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
async fn add_track() {
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
async fn add_tracks() {
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
async fn clear_tracklist() {
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
async fn remove_track() {
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
async fn remove_tracks() {
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
async fn play_track_at() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}
