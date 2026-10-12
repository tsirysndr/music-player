//! Real SQLite and loopback HTTP; credentials and metadata are synthetic.
//! Fixture futures are joined in the test, never detached or driven by sleeps.

use super::*;
use migration::{Migrator, MigratorTrait};
use music_player_provider::emby_playback::AudioPin;
use music_player_storage::saved_servers::{NewServer, PasswordUpdate, SavedServer};
use music_player_types::source::RemoteIdentity;
use sea_orm::{ConnectOptions, ConnectionTrait};
use serde_json::{json, Value};
use std::{cell::Cell, collections::BTreeMap, future::Future, task::Poll};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Notify,
};

const NOW: &str = "2026-09-30T12:00:00Z";
const DEVICE: &str = "source-resolver-fixture";
const TOKEN: &str = "synthetic-source-token";
const AUTH: &str = "/Users/AuthenticateByName";
const ITEM: &str = "/Users/user-a/Items/55508";

async fn bounded<T>(future: impl Future<Output = T>) -> T {
    tokio::time::timeout(Duration::from_secs(10), future)
        .await
        .expect("source resolver fixture timed out")
}

async fn database() -> Database {
    let mut options = ConnectOptions::new("sqlite::memory:");
    options.max_connections(1).sqlx_logging(false);
    let connection = sea_orm::Database::connect(options).await.unwrap();
    Migrator::up(&connection, None).await.unwrap();
    Database { connection }
}

fn identity() -> RemoteIdentity {
    RemoteIdentity {
        server_id: "server-a".into(),
        user_id: "user-a".into(),
    }
}

fn source(account: &SavedServer) -> SourceRef {
    SourceRef {
        resolver: "emby".into(),
        account_id: account.id.clone(),
        remote: identity(),
        kind: ResourceKind::Item,
        item_id: "55508".into(),
    }
}

async fn save(db: &Database, url: &str, username: &str, bind: bool) -> SavedServer {
    let row = saved_servers::upsert(
        db.get_connection(),
        &NewServer::new("emby", "Synthetic account", url)
            .with_credentials(Some(username.into()), None)
            .with_password_update(PasswordUpdate::Set(String::new())),
        NOW,
    )
    .await
    .unwrap();
    if bind {
        saved_servers::bind_remote_identity(db.get_connection(), &row, &identity())
            .await
            .unwrap()
    } else {
        row
    }
}

fn resolver(db: &Database) -> SourceResolver {
    SourceResolver::new(db.clone(), DEVICE.into(), true)
}

#[tokio::test]
async fn extension_reference_cannot_reuse_a_native_account_binding() {
    bounded(async {
        let db = database().await;
        let account = save(&db, "http://127.0.0.1:1", "family", true).await;
        let native = source(&account);
        let resolver = resolver(&db);
        assert!(resolver.saved_snapshot(&native).await.is_ok());
        let mut extension = native;
        extension.resolver = SourceRef::extension_resolver("com.example.emby").unwrap();
        assert!(SourceRef::parse(&extension.to_handle()).is_ok());
        // Same saved account, remote identity and item are insufficient to
        // authorize dispatch through a different resolver implementation.
        let error = resolver.saved_snapshot(&extension).await.err().unwrap();
        assert_eq!(
            error.to_string(),
            "source is not owned by the native Emby resolver"
        );
        // Public dispatch must not authenticate the matching native account
        // when no extension manager was injected. The fixture URL is closed.
        for error in [
            resolver.validate_saved(&extension).await.err().unwrap(),
            resolver.track(&extension).await.err().unwrap(),
            resolver.audio_options(&extension).await.err().unwrap(),
            resolver
                .resolve_playback(
                    &SourceHandle::Remote(extension.clone()),
                    &AudioSelection::Auto,
                    0,
                )
                .await
                .err()
                .unwrap(),
        ] {
            assert_eq!(
                error.to_string(),
                "extension source runtime is not configured"
            );
        }
        let mut container = extension;
        container.kind = ResourceKind::Container;
        assert_eq!(
            resolver
                .validate_saved(&container)
                .await
                .err()
                .unwrap()
                .to_string(),
            "saved source is not an item"
        );
    })
    .await;
}

