use std::sync::Arc;

use futures_util::FutureExt;
use music_player_playback::player::Player;
use tokio::sync::oneshot;
use tonic::transport::Server;

use crate::{
    api::v1alpha1::{
        playback_service_client::PlaybackServiceClient,
        playback_service_server::PlaybackServiceServer,
    },
    playback::Playback,
};

use super::setup_new_params;

#[tokio::test]
async fn get_currently_playing_song() {
    let (backend, audio_format, cmd_tx, cmd_rx, tracklist, _db, addr, url) =
        setup_new_params(7078).await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let (tx, rx) = oneshot::channel();

    let jh = tokio::spawn(async move {
        Server::builder()
            .accept_http1(true)
            .add_service(tonic_web::enable(PlaybackServiceServer::new(
                Playback::new(Arc::clone(&tracklist), Arc::clone(&cmd_tx)),
            )))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let _client = PlaybackServiceClient::connect(url).await.unwrap();

    tx.send(()).unwrap();
    jh.await.unwrap();
}

#[tokio::test]
async fn next() {
    let (backend, audio_format, cmd_tx, cmd_rx, tracklist, db, addr, url) =
        setup_new_params(7079).await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let (tx, rx) = oneshot::channel();
    let jh = tokio::spawn(async move {
        Server::builder()
            .accept_http1(true)
            .add_service(tonic_web::enable(PlaybackServiceServer::new(
                Playback::new(Arc::clone(&tracklist), Arc::clone(&cmd_tx)),
            )))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let _client = PlaybackServiceClient::connect(url).await.unwrap();

    tx.send(()).unwrap();
    jh.await.unwrap();
}

#[tokio::test]
async fn previous() {
    let (backend, audio_format, cmd_tx, cmd_rx, tracklist, db, addr, url) =
        setup_new_params(7080).await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let (tx, rx) = oneshot::channel();
    let jh = tokio::spawn(async move {
        Server::builder()
            .accept_http1(true)
            .add_service(tonic_web::enable(PlaybackServiceServer::new(
                Playback::new(Arc::clone(&tracklist), Arc::clone(&cmd_tx)),
            )))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let _client = PlaybackServiceClient::connect(url).await.unwrap();

    tx.send(()).unwrap();
    jh.await.unwrap();
}

#[tokio::test]
async fn play() {
    let (backend, audio_format, cmd_tx, cmd_rx, tracklist, db, addr, url) =
        setup_new_params(7081).await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let (tx, rx) = oneshot::channel();
    let jh = tokio::spawn(async move {
        Server::builder()
            .accept_http1(true)
            .add_service(tonic_web::enable(PlaybackServiceServer::new(
                Playback::new(Arc::clone(&tracklist), Arc::clone(&cmd_tx)),
            )))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    tx.send(()).unwrap();
    jh.await.unwrap();
}

#[tokio::test]
async fn pause() {
    let (backend, audio_format, cmd_tx, cmd_rx, tracklist, db, addr, url) =
        setup_new_params(7082).await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let (tx, rx) = oneshot::channel();

    let jh = tokio::spawn(async move {
        Server::builder()
            .accept_http1(true)
            .add_service(tonic_web::enable(PlaybackServiceServer::new(
                Playback::new(Arc::clone(&tracklist), Arc::clone(&cmd_tx)),
            )))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    tx.send(()).unwrap();
    jh.await.unwrap();
}
