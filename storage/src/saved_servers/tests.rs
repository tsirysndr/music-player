use super::*;
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;

const NOW: &str = "2026-09-30T10:00:00Z";
const LATER: &str = "2026-09-30T11:00:00Z";

async fn db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    db
}

fn account(user: &str) -> NewServer {
    NewServer::new("jellyfin", "Home", "http://emby:8096/")
        .with_credentials(Some(user.into()), Some(format!("password-{user}")))
}

#[tokio::test]
async fn migration_retains_old_id_and_replaces_url_uniqueness() {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    // Apply the actual old schema, then insert an old deterministic ID.
    let old_count = Migrator::migrations()
        .iter()
        .position(|migration| migration.name() == "m20260930_000001_saved_accounts")
        .unwrap();
    Migrator::up(&db, Some(old_count as u32)).await.unwrap();
    let old_id = format!("{:x}", md5::compute("jellyfin\0http://emby:8096"));
    db.execute_raw(Statement::from_sql_and_values(
        DbBackend::Sqlite,
        "INSERT INTO saved_server(id,kind,name,url,username,password,created_at,updated_at)
        VALUES (?, 'jellyfin','Old','http://emby:8096','family','old-secret',?,?)",
        [old_id.clone().into(), NOW.into(), NOW.into()],
    ))
    .await
    .unwrap();
    db.execute_unprepared(
        "INSERT INTO saved_server(id,kind,name,url,username) VALUES
        ('empty-user','music-player','Peer','http://peer','　'),
        ('spaced-user','jellyfin','Other','http://other',' family ')",
    )
    .await
    .unwrap();
    Migrator::up(&db, None).await.unwrap();
    assert_eq!(
        get(&db, "empty-user").await.unwrap().unwrap().username,
        None
    );
    assert_eq!(
        get(&db, "spaced-user")
            .await
            .unwrap()
            .unwrap()
            .username
            .as_deref(),
        Some(" family ")
    );
    delete(&db, "empty-user").await.unwrap();
    delete(&db, "spaced-user").await.unwrap();
    let existing = get(&db, &old_id).await.unwrap().unwrap();
    let saved = upsert(
        &db,
        &account("family").with_password_update(PasswordUpdate::Keep),
        LATER,
    )
    .await
    .unwrap();
    assert_eq!(saved.id, old_id);
    assert_eq!(saved.created_at, existing.created_at);
    assert_eq!(saved.password, existing.password);
    let other = upsert(&db, &account("guest"), LATER).await.unwrap();
    assert_ne!(other.id, old_id);
    assert!(Uuid::parse_str(&other.id).is_ok());
    // Reach saved_accounts even after later migrations have been added. A
    // downgrade cannot drop either account to restore the former unique index.
    let down_steps = (Migrator::migrations().len() - old_count) as u32;
    assert!(Migrator::down(&db, Some(down_steps)).await.is_err());
    // SQLite may already have removed the identity columns before that failure.
    // Restore the current schema before querying it through the current entity.
    Migrator::up(&db, None).await.unwrap();
    assert_eq!(list(&db).await.unwrap().len(), 2);
    assert!(delete(&db, &old_id).await.unwrap());
    assert!(get(&db, &old_id).await.unwrap().is_none());
    assert_eq!(get(&db, &other.id).await.unwrap(), Some(other));
}

#[tokio::test]
async fn same_address_accounts_never_inherit_each_others_password() {
    let db = db().await;
    let a = upsert(&db, &account("a"), NOW).await.unwrap();
    let b = upsert(
        &db,
        &account("b").with_password_update(PasswordUpdate::Keep),
        NOW,
    )
    .await
    .unwrap();
    assert_ne!(a.id, b.id);
    assert_eq!(b.password, None);
    let again = upsert(
        &db,
        &account("a").with_password_update(PasswordUpdate::Keep),
        LATER,
    )
    .await
    .unwrap();
    assert_eq!(again.id, a.id);
    assert_eq!(again.password, a.password);
    assert_eq!(again.created_at, a.created_at);
    assert_eq!(again.updated_at.as_deref(), Some(LATER));
    let spaced = upsert(&db, &account(" a "), NOW).await.unwrap();
    assert_eq!(spaced.username.as_deref(), Some(" a "));
    assert_ne!(spaced.id, a.id);
    assert_eq!(list(&db).await.unwrap().len(), 3);
}