struct Request {
    method: String,
    target: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

impl Request {
    fn authenticated(&self) {
        // Never print headers, including on failed assertions.
        assert!(self.headers.get("x-emby-token").is_some_and(|v| v == TOKEN));
    }

    fn credentials(&self, username: &str, password: &str) {
        let value: Value = serde_json::from_slice(&self.body).unwrap();
        assert!(value == json!({"Username":username, "Pw":password}));
        assert!(!self.headers.contains_key("x-emby-token"));
        assert!(self
            .headers
            .get("x-emby-authorization")
            .is_some_and(|v| { v.contains(&format!("DeviceId=\"{DEVICE}\"")) }));
    }
}

// Limited to the client's bounded HTTP/1.1 requests. No read can escape the
// surrounding deadline, and malformed or oversized requests fail the fixture.
async fn read_request(stream: &mut TcpStream) -> Request {
    bounded(async {
        let mut bytes = Vec::new();
        let mut buffer = [0; 2048];
        let end = loop {
            if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                break end + 4;
            }
            assert!(bytes.len() < 64 * 1024);
            let count = stream.read(&mut buffer).await.unwrap();
            assert_ne!(count, 0, "request headers were truncated");
            bytes.extend_from_slice(&buffer[..count]);
        };
        let head = std::str::from_utf8(&bytes[..end]).unwrap();
        let mut lines = head.split("\r\n");
        let mut words = lines.next().unwrap().split_ascii_whitespace();
        let method = words.next().unwrap().to_owned();
        let target = words.next().unwrap().to_owned();
        assert_eq!(words.next(), Some("HTTP/1.1"));
        assert!(words.next().is_none());
        let mut headers = BTreeMap::new();
        for line in lines.filter(|line| !line.is_empty()) {
            let (key, value) = line.split_once(':').unwrap();
            assert!(headers
                .insert(key.to_ascii_lowercase(), value.trim().to_owned())
                .is_none());
        }
        assert!(!headers.contains_key("transfer-encoding"));
        let length = headers
            .get("content-length")
            .map(|v| v.parse::<usize>().unwrap())
            .unwrap_or(0);
        assert!(length <= 64 * 1024);
        while bytes.len() < end + length {
            let count = stream.read(&mut buffer).await.unwrap();
            assert_ne!(count, 0, "request body was truncated");
            bytes.extend_from_slice(&buffer[..count]);
        }
        assert_eq!(bytes.len(), end + length);
        Request {
            method,
            target,
            headers,
            body: bytes[end..].to_vec(),
        }
    })
    .await
}

struct Fixture {
    listener: TcpListener,
    requests: Cell<usize>,
}

impl Fixture {
    async fn new() -> Self {
        Self {
            listener: bounded(TcpListener::bind(("127.0.0.1", 0))).await.unwrap(),
            requests: Cell::new(0),
        }
    }

    fn url(&self) -> String {
        format!("http://{}", self.listener.local_addr().unwrap())
    }

    async fn receive(&self, method: &str, path: &str) -> (TcpStream, Request) {
        bounded(async {
            let (mut socket, _) = self.listener.accept().await.unwrap();
            self.requests.set(self.requests.get() + 1);
            let request = read_request(&mut socket).await;
            assert_eq!(request.method, method);
            assert_eq!(request.target.split('?').next(), Some(path));
            (socket, request)
        })
        .await
    }

    async fn json(&self, method: &str, path: &str, body: Value) -> Request {
        let (mut socket, request) = self.receive(method, path).await;
        respond(&mut socket, body).await;
        request
    }

