//! CI-only v1 WAT -> saved local handle -> SourcePlayback -> HTTP MP3 -> TCP PCM.
//! No fake descriptor/session and no generated audio. TCP delivery is not device
//! consumption; the v1 lease releases a local reservation, not a remote session.
use super::*;
use crate::{
    extension_accounts::{AccountError, ExtensionAccounts},
    source_resolver::LegacySourceList,
};
use futures_util::FutureExt;
use migration::{Migrator, MigratorTrait};
use music_player_extensions::{abi, host::HostContext};
use music_player_storage::Database;
use rockbox_playback::{PlayerConfig, StreamOutputBoundary};
use serde_json::json;
use std::{
    collections::BTreeMap,
    panic::AssertUnwindSafe,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::watch,
};

mod v2;

const AUDIO: &[u8] = include_bytes!("../../tests/fixtures/tone-2s.mp3");
const PLUGIN: &str = "legacy.http.pcm.fixture";
const AUTH: &str = "Bearer synthetic-v1-http-fixture";
const GUEST_URI: &str = "https://guest.invalid/temporary?token=never-use-this";
// Reuse the existing MP3 bytes to supply bounded codec lookahead without EOF.
// No terminal HTTP chunk is sent: stop must cancel a still-open input.
const AUDIO_COPIES: usize = 3;

struct Directory(PathBuf);
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn emit_json(body: &str) -> String {
    let mut code = format!("(local.set $p (call $alloc (i64.const {})))", body.len());
    for (offset, byte) in body.bytes().enumerate() {
        code.push_str(&format!(
            "(call $store (i64.add (local.get $p) (i64.const {offset})) (i32.const {byte}))"
        ));
    }
    code.push_str(&format!(
        "(call $output (local.get $p) (i64.const {})) (return (i32.const 0))",
        body.len()
    ));
    code
}

// Compare the actual serialized input inside the guest. A wrong item ID or
// browse payload traps rather than accidentally returning the positive body.
fn matcher(name: &str, input: &str) -> String {
    let mut code = format!(
        "(func ${name} (result i32) (if (i64.ne (call $input_length) (i64.const {})) (then (return (i32.const 0))))",
        input.len()
    );
    for (offset, byte) in input.bytes().enumerate() {
        code.push_str(&format!(
            "(if (i32.ne (call $input (i64.const {offset})) (i32.const {byte})) (then (return (i32.const 0))))"
        ));
    }
    code.push_str("(i32.const 1))");
    code
}

