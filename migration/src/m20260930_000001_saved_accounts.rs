use sea_orm_migration::{
    prelude::*,
    sea_orm::{DbBackend, Statement},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // Match Rust's input normalization, including Unicode whitespace. Keep
        // nonempty usernames verbatim because those bytes are used for login.
        let rows = db
            .query_all_raw(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT id, username FROM saved_server WHERE username IS NOT NULL",
            ))
            .await?;
        for row in rows {
            if row.try_get::<String>("", "username")?.trim().is_empty() {
                db.execute_raw(Statement::from_sql_and_values(
                    DbBackend::Sqlite,
                    "UPDATE saved_server SET username = NULL WHERE id = ?",
                    [row.try_get::<String>("", "id")?.into()],
                ))
                .await?;
            }
        }
        // Create the replacement first: failure never leaves uniqueness absent.
        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_saved_server_account
            ON saved_server(kind, url, COALESCE(username, ''))",
        )
        .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_saved_server_kind_url")
            .await?;
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS config_import (
            name TEXT NOT NULL PRIMARY KEY, completed_at TEXT NOT NULL)",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // Refuse a lossy downgrade when more than one account shares a URL.
        db.execute_unprepared(
            "CREATE UNIQUE INDEX idx_saved_server_kind_url
            ON saved_server(kind, url)",
        )
        .await?;
        db.execute_unprepared("DROP INDEX idx_saved_server_account")
            .await?;
        db.execute_unprepared("DROP TABLE config_import").await?;
        Ok(())
    }
}