    async fn auth(&self, username: &str, password: &str) {
        self.json("POST", AUTH, authentication(&identity()))
            .await
            .credentials(username, password);
    }

    async fn idle(&self, expected: usize) {
        assert_eq!(self.requests.get(), expected);
        // The caller is already complete. Inspect the accept backlog once;
        // waiting for a guessed quiet interval would hide ordering mistakes.
        let extra = std::future::poll_fn(|cx| {
            Poll::Ready(match self.listener.poll_accept(cx) {
                Poll::Ready(Ok(_)) => true,
                Poll::Ready(Err(_)) => panic!("loopback listener failed"),
                Poll::Pending => false,
            })
        })
        .await;
        assert!(!extra, "unexpected network request");
    }
}

async fn respond(socket: &mut TcpStream, body: Value) {
    bounded(async {
        let body = serde_json::to_vec(&body).unwrap();
        let header = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
        socket.write_all(header.as_bytes()).await.unwrap();
        socket.write_all(&body).await.unwrap();
        socket.shutdown().await.unwrap();
    }).await
}

fn authentication(remote: &RemoteIdentity) -> Value {
    json!({"AccessToken":TOKEN, "ServerId":remote.server_id, "User":{"Id":remote.user_id}})
}

fn episode() -> Value {
    json!({"Id":"55508", "Name":"Synthetic episode", "Type":"Episode",
        "MediaType":"Video", "IsFolder":false, "RunTimeTicks":100_000_000_000_u64})
}

#[tokio::test]
async fn audio_candidates_use_handle_account_and_do_not_create_a_playback_session() {
    bounded(async {
        let db = database().await;
        let first = Fixture::new().await;
        let second = Fixture::new().await;
        let account = save(&db, &first.url(), "family", true).await;
        let _other = save(&db, &second.url(), "other-family", true).await;
        let source = source(&account);
        let resolver = resolver(&db);
        let mut metadata = episode();
        metadata["MediaSources"] = json!([{
            "Id":"version-a", "RunTimeTicks":191429666670_u64, "DefaultAudioStreamIndex":0,
            "MediaStreams":[{"Index":0,"Type":"Audio","Language":"eng","Codec":"aac"}]
        }]);
        let (result, ()) = tokio::join!(resolver.audio_options(&source), async {
            first.auth("family", "").await;
            first.json("GET", ITEM, metadata).await.authenticated();
        });
        let options = result.unwrap();
        assert_eq!(options.source, source.to_handle());
        assert_eq!(options.versions[0].audio_streams[0].index, 0);
        assert_eq!(options.versions[0].runtime_ticks, Some(191429666670));
        first.idle(2).await;
        second.idle(0).await;
    })
    .await;
}

#[tokio::test]
async fn audio_candidates_reject_account_edit_during_auth_before_item_lookup() {
    bounded(async {
        let db = database().await;
        let http = Fixture::new().await;
        let account = save(&db, &http.url(), "family", true).await;
        let source = source(&account);
        let resolver = resolver(&db);
        let (result, ()) = tokio::join!(resolver.audio_options(&source), async {
            let (mut socket, request) = http.receive("POST", AUTH).await;
            request.credentials("family", "");
            // Username is immutable for a saved account; edit its credentials
            // through the supported path while the old authentication is pending.
            saved_servers::upsert(
                db.get_connection(),
                &NewServer::new("emby", "Edited account", http.url())
                    .with_id(Some(account.id.clone()))
                    .with_credentials(Some("family".into()), None)
                    .with_password_update(PasswordUpdate::Set("synthetic-changed-password".into())),
                NOW,
            )
            .await
            .unwrap();
            respond(&mut socket, authentication(&identity())).await;
        });
        assert!(matches!(result, Err(ProviderError::Other(message))
            if message == "saved source account changed during authentication"));
        http.idle(1).await;
    })
    .await;
}

