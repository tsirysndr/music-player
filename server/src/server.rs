use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{self, Arc};

use music_player_playback::player::PlayerCommand;
use music_player_settings::{read_settings, Settings};
use music_player_storage::Database;
use music_player_tracklist::Tracklist as TracklistState;
use owo_colors::OwoColorize;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::UnboundedSender as TokioUnboundedSender;
use tokio::sync::Mutex;
use tonic::transport::Server;
use tungstenite::Message;

use crate::{
    addons::Addons, core::Core, history::History, library::Library, mixer::Mixer,
    playback::Playback, playlist::Playlist, tracklist::Tracklist,
};

use crate::api::music::v1alpha1::{
    addons_service_server::AddonsServiceServer, core_service_server::CoreServiceServer,
    history_service_server::HistoryServiceServer, library_service_server::LibraryServiceServer,
    mixer_service_server::MixerServiceServer, playback_service_server::PlaybackServiceServer,
    playlist_service_server::PlaylistServiceServer,
    tracklist_service_server::TracklistServiceServer,
};

const BANNER: &str = r#"
    __  ___           _      ____  __                     
   /  |/  /_  _______(_)____/ __ \/ /___ ___  _____  _____
  / /|_/ / / / / ___/ / ___/ /_/ / / __ `/ / / / _ \/ ___/
 / /  / / /_/ (__  ) / /__/ ____/ / /_/ / /_/ /  __/ /    
/_/  /_/\__,_/____/_/\___/_/   /_/\__,_/\__, /\___/_/     
                                       /____/             
"#;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<sync::Mutex<HashMap<SocketAddr, Tx>>>;

pub struct MusicPlayerServer {
    db: Arc<Mutex<Database>>,
    tracklist: Arc<std::sync::Mutex<TracklistState>>,
    cmd_tx: Arc<std::sync::Mutex<TokioUnboundedSender<PlayerCommand>>>,
    peer_map: PeerMap,
}

impl MusicPlayerServer {
    pub fn new(
        tracklist: Arc<std::sync::Mutex<TracklistState>>,
        cmd_tx: Arc<std::sync::Mutex<TokioUnboundedSender<PlayerCommand>>>,
        peer_map: PeerMap,
        db: Arc<Mutex<Database>>,
    ) -> Self {
        Self {
            db,
            tracklist,
            cmd_tx,
            peer_map,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = read_settings().unwrap();
        let settings = config.try_deserialize::<Settings>().unwrap();
        let addr: SocketAddr = format!("0.0.0.0:{}", settings.port).parse().unwrap();

        println!("{}", BANNER.magenta());
        println!("Server listening on {}", addr.cyan());

        Server::builder()
            .accept_http1(true)
            .add_service(tonic_web::enable(AddonsServiceServer::new(Addons::new(
                Arc::clone(&self.db),
            ))))
            .add_service(tonic_web::enable(CoreServiceServer::new(Core::default())))
            .add_service(tonic_web::enable(HistoryServiceServer::new(History::new(
                Arc::clone(&self.db),
            ))))
            .add_service(tonic_web::enable(LibraryServiceServer::new(Library::new(
                Arc::clone(&self.db),
            ))))
            .add_service(tonic_web::enable(MixerServiceServer::new(Mixer::default())))
            .add_service(tonic_web::enable(PlaybackServiceServer::new(
                Playback::new(Arc::clone(&self.tracklist), Arc::clone(&self.cmd_tx)),
            )))
            .add_service(tonic_web::enable(PlaylistServiceServer::new(
                Playlist::new(Arc::clone(&self.db)),
            )))
            .add_service(tonic_web::enable(TracklistServiceServer::new(
                Tracklist::new(
                    Arc::clone(&self.tracklist),
                    Arc::clone(&self.cmd_tx),
                    Arc::clone(&self.db),
                ),
            )))
            .serve(addr)
            .await?;
        Ok(())
    }

    pub async fn start_ws(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = read_settings().unwrap();
        let settings = config.try_deserialize::<Settings>().unwrap();
        let addr: SocketAddr = format!("0.0.0.0:{}", settings.ws_port).parse().unwrap();

        let try_socket = TcpListener::bind(addr).await;
        let listener = try_socket.expect("Failed to bind");
        println!("Websocket server listening on {}", addr.cyan());

        // Let's spawn the handling of each connection in a separate task.
        while let Ok((stream, addr)) = listener.accept().await {
            tokio::spawn(handle_connection(Arc::clone(&self.peer_map), stream, addr));
        }
        Ok(())
    }
}

async fn handle_connection(peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr.bright_green());
    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr.bright_green());

    let (tx, rx) = unbounded();
    peer_map.lock().unwrap().insert(addr, tx);

    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        println!(
            "Received a message from {}: {}",
            addr,
            msg.to_text().unwrap()
        );
        let peers = peer_map.lock().unwrap();
        // We want to broadcast the message to everyone except ourselves.
        let broadcast_recipients = peers.iter().map(|(_, ws_sink)| ws_sink);

        for recp in broadcast_recipients {
            recp.unbounded_send(msg.clone()).unwrap();
        }

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    println!("{} disconnected", &addr);
    peer_map.lock().unwrap().remove(&addr);
}
