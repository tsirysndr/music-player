//! Actual v2 guest -> shared manager -> Player -> HTTP -> TCP delivery.
//! Synthetic offsets exercise host mapping, not Emby encoder calibration.
use super::*;
use music_player_extensions::abi::source_v2 as wire;
use music_player_extensions::host::LibraryAccess;

const V2_PLUGIN: &str = "v2.http.pcm.fixture";
const OFFSET: u64 = 7_200_123;
const TICKS: u64 = 191_429_666_670;
const REQUEST_ID: &str = "00000000-0000-0000-0000-000000000000";

// Observe acknowledgements from inside the actual guest. The existing host
// callback is only a test event sink; it never supplies playback descriptors.
#[derive(Default)]
struct ReleaseObserver(Mutex<Vec<String>>);
impl LibraryAccess for ReleaseObserver {
    fn query(&self, filter: &str, _limit: u32) -> Result<Vec<abi::TrackInfo>, anyhow::Error> {
        self.0.lock().unwrap().push(filter.to_owned());
        Ok(Vec::new())
    }
}

fn identity() -> wire::SourceIdentity {
    wire::SourceIdentity {
        server_id: "fixture-server".into(),
        user_id: "family".into(),
    }
}
fn pin() -> wire::AudioPin {
    wire::AudioPin {
        media_source_id: "mediasource_55508".into(),
        audio_stream_index: 1,
        runtime_ticks: Some(TICKS),
        etag: Some("fixture-revision".into()),
        codec: Some("aac".into()),
        channels: Some(2),
        sample_rate: Some(44100),
    }
}

// Match the complete typed JSON request except its fresh UUID. Persist each
// UUID in guest globals so cleanup must name the original attempt and lease.
fn match_attempt(name: &str, input: &str) -> (String, usize) {
    let start = input.find(REQUEST_ID).unwrap();
    let mut code = format!("(func ${name} (result i32) (if (i64.ne (call $input_length) (i64.const {})) (then (return (i32.const 0))))", input.len());
    for (offset, byte) in input.bytes().enumerate() {
        if !(start..start + REQUEST_ID.len()).contains(&offset) {
            code.push_str(&format!("(if (i32.ne (call $input (i64.const {offset})) (i32.const {byte})) (then (return (i32.const 0))))"));
        }
    }
    code.push_str("(i32.const 1))");
    (code, start)
}
fn reply_attempt(body: &str, attempt: usize) -> String {
    let code = emit_json(body);
    let at = code.find("(call $output").unwrap();
    let start = body.find(REQUEST_ID).unwrap();
    let mut patched = code[..at].to_string();
    for byte in 0..REQUEST_ID.len() {
        patched.push_str(&format!("(call $store (i64.add (local.get $p) (i64.const {})) (global.get $id{attempt}_{byte}))", start + byte));
    }
    patched.push_str(&code[at..]);
    patched
}

