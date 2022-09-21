use clap::ArgMatches;
use music_player_client::{
    library::LibraryClient, playback::PlaybackClient, playlist::PlaylistClient,
    tracklist::TracklistClient,
};
use music_player_playback::{
    audio_backend::{self, rodio::RodioSink},
    config::AudioFormat,
    player::{Player, PlayerEngine},
};
use owo_colors::OwoColorize;
use tabled::{builder::Builder, Style};

use crate::scan::scan_music_library;

pub async fn parse_args(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(matches) = matches.subcommand_matches("play") {
        let audio_format = AudioFormat::default();
        let backend = audio_backend::find(Some(RodioSink::NAME.to_string())).unwrap();

        let (mut player, _) = Player::new(move || backend(None, audio_format));

        let song = matches.value_of("song").unwrap();

        player.load(song, true, 0);

        player.await_end_of_track().await;
        return Ok(());
    }

    if let Some(_) = matches.subcommand_matches("scan") {
        scan_music_library(true).await.map_err(|e| e.to_string())?;
        return Ok(());
    }

    if let Some(_) = matches.subcommand_matches("albums") {
        let mut client = LibraryClient::new().await?;
        let result = client.albums().await?;

        let mut builder = Builder::default();
        builder.set_columns(["id", "name"]);
        result.iter().for_each(|album| {
            builder.add_record([
                album.id.as_str(),
                album.title.magenta().to_string().as_str(),
            ]);
        });
        let table = builder.build().with(Style::psql());
        println!("\n{}", table);

        return Ok(());
    }

    if let Some(_) = matches.subcommand_matches("artists") {
        let mut client = LibraryClient::new().await?;
        let result = client.artists().await?;

        let mut builder = Builder::default();
        builder.set_columns(["id", "name"]);
        result.iter().for_each(|artist| {
            builder.add_record([
                artist.id.as_str(),
                artist.name.magenta().to_string().as_str(),
            ]);
        });
        let table = builder.build().with(Style::psql());
        println!("\n{}", table);

        return Ok(());
    }

    if let Some(matches) = matches.subcommand_matches("playlist") {
        let mut client = PlaylistClient::new().await?;

        if let Some(matches) = matches.subcommand_matches("add") {
            let id = matches.value_of("id").unwrap();

            return Ok(());
        }

        if let Some(_matches) = matches.subcommand_matches("ls") {
            return Ok(());
        }

        if let Some(matches) = matches.subcommand_matches("clear") {
            let id = matches.value_of("id");

            return Ok(());
        }

        if let Some(matches) = matches.subcommand_matches("play") {
            let id = matches.value_of("id");

            return Ok(());
        }

        if let Some(matches) = matches.subcommand_matches("remove") {
            let id = matches.value_of("id").unwrap();

            return Ok(());
        }

        if let Some(_matches) = matches.subcommand_matches("shuffle") {
            return Ok(());
        }

        if let Some(_matches) = matches.subcommand_matches("all") {
            return Ok(());
        }
    }

    if let Some(matches) = matches.subcommand_matches("queue") {
        let client = TracklistClient::new().await?;

        if let Some(matches) = matches.subcommand_matches("list") {
            let all = matches.is_present("all");

            return Ok(());
        }

        if let Some(matches) = matches.subcommand_matches("add") {
            let song = matches.value_of("song").unwrap();

            return Ok(());
        }

        if let Some(matches) = matches.subcommand_matches("remove") {
            let song = matches.value_of("song").unwrap();

            return Ok(());
        }

        if let Some(matches) = matches.subcommand_matches("clear") {
            let all = matches.is_present("all");

            return Ok(());
        }
    }

    if let Some(_matches) = matches.subcommand_matches("tracks") {
        let mut client = LibraryClient::new().await?;
        let result = client.songs().await?;

        let mut builder = Builder::default();
        builder.set_columns(["id", "title"]);
        result.iter().for_each(|song| {
            builder.add_record([
                song.id.as_str(),
                song.title.magenta().to_string().as_str(),
            ]);
        });
        let table = builder.build().with(Style::psql());
        println!("\n{}", table);

        return Ok(());
    }

    if let Some(matches) = matches.subcommand_matches("search") {
        let client = LibraryClient::new().await?;

        let query = matches.value_of("query").unwrap();
        todo!("search for {}", query);
    }

    if let Some(_) = matches.subcommand_matches("pause") {
        let client = PlaybackClient::new().await?;
        return Ok(());
    }

    if let Some(_) = matches.subcommand_matches("resume") {
        let client = PlaybackClient::new().await?;
        return Ok(());
    }

    if let Some(_) = matches.subcommand_matches("next") {
        let client = PlaybackClient::new().await?;
        return Ok(());
    }

    if let Some(_) = matches.subcommand_matches("prev") {
        let client = PlaybackClient::new().await?;
        return Ok(());
    }

    if let Some(_) = matches.subcommand_matches("stop") {
        let client = PlaybackClient::new().await?;
        return Ok(());
    }

    return Err("No subcommand found".into());
}
