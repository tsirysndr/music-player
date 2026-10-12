//! Saved-account wire compatibility and real handler/storage mapping.
//! All credentials are synthetic; these tests open no network connections.

use migration::{Migrator, MigratorTrait};
use music_player_provider::{
    Album, Artist, MusicProvider, Page, ProviderConfig, ProviderError, ProviderFactory,
    ProviderRegistry, ProviderState, Track,
};
use music_player_server::{
    api::music::v1alpha1::{
        servers_service_server::ServersService, AddServerRequest, ConnectServerRequest,
        DisconnectServerRequest, GetConnectedServerRequest, ListServersRequest, Server,
    },
    servers::Servers,
};
use music_player_storage::{saved_servers, Database};
use music_player_tracklist::Tracklist;
use prost::Message;
use std::sync::{Arc, Mutex};
use tonic::{async_trait, Request, Status};

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

async fn setup() -> (Servers, Database, Arc<ProviderState>) {
    let mut options = sea_orm::ConnectOptions::new("sqlite::memory:");
    options.max_connections(1);
    let connection = sea_orm::Database::connect(options).await.unwrap();
    // Exercise the registered expression-index migration, not an entity-only table.
    Migrator::up(&connection, None).await.unwrap();
    let db = Database { connection };
    let mut registry = ProviderRegistry::new();
    registry.register(TestFactory);
    let providers = Arc::new(ProviderState::new(Arc::new(registry)));
    let service = Servers::new(
        db.clone(),
        Arc::clone(&providers),
        Arc::new(Mutex::new(Tracklist::new_empty())),
    );
    (service, db, providers)
}

fn input() -> AddServerRequest {
    AddServerRequest {
        kind: "test-account".into(),
        name: "Family".into(),
        url: "http://saved.test:8096".into(),
        username: "family".into(),
        ..Default::default()
    }
}

async fn save(service: &Servers, input: AddServerRequest) -> Result<Server, Status> {
    // The handler receives exactly what the generated protobuf decoder delivers.
    let wire = input.encode_to_vec();
    let decoded = AddServerRequest::decode(wire.as_slice()).unwrap();
    Ok(service
        .add_server(Request::new(decoded))
        .await?
        .into_inner()
        .server
        .unwrap())
}

#[test]
fn add_request_wire_preserves_old_fields_and_new_presence() {
    // Fields 4/5 from a legacy caller: username and password, no new fields.
    let old = AddServerRequest::decode(&b"\x22\x06family\x2a\x09synthetic"[..]).unwrap();
    assert_eq!(old.username, "family");
    assert_eq!(old.password, "synthetic");
    assert_eq!(old.id, None);
    assert_eq!(old.password_value, None);
    assert!(!old.clear_password);
    // Tags 6/7 are explicitly present empty strings, while tag 8 is a bool.
    let bytes = [0x32, 0x00, 0x3a, 0x00, 0x40, 0x01];
    let explicit = AddServerRequest::decode(bytes.as_slice()).unwrap();
    assert_eq!(explicit.id.as_deref(), Some(""));
    assert_eq!(explicit.password_value.as_deref(), Some(""));
    assert!(explicit.clear_password);
    assert_eq!(explicit.encode_to_vec(), bytes);
    let absent = AddServerRequest::decode(&[][..]).unwrap();
    assert_eq!(absent.password_value, None);
    assert!(!absent.clear_password);
}

#[tokio::test]
async fn password_presence_and_legacy_keep_reach_storage() {
    let (service, db, _) = setup().await;
    let first = save(
        &service,
        AddServerRequest {
            password: "synthetic-old".into(),
            ..input()
        },
    )
    .await
    .unwrap();
    for (legacy, value, clear, expected) in [
        ("", None, false, Some("synthetic-old")),
        ("", Some(""), false, Some("")),
        ("", None, false, Some("")),
        (
            "",
            Some("  synthetic-next  "),
            false,
            Some("  synthetic-next  "),
        ),
        ("", None, true, None),
        ("", None, false, None),
        ("synthetic-legacy", None, false, Some("synthetic-legacy")),
    ] {
        let saved = save(
            &service,
            AddServerRequest {
                password: legacy.into(),
                password_value: value.map(str::to_owned),
                clear_password: clear,
                ..input()
            },
        )
        .await
        .unwrap();
        assert_eq!(saved.id, first.id);
        assert_eq!(saved.has_password, expected.is_some());
        let row = saved_servers::get(db.get_connection(), &first.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row.password.as_deref(), expected);
    }
}

#[tokio::test]
async fn password_conflicts_are_invalid_arguments_without_writes() {
    let (service, db, _) = setup().await;
    let first = save(
        &service,
        AddServerRequest {
            password: "synthetic-old".into(),
            ..input()
        },
    )
    .await
    .unwrap();
    let before = saved_servers::get(db.get_connection(), &first.id)
        .await
        .unwrap()
        .unwrap();
    for (legacy, value, clear) in [
        ("", Some(""), true),
        ("synthetic-conflict", Some(""), false),
        ("synthetic-conflict", None, true),
    ] {
        let error = save(
            &service,
            AddServerRequest {
                password: legacy.into(),
                password_value: value.map(str::to_owned),
                clear_password: clear,
                ..input()
            },
        )
        .await
        .unwrap_err();
        assert_eq!(error.code(), tonic::Code::InvalidArgument);
        assert_eq!(error.message(), "conflicting password updates");
        assert_eq!(
            saved_servers::get(db.get_connection(), &first.id)
                .await
                .unwrap()
                .unwrap(),
            before
        );
    }
}

