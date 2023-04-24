use std::sync::Arc;

use futures_util::FutureExt;
use music_player_playback::player::Player;
use music_player_server::{
    api::music::v1alpha1::{
        playback_service_server::PlaybackServiceServer,
        tracklist_service_server::TracklistServiceServer,
    },
    playback::Playback,
    tracklist::Tracklist,
};
use tokio::sync::oneshot;
use tonic::transport::Server;

use crate::{tests::setup_new_params, tracklist::TracklistClient};

#[tokio::test]
async fn add() -> Result<(), Box<dyn std::error::Error>> {
    let host = "0.0.0.0".to_owned();
    let port = 4086;
    let (backend, audio_format, cmd_tx, cmd_rx, tracklist, db, addr, _url) =
        setup_new_params(port).await;

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
            .add_service(tonic_web::enable(TracklistServiceServer::new(
                Tracklist::new(Arc::clone(&tracklist), Arc::clone(&cmd_tx), db),
            )))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut client = TracklistClient::new(host, port).await?;

    client.add("3ac1f1651b6ef6d5f3f55b711e3bfcd1").await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    let (previous_tracks, next_tracks) = client.list().await?;
    assert_eq!(previous_tracks.len(), 1);
    assert_eq!(next_tracks.len(), 0);

    assert_eq!(previous_tracks[0].id, "3ac1f1651b6ef6d5f3f55b711e3bfcd1");

    tx.send(()).unwrap();
    jh.await.unwrap();

    Ok(())
}

#[tokio::test]
async fn list() -> Result<(), Box<dyn std::error::Error>> {
    let host = "0.0.0.0".to_owned();
    let port = 4087;
    let (backend, audio_format, cmd_tx, cmd_rx, tracklist, db, addr, _url) =
        setup_new_params(port).await;
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
            .add_service(tonic_web::enable(TracklistServiceServer::new(
                Tracklist::new(Arc::clone(&tracklist), Arc::clone(&cmd_tx), db),
            )))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut client = TracklistClient::new(host, port).await?;
    let (previous_tracks, next_tracks) = client.list().await?;
    assert_eq!(previous_tracks.len(), 0);
    assert_eq!(next_tracks.len(), 0);

    tx.send(()).unwrap();
    jh.await.unwrap();
    Ok(())
}
