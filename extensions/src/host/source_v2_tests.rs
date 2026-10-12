//! Real Extism modules compiled from WAT by the existing runtime in CI.
//! These cover protocol dispatch, not media decoding or daemon integration.
use super::*;

const EXPORTS: [&str; 6] = [
    "source_identity_v2",
    "browse_media_v2",
    "get_media_item_v2",
    "get_audio_options_v2",
    "resolve_audio_v2",
    "release_audio_v2",
];

fn module(missing: Option<&str>, resolve_body: &str) -> String {
    let mut wat = String::from(
        r#"(module
      (import "extism:host/env" "alloc" (func $alloc (param i64) (result i64)))
      (import "extism:host/env" "store_u8" (func $store (param i64 i32)))
      (import "extism:host/env" "input_length" (func $length (result i64)))
      (import "extism:host/env" "input_load_u8" (func $load (param i64) (result i32)))
      (import "extism:host/env" "output_set" (func $output (param i64 i64)))
      (global $legacy_calls (mut i32) (i32.const 0))
      (func $byte (param $value i32) (local $ptr i64)
        (local.set $ptr (call $alloc (i64.const 1)))
        (call $store (local.get $ptr) (local.get $value))
        (call $output (local.get $ptr) (i64.const 1)))
      (func $echo (result i32) (local $ptr i64) (local $len i64) (local $i i64)
        (local.set $len (call $length))
        (local.set $ptr (call $alloc (local.get $len)))
        (block $done (loop $copy
          (br_if $done (i64.ge_u (local.get $i) (local.get $len)))
          (call $store (i64.add (local.get $ptr) (local.get $i)) (call $load (local.get $i)))
          (local.set $i (i64.add (local.get $i) (i64.const 1)))
          (br $copy)))
        (call $output (local.get $ptr) (local.get $len))
        (i32.const 0))
      (func (export "legacy_calls") (result i32)
        (call $byte (i32.add (global.get $legacy_calls) (i32.const 48)))
        (i32.const 0))
    "#,
    );
    let legacy = serde_json::json!({
        "url": "https://fixture.invalid/audio", "mimeType": "audio/flac",
        "headers": "{\"Authorization\":\"fixture\"}"
    })
    .to_string();
    wat.push_str(&format!(
        r#"(func (export "get_stream_url") (result i32) (local $ptr i64)
        (global.set $legacy_calls (i32.add (global.get $legacy_calls) (i32.const 1)))
        (local.set $ptr (call $alloc (i64.const {})))"#,
        legacy.len()
    ));
    for (offset, byte) in legacy.bytes().enumerate() {
        wat.push_str(&format!(
            "(call $store (i64.add (local.get $ptr) (i64.const {offset})) (i32.const {byte}))"
        ));
    }
    wat.push_str(&format!(
        "(call $output (local.get $ptr) (i64.const {})) (i32.const 0))",
        legacy.len()
    ));
    for export in EXPORTS {
        if missing == Some(export) {
            continue;
        }
        let body = if export == "resolve_audio_v2" {
            resolve_body
        } else {
            "(call $echo)"
        };
        wat.push_str(&format!(
            "(func (export \"{export}\") (result i32) {body})\n"
        ));
    }
    wat.push(')');
    wat
}

fn load_fixture(
    version: Option<u32>,
    missing: Option<&str>,
    body: &str,
) -> (tempfile::TempDir, Result<Extension, Error>) {
    load_wat(version, module(missing, body))
}

