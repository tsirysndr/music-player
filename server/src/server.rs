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

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
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
