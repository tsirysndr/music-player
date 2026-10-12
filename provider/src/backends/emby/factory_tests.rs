//! Real registry-to-HTTP authentication. No decoder or external server is used.
use crate::{
    backends::{builtin_registry, emby::EmbyFactory},
    ProviderConfig, ProviderRegistry,
};
use music_player_types::source::RemoteIdentity;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    future::{poll_fn, Future},
    task::Poll,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

const DEVICE: &str = "factory-fixture-device";
const TOKEN: &str = "synthetic-factory-token";

async fn bounded<T>(future: impl Future<Output = T>) -> T {
    tokio::time::timeout(Duration::from_secs(10), future)
        .await
        .expect("factory fixture exceeded its deadline")
}

struct Request {
    method: String,
    target: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

impl Request {
    fn client_identity(&self, expected: Option<&str>) {
        let auth = self
            .headers
            .get("x-emby-authorization")
            .expect("missing client identity");
        // Never include raw headers in assertion failures.
        assert!(auth.contains("Client=\"music-player\""));
        let device = auth
            .split("DeviceId=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        assert!(!device.is_empty());
        if let Some(expected) = expected {
            assert!(
                device == expected,
                "configured device identity was not forwarded"
            );
        }
    }

    fn empty_query_target(&self, expected: &str) {
        // URL serialization may retain a trailing `?` for an empty query.
        // Check the endpoint and lack of parameters independently.
        let (path, query) = self.target.split_once('?').unwrap_or((&self.target, ""));
        assert_eq!(path, expected);
        assert!(query.is_empty(), "unexpected endpoint query");
    }

    fn empty_password(&self) {
        assert_eq!(self.method, "POST");
        self.empty_query_target("/Users/AuthenticateByName");
        let body: Value = serde_json::from_slice(&self.body).unwrap();
        assert!(body == json!({"Username": "family", "Pw": ""}));
        assert!(!self.headers.contains_key("x-emby-token"));
    }

    fn session(&self) {
        self.client_identity(Some(DEVICE));
        assert!(self
            .headers
            .get("x-emby-token")
            .is_some_and(|value| value == TOKEN));
    }
}

struct Fixture(TcpListener);

impl Fixture {
    async fn new() -> Self {
        Self(TcpListener::bind(("127.0.0.1", 0)).await.unwrap())
    }

    fn base(&self) -> String {
        format!("http://{}", self.0.local_addr().unwrap())
    }

    async fn reply(&self, status: u16, location: Option<&str>, body: Value) -> Request {
        let (mut socket, _) = self.0.accept().await.unwrap();
        let request = read_request(&mut socket).await;
        let body = serde_json::to_vec(&body).unwrap();
        let mut head = format!("HTTP/1.1 {status} Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n", body.len());
        if let Some(location) = location {
            assert!(!location.contains(['\r', '\n']));
            head.push_str(&format!("Location: {location}\r\n"));
        }
        head.push_str("\r\n");
        let mut response = head.into_bytes();
        response.extend_from_slice(&body);
        socket.write_all(&response).await.unwrap();
        socket.shutdown().await.unwrap();
        request
    }

    async fn idle(&self) {
        // The client future is finished and owns no detached work. Inspect the
        // accept backlog once; no sleep-based quiet period or unbounded join.
        poll_fn(|cx| {
            assert!(
                matches!(self.0.poll_accept(cx), Poll::Pending),
                "unexpected connection"
            );
            Poll::Ready(())
        })
        .await;
    }
}

// Bounded by each test deadline and a 16 KiB request limit; HTTP/1.1 only.
async fn read_request(socket: &mut TcpStream) -> Request {
    let mut bytes = Vec::new();
    let mut buffer = [0; 1024];
    let header_end = loop {
        assert!(bytes.len() <= 16 * 1024, "oversized request headers");
        if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
            break end + 4;
        }
        let count = socket.read(&mut buffer).await.unwrap();
        assert_ne!(count, 0, "truncated request headers");
        bytes.extend_from_slice(&buffer[..count]);
    };
    let mut lines = std::str::from_utf8(&bytes[..header_end])
        .unwrap()
        .split("\r\n");
    let mut words = lines.next().unwrap().split_ascii_whitespace();
    let method = words.next().unwrap().to_owned();
    let target = words.next().unwrap().to_owned();
    assert_eq!(words.next(), Some("HTTP/1.1"));
    let mut headers = BTreeMap::new();
    for line in lines.filter(|line| !line.is_empty()) {
        let (name, value) = line.split_once(':').unwrap();
        assert!(headers
            .insert(name.to_ascii_lowercase(), value.trim().to_owned())
            .is_none());
    }
    assert!(!headers.contains_key("transfer-encoding"));
    let length = headers
        .get("content-length")
        .map(|value| value.parse::<usize>().unwrap())
        .unwrap_or(0);
    assert!(length <= 16 * 1024, "oversized request body");
    while bytes.len() < header_end + length {
        let count = socket.read(&mut buffer).await.unwrap();
        assert_ne!(count, 0, "truncated request body");
        bytes.extend_from_slice(&buffer[..count]);
    }
    assert_eq!(bytes.len(), header_end + length);
    Request {
        method,
        target,
        headers,
        body: bytes[header_end..].to_vec(),
    }
}

fn config(fixture: &Fixture) -> ProviderConfig {
    ProviderConfig {
        id: "saved-family-account".into(),
        ..ProviderConfig::new("emby", "Family", &fixture.base())
            .with_credentials(Some("family".into()), Some(String::new()))
    }
}

fn registry(follow_redirects: bool) -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();
    registry.register(EmbyFactory::new(DEVICE.into(), follow_redirects));
    registry
}

