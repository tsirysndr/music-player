use std::sync::Arc;

use music_player_playback::{
    audio_backend::{self, rodio::RodioSink},
    config::AudioFormat,
    player::Player,
};
use music_player_webui::start_webui;
use tokio::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let audio_format = AudioFormat::default();
    let backend = audio_backend::find(Some(RodioSink::NAME.to_string())).unwrap();

    let (player, _) = Player::new(move || backend(None, audio_format), move |event| {});
    start_webui(Arc::new(Mutex::new(player))).await
}
