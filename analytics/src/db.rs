//! Opening and migrating the analytics database.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use duckdb::Connection;

use crate::sql;

/// A handle on the DuckDB analytics database.
///
/// Not `Sync`: a DuckDB connection is single-threaded, like the rest of the
/// engine plumbing in this project. Callers that need it from async code
/// should own one per task, or put it behind a mutex and reach it through
/// `spawn_blocking` — queries here are CPU-bound and finish in milliseconds,
/// so blocking the async runtime on one is the thing to avoid.
pub struct Analytics {
    conn: Connection,
}

impl Analytics {
    /// Open (creating if absent) the analytics database beside the player's
    /// SQLite file.
    pub fn open_default() -> Result<Self> {
        Self::open(default_path()?)
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("could not create {}", parent.display()))?;
        }
        let conn = Connection::open(path).map_err(|cause| busy(path, cause))?;
        let this = Self { conn };
        this.migrate()?;
        Ok(this)
    }

    /// Open for querying only.
    ///
    /// DuckDB allows either one writer or any number of readers per file, so a
    /// report run while an import is in progress has to ask for read-only
    /// access or it is refused. Skips [`Self::migrate`] for the same reason —
    /// a reader cannot create anything — which means a read-only open of a
    /// database that has never been written fails, and should: there is
    /// nothing to report on.
    pub fn open_read_only(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            anyhow::bail!(
                "no analytics database yet — run `music-player analytics sync`, or import \
                 a history with `music-player analytics import <path>`"
            );
        }
        let config = duckdb::Config::default().access_mode(duckdb::AccessMode::ReadOnly)?;
        let conn = Connection::open_with_flags(path, config).map_err(|cause| busy(path, cause))?;
        Ok(Self { conn })
    }

    /// Read-only handle on the default database.
    pub fn open_default_read_only() -> Result<Self> {
        Self::open_read_only(default_path()?)
    }

    /// An ephemeral database. Used by the tests, and by any caller that wants
    /// to analyse an import without committing it.
    pub fn open_in_memory() -> Result<Self> {
        let this = Self {
            conn: Connection::open_in_memory()?,
        };
        this.migrate()?;
        Ok(this)
    }

    /// Bring the schema up to date. Idempotent, and run on every open —
    /// there is no version table because every statement in [`sql::SCHEMA`] is
    /// `CREATE ... IF NOT EXISTS` or `CREATE OR REPLACE`, and the database can
    /// be deleted and rebuilt from its sources at any time.
    pub fn migrate(&self) -> Result<()> {
        self.drop_legacy_objects();
        self.conn
            .execute_batch(sql::SCHEMA)
            .context("could not apply the analytics schema")?;
        // The view definitions are cheap to replace and must always match the
        // code that reads them, so they are rebuilt on every read-write open
        // rather than only after an import. Without this an upgraded release
        // reads last release's views and fails on a column it added.
        self.add_missing_columns();
        self.conn
            .execute_batch(sql::VIEWS)
            .context("could not define the analytics views")
    }

    /// Add columns a later release introduced on a table that already exists.
    ///
    /// `CREATE TABLE IF NOT EXISTS` leaves an existing table untouched, and
    /// DuckDB has no `ADD COLUMN IF NOT EXISTS`, so each statement here is
    /// expected to fail once its column is present. Failing is the normal
    /// path, not an error.
    fn add_missing_columns(&self) {
        for statement in [
            "ALTER TABLE song_match ADD COLUMN artist_picture VARCHAR",
            "ALTER TABLE tracks ADD COLUMN cover VARCHAR",
        ] {
            let _ = self.conn.execute_batch(statement);
        }
    }

    /// Recompute the detected per-origin clock offsets and the views that
    /// depend on them. Cheap, but not free, so it is called after an import
    /// rather than on every query.
    pub fn refresh_dedup(&self) -> Result<()> {
        self.conn
            .execute_batch(sql::DEDUP)
            .context("could not detect the per-source clock offsets")?;
        self.conn
            .execute_batch(sql::VIEWS)
            .context("could not define the analytics views")
    }

    /// Drop objects an older release created with a different *kind*.
    ///
    /// `canonical_listens` shipped as a view before it was materialised, and
    /// DuckDB refuses to replace a view with a table — or to `DROP VIEW IF
    /// EXISTS` something that is now a table, since `IF EXISTS` only forgives
    /// absence, not a type mismatch. Trying both and ignoring the failures is
    /// what upgrades an existing database in either direction.
    ///
    /// In reverse dependency order: a drop is refused while something still
    /// selects from it.
    fn drop_legacy_objects(&self) {
        for name in ["sessions", "session_listens", "enriched_listens"] {
            let _ = self
                .conn
                .execute_batch(&format!("DROP VIEW IF EXISTS {name}"));
        }
        // Only the view form is dropped: the table form is the current one and
        // holds the deduplicated corpus, which `refresh_dedup` rebuilds.
        let _ = self
            .conn
            .execute_batch("DROP VIEW IF EXISTS canonical_listens");
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Fold everything staged by [`crate::staging`] into `listens`, returning
    /// how many rows were genuinely new.
    ///
    /// One set-based statement per batch instead of one `INSERT` per row: see
    /// [`crate::sql::SCHEMA`] for why that distinction is worth this much
    /// ceremony on a columnar engine.
    pub fn flush_staging(&self) -> Result<u64> {
        let before = self.count()?;
        self.conn.execute_batch(
            "INSERT OR REPLACE INTO listens
               (origin, origin_key, played_at, track_id, title, artist, album,
                ms_played, length_ms, skipped, source, match_key)
             SELECT origin, origin_key, played_at, track_id, title, artist, album,
                    ms_played, length_ms, skipped, source, match_key(artist, title)
             FROM (
               -- A batch can legitimately carry the same key twice (a feed
               -- page that overlaps the previous one). `INSERT OR REPLACE`
               -- rejects a batch that conflicts with itself, so collapse
               -- duplicates before they reach the constraint.
               SELECT *, row_number() OVER (PARTITION BY origin, origin_key) AS n
               FROM listens_staging
             )
             WHERE n = 1 AND nullif(trim(title), '') IS NOT NULL;
             DELETE FROM listens_staging;",
        )?;
        Ok((self.count()? - before).max(0) as u64)
    }

    /// Total listens recorded, before deduplication.
    pub fn count(&self) -> Result<i64> {
        Ok(self
            .conn
            .query_row("SELECT count(*) FROM listens", [], |row| row.get(0))?)
    }
}

