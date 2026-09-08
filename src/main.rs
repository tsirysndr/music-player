use std::{
    collections::HashMap,
    env,
    io::{self, stdout},
    net::SocketAddr,
    sync::{self, mpsc, Arc},
    thread,
    time::{Duration, Instant},
};

use app::{App, CurrentlyPlaybackContext};
use args::parse_args;
use clap::{arg, Arg, Command};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
    },
};
use event::Key;
use futures::StreamExt;
use futures_channel::mpsc::UnboundedSender;
use music_player_client::{library::LibraryClient, ws_client::WebsocketClient};
use music_player_discovery::register_services;
use music_player_graphql::{
    schema::{
        objects::{player_state::PlayerState, track::Track},
        playback::PositionMilliseconds,
    },
    simple_broker::SimpleBroker,
};
use music_player_playback::player::{Player, PlayerCommand, PlayerEvent};
use music_player_server::event::{Event, TrackEvent};
use music_player_server::server::MusicPlayerServer;
use music_player_settings::{read_settings, Settings};
use music_player_storage::Database;
use music_player_tracklist::Tracklist;
use music_player_webui::start_webui;
use network::{IoEvent, Network};
use owo_colors::OwoColorize;
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};
use scan::auto_scan_music_library;
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use tokio::sync::Mutex;
use tungstenite::Message;

mod app;
mod args;
mod event;
mod extension;
mod handlers;
mod network;
mod scan;
mod smart_playlist_form;
mod ui;
mod user_config;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<sync::Mutex<HashMap<SocketAddr, Tx>>>;

/// Help colors, matching pocketenv's CLI palette: aqua headers, sky-blue
/// literals, violet placeholders, soft red errors.
fn cli_styles() -> clap::builder::Styles {
    use clap::builder::styling::{RgbColor, Style};
    let primary = Style::new()
        .bold()
        .fg_color(Some(RgbColor(0, 232, 198).into()));
    let secondary = Style::new().fg_color(Some(RgbColor(0, 198, 232).into()));
    let accent = Style::new().fg_color(Some(RgbColor(130, 100, 255).into()));
    let highlight = Style::new().fg_color(Some(RgbColor(100, 232, 130).into()));
    let error = Style::new()
        .bold()
        .fg_color(Some(RgbColor(255, 100, 100).into()));
    clap::builder::Styles::styled()
        .header(primary)
        .usage(primary)
        .literal(secondary)
        .placeholder(accent)
        .valid(highlight)
        .invalid(error)
        .error(error)
}

