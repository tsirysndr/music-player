//! Real SQLite migrations and WAT exports. Hooks only gate host scheduling;
//! they never replace guest identity/resolve/release responses. CI-only tests.
use super::*;
use migration::{Migrator, MigratorTrait};
use serde_json::json;
use std::{sync::mpsc, time::Duration};

mod legacy_tests;

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("mp-account-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
async fn bounded(future: impl Future<Output = ()>) {
    let _serial = TEST_LOAD_SERIAL.lock().await;
    tokio::time::timeout(Duration::from_secs(30), future)
        .await
        .expect("account scenario watchdog");
}
async fn eventually(mut condition: impl FnMut() -> bool) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while !condition() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("bounded cleanup/load observation");
}
fn output(export: &str, body: &str, prefix: &str) -> String {
    let mut code = format!("(func (export \"{export}\") (result i32) (local $p i64) {prefix} (local.set $p (call $alloc (i64.const {})))", body.len());
    for (index, byte) in body.bytes().enumerate() {
        code.push_str(&format!(
            "(call $store (i64.add (local.get $p) (i64.const {index})) (i32.const {byte}))"
        ));
    }
    code.push_str(&format!(
        "(call $output (local.get $p) (i64.const {})) (i32.const 0))",
        body.len()
    ));
    code
}
fn write_plugin(root: &Directory, release_failures: u32) {
    let dir = root.0.join("plugin");
    std::fs::create_dir_all(&dir).unwrap();
    let mut wat = String::from(
        r#"(module
      (import "extism:host/env" "alloc" (func $alloc (param i64) (result i64)))
      (import "extism:host/env" "store_u8" (func $store (param i64 i32)))
      (import "extism:host/env" "output_set" (func $output (param i64 i64)))
      (import "extism:host/user" "get_config" (func $config (param i64) (result i64)))
      (import "extism:host/env" "length" (func $length (param i64) (result i64)))
      (import "extism:host/env" "input_length" (func $input_length (result i64)))
      (import "extism:host/env" "input_load_u8" (func $input (param i64) (result i32)))
      (global $releases (mut i32) (i32.const 0))
      (func (export "source_identity_v2") (result i32) (local $key i64) (local $value i64)
        (local.set $key (call $alloc (i64.const 3)))
        (call $store (local.get $key) (i32.const 119))
        (call $store (i64.add (local.get $key) (i64.const 1)) (i32.const 104))
        (call $store (i64.add (local.get $key) (i64.const 2)) (i32.const 111))
        (local.set $value (call $config (local.get $key)))
        (call $output (local.get $value) (call $length (local.get $value))) (i32.const 0))
    "#,
    );
    wat.push_str(&output(
        "browse_media_v2",
        r#"{"items":[],"nextOffset":null}"#,
        "",
    ));
    let item = r#"{"item":{"itemId":"item","title":"0","isContainer":false,"playable":true}}"#;
    let fixed = output("get_media_item_v2", item, "");
    let at = fixed.find("(call $output").unwrap();
    let digit = item.find("\"title\":\"0").unwrap() + "\"title\":\"".len();
    wat.push_str(&fixed[..at]);
    wat.push_str(&format!("(call $store (i64.add (local.get $p) (i64.const {digit})) (i32.add (i32.const 48) (global.get $releases)))"));
    wat.push_str(&fixed[at..]);
    wat.push_str(&output(
        "get_audio_options_v2",
        r#"{"itemId":"item","versions":[]}"#,
        "",
    ));
    wat.push_str(&output(
        "resolve_audio_v2",
        &json!({
            "requestId":"attempt","requestedOffsetMs":0,"url":"http://127.0.0.1/audio",
            "headers":[],"mimeType":"audio/mpeg","formatExt":"mp3","actualPin":{"mediaSourceId":"version","audioStreamIndex":0},
            "start":{"type":"requestedOnly"},"timeline":{"type":"unknown"},
            "seekPolicy":"serverOffset","cachePolicy":"noPersistentCache","leaseId":"lease"
        })
        .to_string(),
        "",
    ));
    // Compare the complete release request. This fixture resolves only account A.
    let expected = serde_json::to_string(&wire::ReleaseRequest {
        request_id: "attempt".into(),
        identity: wire_identity("a"),
        item_id: "item".into(),
        lease_id: Some("lease".into()),
    })
    .unwrap();
    let mut prefix = format!("(global.set $releases (i32.add (global.get $releases) (i32.const 1))) (if (i64.ne (call $input_length) (i64.const {})) (then (return (i32.const 1))))", expected.len());
    for (index, byte) in expected.bytes().enumerate() {
        prefix.push_str(&format!("(if (i32.ne (call $input (i64.const {index})) (i32.const {byte})) (then (return (i32.const 1))))"));
    }
    prefix.push_str(&format!("(if (i32.le_u (global.get $releases) (i32.const {release_failures})) (then (return (i32.const 1))))"));
    wat.push_str(&output(
        "release_audio_v2",
        r#"{"requestId":"attempt","released":true}"#,
        &prefix,
    ));
    wat.push(')');
    std::fs::write(dir.join("plugin.wat"), wat).unwrap();
    std::fs::write(
        dir.join("plugin.json"),
        json!({
            "id":"account.fixture","name":"Account fixture","entry":"plugin.wat",
            "capabilities":["source"],"sourceApiVersion":2,
            "permissions":{"config":{"who":serde_json::to_string(&wire_identity("a")).unwrap()}}
        })
        .to_string(),
    )
    .unwrap();
}
fn wire_identity(user: &str) -> wire::SourceIdentity {
    wire::SourceIdentity {
        server_id: "server".into(),
        user_id: user.into(),
    }
}
fn source(row: &SourceAccount) -> SourceRef {
    SourceRef {
        resolver: SourceRef::extension_resolver(&row.extension_id).unwrap(),
        account_id: row.id.clone(),
        remote: RemoteIdentity {
            server_id: row.remote_server_id.clone().unwrap(),
            user_id: row.remote_user_id.clone().unwrap(),
        },
        kind: ResourceKind::Item,
        item_id: "item".into(),
    }
}
fn request() -> wire::ResolveRequest {
    wire::ResolveRequest {
        request_id: "attempt".into(),
        identity: wire_identity("a"),
        item_id: "item".into(),
        offset_ms: 0,
        selection: wire::AudioSelection::Auto {},
        format_exts: vec!["mp3".into()],
        follow_redirects: true,
    }
}
async fn setup(failures: u32) -> (Directory, ExtensionAccounts, SourceAccount) {
    let directory = Directory::new();
    write_plugin(&directory, failures);
    let connection = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
    Migrator::up(&connection, None).await.unwrap();
    let manager = ExtensionAccounts::new(
        Database { connection },
        vec![directory.0.clone()],
        HostContext::default(),
    );
    let row = manager
        .create("account.fixture", "A", BTreeMap::new(), "t0")
        .await
        .unwrap();
    (directory, manager, row)
}
fn pause(
    manager: &ExtensionAccounts,
    point: TestPoint,
) -> (oneshot::Receiver<()>, oneshot::Sender<()>) {
    let (entered, rx) = oneshot::channel();
    let (tx, resume) = oneshot::channel();
    let mut hooks = manager.inner.hooks.lock().unwrap();
    let hook = Some(TestPause { entered, resume });
    match point {
        TestPoint::Identity => hooks.identity = hook,
        TestPoint::Delivery => hooks.delivery = hook,
    }
    (rx, tx)
}

