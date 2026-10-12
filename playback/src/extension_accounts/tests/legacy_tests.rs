//! Legacy plugin binding exercises actual WAT calls and SQLite rows in CI.
use super::*;

// Observe guest config through a real export, not Extension::config or the DB.
// Hex keeps arbitrary config bytes JSON-safe without baking expected values into
// the module. The same WAT is installed for every manifest default/override.
fn config_source_info() -> String {
    let prefix = r#"{"name":"Legacy fixture","description":"server-hex:"#;
    let mut code = String::from(
        r#"(func $hex (param $n i32) (result i32)
          (if (result i32) (i32.lt_u (local.get $n) (i32.const 10))
            (then (i32.add (local.get $n) (i32.const 48)))
            (else (i32.add (local.get $n) (i32.const 87)))))
        (func (export "source_info") (result i32)
          (local $key i64) (local $value i64) (local $len i64)
          (local $p i64) (local $i i64) (local $end i64) (local $byte i32)
          (local.set $key (call $alloc (i64.const 6)))
        "#,
    );
    for (index, byte) in b"server".iter().enumerate() {
        code.push_str(&format!(
            "(call $store (i64.add (local.get $key) (i64.const {index})) (i32.const {byte}))"
        ));
    }
    // The app's get_config returns null for an empty allocation: never read
    // length or bytes from that null handle.
    code.push_str(&format!(
        r#"(local.set $value (call $config (local.get $key)))
          (if (i64.ne (local.get $value) (i64.const 0))
            (then (local.set $len (call $length (local.get $value)))))
          (local.set $end (i64.add (i64.const {}) (i64.mul (local.get $len) (i64.const 2))))
          (local.set $p (call $alloc (i64.add (local.get $end) (i64.const 2))))
        "#,
        prefix.len()
    ));
    for (index, byte) in prefix.bytes().enumerate() {
        code.push_str(&format!(
            "(call $store (i64.add (local.get $p) (i64.const {index})) (i32.const {byte}))"
        ));
    }
    code.push_str(&format!(
        r#"(block $done (loop $copy
            (br_if $done (i64.ge_u (local.get $i) (local.get $len)))
            (local.set $byte (call $load (i64.add (local.get $value) (local.get $i))))
            (call $store
              (i64.add (local.get $p) (i64.add (i64.const {prefix_len}) (i64.mul (local.get $i) (i64.const 2))))
              (call $hex (i32.shr_u (local.get $byte) (i32.const 4))))
            (call $store
              (i64.add (local.get $p) (i64.add (i64.const {low_offset}) (i64.mul (local.get $i) (i64.const 2))))
              (call $hex (i32.and (local.get $byte) (i32.const 15))))
            (local.set $i (i64.add (local.get $i) (i64.const 1)))
            (br $copy)))
          (call $store (i64.add (local.get $p) (local.get $end)) (i32.const 34))
          (call $store (i64.add (local.get $p) (i64.add (local.get $end) (i64.const 1))) (i32.const 125))
          (if (i64.ne (local.get $value) (i64.const 0))
            (then (call $free (local.get $value))))
          (call $output (local.get $p) (i64.add (local.get $end) (i64.const 2)))
          (i32.const 0))
        "#,
        prefix_len = prefix.len(),
        low_offset = prefix.len() + 1
    ));
    code
}

async fn assert_guest_server(handle: &AccountRuntime, expected_hex: &str) {
    let info = handle.legacy_info().await.unwrap();
    assert_eq!(info.name, "Legacy fixture");
    assert_eq!(info.description, format!("server-hex:{expected_hex}"));
}

