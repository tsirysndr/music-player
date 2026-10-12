//! Actual generated client/TCP -> manager -> SQLite/WASM. Executed only in CI.
use super::*;
use crate::api::music::v1alpha1::{
    extensions_service_client::ExtensionsServiceClient,
    extensions_service_server::ExtensionsServiceServer,
};
use migration::{Migrator, MigratorTrait};
use music_player_extensions::host::HostContext;
use music_player_storage::{extension_state, Database};
use serde_json::json;
use std::{collections::BTreeMap, future::Future, time::Duration};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

const PLUGIN: &str = "rpc.extension.catalog";
struct Running(tokio::task::JoinHandle<()>);
impl Drop for Running {
    fn drop(&mut self) {
        self.0.abort();
    }
}

fn install(directory: &tempfile::TempDir) {
    let path = directory.path().join(PLUGIN);
    std::fs::create_dir_all(&path).unwrap();
    let mut wat = String::from(
        r#"(module
      (import "extism:host/env" "alloc" (func $alloc (param i64) (result i64)))
      (import "extism:host/env" "store_u8" (func $store (param i64 i32)))
      (import "extism:host/env" "output_set" (func $output (param i64 i64)))"#,
    );
    for (name, body) in [
        ("source_info", r#"{"name":"RPC catalog fixture"}"#),
        ("get_stream_url", r#"{"url":"http://127.0.0.1/unused"}"#),
    ] {
        wat.push_str(&format!(r#"(func (export "{name}") (result i32) (local $p i64) (local.set $p (call $alloc (i64.const {})))"#, body.len()));
        for (index, byte) in body.bytes().enumerate() {
            wat.push_str(&format!(
                "(call $store (i64.add (local.get $p) (i64.const {index})) (i32.const {byte}))"
            ));
        }
        wat.push_str(&format!(
            "(call $output (local.get $p) (i64.const {})) (i32.const 0))",
            body.len()
        ));
    }
    wat.push(')');
    std::fs::write(path.join("plugin.wat"), wat).unwrap();
    std::fs::write(
        path.join("plugin.json"),
        json!({
            "id": PLUGIN, "name": "RPC catalog fixture", "entry": "plugin.wat",
            "capabilities": ["source"], "sourceApiVersion": 1,
            "permissions": {"config":{"password":"synthetic-catalog-secret"}}
        })
        .to_string(),
    )
    .unwrap();
    // Listing must not instantiate an enabled module, even when it cannot load.
    let invalid = directory.path().join("invalid.module");
    std::fs::create_dir_all(&invalid).unwrap();
    std::fs::write(
        invalid.join("plugin.json"),
        r#"{"id":"invalid.module","name":"Unloaded","capabilities":["events"]}"#,
    )
    .unwrap();
    std::fs::write(invalid.join("plugin.wasm"), b"not-wasm").unwrap();
}

async fn admitted<T, E, F, Fut>(mut call: F, busy: impl Fn(&E) -> bool) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    loop {
        match call().await {
            Err(error) if busy(&error) => tokio::time::sleep(Duration::from_millis(10)).await,
            result => return result,
        }
    }
}
fn account_busy(error: &AccountError) -> bool {
    *error == AccountError::Busy
}
fn rpc_busy(error: &Status) -> bool {
    error.code() == tonic::Code::ResourceExhausted
        && error.message() == "extension account capacity unavailable"
}

#[tokio::test]
async fn remote_toggle_and_rescan_invalidate_the_original_manager() {
    tokio::time::timeout(Duration::from_secs(45), async {
        let directory = tempfile::tempdir().unwrap();
        install(&directory);
        let mut options = sea_orm::ConnectOptions::new("sqlite::memory:");
        options.max_connections(1).sqlx_logging(false);
        let connection = sea_orm::Database::connect(options).await.unwrap();
        Migrator::up(&connection, None).await.unwrap();
        let db = Database { connection };
        let manager = ExtensionAccounts::new(
            db.clone(),
            vec![directory.path().into()],
            HostContext::default(),
        );
        let resolver = SourceResolver::new(db.clone(), "catalog-test".into(), true)
            .with_extensions(manager.clone(), vec!["mp3".into()]);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let _running = Running(tokio::spawn(async move {
            Server::builder()
                .add_service(ExtensionsServiceServer::new(Extensions::new(resolver)))
                .serve_with_incoming(TcpListenerStream::new(listener))
                .await
                .unwrap();
        }));
        let mut client = ExtensionsServiceClient::connect(format!("http://{address}"))
            .await
            .unwrap();
        let list = admitted(
            || {
                let mut client = client.clone();
                async move { client.list_extensions(ListExtensionsRequest {}).await }
            },
            rpc_busy,
        )
        .await
        .unwrap()
        .into_inner()
        .extensions;
        assert_eq!(list.len(), 2);
        assert!(list.iter().all(|entry| entry.enabled));
        assert!(
            manager.status().is_empty(),
            "catalog must not create an account runtime"
        );
        assert!(!format!("{list:?}").contains("synthetic-catalog-secret"));

        let account = admitted(
            || manager.create(PLUGIN, "Fixture", BTreeMap::new(), "t0"),
            account_busy,
        )
        .await
        .unwrap();
        admitted(|| manager.connect(&account.id), account_busy)
            .await
            .unwrap();
        let original = admitted(|| manager.for_account(&account.id), account_busy)
            .await
            .unwrap();
        assert_eq!(
            original.legacy_info().await.unwrap().name,
            "RPC catalog fixture"
        );
        let disabled = admitted(
            || {
                let mut client = client.clone();
                async move {
                    client
                        .set_extension_enabled(SetExtensionEnabledRequest {
                            id: PLUGIN.into(),
                            enabled: false,
                        })
                        .await
                }
            },
            rpc_busy,
        )
        .await
        .unwrap()
        .into_inner()
        .extension
        .unwrap();
        assert!(!disabled.enabled);
        assert!(matches!(
            original.legacy_info().await,
            Err(AccountError::Changed)
        ));
        assert!(matches!(
            manager.for_account(&account.id).await,
            Err(AccountError::Disabled)
        ));
        assert_eq!(
            extension_state::enabled_map(db.get_connection())
                .await
                .get(PLUGIN),
            Some(&false)
        );

        admitted(
            || {
                let mut client = client.clone();
                async move {
                    client
                        .set_extension_enabled(SetExtensionEnabledRequest {
                            id: PLUGIN.into(),
                            enabled: true,
                        })
                        .await
                }
            },
            rpc_busy,
        )
        .await
        .unwrap();
        let replacement = admitted(|| manager.for_account(&account.id), account_busy)
            .await
            .unwrap();
        assert_eq!(
            replacement.legacy_info().await.unwrap().name,
            "RPC catalog fixture"
        );
        // Old handles cannot revive after re-enable.
        assert!(matches!(
            original.legacy_info().await,
            Err(AccountError::Changed)
        ));

        // Remove a manifest on disk as the CLI would; rescan must invalidate
        // the retained runtime, prune preferences, and keep saved accounts.
        std::fs::remove_file(directory.path().join(PLUGIN).join("plugin.json")).unwrap();
        // Rescan may have applied before its catalog response fails. Send it
        // once, as clients do; never replay an uncertain mutation.
        let rescanned = client
            .rescan_extensions(RescanExtensionsRequest {})
            .await
            .unwrap()
            .into_inner()
            .extensions;
        assert_eq!(rescanned.len(), 1);
        assert_eq!(rescanned[0].id, "invalid.module");
        assert!(matches!(
            replacement.legacy_info().await,
            Err(AccountError::Changed)
        ));
        assert!(!extension_state::enabled_map(db.get_connection())
            .await
            .contains_key(PLUGIN));
        assert_eq!(manager.get(&account.id).await.unwrap().id, account.id);
        let unknown = client
            .set_extension_enabled(SetExtensionEnabledRequest {
                id: "not.installed".into(),
                enabled: false,
            })
            .await
            .unwrap_err();
        assert_eq!(unknown.code(), tonic::Code::NotFound);
        assert!(!extension_state::enabled_map(db.get_connection())
            .await
            .contains_key("not.installed"));
    })
    .await
    .expect("catalog RPC lifecycle exceeded its deadline");
}
