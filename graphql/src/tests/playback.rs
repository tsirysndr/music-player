use super::setup_schema;
use music_player_playback::player::Player;
use std::sync::Arc;

#[tokio::test]
async fn currently_playing_song() {
    setup_schema().await;
}

#[tokio::test]
async fn next() {
    setup_schema().await;
}

#[tokio::test]
async fn previous() {
    setup_schema().await;
}

#[tokio::test]
async fn play() {
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
async fn pause() {
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
async fn stop() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}
