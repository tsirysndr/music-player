use super::*;
use migration::{Migrator, MigratorTrait};

async fn db() -> DatabaseConnection {
    let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    db
}
fn remote(user: &str) -> RemoteIdentity {
    RemoteIdentity {
        server_id: "fixture".into(),
        user_id: user.into(),
    }
}
async fn bind_local_revision(
    db: &DatabaseConnection,
    row: &SourceAccount,
) -> Result<SourceAccount, Error> {
    bind_local_configuration(db, row, &config(row)?).await
}

#[tokio::test]
async fn legacy_binding_has_no_remote_identity_and_config_edits_require_reconnect() {
    let db = db().await;
    let initial = create_for_protocol(
        &db,
        "fixture.source",
        "Legacy",
        &BTreeMap::new(),
        "t0",
        AccountProtocol::V1,
    )
    .await
    .unwrap();
    assert_eq!(protocol(&initial).unwrap(), AccountProtocol::V1);
    assert!(initial.local_bound_revision.is_none());
    assert!(bind_remote_identity(&db, &initial, &remote("family"))
        .await
        .is_err());
    let bound = bind_local_revision(&db, &initial).await.unwrap();
    assert_eq!(bound.local_bound_revision, Some(1));
    assert!(bound.remote_server_id.is_none());
    assert!(bound.remote_user_id.is_none());
    let renamed = update(&db, &bound, "Renamed", &BTreeMap::new(), "t1")
        .await
        .unwrap();
    assert_eq!(renamed.local_bound_revision, Some(1));
    // A name-only change does not invalidate the source_info snapshot.
    assert_eq!(
        bind_local_revision(&db, &bound).await.unwrap().name,
        "Renamed"
    );
    let edited = update(
        &db,
        &renamed,
        "Renamed",
        &BTreeMap::from([("server".into(), ConfigChange::Set("other".into()))]),
        "t2",
    )
    .await
    .unwrap();
    assert_eq!(edited.config_revision, 2);
    assert!(edited.local_bound_revision.is_none());
    assert!(edited.local_config_json.is_none());
    assert!(bind_local_revision(&db, &bound).await.is_err());
    let current = bind_local_revision(&db, &edited).await.unwrap();
    assert_eq!(current.local_bound_revision, Some(2));
    let restored = update(
        &db,
        &current,
        "Renamed",
        &BTreeMap::from([("server".into(), ConfigChange::Remove)]),
        "t3",
    )
    .await
    .unwrap();
    assert_eq!(restored.config_json, initial.config_json);
    assert_eq!(restored.config_revision, 3);
    assert!(restored.local_bound_revision.is_none());
    assert!(bind_local_revision(&db, &initial).await.is_err());
    remove(&db, &restored.id).await.unwrap();
    assert!(bind_local_revision(&db, &restored).await.is_err());
}

#[tokio::test]
async fn source_protocol_and_binding_modes_cannot_be_crossed_by_stale_snapshots() {
    let db = db().await;
    let v2 = create(&db, "fixture.source", "V2", &BTreeMap::new(), "t0")
        .await
        .unwrap();
    assert!(bind_local_revision(&db, &v2).await.is_err());
    let mut forged = v2.clone();
    forged.source_api_version = 1;
    assert!(bind_local_revision(&db, &forged).await.is_err());
    assert!(update(&db, &forged, "Wrong", &BTreeMap::new(), "t1")
        .await
        .is_err());
    let v1 = create_for_protocol(
        &db,
        "fixture.source",
        "V1",
        &BTreeMap::new(),
        "t0",
        AccountProtocol::V1,
    )
    .await
    .unwrap();
    let mut forged = v1.clone();
    forged.source_api_version = 2;
    assert!(bind_remote_identity(&db, &forged, &remote("family"))
        .await
        .is_err());
    assert_eq!(get(&db, &v1.id).await.unwrap().unwrap(), v1);
    assert_eq!(get(&db, &v2.id).await.unwrap().unwrap(), v2);
}

