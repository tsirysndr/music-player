//! Real WAT dispatch/cleanup; barriers control host scheduling, not fake guest results.
use super::*;
use crate::host::HostContext;
use serde_json::json;
use std::{collections::BTreeMap, future::Future, sync::mpsc, task::Poll};

pub(super) fn output(name: &str, body: &str, prefix: &str) -> String {
    let mut wat = format!("(func (export \"{name}\") (result i32) (local $p i64) {prefix} (local.set $p (call $alloc (i64.const {})))", body.len());
    for (index, byte) in body.bytes().enumerate() {
        wat.push_str(&format!(
            "(call $store (i64.add (local.get $p) (i64.const {index})) (i32.const {byte}))"
        ));
    }
    wat.push_str(&format!(
        "(call $output (local.get $p) (i64.const {})) (i32.const 0))",
        body.len()
    ));
    wat
}
pub(super) fn fixture(
    invalid_resolve: bool,
    failed_release: bool,
) -> (tempfile::TempDir, Arc<Extension>) {
    let response = if invalid_resolve {
        ResolveFixture::InvalidShape
    } else {
        ResolveFixture::Valid
    };
    fixture_with_failures(response, failed_release, 0)
}

#[derive(Clone, Copy)]
enum ResolveFixture {
    Valid,
    InvalidShape,
    MissingPin,
}

