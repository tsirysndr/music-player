use std::{
    collections::HashMap,
    env,
    io::{self, stdout},
    net::SocketAddr,
    sync::{self, mpsc, Arc},
    thread,
    time::Instant,
};

use app::{ActiveBlock, App, CurrentlyPlaybackContext, RouteId};
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
use futures::StreamExt;
use futures_channel::mpsc::UnboundedSender;
use music_player_client::{library::LibraryClient, ws_client::WebsocketClient};
use music_player_discovery::register_services;
use music_player_playback::{
    audio_backend::{self, rodio::RodioSink},
    config::AudioFormat,
    player::{Player, PlayerEvent},
};
use music_player_server::{
    event::{Event, TrackEvent},
    metadata::v1alpha1::Track,
};
use music_player_server::{
    metadata::v1alpha1::{Album, Artist},
    server::MusicPlayerServer,
};
use music_player_storage::Database;
use music_player_tracklist::Tracklist;
use music_player_webui::start_webui;
use network::{IoEvent, Network};
use owo_colors::OwoColorize;
use scan::auto_scan_music_library;
use tokio::sync::Mutex;
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use tungstenite::Message;

mod app;
mod args;
mod event;
mod handlers;
mod network;
mod scan;
mod ui;
mod user_config;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<sync::Mutex<HashMap<SocketAddr, Tx>>>;

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
                .about("open audio file")
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
        .subcommand(Command::new("connect").arg(
            arg!(-s --host <host> "The host to connect to").required(true)
        ).arg(
            arg!(-p --port <port> "The port to connect to").default_value("50051").required(false)
        ).about("Connect to the server"))
        .subcommand(Command::new("devices").about("List all `music-player` devices on the network"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = cli().get_matches();

    let audio_format = AudioFormat::default();
    let backend = audio_backend::find(Some(RodioSink::NAME.to_string())).unwrap();
    let peer_map: PeerMap = Arc::new(sync::Mutex::new(HashMap::new()));
    let cloned_peer_map = Arc::clone(&peer_map);

    let tracklist = Arc::new(std::sync::Mutex::new(Tracklist::new_empty()));
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cloned_tracklist = Arc::clone(&tracklist);
    let cmd_tx_ws = cmd_tx.clone();
    let cmd_tx_webui = cmd_tx.clone();
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        move |event| {
            let peers = cloned_peer_map.lock().unwrap();

            let broadcast_recipients = peers.iter().map(|(_, ws_sink)| ws_sink);

            match event {
                PlayerEvent::CurrentTrack {
                    track,
                    position,
                    position_ms,
                    is_playing,
                } => {
                    let track_event = TrackEvent {
                        track,
                        index: position as u32,
                        is_playing,
                        position_ms,
                    };
                    let msg = Event {
                        event_type: "current_track".to_string(),
                        data: serde_json::to_string(&track_event).unwrap(),
                    };
                    for recp in broadcast_recipients {
                        recp.unbounded_send(Message::text(serde_json::to_string(&msg).unwrap()))
                            .unwrap();
                    }
                }
                PlayerEvent::TrackTimePosition { position_ms } => {
                    let msg = Event {
                        event_type: "track_time_position".to_string(),
                        data: format!("{}", position_ms),
                    };
                    for recp in broadcast_recipients {
                        recp.unbounded_send(Message::text(serde_json::to_string(&msg).unwrap()))
                            .unwrap();
                    }
                }
                _ => {}
            }
        },
        cmd_tx.clone(),
        cmd_rx,
        Arc::clone(&tracklist),
    );

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

    let mut mode = match connect_to_server().await {
        true => "client".to_string(),
        false => "server".to_string(),
    };

    mode = env::var("MUSIC_PLAYER_MODE").unwrap_or(mode);

    if mode == "server" {
        migration::run().await;

        thread::spawn(|| {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(auto_scan_music_library());
        });
    }

    let tracklist_ws = Arc::clone(&tracklist);
    let tracklist_webui = Arc::clone(&tracklist);
    let db = Arc::new(Mutex::new(Database::new().await));
    let db_ws = Arc::clone(&db);

    let peer_map_ws = Arc::clone(&peer_map);

    if mode == "server" {
        register_services();

        thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            match runtime.block_on(
                MusicPlayerServer::new(cloned_tracklist, cmd_tx, Arc::clone(&peer_map), db).start(),
            ) {
                Ok(_) => {}
                Err(e) => {
                    panic!("{}", e);
                }
            }
        });
        // Spawn a thread to handle the player events
        thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            match runtime.block_on(
                MusicPlayerServer::new(tracklist_ws, cmd_tx_ws, peer_map_ws, db_ws).start_ws(),
            ) {
                Ok(_) => {}
                Err(e) => {
                    println!("{}", e);
                }
            }
        });
        start_webui(cmd_tx_webui, tracklist_webui).await?;
        return Ok(());
    }

    if mode == "client" {
        let (sync_io_tx, sync_io_rx) = std::sync::mpsc::channel::<IoEvent>();
        let app = Arc::new(Mutex::new(App::new(sync_io_tx)));
        let cloned_app = Arc::clone(&app);
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            match runtime.block_on(Network::new(&app)) {
                Ok(mut network) => start_tokio(sync_io_rx, &mut network),
                Err(err) => println!("{}", err),
            }
        });
        return start_ui(&cloned_app).await;
    }
    Ok(())
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

    let events = event::Events::new(250);

    let mut is_first_render = true;

    listen_for_player_events(app).await;

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
            event::Event::Tick => {
                app.update_on_tick();
            }
        }

        if is_first_render {
            app.dispatch(IoEvent::GetTracks);
            app.dispatch(IoEvent::GetCurrentPlayback);
            is_first_render = false;
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

async fn listen_for_player_events(app: &Arc<Mutex<App>>) {
    let ws_client = WebsocketClient::new().await;
    let (tx, rx) = mpsc::channel::<Event>();

    thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(ws_client.read.for_each(|message| async {
            match message {
                Ok(msg) => match serde_json::from_str(&msg.to_string()) {
                    Ok(event) => {
                        tx.send(event).unwrap();
                    }
                    Err(e) => {
                        println!("{}", e);
                    }
                },
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }));
    });

    {
        let app = app.clone();
        thread::spawn(move || loop {
            let ev = rx.recv();
            if ev.is_ok() {
                let event = ev.unwrap();
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                match event.event_type.as_str() {
                    "current_track" => {
                        let mut app = runtime.block_on(app.lock());
                        let track_event: TrackEvent = serde_json::from_str(&event.data).unwrap();
                        let track = track_event.track.unwrap();
                        app.instant_since_last_current_playback_poll = Instant::now();
                        app.current_playback_context = Some(CurrentlyPlaybackContext {
                            track: Some(Track {
                                id: track.id,
                                title: track.title,
                                uri: track.uri,
                                disc_number: i32::try_from(track.track.unwrap_or(0)).unwrap(),
                                artists: vec![Artist {
                                    name: track.artist,
                                    ..Default::default()
                                }],
                                album: Some(Album {
                                    // id: track.album_id.unwrap(),
                                    title: track.album,
                                    year: i32::try_from(track.year.unwrap_or(0)).unwrap(),
                                    ..Default::default()
                                }),
                                duration: track.duration.unwrap_or(0.0),
                                ..Default::default()
                            }),
                            is_playing: track_event.is_playing,
                            index: track_event.index,
                            ..Default::default()
                        });
                    }
                    _ => {}
                }
            }
        });
    }
}

async fn connect_to_server() -> bool {
    match LibraryClient::new().await {
        Ok(_) => true,
        Err(_) => false,
    }
}
