//! CLI subcommand dispatch. Each subcommand is a small handler; `parse_args`
//! only routes. Handlers that talk to a daemon connect with the host/port
//! from settings.toml.

use std::{env, fs, sync::Mutex};

use clap::ArgMatches;
use futures::StreamExt;
use music_player_client::{
    library::LibraryClient, playback::PlaybackClient, playlist::PlaylistClient,
    tracklist::TracklistClient,
};
use music_player_discovery::{discover, SERVICE_NAME};
use music_player_playback::player::{Player, PlayerEngine};
use music_player_settings::{read_settings, Settings};
use music_player_storage::Database;
use music_player_tracklist::Tracklist;
use owo_colors::OwoColorize;
use std::sync::Arc;
use tabled::{builder::Builder, Style};

use crate::scan::scan_music_library;

type CmdResult = Result<(), Box<dyn std::error::Error>>;

/// A one-shot transport action against the daemon.
enum Transport {
    Play,
    Pause,
    Next,
    Prev,
    Stop,
}

/// The on-disk cache of remote track audio.
///
/// Local: it describes files on *this* machine, so it needs no daemon and
/// works whether or not one is running.
fn cache(matches: &ArgMatches) -> CmdResult {
    use music_player_storage::track_cache;

    match matches.subcommand() {
        Some(("clear", _)) => {
            let freed = track_cache::clear();
            println!(
                "{}",
                track_cache::cache_dir().display().to_string().dimmed()
            );
            println!(
                "Cleared {} track{} ({} freed)",
                freed.tracks.to_string().bright_green(),
                if freed.tracks == 1 { "" } else { "s" },
                track_cache::format_bytes(freed.bytes).bright_green()
            );
        }
        // `ls` or nothing: showing what is there is the harmless default.
        _ => {
            let usage = track_cache::usage();
            // The path first, and dimmed: it is what you need to inspect or
            // delete the cache by hand, but it is not the answer to "how much
            // is it using".
            println!(
                "{}",
                track_cache::cache_dir().display().to_string().dimmed()
            );
            println!(
                "{} track{}, {}",
                usage.tracks.to_string().bright_green(),
                if usage.tracks == 1 { "" } else { "s" },
                track_cache::format_bytes(usage.bytes).bright_green()
            );
            // Otherwise "0 tracks" reads as a cache that is not working,
            // rather than one that was never asked for. Said after the figures
            // so that files left by an earlier run are still accounted for.
            if !track_cache::enabled() {
                println!(
                    "{}",
                    "caching is off — set `cache = true` in settings.toml".dimmed()
                );
            }
        }
    }
    Ok(())
}

/// Serve MCP on stdin/stdout.
///
/// Nothing may be printed here: stdout is the protocol, and a stray line would
/// be a parse error at the host rather than a message anyone reads.
async fn mcp(settings: &Settings) -> CmdResult {
    let session = music_player_mcp::Session::new(settings.host.clone(), settings.port);
    music_player_mcp::serve(session).await?;
    Ok(())
}

pub async fn parse_args(matches: ArgMatches) -> CmdResult {
    let config = read_settings().unwrap();
    let settings = config.try_deserialize::<Settings>().unwrap();

    match matches.subcommand() {
        Some(("open", m)) => open(m).await,
        Some(("scan", _)) => scan().await,
        Some(("albums", m)) => albums(m, &settings).await,
        Some(("artists", _)) => artists(&settings).await,
        Some(("cache", m)) => cache(m),
        Some(("mcp", _)) => mcp(&settings).await,
        Some(("tracks", _)) => tracks(&settings).await,
        Some(("search", m)) => search(m, &settings).await,
        Some(("playlist", m)) => playlist(m, &settings).await,
        Some(("queue", m)) => queue(m, &settings).await,
        Some(("play", _)) => transport(&settings, Transport::Play).await,
        Some(("pause", _)) => transport(&settings, Transport::Pause).await,
        Some(("next", _)) => transport(&settings, Transport::Next).await,
        Some(("prev", _)) => transport(&settings, Transport::Prev).await,
        Some(("stop", _)) => transport(&settings, Transport::Stop).await,
        Some(("current", _)) => current(&settings).await,
        Some(("connect", m)) => connect(m),
        Some(("devices", _)) => devices().await,
        Some(("reset", _)) => reset(),
        Some(("extension", m)) => extension(m).await,
        // No subcommand: the caller starts the daemon or the TUI.
        _ => Err("No subcommand found".into()),
    }
}

