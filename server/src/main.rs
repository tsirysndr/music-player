use music_player_server::api::v1alpha1::addons_service_server::AddonsServiceServer;
use music_player_server::api::v1alpha1::core_service_server::CoreServiceServer;
use music_player_server::api::v1alpha1::history_service_server::HistoryServiceServer;
use music_player_server::api::v1alpha1::library_service_server::LibraryServiceServer;
use music_player_server::api::v1alpha1::mixer_service_server::MixerServiceServer;
use music_player_server::api::v1alpha1::playback_service_server::PlaybackServiceServer;
use music_player_server::api::v1alpha1::playlist_service_server::PlaylistServiceServer;
use music_player_server::api::v1alpha1::tracklist_service_server::TracklistServiceServer;
use tonic::transport::Server;

use crate::addons::Addons;
use crate::core::Core;
use crate::history::History;
use crate::library::Library;
use crate::mixer::Mixer;
use crate::playback::Playback;
use crate::playlist::Playlist;
use crate::tracklist::Tracklist;

mod addons;
mod core;
mod history;
mod library;
mod mixer;
mod playback;
mod playlist;
mod tracklist;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();

    println!("Server listening on {}", addr);

    Server::builder()
        .accept_http1(true)
        .add_service(tonic_web::enable(AddonsServiceServer::new(
            Addons::default(),
        )))
        .add_service(tonic_web::enable(CoreServiceServer::new(Core::default())))
        .add_service(tonic_web::enable(HistoryServiceServer::new(
            History::default(),
        )))
        .add_service(tonic_web::enable(LibraryServiceServer::new(
            Library::default(),
        )))
        .add_service(tonic_web::enable(MixerServiceServer::new(Mixer::default())))
        .add_service(tonic_web::enable(PlaybackServiceServer::new(
            Playback::default(),
        )))
        .add_service(tonic_web::enable(PlaylistServiceServer::new(
            Playlist::default(),
        )))
        .add_service(tonic_web::enable(TracklistServiceServer::new(
            Tracklist::default(),
        )))
        .serve(addr)
        .await?;
    Ok(())
}
