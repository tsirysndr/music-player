//! Saved servers, and the two shapes of "add".
//!
//! The interesting case is a backend whose url is not the user's to give:
//! Rocksky's form has no url field, so a client sends an empty one, and
//! demanding one here is what made it unusable.

use async_graphql::{EmptySubscription, Request, Schema, Variables};
use migration::{async_trait::async_trait, Migrator, MigratorTrait};
use music_player_provider::{
    Album, Artist, MusicProvider, Page, ProviderConfig, ProviderError, ProviderFactory,
    ProviderRegistry, ProviderState, Track,
};
use music_player_storage::{saved_servers, Database};
use music_player_tracklist::Tracklist;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

use crate::schema::servers::{ServersMutation, ServersQuery};

type ServersSchema = Schema<ServersQuery, ServersMutation, EmptySubscription>;

// No discovery or scanned music library is needed here. Use real migrations:
// entity-only schema creation omits the expression index targeted by upsert.
async fn setup_schema() -> (ServersSchema, Database, Arc<ProviderState>) {
    let mut options = sea_orm::ConnectOptions::new("sqlite::memory:");
    options.max_connections(1);
    let connection = sea_orm::Database::connect(options).await.unwrap();
    Migrator::up(&connection, None).await.unwrap();
    let db = Database { connection };
    let mut registry = ProviderRegistry::new();
    music_player_provider::register_builtin(&mut registry);
    registry.register(TestFactory);
    let providers = Arc::new(ProviderState::new(Arc::new(registry)));
    let schema = Schema::build(ServersQuery, ServersMutation, EmptySubscription)
        .data(db.clone())
        .data(Arc::clone(&providers))
        .data(Arc::new(Mutex::new(Tracklist::new_empty())))
        .finish();
    (schema, db, providers)
}

struct TestFactory;
struct TestProvider(String);

#[async_trait]
impl ProviderFactory for TestFactory {
    fn kind(&self) -> &'static str {
        "test-account"
    }
    fn display_name(&self) -> &'static str {
        "Synthetic account"
    }
    fn default_port(&self) -> u16 {
        8096
    }
    async fn connect(
        &self,
        config: &ProviderConfig,
    ) -> Result<Arc<dyn MusicProvider>, ProviderError> {
        Ok(Arc::new(TestProvider(config.url.clone())))
    }
}

#[async_trait]
impl MusicProvider for TestProvider {
    fn kind(&self) -> &'static str {
        "test-account"
    }
    fn base_url(&self) -> &str {
        &self.0
    }
    fn host(&self) -> &str {
        "saved.test"
    }
    async fn albums(&self, _: Option<&str>, _: Page) -> Result<Vec<Album>, ProviderError> {
        Ok(vec![])
    }
    async fn artists(&self, _: Option<&str>, _: Page) -> Result<Vec<Artist>, ProviderError> {
        Ok(vec![])
    }
    async fn tracks(&self, _: Option<&str>, _: Page) -> Result<Vec<Track>, ProviderError> {
        Ok(vec![])
    }
    async fn album(&self, _: &str) -> Result<Album, ProviderError> {
        Err(ProviderError::NotFound("album".into()))
    }
    async fn artist(&self, _: &str) -> Result<Artist, ProviderError> {
        Err(ProviderError::NotFound("artist".into()))
    }
    async fn track(&self, _: &str) -> Result<Track, ProviderError> {
        Err(ProviderError::NotFound("track".into()))
    }
}

fn input(fields: Value) -> Value {
    let mut input = json!({
        "kind": "test-account", "name": "Family",
        "url": "http://saved.test:8096", "username": "family"
    });
    input
        .as_object_mut()
        .unwrap()
        .extend(fields.as_object().unwrap().clone());
    input
}

fn save_request(input: Value) -> Request {
    Request::new(
        "mutation Save($input: ServerInput!) {
            addServer(input: $input) { id kind name url username hasPassword connected }
        }",
    )
    .variables(Variables::from_json(json!({ "input": input })))
}

async fn save(schema: &ServersSchema, input: Value) -> Value {
    let response = schema.execute(save_request(input)).await;
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    response.data.into_json().unwrap()["addServer"].clone()
}

