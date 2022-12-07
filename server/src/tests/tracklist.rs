use std::sync::Arc;

use futures_util::FutureExt;
use music_player_playback::player::Player;
use tokio::sync::oneshot;
use tonic::transport::Server;

use crate::{
    api::v1alpha1::{
        playback_service_server::PlaybackServiceServer,
        tracklist_service_client::TracklistServiceClient,
    },
    playback::Playback,
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
            .add_service(tonic_web::enable(PlaybackServiceServer::new(
                Playback::new(Arc::clone(&tracklist), Arc::clone(&cmd_tx)),
            )))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let _client = TracklistServiceClient::connect(url).await.unwrap();

    tx.send(()).unwrap();
    jh.await.unwrap();
}
