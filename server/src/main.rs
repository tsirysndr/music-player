use music_player_server::api::{
    core_service_server::CoreServiceServer, history_service_server::HistoryServiceServer,
};
use tonic::transport::Server;

use crate::core::Core;
use crate::history::History;

mod core;
mod history;
mod library;
mod mixer;
mod playback;
mod playlist;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();

    println!("Server listening on {}", addr);

    Server::builder()
        .accept_http1(true)
        .add_service(tonic_web::enable(CoreServiceServer::new(Core::default())))
        .add_service(tonic_web::enable(HistoryServiceServer::new(
            History::default(),
        )))
        .serve(addr)
        .await?;
    Ok(())
}