fn install_v2(urls: &[String; 2]) -> Directory {
    let directory =
        Directory(std::env::temp_dir().join(format!("mp-v2-http-pcm-{}", uuid::Uuid::new_v4())));
    let plugin = directory.0.join("plugin");
    std::fs::create_dir_all(&plugin).unwrap();
    let mut wat = String::from(
        r#"(module
      (import "extism:host/env" "alloc" (func $alloc (param i64) (result i64)))
      (import "extism:host/env" "store_u8" (func $store (param i64 i32)))
      (import "extism:host/env" "output_set" (func $output (param i64 i64)))
      (import "extism:host/env" "input_length" (func $input_length (result i64)))
      (import "extism:host/env" "input_load_u8" (func $input (param i64) (result i32)))
      (import "extism:host/user" "query_library" (func $observe (param i64) (result i64)))
      (global $resolves (mut i32) (i32.const 0))
    "#,
    );
    let item = json!({
        "itemId":"55508", "title":"Synthetic Episode PCM fixture",
        "itemType":"Episode", "mediaType":"Video", "isContainer":false,
        "playable":true, "seasonNumber":2025050473u64, "episodeNumber":41,
        "runtimeTicks":TICKS
    });
    for (export, body) in [
        (
            "source_identity_v2",
            serde_json::to_string(&identity()).unwrap(),
        ),
        (
            "browse_media_v2",
            json!({"items":[item.clone()],"nextOffset":null}).to_string(),
        ),
        ("get_media_item_v2", json!({"item":item}).to_string()),
        (
            "get_audio_options_v2",
            json!({"itemId":"55508","versions":[]}).to_string(),
        ),
    ] {
        wat.push_str(&format!(
            "(func (export \"{export}\") (result i32) (local $p i64) {})",
            emit_json(&body)
        ));
    }
    for (attempt, url) in urls.iter().enumerate() {
        wat.push_str(&format!(
            "(global $released{attempt} (mut i32) (i32.const 0))"
        ));
        for byte in 0..REQUEST_ID.len() {
            wat.push_str(&format!(
                "(global $id{attempt}_{byte} (mut i32) (i32.const 0))"
            ));
        }
        let offset = if attempt == 0 { 0 } else { OFFSET };
        let request = wire::ResolveRequest {
            request_id: REQUEST_ID.into(),
            identity: identity(),
            item_id: "55508".into(),
            offset_ms: offset,
            selection: if attempt == 0 {
                wire::AudioSelection::Auto {}
            } else {
                wire::AudioSelection::Pinned { pin: pin() }
            },
            format_exts: vec!["mp3".into()],
            follow_redirects: false,
        };
        let (check, start) = match_attempt(
            &format!("resolve{attempt}"),
            &serde_json::to_string(&request).unwrap(),
        );
        wat.push_str(&check);
        wat.push_str(&format!(
            "(func $response{attempt} (result i32) (local $p i64)
          (if (i32.eqz (call $resolve{attempt})) (then unreachable))
          (global.set $resolves (i32.add (global.get $resolves) (i32.const 1)))"
        ));
        for byte in 0..REQUEST_ID.len() {
            wat.push_str(&format!(
                "(global.set $id{attempt}_{byte} (call $input (i64.const {})))",
                start + byte
            ));
        }
        let response = json!({
            "requestId":REQUEST_ID, "requestedOffsetMs":offset, "url":url,
            "headers":[{"name":"Authorization","value":AUTH}],
            "mimeType":"audio/mpeg", "formatExt":"mp3", "actualPin":pin(),
            "start":{"type":"calibrated","offsetMs":offset,"maxErrorMs":0},
            "timeline":{"type":"finite","durationTicks":TICKS},
            "seekPolicy":"serverOffset", "cachePolicy":"noPersistentCache",
            "leaseId":format!("lease-{attempt}")
        });
        wat.push_str(&reply_attempt(&response.to_string(), attempt));
        wat.push(')');
        let release = wire::ReleaseRequest {
            request_id: REQUEST_ID.into(),
            identity: identity(),
            item_id: "55508".into(),
            lease_id: Some(format!("lease-{attempt}")),
        };
        let (check, start) = match_attempt(
            &format!("release{attempt}"),
            &serde_json::to_string(&release).unwrap(),
        );
        wat.push_str(&check);
        wat.push_str(&format!(
            "(func $ack{attempt} (result i32) (local $p i64)
          (if (global.get $released{attempt}) (then unreachable))
          (if (i32.le_u (global.get $resolves) (i32.const {attempt})) (then unreachable))"
        ));
        for byte in 0..REQUEST_ID.len() {
            wat.push_str(&format!("(if (i32.ne (call $input (i64.const {})) (global.get $id{attempt}_{byte})) (then unreachable))", start + byte));
        }
        wat.push_str(&format!("(global.set $released{attempt} (i32.const 1))"));
        let event =
            emit_json(&json!({"filter":format!("released:{attempt}"),"limit":1}).to_string());
        wat.push_str(&event[..event.find("(call $output").unwrap()]);
        wat.push_str("(drop (call $observe (local.get $p)))");
        wat.push_str(&reply_attempt(
            &json!({"requestId":REQUEST_ID,"released":true}).to_string(),
            attempt,
        ));
        wat.push(')');
    }
    wat.push_str(
        r#"
      (func (export "resolve_audio_v2") (result i32)
        (if (i32.eq (global.get $resolves) (i32.const 0)) (then (return (call $response0))))
        (if (i32.eq (global.get $resolves) (i32.const 1)) (then (return (call $response1))))
        unreachable)
      (func (export "release_audio_v2") (result i32)
        (if (call $release0) (then (return (call $ack0))))
        (if (call $release1) (then (return (call $ack1))))
        unreachable))
    "#,
    );
    std::fs::write(plugin.join("plugin.wat"), wat).unwrap();
    std::fs::write(
        plugin.join("plugin.json"),
        json!({
            "id":V2_PLUGIN, "name":"V2 HTTP fixture", "entry":"plugin.wat",
            "capabilities":["source"], "sourceApiVersion":2, "permissions":{"libraryRead":true}
        })
        .to_string(),
    )
    .unwrap();
    directory
}

