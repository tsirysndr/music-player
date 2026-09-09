#[cfg(test)]
mod tests;

use music_player_entity::{album as album_entity, artist as artist_entity, track as track_entity};
use music_player_settings::{read_settings, Settings};
use sea_orm::{sea_query, ConnectOptions, ConnectionTrait, DatabaseConnection};

pub mod account;
pub mod atproto;
pub mod atradio;
pub mod auto_dj;
pub mod extension_state;
pub mod radio_resolve;
pub mod radio_stream;
pub mod repo_sync;
pub mod rocksky;
pub mod rocksky_likes;
pub mod saved_servers;
pub mod searcher;
pub mod smart_playlist;
pub mod track_analysis;
pub mod track_cache;

pub mod repo;

#[derive(Clone)]
pub struct Database {
    pub connection: DatabaseConnection,
}

/// The process-wide database handle.
///
/// Put sqlite into write-ahead logging.
///
/// In the default rollback-journal mode a writer takes an exclusive lock on the
/// whole database, so *every read blocks* until it finishes. That is fine for a
/// library that is written once a scan, and not at all fine now: the background
/// analysis writes a row per track for as long as it runs, and every screen in
/// every client sat waiting behind it. Under WAL, readers carry on against the
/// last committed state while a writer appends.
///
/// Persistent — the mode is stored in the database file, so this is really
/// "set it if it has not been set". Best-effort: an older sqlite, or a database
/// on a filesystem without proper locking, keeps the old mode and works as
/// before rather than failing to open.
async fn enable_wal(connection: &DatabaseConnection) {
    use sea_orm::Statement;

    let backend = connection.get_database_backend();
    if backend != sea_orm::DatabaseBackend::Sqlite {
        return;
    }

    for pragma in [
        "PRAGMA journal_mode=WAL",
        // With WAL a reader never blocks, but two writers still queue. Five
        // seconds of waiting beats returning "database is locked" to a screen.
        "PRAGMA busy_timeout=5000",
        // WAL's durability trade: a commit no longer waits for the disk to
        // confirm. The risk is losing the last few writes in a power cut, and
        // what is at stake is a play count and a cached tempo.
        "PRAGMA synchronous=NORMAL",
    ] {
        if let Err(cause) = connection
            .execute(Statement::from_string(backend, pragma.to_owned()))
            .await
        {
            tracing::debug!(%pragma, %cause, "could not set");
        }
    }
}

/// Every `Database::new()` opens its own sqlite pool, and a pool costs one file
/// descriptor per connection — so calling it on a timer exhausts the (low)
/// descriptor limit of a desktop app bundle and takes the whole process down
/// with "Too many open files". Anything that needs the database outside of
/// startup should go through here.
pub async fn shared() -> &'static Database {
    static DB: tokio::sync::OnceCell<Database> = tokio::sync::OnceCell::const_new();
    DB.get_or_init(|| async { Database::new().await }).await
}

impl Database {
    /// Open a **new** connection pool.
    ///
    /// Prefer [`shared`]. Each call opens its own pool, and every pooled
    /// connection costs a file descriptor, so calling this repeatedly — on a
    /// poll tick, per request, inside a loop — runs the process out of
    /// descriptors and kills it with "Too many open files". Reserve it for
    /// process startup and for tests that want an isolated database.
    pub async fn new() -> Database {
        let config = read_settings().unwrap();
        let settings = config.try_deserialize::<Settings>().unwrap();

        let mut opt = ConnectOptions::new(settings.database_url);
        // SQLite uses one file descriptor per pooled connection.  The daemon
        // exposes several services in the same process, so eagerly opening
        // five connections for every pool can exhaust the low descriptor
        // limit used by desktop app bundles.  SQLite serializes writes
        // anyway; a small, lazily-grown pool is both sufficient and safer.
        opt.max_connections(8).min_connections(1);
        let connection = sea_orm::Database::connect(opt)
            .await
            .expect("Could not connect to database");

        enable_wal(&connection).await;

        Database { connection }
    }

    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    pub async fn create_indexes(&self) {
        let builder = self.connection.get_database_backend();
        let track_title_idx = sea_query::Index::create()
            .table(track_entity::Entity)
            .name("track_title_index")
            .col(track_entity::Column::Title)
            .to_owned();
        let sql = builder.build(&track_title_idx);
        match self.connection.execute(sql).await {
            Ok(_) => {}
            Err(_) => {
                println!("track_title_index already exists, skipping");
            }
        }

        let album_title_idx = sea_query::Index::create()
            .table(album_entity::Entity)
            .name("album_title_index")
            .col(album_entity::Column::Title)
            .to_owned();
        let sql = builder.build(&album_title_idx);
        match self.connection.execute(sql).await {
            Ok(_) => {}
            Err(_) => {
                println!("album_title_index already exists, skipping");
            }
        }

        let artist_name_idx = sea_query::Index::create()
            .table(artist_entity::Entity)
            .name("artist_name_index")
            .col(artist_entity::Column::Name)
            .to_owned();
        let sql = builder.build(&artist_name_idx);
        match self.connection.execute(sql).await {
            Ok(_) => {}
            Err(_) => {
                println!("artist_name_index already exists, skipping");
            }
        }
    }
}