#[tokio::test]
async fn distinct_declared_configs_and_restart_bind_to_saved_accounts() {
    bounded(async {
        let (directory, manager, a) = setup(0).await;
        let a = manager.connect(&a.id).await.unwrap();
        let b = manager
            .create(
                "account.fixture",
                "B",
                BTreeMap::from([(
                    "who".into(),
                    serde_json::to_string(&wire_identity("b")).unwrap(),
                )]),
                "t0",
            )
            .await
            .unwrap();
        let b = manager.connect(&b.id).await.unwrap();
        assert_ne!(a.id, b.id);
        assert_eq!(a.remote_user_id.as_deref(), Some("a"));
        assert_eq!(b.remote_user_id.as_deref(), Some("b"));
        let handle = manager.for_source(&source(&a)).await.unwrap();
        assert_eq!(handle.item(&source(&a)).await.unwrap().item.title, "0");
        assert!(handle
            .audio_options(&source(&a))
            .await
            .unwrap()
            .versions
            .is_empty());
        assert!(handle
            .browse(None, None, 0, 10)
            .await
            .unwrap()
            .items
            .is_empty());
        assert_eq!(
            handle.item(&source(&b)).await.err(),
            Some(AccountError::Identity)
        );
        let db = manager.inner.db.clone();
        drop(handle);
        drop(manager);
        let restarted =
            ExtensionAccounts::new(db, vec![directory.0.clone()], HostContext::default());
        assert_eq!(
            restarted
                .for_source(&source(&b))
                .await
                .unwrap()
                .remote_identity()
                .unwrap()
                .user_id,
            "b"
        );
    })
    .await;
}

