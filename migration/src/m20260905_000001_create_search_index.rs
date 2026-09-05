use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, Statement},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

const UP_STATEMENTS: [&str; 15] = [
    // FTS5 virtual tables
    "CREATE VIRTUAL TABLE IF NOT EXISTS track_search USING fts5(id UNINDEXED, title, artist, album, genre)",
    "CREATE VIRTUAL TABLE IF NOT EXISTS album_search USING fts5(id UNINDEXED, title, artist)",
    "CREATE VIRTUAL TABLE IF NOT EXISTS artist_search USING fts5(id UNINDEXED, name)",
    // Triggers keeping track_search in sync with track
    r#"CREATE TRIGGER IF NOT EXISTS track_search_ai AFTER INSERT ON track BEGIN
        DELETE FROM track_search WHERE id = NEW.id;
        INSERT INTO track_search(id, title, artist, album, genre)
        VALUES (NEW.id, NEW.title, NEW.artist, (SELECT title FROM album WHERE id = NEW.album_id), NEW.genre);
    END"#,
    r#"CREATE TRIGGER IF NOT EXISTS track_search_au AFTER UPDATE ON track BEGIN
        DELETE FROM track_search WHERE id = OLD.id;
        DELETE FROM track_search WHERE id = NEW.id;
        INSERT INTO track_search(id, title, artist, album, genre)
        VALUES (NEW.id, NEW.title, NEW.artist, (SELECT title FROM album WHERE id = NEW.album_id), NEW.genre);
    END"#,
    r#"CREATE TRIGGER IF NOT EXISTS track_search_ad AFTER DELETE ON track BEGIN
        DELETE FROM track_search WHERE id = OLD.id;
    END"#,
    // Triggers keeping album_search in sync with album
    r#"CREATE TRIGGER IF NOT EXISTS album_search_ai AFTER INSERT ON album BEGIN
        DELETE FROM album_search WHERE id = NEW.id;
        INSERT INTO album_search(id, title, artist) VALUES (NEW.id, NEW.title, NEW.artist);
    END"#,
    r#"CREATE TRIGGER IF NOT EXISTS album_search_au AFTER UPDATE ON album BEGIN
        DELETE FROM album_search WHERE id = OLD.id;
        DELETE FROM album_search WHERE id = NEW.id;
        INSERT INTO album_search(id, title, artist) VALUES (NEW.id, NEW.title, NEW.artist);
    END"#,
    r#"CREATE TRIGGER IF NOT EXISTS album_search_ad AFTER DELETE ON album BEGIN
        DELETE FROM album_search WHERE id = OLD.id;
    END"#,
    // Triggers keeping artist_search in sync with artist
    r#"CREATE TRIGGER IF NOT EXISTS artist_search_ai AFTER INSERT ON artist BEGIN
        DELETE FROM artist_search WHERE id = NEW.id;
        INSERT INTO artist_search(id, name) VALUES (NEW.id, NEW.name);
    END"#,
    r#"CREATE TRIGGER IF NOT EXISTS artist_search_au AFTER UPDATE ON artist BEGIN
        DELETE FROM artist_search WHERE id = OLD.id;
        DELETE FROM artist_search WHERE id = NEW.id;
        INSERT INTO artist_search(id, name) VALUES (NEW.id, NEW.name);
    END"#,
    r#"CREATE TRIGGER IF NOT EXISTS artist_search_ad AFTER DELETE ON artist BEGIN
        DELETE FROM artist_search WHERE id = OLD.id;
    END"#,
    // Backfill existing rows (clear first to guard against duplicates)
    "DELETE FROM track_search",
    "DELETE FROM album_search",
    "DELETE FROM artist_search",
];

const BACKFILL_STATEMENTS: [&str; 3] = [
    r#"INSERT INTO track_search(id, title, artist, album, genre)
        SELECT t.id, t.title, t.artist, (SELECT title FROM album WHERE id = t.album_id), t.genre FROM track t"#,
    "INSERT INTO album_search(id, title, artist) SELECT id, title, artist FROM album",
    "INSERT INTO artist_search(id, name) SELECT id, name FROM artist",
];

const DOWN_STATEMENTS: [&str; 12] = [
    "DROP TRIGGER IF EXISTS track_search_ai",
    "DROP TRIGGER IF EXISTS track_search_au",
    "DROP TRIGGER IF EXISTS track_search_ad",
    "DROP TRIGGER IF EXISTS album_search_ai",
    "DROP TRIGGER IF EXISTS album_search_au",
    "DROP TRIGGER IF EXISTS album_search_ad",
    "DROP TRIGGER IF EXISTS artist_search_ai",
    "DROP TRIGGER IF EXISTS artist_search_au",
    "DROP TRIGGER IF EXISTS artist_search_ad",
    "DROP TABLE IF EXISTS track_search",
    "DROP TABLE IF EXISTS album_search",
    "DROP TABLE IF EXISTS artist_search",
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        let backend = manager.get_database_backend();
        for sql in UP_STATEMENTS.iter().chain(BACKFILL_STATEMENTS.iter()) {
            conn.execute(Statement::from_string(backend, sql.to_string()))
                .await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        let backend = manager.get_database_backend();
        for sql in DOWN_STATEMENTS.iter() {
            conn.execute(Statement::from_string(backend, sql.to_string()))
                .await?;
        }
        Ok(())
    }
}
