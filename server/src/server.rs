use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{self, Arc};

use music_player_playback::player::PlayerCommand;
use music_player_provider::ProviderState;
use music_player_settings::{read_settings, Settings};
use music_player_storage::Database;
use music_player_tracklist::Tracklist as TracklistState;
use owo_colors::OwoColorize;
use tokio::net::{TcpListener, TcpStream, UnixListener};
use tokio::sync::mpsc::UnboundedSender as TokioUnboundedSender;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::Server;
use tungstenite::Message;

use crate::{
    addons::Addons, core::Core, history::History, library::Library, mixer::Mixer,
    playback::Playback, playlist::Playlist, servers::Servers, tracklist::Tracklist,
};

use crate::api::music::v1alpha1::{
    addons_service_server::AddonsServiceServer, analysis_service_server::AnalysisServiceServer,
    core_service_server::CoreServiceServer, history_service_server::HistoryServiceServer,
    library_service_server::LibraryServiceServer, mixer_service_server::MixerServiceServer,
    playback_service_server::PlaybackServiceServer, playlist_service_server::PlaylistServiceServer,
    servers_service_server::ServersServiceServer, tracklist_service_server::TracklistServiceServer,
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
    db: Database,
    tracklist: Arc<std::sync::Mutex<TracklistState>>,
    cmd_tx: Arc<std::sync::Mutex<TokioUnboundedSender<PlayerCommand>>>,
    peer_map: PeerMap,
    /// Where the library is read from, shared with the GraphQL schema.
    providers: Arc<ProviderState>,
    /// Auto-DJ's on/off and target, shared between the rpc handlers and the
    /// loop that tops the queue up.
    auto_dj: Arc<crate::analysis::AutoDj>,
    /// Progress of a background analysis pass, shared for the same reason.
    analysis_progress: Arc<crate::analysis::AnalysisProgress>,
}

impl MusicPlayerServer {
    pub fn new(
        tracklist: Arc<std::sync::Mutex<TracklistState>>,
        cmd_tx: Arc<std::sync::Mutex<TokioUnboundedSender<PlayerCommand>>>,
        peer_map: PeerMap,
        db: Database,
        providers: Arc<ProviderState>,
    ) -> Self {
        Self {
            db,
            tracklist,
            cmd_tx,
            peer_map,
            providers,
            auto_dj: Default::default(),
            analysis_progress: Default::default(),
        }
    }

    /// The analysis service, and the auto-DJ state it shares with the loop.
    fn analysis(&self) -> crate::analysis::Analysis {
        crate::analysis::Analysis::new(
            self.db.clone(),
            Arc::clone(&self.providers),
            Arc::clone(&self.tracklist),
            Arc::clone(&self.cmd_tx),
            Arc::clone(&self.auto_dj),
            Arc::clone(&self.analysis_progress),
        )
    }

    /// Start the loop that keeps the queue full while auto-DJ is on.
    ///
    /// Started with the server rather than when auto-DJ is switched on: it does
    /// nothing at all while disabled, and a loop that only exists sometimes is
    /// a loop that can be started twice.
    fn spawn_auto_dj(&self) {
        tokio::spawn(crate::analysis::run_auto_dj(self.analysis()));
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = read_settings().unwrap();
        let settings = config.try_deserialize::<Settings>().unwrap();
        let addr: SocketAddr = format!("0.0.0.0:{}", settings.port).parse().unwrap();

        self.spawn_auto_dj();

        println!("{}", BANNER.magenta());
        println!("Server listening on {}", addr.cyan());
        debug!("Listening on {:?}", addr);

        Server::builder()
            .accept_http1(true)
            .layer(tonic_web::GrpcWebLayer::new())
            .add_service(AddonsServiceServer::new(Addons::new(self.db.clone())))
            .add_service(CoreServiceServer::new(Core::default()))
            .add_service(HistoryServiceServer::new(History::new(self.db.clone())))
            .add_service(LibraryServiceServer::new(Library::new(
                self.db.clone(),
                Arc::clone(&self.providers),
            )))
            .add_service(MixerServiceServer::new(Mixer::new(Arc::clone(
                &self.cmd_tx,
            ))))
            .add_service(PlaybackServiceServer::new(Playback::new(
                Arc::clone(&self.tracklist),
                Arc::clone(&self.cmd_tx),
            )))
            .add_service(PlaylistServiceServer::new(Playlist::new(
                self.db.clone(),
                Arc::clone(&self.providers),
            )))
            .add_service(ServersServiceServer::new(Servers::new(
                self.db.clone(),
                Arc::clone(&self.providers),
            )))
            .add_service(TracklistServiceServer::new(Tracklist::new(
                Arc::clone(&self.tracklist),
                Arc::clone(&self.cmd_tx),
                self.db.clone(),
            )))
            .add_service(AnalysisServiceServer::new(self.analysis()))
            .serve(addr)
            .await?;
        Ok(())
    }

    pub async fn start_over_unix_domain_socket(
        &self,
        path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let socket_path = PathBuf::from(path);

        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }

        self.spawn_auto_dj();

        let listener = UnixListener::bind(socket_path)?;

        debug!("Listening on {:?}", listener.local_addr()?);

        Server::builder()
            .accept_http1(true)
            .layer(tonic_web::GrpcWebLayer::new())
            .add_service(AddonsServiceServer::new(Addons::new(self.db.clone())))
            .add_service(CoreServiceServer::new(Core::default()))
            .add_service(HistoryServiceServer::new(History::new(self.db.clone())))
            .add_service(LibraryServiceServer::new(Library::new(
                self.db.clone(),
                Arc::clone(&self.providers),
            )))
            .add_service(MixerServiceServer::new(Mixer::new(Arc::clone(
                &self.cmd_tx,
            ))))
            .add_service(PlaybackServiceServer::new(Playback::new(
                Arc::clone(&self.tracklist),
                Arc::clone(&self.cmd_tx),
            )))
            .add_service(PlaylistServiceServer::new(Playlist::new(
                self.db.clone(),
                Arc::clone(&self.providers),
            )))
            .add_service(ServersServiceServer::new(Servers::new(
                self.db.clone(),
                Arc::clone(&self.providers),
            )))
            .add_service(TracklistServiceServer::new(Tracklist::new(
                Arc::clone(&self.tracklist),
                Arc::clone(&self.cmd_tx),
                self.db.clone(),
            )))
            .add_service(AnalysisServiceServer::new(self.analysis()))
            .serve_with_incoming(UnixListenerStream::new(listener))
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
        let broadcast_recipients = peers.values();

        for recp in broadcast_recipients {
            recp.unbounded_send(msg.clone()).unwrap();
        }

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    println!("{} disconnected", addr);
    peer_map.lock().unwrap().remove(&addr);
}