struct V2Fixture {
    accounts: ExtensionAccounts,
    resolver: SourceResolver,
    account_id: String,
    releases: Arc<ReleaseObserver>,
    _directory: Directory,
}
impl V2Fixture {
    async fn new(urls: &[String; 2]) -> Self {
        let directory = install_v2(urls);
        let mut options = sea_orm::ConnectOptions::new("sqlite::memory:");
        options.max_connections(1).sqlx_logging(false);
        let connection = sea_orm::Database::connect(options).await.unwrap();
        Migrator::up(&connection, None).await.unwrap();
        let db = Database { connection };
        let releases = Arc::new(ReleaseObserver::default());
        let accounts = ExtensionAccounts::new(
            db.clone(),
            vec![directory.0.clone()],
            HostContext {
                library: Some(releases.clone()),
            },
        );
        let account = loop {
            match accounts
                .create(V2_PLUGIN, "V2 PCM fixture", BTreeMap::new(), "t0")
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
        assert_eq!(bound.remote_server_id.as_deref(), Some("fixture-server"));
        assert_eq!(bound.remote_user_id.as_deref(), Some("family"));
        let resolver = SourceResolver::new(db, "v2-pcm-fixture".into(), false)
            .with_extensions(accounts.clone(), vec!["mp3".into()]);
        Self {
            accounts,
            resolver,
            account_id: bound.id,
            releases,
            _directory: directory,
        }
    }

    fn reserved(&self) -> usize {
        let states = self.accounts.status();
        assert!(states
            .iter()
            .all(|entry| entry.runtime.failed_cleanup == 0 && !entry.runtime.quarantined));
        states
            .iter()
            .filter(|entry| entry.account_id == self.account_id)
            .map(|entry| entry.runtime.reserved)
            .sum()
    }
}

async fn wait_pcm(
    player: &mut SourcePlayback,
    engine: &Engine,
    observed: &Mutex<HttpObservation>,
    received: &Mutex<Vec<u8>>,
    after: usize,
    offset: u64,
) {
    let mut tick = tokio::time::interval(Duration::from_millis(10));
    loop {
        tick.tick().await;
        player.poll(engine);
        let snapshot = player.snapshot();
        assert_ne!(snapshot.phase, Phase::Failed, "{:?}", player.error());
        if snapshot
            .position
            .is_some_and(|p| p.absolute > Duration::from_millis(offset))
            && received.lock().unwrap()[after..]
                .iter()
                .any(|byte| *byte != 0)
            && observed.lock().unwrap().body_sent
        {
            break;
        }
    }
    let snapshot = player.snapshot();
    assert_eq!(snapshot.phase, Phase::Streaming);
    assert_eq!(
        snapshot.capabilities,
        Some(PlaybackCapabilities {
            timeline: Timeline::Finite {
                duration_ticks: TICKS
            },
            seek_policy: SeekPolicy::ServerOffset,
        })
    );
    let position = snapshot.position.unwrap();
    assert_eq!(position.boundary, StreamOutputBoundary::ByteStream);
    assert_eq!(
        position.start,
        StartPosition::Calibrated {
            offset: Duration::from_millis(offset),
            max_error: Duration::ZERO
        }
    );
    assert!(position.absolute < Duration::from_millis(offset + 10_000));
    let state = observed.lock().unwrap();
    assert_eq!(state.requests, 1);
    assert!(state.required_header && !state.peer_closed);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn v2_seek_pins_audio_and_disabled_extension_releases_original_sessions() {
    let _serial = crate::extension_accounts::TEST_LOAD_SERIAL.lock().await;
    let first = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let second = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let urls = [
        format!("http://{}/stream", first.local_addr().unwrap()),
        format!("http://{}/stream", second.local_addr().unwrap()),
    ];
    let fixture = tokio::time::timeout(Duration::from_secs(30), V2Fixture::new(&urls))
        .await
        .expect("v2 setup deadline");
    // Retain the original runtime for inspection even after manager invalidation.
    // It does not supply fake descriptors or bypass playback resolution.
    let account_runtime = fixture
        .accounts
        .for_account(&fixture.account_id)
        .await
        .unwrap();
    let pcm = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let output = format!("tcp-connect:{}", pcm.local_addr().unwrap())
        .parse()
        .unwrap();
    let received = Arc::new(Mutex::new(Vec::new()));
    let states = [
        Arc::new(Mutex::new(HttpObservation::default())),
        Arc::new(Mutex::new(HttpObservation::default())),
    ];
    let (shutdown, shutdown_rx) = watch::channel(false);
    let mut http_tasks = Vec::new();
    for (listener, state) in [first, second].into_iter().zip(states.iter().cloned()) {
        let shutdown = shutdown_rx.clone();
        http_tasks.push(tokio::spawn(async move {
            tokio::time::timeout(
                Duration::from_secs(40),
                serve_http(listener, state, shutdown),
            )
            .await
            .map_err(|_| "HTTP watchdog")?
        }));
    }
    let pcm_state = received.clone();
    let pcm_task = tokio::spawn(async move {
        tokio::time::timeout(Duration::from_secs(40), collect_pcm(pcm, pcm_state))
            .await
            .map_err(|_| "PCM watchdog")?
    });
    let mut engine = None;
    let mut player = None;
    let scenario = tokio::time::timeout(
        Duration::from_secs(30),
        AssertUnwindSafe(async {
            let page = fixture
                .resolver
                .extension_media(&fixture.account_id, None, None, 0, 2)
                .await
                .unwrap();
            assert_eq!(page.items.len(), 1);
            assert!(page.next_offset.is_none());
            let item = &page.items[0];
            assert_eq!(item.season_number, Some(2025050473));
            assert_eq!(item.episode_number, Some(41));
            let source = SourceHandle::parse(&item.id).unwrap();
            let SourceHandle::Remote(remote) = &source else {
                panic!("v2 must retain remote identity")
            };
            assert_eq!(remote.account_id, fixture.account_id);
            assert_eq!(remote.remote.server_id, "fixture-server");
            assert_eq!(remote.remote.user_id, "family");
            assert_eq!(remote.item_id, "55508");
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
            player.load(source.clone(), Duration::ZERO, true).unwrap();
            wait_pcm(player, engine, &states[0], &received, 0, 0).await;
            let original = player.checkpoint().unwrap();
            let first_generation = player.snapshot().generation.unwrap();
            let accepted = player.accepted_pin().unwrap().1.clone();
            assert_eq!(accepted.media_source_id, "mediasource_55508");
            assert_eq!(accepted.audio_stream_index, 1);
            assert_eq!(fixture.reserved(), 1);
            let previous_bytes = received.lock().unwrap().len();
            player.seek(Duration::from_millis(OFFSET)).unwrap();
            wait_pcm(
                player,
                engine,
                &states[1],
                &received,
                previous_bytes,
                OFFSET,
            )
            .await;
            assert_ne!(player.snapshot().generation, Some(first_generation));
            let saved = player.checkpoint().unwrap();
            assert_eq!(saved.occurrence_id, original.occurrence_id);
            assert_eq!(saved.source, source.to_handle());
            assert_eq!(saved.selection, AudioSelection::Pinned(accepted));
            assert!(saved.offset_ms > OFFSET && saved.offset_ms < OFFSET + 10_000);
            let mut tick = tokio::time::interval(Duration::from_millis(10));
            // The first guest release only acknowledges the exact saved UUID/lease.
            tokio::time::timeout(Duration::from_secs(5), async {
                while !states[0].lock().unwrap().peer_closed || fixture.reserved() != 1 {
                    tick.tick().await;
                }
            })
            .await
            .expect("seek must close and release the first input");
            assert_eq!(*fixture.releases.0.lock().unwrap(), ["released:0"]);
            if let Some(task) = player.task.as_mut() {
                tokio::time::timeout(Duration::from_secs(5), task)
                    .await
                    .unwrap()
                    .unwrap();
            }
            fixture
                .accounts
                .set_enabled(V2_PLUGIN, false, "t1")
                .await
                .unwrap();
            assert!(fixture
                .accounts
                .status()
                .iter()
                .any(|entry| entry.account_id == fixture.account_id
                    && entry.retired
                    && entry.runtime.disabled));
            assert!(matches!(
                fixture.accounts.for_account(&fixture.account_id).await,
                Err(AccountError::Disabled)
            ));
            assert_eq!(
                fixture.reserved(),
                1,
                "disable must retain active cleanup ownership"
            );
            player.stop();
            assert_eq!(player.snapshot().phase, Phase::Stopped);
            tokio::time::timeout(Duration::from_secs(5), async {
                while !states[1].lock().unwrap().peer_closed || fixture.reserved() != 0 {
                    tick.tick().await;
                }
            })
            .await
            .expect("stop must release the second session on its disabled runtime");
            assert_eq!(
                *fixture.releases.0.lock().unwrap(),
                ["released:0", "released:1"]
            );
            // A retained runtime cannot be used to bypass disable.
            assert!(matches!(
                account_runtime.browse(None, None, 0, 2).await,
                Err(AccountError::Changed)
            ));
            player.play().unwrap();
            loop {
                tick.tick().await;
                player.poll(engine);
                if player.snapshot().phase == Phase::Failed {
                    break;
                }
            }
            assert_eq!(player.error(), Some("source extension disabled"));
            assert_eq!(fixture.reserved(), 0);
            for state in &states {
                assert_eq!(state.lock().unwrap().requests, 1);
            }
        })
        .catch_unwind(),
    )
    .await;

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
    drop(engine.take());
    if let Some(task) = abandoned {
        let _ = join_fixture(task).await;
    }
    shutdown.send_replace(true);
    let mut http_results = Vec::new();
    for task in http_tasks {
        http_results.push(join_fixture(task).await);
    }
    let pcm_result = join_fixture(pcm_task).await;
    match scenario {
        Ok(Ok(())) => {}
        Ok(Err(panic)) => std::panic::resume_unwind(panic),
        Err(_) => panic!("v2 playback deadline; no success inferred"),
    }
    for result in http_results {
        result.unwrap().unwrap();
    }
    pcm_result.unwrap().unwrap();
    assert_eq!(fixture.reserved(), 0);
}