#[tokio::test]
async fn explicit_edits_keep_identity_and_reject_retargeting_or_collision() {
    let db = db().await;
    let a = upsert(&db, &account("family"), NOW).await.unwrap();
    let mut edit = account("family").with_id(Some(a.id.clone()));
    edit.name = "Renamed".into();
    edit.url = " http://new-address:8096/ ".into();
    edit.password_update = PasswordUpdate::Set(String::new());
    let moved = upsert(&db, &edit, LATER).await.unwrap();
    assert_eq!(moved.id, a.id);
    assert_eq!(moved.created_at, a.created_at);
    assert_eq!(moved.url, "http://new-address:8096");
    assert_eq!(moved.name, "Renamed");
    assert_eq!(moved.password.as_deref(), Some(""));
    let original_address = upsert(&db, &account("family"), NOW).await.unwrap();
    edit.url = original_address.url.clone();
    assert!(upsert(&db, &edit, LATER).await.is_err());
    assert_eq!(get(&db, &moved.id).await.unwrap(), Some(moved.clone()));
    assert_eq!(
        get(&db, &original_address.id).await.unwrap(),
        Some(original_address)
    );
    edit.url = moved.url.clone();
    edit.username = Some("other".into());
    assert!(upsert(&db, &edit, LATER).await.is_err());
    edit.username = moved.username.clone();
    edit.kind = "subsonic".into();
    assert!(upsert(&db, &edit, LATER).await.is_err());
    assert_eq!(get(&db, &moved.id).await.unwrap(), Some(moved));
    edit.id = Some("missing".into());
    assert!(upsert(&db, &edit, LATER).await.is_err());
}

#[tokio::test]
async fn deletion_never_reuses_an_old_handle() {
    let db = db().await;
    let a = upsert(&db, &account("family"), NOW).await.unwrap();
    assert!(delete(&db, &a.id).await.unwrap());
    assert!(!delete(&db, &a.id).await.unwrap());
    let b = upsert(&db, &account("family"), NOW).await.unwrap();
    assert_ne!(a.id, b.id);
    assert!(get(&db, &a.id).await.unwrap().is_none());
}

#[tokio::test]
async fn password_keep_set_empty_and_clear_survive_storage() {
    let db = db().await;
    let a = upsert(&db, &account("family"), NOW).await.unwrap();
    let legacy_empty =
        account("family").with_credentials(Some("family".into()), Some(String::new()));
    assert_eq!(
        upsert(&db, &legacy_empty, LATER).await.unwrap().password,
        a.password
    );
    for id in [None, Some(a.id.clone())] {
        let input = account("family").with_id(id);
        for (update, expected) in [
            (PasswordUpdate::Set(String::new()), Some("")),
            (PasswordUpdate::Keep, Some("")),
            (PasswordUpdate::Set("   ".into()), Some("   ")),
            (PasswordUpdate::Clear, None),
            (PasswordUpdate::Keep, None),
        ] {
            let saved = upsert(&db, &input.clone().with_password_update(update), LATER)
                .await
                .unwrap();
            assert_eq!(saved.password.as_deref(), expected);
        }
    }
}

#[test]
fn shared_api_password_compatibility_and_conflicts() {
    for legacy in [None, Some(String::new())] {
        assert_eq!(
            PasswordUpdate::from_fields(legacy.clone(), None, false).unwrap(),
            PasswordUpdate::Keep
        );
        assert_eq!(
            PasswordUpdate::from_fields(legacy.clone(), Some(String::new()), false).unwrap(),
            PasswordUpdate::Set(String::new())
        );
        assert_eq!(
            PasswordUpdate::from_fields(legacy, None, true).unwrap(),
            PasswordUpdate::Clear
        );
    }
    assert_eq!(
        PasswordUpdate::from_fields(Some("p".into()), None, false).unwrap(),
        PasswordUpdate::Set("p".into())
    );
    assert!(PasswordUpdate::from_fields(None, Some(String::new()), true).is_err());
    assert!(PasswordUpdate::from_fields(Some("p".into()), Some(String::new()), false).is_err());
    assert!(PasswordUpdate::from_fields(Some("p".into()), None, true).is_err());
    assert!(!format!("{:?}", account("family")).contains("password-family"));
}

