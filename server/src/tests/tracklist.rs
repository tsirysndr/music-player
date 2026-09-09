use std::sync::Arc;

use futures_util::FutureExt;
use music_player_playback::player::{Player, PlayerCommand};
use tokio::sync::oneshot;
use tonic::transport::Server;

use crate::{
    api::{
        metadata::v1alpha1::Track,
        music::v1alpha1::{
            tracklist_service_client::TracklistServiceClient,
            tracklist_service_server::TracklistServiceServer, AddTrackRequest, AddTracksRequest,
            GetTracklistTracksRequest,
        },
    },
    tracklist::Tracklist,
};

use super::setup_new_params;

#[tokio::test]
async fn get_tracklist_tracks() {
    let (cmd_tx, cmd_rx, tracklist, db, addr, url) = setup_new_params(7083).await;
    let (_, _) = Player::new(
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
    let (tx, rx) = oneshot::channel();
    let jh = tokio::spawn(async move {
        Server::builder()
            .accept_http1(true)
            .layer(tonic_web::GrpcWebLayer::new())
            .add_service(TracklistServiceServer::new(Tracklist::new(
                Arc::clone(&tracklist),
                Arc::clone(&cmd_tx),
                db,
            )))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut client = TracklistServiceClient::connect(url).await.unwrap();
    let request = tonic::Request::new(GetTracklistTracksRequest {});
    let response = client.get_tracklist_tracks(request).await.unwrap();
    let response = response.into_inner();
    assert_eq!(response.previous_tracks.len(), 0);
    assert_eq!(response.next_tracks.len(), 0);

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

    let request = tonic::Request::new(GetTracklistTracksRequest {});
    let response = client.get_tracklist_tracks(request).await.unwrap();
    let response = response.into_inner();
    assert_eq!(response.previous_tracks.len(), 1);
    assert_eq!(response.next_tracks.len(), 0);

    tx.send(()).unwrap();
    jh.await.unwrap();
}

/// Appending tracks that are not in the local database.
///
/// This is the case `AddTrack` cannot serve: it looks its id up in the local
/// `track` table, which holds nothing at all when a remote provider is
/// connected. `AddTracks` takes whole tracks and appends them as given, so ids
/// belonging to a Subsonic or Jellyfin library work. It used to be
/// `unimplemented!()` — calling it killed the connection.
///
/// No `Player` runs here, deliberately. What this rpc is responsible for is the
/// command it emits; whether the engine can then open a given stream is the
/// player's business, and mixing the two would make this a test of whether the
/// made-up urls happen to resolve.
#[tokio::test]
async fn add_tracks_appends_tracks_the_database_has_never_seen() {
    let (cmd_tx, cmd_rx, tracklist, db, addr, url) = setup_new_params(7084).await;
    let (tx, rx) = oneshot::channel();
    let jh = tokio::spawn(async move {
        Server::builder()
            .accept_http1(true)
            .layer(tonic_web::GrpcWebLayer::new())
            .add_service(TracklistServiceServer::new(Tracklist::new(
                Arc::clone(&tracklist),
                Arc::clone(&cmd_tx),
                db,
            )))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut client = TracklistServiceClient::connect(url).await.unwrap();

    // An empty list is not an error, and must load nothing: an agent that
    // filtered its selection down to nothing should leave the queue alone.
    client
        .add_tracks(tonic::Request::new(AddTracksRequest { tracks: vec![] }))
        .await
        .unwrap();
    assert!(cmd_rx.lock().unwrap().try_recv().is_err());

    let remote = |id: &str, title: &str| Track {
        id: id.to_owned(),
        title: title.to_owned(),
        artist: "Nobody In The Database".to_owned(),
        uri: format!("https://example.invalid/rest/stream?id={id}"),
        ..Default::default()
    };
    client
        .add_tracks(tonic::Request::new(AddTracksRequest {
            tracks: vec![remote("remote-1", "First"), remote("remote-2", "Second")],
        }))
        .await
        .unwrap();

    match cmd_rx.lock().unwrap().try_recv() {
        Ok(PlayerCommand::LoadTracklist {
            tracks,
            start_index,
        }) => {
            // `None` is what makes this an append: a start index would move
            // playback to the tracks just added.
            assert_eq!(start_index, None);
            assert_eq!(tracks.len(), 2);
            // In the order given, and carrying the uris they arrived with —
            // the point of taking whole tracks is that nothing here resolves
            // them against a table that may not know them.
            assert_eq!(tracks[0].id, "remote-1");
            assert_eq!(tracks[1].id, "remote-2");
            assert!(tracks[0].uri.starts_with("https://"));
        }
        other => panic!("expected a LoadTracklist append, got {other:?}"),
    }

    tx.send(()).unwrap();
    jh.await.unwrap();
}