fn write_legacy_plugin(directory: &Directory, default: &str) {
    let dir = directory.0.join("plugin");
    std::fs::create_dir_all(&dir).unwrap();
    let mut wat = String::from(
        r#"(module
      (import "extism:host/env" "alloc" (func $alloc (param i64) (result i64)))
      (import "extism:host/env" "store_u8" (func $store (param i64 i32)))
      (import "extism:host/env" "output_set" (func $output (param i64 i64)))
      (import "extism:host/user" "get_config" (func $config (param i64) (result i64)))
      (import "extism:host/env" "length" (func $length (param i64) (result i64)))
      (import "extism:host/env" "load_u8" (func $load (param i64) (result i32)))
      (import "extism:host/env" "free" (func $free (param i64)))
      (import "extism:host/env" "input_length" (func $input_length (result i64)))
      (import "extism:host/env" "input_load_u8" (func $input (param i64) (result i32)))
      (global $calls (mut i32) (i32.const 0))
    "#,
    );
    wat.push_str(&config_source_info());
    let track = r#"[{"id":"item","title":"0","artist":"Fixture"}]"#;
    let code = output(
        "list_tracks",
        track,
        "(global.set $calls (i32.add (global.get $calls) (i32.const 1)))",
    );
    let at = code.find("(call $output").unwrap();
    let digit = track.find("\"title\":\"0").unwrap() + "\"title\":\"".len();
    wat.push_str(&code[..at]);
    wat.push_str(&format!("(call $store (i64.add (local.get $p) (i64.const {digit})) (i32.add (i32.const 48) (global.get $calls)))"));
    wat.push_str(&code[at..]);
    let expected = serde_json::to_string(&legacy::StreamRequest {
        track_id: "item".into(),
    })
    .unwrap();
    let mut check = format!(
        "(if (i64.ne (call $input_length) (i64.const {})) (then (return (i32.const 1))))",
        expected.len()
    );
    for (index, byte) in expected.bytes().enumerate() {
        check.push_str(&format!("(if (i32.ne (call $input (i64.const {index})) (i32.const {byte})) (then (return (i32.const 1))))"));
    }
    wat.push_str(&output(
        "get_stream_url",
        r#"{"url":"http://127.0.0.1/audio","mimeType":"audio/mpeg"}"#,
        &check,
    ));
    wat.push(')');
    std::fs::write(dir.join("plugin.wat"), wat).unwrap();
    std::fs::write(
        dir.join("plugin.json"),
        json!({
            "id":"account.fixture","name":"Legacy fixture","entry":"plugin.wat",
            "capabilities":["source"],"sourceApiVersion":1,
            "permissions":{"config":{"server":default}}
        })
        .to_string(),
    )
    .unwrap();
}
async fn setup_legacy() -> (Directory, ExtensionAccounts, SourceAccount) {
    let directory = Directory::new();
    write_legacy_plugin(&directory, "A");
    let connection = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
    Migrator::up(&connection, None).await.unwrap();
    let manager = ExtensionAccounts::new(
        Database { connection },
        vec![directory.0.clone()],
        HostContext::default(),
    );
    let row = manager
        .create("account.fixture", "Legacy", BTreeMap::new(), "t0")
        .await
        .unwrap();
    (directory, manager, row)
}
fn local(row: &SourceAccount) -> LocalSourceRef {
    LocalSourceRef {
        resolver: SourceRef::extension_resolver(&row.extension_id).unwrap(),
        account_id: row.id.clone(),
        config_revision: row.config_revision as u64,
        kind: ResourceKind::Item,
        item_id: "item".into(),
    }
}