#[tokio::test]
async fn absent_username_has_database_enforced_uniqueness() {
    let db = db().await;
    let no_user = NewServer::new("music-player", "Peer", "http://peer");
    let a = upsert(&db, &no_user, NOW).await.unwrap();
    for username in ["", " \t\n", "　"] {
        let b = upsert(
            &db,
            &no_user
                .clone()
                .with_credentials(Some(username.into()), None),
            NOW,
        )
        .await
        .unwrap();
        assert_eq!(a.id, b.id);
    }
    // Bypassing the helper must still fail for SQL NULL versus empty username.
    assert!(db
        .execute_unprepared(
            "INSERT INTO saved_server(id,kind,name,url,username)
        VALUES ('bypass','music-player','Peer','http://peer','')"
        )
        .await
        .is_err());
    assert_eq!(list(&db).await.unwrap().len(), 1);
}

#[tokio::test]
async fn concurrent_saves_share_one_id_without_losing_password() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("accounts.db");
    let db = Database::connect(format!("sqlite://{}?mode=rwc", path.display()))
        .await
        .unwrap();
    Migrator::up(&db, None).await.unwrap();
    crate::enable_wal(&db).await;
    let mut tasks = Vec::new();
    for index in 0..12 {
        let db = db.clone();
        tasks.push(tokio::spawn(async move {
            let update = if index == 0 {
                PasswordUpdate::Set("chosen".into())
            } else {
                PasswordUpdate::Keep
            };
            upsert(&db, &account("family").with_password_update(update), NOW)
                .await
                .unwrap()
                .id
        }));
    }
    let mut ids = std::collections::HashSet::new();
    for task in tasks {
        ids.insert(task.await.unwrap());
    }
    assert_eq!(ids.len(), 1);
    let rows = list(&db).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].password.as_deref(), Some("chosen"));
}

#[tokio::test]
async fn legacy_import_is_atomic_non_overwriting_and_never_resurrects_accounts() {
    let db = db().await;
    let current = upsert(&db, &account("family"), NOW).await.unwrap();
    let legacy = vec![
        account("family").with_password_update(PasswordUpdate::Set("stale".into())),
        account("guest"),
        account("guest").with_password_update(PasswordUpdate::Set("lower-priority".into())),
    ];
    assert_eq!(
        import_legacy_accounts(&db, &legacy, LATER).await.unwrap(),
        1
    );
    assert!(legacy_imported(&db).await.unwrap());
    assert_eq!(get(&db, &current.id).await.unwrap(), Some(current.clone()));
    let guest = list(&db)
        .await
        .unwrap()
        .into_iter()
        .find(|row| row.username.as_deref() == Some("guest"))
        .unwrap();
    assert_eq!(guest.password.as_deref(), Some("password-guest"));
    delete(&db, &guest.id).await.unwrap();
    let mut edited = account("family").with_id(Some(current.id));
    edited.url = "http://moved".into();
    let edited = upsert(&db, &edited, LATER).await.unwrap();
    assert_eq!(
        import_legacy_accounts(&db, &legacy, LATER).await.unwrap(),
        0
    );
    assert_eq!(list(&db).await.unwrap(), vec![edited]);
}

#[tokio::test]
async fn failed_import_rolls_back_rows_and_marker_then_retries() {
    let db = db().await;
    db.execute_unprepared(
        "CREATE TRIGGER reject_guest BEFORE INSERT ON saved_server
        WHEN NEW.username = 'guest' BEGIN SELECT RAISE(ABORT,'fixture failure'); END",
    )
    .await
    .unwrap();
    let accounts = [account("family"), account("guest")];
    assert!(import_legacy_accounts(&db, &accounts, NOW).await.is_err());
    assert!(!legacy_imported(&db).await.unwrap());
    assert!(list(&db).await.unwrap().is_empty());
    db.execute_unprepared("DROP TRIGGER reject_guest")
        .await
        .unwrap();
    assert_eq!(
        import_legacy_accounts(&db, &accounts, NOW).await.unwrap(),
        2
    );
    assert!(legacy_imported(&db).await.unwrap());
}

