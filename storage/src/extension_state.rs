//! Which extensions the user has switched off.
//!
//! [`Registry::load_all`](music_player_extensions::Registry::load_all) and
//! `catalog::installed` both take a map of id → enabled; this is where that map
//! is kept between runs, in the `extension` table.
//!
//! Only what the user has actually changed is stored. An extension with no row
//! is enabled — the same default the registry applies — so dropping a new
//! extension in works without a second step, and removing one leaves behind a
//! row that simply never matches anything again.
//!
//! It lives in `storage` rather than in the `extensions` crate on purpose: the
//! extensions crate has no database dependency and should not grow one, and
//! every client that needs this (the daemon, the GraphQL layer, the CLI, the
//! Slint desktop) already talks to storage.

use std::collections::BTreeMap;

use anyhow::Error;
use music_player_entity::extension;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

/// Every extension the user has an opinion about, as id → enabled.
///
/// A read failure is an empty map with a warning rather than an error: not
/// being able to read a preference should leave the extensions working, not
/// take the list down with it.
pub async fn enabled_map(db: &DatabaseConnection) -> BTreeMap<String, bool> {
    match extension::Entity::find().all(db).await {
        Ok(rows) => rows.into_iter().map(|row| (row.id, row.enabled)).collect(),
        Err(e) => {
            tracing::warn!("could not read the extension state: {e}");
            BTreeMap::new()
        }
    }
}

/// Whether `id` is enabled. Anything with no row is.
pub async fn is_enabled(db: &DatabaseConnection, id: &str) -> bool {
    match extension::Entity::find_by_id(id.to_owned()).one(db).await {
        Ok(Some(row)) => row.enabled,
        Ok(None) => true,
        Err(e) => {
            tracing::warn!(extension = %id, "could not read the extension state: {e}");
            true
        }
    }
}

/// Switch `id` on or off.
///
/// Enabling deletes the row rather than storing `true`: enabled is the
/// default, so an explicit `true` is a row that outlives the extension without
/// ever meaning anything.
pub async fn set_enabled(
    db: &DatabaseConnection,
    id: &str,
    enabled: bool,
    now: &str,
) -> Result<(), Error> {
    if enabled {
        extension::Entity::delete_many()
            .filter(extension::Column::Id.eq(id))
            .exec(db)
            .await?;
        return Ok(());
    }

    let row = extension::ActiveModel {
        id: Set(id.to_owned()),
        enabled: Set(false),
        updated_at: Set(Some(now.to_owned())),
    };

    // `on_conflict` rather than find-then-insert: two clients toggling the
    // same extension at once would otherwise race into a primary-key error.
    extension::Entity::insert(row)
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(extension::Column::Id)
                .update_columns([extension::Column::Enabled, extension::Column::UpdatedAt])
                .to_owned(),
        )
        .exec(db)
        .await?;
    Ok(())
}

/// Forget every stored preference, putting all extensions back to enabled.
pub async fn clear(db: &DatabaseConnection) -> Result<(), Error> {
    extension::Entity::delete_many().exec(db).await?;
    Ok(())
}

/// Drop the rows for extensions that are no longer installed.
///
/// A row for an extension that has been deleted is harmless — it matches
/// nothing — but it would silently switch the extension back off if the user
/// ever reinstalled it, which is not what removing it meant.
pub async fn prune(db: &DatabaseConnection, installed: &[String]) -> Result<(), Error> {
    let rows = extension::Entity::find().all(db).await?;
    let stale: Vec<String> = rows
        .into_iter()
        .map(|row| row.id)
        .filter(|id| !installed.iter().any(|kept| kept == id))
        .collect();
    if stale.is_empty() {
        return Ok(());
    }
    extension::Entity::delete_many()
        .filter(extension::Column::Id.is_in(stale))
        .exec(db)
        .await?;
    Ok(())
}

