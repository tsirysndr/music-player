use async_graphql::{EmptyMutation, EmptySubscription, Request, Schema, Variables};
use migration::async_trait::async_trait;
use music_player_provider::{
    Album, Artist, MediaEntry, MusicProvider, Page, ProviderCapabilities, ProviderConfig,
    ProviderError, ProviderFactory, ProviderRegistry, ProviderState, Track,
};
use music_player_types::source::{RemoteIdentity, ResourceKind, SourceRef};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

use crate::schema::media::MediaQuery;

type TestSchema = Schema<MediaQuery, EmptyMutation, EmptySubscription>;

#[derive(Default)]
struct Fixture {
    calls: Mutex<Vec<(bool, Option<String>, Page)>>,
    gate: Option<(Notify, Notify)>,
}
struct Factory(Arc<Fixture>);
struct Provider(Arc<Fixture>, String);

fn source(account: &str, kind: ResourceKind, id: &str) -> String {
    SourceRef {
        resolver: "emby".into(),
        account_id: account.into(),
        remote: RemoteIdentity {
            server_id: "server".into(),
            user_id: "family".into(),
        },
        kind,
        item_id: id.into(),
    }
    .to_handle()
}

fn leaf(account: &str) -> Track {
    let handle = source(account, ResourceKind::Item, "55508");
    Track {
        id: handle.clone(),
        uri: handle,
        title: "My First Ever Ender Dragon Fight! [First Ever Minecraft Playthrough Ep.40]".into(),
        ..Default::default()
    }
}

#[async_trait]
impl ProviderFactory for Factory {
    fn kind(&self) -> &'static str {
        "media-fixture"
    }
    fn display_name(&self) -> &'static str {
        "Media fixture"
    }
    fn default_port(&self) -> u16 {
        8096
    }
    async fn connect(
        &self,
        config: &ProviderConfig,
    ) -> Result<Arc<dyn MusicProvider>, ProviderError> {
        Ok(Arc::new(Provider(self.0.clone(), config.id.clone())))
    }
}

#[async_trait]
impl MusicProvider for Provider {
    fn kind(&self) -> &'static str {
        "media-fixture"
    }
    fn base_url(&self) -> &str {
        "http://fixture.invalid:8096"
    }
    fn host(&self) -> &str {
        "fixture.invalid"
    }
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            media_browse: true,
            ..Default::default()
        }
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
        Ok(leaf(&self.1))
    }
    async fn browse(
        &self,
        parent: Option<&str>,
        page: Page,
    ) -> Result<Vec<MediaEntry>, ProviderError> {
        self.0
            .calls
            .lock()
            .unwrap()
            .push((false, parent.map(str::to_owned), page));
        if let Some((started, release)) = &self.0.gate {
            started.notify_one();
            release.notified().await;
        }
        Ok(page.slice(vec![
            MediaEntry {
                id: source(&self.1, ResourceKind::Container, "season"),
                title: "Season".into(),
                item_type: Some("Season".into()),
                media_type: None,
                is_container: true,
                season_number: Some(u64::MAX),
                episode_number: None,
                track: None,
            },
            MediaEntry {
                id: leaf(&self.1).id,
                title: leaf(&self.1).title,
                item_type: Some("Episode".into()),
                media_type: Some("Video".into()),
                is_container: false,
                season_number: Some(2_025_050_473),
                episode_number: Some(41),
                track: Some(leaf(&self.1)),
            },
        ]))
    }
    async fn container_tracks(
        &self,
        parent: &str,
        page: Page,
    ) -> Result<Vec<Track>, ProviderError> {
        self.0
            .calls
            .lock()
            .unwrap()
            .push((true, Some(parent.into()), page));
        if parent == "failed-page" {
            return Err(ProviderError::Other("fixture page failed".into()));
        }
        Ok(vec![leaf(&self.1)])
    }
}

async fn connect(providers: &ProviderState, id: &str) {
    let mut config = ProviderConfig::new("media-fixture", "Family", "http://fixture.invalid:8096");
    config.id = id.into();
    providers.connect(config).await.unwrap();
}
async fn setup(fixture: Fixture) -> (TestSchema, Arc<ProviderState>, Arc<Fixture>) {
    let fixture = Arc::new(fixture);
    let mut registry = ProviderRegistry::new();
    registry.register(Factory(fixture.clone()));
    let providers = Arc::new(ProviderState::new(Arc::new(registry)));
    connect(&providers, "account-a").await;
    let schema = Schema::build(MediaQuery, EmptyMutation, EmptySubscription)
        .data(providers.clone())
        .finish();
    (schema, providers, fixture)
}