#[test]
fn legacy_json_failure_preserves_input_and_empty_password_remains_compatible() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("desktop_servers.json");
    assert!(legacy_json_servers(Some(&path)).unwrap().is_empty());
    std::fs::write(&path, "{broken").unwrap();
    assert!(legacy_json_servers(Some(&path)).is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "{broken");
    std::fs::write(&path, r#"[{"kind":"jellyfin","name":"Home","url":"http://emby","username":"family","password":""}]"#).unwrap();
    let accounts = legacy_json_servers(Some(&path)).unwrap();
    assert_eq!(accounts[0].username.as_deref(), Some("family"));
    assert_eq!(accounts[0].password_update, PasswordUpdate::Keep);
}

fn identity(server: &str, user: &str) -> RemoteIdentity {
    RemoteIdentity {
        server_id: server.into(),
        user_id: user.into(),
    }
}

fn assert_identity(saved: &SavedServer, remote: &RemoteIdentity) {
    assert_eq!(
        saved.remote_server_id.as_deref(),
        Some(remote.server_id.as_str())
    );
    assert_eq!(
        saved.remote_user_id.as_deref(),
        Some(remote.user_id.as_str())
    );
}

const BIND_CONFLICT: &str =
    "saved account changed, was removed, or has a different remote identity";

#[tokio::test]
async fn identity_migration_preserves_legacy_rows_and_nullable_credentials() {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let previous_count = Migrator::migrations()
        .iter()
        .position(|migration| migration.name() == "m20260930_000002_source_identity")
        .unwrap();
    Migrator::up(&db, Some(previous_count as u32))
        .await
        .unwrap();
    let mut expected = Vec::new();
    for (id, username, password) in [
        ("legacy-null", None, None),
        ("legacy-empty", Some("family"), Some("")),
    ] {
        db.execute_raw(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "INSERT INTO saved_server(id,kind,name,url,username,password,created_at,updated_at)
            VALUES (?, 'emby', 'Old name', 'http://fixture', ?, ?, NULL, ?)",
            vec![
                id.into(),
                username.map(str::to_owned).into(),
                password.map(str::to_owned).into(),
                NOW.into(),
            ],
        ))
        .await
        .unwrap();
        expected.push(SavedServer {
            id: id.into(),
            kind: "emby".into(),
            name: "Old name".into(),
            url: "http://fixture".into(),
            username: username.map(str::to_owned),
            password: password.map(str::to_owned),
            updated_at: Some(NOW.into()),
            ..Default::default()
        });
    }
    Migrator::up(&db, None).await.unwrap();
    for row in &expected {
        assert_eq!(get(&db, &row.id).await.unwrap().as_ref(), Some(row));
    }
    let down_steps = (Migrator::migrations().len() - previous_count) as u32;
    Migrator::down(&db, Some(down_steps)).await.unwrap();
    let manager = migration::SchemaManager::new(&db);
    assert!(!manager
        .has_column("saved_server", "remote_server_id")
        .await
        .unwrap());
    assert!(!manager
        .has_column("saved_server", "remote_user_id")
        .await
        .unwrap());
    // Re-upgrade also proves that downgrade left old IDs/config/timestamps intact.
    Migrator::up(&db, None).await.unwrap();
    for row in &expected {
        assert_eq!(get(&db, &row.id).await.unwrap().as_ref(), Some(row));
        // NULL created_at and nullable authentication fields must be matchable.
        assert_identity(
            &bind_remote_identity(&db, row, &identity("server", &row.id))
                .await
                .unwrap(),
            &identity("server", &row.id),
        );
    }
}

#[tokio::test]
async fn binding_is_idempotent_but_never_changes_either_remote_id() {
    let db = db().await;
    let snapshot = upsert(&db, &account("family"), NOW).await.unwrap();
    let remote = identity("server-a", "user-a");
    let bound = bind_remote_identity(&db, &snapshot, &remote).await.unwrap();
    let mut expected = snapshot.clone();
    expected.remote_server_id = Some(remote.server_id.clone());
    expected.remote_user_id = Some(remote.user_id.clone());
    assert_eq!(bound, expected);
    // The original unbound auth snapshot and a freshly bound one both work.
    for auth_snapshot in [&snapshot, &bound] {
        assert_eq!(
            bind_remote_identity(&db, auth_snapshot, &remote)
                .await
                .unwrap(),
            bound
        );
        for other in [
            identity("server-b", "user-a"),
            identity("server-a", "user-b"),
        ] {
            assert!(bind_remote_identity(&db, auth_snapshot, &other)
                .await
                .is_err());
        }
    }
    assert_eq!(get(&db, &snapshot.id).await.unwrap(), Some(bound));
}

