use super::super::tests::{barrier, bounded, output, pending};
use super::*;
use crate::host::HostContext;
use serde_json::json;
use std::collections::BTreeMap;

fn fixture(stream_body: &str) -> (tempfile::TempDir, Arc<Extension>) {
    let mut wat = String::from(
        r#"(module
      (import "extism:host/env" "alloc" (func $alloc (param i64) (result i64)))
      (import "extism:host/env" "store_u8" (func $store (param i64 i32)))
      (import "extism:host/env" "output_set" (func $output (param i64 i64)))
      (global $calls (mut i32) (i32.const 0))
    "#,
    );
    wat.push_str(&output("source_info", r#"{"name":"Legacy fixture"}"#, ""));
    wat.push_str(&output(
        "list_tracks",
        r#"[{"id":"55508","title":"Target","artist":"Fixture"}]"#,
        "",
    ));
    wat.push_str(&output(
        "get_stream_url",
        stream_body,
        "(global.set $calls (i32.add (global.get $calls) (i32.const 1)))",
    ));
    let count = output("calls", "0", "");
    let at = count.find("(call $output").unwrap();
    wat.push_str(&count[..at]);
    wat.push_str("(call $store (local.get $p) (i32.add (i32.const 48) (global.get $calls)))");
    wat.push_str(&count[at..]);
    // The host must never invent a v1-to-v2 release call.
    wat.push_str(r#"(func (export "release_audio_v2") (result i32) unreachable))"#);
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("plugin.wat"), wat).unwrap();
    std::fs::write(dir.path().join("plugin.json"), json!({
        "id":"legacy.scheduler", "name":"Legacy fixture", "capabilities":["source"], "entry":"plugin.wat"
    }).to_string()).unwrap();
    let ext = Extension::load(dir.path(), &BTreeMap::new(), HostContext::default()).unwrap();
    (dir, Arc::new(ext))
}
const STREAM: &str = r#"{"url":"http://127.0.0.1/audio","mimeType":"audio/mpeg","headers":"{\"Authorization\":\"fixture\"}"}"#;
fn request() -> legacy::StreamRequest {
    legacy::StreamRequest {
        track_id: "55508".into(),
    }
}
fn page() -> legacy::BrowseRequest {
    legacy::BrowseRequest {
        limit: 10,
        ..Default::default()
    }
}
fn calls(ext: &Extension) -> u32 {
    ext.call_json("calls", &()).unwrap().unwrap()
}

#[tokio::test]
async fn legacy_calls_share_capacity_and_disable_keeps_local_lease_ownership() {
    bounded(async {
        let (_dir, ext) = fixture(STREAM);
        let runtime = SourceRuntime::new(ext.clone()).unwrap();
        assert_eq!(
            runtime.source_info_v1().await.unwrap().name,
            "Legacy fixture"
        );
        assert_eq!(runtime.list_tracks_v1(page()).await.unwrap()[0].id, "55508");
        assert!(runtime.list_albums_v1(page()).await.unwrap().is_empty());
        let mut resolved = Vec::new();
        for _ in 0..SLOT_LIMIT {
            resolved.push(runtime.resolve_audio_v1(request()).await.unwrap());
        }
        assert_eq!(resolved[0].response.mime_type, "audio/mpeg");
        assert!(resolved[0].response.headers.contains("Authorization"));
        assert_eq!(runtime.status().reserved, SLOT_LIMIT);
        let alias = SourceRuntime::new(ext).unwrap();
        assert!(matches!(
            alias.resolve_audio_v1(request()).await,
            Err(RuntimeError::Busy)
        ));
        runtime.disable();
        assert!(matches!(
            alias.source_info_v1().await,
            Err(RuntimeError::Unavailable)
        ));
        assert_eq!(runtime.status().reserved, SLOT_LIMIT);
        drop(resolved);
        assert_eq!(runtime.status().reserved, 0);
        assert_eq!(runtime.status().failed_cleanup, 0);
    })
    .await;
}

#[tokio::test]
async fn cancellation_before_dispatch_skips_guest_and_after_execution_releases_local_permit() {
    bounded(async {
        let (_dir, ext) = fixture(STREAM);
        let runtime = SourceRuntime::new(ext.clone()).unwrap();
        let (job, entered, resume) = barrier();
        runtime.enqueue(job);
        entered.await.unwrap();
        let mut attempt = Box::pin(runtime.resolve_audio_v1(request()));
        pending(attempt.as_mut()).await;
        drop(attempt);
        resume.send(()).unwrap();
        runtime.source_info_v1().await.unwrap();
        assert_eq!(calls(&ext), 0);
        assert_eq!(runtime.status().reserved, 0);
        let (job, entered, resume) = barrier();
        *runtime.core.before_resolve_delivery.lock().unwrap() = Some(job);
        let mut attempt = Box::pin(runtime.resolve_audio_v1(request()));
        pending(attempt.as_mut()).await;
        entered.await.unwrap();
        assert_eq!(runtime.status().reserved, 1);
        drop(attempt);
        resume.send(()).unwrap();
        runtime.source_info_v1().await.unwrap();
        assert_eq!(calls(&ext), 1);
        assert_eq!(runtime.status().reserved, 0);
    })
    .await;
}

#[tokio::test]
async fn invalid_response_is_not_retried_and_versions_never_fall_back() {
    bounded(async {
        let (_dir, ext) = fixture("{}");
        let runtime = SourceRuntime::new(ext.clone()).unwrap();
        assert!(matches!(
            runtime.resolve_audio_v1(request()).await,
            Err(RuntimeError::CallFailed)
        ));
        assert_eq!(calls(&ext), 1);
        assert_eq!(runtime.status().reserved, 0);
        assert!(runtime
            .source_identity_v2(IdentityRequest {})
            .await
            .is_err());
        assert_eq!(calls(&ext), 1);
        let (_v2_dir, v2) = super::super::tests::fixture(false, false);
        let v2_runtime = SourceRuntime::new(v2).unwrap();
        assert!(v2_runtime.resolve_audio_v1(request()).await.is_err());
        let identity = v2_runtime
            .source_identity_v2(IdentityRequest {})
            .await
            .unwrap();
        assert_eq!(identity.server_id, "0");
        assert_eq!(identity.user_id, "0");
    })
    .await;
}