#[tokio::test]
async fn changed_manifest_defaults_replace_local_scope_atomically_without_exposing_config() {
    let db = db().await;
    let initial = create_for_protocol(
        &db,
        "fixture.source",
        "Legacy",
        &BTreeMap::new(),
        "t0",
        AccountProtocol::V1,
    )
    .await
    .unwrap();
    let first = BTreeMap::from([("token".into(), "first-secret".into())]);
    let second = BTreeMap::from([("token".into(), "second-secret".into())]);
    let bound = bind_local_configuration(&db, &initial, &first)
        .await
        .unwrap();
    assert!(bind_local_configuration(&db, &initial, &first)
        .await
        .is_err());
    assert!(!format!("{bound:?}").contains("first-secret"));
    assert_eq!(
        bind_local_configuration(&db, &bound, &first).await.unwrap(),
        bound
    );
    let rebound = bind_local_configuration(&db, &bound, &second)
        .await
        .unwrap();
    assert_eq!(rebound.config_json, bound.config_json);
    assert_eq!(rebound.config_revision, bound.config_revision + 1);
    assert_eq!(rebound.local_bound_revision, Some(rebound.config_revision));
    assert!(bind_local_configuration(&db, &bound, &first).await.is_err());
    let back = bind_local_configuration(&db, &rebound, &first)
        .await
        .unwrap();
    assert_eq!(back.config_revision, bound.config_revision + 2);
    assert_ne!(back.local_bound_revision, bound.local_bound_revision);
    let mut max: account::ActiveModel = back.into();
    max.config_revision = Set(i64::MAX);
    max.local_bound_revision = Set(Some(i64::MAX));
    let max = max.update(&db).await.unwrap();
    assert!(bind_local_configuration(&db, &max, &second).await.is_err());
    assert_eq!(get(&db, &max.id).await.unwrap().unwrap(), max);
    assert_eq!(
        bind_local_configuration(&db, &max, &first).await.unwrap(),
        max
    );
}

#[tokio::test]
async fn stale_authentication_cannot_bind_after_config_edit_or_delete() {
    let db = db().await;
    let initial = create(&db, "fixture.source", "Family", &BTreeMap::new(), "t0")
        .await
        .unwrap();
    let renamed = update(&db, &initial, "Family renamed", &BTreeMap::new(), "t1")
        .await
        .unwrap();
    assert_eq!(initial.config_revision, renamed.config_revision);
    let bound = bind_remote_identity(&db, &initial, &remote("family"))
        .await
        .unwrap();
    assert_eq!(bound.name, "Family renamed");
    let changes = BTreeMap::from([("credential".into(), ConfigChange::Set("new".into()))]);
    let edited = update(&db, &bound, &bound.name, &changes, "t2")
        .await
        .unwrap();
    assert_eq!(edited.config_revision, bound.config_revision + 1);
    assert!(bind_remote_identity(&db, &bound, &remote("family"))
        .await
        .is_err());
    assert!(update(&db, &bound, "stale", &changes, "t3").await.is_err());
    assert!(bind_remote_identity(&db, &edited, &remote("other"))
        .await
        .is_err());
    bind_remote_identity(&db, &edited, &remote("family"))
        .await
        .unwrap();
    remove(&db, &edited.id).await.unwrap();
    assert!(bind_remote_identity(&db, &edited, &remote("family"))
        .await
        .is_err());
}

#[tokio::test]
async fn same_plugin_accounts_are_distinct_and_empty_override_is_not_removal() {
    let db = db().await;
    let values = BTreeMap::from([("credential".into(), "secret-fixture".into())]);
    let a = create(&db, "fixture.source", "A", &values, "t0")
        .await
        .unwrap();
    let b = create(&db, "fixture.source", "B", &values, "t0")
        .await
        .unwrap();
    assert_ne!(a.id, b.id);
    let empty = BTreeMap::from([("credential".into(), ConfigChange::Set(String::new()))]);
    let a = update(&db, &a, &a.name, &empty, "t1").await.unwrap();
    assert_eq!(
        config(&a).unwrap().get("credential").map(String::as_str),
        Some("")
    );
    let remove_override = BTreeMap::from([("credential".into(), ConfigChange::Remove)]);
    let a = update(&db, &a, &a.name, &remove_override, "t2")
        .await
        .unwrap();
    assert!(!config(&a).unwrap().contains_key("credential"));
    let stored_b = get(&db, &b.id).await.unwrap().unwrap();
    assert_eq!(config(&stored_b).unwrap(), values);
    assert!(!format!("{stored_b:?}").contains("secret-fixture"));
    assert_eq!(list(&db).await.unwrap().len(), 2);
}