async fn lookup(
    resolver: &SourceResolver,
    source: &SourceRef,
    audio: bool,
) -> Result<(), ProviderError> {
    if audio {
        resolver
            .resolve(source, &AudioSelection::Auto, 0)
            .await
            .map(|_| ())
    } else {
        resolver.track(source).await.map(|_| ())
    }
}

#[tokio::test]
async fn invalid_missing_unbound_and_mismatched_accounts_never_contact_http() {
    bounded(async {
        let db = database().await;
        let http = Fixture::new().await;
        let bound = save(&db, &http.url(), "family", true).await;
        let unbound = save(&db, &http.url(), "unbound", false).await;
        let partial = save(&db, &http.url(), "partial", false).await;
        let wrong_kind = save(&db, &http.url(), "wrong-kind", true).await;
        let deleted = save(&db, &http.url(), "deleted", true).await;
        saved_servers::delete(db.get_connection(), &deleted.id).await.unwrap();
        db.connection.execute_unprepared(
            "UPDATE saved_server SET remote_server_id = 'server-a' WHERE username = 'partial'"
        ).await.unwrap();
        db.connection.execute_unprepared(
            "UPDATE saved_server SET kind = 'jellyfin' WHERE username = 'wrong-kind'"
        ).await.unwrap();
        let valid = source(&bound);
        resolver(&db).validate_saved(&valid).await.unwrap();
        let mut cases = vec![source(&unbound), source(&partial), source(&wrong_kind), source(&deleted)];
        let mut missing = valid.clone();
        missing.account_id = "unknown-account".into();
        cases.push(missing);
        let mut other_user = valid.clone();
        other_user.remote.user_id = "other-user".into();
        cases.push(other_user);
        let mut other_server = valid.clone();
        other_server.remote.server_id = "other-server".into();
        cases.push(other_server);
        let mut empty = valid.clone();
        empty.remote.user_id.clear();
        cases.push(empty);
        let mut container = valid.clone();
        container.kind = ResourceKind::Container;
        cases.push(container);
        let mut unsupported = valid.clone();
        unsupported.resolver = "extension".into();
        cases.push(unsupported);
        for source in cases {
            assert!(resolver(&db).validate_saved(&source).await.is_err());
            assert!(resolver(&db).audio_options(&source).await.is_err());
            for audio in [false, true] {
                assert!(lookup(&resolver(&db), &source, audio).await.is_err());
            }
        }
        // Query failure must not propagate raw SQL/database diagnostics either.
        db.connection.execute_unprepared("DROP TABLE saved_server").await.unwrap();
        assert!(matches!(resolver(&db).validate_saved(&valid).await,
            Err(ProviderError::Other(message)) if message == "saved source account lookup failed"));
        for audio in [false, true] {
            assert!(matches!(lookup(&resolver(&db), &valid, audio).await,
                Err(ProviderError::Other(message)) if message == "saved source account lookup failed"));
        }
        http.idle(0).await;
    }).await;
}