fn load_wat(version: Option<u32>, wat: String) -> (tempfile::TempDir, Result<Extension, Error>) {
    let dir = tempfile::tempdir().unwrap();
    let mut manifest = serde_json::json!({
        "id": "fixture.source", "name": "Protocol fixture",
        "capabilities": ["source"], "entry": "plugin.wat"
    });
    if let Some(version) = version {
        manifest["sourceApiVersion"] = version.into();
    }
    std::fs::write(
        dir.path().join("plugin.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    std::fs::write(dir.path().join("plugin.wat"), wat).unwrap();
    let loaded = Extension::load(dir.path(), &BTreeMap::new(), HostContext::default());
    (dir, loaded)
}

fn legacy_calls(extension: &Extension) -> u32 {
    extension.call_json("legacy_calls", &()).unwrap().unwrap()
}

fn fixed_resolve_module(response: &serde_json::Value) -> String {
    let mut wat = module(None, "(call $reply)");
    assert_eq!(wat.pop(), Some(')'));
    let bytes = serde_json::to_vec(response).unwrap();
    wat.push_str(&format!(
        "(func $reply (result i32) (local $ptr i64) (local.set $ptr (call $alloc (i64.const {})))",
        bytes.len()
    ));
    for (offset, byte) in bytes.iter().enumerate() {
        wat.push_str(&format!(
            "(call $store (i64.add (local.get $ptr) (i64.const {offset})) (i32.const {byte}))"
        ));
    }
    wat.push_str(&format!(
        "(call $output (local.get $ptr) (i64.const {})) (i32.const 0)))",
        bytes.len()
    ));
    wat
}

fn typed_resolve_request() -> crate::abi::source_v2::ResolveRequest {
    serde_json::from_value(serde_json::json!({
        "requestId": "attempt-41",
        "identity": {"serverId": "server", "userId": "family"},
        "itemId": "55508", "offsetMs": 7200123,
        "selection": {"type": "auto"},
        "formatExts": ["mp3"], "followRedirects": false
    }))
    .unwrap()
}

fn typed_resolve_response() -> serde_json::Value {
    serde_json::json!({
        "requestId": "attempt-41", "requestedOffsetMs": 7200123,
        "url": "https://fixture.invalid/audio",
        "headers": [{"name": "Authorization", "value": "synthetic-credential"}],
        "mimeType": "audio/mpeg", "formatExt": "mp3", "actualPin": {"mediaSourceId":"version-a","audioStreamIndex":0,"runtimeTicks":null,"etag":null,"codec":null,"channels":null,"sampleRate":null},
        "start": {"type": "calibrated", "offsetMs": 7200123, "maxErrorMs": 20},
        "timeline": {"type": "finite", "durationTicks": 191429666670u64},
        "seekPolicy": "serverOffset", "cachePolicy": "noPersistentCache",
        "leaseId": "synthetic-lease"
    })
}

#[test]
fn typed_resolve_preserves_wasm_transport_and_program_time() {
    let expected = typed_resolve_response();
    let (_dir, loaded) = load_wat(Some(2), fixed_resolve_module(&expected));
    let extension = loaded.unwrap();
    let response = extension
        .resolve_audio_v2(&typed_resolve_request())
        .unwrap();
    assert_eq!(serde_json::to_value(response).unwrap(), expected);
    assert_eq!(legacy_calls(&extension), 0);
}

#[test]
fn typed_resolve_rejects_wrong_attempt_offset_and_format_without_fallback() {
    for (key, value) in [
        ("requestId", serde_json::json!("older-attempt")),
        ("requestedOffsetMs", serde_json::json!(0)),
        ("formatExt", serde_json::json!("flac")),
        ("seekPolicy", serde_json::json!("none")),
        ("actualPin", serde_json::Value::Null),
        (
            "start",
            serde_json::json!({"type":"requestedOnly","offsetMs":7200123}),
        ),
        (
            "timeline",
            serde_json::json!({"type":"unknown","durationTicks":191429666670u64}),
        ),
    ] {
        let mut response = typed_resolve_response();
        response[key] = value;
        let (_dir, loaded) = load_wat(Some(2), fixed_resolve_module(&response));
        let extension = loaded.unwrap();
        let result = extension.resolve_audio_v2(&typed_resolve_request());
        let error = result.err().expect("invalid response accepted");
        let diagnostic = format!("{error:#}");
        for secret in ["synthetic-credential", "synthetic-lease", "fixture.invalid"] {
            assert!(!diagnostic.contains(secret));
        }
        assert_eq!(legacy_calls(&extension), 0);
    }
}

#[test]
fn v2_requires_every_declared_export_before_loading() {
    for export in EXPORTS {
        let (_dir, loaded) = load_fixture(Some(2), Some(export), "(call $echo)");
        let error = loaded.err().expect("incomplete v2 module loaded");
        assert!(format!("{error:#}").contains(export));
    }
}

#[test]
fn v2_calls_real_wasm_and_retains_large_integers_and_index_zero() {
    let (_dir, loaded) = load_fixture(Some(2), None, "(call $echo)");
    let extension = loaded.unwrap();
    let input = serde_json::json!({
        "offsetMs": u64::MAX,
        "audioStreamIndex": 0,
        "headers": {"Authorization": "synthetic-fixture-only"},
        "followRedirects": false
    });
    for export in EXPORTS {
        let output: serde_json::Value = extension.call_source_v2(export, &input).unwrap();
        assert_eq!(output, input);
    }
    assert_eq!(legacy_calls(&extension), 0);
}

#[test]
fn v2_empty_invalid_and_trapped_calls_do_not_fall_back_to_v1() {
    for body in [
        "(i32.const 0)",
        "(call $byte (i32.const 33)) (i32.const 0)",
        "unreachable",
    ] {
        let (_dir, loaded) = load_fixture(Some(2), None, body);
        let extension = loaded.unwrap();
        assert!(extension
            .call_source_v2::<_, serde_json::Value>("resolve_audio_v2", &())
            .is_err());
        assert_eq!(legacy_calls(&extension), 0);
        assert!(extension.stream_url("item").is_err());
        assert!(extension.source_info().is_err());
        assert!(extension.list_tracks(&BrowseRequest::default()).is_err());
        assert_eq!(legacy_calls(&extension), 0);
    }
}

#[test]
fn legacy_manifest_keeps_v1_and_cannot_enter_v2_even_if_exports_exist() {
    for version in [None, Some(1)] {
        let (_dir, loaded) = load_fixture(version, None, "(call $echo)");
        let extension = loaded.unwrap();
        let response = extension.stream_url("item").unwrap().unwrap();
        assert_eq!(response.url, "https://fixture.invalid/audio");
        assert_eq!(response.mime_type, "audio/flac");
        assert_eq!(response.headers, "{\"Authorization\":\"fixture\"}");
        assert_eq!(legacy_calls(&extension), 1);
        assert!(extension
            .call_source_v2::<_, serde_json::Value>("resolve_audio_v2", &())
            .is_err());
        assert_eq!(legacy_calls(&extension), 1);
    }
}

#[test]
fn v2_execution_timeout_does_not_fall_back_and_instance_remains_callable() {
    let (_dir, loaded) = load_fixture(Some(2), None, "(loop $forever (br $forever)) (i32.const 0)");
    let extension = loaded.unwrap();
    // A separate test watchdog fails even if the production timer regresses.
    // Do not join a stuck worker: the test process owns its final termination.
    let (done, completed) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let started = std::time::Instant::now();
        assert!(extension
            .call_source_v2::<_, serde_json::Value>("resolve_audio_v2", &())
            .is_err());
        assert!(
            started.elapsed() >= CALL_TIMEOUT.saturating_sub(Duration::from_secs(1)),
            "guest failed before the execution timeout"
        );
        assert_eq!(legacy_calls(&extension), 0);
        let response: serde_json::Value = extension
            .call_source_v2(
                "release_audio_v2",
                &serde_json::json!({"requestId":"finished"}),
            )
            .unwrap();
        assert_eq!(response["requestId"], "finished");
        // Include instance teardown in the watchdog.
        drop(extension);
        done.send(()).unwrap();
    });
    completed
        .recv_timeout(Duration::from_secs(30))
        .expect("source v2 execution/recovery exceeded the test watchdog or worker failed");
    worker.join().unwrap();
}