/// Every kind the daemon can talk to, with what a form needs to render it.
#[tokio::test]
async fn source_kinds_describe_themselves() {
    let (schema, _, _) = setup_schema().await;
    let resp = schema
        .execute(
            r#"
              query Kinds {
                sourceKinds { kind displayName needsCredentials defaultPort fixedUrl }
              }
            "#,
        )
        .await;
    assert!(resp.errors.is_empty(), "{:?}", resp.errors);

    let data = resp.data.into_json().unwrap();
    let kinds = data["sourceKinds"].as_array().unwrap();
    let names: Vec<&str> = kinds
        .iter()
        .map(|kind| kind["kind"].as_str().unwrap())
        .collect();
    for expected in [
        "subsonic",
        "jellyfin",
        "music-player",
        "kodi",
        "plex",
        "rocksky",
    ] {
        assert!(
            names.contains(&expected),
            "{expected} is missing: {names:?}"
        );
    }

    let rocksky = kinds
        .iter()
        .find(|kind| kind["kind"] == "rocksky")
        .expect("rocksky is offered");
    assert_eq!(rocksky["fixedUrl"], "https://navidrome.rocksky.app");
    // It is a login, not an address, that the user supplies.
    assert_eq!(rocksky["needsCredentials"], true);

    // Everything else takes a url, so the field stays.
    let subsonic = kinds
        .iter()
        .find(|kind| kind["kind"] == "subsonic")
        .unwrap();
    assert!(subsonic["fixedUrl"].is_null());
}

/// The regression: a client with no url field sends an empty url, and the
/// daemon has to supply the one the factory knows about.
#[tokio::test]
async fn a_fixed_url_backend_saves_without_one() {
    let (schema, _, _) = setup_schema().await;
    let resp = schema
        .execute(
            r#"
              mutation Add {
                addServer(input: {
                  kind: "rocksky",
                  name: "Rocksky",
                  url: "",
                  username: "tsiry",
                  password: "hunter2"
                }) { id kind name url hasPassword }
              }
            "#,
        )
        .await;
    assert!(resp.errors.is_empty(), "{:?}", resp.errors);

    let data = resp.data.into_json().unwrap();
    let server = &data["addServer"];
    assert_eq!(server["kind"], "rocksky");
    assert_eq!(server["url"], "https://navidrome.rocksky.app");
    // Stored, never returned.
    assert_eq!(server["hasPassword"], true);
}

/// A url that *is* the user's to give is still required.
#[tokio::test]
async fn a_normal_backend_still_needs_a_url() {
    let (schema, _, _) = setup_schema().await;
    let resp = schema
        .execute(
            r#"
              mutation Add {
                addServer(input: { kind: "subsonic", name: "NAS", url: "" }) { id }
              }
            "#,
        )
        .await;
    assert_eq!(resp.errors.len(), 1);
    assert!(resp.errors[0].message.contains("needs a url"));
}

/// An unknown kind is refused rather than saved as something unusable.
#[tokio::test]
async fn an_unknown_kind_is_refused() {
    let (schema, _, _) = setup_schema().await;
    let resp = schema
        .execute(
            r#"
              mutation Add {
                addServer(input: { kind: "napster", name: "x", url: "http://x" }) { id }
              }
            "#,
        )
        .await;
    assert_eq!(resp.errors.len(), 1);
    assert!(resp.errors[0].message.contains("unknown kind"));
}