fn browse(account: &str, offset: i32, limit: i32) -> Request {
    Request::new("query($server: ID!, $offset: Int!, $limit: Int!) { browseMedia(serverId: $server, offset: $offset, limit: $limit) { serverId nextOffset entries { id title itemType mediaType isContainer seasonNumber episodeNumber track { id uri albumId discNumber trackNumber } } } }")
        .variables(Variables::from_json(json!({"server": account, "offset": offset, "limit": limit})))
}

#[tokio::test]
async fn navigation_preserves_large_indices_and_handle_metadata_across_pages() {
    let (schema, _, fixture) = setup(Fixture::default()).await;
    let first = schema.execute(browse("account-a", 0, 1)).await;
    assert!(first.errors.is_empty(), "{:?}", first.errors);
    let first = first.data.into_json().unwrap()["browseMedia"].clone();
    assert_eq!(first["serverId"], "account-a");
    assert_eq!(first["nextOffset"], 1);
    assert_eq!(first["entries"][0]["seasonNumber"], u64::MAX.to_string());
    assert_eq!(first["entries"][0]["track"], Value::Null);
    let second = schema.execute(browse("account-a", 1, 1)).await;
    assert!(second.errors.is_empty(), "{:?}", second.errors);
    let second = second.data.into_json().unwrap()["browseMedia"].clone();
    assert_eq!(second["nextOffset"], Value::Null);
    let row = &second["entries"][0];
    assert_eq!(row["seasonNumber"], "2025050473");
    assert_eq!(row["episodeNumber"], "41");
    assert_eq!(row["title"], leaf("account-a").title);
    assert_eq!(row["track"]["id"], row["id"]);
    assert_eq!(row["track"]["uri"], row["id"]);
    assert_eq!(row["track"]["albumId"], "");
    assert_eq!(row["track"]["discNumber"], 0);
    assert_eq!(row["track"]["trackNumber"], Value::Null);
    assert_eq!(fixture.calls.lock().unwrap()[0].2, Page::new(0, 2));
}

#[tokio::test]
async fn changed_account_and_invalid_pages_are_rejected_before_provider_reads() {
    let (schema, providers, fixture) = setup(Fixture::default()).await;
    connect(&providers, "account-b").await;
    for request in [
        browse("account-a", 0, 100),
        browse("account-b", -1, 100),
        browse("account-b", 0, 0),
        browse("account-b", i32::MAX, 100),
    ] {
        assert!(!schema.execute(request).await.errors.is_empty());
    }
    providers.disconnect().await;
    let result = schema
        .execute("{ mediaBrowser { supported serverId } }")
        .await;
    assert!(result.errors.is_empty());
    assert_eq!(
        result.data.into_json().unwrap()["mediaBrowser"],
        Value::Null
    );
    assert!(fixture.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn in_flight_browse_remains_labeled_with_its_original_account() {
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let (schema, providers, fixture) = setup(Fixture {
            gate: Some((Notify::new(), Notify::new())),
            ..Default::default()
        })
        .await;
        let pending =
            tokio::spawn(async move { schema.execute(browse("account-a", 0, 100)).await });
        let (started, release) = fixture.gate.as_ref().unwrap();
        started.notified().await;
        connect(&providers, "account-b").await;
        release.notify_one();
        let response = pending.await.unwrap();
        assert!(response.errors.is_empty());
        assert_eq!(
            response.data.into_json().unwrap()["browseMedia"]["serverId"],
            "account-a"
        );
        assert_eq!(providers.config().await.unwrap().id, "account-b");
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn container_expansion_forwards_all_pages_and_propagates_errors() {
    let (schema, _, fixture) = setup(Fixture::default()).await;
    let query = "query($parent: ID!) { mediaContainerTracks(serverId: \"account-a\", parent: $parent, limit: 0) { id uri } }";
    let parent = source("account-a", ResourceKind::Container, "season");
    let response = schema
        .execute(Request::new(query).variables(Variables::from_json(json!({"parent": parent}))))
        .await;
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    let rows = response.data.into_json().unwrap()["mediaContainerTracks"].clone();
    assert_eq!(rows[0]["id"], leaf("account-a").id);
    assert_eq!(rows[0]["id"], rows[0]["uri"]);
    assert_eq!(
        fixture.calls.lock().unwrap()[0],
        (true, Some(parent), Page::new(0, 0))
    );
    let failed = schema
        .execute(
            Request::new(query).variables(Variables::from_json(json!({"parent": "failed-page"}))),
        )
        .await;
    assert_eq!(failed.errors[0].message, "fixture page failed");
}