#[tokio::test]
async fn saved_account_a_resolves_its_pin_even_with_account_b_at_the_same_url() {
    bounded(async {
        let db = database().await;
        let http = Fixture::new().await;
        let a = save(&db, &http.url(), "family", true).await;
        let b = save(&db, &http.url(), "guest", false).await;
        saved_servers::bind_remote_identity(
            db.get_connection(),
            &b,
            &RemoteIdentity {
                server_id: "server-a".into(),
                user_id: "user-b".into(),
            },
        )
        .await
        .unwrap();
        let source = source(&a);
        let resolver = SourceResolver::new(db.clone(), DEVICE.into(), false);
        let pin = AudioPin {
            media_source_id: "version-a".into(),
            audio_stream_index: 1,
            runtime_ticks: Some(100_000_000_000),
            etag: Some("revision-a".into()),
            codec: Some("aac".into()),
            channels: Some(2),
            sample_rate: Some(44100),
        };
        let selection = AudioSelection::Pinned(pin.clone());
        let (resolved, ()) =
            tokio::join!(resolver.resolve(&source, &selection, 7_200_123), async {
                http.auth("family", "").await;
                http.json("GET", ITEM, episode()).await.authenticated();
                http.json("GET", "/Items/55508/PlaybackInfo", json!({
                "PlaySessionId":"session-a", "MediaSources":[{
                    "Id":"version-a", "RunTimeTicks":100_000_000_000_u64,
                    "ETag":"revision-a", "DefaultAudioStreamIndex":0,
                    "MediaStreams":[
                        {"Index":0, "Type":"Audio", "Codec":"aac"},
                        {"Index":1, "Type":"Audio", "Codec":"aac", "Channels":2, "SampleRate":44100}
                    ]
                }]
            })).await.authenticated();
            });
        let resolved = resolved.unwrap();
        assert_eq!(resolved.pin, pin);
        assert_eq!(resolved.requested_offset_ms, 7_200_123);
        assert!(!resolved.follow_redirects);
        assert_eq!(resolved.url.path(), "/Audio/55508/stream.mp3");
        let query: BTreeMap<_, _> = resolved.url.query_pairs().into_owned().collect();
        assert_eq!(query.get("UserId").map(String::as_str), Some("user-a"));
        assert_eq!(
            query.get("StartTimeTicks").map(String::as_str),
            Some("72001230000")
        );
        assert_eq!(query.get("AudioStreamIndex").map(String::as_str), Some("1"));
        assert_eq!(
            query.get("EnableAutoStreamCopy").map(String::as_str),
            Some("false")
        );
        assert!(resolved
            .headers
            .get("x-emby-token")
            .is_some_and(|v| v == TOKEN));
        let (released, request) = tokio::join!(
            resolved.lease.release(),
            http.json("DELETE", "/Videos/ActiveEncodings", json!({}))
        );
        released.unwrap();
        request.authenticated();
        assert!(request.target.contains("DeviceId=source-resolver-fixture"));
        assert!(request.target.contains("PlaySessionId=session-a"));
        http.idle(4).await;
    })
    .await;
}

#[tokio::test]
async fn track_lookup_rereads_url_and_credentials_without_changing_the_handle() {
    bounded(async {
        let db = database().await;
        let first = Fixture::new().await;
        let next = Fixture::new().await;
        let account = save(&db, &first.url(), "family", true).await;
        let source = source(&account);
        let resolver = resolver(&db);
        for (http, password) in [(&first, ""), (&next, "synthetic-new-password")] {
            saved_servers::upsert(
                db.get_connection(),
                &NewServer::new("emby", "Renamed account", http.url())
                    .with_id(Some(account.id.clone()))
                    .with_credentials(Some("family".into()), None)
                    .with_password_update(PasswordUpdate::Set(password.into())),
                NOW,
            )
            .await
            .unwrap();
            let (track, ()) = tokio::join!(resolver.track(&source), async {
                http.auth("family", password).await;
                http.json("GET", ITEM, episode()).await.authenticated();
            });
            let track = track.unwrap();
            assert_eq!(track.id, source.to_handle());
            assert_eq!(track.uri, source.to_handle());
            assert_eq!(track.title, "Synthetic episode");
            assert!(track.album.is_none());
            http.idle(2).await;
        }
        first.idle(2).await;
    })
    .await;
}

