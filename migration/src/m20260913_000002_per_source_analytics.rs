use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::ConnectionTrait;

/// Analytics per source — the local library and each remote server apart.
///
/// The counters and the listen log recorded remote plays under the remote
/// server's own track ids, which join nothing in the local `track` table, so
/// the views silently dropped them: the Most Played and Statistics screens
/// were local-only by accident, not by design.
///
/// Two additions fix that at the point of record:
///
/// * `source` — `'local'` for a file of ours, else the host the track streams
///   from (`navidrome.rocksky.app`). The host rather than the saved-server row
///   id, because a queued track outlives a server switch and its uri is the
///   one thing that still says where it came from.
/// * `title` / `artist` — snapshotted at record time, because for a remote
///   track there is no local row to join for a name. The views prefer the
///   local `track` row when there is one (it follows retags), and fall back
///   to the snapshot.
///
/// Rows that predate this migration keep `source = 'local'` — except the ones
/// whose track id matches nothing local: those were remote plays recorded
/// before their server was known, so they are marked `'unknown'` rather than
/// polluting the local numbers with blank-titled rows.
#[derive(DeriveMigrationName)]
pub struct Migration;

const UP: &str = r#"
ALTER TABLE play_history ADD COLUMN source TEXT NOT NULL DEFAULT 'local';
ALTER TABLE play_history ADD COLUMN title TEXT NOT NULL DEFAULT '';
ALTER TABLE play_history ADD COLUMN artist TEXT NOT NULL DEFAULT '';
ALTER TABLE track_stats ADD COLUMN source TEXT NOT NULL DEFAULT 'local';
ALTER TABLE track_stats ADD COLUMN title TEXT NOT NULL DEFAULT '';
ALTER TABLE track_stats ADD COLUMN artist TEXT NOT NULL DEFAULT '';

UPDATE play_history SET source = 'unknown'
WHERE track_id NOT IN (SELECT id FROM track);
UPDATE track_stats SET source = 'unknown'
WHERE track_id NOT IN (SELECT id FROM track);

CREATE INDEX IF NOT EXISTS play_history_source_idx ON play_history(source);
CREATE INDEX IF NOT EXISTS track_stats_source_idx ON track_stats(source);

DROP VIEW IF EXISTS v_most_played;
DROP VIEW IF EXISTS v_most_skipped;
DROP VIEW IF EXISTS v_recently_played;

CREATE VIEW v_most_played AS
SELECT s.track_id,
       COALESCE(t.title, s.title) AS title,
       COALESCE(t.artist, s.artist) AS artist,
       s.play_count, s.last_played, s.source
FROM track_stats s
LEFT JOIN track t ON t.id = s.track_id
WHERE s.play_count > 0
ORDER BY s.play_count DESC, s.last_played DESC;

CREATE VIEW v_most_skipped AS
SELECT s.track_id,
       COALESCE(t.title, s.title) AS title,
       COALESCE(t.artist, s.artist) AS artist,
       s.skip_count, s.last_skipped, s.source
FROM track_stats s
LEFT JOIN track t ON t.id = s.track_id
WHERE s.skip_count > 0
ORDER BY s.skip_count DESC, s.last_skipped DESC;

CREATE VIEW v_recently_played AS
SELECT h.id, h.track_id,
       COALESCE(t.title, h.title) AS title,
       COALESCE(t.artist, h.artist) AS artist,
       h.played_at, h.ms_played, h.length_ms, h.skipped, h.source
FROM play_history h
LEFT JOIN track t ON t.id = h.track_id
ORDER BY h.played_at DESC;
"#;

/// The pre-source view bodies, restored on the way down.
const DOWN: &str = r#"
DROP VIEW IF EXISTS v_recently_played;
DROP VIEW IF EXISTS v_most_skipped;
DROP VIEW IF EXISTS v_most_played;
DROP INDEX IF EXISTS track_stats_source_idx;
DROP INDEX IF EXISTS play_history_source_idx;
ALTER TABLE track_stats DROP COLUMN artist;
ALTER TABLE track_stats DROP COLUMN title;
ALTER TABLE track_stats DROP COLUMN source;
ALTER TABLE play_history DROP COLUMN artist;
ALTER TABLE play_history DROP COLUMN title;
ALTER TABLE play_history DROP COLUMN source;

CREATE VIEW v_most_played AS
SELECT t.id AS track_id, t.title, t.artist,
       COALESCE(s.play_count, 0) AS play_count, s.last_played
FROM track t
JOIN track_stats s ON s.track_id = t.id
WHERE COALESCE(s.play_count, 0) > 0
ORDER BY s.play_count DESC, s.last_played DESC;

CREATE VIEW v_most_skipped AS
SELECT t.id AS track_id, t.title, t.artist,
       COALESCE(s.skip_count, 0) AS skip_count, s.last_skipped
FROM track t
JOIN track_stats s ON s.track_id = t.id
WHERE COALESCE(s.skip_count, 0) > 0
ORDER BY s.skip_count DESC, s.last_skipped DESC;

CREATE VIEW v_recently_played AS
SELECT h.id, h.track_id, t.title, t.artist,
       h.played_at, h.ms_played, h.length_ms, h.skipped
FROM play_history h
JOIN track t ON t.id = h.track_id
ORDER BY h.played_at DESC;
"#;

async fn run(manager: &SchemaManager<'_>, script: &str) -> Result<(), DbErr> {
    let db = manager.get_connection();
    // One statement at a time: sqlite's execute takes a single statement,
    // and a failure should name the statement that caused it.
    for statement in script.split(';') {
        let statement = statement.trim();
        if statement.is_empty() {
            continue;
        }
        db.execute_unprepared(statement).await?;
    }
    Ok(())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        run(manager, UP).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        run(manager, DOWN).await
    }
}
