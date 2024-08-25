use std::{env, sync::Arc};

use extism::UserData;
use music_player_host_fn::state::State;
use music_player_playback::{
    audio_backend::{self},
    config::AudioFormat,
    player::Player,
};
use music_player_settings::{read_settings, Settings};
use music_player_storage::Database;
use music_player_tracklist::Tracklist;
use music_player_webui::start_webui;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let audio_format = AudioFormat::default();
    let config = read_settings().unwrap();
    let settings = config.try_deserialize::<Settings>().unwrap();

    let backend = audio_backend::find(match env::var("MUSIC_PLAYER_AUDIO_BACKEND") {
        Ok(backend) => Some(backend),
        Err(_) => settings.audio_backend,
    })
    .unwrap();

    let tracklist = Arc::new(std::sync::Mutex::new(Tracklist::new_empty()));
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cmd_tx = Arc::new(std::sync::Mutex::new(cmd_tx));
    let cmd_rx = Arc::new(std::sync::Mutex::new(cmd_rx));
    let cloned_tracklist = tracklist.clone();
    let cloned_cmd_tx = Arc::clone(&cmd_tx);
    let cloned_cmd_rx = Arc::clone(&cmd_rx);
    let db = Database::new().await;

    let user_data = UserData::new(State {
        player_cmd_tx: Arc::clone(&cloned_cmd_tx),
        tracklist: Arc::clone(&cloned_tracklist),
        db: db.clone(),
        addons: vec![],
        addon_capabilities: vec![],
    });

    let (_, _) = Player::new(
        move || {
            backend(
                match env::var("MUSIC_PLAYER_DEVICE") {
                    Ok(device) => Some(device),
                    Err(_) => settings.device,
                },
                audio_format,
            )
        },
        move |_| {},
        cloned_cmd_tx,
        cloned_cmd_rx,
        cloned_tracklist,
    );
    env::set_var("MUSIC_PLAYER_HTTP_PORT", "3001");
    start_webui(cmd_tx, tracklist, user_data).await
}
