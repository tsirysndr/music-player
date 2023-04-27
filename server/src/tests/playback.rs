use std::sync::Arc;

use futures_util::FutureExt;
use music_player_playback::player::Player;
use tokio::sync::oneshot;
use tonic::transport::Server;

use crate::{
    api::{
        metadata::v1alpha1::Track,
        music::v1alpha1::{
            playback_service_client::PlaybackServiceClient,
            playback_service_server::PlaybackServiceServer,
            tracklist_service_client::TracklistServiceClient,
            tracklist_service_server::TracklistServiceServer, AddTrackRequest,
            GetCurrentlyPlayingSongRequest, NextRequest, PauseRequest, PlayRequest,
            PreviousRequest,
        },
    },
    playback::Playback,
    tracklist::Tracklist,
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

    let mut client = PlaybackServiceClient::connect(url).await.unwrap();
    let request = tonic::Request::new(GetCurrentlyPlayingSongRequest {});
    let response = client.get_currently_playing_song(request).await.unwrap();
    let response = response.into_inner();
    assert_eq!(response.track, None);

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
            .add_service(tonic_web::enable(TracklistServiceServer::new(
                Tracklist::new(Arc::clone(&tracklist), Arc::clone(&cmd_tx), db),
            )))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut client = TracklistServiceClient::connect(url.clone()).await.unwrap();
    let request = tonic::Request::new(AddTrackRequest {
        track: Some(Track {
            id: "3ac1f1651b6ef6d5f3f55b711e3bfcd1".to_owned(),
            title: "Wet Dreamz".to_owned(),
            artist: "J. Cole".to_owned(),
            uri: "/tmp/audio/06 - J. Cole - Fire Squad(Explicit).m4a".to_owned(),
            ..Default::default()
        }),
    });
    client.add_track(request).await.unwrap();

    let request = tonic::Request::new(AddTrackRequest {
        track: Some(Track {
            id: "dd77dd0ea2de5208e4987001a59ba8e4".to_owned(),
            title: "Fire Squad".to_owned(),
            artist: "J. Cole".to_owned(),
            uri: "/tmp/audio/06 - J. Cole - Fire Squad(Explicit).m4a".to_owned(),
            ..Default::default()
        }),
    });
    client.add_track(request).await.unwrap();

    let mut client = PlaybackServiceClient::connect(url.clone()).await.unwrap();
    client
        .next(tonic::Request::new(NextRequest {}))
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let response = client
        .get_currently_playing_song(tonic::Request::new(GetCurrentlyPlayingSongRequest {}))
        .await
        .unwrap();
    let response = response.into_inner();
    let current_track = response.track.unwrap();
    assert_eq!(current_track.id, "dd77dd0ea2de5208e4987001a59ba8e4");
    assert_eq!(current_track.title, "Fire Squad");
    assert_eq!(current_track.artist, "J. Cole");

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
            .add_service(tonic_web::enable(TracklistServiceServer::new(
                Tracklist::new(Arc::clone(&tracklist), Arc::clone(&cmd_tx), db),
            )))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut client = TracklistServiceClient::connect(url.clone()).await.unwrap();
    let request = tonic::Request::new(AddTrackRequest {
        track: Some(Track {
            id: "3ac1f1651b6ef6d5f3f55b711e3bfcd1".to_owned(),
            title: "Wet Dreamz".to_owned(),
            artist: "J. Cole".to_owned(),
            uri: "/tmp/audio/06 - J. Cole - Fire Squad(Explicit).m4a".to_owned(),
            ..Default::default()
        }),
    });
    client.add_track(request).await.unwrap();

    let request = tonic::Request::new(AddTrackRequest {
        track: Some(Track {
            id: "dd77dd0ea2de5208e4987001a59ba8e4".to_owned(),
            title: "Fire Squad".to_owned(),
            artist: "J. Cole".to_owned(),
            uri: "/tmp/audio/06 - J. Cole - Fire Squad(Explicit).m4a".to_owned(),
            ..Default::default()
        }),
    });
    client.add_track(request).await.unwrap();

    let mut client = PlaybackServiceClient::connect(url.clone()).await.unwrap();
    client
        .next(tonic::Request::new(NextRequest {}))
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    client
        .previous(tonic::Request::new(PreviousRequest {}))
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let response = client
        .get_currently_playing_song(tonic::Request::new(GetCurrentlyPlayingSongRequest {}))
        .await
        .unwrap();
    let response = response.into_inner();
    let current_track = response.track.unwrap();
    assert_eq!(current_track.id, "3ac1f1651b6ef6d5f3f55b711e3bfcd1");
    assert_eq!(current_track.title, "Wet Dreamz");
    assert_eq!(current_track.artist, "J. Cole");

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
            .add_service(tonic_web::enable(TracklistServiceServer::new(
                Tracklist::new(Arc::clone(&tracklist), Arc::clone(&cmd_tx), db),
            )))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut client = TracklistServiceClient::connect(url.clone()).await.unwrap();
    let request = tonic::Request::new(AddTrackRequest {
        track: Some(Track {
            id: "3ac1f1651b6ef6d5f3f55b711e3bfcd1".to_owned(),
            title: "Wet Dreamz".to_owned(),
            artist: "J. Cole".to_owned(),
            uri: "/tmp/audio/06 - J. Cole - Fire Squad(Explicit).m4a".to_owned(),
            ..Default::default()
        }),
    });
    client.add_track(request).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let mut client = PlaybackServiceClient::connect(url.clone()).await.unwrap();
    let response = client
        .get_currently_playing_song(tonic::Request::new(GetCurrentlyPlayingSongRequest {}))
        .await
        .unwrap();
    let response = response.into_inner();
    assert_eq!(response.is_playing, true);

    client
        .pause(tonic::Request::new(PauseRequest {}))
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let response = client
        .get_currently_playing_song(tonic::Request::new(GetCurrentlyPlayingSongRequest {}))
        .await
        .unwrap();
    let response = response.into_inner();
    assert_eq!(response.is_playing, false);

    client
        .play(tonic::Request::new(PlayRequest {}))
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let response = client
        .get_currently_playing_song(tonic::Request::new(GetCurrentlyPlayingSongRequest {}))
        .await
        .unwrap();
    let response = response.into_inner();
    assert_eq!(response.is_playing, true);

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
            .add_service(tonic_web::enable(TracklistServiceServer::new(
                Tracklist::new(Arc::clone(&tracklist), Arc::clone(&cmd_tx), db),
            )))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut client = TracklistServiceClient::connect(url.clone()).await.unwrap();
    let request = tonic::Request::new(AddTrackRequest {
        track: Some(Track {
            id: "3ac1f1651b6ef6d5f3f55b711e3bfcd1".to_owned(),
            title: "Wet Dreamz".to_owned(),
            artist: "J. Cole".to_owned(),
            uri: "/tmp/audio/06 - J. Cole - Fire Squad(Explicit).m4a".to_owned(),
            ..Default::default()
        }),
    });
    client.add_track(request).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let mut client = PlaybackServiceClient::connect(url.clone()).await.unwrap();
    let response = client
        .get_currently_playing_song(tonic::Request::new(GetCurrentlyPlayingSongRequest {}))
        .await
        .unwrap();
    let response = response.into_inner();
    assert_eq!(response.is_playing, true);

    client
        .pause(tonic::Request::new(PauseRequest {}))
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let response = client
        .get_currently_playing_song(tonic::Request::new(GetCurrentlyPlayingSongRequest {}))
        .await
        .unwrap();
    let response = response.into_inner();
    assert_eq!(response.is_playing, false);

    tx.send(()).unwrap();
    jh.await.unwrap();
}
