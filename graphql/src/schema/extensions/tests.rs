//! Actual GraphQL mutations, migrated SQLite and a real WAT source instance.
//! Only catalog directory configuration is substituted; no manager is mocked.
use super::*;
use migration::{Migrator, MigratorTrait};
use music_player_extensions::host::HostContext;
use music_player_playback::{
    extension_accounts::{AccountError, ExtensionAccounts},
    source_resolver::SourceResolver,
};
use music_player_types::source::{RemoteIdentity, ResourceKind, SourceRef};
use serde_json::{json, Value};
use std::{collections::BTreeMap, future::Future, time::Duration};

pub(super) struct CatalogPaths(pub Vec<PathBuf>);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("mp-graphql-extensions-{}", cuid::cuid2()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const SOURCE: &str = "graphql.source.fixture";
const METADATA: &str = "graphql.metadata.fixture";
const MISSING: &str = "graphql.removed.fixture";
const NOW: &str = "2026-10-01T00:00:00Z";
type Api = Schema<ExtensionsQuery, ExtensionsMutation, EmptySubscription>;

async fn bounded(future: impl Future<Output = ()>) {
    tokio::time::timeout(Duration::from_secs(30), future)
        .await
        .expect("extension mutation scenario deadline");
}

async fn database() -> Database {
    let mut options = sea_orm::ConnectOptions::new("sqlite::memory:");
    options.max_connections(1).sqlx_logging(false);
    let connection = sea_orm::Database::connect(options).await.unwrap();
    Migrator::up(&connection, None).await.unwrap();
    Database { connection }
}

fn output(export: &str, body: &str) -> String {
    let mut wat = format!(
        "(func (export \"{export}\") (result i32) (local $p i64) (local.set $p (call $alloc (i64.const {})))",
        body.len()
    );
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

fn fixtures(root: &Directory) {
    let source = root.0.join("source");
    std::fs::create_dir(&source).unwrap();
    let mut wat = String::from(
        r#"(module
        (import "extism:host/env" "alloc" (func $alloc (param i64) (result i64)))
        (import "extism:host/env" "store_u8" (func $store (param i64 i32)))
        (import "extism:host/env" "output_set" (func $output (param i64 i64)))"#,
    );
    for (export, body) in [
        (
            "source_identity_v2",
            r#"{"serverId":"server","userId":"user"}"#,
        ),
        (
            "get_media_item_v2",
            r#"{"item":{"itemId":"item","title":"Guest item","isContainer":false,"playable":true}}"#,
        ),
        ("browse_media_v2", r#"{"items":[],"nextOffset":null}"#),
        ("get_audio_options_v2", r#"{"itemId":"item","versions":[]}"#),
        // Never called: this test observes identity/item calls, not playback.
        ("resolve_audio_v2", "{}"),
        ("release_audio_v2", "{}"),
    ] {
        wat.push_str(&output(export, body));
    }
    wat.push(')');
    std::fs::write(source.join("plugin.wat"), wat).unwrap();
    std::fs::write(
        source.join("plugin.json"),
        json!({
            "id":SOURCE, "name":"GraphQL source fixture", "entry":"plugin.wat",
            "capabilities":["source"], "sourceApiVersion":2
        })
        .to_string(),
    )
    .unwrap();
    let metadata = root.0.join("metadata");
    std::fs::create_dir(&metadata).unwrap();
    // Catalog validation requires the declared entry file even without loading it.
    std::fs::write(metadata.join("plugin.wasm"), b"\0asm\x01\0\0\0").unwrap();
    std::fs::write(
        metadata.join("plugin.json"),
        json!({
            "id":METADATA, "name":"GraphQL metadata fixture", "capabilities":["metadata"]
        })
        .to_string(),
    )
    .unwrap();
    music_player_extensions::manifest::Manifest::load(&metadata).unwrap();
}

fn api(db: Database, root: &Directory, resolver: Option<SourceResolver>) -> Api {
    let mut builder = Schema::build(ExtensionsQuery, ExtensionsMutation, EmptySubscription)
        .data(db)
        .data(CatalogPaths(vec![root.0.clone()]));
    if let Some(resolver) = resolver {
        builder = builder.data(resolver);
    }
    builder.finish()
}

fn toggle(id: &str, enabled: bool) -> Request {
    Request::new("mutation($id: ID!, $enabled: Boolean!) { setExtensionEnabled(id: $id, enabled: $enabled) { id status } }")
        .variables(Variables::from_json(json!({"id":id,"enabled":enabled})))
}

async fn success(api: &Api, request: impl Into<Request>) -> Value {
    let response = api.execute(request).await;
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    response.data.into_json().unwrap()
}

#[tokio::test]
async fn mutations_invalidate_the_shared_manager_and_prune_only_missing_flags() {
    bounded(async {
        let directory = Directory::new();
        fixtures(&directory);
        let db = database().await;
        let manager = ExtensionAccounts::new(
            db.clone(),
            vec![directory.0.clone()],
            HostContext::default(),
        );
        let row = manager
            .create(SOURCE, "Fixture account", BTreeMap::new(), NOW)
            .await
            .unwrap();
        let bound = manager.connect(&row.id).await.unwrap();
        assert_eq!(bound.remote_server_id.as_deref(), Some("server"));
        assert_eq!(bound.remote_user_id.as_deref(), Some("user"));
        let source = SourceRef {
            resolver: SourceRef::extension_resolver(SOURCE).unwrap(),
            account_id: row.id,
            remote: RemoteIdentity {
                server_id: "server".into(),
                user_id: "user".into(),
            },
            kind: ResourceKind::Item,
            item_id: "item".into(),
        };
        let old = manager.for_source(&source).await.unwrap();
        assert_eq!(old.item(&source).await.unwrap().item.title, "Guest item");
        let resolver = SourceResolver::new(db.clone(), "graphql-fixture".into(), true)
            .with_extensions(manager.clone(), vec!["mp3".into()]);
        let api = api(db.clone(), &directory, Some(resolver));

        assert_eq!(
            success(&api, toggle(SOURCE, false)).await["setExtensionEnabled"]["status"],
            "disabled"
        );
        assert!(matches!(
            old.item(&source).await,
            Err(AccountError::Changed)
        ));
        assert!(matches!(
            manager.for_source(&source).await,
            Err(AccountError::Disabled)
        ));
        assert_eq!(
            extension_state::enabled_map(db.get_connection())
                .await
                .get(SOURCE),
            Some(&false)
        );

        success(&api, toggle(SOURCE, true)).await;
        // Re-enable must not resurrect a retained handle from the old generation.
        assert!(matches!(
            old.item(&source).await,
            Err(AccountError::Changed)
        ));
        let current = manager.for_source(&source).await.unwrap();
        assert_eq!(
            current.item(&source).await.unwrap().item.title,
            "Guest item"
        );
        assert!(!extension_state::enabled_map(db.get_connection())
            .await
            .contains_key(SOURCE));

        // A non-source plugin still supports ordinary persisted toggles/listing.
        assert_eq!(
            success(&api, toggle(METADATA, false)).await["setExtensionEnabled"]["status"],
            "disabled"
        );
        // Seed a removed installation preference through the same manager.
        manager.set_enabled(MISSING, false, NOW).await.unwrap();
        let data = success(&api, "mutation { rescanExtensions { id status } }").await;
        assert!(data["rescanExtensions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["id"] == METADATA && row["status"] == "disabled"));
        let flags = extension_state::enabled_map(db.get_connection()).await;
        assert_eq!(flags.get(METADATA), Some(&false));
        assert!(!flags.contains_key(MISSING));
        // The actual manager held outside GraphQL sees the rescan invalidation.
        assert!(matches!(
            current.item(&source).await,
            Err(AccountError::Changed)
        ));
        let after = manager.for_source(&source).await.unwrap();
        assert_eq!(after.item(&source).await.unwrap().item.title, "Guest item");
    })
    .await;
}

#[tokio::test]
async fn missing_manager_rejects_mutations_without_database_fallback_but_keeps_listing() {
    bounded(async {
        let directory = Directory::new();
        fixtures(&directory);
        let db = database().await;
        extension_state::set_enabled(db.get_connection(), MISSING, false, NOW)
            .await
            .unwrap();
        for (resolver, expected) in [
            (None, "Shared source resolver is not configured"),
            (
                Some(SourceResolver::new(db.clone(), "native-only".into(), true)),
                "extension source runtime is not configured",
            ),
        ] {
            let api = api(db.clone(), &directory, resolver);
            let listing = success(&api, "{ extensions(filter: \"metadata\") { id } }").await;
            assert_eq!(listing["extensions"], json!([{"id":METADATA}]));
            for request in [
                toggle(SOURCE, false),
                Request::new("mutation { rescanExtensions { id } }"),
            ] {
                let response = api.execute(request).await;
                assert_eq!(response.errors.len(), 1);
                assert!(response.errors[0].message.contains(expected));
            }
            let flags = extension_state::enabled_map(db.get_connection()).await;
            assert!(
                !flags.contains_key(SOURCE),
                "failed mutation must not store a toggle"
            );
            assert_eq!(
                flags.get(MISSING),
                Some(&false),
                "failed rescan must not prune"
            );
        }
    })
    .await;
}