fn install_plugin(url: &str) -> Directory {
    let directory =
        Directory(std::env::temp_dir().join(format!("mp-v1-http-pcm-{}", uuid::Uuid::new_v4())));
    let plugin = directory.0.join("plugin");
    std::fs::create_dir_all(&plugin).unwrap();
    // Extism's locked kernel uses i64 pointers/lengths and i32 byte values.
    let mut wat = String::from(
        r#"(module
          (import "extism:host/env" "alloc" (func $alloc (param i64) (result i64)))
          (import "extism:host/env" "store_u8" (func $store (param i64 i32)))
          (import "extism:host/env" "output_set" (func $output (param i64 i64)))
          (import "extism:host/env" "input_length" (func $input_length (result i64)))
          (import "extism:host/env" "input_load_u8" (func $input (param i64) (result i32)))
          (global $resolves (mut i32) (i32.const 0))
        "#,
    );
    let info = r#"{"name":"Legacy HTTP fixture","description":"0"}"#;
    let info_code = emit_json(info);
    let at = info_code.find("(call $output").unwrap();
    let digit = info.rfind('0').unwrap();
    wat.push_str("(func (export \"source_info\") (result i32) (local $p i64)");
    wat.push_str(&info_code[..at]);
    wat.push_str(&format!(
        "(call $store (i64.add (local.get $p) (i64.const {digit})) (i32.add (global.get $resolves) (i32.const 48)))"
    ));
    wat.push_str(&info_code[at..]);
    wat.push(')');
    let browse = abi::BrowseRequest {
        parent_id: String::new(),
        query: String::new(),
        offset: 0,
        limit: 2,
    };
    wat.push_str(&matcher("browse", &serde_json::to_string(&browse).unwrap()));
    wat.push_str(
        r#"(func (export "list_tracks") (result i32) (local $p i64)
          (if (i32.eqz (call $browse)) (then unreachable))"#,
    );
    wat.push_str(&emit_json(
        &json!([{
            "id":"tone", "title":"Synthetic MP3 fixture", "artist":"CI fixture",
            "uri":GUEST_URI
        }])
        .to_string(),
    ));
    wat.push(')');
    for export in ["list_albums", "list_artists", "list_playlists"] {
        wat.push_str(&format!(
            "(func (export \"{export}\") (result i32) (local $p i64) {})",
            emit_json("[]")
        ));
    }
    let stream = abi::StreamRequest {
        track_id: "tone".into(),
    };
    wat.push_str(&matcher("stream", &serde_json::to_string(&stream).unwrap()));
    wat.push_str(
        r#"(func (export "get_stream_url") (result i32) (local $p i64)
          (global.set $resolves (i32.add (global.get $resolves) (i32.const 1)))
          (if (i32.ne (global.get $resolves) (i32.const 1)) (then unreachable))
          (if (i32.eqz (call $stream)) (then unreachable))"#,
    );
    wat.push_str(&emit_json(
        &json!({
            "url":url, "mimeType":"audio/mpeg",
            "headers":json!({"Authorization":AUTH}).to_string()
        })
        .to_string(),
    ));
    wat.push_str("))");
    std::fs::write(plugin.join("plugin.wat"), wat).unwrap();
    std::fs::write(
        plugin.join("plugin.json"),
        json!({
            "id":PLUGIN, "name":"Legacy HTTP fixture", "entry":"plugin.wat",
            "capabilities":["source"], "sourceApiVersion":1
        })
        .to_string(),
    )
    .unwrap();
    directory
}

struct Fixture {
    accounts: ExtensionAccounts,
    resolver: SourceResolver,
    account_id: String,
    _directory: Directory,
}
impl Fixture {
    async fn new(url: &str) -> Self {
        let directory = install_plugin(url);
        let mut options = sea_orm::ConnectOptions::new("sqlite::memory:");
        options.max_connections(1).sqlx_logging(false);
        let connection = sea_orm::Database::connect(options).await.unwrap();
        Migrator::up(&connection, None).await.unwrap();
        let db = Database { connection };
        let accounts = ExtensionAccounts::new(
            db.clone(),
            vec![directory.0.clone()],
            HostContext::default(),
        );
        // Only admission Busy is retried, under the setup watchdog. It precedes
        // the storage mutation. Never retry a guest failure or playback resolve.
        let account = loop {
            match accounts
                .create(PLUGIN, "PCM fixture", BTreeMap::new(), "t0")
                .await
            {
                Err(AccountError::Busy) => tokio::task::yield_now().await,
                result => break result.unwrap(),
            }
        };
        let bound = loop {
            match accounts.connect(&account.id).await {
                Err(AccountError::Busy) => tokio::task::yield_now().await,
                result => break result.unwrap(),
            }
        };
        assert!(bound.remote_server_id.is_none() && bound.remote_user_id.is_none());
        assert_eq!(bound.local_bound_revision, Some(bound.config_revision));
        let resolver = SourceResolver::new(db, "v1-pcm-fixture".into(), false)
            .with_extensions(accounts.clone(), vec!["mp3".into()]);
        Self {
            accounts,
            resolver,
            account_id: bound.id,
            _directory: directory,
        }
    }

    async fn resolve_count(&self) -> String {
        self.accounts
            .for_account(&self.account_id)
            .await
            .unwrap()
            .legacy_info()
            .await
            .unwrap()
            .description
    }

    fn reserved(&self) -> usize {
        self.accounts
            .status()
            .iter()
            .filter(|entry| entry.account_id == self.account_id)
            .map(|entry| entry.runtime.reserved)
            .sum()
    }
}

