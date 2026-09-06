//! Embedded music-player daemon boot — mirrors the server mode of the root
//! `music-player` binary (src/main.rs).
//!
//! When nothing is listening on the configured gRPC port, the whole daemon
//! (player engine, gRPC + websocket servers, webui with the /covers route,
//! library scanner, scrobbler, mDNS discovery) is booted in-process. When a
//! daemon is already running the app just connects to it.

use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream};
use std::sync::{self, Arc};
use std::time::{Duration, Instant};

use futures_channel::mpsc::UnboundedSender as WsSender;
use music_player_graphql::schema::{
    objects::{player_state::PlayerState, track::Track as GraphqlTrack},
    playback::PositionMilliseconds,
};
use music_player_graphql::simple_broker::SimpleBroker;
use music_player_playback::player::{Player, PlayerEvent};
use music_player_server::event::{Event, TrackEvent};
use music_player_server::server::MusicPlayerServer;
use music_player_settings::{read_settings, Settings};
use music_player_storage::Database;
use music_player_tracklist::Tracklist;
use music_player_webui::start_webui;
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use tungstenite::Message;

type PeerMap = Arc<sync::Mutex<HashMap<SocketAddr, WsSender<Message>>>>;

const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);

pub fn settings() -> Settings {
    read_settings()
        .expect("failed to read settings")
        .try_deserialize::<Settings>()
        .expect("failed to parse settings")
}

fn remote_host() -> Option<String> {
    std::env::var("MUSIC_PLAYER_HOST")
        .ok()
        .filter(|h| h != "127.0.0.1" && h != "localhost")
}

/// Returns true if the gRPC port is already accepting connections locally.
pub fn is_running() -> bool {
    let port = settings().port;
    let addr: SocketAddr = (std::net::Ipv4Addr::LOCALHOST, port).into();
    TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT).is_ok()
}

/// Ensure a daemon is reachable. Boots the embedded daemon when no local one
/// is running; never returns in that case (it parks on the webui server), so
/// call it from a dedicated thread.
pub fn ensure_running() {
    // Respect an explicit remote host — never boot a local daemon then.
    if remote_host().is_some() {
        return;
    }
    if is_running() {
        wait_for_webui();
        return;
    }
    tracing::info!("no local music-player daemon — booting embedded daemon");
    boot();
}

/// Full daemon bootstrap, same order as the root binary's server mode:
/// migrations → db indexes → player engine → scanner → discovery/scrobbler →
/// gRPC server → websocket server → webui (blocks forever).
fn boot() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    let db = runtime.block_on(async {
        migration::apply().await;
        let db = Database::new().await;
        db.get_connection()
            .execute(Statement::from_string(
                DbBackend::Sqlite,
                "PRAGMA case_sensitive_like=OFF;".to_owned(),
            ))
            .await
            .expect("PRAGMA case_sensitive_like");
        db.create_indexes().await;
        db
    });

    let peer_map: PeerMap = Arc::new(sync::Mutex::new(HashMap::new()));
    let tracklist = Arc::new(sync::Mutex::new(Tracklist::new_empty()));
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cmd_tx = Arc::new(sync::Mutex::new(cmd_tx));
    let cmd_rx = Arc::new(sync::Mutex::new(cmd_rx));

    // The engine handle is not Send — Player::new parks it on its own
    // dedicated current-thread runtime internally.
    let broadcast_peers = Arc::clone(&peer_map);
    let (_player, _) = Player::new(
        move |event| match event {
            PlayerEvent::CurrentTrack {
                track,
                position,
                position_ms,
                is_playing,
            } => {
                if let Some(track) = track.clone() {
                    SimpleBroker::publish(GraphqlTrack::from(track));
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
                let peers = broadcast_peers.lock().unwrap();
                for (_, recp) in peers.iter() {
                    let _ =
                        recp.unbounded_send(Message::text(serde_json::to_string(&msg).unwrap()));
                }
            }
            PlayerEvent::TrackTimePosition { position_ms } => {
                SimpleBroker::publish(PositionMilliseconds { position_ms });
            }
            _ => {}
        },
        Arc::clone(&cmd_tx),
        cmd_rx,
        Arc::clone(&tracklist),
    );

    // Library scan: initial (only when the library is empty) + periodic.
    std::thread::spawn(|| {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let db = runtime.block_on(Database::new());
        runtime.block_on(auto_scan_music_library(db));
        runtime.block_on(periodic_scan_music_library());
    });

    music_player_discovery::register_services();
    runtime.block_on(async {
        music_player_server::scrobbler::spawn(Arc::clone(&tracklist));
        music_player_server::remote::spawn(Arc::clone(&tracklist), Arc::clone(&cmd_tx));
    });
    // MPRIS media controls (Linux only; the app handles macOS Now Playing).
    music_player_server::media_controls::spawn(Arc::clone(&tracklist), Arc::clone(&cmd_tx));
    // Arm queue persistence + restore the last session's queue (cued paused).
    let _ = cmd_tx
        .lock()
        .unwrap()
        .send(music_player_playback::player::PlayerCommand::RestoreQueue);

    // gRPC server
    {
        let tracklist = Arc::clone(&tracklist);
        let cmd_tx = Arc::clone(&cmd_tx);
        let peer_map = Arc::clone(&peer_map);
        let db = db.clone();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            if let Err(e) =
                runtime.block_on(MusicPlayerServer::new(tracklist, cmd_tx, peer_map, db).start())
            {
                tracing::error!("gRPC server failed: {e}");
            }
        });
    }

    // Websocket server (player events for other clients, e.g. the TUI)
    {
        let tracklist = Arc::clone(&tracklist);
        let cmd_tx = Arc::clone(&cmd_tx);
        let peer_map = Arc::clone(&peer_map);
        let db = db.clone();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            if let Err(e) =
                runtime.block_on(MusicPlayerServer::new(tracklist, cmd_tx, peer_map, db).start_ws())
            {
                tracing::error!("websocket server failed: {e}");
            }
        });
    }

    // Webui (GraphQL + /covers/ album art) — parks this thread forever.
    if let Err(e) = runtime.block_on(start_webui(cmd_tx, tracklist)) {
        tracing::error!("webui failed: {e}");
    }
}

async fn auto_scan_music_library(db: Database) {
    use music_player_entity::track;
    use sea_orm::EntityTrait;
    match track::Entity::find().all(db.get_connection()).await {
        Ok(result) => {
            if result.is_empty() {
                if let Err(e) = music_player_scanner::refresh_music_library(false, db).await {
                    tracing::error!("initial library scan failed: {e}");
                }
            }
        }
        Err(e) => tracing::error!("library check failed: {e}"),
    }
}

/// Periodically rescan the music directory (settings.library_refresh_interval
/// minutes; 0 disables) — same behaviour as the root binary.
async fn periodic_scan_music_library() {
    let interval_minutes = settings().library_refresh_interval;
    if interval_minutes == 0 {
        return;
    }
    let interval = Duration::from_secs(interval_minutes * 60);
    loop {
        tokio::time::sleep(interval).await;
        let db = Database::new().await;
        if let Err(e) = music_player_scanner::refresh_music_library(false, db).await {
            tracing::error!("periodic library scan failed: {e}");
        }
    }
}

/// Wait up to 10 s for the webui (album art, /covers/) to bind.
/// Not fatal — art is just missing until it comes up.
fn wait_for_webui() {
    let http_port = settings().http_port;
    let addr: SocketAddr = (std::net::Ipv4Addr::LOCALHOST, http_port).into();
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if TcpStream::connect_timeout(&addr, Duration::from_millis(50)).is_ok() {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
