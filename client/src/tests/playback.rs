use futures_util::FutureExt;
use music_player_playback::player::Player;
use music_player_server::{
    api::v1alpha1::{
        playback_service_server::PlaybackServiceServer,
        tracklist_service_server::TracklistServiceServer,
    },
    playback::Playback,
    tracklist::Tracklist,
};
use std::sync::Arc;
use tokio::sync::oneshot;
use tonic::transport::Server;

use crate::{playback::PlaybackClient, tests::setup_new_params, tracklist::TracklistClient};

#[tokio::test]
async fn play() -> Result<(), Box<dyn std::error::Error>> {
    let (backend, audio_format, cmd_tx, cmd_rx, tracklist, db, addr, _url) =
        setup_new_params(4081).await;
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
                Tracklist::new(Arc::clone(&tracklist), Arc::clone(&cmd_tx), Arc::clone(&db)),
            )))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let mut client = TracklistClient::new(4081).await?;

    client.add("3ac1f1651b6ef6d5f3f55b711e3bfcd1").await?;

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let mut client = PlaybackClient::new(4081).await.unwrap();

    client.pause().await.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let (__, _, _, is_playing) = client.current().await.unwrap();
    assert_eq!(is_playing, false);

    client.play().await.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let (__, _, _, is_playing) = client.current().await.unwrap();
    assert_eq!(is_playing, true);

    tx.send(()).unwrap();
    jh.await.unwrap();
    Ok(())
}

#[tokio::test]
async fn pause() -> Result<(), Box<dyn std::error::Error>> {
    let (backend, audio_format, cmd_tx, cmd_rx, tracklist, db, addr, _url) =
        setup_new_params(4079).await;
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
                Tracklist::new(Arc::clone(&tracklist), Arc::clone(&cmd_tx), Arc::clone(&db)),
            )))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let mut client = TracklistClient::new(4079).await?;

    client.add("3ac1f1651b6ef6d5f3f55b711e3bfcd1").await?;

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let mut client = PlaybackClient::new(4079).await.unwrap();

    let (_, _, _, is_playing) = client.current().await?;
    assert_eq!(is_playing, true);

    client.pause().await.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let (_, _, _, is_playing) = client.current().await?;

    assert_eq!(is_playing, false);

    tx.send(()).unwrap();
    jh.await.unwrap();
    Ok(())
}

#[tokio::test]
async fn stop() -> Result<(), Box<dyn std::error::Error>> {
    let (backend, audio_format, cmd_tx, cmd_rx, tracklist, db, addr, _url) =
        setup_new_params(4078).await;
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
                Tracklist::new(Arc::clone(&tracklist), Arc::clone(&cmd_tx), Arc::clone(&db)),
            )))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let mut client = TracklistClient::new(4078).await?;

    client.add("3ac1f1651b6ef6d5f3f55b711e3bfcd1").await?;

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let mut client = PlaybackClient::new(4078).await.unwrap();

    client.stop().await.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let (current_track, _, _, is_playing) = client.current().await?;

    assert_eq!(is_playing, false);
    assert_eq!(current_track, None);

    tx.send(()).unwrap();
    jh.await.unwrap();
    Ok(())
}

#[tokio::test]
async fn next() -> Result<(), Box<dyn std::error::Error>> {
    let (backend, audio_format, cmd_tx, cmd_rx, tracklist, db, addr, _url) =
        setup_new_params(4082).await;
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
                Tracklist::new(Arc::clone(&tracklist), Arc::clone(&cmd_tx), Arc::clone(&db)),
            )))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let mut client = TracklistClient::new(4082).await?;

    client.add("3ac1f1651b6ef6d5f3f55b711e3bfcd1").await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    client.add("dd77dd0ea2de5208e4987001a59ba8e4").await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    let mut client = PlaybackClient::new(4082).await?;
    client.next().await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let (current_track, _, _, _) = client.current().await?;
    let current_track = current_track.unwrap();
    assert_eq!(current_track.id, "dd77dd0ea2de5208e4987001a59ba8e4");
    assert_eq!(current_track.title, "Fire Squad");
    assert_eq!(current_track.artist, "J. Cole");

    tx.send(()).unwrap();
    jh.await.unwrap();

    Ok(())
}

#[tokio::test]
async fn prev() -> Result<(), Box<dyn std::error::Error>> {
    let (backend, audio_format, cmd_tx, cmd_rx, tracklist, db, addr, _url) =
        setup_new_params(4083).await;
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
                Tracklist::new(Arc::clone(&tracklist), Arc::clone(&cmd_tx), Arc::clone(&db)),
            )))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let mut client = TracklistClient::new(4083).await?;

    client.add("3ac1f1651b6ef6d5f3f55b711e3bfcd1").await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    client.add("dd77dd0ea2de5208e4987001a59ba8e4").await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    let mut client = PlaybackClient::new(4083).await?;
    client.next().await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    client.prev().await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let (current_track, _, _, _) = client.current().await?;
    let current_track = current_track.unwrap();
    assert_eq!(current_track.id, "3ac1f1651b6ef6d5f3f55b711e3bfcd1");
    assert_eq!(current_track.title, "Wet Dreamz");
    assert_eq!(current_track.artist, "J. Cole");

    tx.send(()).unwrap();
    jh.await.unwrap();
    Ok(())
}

#[tokio::test]
async fn current() -> Result<(), Box<dyn std::error::Error>> {
    let (backend, audio_format, cmd_tx, cmd_rx, tracklist, db, addr, _url) =
        setup_new_params(4084).await;
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
                Tracklist::new(Arc::clone(&tracklist), Arc::clone(&cmd_tx), Arc::clone(&db)),
            )))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let mut client = TracklistClient::new(4084).await?;
    client.add("3ac1f1651b6ef6d5f3f55b711e3bfcd1").await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    let mut client = PlaybackClient::new(4084).await?;
    let (current_track, _, _, _) = client.current().await?;
    assert_ne!(current_track, None);

    tx.send(()).unwrap();
    jh.await.unwrap();
    Ok(())
}
