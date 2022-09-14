use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use music_player_playback::player::Player;
use music_player_settings::read_settings;
use music_player_storage::Database;
use owo_colors::OwoColorize;
use tokio::sync::Mutex;
use tonic::transport::Server;

use crate::{
    addons::Addons, core::Core, history::History, library::Library, mixer::Mixer,
    playback::Playback, playlist::Playlist, tracklist::Tracklist,
};

use crate::api::v1alpha1::{
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

pub struct MusicPlayerServer {
    player: Arc<Mutex<Player>>,
}

impl MusicPlayerServer {
    pub fn new(player: Arc<Mutex<Player>>) -> Self {
        Self { player }
    }
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings = read_settings().unwrap();
        let settings = settings
            .try_deserialize::<HashMap<String, String>>()
            .unwrap();
        let port: u16 = settings.get("port").unwrap().parse().unwrap();
        let addr: SocketAddr = format!("[::1]:{}", port).parse().unwrap();

        println!("{}", BANNER.magenta());
        println!("Server listening on {}", addr.cyan());

        let db = Database::new().await;

        Server::builder()
            .accept_http1(true)
            .add_service(tonic_web::enable(AddonsServiceServer::new(Addons::new(
                Arc::new(db.clone()),
            ))))
            .add_service(tonic_web::enable(CoreServiceServer::new(Core::default())))
            .add_service(tonic_web::enable(HistoryServiceServer::new(History::new(
                Arc::new(db.clone()),
            ))))
            .add_service(tonic_web::enable(LibraryServiceServer::new(Library::new(
                Arc::new(db.clone()),
            ))))
            .add_service(tonic_web::enable(MixerServiceServer::new(Mixer::default())))
            .add_service(tonic_web::enable(PlaybackServiceServer::new(
                Playback::new(Arc::clone(&self.player)),
            )))
            .add_service(tonic_web::enable(PlaylistServiceServer::new(
                Playlist::new(Arc::new(db.clone())),
            )))
            .add_service(tonic_web::enable(TracklistServiceServer::new(
                Tracklist::new(Arc::clone(&self.player), Arc::new(db.clone())),
            )))
            .serve(addr)
            .await?;
        Ok(())
    }
}
