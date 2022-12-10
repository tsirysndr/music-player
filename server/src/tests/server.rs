use std::sync::Arc;

use music_player_playback::player::Player;

use super::setup_new_params;

#[tokio::test]
async fn new() {
    let (backend, audio_format, cmd_tx, cmd_rx, tracklist, db, addr, url) =
        setup_new_params(6062).await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}

#[tokio::test]
async fn start() {
    let (backend, audio_format, cmd_tx, cmd_rx, tracklist, db, addr, url) =
        setup_new_params(6060).await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}

#[tokio::test]
async fn start_ws() {
    let (backend, audio_format, cmd_tx, cmd_rx, tracklist, db, addr, url) =
        setup_new_params(6061).await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}