#[tokio::test]
async fn changed_authenticated_server_or_user_is_rejected_without_rebinding() {
    bounded(async {
        let db = database().await;
        let http = Fixture::new().await;
        let account = save(&db, &http.url(), "family", true).await;
        let source = source(&account);
        let resolver = resolver(&db);
        for (actual, audio) in [
            (
                RemoteIdentity {
                    server_id: "server-b".into(),
                    user_id: "user-a".into(),
                },
                false,
            ),
            (
                RemoteIdentity {
                    server_id: "server-a".into(),
                    user_id: "user-b".into(),
                },
                true,
            ),
        ] {
            let (result, request) = tokio::join!(
                lookup(&resolver, &source, audio),
                http.json("POST", AUTH, authentication(&actual))
            );
            request.credentials("family", "");
            assert!(matches!(result, Err(ProviderError::Other(message))
                if message == "authenticated source identity does not match the saved source"));
        }
        let stored = saved_servers::get(db.get_connection(), &account.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored.remote_server_id, account.remote_server_id);
        assert_eq!(stored.remote_user_id, account.remote_user_id);
        http.idle(2).await;
    })
    .await;
}

#[tokio::test]
async fn edits_or_deletion_while_authentication_is_pending_block_item_and_session_requests() {
    bounded(async {
        let db = database().await;
        for (edit, audio) in [
            ("url", false),
            ("password", true),
            ("clear", false),
            ("delete", true),
        ] {
            let http = Fixture::new().await;
            let replacement = Fixture::new().await;
            let account = save(&db, &http.url(), edit, true).await;
            let source = source(&account);
            let resolver = resolver(&db);
            let (result, ()) = tokio::join!(lookup(&resolver, &source, audio), async {
                // Receiving the request proves the old snapshot was loaded.
                // Hold its response until the conflicting write has committed.
                let (mut socket, request) = http.receive("POST", AUTH).await;
                request.credentials(edit, "");
                if edit == "delete" {
                    assert!(saved_servers::delete(db.get_connection(), &account.id)
                        .await
                        .unwrap());
                } else {
                    let update = match edit {
                        "password" => PasswordUpdate::Set("synthetic-changed-password".into()),
                        "clear" => PasswordUpdate::Clear,
                        _ => PasswordUpdate::Keep,
                    };
                    saved_servers::upsert(
                        db.get_connection(),
                        &NewServer::new(
                            "emby",
                            "Edited during authentication",
                            if edit == "url" {
                                replacement.url()
                            } else {
                                http.url()
                            },
                        )
                        .with_id(Some(account.id.clone()))
                        .with_credentials(Some(edit.into()), None)
                        .with_password_update(update),
                        NOW,
                    )
                    .await
                    .unwrap();
                }
                respond(&mut socket, authentication(&identity())).await;
            });
            assert!(matches!(result, Err(ProviderError::Other(message))
                if message == "saved source account changed during authentication"));
            http.idle(1).await;
            replacement.idle(0).await;
        }
    })
    .await;
}

#[tokio::test]
async fn dropping_resolution_during_authentication_closes_http_and_allows_a_fresh_lookup() {
    bounded(async {
        let db = database().await;
        let http = Fixture::new().await;
        let account = save(&db, &http.url(), "family", true).await;
        let source = source(&account);
        let resolver = resolver(&db);
        let entered = Notify::new();
        tokio::join!(
            async {
                let selection = AudioSelection::Auto;
                let pending = resolver.resolve(&source, &selection, 0);
                tokio::pin!(pending);
                tokio::select! {
                    _ = &mut pending => panic!("resolution completed before the auth response"),
                    _ = entered.notified() => {}
                }
                // Leaving this scope drops the actual caller-owned future.
            },
            async {
                let (mut socket, request) = http.receive("POST", AUTH).await;
                request.credentials("family", "");
                entered.notify_one();
                let mut byte = [0];
                match bounded(socket.read(&mut byte)).await {
                    Ok(0) => {}
                    Err(error)
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::ConnectionReset
                                | std::io::ErrorKind::ConnectionAborted
                        ) => {}
                    _ => panic!("cancelled authentication connection remained open"),
                }
            }
        );
        let (track, ()) = tokio::join!(resolver.track(&source), async {
            http.auth("family", "").await;
            http.json("GET", ITEM, episode()).await.authenticated();
        });
        assert_eq!(track.unwrap().id, source.to_handle());
        http.idle(3).await;
    })
    .await;
}