#[test]
fn v2_busy_instance_rejects_without_waiting_and_can_be_retried() {
    let (_dir, loaded) = load_fixture(Some(2), None, "(call $echo)");
    let (finished, completion) = std::sync::mpsc::channel();
    let scenario = std::thread::spawn(move || {
        busy_instance_scenario(loaded.unwrap());
        // The scenario drops all instance owners before acknowledging.
        finished.send(()).unwrap();
    });
    completion
        .recv_timeout(Duration::from_secs(30))
        .expect("busy instance admission/retry/teardown exceeded watchdog or worker failed");
    scenario.join().unwrap();
}

fn busy_instance_scenario(loaded: Extension) {
    let extension = Arc::new(loaded);
    let held = extension.plugin.lock().unwrap();
    let caller = Arc::clone(&extension);
    let (done, completed) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let result = caller.call_source_v2::<_, serde_json::Value>("resolve_audio_v2", &());
        // Report before releasing the held lock; a blocking implementation
        // cannot satisfy this observation.
        let _ = done.send(result);
    });
    let observed = completed.recv_timeout(Duration::from_secs(2));
    // Release even on regression so the test does not leave a blocked worker.
    drop(held);
    worker.join().unwrap();
    let error = observed
        .expect("source v2 waited on a busy instance")
        .expect_err("busy instance executed a guest call");
    assert!(error.downcast_ref::<SourceInstanceBusy>().is_some());
    let input = serde_json::json!({"requestId": "still-current"});
    let response: serde_json::Value = extension
        .call_source_v2("resolve_audio_v2", &input)
        .unwrap();
    assert_eq!(response, input);
    assert_eq!(legacy_calls(&extension), 0);
}

#[test]
fn loaded_protocol_cannot_be_changed_by_mutating_manifest() {
    let (_dir, loaded) = load_fixture(Some(2), None, "(call $echo)");
    let mut extension = loaded.unwrap();
    extension.manifest.source_api_version = Some(1);
    assert!(extension.stream_url("item").is_err());
    assert!(extension
        .call_json::<_, serde_json::Value>("get_stream_url", &())
        .is_err());
    let response: serde_json::Value = extension
        .call_source_v2("resolve_audio_v2", &serde_json::json!({"ok":true}))
        .unwrap();
    assert_eq!(response["ok"], true);
    assert_eq!(legacy_calls(&extension), 0);
}