#[derive(Default)]
struct HttpObservation {
    requests: usize,
    required_header: bool,
    body_sent: bool,
    peer_closed: bool,
}

async fn serve_connection(
    mut socket: TcpStream,
    observed: &Mutex<HttpObservation>,
) -> Result<(), &'static str> {
    let mut bytes = Vec::new();
    let mut buffer = [0; 2048];
    let end = loop {
        if let Some(end) = bytes.windows(4).position(|p| p == b"\r\n\r\n") {
            break end + 4;
        }
        if bytes.len() >= 16 * 1024 {
            return Err("oversized HTTP fixture request");
        }
        let count = socket.read(&mut buffer).await.map_err(|_| "read request")?;
        if count == 0 {
            return Err("incomplete HTTP fixture request");
        }
        bytes.extend_from_slice(&buffer[..count]);
    };
    let headers = std::str::from_utf8(&bytes[..end]).map_err(|_| "request encoding")?;
    if headers.lines().next() != Some("GET /stream HTTP/1.1") {
        return Err("unexpected HTTP fixture method or target");
    }
    let auth: Vec<_> = headers
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once(':')?;
            key.eq_ignore_ascii_case("authorization")
                .then_some(value.trim())
        })
        .collect();
    if auth != [AUTH] {
        return Err("guest required header was not forwarded exactly once");
    }
    observed.lock().unwrap().required_header = true;
    socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: audio/mpeg\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n")
        .await.map_err(|_| "write response headers")?;
    for _ in 0..AUDIO_COPIES {
        socket
            .write_all(format!("{:X}\r\n", AUDIO.len()).as_bytes())
            .await
            .map_err(|_| "write chunk size")?;
        socket
            .write_all(AUDIO)
            .await
            .map_err(|_| "write MP3 fixture")?;
        socket
            .write_all(b"\r\n")
            .await
            .map_err(|_| "write chunk end")?;
    }
    observed.lock().unwrap().body_sent = true;
    // Deliberately withhold the terminal chunk. No EOF can release the input
    // on our behalf; the client's cancellation must close this connection.
    match socket.read(&mut buffer[..1]).await {
        Ok(0) => {}
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted
            ) => {}
        _ => return Err("expected cancellation of the HTTP body"),
    }
    observed.lock().unwrap().peer_closed = true;
    Ok(())
}

async fn serve_http(
    listener: TcpListener,
    observed: Arc<Mutex<HttpObservation>>,
    mut shutdown: watch::Receiver<bool>,
) -> Result<(), &'static str> {
    let socket = tokio::select! {
        result = listener.accept() => result.map_err(|_| "accept HTTP")?.0,
        _ = shutdown.changed() => return Err("HTTP fixture stopped without a request"),
    };
    observed.lock().unwrap().requests += 1;
    let connection = serve_connection(socket, &observed);
    tokio::pin!(connection);
    let mut closed = false;
    loop {
        tokio::select! {
            biased;
            result = listener.accept() => {
                let _extra = result.map_err(|_| "accept extra HTTP")?;
                observed.lock().unwrap().requests += 1;
                return Err("unexpected second HTTP request (no playback retries allowed)");
            }
            result = &mut connection, if !closed => { result?; closed = true; }
            _ = shutdown.changed() => {
                return if closed { Ok(()) } else { Err("HTTP input did not close before teardown") };
            }
        }
    }
}

