//! Execute the real GraphQL mutations and inspect commands; no engine runs.
use super::*;
use crate::load_tracks;
use async_graphql::{EmptySubscription, Request, Schema, Variables};
use migration::{Migrator, MigratorTrait};
use music_player_playback::source_resolver::SourceResolver;
use music_player_storage::saved_servers::{self, NewServer};
use music_player_types::source::{LocalSourceRef, RemoteIdentity, ResourceKind, SourceRef};
use serde_json::{json, Value};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::mpsc::{unbounded_channel, UnboundedReceiver},
};

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

type TestSchema = Schema<TracklistQuery, TracklistMutation, EmptySubscription>;

struct Api {
    schema: TestSchema,
    device: Arc<Mutex<CurrentReceiverDevice>>,
    commands: Arc<StdMutex<UnboundedSender<PlayerCommand>>>,
    rx: UnboundedReceiver<PlayerCommand>,
}

fn api(db: Database) -> Api {
    let resolver = SourceResolver::new(db.clone(), "graphql-fixture".into(), true);
    api_with_resolver(db, resolver)
}

fn api_with_resolver(db: Database, resolver: SourceResolver) -> Api {
    let (tx, rx) = unbounded_channel();
    let commands = Arc::new(StdMutex::new(tx));
    let device = Arc::new(Mutex::new(CurrentReceiverDevice::new()));
    // Deliberately no ProviderState or discovery devices. A reserved handle
    // reaching the legacy browsing/decorating branch is a regression.
    let schema = Schema::build(TracklistQuery, TracklistMutation, EmptySubscription)
        .data(resolver)
        .data(db)
        .data(Arc::clone(&commands))
        .data(Arc::clone(&device))
        .data(Arc::new(StdMutex::new(TracklistState::new_empty())))
        .finish();
    Api {
        schema,
        device,
        commands,
        rx,
    }
}

fn input(id: &str, uri: &str) -> Value {
    json!({"id":id, "uri":uri, "title":"Supplied metadata", "discNumber":0})
}

fn single(method: &str, id: &str) -> Request {
    if method == "addTrack" {
        Request::new("mutation($track: TrackInput!) { addTrack(track: $track) { id title uri } }")
            .variables(Variables::from_json(json!({"track":input(id, "")})))
    } else {
        Request::new(format!("mutation($id: ID!) {{ {method}(id: $id) }}"))
            .variables(Variables::from_json(json!({"id":id})))
    }
}

fn batch(tracks: Vec<Value>) -> Request {
    Request::new("mutation($tracks: [TrackInput!]!) { addTracks(tracks: $tracks) }")
        .variables(Variables::from_json(json!({"tracks":tracks})))
}

fn no_commands(rx: &mut UnboundedReceiver<PlayerCommand>) {
    assert!(matches!(
        rx.try_recv(),
        Err(tokio::sync::mpsc::error::TryRecvError::Empty)
    ));
}

#[tokio::test]
async fn local_metadata_batch_retains_tags_and_checks_each_revision_without_loading_guest() {
    bounded(async {
        use music_player_playback::extension_accounts::ExtensionAccounts;
        use music_player_storage::extension_source_accounts::{self as accounts, AccountProtocol};
        let db = database().await;
        let config = std::collections::BTreeMap::new();
        let row = accounts::create_for_protocol(db.get_connection(), "fixture.legacy", "Legacy",
            &config, NOW, AccountProtocol::V1).await.unwrap();
        // Known bound state for API admission only; connect/WAT has separate tests.
        let row = accounts::bind_local_configuration(db.get_connection(), &row, &config).await.unwrap();
        let source = LocalSourceRef { resolver: "extension:fixture.legacy".into(), account_id: row.id,
            config_revision: row.config_revision as u64, kind: ResourceKind::Item, item_id: "item".into() };
        let manager = ExtensionAccounts::new(db.clone(), vec![], Default::default());
        let resolver = SourceResolver::from_settings(db.clone()).with_extensions(manager.clone(), vec!["mp3".into()]);
        let mut api = api_with_resolver(db.clone(), resolver.clone());
        let handle = source.to_handle();
        let response = api.schema.execute(batch(vec![input(&handle, &handle)])).await;
        assert!(response.errors.is_empty(), "{:?}", response.errors);
        assert!(matches!(api.rx.try_recv().unwrap(), PlayerCommand::LoadTracklist { tracks, .. }
            if tracks.len() == 1 && tracks[0].title == "Supplied metadata" && tracks[0].uri == handle));
        let mut wrong = source.clone();
        wrong.config_revision += 1;
        let response = api.schema.execute(batch(vec![input(&handle, &handle),
            input(&wrong.to_handle(), &wrong.to_handle())])).await;
        assert_eq!(response.errors.len(), 1);
        no_commands(&mut api.rx);
        for method in ["addTrack", "playNext", "playTrack"] {
            let response = api.schema.execute(single(method, &handle)).await;
            assert_eq!(response.errors.len(), 1);
            assert!(response.errors[0].message.contains("requires browse metadata"));
            no_commands(&mut api.rx);
        }
        let schema = Schema::build(crate::schema::media::MediaQuery, async_graphql::EmptyMutation, EmptySubscription)
            .data(db).data(resolver).finish();
        let response = schema.execute(Request::new("query($source: ID!) { mediaAudioOptions(source: $source) { source } }")
            .variables(Variables::from_json(json!({"source":handle})))).await;
        assert_eq!(response.errors.len(), 1);
        assert!(response.errors[0].message.contains("source v1 does not support audio selection"));
        assert!(manager.status().is_empty(), "admission must not load a guest");
    }).await;
}

