//! Actual protobuf/TCP -> shared manager -> SQLite/WASM; no local execution.
use super::*;
use crate::api::music::v1alpha1::{
    source_accounts_service_client::SourceAccountsServiceClient,
    source_accounts_service_server::SourceAccountsServiceServer,
};
use migration::{Migrator, MigratorTrait};
use music_player_extensions::{abi::source_v2 as wire, host::HostContext};
use music_player_storage::Database;
use serde_json::json;
use std::{future::Future, time::Duration};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::{Channel, Server};

const PLUGIN: &str = "rpc.source.accounts";
const OFFSET: u64 = 9_007_199_254_740_993;
const SECRET: &str = "synthetic-rpc-password";
struct Running(tokio::task::JoinHandle<()>);
impl Drop for Running {
    fn drop(&mut self) {
        self.0.abort();
    }
}

async fn database() -> Database {
    let mut options = sea_orm::ConnectOptions::new("sqlite::memory:");
    options.max_connections(1).sqlx_logging(false);
    let connection = sea_orm::Database::connect(options).await.unwrap();
    Migrator::up(&connection, None).await.unwrap();
    Database { connection }
}
fn output(body: &str) -> String {
    let mut wat = format!("(local.set $p (call $alloc (i64.const {})))", body.len());
    for (index, byte) in body.bytes().enumerate() {
        wat.push_str(&format!(
            "(call $store (i64.add (local.get $p) (i64.const {index})) (i32.const {byte}))"
        ));
    }
    wat.push_str(&format!(
        "(call $output (local.get $p) (i64.const {})) (i32.const 0)",
        body.len()
    ));
    wat
}
fn install(directory: &tempfile::TempDir) {
    let path = directory.path().join(PLUGIN);
    std::fs::create_dir_all(&path).unwrap();
    let mut wat = String::from(
        r#"(module
      (import "extism:host/env" "alloc" (func $alloc (param i64) (result i64)))
      (import "extism:host/env" "store_u8" (func $store (param i64 i32)))
      (import "extism:host/env" "output_set" (func $output (param i64 i64)))
      (import "extism:host/env" "input_length" (func $length (result i64)))
      (import "extism:host/env" "input_load_u8" (func $input (param i64) (result i32)))"#,
    );
    for (name, body) in [
        (
            "source_identity_v2",
            r#"{"serverId":"server","userId":"user"}"#,
        ),
        ("get_media_item_v2", "{}"),
        ("get_audio_options_v2", "{}"),
        ("resolve_audio_v2", "{}"),
        ("release_audio_v2", "{}"),
    ] {
        wat.push_str(&format!(
            "(func (export \"{name}\") (result i32) (local $p i64) {})",
            output(body)
        ));
    }
    let expected = serde_json::to_string(&wire::BrowseRequest {
        identity: wire::SourceIdentity {
            server_id: "server".into(),
            user_id: "user".into(),
        },
        parent_id: None,
        query: Some("needle".into()),
        offset: OFFSET,
        limit: 1,
    })
    .unwrap();
    wat.push_str(&format!("(func (export \"browse_media_v2\") (result i32) (local $p i64) (if (i64.ne (call $length) (i64.const {})) (then unreachable))", expected.len()));
    for (index, byte) in expected.bytes().enumerate() {
        wat.push_str(&format!(
            "(if (i32.ne (call $input (i64.const {index})) (i32.const {byte})) (then unreachable))"
        ));
    }
    wat.push_str(&output(
        &json!({"items":[{
        "itemId":"55508", "title":"RPC guest episode", "itemType":"Episode", "mediaType":"Video",
        "isContainer":false, "playable":true, "artist":"Guest series",
        "seasonNumber":2025050473_u64, "episodeNumber":41,
        "runtimeTicks":191429666670_u64
    }],"nextOffset":OFFSET + 1})
        .to_string(),
    ));
    wat.push_str("))");
    std::fs::write(path.join("plugin.wat"), wat).unwrap();
    std::fs::write(
        path.join("plugin.json"),
        json!({
            "id":PLUGIN,"name":"RPC source","entry":"plugin.wat","sourceApiVersion":2,
            "capabilities":["source"],"permissions":{"config":{"password":SECRET}}
        })
        .to_string(),
    )
    .unwrap();
}

type Client = SourceAccountsServiceClient<Channel>;
async fn start(sources: SourceResolver) -> (Client, Running) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        Server::builder()
            .add_service(SourceAccountsServiceServer::new(SourceAccounts::new(
                sources,
            )))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .unwrap();
    });
    let running = Running(task);
    let client = Client::connect(format!("http://{address}")).await.unwrap();
    (client, running)
}
// Only a known admission rejection is retried. The test's outer timeout bounds
// global loader contention; uncertain mutation replies are never retried.
async fn admitted<T, F, Fut>(mut call: F) -> Result<Response<T>, Status>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<Response<T>, Status>>,
{
    loop {
        let result = call().await;
        if result.as_ref().is_err_and(|e| {
            e.code() == tonic::Code::ResourceExhausted
                && e.message() == "extension account capacity unavailable"
        }) {
            tokio::time::sleep(Duration::from_millis(10)).await;
            continue;
        }
        return result;
    }
}

