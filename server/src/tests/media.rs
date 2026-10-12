//! Generated gRPC round trips; SQLite is empty so no local-library fallback can pass.
use crate::{
    api::music::v1alpha1::{
        library_service_client::LibraryServiceClient, library_service_server::LibraryServiceServer,
        BrowseMediaRequest, GetMediaBrowserRequest, GetMediaContainerTracksRequest,
    },
    library::Library,
};
use migration::async_trait::async_trait;
use music_player_provider::{
    Album, Artist, MediaEntry, MusicProvider, Page, ProviderCapabilities, ProviderConfig,
    ProviderError, ProviderFactory, ProviderRegistry, ProviderState, Track,
};
use music_player_types::source::{RemoteIdentity, ResourceKind, SourceRef};
use std::sync::{Arc, Mutex};
use tokio::{net::TcpListener, sync::Notify};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{
    transport::{Channel, Server},
    Code,
};

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
            media_browse: self.1 != "unsupported",
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
        if parent == "large-library" {
            return Ok(page.slice(
                (0..17_462)
                    .map(|index| {
                        let mut track = leaf(&self.1);
                        let handle = source(&self.1, ResourceKind::Item, &index.to_string());
                        track.id = handle.clone();
                        track.uri = handle;
                        track
                    })
                    .collect(),
            ));
        }
        Ok(vec![leaf(&self.1)])
    }
}

async fn connect(providers: &ProviderState, id: &str) {
    let mut config = ProviderConfig::new("media-fixture", "Family", "http://fixture.invalid:8096");
    config.id = id.into();
    providers.connect(config).await.unwrap();
}
struct TestServer {
    client: LibraryServiceClient<Channel>,
    providers: Arc<ProviderState>,
    fixture: Arc<Fixture>,
    queue: Arc<Mutex<music_player_tracklist::Tracklist>>,
    task: tokio::task::JoinHandle<()>,
}

#[tokio::test]
async fn grpc_full_container_above_default_message_limit_preserves_every_source() {
    use prost::Message;
    let mut server = setup(Fixture::default()).await;
    let response = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        server
            .client
            .get_media_container_tracks(GetMediaContainerTracksRequest {
                server_id: "account-a".into(),
                parent: "large-library".into(),
                offset: 0,
                limit: Some(0),
            }),
    )
    .await
    .unwrap()
    .unwrap()
    .into_inner();
    assert!(response.encoded_len() > 4 * 1024 * 1024);
    assert_eq!(response.server_id, "account-a");
    assert_eq!(response.tracks.len(), 17_462);
    for (index, track) in response.tracks.iter().enumerate() {
        let expected = source("account-a", ResourceKind::Item, &index.to_string());
        assert_eq!(track.id, expected);
        assert_eq!(track.uri, expected);
    }
}
impl Drop for TestServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}
async fn setup(fixture: Fixture) -> TestServer {
    let fixture = Arc::new(fixture);
    let mut registry = ProviderRegistry::new();
    registry.register(Factory(fixture.clone()));
    let providers = Arc::new(ProviderState::new(Arc::new(registry)));
    connect(&providers, "account-a").await;
    let mut options = sea_orm::ConnectOptions::new("sqlite::memory:");
    options.max_connections(1).sqlx_logging(false);
    let db = music_player_storage::Database {
        connection: sea_orm::Database::connect(options).await.unwrap(),
    };
    let queue = Arc::new(Mutex::new(music_player_tracklist::Tracklist::new(vec![
        leaf("queued-account").into(),
    ])));
    let library = Library::new(db, providers.clone(), queue.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        Server::builder()
            .add_service(LibraryServiceServer::new(library))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .unwrap();
    });
    let client = LibraryServiceClient::connect(endpoint)
        .await
        .unwrap()
        .max_decoding_message_size(crate::LIBRARY_MESSAGE_LIMIT);
    TestServer {
        client,
        providers,
        fixture,
        queue,
        task,
    }
}
fn browse(account: &str, offset: i32, limit: Option<i32>) -> BrowseMediaRequest {
    BrowseMediaRequest {
        server_id: account.into(),
        parent: None,
        offset,
        limit,
    }
}

