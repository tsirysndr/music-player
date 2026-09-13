use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::ConnectionTrait;

/// The listen log, and the analytics views over it.
///
/// `track_stats` keeps running counters — play_count, skip_count, last_played
/// — which is what smart playlists and rsql's `playcount` / `lastplayed`
/// fields read. It cannot answer anything about *when*: one row per track,
/// overwritten in place, so a play last night and a play last year are
/// indistinguishable.
///
/// `play_history` is the event log those counters are a summary of: one row
/// per listen, appended and never updated, so "what did I play this week" and
/// "what have I not touched since March" become ordinary queries. The counters
/// stay — they are the fast path.
///
/// The views live in SQL rather than Rust so every consumer — the gRPC and
/// GraphQL servers, and anyone with the sqlite3 CLI — sees one definition.
#[derive(DeriveMigrationName)]
pub struct Migration;

const UP: &str = r#"
CREATE TABLE IF NOT EXISTS play_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    track_id TEXT NOT NULL,
    played_at INTEGER NOT NULL,
    ms_played INTEGER NOT NULL DEFAULT 0,
    length_ms INTEGER NOT NULL DEFAULT 0,
    skipped INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS play_history_track_idx ON play_history(track_id);
CREATE INDEX IF NOT EXISTS play_history_played_at_idx ON play_history(played_at DESC);

CREATE VIEW IF NOT EXISTS v_most_played AS
SELECT t.id AS track_id, t.title, t.artist,
       COALESCE(s.play_count, 0) AS play_count, s.last_played
FROM track t
JOIN track_stats s ON s.track_id = t.id
WHERE COALESCE(s.play_count, 0) > 0
ORDER BY s.play_count DESC, s.last_played DESC;

CREATE VIEW IF NOT EXISTS v_most_skipped AS
SELECT t.id AS track_id, t.title, t.artist,
       COALESCE(s.skip_count, 0) AS skip_count, s.last_skipped
FROM track t
JOIN track_stats s ON s.track_id = t.id
WHERE COALESCE(s.skip_count, 0) > 0
ORDER BY s.skip_count DESC, s.last_skipped DESC;

-- No counter row, or one that has only ever recorded skips: a track skipped
-- ten times has still never been listened to.
CREATE VIEW IF NOT EXISTS v_never_played AS
SELECT t.id AS track_id, t.title, t.artist, t.created_at
FROM track t
LEFT JOIN track_stats s ON s.track_id = t.id
WHERE COALESCE(s.play_count, 0) = 0
ORDER BY t.created_at DESC;

CREATE VIEW IF NOT EXISTS v_recently_added AS
SELECT t.id AS track_id, t.title, t.artist, t.created_at
FROM track t
ORDER BY t.created_at DESC;

-- One row per listen, newest first — distinct from v_most_played in that a
-- track appears once per time it was played.
CREATE VIEW IF NOT EXISTS v_recently_played AS
SELECT h.id, h.track_id, t.title, t.artist,
       h.played_at, h.ms_played, h.length_ms, h.skipped
FROM play_history h
JOIN track t ON t.id = h.track_id
ORDER BY h.played_at DESC;
"#;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();
        // One statement at a time: sqlite's execute takes a single statement,
        // and a failure should name the statement that caused it.
        for statement in UP.split(';') {
            let statement = statement.trim();
            if statement.is_empty() {
                continue;
            }
            db.execute(sea_orm_migration::sea_orm::Statement::from_string(
                backend,
                statement.to_string(),
            ))
            .await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();
        for statement in [
            "DROP VIEW IF EXISTS v_recently_played",
            "DROP VIEW IF EXISTS v_recently_added",
            "DROP VIEW IF EXISTS v_never_played",
            "DROP VIEW IF EXISTS v_most_skipped",
            "DROP VIEW IF EXISTS v_most_played",
            "DROP INDEX IF EXISTS play_history_played_at_idx",
            "DROP INDEX IF EXISTS play_history_track_idx",
            "DROP TABLE IF EXISTS play_history",
        ] {
            db.execute(sea_orm_migration::sea_orm::Statement::from_string(
                backend,
                statement.to_string(),
            ))
            .await?;
        }
        Ok(())
    }
}