#[tokio::test]
async fn concurrent_remote_pairs_have_one_winner_and_same_pair_can_repeat() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("binding.db");
    let mut options = sea_orm::ConnectOptions::new(format!("sqlite://{}?mode=rwc", path.display()));
    // Separate single-connection pools exercise SQLite serialization, not one
    // pool's queue. Every connection gets a finite busy timeout.
    options
        .max_connections(1)
        .min_connections(1)
        .sqlx_logging(false);
    let db_a = Database::connect(options.clone()).await.unwrap();
    Migrator::up(&db_a, None).await.unwrap();
    let db_b = Database::connect(options).await.unwrap();
    for connection in [&db_a, &db_b] {
        connection
            .execute_unprepared("PRAGMA journal_mode=WAL")
            .await
            .unwrap();
        connection
            .execute_unprepared("PRAGMA busy_timeout=5000")
            .await
            .unwrap();
    }
    let snapshot = upsert(&db_a, &account("family"), NOW).await.unwrap();
    let a = identity("server-a", "user-a");
    let b = identity("server-b", "user-b");
    let (first, second) = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        tokio::join!(
            bind_remote_identity(&db_a, &snapshot, &a),
            bind_remote_identity(&db_b, &snapshot, &b),
        )
    })
    .await
    .unwrap();
    let (winner, loser, remote) = match (first, second) {
        (Ok(winner), Err(loser)) => (winner, loser, a),
        (Err(loser), Ok(winner)) => (winner, loser, b),
        _ => panic!("exactly one different remote pair must bind"),
    };
    // A lock/SQL error is not evidence of correctly resolving a binding race.
    assert_eq!(loser.to_string(), BIND_CONFLICT);
    assert_identity(&winner, &remote);
    assert_eq!(
        get(&db_a, &snapshot.id).await.unwrap(),
        Some(winner.clone())
    );
    let (first, second) = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        tokio::join!(
            bind_remote_identity(&db_a, &snapshot, &remote),
            bind_remote_identity(&db_b, &snapshot, &remote),
        )
    })
    .await
    .unwrap();
    assert_eq!(first.unwrap(), winner);
    assert_eq!(second.unwrap(), winner);
    db_a.close().await.unwrap();
    db_b.close().await.unwrap();
}

#[tokio::test]
async fn url_or_password_edit_during_authentication_rejects_old_snapshot() {
    for change_url in [true, false] {
        let db = db().await;
        let snapshot = upsert(&db, &account("family"), NOW).await.unwrap();
        let mut edit = account("family")
            .with_id(Some(snapshot.id.clone()))
            .with_password_update(PasswordUpdate::Keep);
        if change_url {
            edit.url = "http://moved".into();
        } else {
            edit.password_update = PasswordUpdate::Set("replacement-secret".into());
        }
        let edited = upsert(&db, &edit, LATER).await.unwrap();
        let remote = identity("server", "user");
        assert_eq!(
            bind_remote_identity(&db, &snapshot, &remote)
                .await
                .unwrap_err()
                .to_string(),
            BIND_CONFLICT,
        );
        assert_eq!(get(&db, &snapshot.id).await.unwrap(), Some(edited.clone()));
        assert_identity(
            &bind_remote_identity(&db, &edited, &remote).await.unwrap(),
            &remote,
        );
    }
}

#[tokio::test]
async fn name_only_edit_is_allowed_and_binding_returns_current_metadata() {
    let db = db().await;
    let snapshot = upsert(&db, &account("family"), NOW).await.unwrap();
    let mut edit = account("family")
        .with_id(Some(snapshot.id.clone()))
        .with_password_update(PasswordUpdate::Keep);
    edit.name = "Renamed during authentication".into();
    let mut expected = upsert(&db, &edit, LATER).await.unwrap();
    let remote = identity("server", "user");
    expected.remote_server_id = Some(remote.server_id.clone());
    expected.remote_user_id = Some(remote.user_id.clone());
    assert_eq!(
        bind_remote_identity(&db, &snapshot, &remote).await.unwrap(),
        expected
    );
    assert_eq!(expected.created_at.as_deref(), Some(NOW));
    assert_eq!(expected.updated_at.as_deref(), Some(LATER));
}