#[tokio::test]
async fn competing_first_bind_cannot_replace_winning_identity() {
    let db = db().await;
    let snapshot = create(&db, "fixture.source", "A", &BTreeMap::new(), "t0")
        .await
        .unwrap();
    bind_remote_identity(&db, &snapshot, &remote("first"))
        .await
        .unwrap();
    assert!(bind_remote_identity(&db, &snapshot, &remote("second"))
        .await
        .is_err());
    let same = bind_remote_identity(&db, &snapshot, &remote("first"))
        .await
        .unwrap();
    assert_eq!(same.remote_user_id.as_deref(), Some("first"));
}

#[tokio::test]
async fn rename_rejects_stale_edit_but_allows_authentication_snapshot() {
    let db = db().await;
    let initial = create(&db, "fixture.source", "Original", &BTreeMap::new(), "t0")
        .await
        .unwrap();
    let renamed = update(&db, &initial, "Renamed", &BTreeMap::new(), "t1")
        .await
        .unwrap();
    let changes = BTreeMap::from([("credential".into(), ConfigChange::Set("new".into()))]);
    assert!(update(&db, &initial, &initial.name, &changes, "t2")
        .await
        .is_err());
    assert!(
        update(&db, &initial, "Other rename", &BTreeMap::new(), "t2")
            .await
            .is_err()
    );
    let bound = bind_remote_identity(&db, &initial, &remote("family"))
        .await
        .unwrap();
    assert_eq!(bound.name, renamed.name);
    assert_eq!(bound.config_revision, initial.config_revision);
    assert_eq!(bound.config_json, initial.config_json);
    let edited = update(&db, &bound, &bound.name, &changes, "t3")
        .await
        .unwrap();
    assert_eq!(edited.remote_server_id, bound.remote_server_id);
    assert_eq!(edited.remote_user_id, bound.remote_user_id);
}

#[tokio::test]
async fn config_aba_rejects_old_auth_and_edit_even_when_values_match_again() {
    let db = db().await;
    let initial = create(&db, "fixture.source", "A", &BTreeMap::new(), "t0")
        .await
        .unwrap();
    let changes = BTreeMap::from([("credential".into(), ConfigChange::Set(String::new()))]);
    let changed = update(&db, &initial, &initial.name, &changes, "t1")
        .await
        .unwrap();
    let restored = update(
        &db,
        &changed,
        &changed.name,
        &BTreeMap::from([("credential".into(), ConfigChange::Remove)]),
        "t2",
    )
    .await
    .unwrap();
    assert_eq!(restored.config_json, initial.config_json);
    assert_eq!(restored.config_revision, initial.config_revision + 2);
    assert!(bind_remote_identity(&db, &initial, &remote("family"))
        .await
        .is_err());
    assert!(update(&db, &initial, "Stale", &changes, "t3")
        .await
        .is_err());
    let current = bind_remote_identity(&db, &restored, &remote("family"))
        .await
        .unwrap();
    remove(&db, &current.id).await.unwrap();
    remove(&db, &current.id).await.unwrap(); // deletion is deliberately idempotent
    assert!(get(&db, &current.id).await.unwrap().is_none());
    assert!(update(&db, &current, "Deleted", &changes, "t4")
        .await
        .is_err());
    assert!(bind_remote_identity(&db, &current, &remote("family"))
        .await
        .is_err());
    let replacement = create(&db, "fixture.source", "A", &BTreeMap::new(), "t5")
        .await
        .unwrap();
    assert_ne!(replacement.id, initial.id);
}

#[tokio::test]
async fn partial_binding_is_rejected_in_snapshot_and_in_current_row() {
    let db = db().await;
    for (server, user) in [(Some("fixture"), None), (None, Some("family"))] {
        let unbound = create(&db, "fixture.source", "A", &BTreeMap::new(), "t0")
            .await
            .unwrap();
        // Simulate an incomplete legacy/manual row; production bind must not repair it.
        let mut row: account::ActiveModel = unbound.clone().into();
        row.remote_server_id = Set(server.map(str::to_owned));
        row.remote_user_id = Set(user.map(str::to_owned));
        let partial = row.update(&db).await.unwrap();
        assert!(bind_remote_identity(&db, &unbound, &remote("family"))
            .await
            .is_err());
        assert!(bind_remote_identity(&db, &partial, &remote("family"))
            .await
            .is_err());
        let stored = get(&db, &unbound.id).await.unwrap().unwrap();
        assert_eq!(stored.remote_server_id, partial.remote_server_id);
        assert_eq!(stored.remote_user_id, partial.remote_user_id);
    }
}