#[tokio::test]
async fn legacy_binding_browse_resolve_and_restart_keep_local_identity() {
    bounded(async {
        let (directory, manager, unbound) = setup_legacy().await;
        assert_eq!(unbound.source_api_version, 1);
        assert_eq!(
            manager.for_account(&unbound.id).await.err(),
            Some(AccountError::Identity)
        );
        assert!(manager.status().is_empty());
        let row = manager.connect(&unbound.id).await.unwrap();
        assert!(row.remote_server_id.is_none() && row.remote_user_id.is_none());
        assert_eq!(row.local_bound_revision, Some(1));
        let saved = local(&row);
        let handle = manager.for_local_source(&saved).await.unwrap();
        assert_guest_server(&handle, "41").await; // manifest default A
        assert_eq!(handle.remote_identity(), Err(AccountError::Identity));
        assert_eq!(
            handle
                .source_handle(ResourceKind::Item, "item".into())
                .unwrap(),
            SourceHandle::Local(saved.clone())
        );
        assert_eq!(
            handle.legacy_tracks(None, None, 0, 10).await.unwrap()[0].title,
            "1"
        );
        // Missing optional list exports remain valid v1 empty pages.
        assert!(handle
            .legacy_albums(None, None, 0, 10)
            .await
            .unwrap()
            .is_empty());
        assert!(handle
            .legacy_artists(None, None, 0, 10)
            .await
            .unwrap()
            .is_empty());
        assert!(handle
            .legacy_playlists(None, None, 0, 10)
            .await
            .unwrap()
            .is_empty());
        assert_eq!(
            handle
                .legacy_tracks(None, None, u64::from(u32::MAX) + 1, 10)
                .await
                .err(),
            Some(AccountError::InvalidSource)
        );
        let resolved = handle.resolve_legacy(&saved).await.unwrap();
        assert_eq!(resolved.response.mime_type, "audio/mpeg");
        assert_eq!(handle.entry.runtime.status().reserved, 1);
        drop(resolved);
        eventually(|| handle.entry.runtime.status().reserved == 0).await;
        let other = manager
            .create("account.fixture", "Other", BTreeMap::new(), "t0")
            .await
            .unwrap();
        let other = manager.connect(&other.id).await.unwrap();
        assert_eq!(
            handle.resolve_legacy(&local(&other)).await.err(),
            Some(AccountError::Identity)
        );
        let renamed = manager
            .update(&row, "Renamed", BTreeMap::new(), "t1")
            .await
            .unwrap();
        assert_eq!(renamed.local_bound_revision, row.local_bound_revision);
        assert_eq!(
            handle.legacy_tracks(None, None, 0, 10).await.unwrap()[0].title,
            "2"
        );
        let restarted = ExtensionAccounts::new(
            manager.inner.db.clone(),
            vec![directory.0.clone()],
            HostContext::default(),
        );
        let current = restarted.for_local_source(&saved).await.unwrap();
        assert_guest_server(&current, "41").await;
        assert_eq!(
            current.legacy_tracks(None, None, 0, 10).await.unwrap()[0].title,
            "1"
        );
        assert_eq!(
            current
                .source_handle(ResourceKind::Item, "item".into())
                .unwrap()
                .to_handle(),
            saved.to_handle()
        );
    })
    .await;
}