#[tokio::test]
async fn binding_distinguishes_null_and_empty_password_in_both_directions() {
    for initially_empty in [true, false] {
        let db = db().await;
        let input = NewServer::new("emby", "Anonymous", "http://fixture").with_password_update(
            if initially_empty {
                PasswordUpdate::Set(String::new())
            } else {
                PasswordUpdate::Clear
            },
        );
        let snapshot = upsert(&db, &input, NOW).await.unwrap();
        assert_eq!(snapshot.password.as_deref(), initially_empty.then_some(""));
        assert_eq!(snapshot.username, None);
        let edit = input
            .with_id(Some(snapshot.id.clone()))
            .with_password_update(if initially_empty {
                PasswordUpdate::Clear
            } else {
                PasswordUpdate::Set(String::new())
            });
        let edited = upsert(&db, &edit, LATER).await.unwrap();
        assert_eq!(edited.password.as_deref(), (!initially_empty).then_some(""));
        let remote = identity("server", "user");
        assert_eq!(
            bind_remote_identity(&db, &snapshot, &remote)
                .await
                .unwrap_err()
                .to_string(),
            BIND_CONFLICT
        );
        assert_eq!(get(&db, &snapshot.id).await.unwrap(), Some(edited.clone()));
        assert_identity(
            &bind_remote_identity(&db, &edited, &remote).await.unwrap(),
            &remote,
        );
    }
}

#[tokio::test]
async fn binding_checks_kind_username_and_creation_stamp_as_well() {
    // Ordinary edits forbid changing kind/username; raw updates exercise legacy
    // writers and nullable comparison without weakening that public API rule.
    for sql in [
        "UPDATE saved_server SET kind = 'emby' WHERE id = ?",
        "UPDATE saved_server SET username = NULL WHERE id = ?",
        "UPDATE saved_server SET username = '' WHERE id = ?",
        "UPDATE saved_server SET created_at = NULL WHERE id = ?",
        "UPDATE saved_server SET created_at = 'replacement' WHERE id = ?",
    ] {
        let db = db().await;
        let snapshot = upsert(&db, &account("family"), NOW).await.unwrap();
        db.execute_raw(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            sql,
            [snapshot.id.clone().into()],
        ))
        .await
        .unwrap();
        let edited = get(&db, &snapshot.id).await.unwrap().unwrap();
        let remote = identity("server", "user");
        assert_eq!(
            bind_remote_identity(&db, &snapshot, &remote)
                .await
                .unwrap_err()
                .to_string(),
            BIND_CONFLICT
        );
        assert_eq!(get(&db, &snapshot.id).await.unwrap(), Some(edited.clone()));
        assert_identity(
            &bind_remote_identity(&db, &edited, &remote).await.unwrap(),
            &remote,
        );
    }
    let db = db().await;
    let snapshot = upsert(
        &db,
        &NewServer::new("emby", "Anonymous", "http://fixture"),
        NOW,
    )
    .await
    .unwrap();
    db.execute_unprepared("UPDATE saved_server SET username = ''")
        .await
        .unwrap();
    // COALESCE would incorrectly treat the old NULL username as still current.
    assert_eq!(
        bind_remote_identity(&db, &snapshot, &identity("s", "u"))
            .await
            .unwrap_err()
            .to_string(),
        BIND_CONFLICT
    );
}

#[tokio::test]
async fn ordinary_edits_and_tuple_upserts_preserve_a_pinned_remote_pair() {
    let db = db().await;
    let snapshot = upsert(&db, &account("family"), NOW).await.unwrap();
    let remote = identity("server", "user");
    let bound = bind_remote_identity(&db, &snapshot, &remote).await.unwrap();
    let mut edit = account("family").with_id(Some(bound.id.clone()));
    edit.url = "http://new-address".into();
    edit.name = "New display name".into();
    for explicit_id in [true, false] {
        // Exercise both UPDATE by ID and INSERT ... ON CONFLICT.
        edit.id = explicit_id.then(|| bound.id.clone());
        for (update, password) in [
            (PasswordUpdate::Keep, Some("password-family")),
            (PasswordUpdate::Set(String::new()), Some("")),
            (PasswordUpdate::Clear, None),
            (
                PasswordUpdate::Set("password-family".into()),
                Some("password-family"),
            ),
        ] {
            let saved = upsert(&db, &edit.clone().with_password_update(update), LATER)
                .await
                .unwrap();
            assert_identity(&saved, &remote);
            assert_eq!(saved.id, bound.id);
            assert_eq!(saved.created_at, bound.created_at);
            assert_eq!(saved.name, edit.name);
            assert_eq!(saved.url, edit.url);
            assert_eq!(saved.password.as_deref(), password);
            // A previous binding never exempts authentication from the config check.
            assert_eq!(
                bind_remote_identity(&db, &bound, &remote)
                    .await
                    .unwrap_err()
                    .to_string(),
                BIND_CONFLICT
            );
            assert_eq!(
                bind_remote_identity(&db, &saved, &remote).await.unwrap(),
                saved
            );
            assert!(
                bind_remote_identity(&db, &saved, &identity("other", "user"))
                    .await
                    .is_err()
            );
        }
    }
    assert_eq!(list(&db).await.unwrap().len(), 1);
}

