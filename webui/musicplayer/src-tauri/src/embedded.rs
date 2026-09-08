//! Start the full music-player daemon for the Tauri wrapper when needed.
//!
//! The web UI still uses the in-process GraphQL schema for low-latency IPC,
//! while this background daemon exposes the normal gRPC, websocket and HTTP
//! endpoints to other music-player clients.

use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream};
use std::sync::{self, Arc};
use std::time::Duration;

use futures_channel::mpsc::UnboundedSender;
use music_player_graphql::schema::{
    objects::{player_state::PlayerState, track::Track},
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

type PeerMap = Arc<sync::Mutex<HashMap<SocketAddr, UnboundedSender<Message>>>>;

fn settings() -> Settings {
    read_settings()
        .expect("failed to read settings")
        .try_deserialize::<Settings>()
        .expect("failed to parse settings")
}

fn remote_host_configured() -> bool {
    std::env::var("MUSIC_PLAYER_HOST")
        .ok()
        .is_some_and(|host| host != "127.0.0.1" && host != "localhost")
}

fn is_running() -> bool {
    let port = settings().port;
    let addr: SocketAddr = (std::net::Ipv4Addr::LOCALHOST, port).into();
    TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_ok()
}

/// Spawn the embedded daemon unless a local or explicitly configured remote
/// daemon is already being used.
pub fn ensure_running() -> bool {
    if remote_host_configured() {
        return false;
    }
    if is_running() {
        return true;
    }
    std::thread::Builder::new()
        .name("music-player-embedded-daemon".into())
        .spawn(boot)
        .expect("failed to spawn embedded music-player daemon");
    true
}

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

    let peers: PeerMap = Arc::new(sync::Mutex::new(HashMap::new()));
    let tracklist = Arc::new(sync::Mutex::new(Tracklist::new_empty()));
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cmd_tx = Arc::new(sync::Mutex::new(cmd_tx));

    let broadcast_peers = Arc::clone(&peers);
    let (_player, _) = Player::new(
        move |event| match event {
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
                let msg = Event {
                    event_type: "current_track".into(),
                    data: serde_json::to_string(&TrackEvent {
                        track,
                        index: position as u32,
                        is_playing,
                        position_ms,
                    })
                    .unwrap(),
                };
                let peers = broadcast_peers.lock().unwrap();
                for peer in peers.values() {
                    let _ =
                        peer.unbounded_send(Message::text(serde_json::to_string(&msg).unwrap()));
                }
            }
            PlayerEvent::TrackTimePosition { position_ms } => {
                SimpleBroker::publish(PositionMilliseconds { position_ms });
            }
            _ => {}
        },
        Arc::clone(&cmd_tx),
        Arc::new(sync::Mutex::new(cmd_rx)),
        Arc::clone(&tracklist),
    );

    music_player_discovery::register_services();
    runtime.block_on(async {
        music_player_server::scrobbler::spawn(Arc::clone(&tracklist));
        music_player_server::remote::spawn(Arc::clone(&tracklist), Arc::clone(&cmd_tx));
    });
    music_player_server::media_controls::spawn(Arc::clone(&tracklist), Arc::clone(&cmd_tx));

    // One provider registry per process, shared by the gRPC server and the
    // GraphQL/web server — otherwise the two disagree about which server the
    // library screens read from.
    let providers = {
        let mut registry = music_player_provider::ProviderRegistry::new();
        music_player_provider::register_builtin(&mut registry);
        Arc::new(music_player_provider::ProviderState::new(Arc::new(
            registry,
        )))
    };
    let grpc_providers = Arc::clone(&providers);
    let ws_providers = Arc::clone(&providers);

    let grpc_tracklist = Arc::clone(&tracklist);
    let grpc_cmd_tx = Arc::clone(&cmd_tx);
    let grpc_peers = Arc::clone(&peers);
    let grpc_db = db.clone();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("gRPC runtime");
        if let Err(error) = runtime.block_on(
            MusicPlayerServer::new(
                grpc_tracklist,
                grpc_cmd_tx,
                grpc_peers,
                grpc_db,
                grpc_providers,
            )
            .start(),
        ) {
            tracing::error!("embedded gRPC server failed: {error}");
        }
    });

    let ws_tracklist = Arc::clone(&tracklist);
    let ws_cmd_tx = Arc::clone(&cmd_tx);
    let ws_peers = Arc::clone(&peers);
    let ws_db = db.clone();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("websocket runtime");
        if let Err(error) = runtime.block_on(
            MusicPlayerServer::new(ws_tracklist, ws_cmd_tx, ws_peers, ws_db, ws_providers)
                .start_ws(),
        ) {
            tracing::error!("embedded websocket server failed: {error}");
        }
    });

    // One provider registry, shared by every API surface so they cannot
    // disagree about which server the library screens are reading from.

    if let Err(error) = runtime.block_on(start_webui(cmd_tx, tracklist, providers)) {
        tracing::error!("embedded web UI failed: {error}");
    }
}
