use std::sync::Arc;

use futures_util::FutureExt;
use music_player_playback::player::Player;
use tokio::sync::oneshot;
use tonic::transport::Server;

use crate::{
    api::{
        metadata::v1alpha1::Track,
        music::v1alpha1::{
            tracklist_service_client::TracklistServiceClient,
            tracklist_service_server::TracklistServiceServer, AddTrackRequest,
            GetTracklistTracksRequest,
        },
    },
    tracklist::Tracklist,
};

use super::setup_new_params;

#[tokio::test]
async fn get_tracklist_tracks() {
    let (backend, audio_format, cmd_tx, cmd_rx, tracklist, db, addr, url) =
        setup_new_params(7083).await;
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
            .add_service(tonic_web::enable(TracklistServiceServer::new(
                Tracklist::new(Arc::clone(&tracklist), Arc::clone(&cmd_tx), db),
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