#[tokio::test]
async fn grpc_navigation_preserves_large_indices_and_stable_sources_across_pages() {
    let mut server = setup(Fixture::default()).await;
    let browser = server
        .client
        .get_media_browser(GetMediaBrowserRequest {})
        .await
        .unwrap()
        .into_inner()
        .browser
        .unwrap();
    assert_eq!(browser.server_id, "account-a");
    assert!(browser.supported);
    let first = server
        .client
        .browse_media(browse("account-a", 0, Some(1)))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(first.server_id, "account-a");
    assert_eq!(first.next_offset, Some(1));
    assert_eq!(first.entries[0].season_number, Some(u64::MAX));
    assert!(first.entries[0].is_container);
    assert!(first.entries[0].track.is_none());
    let second = server
        .client
        .browse_media(browse("account-a", 1, Some(1)))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(second.next_offset, None);
    let row = &second.entries[0];
    assert_eq!(row.season_number, Some(2_025_050_473));
    assert_eq!(row.episode_number, Some(41));
    assert_eq!(row.title, leaf("account-a").title);
    assert_eq!(row.track.as_ref().unwrap().id, row.id);
    assert_eq!(row.track.as_ref().unwrap().uri, row.id);
    assert!(row.track.as_ref().unwrap().album.is_none());
    assert_eq!(server.fixture.calls.lock().unwrap()[0].2, Page::new(0, 2));
    server
        .client
        .browse_media(browse("account-a", 0, None))
        .await
        .unwrap();
    assert_eq!(
        server.fixture.calls.lock().unwrap().last().unwrap().2,
        Page::new(0, 101)
    );
}

#[tokio::test]
async fn grpc_rejects_wrong_account_unsupported_and_bad_pages_before_provider_reads() {
    let mut server = setup(Fixture::default()).await;
    connect(&server.providers, "account-b").await;
    assert_eq!(
        server
            .client
            .browse_media(browse("account-a", 0, None))
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition
    );
    for (offset, limit) in [
        (-1, Some(100)),
        (0, Some(0)),
        (0, Some(501)),
        (i32::MAX, Some(1)),
    ] {
        assert_eq!(
            server
                .client
                .browse_media(browse("account-b", offset, limit))
                .await
                .unwrap_err()
                .code(),
            Code::InvalidArgument
        );
    }
    connect(&server.providers, "unsupported").await;
    assert!(
        !server
            .client
            .get_media_browser(GetMediaBrowserRequest {})
            .await
            .unwrap()
            .into_inner()
            .browser
            .unwrap()
            .supported
    );
    assert_eq!(
        server
            .client
            .browse_media(browse("unsupported", 0, None))
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition
    );
    server.providers.disconnect().await;
    assert!(server
        .client
        .get_media_browser(GetMediaBrowserRequest {})
        .await
        .unwrap()
        .into_inner()
        .browser
        .is_none());
    assert!(server.fixture.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn grpc_in_flight_browse_stays_labelled_with_its_original_account() {
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let server = setup(Fixture {
            gate: Some((Notify::new(), Notify::new())),
            ..Default::default()
        })
        .await;
        let mut client = server.client.clone();
        let pending =
            tokio::spawn(async move { client.browse_media(browse("account-a", 0, None)).await });
        let (started, release) = server.fixture.gate.as_ref().unwrap();
        started.notified().await;
        connect(&server.providers, "account-b").await;
        release.notify_one();
        let response = pending.await.unwrap().unwrap().into_inner();
        assert_eq!(response.server_id, "account-a");
        assert_eq!(
            response.entries[1].track.as_ref().unwrap().id,
            leaf("account-a").id
        );
        assert_eq!(server.providers.config().await.unwrap().id, "account-b");
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn grpc_container_expansion_is_complete_or_error_without_queue_changes() {
    let mut server = setup(Fixture::default()).await;
    let parent = source("account-a", ResourceKind::Container, "season");
    let request = GetMediaContainerTracksRequest {
        server_id: "account-a".into(),
        parent: parent.clone(),
        offset: 0,
        limit: Some(0),
    };
    let result = server
        .client
        .get_media_container_tracks(request.clone())
        .await
        .unwrap()
        .into_inner();
    assert_eq!(result.server_id, "account-a");
    assert_eq!(result.tracks[0].id, leaf("account-a").id);
    assert_eq!(
        server.fixture.calls.lock().unwrap()[0],
        (true, Some(parent), Page::new(0, 0))
    );
    let failed = server
        .client
        .get_media_container_tracks(GetMediaContainerTracksRequest {
            parent: "failed-page".into(),
            ..request.clone()
        })
        .await
        .unwrap_err();
    assert_eq!(failed.code(), Code::Unavailable);
    let count = server.fixture.calls.lock().unwrap().len();
    assert_eq!(
        server
            .client
            .get_media_container_tracks(GetMediaContainerTracksRequest {
                offset: -1,
                ..request.clone()
            })
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    assert_eq!(
        server
            .client
            .get_media_container_tracks(GetMediaContainerTracksRequest {
                server_id: "account-b".into(),
                ..request
            })
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition
    );
    assert_eq!(server.fixture.calls.lock().unwrap().len(), count);
    let queue = server.queue.lock().unwrap();
    assert_eq!(queue.len(), 1);
    assert!(queue.current_track().0.is_none());
    let (played, queued) = queue.tracks();
    assert!(played.is_empty());
    assert_eq!(queued[0].id, leaf("queued-account").id);
}