#[tokio::test]
async fn revision_overflow_is_atomic_but_unchanged_config_can_still_be_renamed() {
    let db = db().await;
    let values = BTreeMap::from([("credential".into(), "fixture-secret".into())]);
    let initial = create(&db, "fixture.source", "A", &values, "t0")
        .await
        .unwrap();
    let bound = bind_remote_identity(&db, &initial, &remote("family"))
        .await
        .unwrap();
    let mut row: account::ActiveModel = bound.into();
    row.config_revision = Set(i64::MAX);
    let max = row.update(&db).await.unwrap();
    let error = update(
        &db,
        &max,
        "Must not rename",
        &BTreeMap::from([("credential".into(), ConfigChange::Remove)]),
        "t1",
    )
    .await
    .unwrap_err();
    assert_eq!(
        format!("{error:#}"),
        "extension source account operation failed"
    );
    assert_eq!(get(&db, &max.id).await.unwrap().unwrap(), max);
    // Setting the same value or removing an absent key is a configuration no-op.
    let same = BTreeMap::from([
        (
            "credential".into(),
            ConfigChange::Set("fixture-secret".into()),
        ),
        ("absent".into(), ConfigChange::Remove),
    ]);
    let renamed = update(&db, &max, "Renamed", &same, "t2").await.unwrap();
    assert_eq!(renamed.config_revision, i64::MAX);
    assert_eq!(renamed.remote_server_id, max.remote_server_id);
    assert_eq!(renamed.remote_user_id, max.remote_user_id);
}

#[tokio::test]
async fn invalid_config_and_database_errors_do_not_expose_values_or_error_chains() {
    let db = db().await;
    let secret = "synthetic-credential-never-in-errors";
    let initial = create(
        &db,
        "fixture.source",
        "A",
        &BTreeMap::from([("credential".into(), secret.into())]),
        "t0",
    )
    .await
    .unwrap();
    let mut invalid = initial.clone();
    invalid.config_json = format!("{{\"credential\":\"{secret}\"");
    let error = config(&invalid).unwrap_err();
    assert_eq!(
        format!("{error:#}"),
        "extension source account operation failed"
    );
    let error = update(&db, &invalid, "Changed", &BTreeMap::new(), "t1")
        .await
        .unwrap_err();
    assert_eq!(
        format!("{error:#}"),
        "extension source account operation failed"
    );
    assert_eq!(error.chain().count(), 1);
    assert!(!format!("{error:?}").contains(secret));
    assert_eq!(get(&db, &initial.id).await.unwrap().unwrap(), initial);
    assert!(!format!("{:?}", ConfigChange::Set(secret.into())).contains(secret));
    // Deliberately unmigrated separate DB: real SQL errors must lose driver context
    // even when the failed statement has credential-bearing bound parameters.
    let unavailable = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
    let errors = [
        create(
            &unavailable,
            "fixture.source",
            "A",
            &config(&initial).unwrap(),
            "t0",
        )
        .await
        .unwrap_err(),
        get(&unavailable, &initial.id).await.unwrap_err(),
        list(&unavailable).await.unwrap_err(),
        update(&unavailable, &initial, "A", &BTreeMap::new(), "t1")
            .await
            .unwrap_err(),
        bind_remote_identity(&unavailable, &initial, &remote("family"))
            .await
            .unwrap_err(),
        remove(&unavailable, &initial.id).await.unwrap_err(),
    ];
    for error in errors {
        assert_eq!(
            format!("{error:#}"),
            "extension source account operation failed"
        );
        assert_eq!(error.chain().count(), 1);
        // Debug may include a CI-enabled backtrace, but never the credentials.
        assert!(!format!("{error:?}").contains(secret));
    }
}
