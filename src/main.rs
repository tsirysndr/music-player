use std::sync::Arc;

use clap::{Arg, Command};
use music_player_playback::{
    audio_backend::{self, rodio::RodioSink},
    config::AudioFormat,
    player::{Player, PlayerEngine},
};
use music_player_server::server::MusicPlayerServer;
use tokio::sync::Mutex;
use std::{thread, time};

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
                .arg_from_usage("<song> 'The path to the song'")
                .arg(
                    Arg::new("loop")
                    .short('l')
                    .long("loop")
                    .takes_value(false)
                    .help("Play loop a song")
                ),
        )
        .subcommand(
            Command::new("playlists")
                .about("Add playlists")
                .arg(
                    Arg::with_name("paths")
                    .required(true)
                    .multiple(true)
                    .min_values(2)
                    .help("The songs path lists"),
                )
                .arg(
                    Arg::new("loop")
                    .short('l')
                    .long("loop")
                    .takes_value(false)
                    .help("Play loop the playlists")
                ),
        )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = cli().get_matches();

    let audio_format = AudioFormat::default();
    let backend = audio_backend::find(Some(RodioSink::NAME.to_string())).unwrap();

    let (mut player, _) = Player::new(move || backend(None, audio_format));

    if let Some(matches) = matches.subcommand_matches("play") {
        let is_loop: bool = matches.contains_id("loop");
        let song = matches.value_of("song").unwrap();

        if is_loop {
            loop {
                player.load(song, true, 0);
    
                player.await_end_of_track().await;
    
                thread::sleep(time::Duration::from_secs(3));
            }
        } else {
            
            player.load(song, true, 0);
    
            player.await_end_of_track().await;
        }

        return Ok(());
    }

    if let Some(matches) = matches.subcommand_matches("playlists") {
        let is_loop: bool = matches.contains_id("loop");
        let playlists: Vec<_> = matches.values_of("paths").unwrap().collect();

        if is_loop {
            loop {
                for path in &playlists {
                    player.load(path, true, 0);
        
                    player.await_end_of_track().await;
        
                    thread::sleep(time::Duration::from_secs(3));
                }
            }
        } else {
            for path in playlists {
                player.load(path, true, 0);
    
                player.await_end_of_track().await;
    
                thread::sleep(time::Duration::from_secs(3));
            }
        }
        return Ok(());
    }

    migration::run().await;

    MusicPlayerServer::new(Arc::new(Mutex::new(player)))
        .start()
        .await
}
