use futures_util::FutureExt;
use music_player_server::{
    api::music::v1alpha1::library_service_server::LibraryServiceServer, library::Library,
};
use tokio::sync::oneshot;
use tonic::transport::Server;

use crate::{library::LibraryClient, tests::setup_new_params};

/// Spawn an in-process gRPC server exposing the library service and return a
/// connected client plus the server's shutdown handle.
async fn setup_client(
    port: u16,
) -> Result<(LibraryClient, oneshot::Sender<()>), Box<dyn std::error::Error>> {
    let (_cmd_tx, _cmd_rx, _tracklist, db, addr, _url) = setup_new_params(port).await;
    let (tx, rx) = oneshot::channel();
    tokio::spawn(async move {
        Server::builder()
            .add_service(LibraryServiceServer::new(Library::new(db)))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let client = LibraryClient::new("0.0.0.0".to_owned(), port).await?;
    Ok((client, tx))
}

#[tokio::test]
async fn album() -> Result<(), Box<dyn std::error::Error>> {
    let (mut client, _shutdown) = setup_client(6091).await?;
    let response = client.album("ecd3fb5214ef4faf77a0eba4eedb2638").await?;
    let response = response.unwrap();
    assert_eq!(response.id, "ecd3fb5214ef4faf77a0eba4eedb2638");
    assert_eq!(response.title, "2014 Forest Hills Drive");
    assert_eq!(response.artist, "J. Cole");
    assert_eq!(response.year, 2014);
    Ok(())
}

#[tokio::test]
async fn albums() -> Result<(), Box<dyn std::error::Error>> {
    let (mut client, _shutdown) = setup_client(6092).await?;
    let response = client.albums(None, 0, 100).await?;
    assert_eq!(response.len(), 1);
    assert_eq!(response[0].id, "ecd3fb5214ef4faf77a0eba4eedb2638");
    assert_eq!(response[0].title, "2014 Forest Hills Drive");
    assert_eq!(response[0].artist, "J. Cole");
    assert_eq!(response[0].year, 2014);
    Ok(())
}

#[tokio::test]
async fn artist() -> Result<(), Box<dyn std::error::Error>> {
    let (mut client, _shutdown) = setup_client(6093).await?;
    let response = client.artist("b03cc90c455d92d8e9a0ce331e6de54d").await?;
    let response = response.unwrap();
    assert_eq!(response.id, "b03cc90c455d92d8e9a0ce331e6de54d");
    assert_eq!(response.name, "J. Cole");
    Ok(())
}

#[tokio::test]
async fn artists() -> Result<(), Box<dyn std::error::Error>> {
    let (mut client, _shutdown) = setup_client(6094).await?;
    let response = client.artists(None, 0, 100).await?;
    assert_eq!(response.len(), 1);
    assert_eq!(response[0].id, "b03cc90c455d92d8e9a0ce331e6de54d");
    assert_eq!(response[0].name, "J. Cole");
    Ok(())
}

#[tokio::test]
async fn songs() -> Result<(), Box<dyn std::error::Error>> {
    let (mut client, _shutdown) = setup_client(6095).await?;
    let response = client.songs(None, 0, 100).await?;
    assert_eq!(response.len(), 2);
    assert_eq!(response[0].id, "dd77dd0ea2de5208e4987001a59ba8e4");
    assert_eq!(response[0].title, "Fire Squad");
    assert_eq!(response[0].artist, "J. Cole");
    assert_eq!(
        response[0].album.as_ref().unwrap().title,
        "2014 Forest Hills Drive"
    );
    assert_eq!(response[1].id, "3ac1f1651b6ef6d5f3f55b711e3bfcd1");
    assert_eq!(response[1].title, "Wet Dreamz");
    assert_eq!(response[1].artist, "J. Cole");
    assert_eq!(
        response[1].album.as_ref().unwrap().title,
        "2014 Forest Hills Drive"
    );
    Ok(())
}

#[tokio::test]
async fn search() -> Result<(), Box<dyn std::error::Error>> {
    let (mut client, _shutdown) = setup_client(6096).await?;
    let response = client.search("fire").await?;
    assert_eq!(response.tracks.len(), 1);
    assert_eq!(response.tracks[0].title, "Fire Squad");
    Ok(())
}