/// Turn DuckDB's lock error into something that says what to do about it.
///
/// DuckDB allows one writer or many readers per file, so "could not open" here
/// almost always means an import is running or the daemon has it open — which
/// the bare message gives no hint of.
fn busy(path: &Path, cause: duckdb::Error) -> anyhow::Error {
    let text = cause.to_string();
    if text.contains("lock") || text.contains("Conflicting") || text.contains("being used") {
        return anyhow::anyhow!(
            "{} is open in another process (an import, or the music-player daemon). \
             DuckDB allows a single writer at a time — wait for it to finish.",
            path.display()
        );
    }
    anyhow::Error::new(cause).context(format!("could not open {}", path.display()))
}

/// `analytics.duckdb`, next to the player's SQLite database.
///
/// Derived from the same settings the rest of the project uses, so the two
/// files always travel together — a user who copies their music-player
/// directory to a new machine keeps their history.
fn default_path() -> Result<PathBuf> {
    let config = music_player_settings::read_settings()
        .map_err(|e| anyhow::anyhow!("could not read settings: {e}"))?;
    let settings = config
        .try_deserialize::<music_player_settings::Settings>()
        .context("could not deserialize settings")?;

    // `database_url` is a sqlite URL such as `sqlite:/path/music-player.sqlite3?mode=rwc`.
    let raw = settings.database_url;
    let trimmed = raw
        .strip_prefix("sqlite://")
        .or_else(|| raw.strip_prefix("sqlite:"))
        .unwrap_or(&raw);
    let trimmed = trimmed.split('?').next().unwrap_or(trimmed);

    let sqlite_path = PathBuf::from(trimmed);
    let dir = sqlite_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    Ok(dir.join("analytics.duckdb"))
}