#[tokio::test]
async fn injected_resolver_owns_metadata_options_and_queue_validation() {
    bounded(async {
        let owned = database().await;
        let unrelated = database().await;
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let reference = saved(&owned, &format!("http://{}", listener.local_addr().unwrap()), true).await;
        let resolver = SourceResolver::new(owned, "injected-fixture".into(), true);
        // The context DB deliberately has no account. Recreating a resolver
        // from that DB must fail these actual GraphQL requests.
        let mut api = api_with_resolver(unrelated.clone(), resolver.clone());
        for method in ["addTrack", "playNext", "playTrack"] {
            let (response, ()) = tokio::join!(
                api.schema.execute(single(method, &reference.to_handle())),
                serve_metadata(&listener),
            );
            assert!(response.errors.is_empty(), "{:?}", response.errors);
            if method == "playNext" {
                assert!(matches!(api.rx.try_recv().unwrap(), PlayerCommand::PlayNext(track) if track.id == reference.to_handle()));
            } else {
                if method == "playTrack" {
                    assert!(matches!(api.rx.try_recv().unwrap(), PlayerCommand::Stop));
                    assert!(matches!(api.rx.try_recv().unwrap(), PlayerCommand::Clear));
                }
                assert!(matches!(api.rx.try_recv().unwrap(), PlayerCommand::LoadTracklist { tracks, .. } if tracks.len() == 1 && tracks[0].id == reference.to_handle()));
            }
            no_commands(&mut api.rx);
        }
        let response = api.schema.execute(batch(vec![input(&reference.to_handle(), "")])).await;
        assert!(response.errors.is_empty(), "{:?}", response.errors);
        assert!(matches!(api.rx.try_recv().unwrap(), PlayerCommand::LoadTracklist { .. }));
        let schema = Schema::build(crate::schema::media::MediaQuery, async_graphql::EmptyMutation, EmptySubscription)
            .data(unrelated).data(resolver).finish();
        let request = Request::new("query($source: ID!) { mediaAudioOptions(source: $source) { source versions { id } } }")
            .variables(Variables::from_json(json!({"source":reference.to_handle()})));
        let (response, ()) = tokio::join!(schema.execute(request), serve_metadata(&listener));
        assert!(response.errors.is_empty(), "{:?}", response.errors);
        assert_eq!(response.data.into_json().unwrap()["mediaAudioOptions"]["source"], reference.to_handle());
        no_connections(&listener).await;
    }).await;
}

#[tokio::test]
async fn different_resolver_cannot_skip_batch_validation_for_same_account_and_remote() {
    bounded(async {
        let db = database().await;
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let native = saved(
            &db,
            &format!("http://{}", listener.local_addr().unwrap()),
            true,
        )
        .await;
        let mut extension = native.clone();
        extension.resolver = SourceRef::extension_resolver("fixture.other").unwrap();
        let resolver = SourceResolver::new(db.clone(), "batch-fixture".into(), true);
        let mut api = api_with_resolver(db, resolver.clone());
        let response = api
            .schema
            .execute(batch(vec![
                input(&native.to_handle(), ""),
                input(&extension.to_handle(), ""),
            ]))
            .await;
        assert_eq!(response.errors.len(), 1);
        assert!(response.errors[0]
            .message
            .contains("extension source runtime is not configured"));
        no_commands(&mut api.rx);
        let tracks = [native, extension]
            .into_iter()
            .map(|source| track_entity::Model {
                id: source.to_handle(),
                ..Default::default()
            })
            .collect();
        assert!(crate::load_tracks_with_resolver(
            &resolver,
            &api.commands,
            None,
            None,
            tracks,
            Some(0),
            false
        )
        .await
        .is_err());
        no_commands(&mut api.rx); // includes Stop/Clear: replacement is all-or-none
        no_connections(&listener).await;
    })
    .await;
}

