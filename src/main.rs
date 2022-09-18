use std::sync::Arc;

use clap::Command;
use music_player_playback::{
    audio_backend::{self, rodio::RodioSink},
    config::AudioFormat,
    player::{Player, PlayerEngine},
};
use music_player_server::server::MusicPlayerServer;
use scan::scan_music_library;
use tokio::sync::Mutex;

mod scan;

fn cli() -> Command<'static> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    Command::new("music-player")
        .version(VERSION)
        .author("Tsiry Sandratraina <tsiry.sndr@aol.com>")
        .about(
            r#"
     __  ___           _      ____  __                     
    /  |/  /_  _______(_)____/ __ \/ /___ ___  _____  _____
   / /|_/ / / / / ___/ / ___/ /_/ / / __ `/ / / / _ \/ ___/
  / /  / / /_/ (__  ) / /__/ ____/ / /_/ / /_/ /  __/ /    
 /_/  /_/\__,_/____/_/\___/_/   /_/\__,_/\__, /\___/_/     
                                        /____/             
 
A simple music player written in Rust"#,
        )
        .subcommand(
            Command::new("play")
                .about("Play a song")
                .arg_from_usage("<song> 'The path to the song'"),
        )
        .subcommand(Command::new("scan").about("Scan music library: $HOME/Music"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = cli().get_matches();

    let audio_format = AudioFormat::default();
    let backend = audio_backend::find(Some(RodioSink::NAME.to_string())).unwrap();

    let (mut player, _) = Player::new(move || backend(None, audio_format));

    if let Some(matches) = matches.subcommand_matches("play") {
        let song = matches.value_of("song").unwrap();

        player.load(song, true, 0);

        player.await_end_of_track().await;
        return Ok(());
    }

    if let Some(_matches) = matches.subcommand_matches("scan") {
        scan_music_library().await.map_err(|e| e.to_string())?;
        return Ok(());
    }

    migration::run().await;

    MusicPlayerServer::new(Arc::new(Mutex::new(player)))
        .start()
        .await
}