/// Nothing connected means the local library, not an error.
#[tokio::test]
async fn nothing_is_connected_by_default() {
    let (schema, _, _) = setup_schema().await;
    let resp = schema.execute(r#"query { connectedServer { id } }"#).await;
    assert!(resp.errors.is_empty(), "{:?}", resp.errors);
    assert!(resp.data.into_json().unwrap()["connectedServer"].is_null());
}

#[tokio::test]
async fn passwords_keep_set_empty_and_clear_through_graphql() {
    let (schema, db, _) = setup_schema().await;
    let first = save(&schema, input(json!({ "password": "synthetic-old" }))).await;
    let id = first["id"].as_str().unwrap();
    for (fields, expected) in [
        (json!({}), Some("synthetic-old")),
        (json!({ "password": null }), Some("synthetic-old")),
        (json!({ "password": "" }), Some("synthetic-old")),
        (json!({ "passwordValue": "" }), Some("")),
        (json!({ "password": "" }), Some("")),
        (
            json!({ "passwordValue": "  synthetic-next  " }),
            Some("  synthetic-next  "),
        ),
        (json!({ "clearPassword": true }), None),
        (
            json!({ "passwordValue": null, "clearPassword": false }),
            None,
        ),
    ] {
        let saved = save(&schema, input(fields)).await;
        assert_eq!(saved["id"], id);
        assert_eq!(saved["hasPassword"], expected.is_some());
        let row = saved_servers::get(db.get_connection(), id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row.password.as_deref(), expected);
    }
}

#[tokio::test]
async fn conflicting_password_fields_do_not_mutate_the_account() {
    let (schema, db, _) = setup_schema().await;
    let first = save(&schema, input(json!({ "password": "synthetic-old" }))).await;
    let id = first["id"].as_str().unwrap();
    let before = saved_servers::get(db.get_connection(), id)
        .await
        .unwrap()
        .unwrap();
    for fields in [
        json!({ "passwordValue": "", "clearPassword": true }),
        json!({ "password": "synthetic-conflict", "passwordValue": "" }),
        json!({ "password": "synthetic-conflict", "clearPassword": true }),
    ] {
        let response = schema.execute(save_request(input(fields))).await;
        assert_eq!(response.errors.len(), 1);
        assert!(response.errors[0]
            .message
            .contains("conflicting password updates"));
        assert_eq!(
            saved_servers::get(db.get_connection(), id)
                .await
                .unwrap()
                .unwrap(),
            before
        );
    }
}

#[tokio::test]
async fn explicit_edits_keep_identity_and_never_merge_accounts() {
    let (schema, db, _) = setup_schema().await;
    let first = save(&schema, input(json!({ "password": "synthetic-a" }))).await;
    let id = first["id"].as_str().unwrap();
    let other = save(&schema, input(json!({ "username": "guest" }))).await;
    assert_ne!(other["id"], id);
    assert_eq!(other["hasPassword"], false);
    let moved = save(
        &schema,
        input(json!({
            "id": id, "url": "http://moved.test:8096", "name": "Moved"
        })),
    )
    .await;
    assert_eq!(moved["id"], id);
    assert_eq!(moved["hasPassword"], true);
    let before = saved_servers::get(db.get_connection(), id)
        .await
        .unwrap()
        .unwrap();
    let rows_before = saved_servers::list(db.get_connection()).await.unwrap();
    for fields in [
        json!({ "id": id, "username": "guest" }),
        json!({ "id": id, "kind": "jellyfin" }),
        json!({ "id": "missing-account" }),
        json!({ "id": "" }),
    ] {
        let response = schema.execute(save_request(input(fields))).await;
        assert_eq!(response.errors.len(), 1);
        assert_eq!(
            saved_servers::list(db.get_connection()).await.unwrap(),
            rows_before
        );
    }
    // Another row now owns the original tuple; moving back must not merge it.
    let replacement = save(&schema, input(json!({}))).await;
    assert_ne!(replacement["id"], id);
    let collision = schema
        .execute(save_request(input(json!({ "id": id }))))
        .await;
    assert_eq!(collision.errors.len(), 1);
    assert_eq!(
        saved_servers::get(db.get_connection(), id)
            .await
            .unwrap()
            .unwrap(),
        before
    );
    let response = schema
        .execute(
            Request::new("query Saved($id: ID!) { savedServer(id: $id) { id url } }")
                .variables(Variables::from_json(json!({ "id": id }))),
        )
        .await;
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    assert_eq!(
        response.data.into_json().unwrap()["savedServer"]["url"],
        "http://moved.test:8096"
    );
}

#[tokio::test]
async fn connected_server_reports_its_config_until_reconnect() {
    let (schema, _, providers) = setup_schema().await;
    let first = save(&schema, input(json!({ "passwordValue": "" }))).await;
    let id = first["id"].as_str().unwrap();
    let connect = || {
        Request::new(
            "mutation Connect($id: ID!) {
            connectToServer(id: $id) { id name url hasPassword connected }
        }",
        )
        .variables(Variables::from_json(json!({ "id": id })))
    };
    let response = schema.execute(connect()).await;
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    assert_eq!(
        response.data.into_json().unwrap()["connectToServer"]["hasPassword"],
        true
    );
    let original = providers.config().await.unwrap();
    assert_eq!(original.id, id);
    assert_eq!(original.password.as_deref(), Some(""));
    let edited = save(
        &schema,
        input(json!({
            "id": id, "name": "Moved", "url": "http://moved.test:8096", "clearPassword": true
        })),
    )
    .await;
    assert_eq!(edited["hasPassword"], false);
    let response = schema
        .execute(
            "query { connectedServer { id name url hasPassword connected }
                 savedServers { id url hasPassword } }",
        )
        .await;
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    let data = response.data.into_json().unwrap();
    assert_eq!(data["connectedServer"]["id"], id);
    assert_eq!(data["connectedServer"]["name"], "Family");
    assert_eq!(data["connectedServer"]["url"], "http://saved.test:8096");
    assert_eq!(data["connectedServer"]["hasPassword"], true);
    assert_eq!(data["connectedServer"]["connected"], true);
    assert_eq!(data["savedServers"][0]["url"], "http://moved.test:8096");
    assert_eq!(data["savedServers"][0]["hasPassword"], false);
    assert_eq!(providers.config().await.unwrap(), original);
    let response = schema.execute(connect()).await;
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    let reconnected = providers.current().await.unwrap();
    assert_eq!(reconnected.config.url, "http://moved.test:8096");
    assert_eq!(reconnected.config.password, None);
    assert_eq!(reconnected.provider.base_url(), "http://moved.test:8096");
    let response = schema
        .execute("mutation { disconnectFromServer { id url hasPassword connected } }")
        .await;
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    let disconnected = response.data.into_json().unwrap();
    assert_eq!(disconnected["disconnectFromServer"]["hasPassword"], false);
    assert_eq!(disconnected["disconnectFromServer"]["connected"], false);
}