fn fixture_with_failures(
    resolve_fixture: ResolveFixture,
    failed_release: bool,
    initial_failures: u32,
) -> (tempfile::TempDir, Arc<Extension>) {
    let mut wat = String::from(
        r#"(module
      (import "extism:host/env" "alloc" (func $alloc (param i64) (result i64)))
      (import "extism:host/env" "store_u8" (func $store (param i64 i32)))
      (import "extism:host/env" "output_set" (func $output (param i64 i64)))
      (import "extism:host/env" "input_length" (func $length (result i64)))
      (import "extism:host/env" "input_load_u8" (func $load (param i64) (result i32)))
      (global $resolve (mut i32) (i32.const 0))
      (global $release (mut i32) (i32.const 0))
    "#,
    );
    let identity_json = r#"{"serverId":"0","userId":"0"}"#;
    let fixed = output("source_identity_v2", identity_json, "");
    let at = fixed.find("(call $output").unwrap();
    wat.push_str(&fixed[..at]);
    for (field, counter) in [("serverId", "$resolve"), ("userId", "$release")] {
        let index = identity_json.find(&format!("\"{field}\":\"0")).unwrap() + field.len() + 4;
        wat.push_str(&format!("(call $store (i64.add (local.get $p) (i64.const {index})) (i32.add (i32.const 48) (global.get {counter})))"));
    }
    wat.push_str(&fixed[at..]);
    wat.push_str(&output(
        "browse_media_v2",
        r#"{"items":[],"nextOffset":null}"#,
        "",
    ));
    wat.push_str(&output(
        "get_media_item_v2",
        r#"{"item":{"itemId":"item","title":"Fixture","isContainer":false,"playable":true}}"#,
        "",
    ));
    wat.push_str(&output(
        "get_audio_options_v2",
        r#"{"itemId":"item","versions":[]}"#,
        "",
    ));
    let mut resolved = json!({
        "requestId":"attempt", "requestedOffsetMs":0, "url":"http://127.0.0.1/audio",
        "headers":[], "mimeType":"audio/mpeg", "formatExt":"mp3",
        "actualPin":{"mediaSourceId":"version","audioStreamIndex":0},
        "start":{"type":"requestedOnly"}, "timeline":{"type":"unknown"},
        "seekPolicy":"serverOffset", "cachePolicy":"noPersistentCache", "leaseId":"lease"
    });
    match resolve_fixture {
        ResolveFixture::Valid => (),
        ResolveFixture::InvalidShape => resolved = json!({}),
        ResolveFixture::MissingPin => resolved["actualPin"] = serde_json::Value::Null,
    }
    wat.push_str(&output(
        "resolve_audio_v2",
        &resolved.to_string(),
        "(global.set $resolve (i32.add (global.get $resolve) (i32.const 1)))",
    ));
    let released = json!({"requestId":"attempt","released":!failed_release}).to_string();
    let expected = serde_json::to_string(&ReleaseRequest {
        request_id: "attempt".into(),
        identity: identity(),
        item_id: "item".into(),
        lease_id: if matches!(resolve_fixture, ResolveFixture::Valid) {
            Some("lease".into())
        } else {
            None
        },
    })
    .unwrap();
    let mut check = format!("(global.set $release (i32.add (global.get $release) (i32.const 1))) (if (i64.ne (call $length) (i64.const {})) (then (return (i32.const 1))))", expected.len());
    for (index, byte) in expected.bytes().enumerate() {
        check.push_str(&format!("(if (i32.ne (call $load (i64.const {index})) (i32.const {byte})) (then (return (i32.const 1))))"));
    }
    check.push_str(&format!("(if (i32.le_u (global.get $release) (i32.const {initial_failures})) (then (return (i32.const 1))))"));
    wat.push_str(&output("release_audio_v2", &released, &check));
    wat.push(')');
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("plugin.wat"), wat).unwrap();
    std::fs::write(
        dir.path().join("plugin.json"),
        json!({
            "id":"scheduler.fixture", "name":"Scheduler fixture", "capabilities":["source"],
            "entry":"plugin.wat", "sourceApiVersion":2
        })
        .to_string(),
    )
    .unwrap();
    let ext = Extension::load(dir.path(), &BTreeMap::new(), HostContext::default()).unwrap();
    (dir, Arc::new(ext))
}
fn identity() -> SourceIdentity {
    SourceIdentity {
        server_id: "server".into(),
        user_id: "user".into(),
    }
}
fn request() -> ResolveRequest {
    ResolveRequest {
        request_id: "attempt".into(),
        identity: identity(),
        item_id: "item".into(),
        offset_ms: 0,
        selection: AudioSelection::Auto {},
        format_exts: vec!["mp3".into()],
        follow_redirects: true,
    }
}
pub(super) async fn pending<F: Future>(mut future: std::pin::Pin<&mut F>) {
    std::future::poll_fn(|cx| {
        assert!(future.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
}
pub(super) fn barrier() -> (Job, oneshot::Receiver<()>, mpsc::SyncSender<()>) {
    let (entered_tx, entered_rx) = oneshot::channel();
    let (exit_tx, exit_rx) = mpsc::sync_channel(1);
    let job = Box::new(move || {
        let _ = entered_tx.send(());
        let _ = exit_rx.recv_timeout(Duration::from_secs(5));
    });
    (job, entered_rx, exit_tx)
}
pub(super) async fn bounded(future: impl Future<Output = ()>) {
    tokio::time::timeout(Duration::from_secs(20), future)
        .await
        .expect("scheduler scenario watchdog");
}
async fn counters(runtime: &SourceRuntime) -> SourceIdentity {
    runtime
        .source_identity_v2(IdentityRequest {})
        .await
        .unwrap()
}

#[tokio::test]
async fn last_idle_handle_wakes_retirement_after_clone_and_instance_reuse() {
    bounded(async {
        let (_dir, ext) = fixture(false, false);
        let runtime = SourceRuntime::new(ext.clone()).unwrap();
        let cloned = runtime.clone();
        let reused = SourceRuntime::new(ext.clone()).unwrap();
        assert!(Arc::ptr_eq(&runtime.core, &reused.core));
        assert_eq!(runtime.core.state.lock().unwrap().handles, 3);
        drop(runtime);
        drop(cloned);
        assert_eq!(reused.core.state.lock().unwrap().handles, 1);
        assert_eq!(counters(&reused).await.server_id, "0");

        let (retired, done) = oneshot::channel();
        *reused.core.lifetime_observer.lock().unwrap() = Some(retired);
        let (idle, entered) = oneshot::channel();
        // The notification and worker predicate share the state lock. Waiting
        // for this hook ensures last-handle drop exercises the idle wake path.
        {
            let _state = reused.core.state.lock().unwrap();
            *reused.core.before_idle_wait.lock().unwrap() = Some(idle);
            reused.core.wake.notify_one();
        }
        entered.await.unwrap();
        drop(reused);
        assert!(done.await.is_err());
        // A non-disabled instance can be reused after full scheduler retirement.
        let restarted = SourceRuntime::new(ext).unwrap();
        assert_eq!(counters(&restarted).await.server_id, "0");
    })
    .await;
}

#[tokio::test]
async fn lease_without_handles_retains_worker_until_original_cleanup_finishes() {
    bounded(async {
        let (_dir, ext) = fixture(false, false);
        let runtime = SourceRuntime::new(ext.clone()).unwrap();
        let resolved = runtime.resolve_audio_v2(request()).await.unwrap();
        let weak = Arc::downgrade(&runtime.core);
        let (retired, mut done) = oneshot::channel();
        *runtime.core.lifetime_observer.lock().unwrap() = Some(retired);
        drop(runtime);
        {
            let core = weak.upgrade().expect("lease retains original worker");
            let state = core.state.lock().unwrap();
            assert_eq!(state.handles, 0);
            assert_eq!(state.reserved, 1);
        }
        assert!(matches!(
            done.try_recv(),
            Err(oneshot::error::TryRecvError::Empty)
        ));
        let observer = SourceRuntime::new(ext.clone()).unwrap();
        assert!(Arc::ptr_eq(&observer.core, &weak.upgrade().unwrap()));
        assert_eq!(observer.core.state.lock().unwrap().handles, 1);
        drop(observer);
        drop(resolved);
        assert!(done.await.is_err());
        assert!(weak.upgrade().is_none());
        // The real WAT export, not reservation accounting, proves release ran.
        assert_eq!(
            ext.source_identity_v2(&IdentityRequest {}).unwrap().user_id,
            "1"
        );
    })
    .await;
}

#[tokio::test]
async fn six_exports_and_owned_release_use_real_instance() {
    bounded(async {
        let (_dir, ext) = fixture(false, false);
        let runtime = SourceRuntime::new(ext).unwrap();
        assert_eq!(counters(&runtime).await.server_id, "0");
        assert!(runtime
            .browse_media_v2(BrowseRequest {
                identity: identity(),
                parent_id: None,
                query: None,
                offset: 0,
                limit: 10
            })
            .await
            .unwrap()
            .items
            .is_empty());
        assert_eq!(
            runtime
                .get_media_item_v2(ItemRequest {
                    identity: identity(),
                    item_id: "item".into()
                })
                .await
                .unwrap()
                .item
                .title,
            "Fixture"
        );
        assert!(runtime
            .get_audio_options_v2(AudioOptionsRequest {
                identity: identity(),
                item_id: "item".into()
            })
            .await
            .unwrap()
            .versions
            .is_empty());
        let resolved = runtime.resolve_audio_v2(request()).await.unwrap();
        assert_eq!(resolved.response.format_ext, "mp3");
        let pin = resolved.response.actual_pin.as_ref().unwrap();
        assert_eq!(pin.media_source_id, "version");
        assert_eq!(pin.audio_stream_index, 0);
        assert_eq!(runtime.status().reserved, 1);
        assert!(
            runtime
                .release_audio_v2(resolved.lease)
                .await
                .unwrap()
                .released
        );
        let count = counters(&runtime).await;
        assert_eq!(
            (count.server_id.as_str(), count.user_id.as_str()),
            ("1", "1")
        );
    })
    .await;
}
#[tokio::test]
async fn cancelled_queued_resolve_never_enters_guest_and_instances_share_budget() {
    bounded(async {
        let (_dir, ext) = fixture(false, false);
        let runtime = SourceRuntime::new(ext.clone()).unwrap();
        let alias = SourceRuntime::new(ext).unwrap();
        assert!(Arc::ptr_eq(&runtime.core, &alias.core));
        let (gate, entered, exit) = barrier();
        runtime.enqueue(gate);
        entered.await.unwrap();
        let mut calls = Vec::new();
        for n in 0..SLOT_LIMIT {
            let mut req = request();
            req.request_id = format!("queued-{n}");
            let mut future = Box::pin(runtime.resolve_audio_v2(req));
            pending(future.as_mut()).await;
            calls.push(future);
        }
        assert!(matches!(
            alias.source_identity_v2(IdentityRequest {}).await,
            Err(RuntimeError::Busy)
        ));
        drop(calls);
        exit.send(()).unwrap();
        let (done, seen, release) = barrier();
        runtime.enqueue(done);
        seen.await.unwrap();
        assert_eq!(runtime.status().reserved, 0);
        release.send(()).unwrap();
        let count = counters(&runtime).await;
        assert_eq!(
            (count.server_id.as_str(), count.user_id.as_str()),
            ("0", "0")
        );
    })
    .await;
}
#[tokio::test]
async fn dropped_executed_resolve_cleans_original_instance_without_waiter() {
    bounded(async {
        let (_dir, ext) = fixture(false, false);
        let (_other_dir, other) = fixture(false, false);
        let runtime = SourceRuntime::new(ext).unwrap();
        let other = SourceRuntime::new(other).unwrap();
        let (hook, entered, exit) = barrier();
        *runtime.core.before_resolve_delivery.lock().unwrap() = Some(hook);
        let mut future = Box::pin(runtime.resolve_audio_v2(request()));
        pending(future.as_mut()).await;
        entered.await.unwrap(); // guest already returned a valid lease
        drop(future);
        exit.send(()).unwrap();
        let count = counters(&runtime).await;
        assert_eq!(
            (count.server_id.as_str(), count.user_id.as_str()),
            ("1", "1")
        );
        assert_eq!(counters(&other).await.user_id, "0");
    })
    .await;
}
#[tokio::test]
async fn lease_drop_on_plain_thread_releases_without_async_cleanup_task() {
    bounded(async {
        let (_dir, ext) = fixture(false, false);
        let runtime = SourceRuntime::new(ext).unwrap();
        let resolved = runtime.resolve_audio_v2(request()).await.unwrap();
        assert!(matches!(
            runtime.resolve_audio_v2(request()).await,
            Err(RuntimeError::InvalidRequest)
        ));
        let (sent, received) = oneshot::channel();
        std::thread::spawn(move || {
            drop(resolved);
            let _ = sent.send(());
        });
        received.await.unwrap();
        assert_eq!(counters(&runtime).await.user_id, "1");
        assert_eq!(runtime.status().reserved, 0);
    })
    .await;
}
#[tokio::test]
async fn rejected_unpinned_server_offset_cleans_the_original_attempt() {
    bounded(async {
        let (_dir, ext) = fixture_with_failures(ResolveFixture::MissingPin, false, 0);
        let runtime = SourceRuntime::new(ext).unwrap();
        assert!(matches!(
            runtime.resolve_audio_v2(request()).await,
            Err(RuntimeError::CallFailed)
        ));
        // The real guest release checks the full original requestId, identity,
        // item and absent leaseId after typed response validation failed.
        let count = counters(&runtime).await;
        assert_eq!(
            (count.server_id.as_str(), count.user_id.as_str()),
            ("1", "1")
        );
        let status = runtime.status();
        assert_eq!(status.reserved, 0);
        assert_eq!(status.pending_cleanup, 0);
        assert_eq!(status.failed_cleanup, 0);
        assert!(!status.quarantined);
    })
    .await;
}

#[tokio::test]
async fn invalid_response_releases_attempt_and_failed_cleanup_pins_budget() {
    bounded(async {
        let (_dir, ext) = fixture(true, false);
        let runtime = SourceRuntime::new(ext).unwrap();
        assert!(matches!(
            runtime.resolve_audio_v2(request()).await,
            Err(RuntimeError::CallFailed)
        ));
        assert_eq!(counters(&runtime).await.user_id, "1");
        let (_bad_dir, bad_ext) = fixture(false, true);
        let bad = SourceRuntime::new(bad_ext.clone()).unwrap();
        let resolved = bad.resolve_audio_v2(request()).await.unwrap();
        assert!(matches!(
            bad.release_audio_v2(resolved.lease).await,
            Err(RuntimeError::CleanupFailed)
        ));
        let status = bad.status();
        assert!(status.quarantined);
        assert_eq!(status.failed_cleanup, 1);
        assert_eq!(status.reserved, 1);
        assert!(matches!(
            SourceRuntime::new(bad_ext.clone())
                .unwrap()
                .resolve_audio_v2(request())
                .await,
            Err(RuntimeError::Unavailable)
        ));
        assert_eq!(
            bad_ext
                .source_identity_v2(&IdentityRequest {})
                .unwrap()
                .user_id,
            "3"
        );
    })
    .await;
}

#[tokio::test]
async fn busy_retries_are_bounded_and_recheck_caller_before_guest() {
    bounded(async {
        let (_dir, ext) = fixture(false, false);
        let runtime = SourceRuntime::new(ext).unwrap();
        let attempts = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let seen = attempts.clone();
        // Inject only the typed no-execution outcome. The successful retry
        // below still invokes the real guest; this is not mutex-contention evidence.
        let result: Result<SourceIdentity, _> = runtime
            .call(move |_| {
                seen.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Err(SourceInstanceBusy.into())
            })
            .await;
        assert!(matches!(result, Err(RuntimeError::Busy)));
        assert_eq!(attempts.load(std::sync::atomic::Ordering::SeqCst), ATTEMPTS);
        let seen = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        runtime
            .call(move |ext| {
                if seen.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
                    Err(SourceInstanceBusy.into())
                } else {
                    ext.source_identity_v2(&IdentityRequest {})
                }
            })
            .await
            .unwrap();

        let (entered_tx, entered_rx) = oneshot::channel();
        let entered_tx = Mutex::new(Some(entered_tx));
        let (continue_tx, continue_rx) = mpsc::sync_channel(1);
        let mut future = Box::pin(runtime.call(move |ext| {
            if let Some(entered) = entered_tx.lock().unwrap().take() {
                let _ = entered.send(());
                let _ = continue_rx.recv_timeout(Duration::from_secs(5));
                Err(SourceInstanceBusy.into())
            } else {
                ext.resolve_audio_v2(&request())
            }
        }));
        pending(future.as_mut()).await;
        entered_rx.await.unwrap();
        drop(future);
        continue_tx.send(()).unwrap();
        assert_eq!(counters(&runtime).await.server_id, "0");
    })
    .await;
}

#[tokio::test]
async fn cleanup_outlives_originating_async_runtime_and_last_source_handle() {
    bounded(async {
        let (_dir, ext) = fixture(false, false);
        let original = ext.clone();
        let (tx, rx) = oneshot::channel();
        std::thread::spawn(move || {
            let executor = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let runtime = SourceRuntime::new(original).unwrap();
            let resolved = executor
                .block_on(runtime.resolve_audio_v2(request()))
                .unwrap();
            let weak = Arc::downgrade(&runtime.core);
            drop(runtime);
            drop(executor);
            let _ = tx.send((resolved, weak));
        });
        let (resolved, weak) = rx.await.unwrap();
        // Recreate only an observer after the originating runtime and handle died.
        // The lease alone retained the exact scheduler and Extension instance.
        let observer = SourceRuntime::new(ext).unwrap();
        assert!(Arc::ptr_eq(&observer.core, &weak.upgrade().unwrap()));
        let (done_tx, done_rx) = oneshot::channel();
        std::thread::spawn(move || {
            drop(resolved);
            let _ = done_tx.send(());
        });
        done_rx.await.unwrap();
        assert_eq!(counters(&observer).await.user_id, "1");
    })
    .await;
}

#[tokio::test]
async fn explicit_release_survives_dropped_waiter() {
    bounded(async {
        let (_dir, ext) = fixture(false, false);
        let runtime = SourceRuntime::new(ext).unwrap();
        let resolved = runtime.resolve_audio_v2(request()).await.unwrap();
        let (gate, entered, exit) = barrier();
        runtime.enqueue(gate);
        entered.await.unwrap();
        let mut release = Box::pin(runtime.release_audio_v2(resolved.lease));
        pending(release.as_mut()).await;
        drop(release);
        assert_eq!(runtime.status().pending_cleanup, 1);
        exit.send(()).unwrap();
        assert_eq!(counters(&runtime).await.user_id, "1");
        assert_eq!(runtime.status().reserved, 0);
    })
    .await;
}

#[tokio::test]
async fn disable_rejects_queued_calls_but_keeps_lease_cleanup_and_tombstone() {
    bounded(async {
        let (_dir, ext) = fixture(false, false);
        let runtime = SourceRuntime::new(ext.clone()).unwrap();
        let resolved = runtime.resolve_audio_v2(request()).await.unwrap();
        let (gate, entered, exit) = barrier();
        runtime.enqueue(gate);
        entered.await.unwrap();
        let mut req = request();
        req.request_id = "queued-at-disable".into();
        let mut queued = Box::pin(runtime.resolve_audio_v2(req));
        pending(queued.as_mut()).await;
        let mut metadata = Box::pin(runtime.source_identity_v2(IdentityRequest {}));
        pending(metadata.as_mut()).await;
        runtime.disable();
        let alias = SourceRuntime::new(ext.clone()).unwrap();
        assert!(alias.status().disabled);
        assert!(matches!(
            alias
                .get_audio_options_v2(AudioOptionsRequest {
                    identity: identity(),
                    item_id: "item".into()
                })
                .await,
            Err(RuntimeError::Unavailable)
        ));
        exit.send(()).unwrap();
        assert!(matches!(queued.await, Err(RuntimeError::Unavailable)));
        assert!(matches!(metadata.await, Err(RuntimeError::Unavailable)));
        assert!(
            runtime
                .release_audio_v2(resolved.lease)
                .await
                .unwrap()
                .released
        );
        assert_eq!(runtime.status().reserved, 0);
        let count = ext.source_identity_v2(&IdentityRequest {}).unwrap();
        assert_eq!(
            (count.server_id.as_str(), count.user_id.as_str()),
            ("1", "1")
        );
        let (tx, rx) = oneshot::channel();
        *runtime.core.lifetime_observer.lock().unwrap() = Some(tx);
        drop(alias);
        drop(runtime);
        assert!(rx.await.is_err()); // core was actually reaped, not merely disabled
        assert!(matches!(
            SourceRuntime::new(ext),
            Err(RuntimeError::Unavailable)
        ));
    })
    .await;
}

#[tokio::test]
async fn explicit_retry_recovers_real_release_without_undoing_disable_or_waiter_ownership() {
    bounded(async {
        for disabled in [false, true] {
            let (_dir, ext) = fixture_with_failures(ResolveFixture::Valid, false, 3);
            let runtime = SourceRuntime::new(ext.clone()).unwrap();
            let resolved = runtime.resolve_audio_v2(request()).await.unwrap();
            assert!(matches!(
                runtime.release_audio_v2(resolved.lease).await,
                Err(RuntimeError::CleanupFailed)
            ));
            assert_eq!(runtime.status().reserved, 1);
            assert_eq!(
                ext.source_identity_v2(&IdentityRequest {}).unwrap().user_id,
                "3"
            );
            let (gate, entered, exit) = barrier();
            runtime.enqueue(gate);
            entered.await.unwrap();
            let mut retry = Box::pin(runtime.retry_failed_cleanup());
            pending(retry.as_mut()).await;
            assert_eq!(runtime.status().retrying_cleanup, 1);
            assert!(runtime.status().quarantined);
            assert!(matches!(
                runtime.retry_failed_cleanup().await,
                Err(RuntimeError::Busy)
            ));
            if disabled {
                runtime.disable();
            }
            let (done, seen, release) = barrier();
            runtime.enqueue(done);
            if disabled {
                // Cleanup cannot depend on the explicit retry waiter staying alive.
                drop(retry);
                exit.send(()).unwrap();
            } else {
                exit.send(()).unwrap();
                assert_eq!(retry.await.unwrap(), 1);
            }
            seen.await.unwrap();
            let status = runtime.status();
            assert_eq!(status.reserved, 0);
            assert_eq!(status.failed_cleanup, 0);
            assert_eq!(status.retrying_cleanup, 0);
            assert!(!status.quarantined);
            assert_eq!(status.disabled, disabled);
            release.send(()).unwrap();
            assert_eq!(
                ext.source_identity_v2(&IdentityRequest {}).unwrap().user_id,
                "4"
            );
            let alias = SourceRuntime::new(ext).unwrap();
            if disabled {
                assert!(matches!(
                    alias.resolve_audio_v2(request()).await,
                    Err(RuntimeError::Unavailable)
                ));
            } else {
                let next = alias.resolve_audio_v2(request()).await.unwrap();
                alias.release_audio_v2(next.lease).await.unwrap();
                assert_eq!(alias.retry_failed_cleanup().await.unwrap(), 0);
            }
        }
    })
    .await;
}

#[tokio::test]
async fn repeated_release_failure_keeps_same_obligation_and_each_retry_is_finite() {
    bounded(async {
        let (_dir, ext) = fixture(false, true);
        let runtime = SourceRuntime::new(ext.clone()).unwrap();
        let resolved = runtime.resolve_audio_v2(request()).await.unwrap();
        assert!(matches!(
            runtime.release_audio_v2(resolved.lease).await,
            Err(RuntimeError::CleanupFailed)
        ));
        assert!(matches!(
            runtime.retry_failed_cleanup().await,
            Err(RuntimeError::CleanupFailed)
        ));
        assert_eq!(
            ext.source_identity_v2(&IdentityRequest {}).unwrap().user_id,
            "6"
        );
        let state = runtime.status();
        assert!(state.quarantined);
        assert_eq!(state.failed_cleanup, 1);
        assert_eq!(state.retrying_cleanup, 0);
        assert_eq!(state.reserved, 1);
        assert!(matches!(
            runtime.resolve_audio_v2(request()).await,
            Err(RuntimeError::Unavailable)
        ));
    })
    .await;
}