#[tokio::test]
async fn deleted_account_authentication_cannot_bind_a_replacement_account() {
    let db = db().await;
    let snapshot = upsert(&db, &account("family"), NOW).await.unwrap();
    let remote = identity("server", "user");
    assert!(delete(&db, &snapshot.id).await.unwrap());
    assert_eq!(
        bind_remote_identity(&db, &snapshot, &remote)
            .await
            .unwrap_err()
            .to_string(),
        BIND_CONFLICT
    );
    let replacement = upsert(&db, &account("family"), NOW).await.unwrap();
    assert_ne!(replacement.id, snapshot.id);
    assert_eq!(
        bind_remote_identity(&db, &snapshot, &remote)
            .await
            .unwrap_err()
            .to_string(),
        BIND_CONFLICT
    );
    assert_eq!(
        get(&db, &replacement.id).await.unwrap(),
        Some(replacement.clone())
    );
    assert_identity(
        &bind_remote_identity(&db, &replacement, &remote)
            .await
            .unwrap(),
        &remote,
    );
}

#[tokio::test]
async fn empty_or_partial_identities_are_rejected_without_overwriting_rows() {
    let db = db().await;
    let snapshot = upsert(&db, &account("family"), NOW).await.unwrap();
    for remote in [
        identity("", "user"),
        identity("server", ""),
        identity(" ", "user"),
        identity("server", "　"),
    ] {
        assert!(bind_remote_identity(&db, &snapshot, &remote).await.is_err());
        assert_eq!(
            get(&db, &snapshot.id).await.unwrap(),
            Some(snapshot.clone())
        );
    }
    let remote = identity("server", "user");
    for (server, user) in [(Some("server"), None), (None, Some("user"))] {
        let mut partial = snapshot.clone();
        partial.remote_server_id = server.map(str::to_owned);
        partial.remote_user_id = user.map(str::to_owned);
        assert!(bind_remote_identity(&db, &partial, &remote).await.is_err());
        db.execute_raw(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "UPDATE saved_server SET remote_server_id = ?, remote_user_id = ? WHERE id = ?",
            vec![
                partial.remote_server_id.clone().into(),
                partial.remote_user_id.clone().into(),
                snapshot.id.clone().into(),
            ],
        ))
        .await
        .unwrap();
        assert_eq!(
            bind_remote_identity(&db, &snapshot, &remote)
                .await
                .unwrap_err()
                .to_string(),
            BIND_CONFLICT
        );
        assert_eq!(get(&db, &snapshot.id).await.unwrap(), Some(partial));
    }
}

#[tokio::test]
async fn binding_database_errors_do_not_return_credential_diagnostics() {
    let db = db().await;
    let snapshot = upsert(&db, &account("family"), NOW).await.unwrap();
    // A deliberately credential-bearing database error must not escape through
    // anyhow's source chain or Debug, even if a backend embeds query parameters.
    db.execute_unprepared(
        "CREATE TRIGGER reject_binding BEFORE UPDATE ON saved_server
        BEGIN SELECT RAISE(ABORT, 'password-family'); END",
    )
    .await
    .unwrap();
    let error = bind_remote_identity(&db, &snapshot, &identity("server", "user"))
        .await
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "remote identity binding database operation failed"
    );
    assert!(!format!("{error:?}").contains("password-family"));
    assert_eq!(get(&db, &snapshot.id).await.unwrap(), Some(snapshot));
}