async fn collect_pcm(
    listener: TcpListener,
    received: Arc<Mutex<Vec<u8>>>,
) -> Result<(), &'static str> {
    let (mut socket, _) = listener.accept().await.map_err(|_| "accept PCM")?;
    let mut buffer = [0; 8192];
    loop {
        let count = socket.read(&mut buffer).await.map_err(|_| "read PCM")?;
        if count == 0 {
            return Ok(());
        }
        let mut bytes = received.lock().unwrap();
        if bytes.len() + count > 8_000_000 {
            return Err("PCM fixture exceeded its byte budget");
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
}

// Join even a failed/timed-out scenario's fixtures. Forced abort is only failure
// cleanup and must return Err, never count as evidence of product cancellation.
async fn join_fixture<T>(mut task: JoinHandle<T>) -> Result<T, &'static str> {
    match tokio::time::timeout(Duration::from_secs(5), &mut task).await {
        Ok(result) => result.map_err(|_| "fixture task failed"),
        Err(_) => {
            task.abort();
            let _ = tokio::time::timeout(Duration::from_secs(5), task).await;
            Err("fixture task did not join without abort")
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn legacy_extension_stream_reaches_tcp_pcm_and_stop_releases_its_local_lease() {
    let _serial = crate::extension_accounts::TEST_LOAD_SERIAL.lock().await;
    let http = TcpListener::bind("127.0.0.1:0").await.unwrap();
    // Extension MIME supplies the decoder format; this URL has no suffix.
    let url = format!("http://{}/stream", http.local_addr().unwrap());
    let fixture = tokio::time::timeout(Duration::from_secs(30), Fixture::new(&url))
        .await
        .expect("legacy HTTP setup watchdog");
    let pcm = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let output = format!("tcp-connect:{}", pcm.local_addr().unwrap())
        .parse()
        .unwrap();
    let observed = Arc::new(Mutex::new(HttpObservation::default()));
    let received = Arc::new(Mutex::new(Vec::new()));
    let (shutdown, shutdown_rx) = watch::channel(false);
    let http_state = observed.clone();
    let http_task = tokio::spawn(async move {
        tokio::time::timeout(
            Duration::from_secs(40),
            serve_http(http, http_state, shutdown_rx),
        )
        .await
        .map_err(|_| "HTTP fixture watchdog")?
    });
    let pcm_state = received.clone();
    let pcm_task = tokio::spawn(async move {
        tokio::time::timeout(Duration::from_secs(40), collect_pcm(pcm, pcm_state))
            .await
            .map_err(|_| "PCM fixture watchdog")?
    });
    // Keep resource owners outside the deadline/catch boundary so assertions
    // and watchdog failures still run stop, engine Drop and fixture joins.
    let mut engine = None;
    let mut player = None;
    let scenario = tokio::time::timeout(
        Duration::from_secs(30),
        AssertUnwindSafe(async {
            let page = fixture
                .resolver
                .extension_legacy_media(
                    &fixture.account_id,
                    LegacySourceList::Tracks,
                    None,
                    None,
                    0,
                    2,
                )
                .await
                .unwrap();
            assert_eq!(page.items.len(), 1);
            assert!(page.next_offset.is_none());
            let item = &page.items[0];
            assert!(!item.is_container);
            let source = SourceHandle::parse(&item.id).unwrap();
            let SourceHandle::Local(local) = &source else {
                panic!("v1 must browse a local handle")
            };
            assert_eq!(local.extension_id(), Some(PLUGIN));
            assert_eq!(local.account_id, fixture.account_id);
            assert_eq!(local.kind, ResourceKind::Item);
            assert_eq!(local.item_id, "tone");
            let track = item.track.as_ref().unwrap();
            assert_eq!(track.id, source.to_handle());
            assert_eq!(track.uri, track.id);
            assert_ne!(track.uri, GUEST_URI);
            assert!(!track.uri.contains(&url));
            assert_eq!(fixture.resolve_count().await, "0");

            let engine = engine.insert(
                Engine::with_config(PlayerConfig {
                    output,
                    sample_rate: Some(44100),
                    buffer_seconds: 1.0,
                    ..Default::default()
                })
                .unwrap(),
            );
            let player = player.insert(SourcePlayback::new(fixture.resolver.clone()));
            player.load(source.clone(), Duration::ZERO, false).unwrap();
            player.poll(engine);
            assert_eq!(player.snapshot().phase, Phase::Paused);
            assert_eq!(fixture.resolve_count().await, "0");
            assert_eq!(observed.lock().unwrap().requests, 0);
            player.play().unwrap();
            // This is the normal host polling cadence, not a sleep-based race. The
            // pass condition includes actual nonzero bytes read from the PCM socket.
            let mut tick = tokio::time::interval(Duration::from_millis(10));
            loop {
                tick.tick().await;
                player.poll(engine);
                assert_ne!(
                    player.snapshot().phase,
                    Phase::Failed,
                    "{:?}",
                    player.error()
                );
                if player
                    .snapshot()
                    .position
                    .is_some_and(|p| p.absolute > Duration::ZERO)
                    && received.lock().unwrap().iter().any(|byte| *byte != 0)
                    && observed.lock().unwrap().body_sent
                {
                    break;
                }
            }
            let snapshot = player.snapshot();
            assert_eq!(snapshot.phase, Phase::Streaming);
            let position = snapshot.position.unwrap();
            assert_eq!(position.boundary, StreamOutputBoundary::ByteStream);
            assert_eq!(position.start, StartPosition::RequestedOnly);
            assert!(position.absolute > Duration::ZERO);
            assert_eq!(
                snapshot.capabilities,
                Some(PlaybackCapabilities {
                    timeline: Timeline::Unknown,
                    seek_policy: SeekPolicy::None,
                })
            );
            assert!(player.accepted_pin().is_none());
            let checkpoint = player.checkpoint().unwrap();
            assert_eq!(checkpoint.source, source.to_handle());
            assert_eq!(checkpoint.selection, AudioSelection::Auto);
            assert_eq!(fixture.resolve_count().await, "1");
            {
                let http = observed.lock().unwrap();
                assert_eq!(http.requests, 1);
                assert!(http.required_header && !http.peer_closed);
            }
            assert_eq!(
                fixture.reserved(),
                1,
                "input/lease must still be alive before stop"
            );
            // The resolve has finished. Join its task while leaving its ownership
            // in SourcePlayback so normal stop still uses its production path.
            if let Some(task) = player.task.as_mut() {
                tokio::time::timeout(Duration::from_secs(5), task)
                    .await
                    .expect("resolve task join deadline")
                    .unwrap();
            }
            player.stop();
            assert_eq!(player.snapshot().phase, Phase::Stopped);
            assert_eq!(
                fixture.reserved(),
                0,
                "stop must release the v1 local lease"
            );
            // Check client-side HTTP cancellation before engine Drop or fixture
            // shutdown can close anything on its behalf.
            tokio::time::timeout(Duration::from_secs(5), async {
                while !observed.lock().unwrap().peer_closed {
                    tick.tick().await;
                }
            })
            .await
            .expect("stop did not cancel the open HTTP input");
            assert_eq!(fixture.resolve_count().await, "1");
        })
        .catch_unwind(),
    )
    .await;

    // On failure, retain the resolution JoinHandle before stop takes it, solely
    // to join teardown. On success production stop already consumed that handle.
    let abandoned = player.as_mut().and_then(|player| player.task.take());
    if let Some(task) = &abandoned {
        task.abort();
    }
    if let Some(player) = player.as_mut() {
        player.stop();
    }
    drop(player.take());
    if let Some(engine) = engine.as_ref() {
        engine.stop();
    }
    drop(engine.take()); // public engine Drop joins its engine/decoder ownership
    if let Some(task) = abandoned {
        let _ = join_fixture(task).await; // cancellation is expected only on failure
    }
    shutdown.send_replace(true);
    let (http_result, pcm_result) = tokio::join!(join_fixture(http_task), join_fixture(pcm_task));
    // All owned tasks have been awaited before propagating any test failure.
    match scenario {
        Ok(Ok(())) => {}
        Ok(Err(panic)) => std::panic::resume_unwind(panic),
        Err(_) => panic!("extension-to-PCM scenario watchdog; no audio success inferred"),
    }
    http_result.unwrap().unwrap();
    pcm_result.unwrap().unwrap();
    assert_eq!(observed.lock().unwrap().requests, 1);
    assert!(received.lock().unwrap().iter().any(|byte| *byte != 0));
    assert_eq!(fixture.reserved(), 0);
}