#[tokio::test]
async fn editing_an_id_preserves_identity_and_rejects_account_rebinding() {
    let (service, db, _) = setup().await;
    let first = save(
        &service,
        AddServerRequest {
            password: "synthetic-a".into(),
            ..input()
        },
    )
    .await
    .unwrap();
    let other = save(
        &service,
        AddServerRequest {
            username: "guest".into(),
            ..input()
        },
    )
    .await
    .unwrap();
    assert_ne!(first.id, other.id);
    assert!(!other.has_password);
    let moved = save(
        &service,
        AddServerRequest {
            id: Some(first.id.clone()),
            url: "http://moved.test:8096".into(),
            name: "Moved".into(),
            ..input()
        },
    )
    .await
    .unwrap();
    assert_eq!(moved.id, first.id);
    assert!(moved.has_password);
    let before = saved_servers::list(db.get_connection()).await.unwrap();
    for request in [
        AddServerRequest {
            id: Some(first.id.clone()),
            username: "guest".into(),
            ..input()
        },
        AddServerRequest {
            id: Some("missing-account".into()),
            ..input()
        },
        AddServerRequest {
            id: Some(String::new()),
            ..input()
        },
    ] {
        assert!(save(&service, request).await.is_err());
        assert_eq!(
            saved_servers::list(db.get_connection()).await.unwrap(),
            before
        );
    }
    let replacement = save(&service, input()).await.unwrap();
    assert_ne!(replacement.id, first.id);
    let before = saved_servers::list(db.get_connection()).await.unwrap();
    assert!(save(
        &service,
        AddServerRequest {
            id: Some(first.id.clone()),
            ..input()
        }
    )
    .await
    .is_err());
    assert_eq!(
        saved_servers::list(db.get_connection()).await.unwrap(),
        before
    );
}

#[tokio::test]
async fn connected_and_disconnected_responses_use_the_live_config_snapshot() {
    let (service, _, providers) = setup().await;
    let first = save(
        &service,
        AddServerRequest {
            password_value: Some(String::new()),
            ..input()
        },
    )
    .await
    .unwrap();
    let connected = service
        .connect_server(Request::new(ConnectServerRequest {
            id: first.id.clone(),
        }))
        .await
        .unwrap()
        .into_inner()
        .server
        .unwrap();
    assert!(connected.has_password);
    let original = providers.config().await.unwrap();
    assert_eq!(original.id, first.id);
    assert_eq!(original.password.as_deref(), Some(""));
    let listed = service
        .list_servers(Request::new(ListServersRequest {}))
        .await
        .unwrap()
        .into_inner()
        .servers;
    assert!(listed[0].has_password);
    let edited = save(
        &service,
        AddServerRequest {
            id: Some(first.id.clone()),
            name: "Moved".into(),
            url: "http://moved.test:8096".into(),
            clear_password: true,
            ..input()
        },
    )
    .await
    .unwrap();
    assert!(!edited.has_password);
    let live = service
        .get_connected_server(Request::new(GetConnectedServerRequest {}))
        .await
        .unwrap()
        .into_inner()
        .server
        .unwrap();
    assert_eq!(live.id, first.id);
    assert_eq!(live.name, "Family");
    assert_eq!(live.url, "http://saved.test:8096");
    assert!(live.connected && live.has_password);
    assert_eq!(providers.config().await.unwrap(), original);
    let disconnected = service
        .disconnect_server(Request::new(DisconnectServerRequest {}))
        .await
        .unwrap()
        .into_inner()
        .server
        .unwrap();
    assert_eq!(disconnected.url, "http://saved.test:8096");
    assert!(disconnected.has_password);
    assert!(!disconnected.connected);
    assert!(service
        .get_connected_server(Request::new(GetConnectedServerRequest {}))
        .await
        .unwrap()
        .into_inner()
        .server
        .is_none());
    service
        .connect_server(Request::new(ConnectServerRequest {
            id: first.id.clone(),
        }))
        .await
        .unwrap();
    let live = service
        .get_connected_server(Request::new(GetConnectedServerRequest {}))
        .await
        .unwrap()
        .into_inner()
        .server
        .unwrap();
    assert_eq!(live.url, "http://moved.test:8096");
    assert!(!live.has_password);
    let current = providers.config().await.unwrap();
    // Saving another account never changes which config is actually connected.
    save(
        &service,
        AddServerRequest {
            username: "guest".into(),
            ..input()
        },
    )
    .await
    .unwrap();
    assert_eq!(providers.config().await.unwrap(), current);
}
