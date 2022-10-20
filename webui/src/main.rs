use std::sync::Arc;

use music_player_playback::{
    audio_backend::{self, rodio::RodioSink},
    config::AudioFormat,
    player::Player,
};
use music_player_tracklist::Tracklist;
use music_player_webui::start_webui;
use tokio::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let audio_format = AudioFormat::default();
    let backend = audio_backend::find(Some(RodioSink::NAME.to_string())).unwrap();
    let tracklist = Arc::new(std::sync::Mutex::new(Tracklist::new_empty()));
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cloned_tracklist = tracklist.clone();
    let cloned_cmd_tx = cmd_tx.clone();
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        move |_| {},
        cloned_cmd_tx,
        cmd_rx,
        cloned_tracklist,
    );
    start_webui(cmd_tx, tracklist).await
}
