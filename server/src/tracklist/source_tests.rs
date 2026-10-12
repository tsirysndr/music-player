//! API commands only: no Player, discovery, decoder, or audio output is started.
use super::*;
use migration::{Migrator, MigratorTrait};
use music_player_storage::saved_servers::{self, NewServer};
use music_player_types::source::{RemoteIdentity, ResourceKind};
use serde_json::json;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::mpsc::{unbounded_channel, UnboundedReceiver},
};
use tonic::{Code, Request};

const NOW: &str = "2026-09-30T12:00:00Z";

async fn bounded<T>(future: impl std::future::Future<Output = T>) -> T {
    tokio::time::timeout(std::time::Duration::from_secs(10), future)
        .await
        .expect("source API test timed out")
}

async fn database() -> Database {
    let mut options = sea_orm::ConnectOptions::new("sqlite::memory:");
    options.max_connections(1).sqlx_logging(false);
    let connection = sea_orm::Database::connect(options).await.unwrap();
    Migrator::up(&connection, None).await.unwrap();
    Database { connection }
}

fn source(account: &str) -> SourceRef {
    SourceRef {
        resolver: "emby".into(),
        account_id: account.into(),
        remote: RemoteIdentity {
            server_id: "server-a".into(),
            user_id: "user-a".into(),
        },
        kind: ResourceKind::Item,
        item_id: "55508".into(),
    }
}

async fn saved(db: &Database, url: &str, bind: bool) -> SourceRef {
    let row = saved_servers::upsert(
        db.get_connection(),
        &NewServer::new("emby", "Synthetic source", url)
            .with_credentials(Some("family".into()), Some("synthetic-password".into())),
        NOW,
    )
    .await
    .unwrap();
    let source = source(&row.id);
    if bind {
        saved_servers::bind_remote_identity(db.get_connection(), &row, &source.remote)
            .await
            .unwrap();
    }
    source
}

async fn no_connections(listener: &TcpListener) {
    let connected = std::future::poll_fn(|cx| {
        std::task::Poll::Ready(match listener.poll_accept(cx) {
            std::task::Poll::Ready(Ok(_)) => true,
            std::task::Poll::Ready(Err(_)) => panic!("fixture listener failed"),
            std::task::Poll::Pending => false,
        })
    })
    .await;
    assert!(!connected, "validation attempted a network connection");
}

// Exactly authentication then metadata. All I/O belongs to the caller's
// timeout/join, so a failed assertion leaves no spawned fixture or socket.
async fn serve_metadata(listener: &TcpListener) {
    for (method, path, response) in [
        (
            "POST",
            "/Users/AuthenticateByName",
            json!({
                "AccessToken":"synthetic-token", "ServerId":"server-a", "User":{"Id":"user-a"}
            }),
        ),
        (
            "GET",
            "/Users/user-a/Items/55508",
            json!({
                "Id":"55508", "Name":"Synthetic episode", "Type":"Episode", "MediaType":"Video",
                "IsFolder":false, "RunTimeTicks":100_000_000_u64
            }),
        ),
    ] {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut bytes = Vec::new();
        let mut buffer = [0; 2048];
        let end = loop {
            if let Some(pos) = bytes.windows(4).position(|v| v == b"\r\n\r\n") {
                break pos + 4;
            }
            assert!(bytes.len() < 64 * 1024);
            let n = socket.read(&mut buffer).await.unwrap();
            assert!(n > 0, "truncated fixture request");
            bytes.extend_from_slice(&buffer[..n]);
        };
        let head = std::str::from_utf8(&bytes[..end]).unwrap().to_owned();
        let first = head
            .lines()
            .next()
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>();
        assert_eq!(first[0], method);
        assert_eq!(first[1].split('?').next(), Some(path));
        let length = head
            .lines()
            .filter_map(|line| line.split_once(':'))
            .find(|(key, _)| key.eq_ignore_ascii_case("content-length"))
            .map(|(_, value)| value.trim().parse::<usize>().unwrap())
            .unwrap_or(0);
        assert!(length <= 64 * 1024);
        while bytes.len() < end + length {
            let n = socket.read(&mut buffer).await.unwrap();
            assert!(n > 0, "truncated fixture body");
            bytes.extend_from_slice(&buffer[..n]);
        }
        if method == "POST" {
            // Do not include credentials or headers in assertion diagnostics.
            assert!(
                serde_json::from_slice::<serde_json::Value>(&bytes[end..end + length]).unwrap()
                    == json!({"Username":"family", "Pw":"synthetic-password"})
            );
        } else {
            assert!(head
                .lines()
                .filter_map(|line| line.split_once(':'))
                .any(|(key, value)| key.eq_ignore_ascii_case("x-emby-token")
                    && value.trim() == "synthetic-token"));
        }
        let body = response.to_string();
        socket.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).as_bytes()).await.unwrap();
        socket.shutdown().await.unwrap();
    }
}