#[tokio::test]
async fn changed_defaults_require_explicit_reconnect_and_new_scope_even_after_aba() {
    bounded(async {
        let (directory, manager, row) = setup_legacy().await;
        let row = manager.connect(&row.id).await.unwrap();
        let saved = local(&row);
        let old = manager.for_local_source(&saved).await.unwrap();
        assert_guest_server(&old, "41").await;
        write_legacy_plugin(&directory, "B");
        manager.rescan().await.unwrap();
        assert_eq!(
            manager.for_local_source(&saved).await.err(),
            Some(AccountError::Changed)
        );
        assert_eq!(
            old.resolve_legacy(&saved).await.err(),
            Some(AccountError::Changed)
        );
        let before = accounts::get(manager.inner.db.get_connection(), &row.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(before, row); // playback rejection did not rebind
        let bound = manager.connect(&row.id).await.unwrap();
        assert_eq!(bound.config_json, row.config_json);
        assert_eq!(bound.config_revision, 2);
        let effective: BTreeMap<String, String> =
            serde_json::from_str(bound.local_config_json.as_deref().unwrap()).unwrap();
        assert_eq!(effective.get("server").map(String::as_str), Some("B"));
        assert_eq!(
            manager
                .validate_handle(&SourceHandle::Local(saved.clone()))
                .await,
            Err(AccountError::Identity)
        );
        let current = manager.for_local_source(&local(&bound)).await.unwrap();
        assert_guest_server(&current, "42").await; // newly loaded default B
        assert_eq!(
            current.legacy_tracks(None, None, 0, 10).await.unwrap()[0].title,
            "1"
        );
        write_legacy_plugin(&directory, "A");
        manager.rescan().await.unwrap();
        let back = manager.connect(&row.id).await.unwrap();
        assert_eq!(back.config_revision, 3);
        let current = manager.for_local_source(&local(&back)).await.unwrap();
        assert_guest_server(&current, "41").await;
        assert_ne!(local(&back).to_handle(), saved.to_handle());
        assert_eq!(
            manager.for_local_source(&saved).await.err(),
            Some(AccountError::Identity)
        );
    })
    .await;
}

#[tokio::test]
async fn guest_config_overrides_and_empty_values_survive_restart_and_default_changes() {
    bounded(async {
        let (directory, manager, default) = setup_legacy().await;
        let overridden = manager
            .create(
                "account.fixture",
                "Override",
                BTreeMap::from([("server".into(), "override".into())]),
                "t0",
            )
            .await
            .unwrap();
        let empty = manager
            .create(
                "account.fixture",
                "Empty override",
                BTreeMap::from([("server".into(), String::new())]),
                "t0",
            )
            .await
            .unwrap();
        let mut cases = Vec::new();
        // The app's get_config also returns empty for undeclared keys.
        // Against the same nonempty manifest default, this proves that the
        // explicit empty override does not silently fall back to that default.
        for (row, expected) in [
            (default, "41"),
            (overridden, "6f76657272696465"),
            (empty, ""),
        ] {
            let bound = manager.connect(&row.id).await.unwrap();
            let handle = manager.for_local_source(&local(&bound)).await.unwrap();
            assert_guest_server(&handle, expected).await;
            cases.push((bound, expected));
        }

        // Reconstruct the manager from the existing SQLite DB and installation;
        // no connect/rebind is allowed just to read the saved scopes again.
        let db = manager.inner.db.clone();
        drop(manager);
        let restarted =
            ExtensionAccounts::new(db, vec![directory.0.clone()], HostContext::default());
        for (row, expected) in &cases {
            let handle = restarted.for_local_source(&local(row)).await.unwrap();
            assert_guest_server(&handle, expected).await;
        }

        write_legacy_plugin(&directory, "B");
        restarted.rescan().await.unwrap();
        for (row, previous) in &cases {
            let saved = local(row);
            if *previous == "41" {
                assert_eq!(
                    restarted.for_local_source(&saved).await.err(),
                    Some(AccountError::Changed)
                );
                let rebound = restarted.connect(&row.id).await.unwrap();
                assert_eq!(rebound.config_revision, row.config_revision + 1);
                assert_ne!(local(&rebound).to_handle(), saved.to_handle());
                let handle = restarted.for_local_source(&local(&rebound)).await.unwrap();
                assert_guest_server(&handle, "42").await;
            } else {
                // The changed default must not override either account value,
                // including empty; their unchanged effective configs retain scope.
                let handle = restarted.for_local_source(&saved).await.unwrap();
                assert_guest_server(&handle, previous).await;
                let rebound = restarted.connect(&row.id).await.unwrap();
                assert_eq!(rebound.config_revision, row.config_revision);
                assert_eq!(local(&rebound).to_handle(), saved.to_handle());
                assert_guest_server(&handle, previous).await;
            }
        }
    })
    .await;
}

#[tokio::test]
async fn config_edit_suppresses_delayed_legacy_resolve_and_releases_only_its_local_lease() {
    bounded(async {
        let (_directory, manager, row) = setup_legacy().await;
        let row = manager.connect(&row.id).await.unwrap();
        let old = manager.for_local_source(&local(&row)).await.unwrap();
        let b = manager
            .create("account.fixture", "B", BTreeMap::new(), "t0")
            .await
            .unwrap();
        let b = manager.connect(&b.id).await.unwrap();
        let other = manager.for_local_source(&local(&b)).await.unwrap();
        let (entered, resume) = pause(&manager, TestPoint::Delivery);
        let worker = old.clone();
        let saved = local(&row);
        let task = tokio::spawn(async move { worker.resolve_legacy(&saved).await });
        entered.await.unwrap();
        assert_eq!(old.entry.runtime.status().reserved, 1);
        let updated = manager
            .update(
                &row,
                "Legacy",
                BTreeMap::from([("server".into(), ConfigChange::Set("B".into()))]),
                "t1",
            )
            .await
            .unwrap();
        assert_eq!(updated.local_bound_revision, None);
        resume.send(()).unwrap();
        assert_eq!(task.await.unwrap().err(), Some(AccountError::Changed));
        eventually(|| old.entry.runtime.status().reserved == 0).await;
        assert_eq!(old.entry.runtime.status().pending_cleanup, 0);
        assert_eq!(old.entry.runtime.status().failed_cleanup, 0);
        assert_eq!(
            manager.for_account(&row.id).await.err(),
            Some(AccountError::Identity)
        );
        let resolved_b = other.resolve_legacy(&local(&b)).await.unwrap();
        assert!(!other.entry.runtime.status().disabled);
        drop(resolved_b);
        let connected = manager.connect(&row.id).await.unwrap();
        assert_eq!(connected.config_revision, 2);
        let current = manager.for_local_source(&local(&connected)).await.unwrap();
        assert_guest_server(&current, "42").await;
    })
    .await;
}

#[tokio::test]
async fn stale_legacy_connect_cannot_bind_after_config_edit() {
    bounded(async {
        let (_directory, manager, row) = setup_legacy().await;
        let (entered, resume) = pause(&manager, TestPoint::Identity);
        let worker = manager.clone();
        let id = row.id.clone();
        let task = tokio::spawn(async move { worker.connect(&id).await });
        entered.await.unwrap();
        manager
            .update(
                &row,
                &row.name,
                BTreeMap::from([("server".into(), ConfigChange::Set("B".into()))]),
                "t1",
            )
            .await
            .unwrap();
        resume.send(()).unwrap();
        assert_eq!(task.await.unwrap().err(), Some(AccountError::Changed));
        let current = accounts::get(manager.inner.db.get_connection(), &row.id)
            .await
            .unwrap()
            .unwrap();
        assert!(current.local_bound_revision.is_none());
        assert!(current.local_config_json.is_none());
        let bound = manager.connect(&row.id).await.unwrap();
        assert_eq!(bound.local_bound_revision, Some(2));
        let handle = manager.for_local_source(&local(&bound)).await.unwrap();
        assert_guest_server(&handle, "42").await;
    })
    .await;
}

#[tokio::test]
async fn installed_protocol_change_never_promotes_a_local_account_to_remote_identity() {
    bounded(async {
        let (directory, manager, row) = setup_legacy().await;
        let row = manager.connect(&row.id).await.unwrap();
        write_plugin(&directory, 0);
        manager.rescan().await.unwrap();
        assert_eq!(
            manager.for_local_source(&local(&row)).await.err(),
            Some(AccountError::InvalidSource)
        );
        assert_eq!(
            manager.connect(&row.id).await.err(),
            Some(AccountError::InvalidSource)
        );
        let renamed = manager
            .update(&row, "Renamed legacy account", BTreeMap::new(), "t1")
            .await
            .unwrap();
        let mut expected = row.clone();
        expected.name = "Renamed legacy account".into();
        expected.updated_at = "t1".into();
        assert_eq!(renamed, expected);
        assert_eq!(
            manager.for_local_source(&local(&renamed)).await.err(),
            Some(AccountError::InvalidSource)
        );
        assert_eq!(
            manager.connect(&renamed.id).await.err(),
            Some(AccountError::InvalidSource)
        );
        assert_eq!(
            manager
                .update(
                    &renamed,
                    &renamed.name,
                    BTreeMap::from([("who".into(), ConfigChange::Set("different".into()))]),
                    "t2",
                )
                .await
                .err(),
            Some(AccountError::Changed)
        );
        assert_eq!(
            accounts::get(manager.inner.db.get_connection(), &row.id)
                .await
                .unwrap()
                .unwrap(),
            expected
        );
        let new = manager
            .create("account.fixture", "New v2", BTreeMap::new(), "t1")
            .await
            .unwrap();
        assert_eq!(new.source_api_version, 2);
        let new = manager.connect(&new.id).await.unwrap();
        assert_eq!(new.remote_user_id.as_deref(), Some("a"));
        assert!(new.local_bound_revision.is_none());
        assert_ne!(new.id, row.id);
    })
    .await;
}