#[tokio::test]
async fn missing_unbound_and_wrong_pair_never_load_guest_and_changed_actual_pair_is_rejected() {
    bounded(async {
        let (_directory, manager, row) = setup(0).await;
        assert_eq!(
            manager.for_account("missing").await.err().unwrap(),
            AccountError::Missing
        );
        assert_eq!(
            manager.for_account(&row.id).await.err().unwrap(),
            AccountError::Identity
        );
        assert!(manager.inner.state.lock().unwrap().slots.is_empty());
        let bound = manager.connect(&row.id).await.unwrap();
        let mut wrong = source(&bound);
        wrong.remote.user_id = "other".into();
        assert_eq!(
            manager.for_source(&wrong).await.err().unwrap(),
            AccountError::Identity
        );
        let changed = manager
            .update(
                &bound,
                "A",
                BTreeMap::from([(
                    "who".into(),
                    ConfigChange::Set(serde_json::to_string(&wire_identity("b")).unwrap()),
                )]),
                "t1",
            )
            .await
            .unwrap();
        assert_eq!(
            manager.for_source(&source(&bound)).await.err().unwrap(),
            AccountError::Identity
        );
        assert_eq!(changed.remote_user_id.as_deref(), Some("a"));
        assert_eq!(
            accounts::get(manager.inner.db.get_connection(), &row.id)
                .await
                .unwrap()
                .unwrap()
                .remote_user_id
                .as_deref(),
            Some("a")
        );
    })
    .await;
}

#[tokio::test]
async fn edit_or_delete_after_guest_identity_prevents_binding_and_publication() {
    bounded(async {
        for delete in [false, true] {
            let (_directory, manager, row) = setup(0).await;
            let (entered, resume) = pause(&manager, TestPoint::Identity);
            let task_manager = manager.clone();
            let id = row.id.clone();
            let task = tokio::spawn(async move { task_manager.connect(&id).await });
            entered.await.unwrap();
            if delete {
                manager.remove(&row.id).await.unwrap();
            } else {
                manager
                    .update(
                        &row,
                        "Edited",
                        BTreeMap::from([(
                            "who".into(),
                            ConfigChange::Set(serde_json::to_string(&wire_identity("b")).unwrap()),
                        )]),
                        "t1",
                    )
                    .await
                    .unwrap();
            }
            resume.send(()).unwrap();
            assert_eq!(task.await.unwrap().err(), Some(AccountError::Changed));
            let stored = accounts::get(manager.inner.db.get_connection(), &row.id)
                .await
                .unwrap();
            if delete {
                assert!(stored.is_none());
            } else {
                assert!(stored.unwrap().remote_server_id.is_none());
            }
        }
    })
    .await;
}

#[tokio::test]
async fn cancelled_blocking_load_keeps_budget_and_cannot_publish_after_rescan() {
    bounded(async {
        let (_directory, manager, row) = setup(0).await;
        let (entered_tx, entered) = oneshot::channel();
        let (resume, wait) = mpsc::sync_channel(1);
        manager.inner.hooks.lock().unwrap().load = Some(Box::new(move || {
            let _ = entered_tx.send(());
            wait.recv_timeout(Duration::from_secs(5))
                .expect("load gate watchdog");
        }));
        let task_manager = manager.clone();
        let id = row.id.clone();
        let task = tokio::spawn(async move { task_manager.connect(&id).await });
        entered.await.unwrap();
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        assert_eq!(LOADS.get().unwrap().available_permits(), 1);
        assert_eq!(manager.inner.state.lock().unwrap().slots.len(), 1);
        manager.rescan().await.unwrap();
        resume.send(()).unwrap();
        eventually(|| manager.inner.state.lock().unwrap().slots.is_empty()).await;
        eventually(|| LOADS.get().unwrap().available_permits() == 2).await;
        assert!(accounts::get(manager.inner.db.get_connection(), &row.id)
            .await
            .unwrap()
            .unwrap()
            .remote_user_id
            .is_none());
    })
    .await;
}