fn service(db: Database) -> (Tracklist, UnboundedReceiver<PlayerCommand>) {
    let (tx, rx) = unbounded_channel();
    (
        Tracklist::new(
            Arc::new(std::sync::Mutex::new(TracklistState::new_empty())),
            Arc::new(std::sync::Mutex::new(tx)),
            db,
        ),
        rx,
    )
}

fn legacy() -> Track {
    Track {
        id: "legacy".into(),
        uri: "https://example.invalid/music.mp3".into(),
        ..Default::default()
    }
}

fn entry(source: &SourceRef) -> Track {
    Track {
        id: source.to_handle(),
        title: "Supplied metadata".into(),
        ..Default::default()
    }
}

fn no_commands(rx: &mut UnboundedReceiver<PlayerCommand>) {
    assert!(matches!(
        rx.try_recv(),
        Err(tokio::sync::mpsc::error::TryRecvError::Empty)
    ));
}

#[tokio::test]
async fn malformed_and_container_batches_emit_no_stop_clear_or_append() {
    bounded(async {
        let (api, mut rx) = service(database().await);
        let mut folder = source("unused");
        folder.kind = ResourceKind::Container;
        let mut conflict = entry(&source("unused"));
        conflict.uri = "https://example.invalid/fallback.mp3".into();
        for invalid in [
            Track {
                id: "mp-source:v9?bad".into(),
                ..Default::default()
            },
            entry(&folder),
            conflict,
        ] {
            let tracks = vec![legacy(), invalid];
            let err = api
                .load_tracks(Request::new(LoadTracksRequest {
                    tracks: tracks.clone(),
                    start_index: 0,
                }))
                .await
                .unwrap_err();
            assert_eq!(err.code(), Code::InvalidArgument);
            no_commands(&mut rx);
            let err = api
                .add_tracks(Request::new(AddTracksRequest { tracks }))
                .await
                .unwrap_err();
            assert_eq!(err.code(), Code::InvalidArgument);
            no_commands(&mut rx);
        }
    })
    .await;
}

#[tokio::test]
async fn missing_unbound_and_wrong_identity_reject_the_whole_batch() {
    bounded(async {
        let db = database().await;
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let unbound = saved(&db, &url, false).await;
        let (api, mut rx) = service(db.clone());
        for source in [source("missing"), unbound.clone()] {
            assert_eq!(
                api.load_tracks(Request::new(LoadTracksRequest {
                    tracks: vec![legacy(), entry(&source)],
                    start_index: 0,
                }))
                .await
                .unwrap_err()
                .code(),
                Code::FailedPrecondition
            );
            no_commands(&mut rx);
        }
        let bound = saved(&db, &url, true).await;
        let mut wrong_user = bound.clone();
        wrong_user.remote.user_id = "different-user".into();
        // Same account is insufficient for dedup: this second identity must fail.
        assert_eq!(
            api.add_tracks(Request::new(AddTracksRequest {
                tracks: vec![entry(&bound), entry(&wrong_user)],
            }))
            .await
            .unwrap_err()
            .code(),
            Code::FailedPrecondition
        );
        no_commands(&mut rx);
        no_connections(&listener).await;
    })
    .await;
}