#[tokio::test]
async fn rpc_account_lifecycle_and_exact_offsets_use_shared_wasm_manager() {
    tokio::time::timeout(Duration::from_secs(45), async {
        let directory = tempfile::tempdir().unwrap();
        install(&directory);
        let db = database().await;
        let manager = ExtensionAccounts::new(
            db.clone(),
            vec![directory.path().to_path_buf()],
            HostContext::default(),
        );
        let resolver = SourceResolver::new(db.clone(), "rpc-test".into(), true)
            .with_extensions(manager.clone(), vec!["mp3".into()]);
        let (mut client, _running) = start(resolver).await;
        let definitions = admitted(|| {
            let mut client = client.clone();
            async move {
                client
                    .list_extension_source_definitions(ListExtensionSourceDefinitionsRequest {})
                    .await
            }
        })
        .await
        .unwrap()
        .into_inner()
        .definitions;
        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].config_keys, ["password"]);
        assert_eq!(definitions[0].source_api_version, 2);
        assert!(manager.status().is_empty());

        let saved = admitted(|| {
            let mut client = client.clone();
            async move {
                client
                    .create_source_account(CreateSourceAccountRequest {
                        extension_id: PLUGIN.into(),
                        name: "RPC account".into(),
                        config: vec![SourceConfigValue {
                            key: "password".into(),
                            value: String::new(),
                        }],
                    })
                    .await
            }
        })
        .await
        .unwrap()
        .into_inner()
        .account
        .unwrap();
        assert!(!saved.identity_bound);
        assert_eq!(saved.config_revision, 1);
        let stored = manager.get(&saved.id).await.unwrap();
        assert_eq!(accounts::config(&stored).unwrap()["password"], "");
        assert!(!format!("{saved:?}").contains(SECRET));
        assert!(manager.status().is_empty());

        let connected = admitted(|| {
            let mut client = client.clone();
            let id = saved.id.clone();
            async move {
                client
                    .connect_source_account(ConnectSourceAccountRequest { id })
                    .await
            }
        })
        .await
        .unwrap()
        .into_inner()
        .account
        .unwrap();
        assert!(connected.identity_bound);
        let page = client
            .browse_source_media(BrowseSourceMediaRequest {
                account_id: saved.id.clone(),
                parent: None,
                list: 0,
                query: Some("needle".into()),
                offset: OFFSET,
                limit: Some(1),
            })
            .await
            .unwrap()
            .into_inner();
        assert_eq!(page.next_offset, Some(OFFSET + 1));
        assert_eq!(page.entries.len(), 1);
        assert_eq!(page.entries[0].season_number, Some(2025050473));
        assert_eq!(page.entries[0].episode_number, Some(41));
        let handle = SourceHandle::parse(&page.entries[0].id).unwrap();
        assert_eq!(handle.account_id(), saved.id);
        assert!(matches!(&handle, SourceHandle::Remote(remote) if remote.item_id == "55508"));

        // A mutation through the already shared manager is immediately visible
        // to RPC clients; a disabled source remains inspectable and editable.
        manager
            .set_enabled(PLUGIN, false, "2026-10-02")
            .await
            .unwrap();
        let inspected = client
            .get_source_account(GetSourceAccountRequest {
                id: saved.id.clone(),
            })
            .await
            .unwrap()
            .into_inner()
            .account
            .unwrap();
        assert_eq!(inspected.id, saved.id);
        let error = client
            .browse_source_media(BrowseSourceMediaRequest {
                account_id: saved.id.clone(),
                offset: OFFSET,
                limit: Some(1),
                ..Default::default()
            })
            .await
            .unwrap_err();
        assert_eq!(error.message(), "source extension disabled");

        let edited = admitted(|| {
            let mut client = client.clone();
            let id = saved.id.clone();
            async move {
                client
                    .update_source_account(UpdateSourceAccountRequest {
                        id,
                        expected_revision: 1,
                        name: Some("Edited through RPC".into()),
                        changes: vec![SourceConfigChange {
                            key: "password".into(),
                            change: Some(Change::Remove(true)),
                        }],
                    })
                    .await
            }
        })
        .await
        .unwrap()
        .into_inner()
        .account
        .unwrap();
        assert_eq!(edited.config_revision, 2);
        assert!(edited.overridden_config_keys.is_empty());
        assert_eq!(
            manager.get(&saved.id).await.unwrap().name,
            "Edited through RPC"
        );
        let error = client
            .update_source_account(UpdateSourceAccountRequest {
                id: saved.id.clone(),
                expected_revision: 1,
                name: Some("Stale".into()),
                ..Default::default()
            })
            .await
            .unwrap_err();
        assert_eq!(error.code(), tonic::Code::Aborted);
        let error = client
            .update_source_account(UpdateSourceAccountRequest {
                id: saved.id.clone(),
                expected_revision: 2,
                changes: vec![SourceConfigChange {
                    key: "password".into(),
                    change: Some(Change::Remove(false)),
                }],
                ..Default::default()
            })
            .await
            .unwrap_err();
        assert_eq!(error.code(), tonic::Code::InvalidArgument);
        assert_eq!(manager.get(&saved.id).await.unwrap().config_revision, 2);
        assert!(
            client
                .delete_source_account(DeleteSourceAccountRequest {
                    id: saved.id.clone()
                })
                .await
                .unwrap()
                .into_inner()
                .deleted
        );
        assert!(accounts::list(db.get_connection())
            .await
            .unwrap()
            .is_empty());
        assert!(client
            .list_source_accounts(ListSourceAccountsRequest {})
            .await
            .unwrap()
            .into_inner()
            .accounts
            .is_empty());
    })
    .await
    .expect("source account RPC watchdog");
}

#[tokio::test]
async fn rpc_without_shared_extension_manager_fails_explicitly() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let sources = SourceResolver::new(database().await, "native-only".into(), true);
        let (mut client, _running) = start(sources).await;
        let error = client
            .list_source_accounts(ListSourceAccountsRequest {})
            .await
            .unwrap_err();
        assert_eq!(error.code(), tonic::Code::FailedPrecondition);
        assert_eq!(
            error.message(),
            "extension source runtime is not configured"
        );
    })
    .await
    .expect("missing manager RPC watchdog");
}
