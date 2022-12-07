use std::sync::Arc;

use futures_util::FutureExt;
use tokio::sync::oneshot;
use tonic::transport::Server;
use tungstenite::http::response;

use crate::{
    api::v1alpha1::{
        library_service_client::LibraryServiceClient, library_service_server::LibraryServiceServer,
        GetAlbumsRequest, GetArtistsRequest, GetTracksRequest,
    },
    library::Library,
};

use super::setup_new_params;

#[tokio::test]
async fn scan() {
    let (_backend, _audio_format, _cmd_tx, _cmd_rx, _tracklist, db, addr, url) =
        setup_new_params(7070).await;
    let (tx, rx) = oneshot::channel();
    let jh = tokio::spawn(async move {
        Server::builder()
            .accept_http1(true)
            .add_service(tonic_web::enable(LibraryServiceServer::new(Library::new(
                Arc::clone(&db),
            ))))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let _client = LibraryServiceClient::connect(url).await.unwrap();

    tx.send(()).unwrap();

    jh.await.unwrap();
}

#[tokio::test]
async fn search() {
    let (_backend, _audio_format, _cmd_tx, _cmd_rx, _tracklist, db, addr, url) =
        setup_new_params(7071).await;
    let (tx, rx) = oneshot::channel();
    let jh = tokio::spawn(async move {
        Server::builder()
            .accept_http1(true)
            .add_service(tonic_web::enable(LibraryServiceServer::new(Library::new(
                Arc::clone(&db),
            ))))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let _client = LibraryServiceClient::connect(url).await.unwrap();
    tx.send(()).unwrap();
    jh.await.unwrap();
}

#[tokio::test]
async fn get_artists() -> Result<(), Box<dyn std::error::Error>> {
    let (_backend, _audio_format, _cmd_tx, _cmd_rx, _tracklist, db, addr, url) =
        setup_new_params(6072).await;
    let (tx, rx) = oneshot::channel();
    let jh = tokio::spawn(async move {
        Server::builder()
            .accept_http1(true)
            .add_service(tonic_web::enable(LibraryServiceServer::new(Library::new(
                Arc::clone(&db),
            ))))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut client = LibraryServiceClient::connect(url).await.unwrap();

    let request = tonic::Request::new(GetArtistsRequest {});
    let response = client.get_artists(request).await?;
    let response = response.into_inner();

    assert_eq!(response.artists.len(), 1);
    assert_eq!(response.artists[0].id, "b03cc90c455d92d8e9a0ce331e6de54d");
    assert_eq!(response.artists[0].name, "J. Cole");
    tx.send(()).unwrap();
    jh.await.unwrap();
    Ok(())
}

#[tokio::test]
async fn get_albums() {
    let (_backend, _audio_format, _cmd_tx, _cmd_rx, _tracklist, db, addr, url) =
        setup_new_params(5073).await;
    let (tx, rx) = oneshot::channel();
    let jh = tokio::spawn(async move {
        Server::builder()
            .accept_http1(true)
            .add_service(tonic_web::enable(LibraryServiceServer::new(Library::new(
                Arc::clone(&db),
            ))))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut client = LibraryServiceClient::connect(url).await.unwrap();
    let request = tonic::Request::new(GetAlbumsRequest {});
    let response = client.get_albums(request).await.unwrap();
    let response = response.into_inner();
    assert_eq!(response.albums.len(), 1);
    assert_eq!(response.albums[0].id, "216ccc791352fbbffc11268b984db19a");
    assert_eq!(response.albums[0].title, "2014 Forest Hills Drive");
    tx.send(()).unwrap();
    jh.await.unwrap();
}

#[tokio::test]
async fn get_tracks() {
    let (_backend, _audio_format, _cmd_tx, _cmd_rx, _tracklist, db, addr, url) =
        setup_new_params(7074).await;
    let (tx, rx) = oneshot::channel();
    let jh = tokio::spawn(async move {
        Server::builder()
            .accept_http1(true)
            .add_service(tonic_web::enable(LibraryServiceServer::new(Library::new(
                Arc::clone(&db),
            ))))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut client = LibraryServiceClient::connect(url).await.unwrap();
    let request = tonic::Request::new(GetTracksRequest {});
    let response = client.get_tracks(request).await.unwrap();

    assert_eq!(response.into_inner().tracks.len(), 2);

    tx.send(()).unwrap();
    jh.await.unwrap();
}

#[tokio::test]
async fn get_track_details() {
    let (_backend, _audio_format, _cmd_tx, _cmd_rx, _tracklist, db, addr, url) =
        setup_new_params(7075).await;
    let (tx, rx) = oneshot::channel::<()>();
    let jh = tokio::spawn(async move {
        Server::builder()
            .accept_http1(true)
            .add_service(tonic_web::enable(LibraryServiceServer::new(Library::new(
                Arc::clone(&db),
            ))))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let _client = LibraryServiceClient::connect(url).await.unwrap();

    tx.send(()).unwrap();
    jh.await.unwrap();
}

#[tokio::test]
async fn get_album_details() {
    let (_backend, _audio_format, _cmd_tx, _cmd_rx, _tracklist, db, addr, url) =
        setup_new_params(7076).await;
    let (tx, rx) = oneshot::channel::<()>();
    let jh = tokio::spawn(async move {
        Server::builder()
            .accept_http1(true)
            .add_service(tonic_web::enable(LibraryServiceServer::new(Library::new(
                Arc::clone(&db),
            ))))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });

    let url = format!("http://{}", addr.to_string());

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let _client = LibraryServiceClient::connect(url).await.unwrap();

    tx.send(()).unwrap();
    jh.await.unwrap();
}

#[tokio::test]
async fn get_artist_details() {
    let (_backend, _audio_format, _cmd_tx, _cmd_rx, _tracklist, db, addr, url) =
        setup_new_params(7067).await;
    let (tx, rx) = oneshot::channel();
    let jh = tokio::spawn(async move {
        Server::builder()
            .accept_http1(true)
            .add_service(tonic_web::enable(LibraryServiceServer::new(Library::new(
                Arc::clone(&db),
            ))))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let _client = LibraryServiceClient::connect(url).await.unwrap();

    tx.send(()).unwrap();
    jh.await.unwrap();
}