/// `extension …` — scaffold, list and install WebAssembly extensions.
async fn extension(matches: &ArgMatches) -> CmdResult {
    match matches.subcommand() {
        Some(("init", m)) => {
            crate::extension::init(
                m.get_one::<String>("id").unwrap(),
                m.get_one::<String>("name").map(String::as_str),
                m.get_one::<String>("capabilities").unwrap(),
                m.get_one::<String>("language").unwrap(),
                m.get_one::<String>("path").map(String::as_str),
            )
            .await
        }
        Some(("list", _)) => crate::extension::list().await,
        Some(("uninstall", m)) => {
            crate::extension::uninstall(m.get_one::<String>("id").unwrap(), m.get_flag("yes")).await
        }
        Some(("install", m)) => {
            crate::extension::install(
                m.get_one::<String>("url").unwrap(),
                m.get_one::<String>("capabilities").unwrap(),
            )
            .await
        }
        _ => Err("No subcommand found".into()),
    }
}

// ── Local playback / library maintenance ────────────────────────────────────

/// `open <song>` — play a single file through the local engine and wait for
/// it to finish.
async fn open(matches: &ArgMatches) -> CmdResult {
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cmd_tx = Arc::new(Mutex::new(cmd_tx));
    let cmd_rx = Arc::new(Mutex::new(cmd_rx));
    let tracklist = Arc::new(Mutex::new(Tracklist::new_empty()));
    let (mut player, _) = Player::new(|_| {}, cmd_tx, cmd_rx, tracklist);

    let song = matches.get_one::<String>("song").unwrap().as_str();
    player.load(song, true, 0);
    player.await_end_of_track().await;
    Ok(())
}

/// `scan` — refresh the library from the music directory.
async fn scan() -> CmdResult {
    migration::apply().await;
    let db = Database::new().await;
    scan_music_library(true, db.clone())
        .await
        .map_err(|e| e.to_string())?;

    // Awaited, not spawned: this command exits as soon as it returns, so a
    // background task would be killed before it decoded anything — which is
    // exactly why a scan used to leave every key and tempo null.
    //
    // It decodes each new track in full, so a first scan of a large library
    // takes a while. The tracks are already indexed and playable by this point;
    // what is still running only fills in two columns.
    let analysed = music_player_scanner::analyse_missing_key_and_bpm(&db).await;
    if analysed > 0 {
        println!(
            "{} track{} analysed for key and tempo",
            analysed.to_string().bright_green(),
            if analysed == 1 { "" } else { "s" }
        );
    }
    Ok(())
}

// ── Library listings ────────────────────────────────────────────────────────

/// `albums [-i <id>]` — list all albums, or one album's tracks.
async fn albums(matches: &ArgMatches, settings: &Settings) -> CmdResult {
    let mut client = LibraryClient::new(settings.host.clone(), settings.port).await?;

    if let Some(id) = matches.get_one::<String>("id") {
        let album = client.album(id).await?.ok_or("Album not found")?;
        let mut builder = Builder::default();
        builder.set_columns(["id", "title", "album"]);
        for track in &album.tracks {
            let title = format!(
                "{} {}",
                format_number(usize::try_from(track.track_number).unwrap()),
                track.title
            );
            builder.add_record([track.id.as_str(), title.as_str(), album.title.as_str()]);
        }
        print_table(builder);
        return Ok(());
    }

    let result = client.albums(None, 0, 10000).await?;
    let mut builder = Builder::default();
    builder.set_columns(["id", "name"]);
    for album in &result {
        builder.add_record([
            album.id.as_str(),
            album.title.magenta().to_string().as_str(),
        ]);
    }
    print_table(builder);
    Ok(())
}

/// `artists` — list all artists.
async fn artists(settings: &Settings) -> CmdResult {
    let mut client = LibraryClient::new(settings.host.clone(), settings.port).await?;
    let result = client.artists(None, 0, 10000).await?;

    let mut builder = Builder::default();
    builder.set_columns(["id", "name"]);
    for artist in &result {
        builder.add_record([
            artist.id.as_str(),
            artist.name.magenta().to_string().as_str(),
        ]);
    }
    print_table(builder);
    Ok(())
}

