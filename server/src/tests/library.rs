use std::sync::Arc;

use futures_util::FutureExt;
use tokio::sync::oneshot;
use tonic::transport::Server;

use crate::{
    api::music::v1alpha1::{
        library_service_client::LibraryServiceClient, library_service_server::LibraryServiceServer,
        GetAlbumDetailsRequest, GetAlbumsRequest, GetArtistDetailsRequest, GetArtistsRequest,
        GetTrackDetailsRequest, GetTracksRequest,
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

    let request = tonic::Request::new(GetArtistsRequest {
        offset: 0,
        limit: 10,
    });
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
    let request = tonic::Request::new(GetAlbumsRequest {
        offset: 0,
        limit: 10,
    });
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
    let request = tonic::Request::new(GetTracksRequest {
        offset: 0,
        limit: 10,
    });
    let response = client.get_tracks(request).await.unwrap();
    let response = response.into_inner();

    assert_eq!(response.tracks.len(), 2);
    assert_eq!(response.tracks[0].id, "dd77dd0ea2de5208e4987001a59ba8e4");
    assert_eq!(response.tracks[0].title, "Fire Squad");
    assert_eq!(response.tracks[0].artist, "J. Cole");
    assert_eq!(response.tracks[0].duration, 288.238);
    assert_eq!(response.tracks[0].track_number, 6);
    assert_eq!(
        response.tracks[0].uri,
        "/tmp/audio/06 - J. Cole - Fire Squad(Explicit).m4a"
    );
    assert_eq!(response.tracks[1].id, "3ac1f1651b6ef6d5f3f55b711e3bfcd1");
    assert_eq!(response.tracks[1].title, "Wet Dreamz");
    assert_eq!(response.tracks[1].artist, "J. Cole");
    assert_eq!(response.tracks[1].duration, 239.381);
    assert_eq!(response.tracks[1].track_number, 3);
    assert_eq!(
        response.tracks[1].uri,
        "/tmp/audio/03 - J. Cole - Wet Dreamz(Explicit).m4a"
    );
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

    let mut client = LibraryServiceClient::connect(url).await.unwrap();
    let request = tonic::Request::new(GetTrackDetailsRequest {
        id: "3ac1f1651b6ef6d5f3f55b711e3bfcd1".to_owned(),
    });
    let response = client.get_track_details(request).await.unwrap();
    let response = response.into_inner();
    let track = response.track.unwrap();
    assert_eq!(track.id, "3ac1f1651b6ef6d5f3f55b711e3bfcd1");
    assert_eq!(track.title, "Wet Dreamz");
    assert_eq!(track.artist, "J. Cole");
    assert_eq!(track.duration, 239.381);
    assert_eq!(track.track_number, 3);
    assert_eq!(
        track.uri,
        "/tmp/audio/03 - J. Cole - Wet Dreamz(Explicit).m4a"
    );
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

    let mut client = LibraryServiceClient::connect(url).await.unwrap();
    let request = tonic::Request::new(GetAlbumDetailsRequest {
        id: "216ccc791352fbbffc11268b984db19a".to_owned(),
    });
    let response = client.get_album_details(request).await.unwrap();
    let response = response.into_inner();
    let album = response.album.unwrap();
    assert_eq!(album.id, "216ccc791352fbbffc11268b984db19a");
    assert_eq!(album.title, "2014 Forest Hills Drive");
    assert_eq!(album.artist, "J. Cole");
    assert_eq!(album.year, 2014);

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

    let mut client = LibraryServiceClient::connect(url).await.unwrap();
    let request = tonic::Request::new(GetArtistDetailsRequest {
        id: "b03cc90c455d92d8e9a0ce331e6de54d".to_owned(),
    });
    let response = client.get_artist_details(request).await.unwrap();
    let response = response.into_inner();
    let artist = response.artist.unwrap();
    assert_eq!(artist.id, "b03cc90c455d92d8e9a0ce331e6de54d");
    assert_eq!(artist.name, "J. Cole");

    tx.send(()).unwrap();
    jh.await.unwrap();
}
