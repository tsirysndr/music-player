use clap::Command;
use music_player::{
    audio_backend::{self, rodio::RodioSink},
    config::AudioFormat,
    player::Player,
};

mod formatter;

fn cli() -> Command<'static> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    Command::new("music-player")
        .version(VERSION)
        .author("Tsiry Sandratraina <tsiry.sndr@aol.com>")
        .about("A simple music player written in Rust")
        .subcommand(
            Command::new("play")
                .about("Play a song")
                .arg_from_usage("<song> 'The path to the song'"),
        )
}

#[tokio::main]
async fn main() {
    let matches = cli().get_matches();

    if let Some(matches) = matches.subcommand_matches("play") {
        let song = matches.value_of("song").unwrap();

        formatter::print_format(song);

        let audio_format = AudioFormat::default();
        let backend = audio_backend::find(Some(RodioSink::NAME.to_string())).unwrap();

        let (mut player, _) = Player::new(move || backend(None, audio_format));
        player.load(song, true, 0);

        player.await_end_of_track().await;
    }
}