/// `tracks` — list all tracks.
async fn tracks(settings: &Settings) -> CmdResult {
    let mut client = LibraryClient::new(settings.host.clone(), settings.port).await?;
    let result = client.songs(None, 0, 10000).await?;

    let mut builder = Builder::default();
    builder.set_columns(["id", "title"]);
    for song in &result {
        builder.add_record([song.id.as_str(), song.title.magenta().to_string().as_str()]);
    }
    print_table(builder);
    Ok(())
}

/// `search <query>` — search artists, albums and tracks.
async fn search(matches: &ArgMatches, settings: &Settings) -> CmdResult {
    let mut client = LibraryClient::new(settings.host.clone(), settings.port).await?;
    let query = matches.get_one::<String>("query").unwrap().as_str();
    let result = client.search(query).await?;

    if !result.artists.is_empty() {
        let mut builder = Builder::default();
        builder.set_columns(["id", "name"]);
        for artist in &result.artists {
            builder.add_record([
                artist.id.as_str(),
                artist.name.magenta().to_string().as_str(),
            ]);
        }
        println!("\nArtists:");
        print_table_inline(builder);
    }
    if !result.albums.is_empty() {
        let mut builder = Builder::default();
        builder.set_columns(["id", "title", "artist"]);
        for album in &result.albums {
            builder.add_record([
                album.id.as_str(),
                album.title.magenta().to_string().as_str(),
                album.artist.as_str(),
            ]);
        }
        println!("\nAlbums:");
        print_table_inline(builder);
    }
    if !result.tracks.is_empty() {
        let mut builder = Builder::default();
        builder.set_columns(["id", "title", "artist"]);
        for track in &result.tracks {
            builder.add_record([
                track.id.as_str(),
                track.title.magenta().to_string().as_str(),
                track.artist.as_str(),
            ]);
        }
        println!("\nTracks:");
        print_table_inline(builder);
    }
    Ok(())
}

// ── Playlists / queue ───────────────────────────────────────────────────────

/// `playlist <ls|show|open>` — manage saved playlists.
async fn playlist(matches: &ArgMatches, settings: &Settings) -> CmdResult {
    let mut client = PlaylistClient::new(settings.host.clone(), settings.port).await?;

    match matches.subcommand() {
        Some(("ls", _)) => {
            let playlists = client.list_all().await?;
            let mut builder = Builder::default();
            builder.set_columns(["id", "name", "tracks"]);
            for playlist in &playlists {
                builder.add_record([
                    playlist.id.as_str(),
                    playlist.name.magenta().to_string().as_str(),
                    playlist.tracks.len().to_string().as_str(),
                ]);
            }
            print_table(builder);
        }
        Some(("show", m)) => {
            let id = m.get_one::<String>("id").unwrap().as_str();
            let playlist = client.find(id).await?;
            println!("\n{}", playlist.name.magenta());
            let mut builder = Builder::default();
            builder.set_columns(["id", "title", "artist"]);
            for track in &playlist.tracks {
                builder.add_record([
                    track.id.as_str(),
                    track.title.magenta().to_string().as_str(),
                    track.artist.as_str(),
                ]);
            }
            print_table_inline(builder);
        }
        Some(("open", m)) => {
            let id = m.get_one::<String>("id").unwrap().as_str();
            let playlist = client.find(id).await?;
            if playlist.tracks.is_empty() {
                println!("The playlist is empty");
                return Ok(());
            }
            let mut tracklist = TracklistClient::new(settings.host.clone(), settings.port).await?;
            tracklist.load_tracks(playlist.tracks, 0).await?;
        }
        _ => {}
    }
    Ok(())
}