fn cli() -> Command {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    Command::new("music-player")
        .version(VERSION)
        .author("Tsiry Sandratraina <tsiry.sndr@rocksky.app>")
        .styles(cli_styles())
        .about(
            r#"
     __  ___           _      ____  __
    /  |/  /_  _______(_)____/ __ \/ /___ ___  _____  _____
   / /|_/ / / / / ___/ / ___/ /_/ / / __ `/ / / / _ \/ ___/
  / /  / / /_/ (__  ) / /__/ ____/ / /_/ / /_/ /  __/ /
 /_/  /_/\__,_/____/_/\___/_/   /_/\__,_/\__, /\___/_/
                                        /____/

A simple music player written in Rust — single binary, zero dependency"#,
        )
        .subcommand(
            Command::new("open")
                .about("open audio file")
                .arg(Arg::new("song").help("The path to the song").required(true)),
        )
        .subcommand(Command::new("scan").about("Scan music library: $HOME/Music"))
        .subcommand(
            Command::new("albums")
                .arg(arg!(-i --id <id> "Show the album with the given id").required(false))
                .about("List all albums"),
        )
        .subcommand(Command::new("artists").about("List all artists"))
        .subcommand(
            Command::new("playlist")
                .subcommand(Command::new("ls").about("List all playlists"))
                .subcommand(
                    Command::new("open")
                        .about("Play the playlist")
                        .arg(Arg::new("id").help("The playlist id").required(true)),
                )
                .subcommand(
                    Command::new("show")
                        .about("Show the playlist details")
                        .arg(Arg::new("id").help("The playlist id").required(true)),
                )
                .about("Manage playlists")
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("queue")
                .subcommand(
                    Command::new("list")
                        .about("List all songs in the queue")
                        .arg(
                            Arg::new("all")
                                .short('a')
                                .long("all")
                                .help("List all songs in the queue")
                                .action(clap::ArgAction::SetTrue),
                        ),
                )
                .subcommand(
                    Command::new("add")
                        .about("Add a song to the queue")
                        .arg(Arg::new("track_id").help("The track id").required(true)),
                )
                .about("Manage the queue")
                .arg_required_else_help(true),
        )
        .subcommand(Command::new("tracks").about("List all tracks"))
        .subcommand(
            Command::new("search")
                .about("Search for a song, album, artist or playlist")
                .arg(
                    Arg::new("query")
                        .help("The query to search for")
                        .required(true),
                ),
        )
        .subcommand(Command::new("pause").about("Pause the current song"))
        .subcommand(Command::new("play").about("Resume the current song"))
        .subcommand(Command::new("next").about("Play the next song"))
        .subcommand(Command::new("prev").about("Play the previous song"))
        .subcommand(Command::new("stop").about("Stop the current song"))
        .subcommand(Command::new("current").about("Show the current song"))
        .subcommand(
            Command::new("connect")
                .arg(arg!(-s --host <host> "The host to connect to").required(true))
                .arg(
                    arg!(-p --port <port> "The port to connect to")
                        .default_value("50051")
                        .required(false),
                )
                .about("Connect to the server"),
        )
        .subcommand(Command::new("devices").about("List all `music-player` devices on the network"))
        .subcommand(
            Command::new("reset").about("Reset the database and clear the config directory"),
        )
        .subcommand(
            Command::new("extension")
                .alias("ext")
                .about("Create, list and install WebAssembly extensions")
                .subcommand(
                    Command::new("init")
                        .alias("new")
                        .about("Scaffold a new extension")
                        .arg(arg!(<id> "Extension id, e.g. com.example.lyrics"))
                        .arg(
                            Arg::new("name")
                                .long("name")
                                .short('n')
                                .help("Display name (defaults to the last part of the id)"),
                        )
                        .arg(
                            Arg::new("capabilities")
                                .long("capabilities")
                                .short('c')
                                .default_value("events")
                                .help(
                                    "Comma-separated: events, metadata, commands, \
                                     predicates, source",
                                ),
                        )
                        .arg(
                            Arg::new("language")
                                .long("language")
                                .short('l')
                                .default_value("rust")
                                .help("rust, go, typescript, python, csharp, zig or cpp"),
                        )
                        .arg(
                            Arg::new("path")
                                .long("path")
                                .short('p')
                                .help("Where to write it (defaults to the name)"),
                        ),
                )
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("list")
                        .alias("ls")
                        .about("List installed extensions"),
                )
                .subcommand(
                    Command::new("uninstall")
                        .aliases(["remove", "rm"])
                        .about("Remove an installed extension")
                        .arg(arg!(<id> "Extension id, as shown by `extension list`"))
                        .arg(
                            Arg::new("yes")
                                .long("yes")
                                .short('y')
                                .help("Do not ask for confirmation")
                                .action(clap::ArgAction::SetTrue),
                        ),
                )
                .subcommand(
                    Command::new("install")
                        .alias("add")
                        .about("Install an extension from a url")
                        .arg(arg!(<url> "Url of a .wasm, or of a plugin.toml/plugin.json"))
                        .arg(
                            Arg::new("capabilities")
                                .long("capabilities")
                                .short('c')
                                .default_value("")
                                .help("Required for a bare .wasm, which carries no manifest"),
                        ),
                ),
        )
        .arg(
            Arg::new("force-car-sync")
                .long("force-car-sync")
                .help(
                    "Re-download the atproto repo archive on start, even if the last \
                     download is still recent",
                )
                .action(clap::ArgAction::SetTrue),
        )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A dependency may have installed a global dispatcher already; ours is
    // best-effort. sqlx logs every statement at INFO, hence the directive.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .try_init();
    let matches = cli().get_matches();
    if matches.get_flag("force-car-sync") {
        music_player_storage::repo_sync::force_download();
    }

    let parsed = parse_args(matches.clone()).await;

    if parsed.is_ok() {
        return Ok(());
    }
    let peer_map: PeerMap = Arc::new(sync::Mutex::new(HashMap::new()));
    let cloned_peer_map = Arc::clone(&peer_map);

    let mut mode = match connect_to_server().await {
        true => "client".to_string(),
        false => "server".to_string(),
    };

    mode = env::var("MUSIC_PLAYER_MODE").unwrap_or(mode);
    if mode == "server" {
        migration::apply().await;
        let db = Database::new().await;
        let conn = db.get_connection();
        conn.execute(Statement::from_string(
            DbBackend::Sqlite,
            "PRAGMA case_sensitive_like=OFF;".to_owned(),
        ))
        .await?;
        db.create_indexes().await;
    }

    let db = Database::new().await;
    let tracklist = Arc::new(std::sync::Mutex::new(Tracklist::new_empty()));
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cloned_tracklist = Arc::clone(&tracklist);
    let cmd_tx = Arc::new(std::sync::Mutex::new(cmd_tx));
    let cmd_rx = Arc::new(std::sync::Mutex::new(cmd_rx));
    let cloned_cmd_tx = Arc::clone(&cmd_tx);
    let cloned_cmd_rx = Arc::clone(&cmd_rx);
    let cmd_tx_ws = Arc::clone(&cloned_cmd_tx);
    let cmd_tx_webui = Arc::clone(&cloned_cmd_tx);
    let (_, _) = Player::new(
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
                    if let Some(track) = track.clone() {
                        SimpleBroker::publish(Track::from(track));
                        SimpleBroker::publish(PlayerState {
                            index: position as u32,
                            position_ms,
                            is_playing,
                        });
                    }

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
                    SimpleBroker::publish(PositionMilliseconds { position_ms });
                    /*
                    let msg = Event {
                            event_type: "track_time_position".to_string(),
                            data: format!("{}", position_ms),
                        };
                        for recp in broadcast_recipients {
                            recp.unbounded_send(Message::text(serde_json::to_string(&msg).unwrap()))
                                .unwrap();
                        }
                        */
                }
                _ => {}
            }
        },
        cloned_cmd_tx,
        cloned_cmd_rx,
        Arc::clone(&tracklist),
    );

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

    if mode == "server" {
        thread::spawn(|| {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            let db = runtime.block_on(Database::new());
            runtime.block_on(auto_scan_music_library(db));
            runtime.block_on(scan::periodic_scan_music_library());
        });
    }

    let tracklist_ws = Arc::clone(&tracklist);
    let tracklist_webui = Arc::clone(&tracklist);

    let peer_map_ws = Arc::clone(&peer_map);

    // One provider registry, one piece of state, shared by the gRPC server and
    // the web/GraphQL server. Both API surfaces route library reads through it,
    // so they cannot disagree about which server the screens are showing.
    let providers = {
        let mut registry = music_player_provider::ProviderRegistry::new();
        music_player_provider::register_builtin(&mut registry);
        Arc::new(music_player_provider::ProviderState::new(Arc::new(
            registry,
        )))
    };
    // Refuse to use ourselves as a provider: with gRPC reads routed through
    // one, that would recurse until something ran out.
    if let Ok(config) = read_settings() {
        if let Ok(settings) = config.try_deserialize::<Settings>() {
            let port = settings.http_port as u16;
            for host in ["127.0.0.1", "localhost", "::1"] {
                providers.add_own_address(host, port).await;
            }
        }
    }
    let providers_webui = Arc::clone(&providers);

    if mode == "server" {
        register_services();
        music_player_server::scrobbler::spawn(Arc::clone(&tracklist));
        // Local play counts: unlike scrobbling these need no account, and the
        // smart playlists filter on them.
        music_player_server::play_stats::spawn(Arc::clone(&tracklist));
        music_player_server::atproto_sync::spawn(Arc::clone(&tracklist));
        music_player_server::remote::spawn(Arc::clone(&tracklist), Arc::clone(&cmd_tx));
        // MPRIS media controls (Linux only; no-op elsewhere).
        music_player_server::media_controls::spawn(Arc::clone(&tracklist), Arc::clone(&cmd_tx));
        // Arm queue persistence + restore the last session's queue (cued
        // paused). Daemon mode only — `open`/tests must not touch it.
        let _ = cmd_tx.lock().unwrap().send(PlayerCommand::RestoreQueue);

        let grpc_db = db.clone();
        let grpc_providers = Arc::clone(&providers);
        let ws_providers = Arc::clone(&providers);
        thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            match runtime.block_on(
                MusicPlayerServer::new(
                    cloned_tracklist,
                    Arc::clone(&cmd_tx),
                    Arc::clone(&peer_map),
                    grpc_db,
                    grpc_providers,
                )
                .start(),
            ) {
                Ok(_) => {}
                Err(e) => {
                    panic!("{}", e);
                }
            }
        });
        // Spawn a thread to handle the player events
        let ws_db = db.clone();
        thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            match runtime.block_on(
                MusicPlayerServer::new(tracklist_ws, cmd_tx_ws, peer_map_ws, ws_db, ws_providers)
                    .start_ws(),
            ) {
                Ok(_) => {}
                Err(e) => {
                    println!("{}", e);
                }
            }
        });
        start_webui(cmd_tx_webui, tracklist_webui, providers_webui).await?;
        return Ok(());
    }

    if mode == "client" {
        let (sync_io_tx, sync_io_rx) = std::sync::mpsc::channel::<IoEvent>();
        let mut app_state = App::new(sync_io_tx);
        if let Ok(config) = read_settings() {
            if let Ok(settings) = config.try_deserialize::<Settings>() {
                app_state.server_addr = format!("{}:{}", settings.host, settings.port);
            }
        }
        let app = Arc::new(Mutex::new(app_state));
        let cloned_app = Arc::clone(&app);
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            match runtime.block_on(Network::new(&app)) {
                Ok(mut network) => start_tokio(sync_io_rx, &mut network),
                // Printing would corrupt the TUI this thread runs beside.
                Err(err) => tracing::error!("network worker failed: {err}"),
            }
        });
        return start_ui(&cloned_app).await;
    }
    Ok(())
}

#[tokio::main]
async fn start_tokio<'a>(io_rx: std::sync::mpsc::Receiver<IoEvent>, network: &mut Network) {
    while let Ok(io_event) = io_rx.recv() {
        // Network errors (e.g. the server going away mid-session) must not
        // crash the UI thread's worker; they are simply dropped.
        let _ = network.handle_network_event(io_event).await;
    }
}

async fn start_ui(app: &Arc<Mutex<App>>) -> Result<(), Box<dyn std::error::Error>> {
    // Terminal initialization
    let mut stdout = stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        SetTitle("mpt - Music Player TUI")
    )?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = event::Events::new(250);

    let mut is_first_render = true;

    listen_for_player_events(app).await;

    loop {
        let mut app = app.lock().await;

        if let Ok(size) = terminal.size() {
            app.size = Rect::new(0, 0, size.width, size.height);
        }

        terminal.draw(|f| ui::draw_main_layout(f, &app))?;

        match events.next()? {
            event::Event::Input(key) => {
                if key == Key::Ctrl('c') {
                    break;
                }
                if handlers::handle_app(key, &mut app) {
                    break; // Exit application
                }
            }
            event::Event::Tick => {
                app.update_on_tick();
            }
        }

        if is_first_render {
            app.dispatch(IoEvent::GetTracks);
            app.dispatch(IoEvent::GetCurrentPlayback);
            app.dispatch(IoEvent::GetVolume);
            app.dispatch(IoEvent::GetPlaylists);
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
                    // A closed receiver just means the TUI is gone.
                    Ok(event) => {
                        let _ = tx.send(event);
                    }
                    Err(e) => {
                        tracing::debug!("ignoring malformed player event: {e}");
                    }
                },
                // Expected on exit: the socket's reactor lives on the main
                // runtime, which shuts down first — printing here would land
                // on the restored terminal after the TUI closes.
                Err(e) => {
                    tracing::debug!("player event stream ended: {e}");
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
                            track: Some(track.into()),
                            is_playing: track_event.is_playing,
                            index: track_event.index,
                            position_ms: track_event.position_ms,
                            ..Default::default()
                        });
                    }
                    _ => {}
                }
            }
            thread::sleep(Duration::from_millis(10));
        });
    }
}

async fn connect_to_server() -> bool {
    let config = read_settings().unwrap();
    let settings = config.try_deserialize::<Settings>().unwrap();
    match LibraryClient::new(settings.host, settings.port).await {
        Ok(_) => true,
        Err(_) => false,
    }
}