#[tokio::test]
async fn missing_injected_resolver_is_explicit_and_never_falls_back_to_context_db() {
    bounded(async {
        let db = database().await;
        let reference = saved(&db, "http://127.0.0.1:1", true).await;
        let schema = Schema::build(
            crate::schema::media::MediaQuery,
            async_graphql::EmptyMutation,
            EmptySubscription,
        )
        .data(db)
        .finish();
        let response = schema
            .execute(
                Request::new(
                    "query($source: ID!) { mediaAudioOptions(source: $source) { source } }",
                )
                .variables(Variables::from_json(
                    json!({"source":reference.to_handle()}),
                )),
            )
            .await;
        assert_eq!(response.errors.len(), 1);
        assert_eq!(
            response.errors[0].message,
            "Shared source resolver is not configured"
        );
    })
    .await;
}

#[tokio::test]
async fn reserved_invalid_or_container_ids_never_fall_back_to_browsing() {
    bounded(async {
        let mut api = api(database().await);
        let mut container = source("missing");
        container.kind = ResourceKind::Container;
        for id in ["mp-source:v8?invalid".into(), container.to_handle()] {
            for method in ["addTrack", "playNext", "playTrack"] {
                let response = api.schema.execute(single(method, &id)).await;
                assert_eq!(response.errors.len(), 1);
                no_commands(&mut api.rx);
            }
        }
        let response = api.schema.execute(
            Request::new("mutation($track: TrackInput!) { addTrack(track: $track) { id } }")
                .variables(Variables::from_json(json!({"track":input(&source("missing").to_handle(), "https://example.invalid/fallback.mp3")})))
        ).await;
        assert_eq!(response.errors.len(), 1);
        no_commands(&mut api.rx);
    }).await;
}

#[tokio::test]
async fn invalid_and_unbound_batches_emit_no_commands() {
    bounded(async {
        let db = database().await;
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let unbound = saved(&db, &url, false).await;
        let mut api = api(db.clone());
        for id in [
            "mp-source:v99".into(),
            source("missing").to_handle(),
            unbound.to_handle(),
        ] {
            let response = api
                .schema
                .execute(batch(vec![
                    input("legacy", "https://example.invalid/track.mp3"),
                    input(&id, ""),
                ]))
                .await;
            assert_eq!(response.errors.len(), 1);
            no_commands(&mut api.rx);
            // This is the common replacement path used by playAlbum/Artist/
            // Playlist/Track. It must validate before emitting Stop or Clear.
            assert!(load_tracks(
                &db,
                &api.commands,
                None,
                None,
                vec![
                    track_entity::Model {
                        id: "legacy".into(),
                        uri: "file.mp3".into(),
                        ..Default::default()
                    },
                    track_entity::Model {
                        id,
                        ..Default::default()
                    },
                ],
                Some(0),
                false
            )
            .await
            .is_err());
            no_commands(&mut api.rx);
        }
        let bound = saved(&db, &url, true).await;
        let mut wrong_user = bound.clone();
        wrong_user.remote.user_id = "different-user".into();
        // Dedup must include remote identity, not just account ID.
        let response = api
            .schema
            .execute(batch(vec![
                input(&bound.to_handle(), ""),
                input(&wrong_user.to_handle(), ""),
            ]))
            .await;
        assert_eq!(response.errors.len(), 1);
        no_commands(&mut api.rx);
        no_connections(&listener).await;
    })
    .await;
}