#[tokio::test]
async fn bound_metadata_batch_normalizes_without_authentication() {
    bounded(async {
        let db = database().await;
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let source = saved(
            &db,
            &format!("http://{}", listener.local_addr().unwrap()),
            true,
        )
        .await;
        let (api, mut rx) = service(db);
        let mut uri_only = entry(&source);
        uri_only.uri = uri_only.id.clone();
        uri_only.id.clear();
        let tracks = vec![entry(&source), uri_only];
        api.load_tracks(Request::new(LoadTracksRequest {
            tracks: tracks.clone(),
            start_index: 1,
        }))
        .await
        .unwrap();
        assert!(matches!(rx.try_recv().unwrap(), PlayerCommand::Stop));
        assert!(matches!(rx.try_recv().unwrap(), PlayerCommand::Clear));
        match rx.try_recv().unwrap() {
            PlayerCommand::LoadTracklist {
                tracks,
                start_index,
            } => {
                assert_eq!(start_index, Some(1));
                assert_eq!(tracks.len(), 2);
                for track in tracks {
                    assert_eq!(track.id, source.to_handle());
                    assert_eq!(track.uri, source.to_handle());
                    assert_eq!(track.title, "Supplied metadata");
                }
            }
            _ => panic!("expected atomic load"),
        }
        no_commands(&mut rx);
        api.add_tracks(Request::new(AddTracksRequest { tracks }))
            .await
            .unwrap();
        assert!(matches!(
            rx.try_recv().unwrap(),
            PlayerCommand::LoadTracklist {
                start_index: None,
                ..
            }
        ));
        no_commands(&mut rx);
        no_connections(&listener).await;
    })
    .await;
}

#[tokio::test]
async fn id_only_add_and_play_next_use_saved_metadata() {
    bounded(async {
        let db = database().await;
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let source = saved(
            &db,
            &format!("http://{}", listener.local_addr().unwrap()),
            true,
        )
        .await;
        let (api, mut rx) = service(db);
        let (reply, ()) = tokio::join!(
            api.add_track(Request::new(AddTrackRequest {
                track: Some(entry(&source)),
            })),
            serve_metadata(&listener)
        );
        reply.unwrap();
        let track = match rx.try_recv().unwrap() {
            PlayerCommand::LoadTracklist {
                mut tracks,
                start_index: None,
            } => {
                assert_eq!(tracks.len(), 1);
                tracks.remove(0)
            }
            _ => panic!("expected append"),
        };
        assert_eq!(track.title, "Synthetic episode");
        assert_eq!(track.id, source.to_handle());
        assert_eq!(track.uri, track.id);
        assert!(track.album_id.is_none());
        assert!(track.album.cover.is_none());
        let (reply, ()) = tokio::join!(
            api.play_next(Request::new(PlayNextRequest {
                track: Some(entry(&source)),
            })),
            serve_metadata(&listener)
        );
        reply.unwrap();
        match rx.try_recv().unwrap() {
            PlayerCommand::PlayNext(track) => {
                assert_eq!(track.title, "Synthetic episode");
                assert_eq!(track.id, source.to_handle());
                assert_eq!(track.uri, track.id);
            }
            _ => panic!("expected play next"),
        }
        no_commands(&mut rx);
        no_connections(&listener).await;
    })
    .await;
}

#[tokio::test]
async fn missing_inputs_and_invalid_start_indices_leave_playback_untouched() {
    bounded(async {
        let (api, mut rx) = service(database().await);
        assert_eq!(
            api.add_track(Request::new(AddTrackRequest { track: None }))
                .await
                .unwrap_err()
                .code(),
            Code::InvalidArgument
        );
        assert_eq!(
            api.play_next(Request::new(PlayNextRequest { track: None }))
                .await
                .unwrap_err()
                .code(),
            Code::InvalidArgument
        );
        for start_index in [-1, 1] {
            assert_eq!(
                api.load_tracks(Request::new(LoadTracksRequest {
                    tracks: vec![legacy()],
                    start_index,
                }))
                .await
                .unwrap_err()
                .code(),
                Code::InvalidArgument
            );
        }
        for track in [
            Track {
                id: "mp-source:v0".into(),
                ..Default::default()
            },
            Track {
                id: "bare-id".into(),
                ..Default::default()
            },
        ] {
            assert_eq!(
                api.play_next(Request::new(PlayNextRequest { track: Some(track) }))
                    .await
                    .unwrap_err()
                    .code(),
                Code::InvalidArgument
            );
        }
        no_commands(&mut rx);
    })
    .await;
}
