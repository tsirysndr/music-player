//! SQLite-only RPC admission tests. No decoder, guest or production network.
use super::*;
use migration::{Migrator, MigratorTrait};
use music_player_storage::saved_servers::{self, NewServer, PasswordUpdate};
use music_player_types::source::{LocalSourceRef, RemoteIdentity, ResourceKind};

#[tokio::test]
async fn local_browse_metadata_enqueues_but_mixed_revisions_and_id_lookup_do_not() {
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        use music_player_playback::extension_accounts::ExtensionAccounts;
        use music_player_storage::extension_source_accounts::{self as accounts, AccountProtocol};
        let db = database().await;
        let config = std::collections::BTreeMap::new();
        let row = accounts::create_for_protocol(db.get_connection(), "fixture.legacy", "Legacy",
            &config, "t0", AccountProtocol::V1).await.unwrap();
        // This is an admission test: install a known bound DB row, without
        // claiming connect or guest execution. Empty paths detect accidental loading.
        let row = accounts::bind_local_configuration(db.get_connection(), &row, &config).await.unwrap();
        let source = LocalSourceRef { resolver: "extension:fixture.legacy".into(), account_id: row.id,
            config_revision: row.config_revision as u64, kind: ResourceKind::Item, item_id: "item".into() };
        let manager = ExtensionAccounts::new(db.clone(), vec![], Default::default());
        let sources = SourceResolver::from_settings(db.clone()).with_extensions(manager.clone(), vec!["mp3".into()]);
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let service = Tracklist::new(Arc::new(std::sync::Mutex::new(TracklistState::new_empty())),
            Arc::new(std::sync::Mutex::new(tx)), db).with_source_resolver(sources);
        let metadata = |source: &LocalSourceRef| Track { id: source.to_handle(), uri: source.to_handle(),
            title: "Browse title".into(), ..Default::default() };
        service.add_tracks(tonic::Request::new(AddTracksRequest { tracks: vec![metadata(&source)] })).await.unwrap();
        assert!(matches!(rx.try_recv().unwrap(), PlayerCommand::LoadTracklist { tracks, .. }
            if tracks.len() == 1 && tracks[0].title == "Browse title" && tracks[0].uri == source.to_handle()));
        let mut wrong = source.clone();
        wrong.config_revision += 1;
        let error = service.add_tracks(tonic::Request::new(AddTracksRequest {
            tracks: vec![metadata(&source), metadata(&wrong)] })).await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::FailedPrecondition);
        assert!(matches!(rx.try_recv(), Err(tokio::sync::mpsc::error::TryRecvError::Empty)));
        let error = service.add_track(tonic::Request::new(AddTrackRequest { track: Some(metadata(&source)) }))
            .await.unwrap_err();
        assert_eq!(error.code(), tonic::Code::FailedPrecondition);
        assert!(error.message().contains("requires browse metadata"));
        assert!(matches!(rx.try_recv(), Err(tokio::sync::mpsc::error::TryRecvError::Empty)));
        assert!(manager.status().is_empty(), "admission must not load a guest");
    }).await.expect("legacy RPC admission deadline");
}

async fn database() -> Database {
    let connection = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
    Migrator::up(&connection, None).await.unwrap();
    Database { connection }
}

#[tokio::test]
async fn injected_resolver_validates_its_account_and_distinguishes_namespaces_before_enqueue() {
    let source_db = database().await;
    let row = saved_servers::upsert(
        source_db.get_connection(),
        &NewServer::new("emby", "Family", "http://127.0.0.1:1")
            .with_credentials(Some("family".into()), None)
            .with_password_update(PasswordUpdate::Set(String::new())),
        "t0",
    )
    .await
    .unwrap();
    let remote = RemoteIdentity {
        server_id: "server".into(),
        user_id: "family".into(),
    };
    saved_servers::bind_remote_identity(source_db.get_connection(), &row, &remote)
        .await
        .unwrap();
    let native = SourceRef {
        resolver: "emby".into(),
        account_id: row.id,
        remote,
        kind: ResourceKind::Item,
        item_id: "55508".into(),
    };
    // The service DB intentionally has no account: success requires using the
    // injected resolver rather than recreating one from the service connection.
    let service_db = database().await;
    let sources = SourceResolver::from_settings(source_db);
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let service = Tracklist::new(
        Arc::new(std::sync::Mutex::new(TracklistState::new_empty())),
        Arc::new(std::sync::Mutex::new(tx)),
        service_db,
    )
    .with_source_resolver(sources);
    let metadata = |source: &SourceRef| Track {
        id: source.to_handle(),
        uri: source.to_handle(),
        title: "Episode".into(),
        ..Default::default()
    };
    service
        .add_tracks(tonic::Request::new(AddTracksRequest {
            tracks: vec![metadata(&native)],
        }))
        .await
        .unwrap();
    assert!(
        matches!(rx.try_recv().unwrap(), PlayerCommand::LoadTracklist { tracks, .. } if tracks.len() == 1)
    );
    let mut extension = native.clone();
    extension.resolver = SourceRef::extension_resolver("fixture.source").unwrap();
    let error = service
        .add_tracks(tonic::Request::new(AddTracksRequest {
            tracks: vec![metadata(&native), metadata(&extension)],
        }))
        .await
        .unwrap_err();
    assert_eq!(error.code(), tonic::Code::FailedPrecondition);
    assert_eq!(
        error.message(),
        "extension source runtime is not configured"
    );
    assert!(matches!(
        rx.try_recv(),
        Err(tokio::sync::mpsc::error::TryRecvError::Empty)
    ));
}