#[tokio::test]
async fn bound_batch_preserves_handles_and_metadata_without_authentication() {
    bounded(async {
        let db = database().await;
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let source = saved(
            &db,
            &format!("http://{}", listener.local_addr().unwrap()),
            true,
        )
        .await;
        let handle = source.to_handle();
        let mut api = api(db.clone());
        let response = api
            .schema
            .execute(batch(vec![input(&handle, ""), input("", &handle)]))
            .await;
        assert!(response.errors.is_empty(), "{:?}", response.errors);
        match api.rx.try_recv().unwrap() {
            PlayerCommand::LoadTracklist {
                tracks,
                start_index: None,
            } => {
                assert_eq!(tracks.len(), 2);
                for track in tracks {
                    assert_eq!(track.id, handle);
                    assert_eq!(track.uri, handle);
                    assert_eq!(track.title, "Supplied metadata");
                }
            }
            _ => panic!("expected batch append"),
        }
        no_commands(&mut api.rx);
        load_tracks(
            &db,
            &api.commands,
            None,
            None,
            vec![track_entity::Model {
                id: handle.clone(),
                ..Default::default()
            }],
            Some(0),
            false,
        )
        .await
        .unwrap();
        assert!(matches!(api.rx.try_recv().unwrap(), PlayerCommand::Stop));
        assert!(matches!(api.rx.try_recv().unwrap(), PlayerCommand::Clear));
        match api.rx.try_recv().unwrap() {
            PlayerCommand::LoadTracklist {
                tracks,
                start_index: Some(0),
            } => {
                assert_eq!(tracks.len(), 1);
                assert_eq!(tracks[0].id, handle);
                assert_eq!(tracks[0].uri, handle);
            }
            _ => panic!("expected replacement"),
        }
        no_commands(&mut api.rx);
        no_connections(&listener).await;
    })
    .await;
}

#[tokio::test]
async fn chromecast_and_remote_music_player_cannot_receive_source_handles() {
    bounded(async {
        let db = database().await;
        let mut api = api(db.clone());
        let handle = source("unavailable-account").to_handle();
        let receivers: Vec<Box<dyn music_player_renderer::Player + Send>> = vec![
            Box::new(music_player_renderer::chromecast::Chromecast::new()),
            Box::new(music_player_renderer::local::Local::new()),
        ];
        for receiver in receivers {
            api.device.lock().await.client = Some(receiver);
            for method in ["addTrack", "playNext", "playTrack"] {
                let response = api.schema.execute(single(method, &handle)).await;
                assert_eq!(response.errors.len(), 1);
                assert!(response.errors[0]
                    .message
                    .contains("unsupported on remote receivers"));
                no_commands(&mut api.rx);
            }
            let response = api.schema.execute(batch(vec![input(&handle, "")])).await;
            assert_eq!(response.errors.len(), 1);
            assert!(response.errors[0]
                .message
                .contains("unsupported on remote receivers"));
            let mut device = api.device.lock().await;
            let error = load_tracks(
                &db,
                &api.commands,
                device.client.as_mut(),
                Some("127.0.0.1".into()),
                vec![track_entity::Model {
                    id: handle.clone(),
                    ..Default::default()
                }],
                Some(0),
                false,
            )
            .await
            .unwrap_err();
            assert!(error
                .to_string()
                .contains("unsupported on remote receivers"));
            no_commands(&mut api.rx);
        }
    })
    .await;
}

#[tokio::test]
async fn id_only_mutations_resolve_saved_metadata_without_provider_state() {
    bounded(async {
        let db = database().await;
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let source = saved(
            &db,
            &format!("http://{}", listener.local_addr().unwrap()),
            true,
        )
        .await;
        let mut api = api(db);
        let handle = source.to_handle();
        for method in ["addTrack", "playNext", "playTrack"] {
            let (response, ()) = tokio::join!(
                api.schema.execute(single(method, &handle)),
                serve_metadata(&listener)
            );
            assert!(response.errors.is_empty(), "{:?}", response.errors);
            if method == "playTrack" {
                assert!(matches!(api.rx.try_recv().unwrap(), PlayerCommand::Stop));
                assert!(matches!(api.rx.try_recv().unwrap(), PlayerCommand::Clear));
            }
            let track = match api.rx.try_recv().unwrap() {
                PlayerCommand::LoadTracklist {
                    mut tracks,
                    start_index,
                } if method != "playNext" => {
                    assert_eq!(
                        start_index,
                        if method == "playTrack" { Some(0) } else { None }
                    );
                    assert_eq!(tracks.len(), 1);
                    tracks.remove(0)
                }
                PlayerCommand::PlayNext(track) if method == "playNext" => track,
                _ => panic!("unexpected queue command"),
            };
            assert_eq!(track.id, handle);
            assert_eq!(track.uri, handle);
            assert_eq!(track.title, "Synthetic episode");
            assert!(track.album_id.is_none());
            assert!(track.album.cover.is_none());
            no_commands(&mut api.rx);
        }
        no_connections(&listener).await;
    })
    .await;
}