#[tokio::test]
async fn stale_resolve_delivery_releases_original_lease_without_delivering_success() {
    bounded(async {
        let (_directory, manager, row) = setup(0).await;
        let row = manager.connect(&row.id).await.unwrap();
        let handle = manager.for_source(&source(&row)).await.unwrap();
        let (entered, resume) = pause(&manager, TestPoint::Delivery);
        let reference = source(&row);
        let worker = handle.clone();
        let task = tokio::spawn(async move { worker.resolve(&reference, request()).await });
        entered.await.unwrap(); // real WAT returned a lease; manager has not published it
        assert_eq!(handle.entry.runtime.status().reserved, 1);
        manager.rescan().await.unwrap();
        resume.send(()).unwrap();
        assert_eq!(task.await.unwrap().err(), Some(AccountError::Changed));
        eventually(|| handle.entry.runtime.status().reserved == 0).await;
        assert!(handle.entry.runtime.status().disabled);
        let current = manager.for_account(&row.id).await.unwrap();
        assert_eq!(current.item(&source(&row)).await.unwrap().item.title, "0");
    })
    .await;
}

#[tokio::test]
async fn failed_cleanup_survives_disable_and_retries_original_instance_only() {
    bounded(async {
        let (_directory, manager, row) = setup(3).await;
        let row = manager.connect(&row.id).await.unwrap();
        let handle = manager.for_source(&source(&row)).await.unwrap();
        let resolved = handle.resolve(&source(&row), request()).await.unwrap();
        drop(resolved);
        eventually(|| handle.entry.runtime.status().failed_cleanup == 1).await;
        manager
            .set_enabled("account.fixture", false, "t1")
            .await
            .unwrap();
        assert_eq!(
            manager.for_account(&row.id).await.err().unwrap(),
            AccountError::Disabled
        );
        assert!(manager
            .status()
            .iter()
            .any(|s| s.runtime.failed_cleanup == 1 && s.retired));
        // WAT fails its first THREE releases; this succeeds only if the exact
        // original instance's counter survives, not a newly loaded replacement.
        assert_eq!(manager.retry_failed_cleanup(&row.id).await.unwrap(), 1);
        assert_eq!(handle.entry.runtime.status().reserved, 0);
        assert!(handle.entry.runtime.status().disabled);
        assert_eq!(
            handle.item(&source(&row)).await.err(),
            Some(AccountError::Changed)
        );
        manager
            .set_enabled("account.fixture", true, "t2")
            .await
            .unwrap();
        let current = manager.for_source(&source(&row)).await.unwrap();
        assert_eq!(current.item(&source(&row)).await.unwrap().item.title, "0");
    })
    .await;
}

#[tokio::test]
async fn rename_noop_and_failed_cas_preserve_pending_resolve_and_original_handle() {
    bounded(async {
        let (_directory, manager, row) = setup(0).await;
        let row = manager.connect(&row.id).await.unwrap();
        let handle = manager.for_source(&source(&row)).await.unwrap();
        let (entered, resume) = pause(&manager, TestPoint::Delivery);
        let worker = handle.clone();
        let reference = source(&row);
        let task = tokio::spawn(async move { worker.resolve(&reference, request()).await });
        entered.await.unwrap();
        let renamed = manager
            .update(&row, "Renamed", BTreeMap::new(), "t1")
            .await
            .unwrap();
        assert_eq!(renamed.config_revision, row.config_revision);
        let noop = manager
            .update(
                &renamed,
                "Renamed",
                BTreeMap::from([("absent".into(), ConfigChange::Remove)]),
                "t2",
            )
            .await
            .unwrap();
        assert_eq!(noop.config_revision, row.config_revision);
        // Old name is part of storage CAS. Failure must not retire a runtime.
        assert_eq!(
            manager
                .update(&row, "Stale", BTreeMap::new(), "t3")
                .await
                .err(),
            Some(AccountError::Changed)
        );
        assert!(handle.entry.valid.load(Ordering::SeqCst));
        assert!(!handle.entry.runtime.status().disabled);
        resume.send(()).unwrap();
        let resolved = task.await.unwrap().unwrap();
        assert_eq!(resolved.response.request_id, "attempt");
        drop(resolved);
        eventually(|| handle.entry.runtime.status().reserved == 0).await;
        assert_eq!(handle.item(&source(&row)).await.unwrap().item.title, "1");
        let current = manager.for_account(&row.id).await.unwrap();
        assert!(Arc::ptr_eq(&handle.entry, &current.entry));
    })
    .await;
}

