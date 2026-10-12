//! Real GraphQL -> shared account manager -> migrated SQLite/WAT. No player is
//! replaced by a fake resolver; commands are observed before engine execution.
use super::*;
use async_graphql::{EmptySubscription, MergedObject, Request, Schema, Variables};
use migration::{Migrator, MigratorTrait};
use music_player_extensions::{abi, host::HostContext};
use music_player_playback::{player::PlayerCommand, source_resolver::SourceResolver};
use music_player_renderer::CurrentReceiverDevice;
use music_player_storage::Database;
use music_player_types::source::ResourceKind;
use serde_json::{json, Value};
use std::{
    future::Future,
    path::PathBuf,
    sync::{Arc, Mutex as StdMutex},
    time::Duration,
};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver},
    Mutex,
};

const V1: &str = "graphql.legacy.accounts";
const V2: &str = "graphql.modern.accounts";
const SECRET: &str = "synthetic-config-secret";
const LARGE_OFFSET: &str = "9007199254740993";
static SERIAL: Mutex<()> = Mutex::const_new(());

struct Directory(PathBuf);
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
async fn bounded(future: impl Future<Output = ()>) {
    let _serial = SERIAL.lock().await;
    tokio::time::timeout(Duration::from_secs(45), future)
        .await
        .expect("source account API watchdog");
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
        "(call $output (local.get $p) (i64.const {})) (return (i32.const 0))",
        body.len()
    ));
    wat
}
fn export(name: &str, body: &str) -> String {
    format!(
        "(func (export \"{name}\") (result i32) (local $p i64) {})",
        output(body)
    )
}
fn matcher(name: &str, input: &str) -> String {
    let mut wat = format!("(func ${name} (result i32) (if (i64.ne (call $length) (i64.const {})) (then (return (i32.const 0))))", input.len());
    for (index, byte) in input.bytes().enumerate() {
        wat.push_str(&format!("(if (i32.ne (call $input (i64.const {index})) (i32.const {byte})) (then (return (i32.const 0))))"));
    }
    wat.push_str("(i32.const 1))");
    wat
}
fn paged(name: &str, requests: &[(String, String)]) -> String {
    let mut wat = String::new();
    let mut function = format!("(func (export \"{name}\") (result i32) (local $p i64)");
    for (index, (input, body)) in requests.iter().enumerate() {
        let check = format!("{name}_{index}");
        wat.push_str(&matcher(&check, input));
        function.push_str(&format!("(if (call ${check}) (then {}))", output(body)));
    }
    function.push_str("unreachable)");
    wat + &function
}
fn install(root: &Directory, id: &str, version: i32) {
    let directory = root.0.join(id);
    std::fs::create_dir_all(&directory).unwrap();
    let mut wat = String::from(
        r#"(module
      (import "extism:host/env" "alloc" (func $alloc (param i64) (result i64)))
      (import "extism:host/env" "store_u8" (func $store (param i64 i32)))
      (import "extism:host/env" "output_set" (func $output (param i64 i64)))
      (import "extism:host/env" "input_length" (func $length (result i64)))
      (import "extism:host/env" "input_load_u8" (func $input (param i64) (result i32)))"#,
    );
    if version == 1 {
        wat.push_str(&export(
            "source_info",
            r#"{"name":"Legacy fixture","description":"Actual WASM"}"#,
        ));
        let tracks = r#"[{"id":"episode","title":"Actual guest title","artist":"Actual guest artist","album":"Actual album","albumArtist":"Actual album artist","year":2024,"duration":123.5,"trackNumber":41,"uri":"https://temporary.invalid/audio?token=do-not-persist"}]"#;
        let requests = [0, 1]
            .into_iter()
            .map(|offset| {
                (
                    serde_json::to_string(&abi::BrowseRequest {
                        parent_id: "".into(),
                        query: "needle".into(),
                        offset,
                        limit: 1,
                    })
                    .unwrap(),
                    if offset == 0 {
                        tracks.into()
                    } else {
                        "[]".into()
                    },
                )
            })
            .collect::<Vec<_>>();
        wat.push_str(&paged("list_tracks", &requests));
        for (name, body) in [
            ("list_albums", r#"[{"id":"album","title":"Guest album"}]"#),
            ("list_artists", r#"[{"id":"artist","name":"Guest artist"}]"#),
            (
                "list_playlists",
                r#"[{"id":"playlist","name":"Guest playlist"}]"#,
            ),
            ("get_stream_url", "{}"),
        ] {
            wat.push_str(&export(name, body));
        }
    } else {
        wat.push_str(&export(
            "source_identity_v2",
            r#"{"serverId":"server","userId":"user"}"#,
        ));
        let requests = [9007199254740993_u64, 9007199254740994]
            .into_iter()
            .map(|offset| {
                (
                    serde_json::to_string(&abi::source_v2::BrowseRequest {
                        identity: abi::source_v2::SourceIdentity {
                            server_id: "server".into(),
                            user_id: "user".into(),
                        },
                        parent_id: None,
                        query: Some("needle".into()),
                        offset,
                        limit: 1,
                    })
                    .unwrap(),
                    if offset == 9007199254740993 {
                        json!({"items":[{
                "itemId":"55508","title":"Episode from v2","isContainer":false,"playable":true,
                "itemType":"Episode","mediaType":"Video","seasonNumber":2025050473_u64,
                "episodeNumber":41,"artist":"Guest series","runtimeTicks":191429666670_u64
            }],"nextOffset":9007199254740994_u64})
                        .to_string()
                    } else {
                        r#"{"items":[],"nextOffset":null}"#.into()
                    },
                )
            })
            .collect::<Vec<_>>();
        wat.push_str(&paged("browse_media_v2", &requests));
        for name in [
            "get_media_item_v2",
            "get_audio_options_v2",
            "resolve_audio_v2",
            "release_audio_v2",
        ] {
            wat.push_str(&export(name, "{}"));
        }
    }
    wat.push(')');
    std::fs::write(directory.join("plugin.wat"), wat).unwrap();
    std::fs::write(directory.join("plugin.json"), json!({
        "id":id,"name":id,"entry":"plugin.wat","sourceApiVersion":version,
        "capabilities":["source"],"permissions":{"config":{"server":"default-server","password":SECRET}}
    }).to_string()).unwrap();
}

#[derive(MergedObject, Default)]
struct Mutations(
    SourceAccountsMutation,
    crate::schema::tracklist::TracklistMutation,
    crate::schema::media::MediaMutation,
);
type Api = Schema<SourceAccountsQuery, Mutations, EmptySubscription>;
struct Fixture {
    api: Api,
    manager: ExtensionAccounts,
    unrelated_db: Database,
    commands: UnboundedReceiver<PlayerCommand>,
    _directory: Directory,
}
impl Fixture {
    async fn new() -> Self {
        let directory =
            Directory(std::env::temp_dir().join(format!("mp-source-accounts-{}", cuid::cuid2())));
        install(&directory, V1, 1);
        install(&directory, V2, 2);
        let db = database().await;
        let unrelated_db = database().await;
        let manager = ExtensionAccounts::new(
            db.clone(),
            vec![directory.0.clone()],
            HostContext::default(),
        );
        let resolver = SourceResolver::new(db, "fixture".into(), true)
            .with_extensions(manager.clone(), vec!["mp3".into()]);
        let (tx, commands) = unbounded_channel();
        let api = Schema::build(SourceAccountsQuery, Mutations::default(), EmptySubscription)
            .data(unrelated_db.clone())
            .data(resolver)
            .data(Arc::new(StdMutex::new(tx)))
            .data(Arc::new(Mutex::new(CurrentReceiverDevice::new())))
            .finish();
        Self {
            api,
            manager,
            unrelated_db,
            commands,
            _directory: directory,
        }
    }
}
fn request(query: &str, variables: Value) -> Request {
    Request::new(query).variables(Variables::from_json(variables))
}
async fn success(api: &Api, query: &str, variables: Value) -> Value {
    loop {
        let response = api.execute(request(query, variables.clone())).await;
        // Only the explicit pre-admission capacity rejection can be retried;
        // no mutation was started. Lost replies and guest failures are not retried.
        if response.errors.len() == 1
            && response.errors[0].message == "extension account capacity unavailable"
        {
            tokio::time::sleep(Duration::from_millis(10)).await;
            continue;
        }
        assert!(response.errors.is_empty(), "{:?}", response.errors);
        let data = response.data.into_json().unwrap();
        assert!(!data.to_string().contains(SECRET));
        return data;
    }
}
async fn failure(api: &Api, query: &str, variables: Value) -> String {
    loop {
        let response = api.execute(request(query, variables.clone())).await;
        assert_eq!(response.errors.len(), 1, "{:?}", response.errors);
        if response.errors[0].message == "extension account capacity unavailable" {
            tokio::time::sleep(Duration::from_millis(10)).await;
            continue;
        }
        assert!(!response.errors[0].message.contains(SECRET));
        return response.errors[0].message.clone();
    }
}
const CREATE: &str = "mutation($plugin: ID!, $name: String!, $config: [SourceConfigValueInput!]) { createSourceAccount(extensionId:$plugin, name:$name, config:$config) { id configRevision identityBound sourceApiVersion overriddenConfigKeys } }";
const CONNECT: &str =
    "mutation($id: ID!) { connectSourceAccount(id:$id) { id configRevision identityBound } }";
const EDIT: &str = "mutation($id: ID!, $revision: String!, $name: String, $changes: [SourceConfigChangeInput!]) { updateSourceAccount(id:$id, expectedRevision:$revision, name:$name, changes:$changes) { id name configRevision identityBound overriddenConfigKeys } }";
const BROWSE: &str = "query($id:ID!, $list:SourceBrowseList, $parent:ID, $offset:String, $limit:Int) { browseSourceMedia(accountId:$id,list:$list,parent:$parent,query:\"needle\",offset:$offset,limit:$limit) { accountId nextOffset entries { id title itemType mediaType isContainer seasonNumber episodeNumber track { id uri title artist duration trackNumber discNumber album { id title artist year } } } } }";

#[tokio::test]
async fn legacy_account_api_connects_browses_enqueues_and_invalidates_edits() {
    bounded(async {
        let mut f = Fixture::new().await;
        let definitions = success(
            &f.api,
            "{ extensionSourceDefinitions { extensionId enabled sourceApiVersion configKeys } }",
            json!({}),
        )
        .await;
        assert_eq!(
            definitions["extensionSourceDefinitions"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            definitions["extensionSourceDefinitions"][0]["configKeys"],
            json!(["password", "server"])
        );
        let data = success(
            &f.api,
            CREATE,
            json!({"plugin":V1,"name":"Legacy","config":[{"key":"password","value":""}]}),
        )
        .await;
        let id = data["createSourceAccount"]["id"]
            .as_str()
            .unwrap()
            .to_owned();
        assert_eq!(data["createSourceAccount"]["identityBound"], false);
        assert_eq!(data["createSourceAccount"]["sourceApiVersion"], 1);
        assert!(f.manager.status().is_empty());
        assert!(accounts::list(f.unrelated_db.get_connection())
            .await
            .unwrap()
            .is_empty());
        let initial = f.manager.get(&id).await.unwrap();
        assert_eq!(accounts::config(&initial).unwrap()["password"], "");
        assert_eq!(
            failure(&f.api, BROWSE, json!({"id":id,"list":"TRACKS","limit":1})).await,
            "extension account identity missing or different"
        );
        assert!(f.manager.status().is_empty());
        let connected = success(&f.api, CONNECT, json!({"id":id})).await;
        assert_eq!(connected["connectSourceAccount"]["identityBound"], true);
        assert!(f.commands.try_recv().is_err());

        let page = success(&f.api, BROWSE, json!({"id":id,"list":"TRACKS","limit":1})).await;
        assert_eq!(page["browseSourceMedia"]["nextOffset"], "1");
        let entry = &page["browseSourceMedia"]["entries"][0];
        let handle = SourceHandle::parse(entry["id"].as_str().unwrap()).unwrap();
        assert!(matches!(&handle, SourceHandle::Local(local) if local.config_revision == 1));
        assert_eq!(handle.account_id(), id);
        assert!(!page.to_string().contains("do-not-persist"));
        assert_eq!(entry["track"]["uri"], entry["id"]);
        let end = success(
            &f.api,
            BROWSE,
            json!({"id":id,"list":"TRACKS","limit":1,"offset":"1"}),
        )
        .await;
        assert_eq!(end["browseSourceMedia"]["entries"], json!([]));
        assert!(end["browseSourceMedia"]["nextOffset"].is_null());
        for (list, kind) in [
            ("ALBUMS", ResourceKind::Album),
            ("ARTISTS", ResourceKind::Artist),
            ("PLAYLISTS", ResourceKind::Playlist),
        ] {
            let containers = success(&f.api, BROWSE, json!({"id":id,"list":list})).await;
            let container = &containers["browseSourceMedia"]["entries"][0];
            assert_eq!(container["isContainer"], true);
            assert!(container["track"].is_null());
            assert_eq!(
                SourceHandle::parse(container["id"].as_str().unwrap())
                    .unwrap()
                    .kind(),
                kind
            );
        }
        let mut input = entry["track"].clone();
        let album = input.as_object_mut().unwrap().remove("album").unwrap();
        assert_eq!(album["id"], "");
        input["albumTitle"] = album["title"].clone();
        input["albumArtist"] = album["artist"].clone();
        input["albumYear"] = album["year"].clone();
        success(
            &f.api,
            "mutation($tracks:[TrackInput!]!) { addTracks(tracks:$tracks) }",
            json!({"tracks":[input]}),
        )
        .await;
        let PlayerCommand::LoadTracklist {
            tracks,
            start_index,
        } = f.commands.try_recv().unwrap()
        else {
            panic!("unexpected command")
        };
        assert!(start_index.is_none());
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].title, "Actual guest title");
        assert_eq!(tracks[0].artist, "Actual guest artist");
        assert_eq!(tracks[0].album.title, "Actual album");
        assert_eq!(tracks[0].album.artist, "Actual album artist");
        assert_eq!(tracks[0].album.year, Some(2024));
        assert!(tracks[0].album_id.is_none() && tracks[0].album.id.is_empty());
        assert_eq!(tracks[0].track, Some(41));
        assert_eq!(tracks[0].id, handle.to_handle());

        // The v1 guest has no item lookup. Playing its full browse entry must
        // preserve tags and dispatch exactly one command with the chosen mode.
        let play = "mutation($entry:SelectedMediaInput!, $next:Boolean!){playSelectedMedia(entry:$entry,next:$next)}";
        for next in [true, false] {
            let (response, ()) = tokio::join!(
                f.api.execute(request(play, json!({"entry":{"track":input},"next":next}))),
                async {
                    let PlayerCommand::PlaySelectedMedia { track, selection, next: actual, reply } = f.commands.recv().await.unwrap() else {
                        panic!("expected one full-metadata play command");
                    };
                    assert_eq!(actual, next);
                    assert_eq!(selection, music_player_types::audio::AudioSelection::Auto);
                    assert_eq!(track, tracks[0]);
                    reply.send(Ok("accepted-occurrence".into())).unwrap();
                }
            );
            assert!(response.errors.is_empty(), "{:?}", response.errors);
            assert_eq!(response.data.into_json().unwrap()["playSelectedMedia"], "accepted-occurrence");
            assert!(f.commands.try_recv().is_err());
        }
        assert_eq!(failure(&f.api, play, json!({"entry":{"track":input,
            "choice":{"mediaSourceId":"invented","audioStreamIndex":0}},"next":false})).await,
            "source v1 does not support audio selection");
        assert!(f.commands.try_recv().is_err());

        let renamed = success(
            &f.api,
            EDIT,
            json!({"id":id,"revision":"1","name":"Renamed"}),
        )
        .await;
        assert_eq!(renamed["updateSourceAccount"]["configRevision"], "1");
        assert_eq!(renamed["updateSourceAccount"]["identityBound"], true);
        f.manager.validate_handle(&handle).await.unwrap();
        let edited = success(
            &f.api,
            EDIT,
            json!({"id":id,"revision":"1","changes":[{"key":"password","operation":"REMOVE"}]}),
        )
        .await;
        assert_eq!(edited["updateSourceAccount"]["configRevision"], "2");
        assert_eq!(edited["updateSourceAccount"]["identityBound"], false);
        assert!(f.manager.validate_handle(&handle).await.is_err());
        let stale_play = failure(&f.api, play, json!({"entry":{"track":input},"next":false})).await;
        assert_eq!(stale_play, "extension account identity missing or different");
        assert!(f.commands.try_recv().is_err());
        assert!(accounts::config(&f.manager.get(&id).await.unwrap())
            .unwrap()
            .is_empty());
        assert_eq!(
            failure(&f.api, EDIT, json!({"id":id,"revision":"1","name":"Stale"})).await,
            "Source account changed; refresh before editing"
        );
        success(&f.api, CONNECT, json!({"id":id})).await;
        let fresh = success(&f.api, BROWSE, json!({"id":id,"list":"TRACKS","limit":1})).await;
        assert_ne!(fresh["browseSourceMedia"]["entries"][0]["id"], entry["id"]);
        f.manager
            .set_enabled(V1, false, "2026-10-02")
            .await
            .unwrap();
        let inspected = success(
            &f.api,
            "query($id:ID!){ sourceAccount(id:$id){name configRevision} }",
            json!({"id":id}),
        )
        .await;
        assert_eq!(inspected["sourceAccount"]["name"], "Renamed");
        success(
            &f.api,
            EDIT,
            json!({"id":id,"revision":"2","name":"Disabled account"}),
        )
        .await;
        assert_eq!(
            failure(&f.api, BROWSE, json!({"id":id,"list":"TRACKS","limit":1})).await,
            "source extension disabled"
        );
        success(
            &f.api,
            "mutation($id:ID!){deleteSourceAccount(id:$id)}",
            json!({"id":id}),
        )
        .await;
        assert!(f.manager.list().await.unwrap().is_empty());
        assert!(f.commands.try_recv().is_err());
    })
    .await;
}

#[tokio::test]
async fn modern_account_browse_preserves_large_offsets_and_rejects_mixed_protocols() {
    bounded(async {
        let f = Fixture::new().await;
        let data = success(&f.api, CREATE, json!({"plugin":V2,"name":"Modern"})).await;
        let id = data["createSourceAccount"]["id"].as_str().unwrap();
        success(&f.api, CONNECT, json!({"id":id})).await;
        let page = success(&f.api, BROWSE, json!({"id":id,"limit":1,"offset":LARGE_OFFSET})).await;
        assert_eq!(page["browseSourceMedia"]["nextOffset"], "9007199254740994");
        let entry = &page["browseSourceMedia"]["entries"][0];
        assert_eq!(entry["itemType"], "Episode");
        assert_eq!(entry["seasonNumber"], "2025050473");
        assert_eq!(entry["episodeNumber"], "41");
        let SourceHandle::Remote(remote) = SourceHandle::parse(entry["id"].as_str().unwrap()).unwrap() else { panic!("v2 was downgraded") };
        assert_eq!(remote.remote.server_id, "server");
        assert_eq!(remote.remote.user_id, "user");
        let end = success(&f.api, BROWSE, json!({"id":id,"limit":1,"offset":"9007199254740994"})).await;
        assert!(end["browseSourceMedia"]["nextOffset"].is_null());
        assert_eq!(end["browseSourceMedia"]["entries"], json!([]));
        assert_eq!(failure(&f.api, BROWSE, json!({"id":id,"list":"TRACKS"})).await, "This source uses root and container navigation");
        for offset in ["-1", "01", "+1", "18446744073709551616"] {
            assert_eq!(failure(&f.api, BROWSE, json!({"id":id,"offset":offset})).await, "Invalid source page offset");
        }
        let before = f.manager.get(id).await.unwrap();
        for (changes, error) in [
            (json!([{"key":"password","operation":"SET"}]), "SET requires a value; REMOVE must omit the value"),
            (json!([{"key":"password","operation":"REMOVE","value":""}]), "SET requires a value; REMOVE must omit the value"),
            (json!([{"key":"password","operation":"SET","value":""},{"key":"password","operation":"REMOVE"}]), "A configuration key was supplied more than once"),
            (json!([{"key":"not-declared","operation":"SET","value":SECRET}]), "invalid extension source or configuration"),
        ] {
            assert_eq!(failure(&f.api, EDIT, json!({"id":id,"revision":"1","changes":changes})).await, error);
            assert_eq!(f.manager.get(id).await.unwrap(), before);
        }
        let legacy = success(&f.api, CREATE, json!({"plugin":V1,"name":"Legacy"})).await;
        let legacy_id = legacy["createSourceAccount"]["id"].as_str().unwrap();
        assert_eq!(failure(&f.api, BROWSE, json!({"id":legacy_id})).await, "Choose a list for this legacy source");
        assert_eq!(failure(&f.api, BROWSE, json!({"id":legacy_id,"list":"TRACKS","parent":remote.to_handle()})).await, "Parent does not belong to this source protocol");
        let listed = success(&f.api, "{sourceAccounts{id sourceApiVersion configRevision overriddenConfigKeys}}", json!({})).await;
        assert_eq!(listed["sourceAccounts"].as_array().unwrap().len(), 2);
        assert!(accounts::list(f.unrelated_db.get_connection()).await.unwrap().is_empty());
    }).await;
}

#[tokio::test]
async fn account_api_requires_shared_resolver_without_context_database_fallback() {
    let schema = Schema::build(SourceAccountsQuery, Mutations::default(), EmptySubscription)
        .data(database().await)
        .finish();
    assert_eq!(
        failure(&schema, "{sourceAccounts{id}}", json!({})).await,
        "Shared source resolver is not configured"
    );
}