/// `ActiveValue` is imported for the `Set` alias only; keeping the import
/// referenced here documents that rather than letting it read as dead.
const _: fn() -> ActiveValue<String> = || Set(String::new());

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database, DbBackend, Schema};

    /// An in-memory database with just the extension table.
    async fn db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let schema = Schema::new(DbBackend::Sqlite);
        db.execute(
            db.get_database_backend()
                .build(&schema.create_table_from_entity(extension::Entity)),
        )
        .await
        .unwrap();
        db
    }

    const NOW: &str = "2026-09-07T16:00:00Z";

    #[tokio::test]
    async fn everything_is_enabled_before_anything_is_stored() {
        let db = db().await;
        assert!(enabled_map(&db).await.is_empty());
        assert!(is_enabled(&db, "com.example.anything").await);
    }

    #[tokio::test]
    async fn disabling_persists() {
        let db = db().await;
        set_enabled(&db, "com.example.off", false, NOW)
            .await
            .unwrap();

        assert!(!is_enabled(&db, "com.example.off").await);
        // Everything else is untouched.
        assert!(is_enabled(&db, "com.example.other").await);
        assert_eq!(enabled_map(&db).await.get("com.example.off"), Some(&false));
    }

    /// Enabling deletes the row rather than storing `true`, so the table does
    /// not accumulate defaults for extensions that were toggled twice.
    #[tokio::test]
    async fn re_enabling_removes_the_row() {
        let db = db().await;
        set_enabled(&db, "com.example.x", false, NOW).await.unwrap();
        assert_eq!(enabled_map(&db).await.len(), 1);

        set_enabled(&db, "com.example.x", true, NOW).await.unwrap();
        assert!(enabled_map(&db).await.is_empty());
        assert!(is_enabled(&db, "com.example.x").await);
    }

    /// Disabling something already disabled must update rather than collide on
    /// the primary key — two clients can toggle the same extension at once.
    #[tokio::test]
    async fn disabling_twice_is_not_an_error() {
        let db = db().await;
        set_enabled(&db, "com.example.x", false, NOW).await.unwrap();
        set_enabled(&db, "com.example.x", false, "later")
            .await
            .unwrap();

        assert_eq!(enabled_map(&db).await.len(), 1);
        assert!(!is_enabled(&db, "com.example.x").await);
    }

    #[tokio::test]
    async fn several_extensions_are_tracked_independently() {
        let db = db().await;
        set_enabled(&db, "a", false, NOW).await.unwrap();
        set_enabled(&db, "b", false, NOW).await.unwrap();
        set_enabled(&db, "a", true, NOW).await.unwrap();

        assert!(is_enabled(&db, "a").await);
        assert!(!is_enabled(&db, "b").await);
    }

    #[tokio::test]
    async fn clearing_puts_everything_back_to_enabled() {
        let db = db().await;
        set_enabled(&db, "a", false, NOW).await.unwrap();
        set_enabled(&db, "b", false, NOW).await.unwrap();

        clear(&db).await.unwrap();
        assert!(enabled_map(&db).await.is_empty());
    }

    /// A row for an uninstalled extension would switch it back off if it were
    /// ever reinstalled, which is not what deleting it meant.
    #[tokio::test]
    async fn pruning_drops_rows_for_uninstalled_extensions() {
        let db = db().await;
        set_enabled(&db, "still.here", false, NOW).await.unwrap();
        set_enabled(&db, "long.gone", false, NOW).await.unwrap();

        prune(&db, &["still.here".to_string()]).await.unwrap();

        let map = enabled_map(&db).await;
        assert_eq!(map.get("still.here"), Some(&false));
        assert!(!map.contains_key("long.gone"));
    }

    #[tokio::test]
    async fn pruning_with_nothing_stale_is_a_no_op() {
        let db = db().await;
        set_enabled(&db, "a", false, NOW).await.unwrap();

        prune(&db, &["a".to_string(), "b".to_string()])
            .await
            .unwrap();
        assert_eq!(enabled_map(&db).await.len(), 1);
    }
}