/// `queue <list|add>` — inspect or extend the play queue.
async fn queue(matches: &ArgMatches, settings: &Settings) -> CmdResult {
    let mut client = TracklistClient::new(settings.host.clone(), settings.port).await?;

    match matches.subcommand() {
        Some(("list", _)) => {
            let (mut previous_tracks, next_tracks) = client.list().await?;
            if previous_tracks.is_empty() && next_tracks.is_empty() {
                println!("The queue is empty");
                return Ok(());
            }
            // The last "previous" track is the one currently playing.
            match previous_tracks.pop() {
                Some(current) => {
                    for (i, track) in previous_tracks.iter().enumerate() {
                        println!("{} {}", format_number(i + 1), track.title);
                    }
                    println!(
                        "{} {}",
                        format_number(previous_tracks.len() + 1).magenta(),
                        current.title.magenta()
                    );
                    for (i, track) in next_tracks.iter().enumerate() {
                        println!(
                            "{} {}",
                            format_number(i + previous_tracks.len() + 2),
                            track.title
                        );
                    }
                }
                None => {
                    for (i, track) in next_tracks.iter().enumerate() {
                        println!("{} {}", format_number(i + 1), track.title);
                    }
                }
            }
        }
        Some(("add", m)) => {
            let id = m.get_one::<String>("track_id").unwrap().as_str();
            client.add(id).await?;
        }
        _ => {}
    }
    Ok(())
}

// ── Transport ───────────────────────────────────────────────────────────────

/// One-shot transport commands (`play`, `pause`, `next`, `prev`, `stop`).
async fn transport(settings: &Settings, action: Transport) -> CmdResult {
    let mut client = PlaybackClient::new(settings.host.clone(), settings.port).await?;
    match action {
        Transport::Play => client.play().await?,
        Transport::Pause => client.pause().await?,
        Transport::Next => client.next().await?,
        Transport::Prev => client.prev().await?,
        Transport::Stop => client.stop().await?,
    }
    Ok(())
}

/// `current` — show the currently playing song.
async fn current(settings: &Settings) -> CmdResult {
    let mut client = PlaybackClient::new(settings.host.clone(), settings.port).await?;
    let (result, _, _, _) = client.current().await?;
    let Some(track) = result else {
        println!("No song is currently playing");
        return Ok(());
    };

    println!();
    println!("Title  : {}", track.title.magenta());
    println!(
        "Artist : {}",
        track
            .artists
            .iter()
            .map(|a| a.name.clone())
            .collect::<Vec<String>>()
            .join(", ")
            .magenta()
    );
    if let Some(album) = track.album {
        println!("Album  : {}", album.title.magenta());
    }
    Ok(())
}

// ── Environment / discovery ─────────────────────────────────────────────────

/// `connect -s <host> [-p <port>]` — point this process at a remote daemon.
/// Returns the "No subcommand found" sentinel so the caller continues into
/// client mode against the new target.
fn connect(matches: &ArgMatches) -> CmdResult {
    let host = matches.get_one::<String>("host").unwrap().as_str();
    let port = matches.get_one::<String>("port").unwrap().as_str();
    env::set_var("MUSIC_PLAYER_HOST", host);
    env::set_var("MUSIC_PLAYER_PORT", port);
    env::set_var("MUSIC_PLAYER_MODE", "client");
    Err("No subcommand found".into())
}

/// `devices` — list every music-player daemon advertised on the LAN.
async fn devices() -> CmdResult {
    let services = discover(SERVICE_NAME);
    tokio::pin!(services);
    while let Some(info) = services.next().await {
        println!(
            "{} - {} - {:?} - port: {}",
            info.get_fullname().bright_green(),
            info.get_hostname().to_lowercase(),
            info.get_addresses(),
            info.get_port()
        );
    }
    Ok(())
}

/// `reset` — wipe the database and the config directory.
fn reset() -> CmdResult {
    let path = format!(
        "{}/music-player",
        dirs::config_dir().unwrap().to_str().unwrap()
    );
    match fs::remove_dir_all(path.clone()) {
        Ok(_) => {
            println!("Reset complete ✅ ({})", path);
            println!("Please restart the application")
        }
        Err(e) => {
            println!("Reset failed ❌ ({})", e);
        }
    }
    Ok(())
}

// ── Output helpers ──────────────────────────────────────────────────────────

fn print_table(builder: Builder) {
    println!("\n{}", builder.build().with(Style::psql()));
}

fn print_table_inline(builder: Builder) {
    println!("{}", builder.build().with(Style::psql()));
}

fn format_number(number: usize) -> String {
    if number < 10 {
        return format!("0{}", number);
    }
    format!("{}", number)
}