#[tokio::test]
async fn uninstalled_source_can_be_renamed_without_losing_cas_binding_or_pending_resolve() {
    bounded(async {
        let (directory, manager, row) = setup(0).await;
        let row = manager.connect(&row.id).await.unwrap();
        let handle = manager.for_source(&source(&row)).await.unwrap();
        let (entered, resume) = pause(&manager, TestPoint::Delivery);
        let reference = source(&row);
        let worker = handle.clone();
        let task = tokio::spawn(async move { worker.resolve(&reference, request()).await });
        entered.await.unwrap();
        std::fs::remove_file(directory.0.join("plugin/plugin.json")).unwrap();

        let renamed = manager
            .update(&row, "Renamed offline", BTreeMap::new(), "t1")
            .await
            .unwrap();
        let mut expected = row.clone();
        expected.name = "Renamed offline".into();
        expected.updated_at = "t1".into();
        assert_eq!(renamed, expected);
        assert_eq!(
            manager
                .update(&row, "Stale", BTreeMap::new(), "t2")
                .await
                .err(),
            Some(AccountError::Changed)
        );
        assert_eq!(
            manager
                .update(
                    &renamed,
                    "Changed config",
                    BTreeMap::from([("who".into(), ConfigChange::Set(String::new()))]),
                    "t3",
                )
                .await
                .err(),
            Some(AccountError::Load)
        );
        assert_eq!(manager.get(&row.id).await.unwrap(), renamed);
        assert!(handle.entry.valid.load(Ordering::SeqCst));
        assert!(!handle.entry.runtime.status().disabled);
        resume.send(()).unwrap();
        let resolved = task.await.unwrap().unwrap();
        assert_eq!(resolved.response.request_id, "attempt");
        drop(resolved);
        eventually(|| handle.entry.runtime.status().reserved == 0).await;
        let current = manager.for_account(&row.id).await.unwrap();
        assert!(Arc::ptr_eq(&handle.entry, &current.entry));

        manager.rescan().await.unwrap();
        manager
            .set_enabled("account.fixture", false, "t4")
            .await
            .unwrap();
        let offline = manager
            .update(&renamed, "Still editable", BTreeMap::new(), "t5")
            .await
            .unwrap();
        assert_eq!(offline.config_revision, renamed.config_revision);
        assert_eq!(offline.remote_server_id, renamed.remote_server_id);
        assert_eq!(offline.remote_user_id, renamed.remote_user_id);
    })
    .await;
}

#[tokio::test]
async fn invalid_manifest_contract_is_rejected_before_instantiation() {
    bounded(async {
        for legacy in [false, true] {
            let (directory, manager, row) = setup(0).await;
            // Broken module distinguishes preflight rejection from load failure.
            std::fs::write(directory.0.join("plugin/plugin.wat"), "invalid wasm").unwrap();
            let row = if legacy {
                let path = directory.0.join("plugin/plugin.json");
                let mut manifest: serde_json::Value =
                    serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
                manifest["sourceApiVersion"] = json!(1);
                std::fs::write(path, manifest.to_string()).unwrap();
                row
            } else {
                // Simulate a persisted configuration whose declaration was removed.
                accounts::update(
                    manager.inner.db.get_connection(),
                    &row,
                    "A",
                    &BTreeMap::from([("undeclared".into(), ConfigChange::Set(String::new()))]),
                    "t1",
                )
                .await
                .unwrap()
            };
            assert_eq!(
                manager.connect(&row.id).await.err(),
                Some(AccountError::InvalidSource)
            );
            assert!(manager.inner.state.lock().unwrap().slots.is_empty());
        }
    })
    .await;
}
