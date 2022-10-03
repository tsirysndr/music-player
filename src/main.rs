use std::{
    io::{self, stdout},
    sync::Arc,
    thread,
};

use app::{ActiveBlock, App, RouteId};
use args::parse_args;
use clap::{arg, Command};
use crossterm::{
    cursor::MoveTo,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
    },
    ExecutableCommand,
};
use event::Key;
use migration::SeaRc;
use music_player_playback::{
    audio_backend::{self, rodio::RodioSink},
    config::AudioFormat,
    player::Player,
};
use music_player_server::server::MusicPlayerServer;
use network::{IoEvent, Network};
use owo_colors::OwoColorize;
use scan::auto_scan_music_library;
use tokio::sync::Mutex;
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

mod app;
mod args;
mod event;
mod handlers;
mod network;
mod scan;
mod ui;
mod user_config;

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
            Command::new("open")
                .about("Play a song")
                .arg_from_usage("<song> 'The path to the song'"),
        )
        .subcommand(Command::new("scan").about("Scan music library: $HOME/Music"))
        .subcommand(Command::new("albums").arg(
            arg!(-i --id <id> "Show the album with the given id").required(false)
        ).about("List all albums"))
        .subcommand(Command::new("artists").about("List all artists"))
        .subcommand(
            Command::new("playlist")
                .subcommand(
                    Command::new("add")
                        .about("Add a song to the playlist")
                        .arg_from_usage("<id> 'The track id'"),
                )
                .subcommand(Command::new("ls").about("List all playlists"))
                .subcommand(Command::new("clear").about("Clear the playlist").arg_from_usage(
                    "[id] 'The playlist id, if not specified, the current playlist will be cleared'",
                ))
                .subcommand(
                    Command::new("open")
                        .about("Play the playlist")
                        .arg_from_usage("[id] 'The playlist id'"),
                )
                .subcommand(
                    Command::new("remove")
                        .about("Remove a song from the playlist")
                        .arg_from_usage("<id> 'The track id'"),
                )
                .subcommand(Command::new("shuffle").about("Shuffle the playlist"))
                .subcommand(Command::new("all").about("List all songs in the playlist"))
                .subcommand(
                    Command::new("show")
                        .about("Show the playlist details")
                        .arg_from_usage("<id> 'The track id'")
                )
                .about("Manage playlists")
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("queue")
                .subcommand(
                    Command::new("list")
                        .about("List all songs in the queue")
                        .arg_from_usage("-a, --all 'List all songs in the queue'"),
                )
                .subcommand(
                    Command::new("add")
                        .about("Add a song to the queue")
                        .arg_from_usage("<track_id> 'The track id'"),
                )
                .subcommand(
                    Command::new("remove")
                        .about("Remove a song from the queue")
                        .arg_from_usage("<song> 'The path to the song'"),
                )
                .subcommand(
                    Command::new("clear")
                        .about("Clear the queue")
                        .arg_from_usage("-a, --all 'Clear the queue'"),
                )
                .about("Manage the queue")
                .arg_required_else_help(true),
        )
        .subcommand(Command::new("tracks").about("List all tracks"))
        .subcommand(
            Command::new("search")
                .about("Search for a song, album, artist or playlist")
                .arg_from_usage("<query> 'The query to search for'"),
        )
        .subcommand(Command::new("pause").about("Pause the current song"))
        .subcommand(Command::new("play").about("Resume the current song"))
        .subcommand(Command::new("next").about("Play the next song"))
        .subcommand(Command::new("prev").about("Play the previous song"))
        .subcommand(Command::new("stop").about("Stop the current song"))
        .subcommand(Command::new("current").about("Show the current song"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = cli().get_matches();

    let audio_format = AudioFormat::default();
    let backend = audio_backend::find(Some(RodioSink::NAME.to_string())).unwrap();

    let (player, _) = Player::new(move || backend(None, audio_format));

    let parsed = parse_args(matches.clone()).await;

    if parsed.is_ok() {
        return Ok(());
    }

    let err = parsed.err().unwrap().to_string();
    if !err.eq("No subcommand found") {
        if err.eq("transport error") {
            println!(
                "The server is not running, please run {}",
                "`music-player`".bright_green()
            );
        }
        return Err(err.into());
    }

    migration::run().await;

    thread::spawn(|| {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(auto_scan_music_library());
    });

    let start_server = MusicPlayerServer::new(Arc::new(Mutex::new(player)))
        .start()
        .await;
    if start_server.is_err() {
        let (sync_io_tx, sync_io_rx) = std::sync::mpsc::channel::<IoEvent>();
        let app = Arc::new(Mutex::new(App::new(sync_io_tx)));
        return start_ui(&app).await;
    }
    start_server
}

#[tokio::main]
async fn start_tokio<'a>(io_rx: std::sync::mpsc::Receiver<IoEvent>, network: &mut Network) {
    while let Ok(io_event) = io_rx.recv() {
        network.handle_network_event(io_event).await;
    }
}

async fn start_ui(app: &Arc<Mutex<App>>) -> Result<(), Box<dyn std::error::Error>> {
    // Terminal initialization
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;

    let mut backend = CrosstermBackend::new(stdout);

    backend.execute(SetTitle("mpt - Music Player TUI"))?;

    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = event::Events::new(800);

    loop {
        let mut app = app.lock().await;

        if let Ok(size) = terminal.backend().size() {
            app.size = size;
        }

        let current_route = app.get_current_route();
        terminal.draw(|mut f| ui::draw_main_layout(&mut f, &app))?;

        if current_route.active_block == ActiveBlock::Input {
            terminal.show_cursor()?;
        } else {
            terminal.hide_cursor()?;
        }

        let cursor_offset = if app.size.height > ui::util::SMALL_TERMINAL_HEIGHT {
            2
        } else {
            1
        };

        // Put the cursor back inside the input box
        terminal.backend_mut().execute(MoveTo(
            cursor_offset + app.input_cursor_position,
            cursor_offset,
        ))?;

        match events.next()? {
            event::Event::Input(key) => {
                if key == Key::Ctrl('c') {
                    break;
                }
                if key == app.user_config.keys.back {
                    let pop_result = match app.pop_navigation_stack() {
                        Some(ref x) if x.id == RouteId::Search => app.pop_navigation_stack(),
                        Some(x) => Some(x),
                        None => None,
                    };
                    if pop_result.is_none() {
                        break; // Exit application
                    }
                } else {
                    handlers::handle_app(key, &mut app);
                }
            }
            event::Event::Tick => {}
        }
    }

    close_application()?;
    Ok(())
}

fn close_application() -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}