#[tokio::test]
async fn builtin_form_descriptor_connects_an_empty_password_account() {
    bounded(async {
        let registry = builtin_registry();
        let entries: Vec<_> = registry
            .describe()
            .into_iter()
            .filter(|entry| entry.kind == "emby")
            .collect();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].display_name, "Emby");
        assert_eq!(entries[0].default_port, 8096);
        assert!(entries[0].needs_credentials);
        assert!(entries[0].fixed_url.is_none());
        let fixture = Fixture::new().await;
        let config = config(&fixture);
        let (result, request) = tokio::join!(
            registry.connect(&config),
            fixture.reply(
                200,
                None,
                json!({"AccessToken": TOKEN, "User": {"Id": "user-a"}, "ServerId": "server-a"})
            ),
        );
        request.empty_password();
        request.client_identity(None);
        let provider = result.unwrap();
        assert_eq!(provider.kind(), "emby");
        assert_eq!(
            provider.remote_identity(),
            Some(RemoteIdentity {
                server_id: "server-a".into(),
                user_id: "user-a".into()
            })
        );
        fixture.idle().await;
    })
    .await;
}

#[tokio::test]
async fn factory_forwards_device_and_token_through_cross_origin_identity_redirect() {
    bounded(async {
        let origin = Fixture::new().await;
        let target = Fixture::new().await;
        let config = config(&origin);
        let registry = registry(true);
        let location = format!("{}/identity?signature=a%2Fb%2Bc&part=1", target.base());
        let (result, ()) = tokio::join!(registry.connect(&config), async {
            let request = origin
                .reply(
                    200,
                    None,
                    json!({"AccessToken": TOKEN, "User": {"Id": "user-a"}}),
                )
                .await;
            request.empty_password();
            request.client_identity(Some(DEVICE));
            let request = origin.reply(302, Some(&location), Value::Null).await;
            assert_eq!(request.method, "GET");
            request.empty_query_target("/System/Info");
            request.session();
            let request = target.reply(200, None, json!({"Id": "server-a"})).await;
            assert_eq!(request.method, "GET");
            assert_eq!(request.target, "/identity?signature=a%2Fb%2Bc&part=1");
            request.session();
        });
        assert_eq!(
            result.unwrap().remote_identity(),
            Some(RemoteIdentity {
                server_id: "server-a".into(),
                user_id: "user-a".into()
            })
        );
        origin.idle().await;
        target.idle().await;
    })
    .await;
}

#[tokio::test]
async fn factory_redirect_opt_out_fails_connection_without_contacting_target() {
    bounded(async {
        let origin = Fixture::new().await;
        let target = Fixture::new().await;
        let config = config(&origin);
        let registry = registry(false);
        let location = format!("{}/identity", target.base());
        let (result, ()) = tokio::join!(registry.connect(&config), async {
            let request = origin
                .reply(
                    200,
                    None,
                    json!({"AccessToken": TOKEN, "User": {"Id": "user-a"}}),
                )
                .await;
            request.empty_password();
            request.client_identity(Some(DEVICE));
            let request = origin.reply(302, Some(&location), Value::Null).await;
            request.empty_query_target("/System/Info");
            request.session();
        });
        let error = result
            .err()
            .expect("disabled redirect must fail authentication");
        assert!(error.to_string().contains("redirects are disabled"));
        origin.idle().await;
        target.idle().await;
    })
    .await;
}
