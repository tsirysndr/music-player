//! Full library metadata must survive both directions of the real gRPC wire.
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use music_player_playback::player::PlayerCommand;
use prost::Message;
use tokio::{net::TcpListener, sync::mpsc};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

use crate::{
    api::{
        metadata::v1alpha1::Track,
        music::v1alpha1::{
            tracklist_service_client::TracklistServiceClient,
            tracklist_service_server::TracklistServiceServer, AddTracksRequest,
            GetTracklistTracksRequest,
        },
    },
    tracklist::Tracklist,
    LIBRARY_MESSAGE_LIMIT,
};

struct RunningServer(tokio::task::JoinHandle<()>);
impl Drop for RunningServer {
    fn drop(&mut self) {
        self.0.abort();
    }
}

#[tokio::test]
async fn full_library_above_default_grpc_limit_appends_once_and_reads_without_truncation() {
    const COUNT: usize = 17_462;
    let tracks: Vec<Track> = (0..COUNT)
        .map(|index| Track {
            id: format!("large-library-{index}"),
            title: format!(
                "Episode {index}: {}",
                "A long original title with complete metadata. ".repeat(8)
            ),
            artist: "Series title".into(),
            uri: format!("https://fixture.invalid/audio/{index}"),
            ..Default::default()
        })
        .collect();
    let request = AddTracksRequest {
        tracks: tracks.clone(),
    };
    assert!(request.encoded_len() > 4 * 1024 * 1024);
    assert!(request.encoded_len() < LIBRARY_MESSAGE_LIMIT);

    // No engine or local library: assert the single append command itself.
    let db = music_player_storage::Database {
        connection: sea_orm::Database::connect("sqlite::memory:").await.unwrap(),
    };
    let queue = Arc::new(Mutex::new(music_player_tracklist::Tracklist::new_empty()));
    let (sender, mut commands) = mpsc::unbounded_channel();
    let service = Tracklist::new(queue.clone(), Arc::new(Mutex::new(sender)), db);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let _server = RunningServer(tokio::spawn(async move {
        Server::builder()
            .add_service(
                TracklistServiceServer::new(service)
                    .max_decoding_message_size(LIBRARY_MESSAGE_LIMIT),
            )
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .unwrap();
    }));
    let mut client = TracklistServiceClient::connect(endpoint)
        .await
        .unwrap()
        .max_decoding_message_size(LIBRARY_MESSAGE_LIMIT);
    tokio::time::timeout(Duration::from_secs(20), client.add_tracks(request))
        .await
        .unwrap()
        .unwrap();
    let loaded = match commands.try_recv().unwrap() {
        PlayerCommand::LoadTracklist {
            tracks,
            start_index,
        } => {
            assert_eq!(start_index, None);
            assert_eq!(tracks.len(), COUNT);
            tracks
        }
        other => panic!("expected one append command, got {other:?}"),
    };
    assert!(commands.try_recv().is_err());
    *queue.lock().unwrap() = music_player_tracklist::Tracklist::new(loaded);

    let response = tokio::time::timeout(
        Duration::from_secs(20),
        client.get_tracklist_tracks(GetTracklistTracksRequest {}),
    )
    .await
    .unwrap()
    .unwrap()
    .into_inner();
    assert!(response.encoded_len() > 4 * 1024 * 1024);
    assert!(response.previous_tracks.is_empty());
    assert_eq!(response.next_tracks.len(), COUNT);
    for (actual, expected) in response.next_tracks.iter().zip(&tracks) {
        assert_eq!(actual.id, expected.id);
        assert_eq!(actual.title, expected.title);
        assert_eq!(actual.uri, expected.uri);
    }
}
