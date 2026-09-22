//! Bulk-loading listens that arrive one row at a time.
//!
//! The file-based imports hand DuckDB a whole export in one statement and it
//! does the rest. The Rocksky feed and the SQLite mirror cannot: they produce
//! rows in Rust, a page or a query at a time.
//!
//! Feeding those to `INSERT` individually is the obvious approach and the
//! wrong one — see [`crate::sql::SCHEMA`]'s note on `listens_staging`. This
//! wraps DuckDB's appender, which writes columnar batches directly, and
//! flushes into `listens` every [`BATCH`] rows.

use anyhow::Result;

use crate::Analytics;

/// Rows buffered before folding into `listens`.
///
/// Large enough that the per-batch statement is amortised, small enough that
/// an interrupted import still committed most of its work — the Rocksky feed
/// is walked newest-first, so a partial import is a useful prefix rather than
/// a wasted run.
pub const BATCH: usize = 2_000;

/// A listen on its way into the database.
#[derive(Debug, Default, Clone)]
pub struct Row {
    pub origin: String,
    pub origin_key: String,
    /// Unix seconds, or an RFC-3339 string — whichever the source speaks.
    pub played_at: When,
    pub track_id: Option<String>,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub ms_played: Option<i64>,
    pub length_ms: Option<i64>,
    pub skipped: Option<bool>,
    pub source: String,
}

/// Sources disagree about how to express a moment, and converting in Rust
/// would mean parsing timestamps DuckDB already parses better.
#[derive(Debug, Clone)]
pub enum When {
    Epoch(i64),
    Rfc3339(String),
}

impl Default for When {
    fn default() -> Self {
        When::Epoch(0)
    }
}

/// Buffers rows and flushes them in batches.
pub struct Staging<'a> {
    analytics: &'a Analytics,
    pending: Vec<Row>,
    /// Rows that reached `listens` and were not already there.
    pub inserted: u64,
}

impl<'a> Staging<'a> {
    pub fn new(analytics: &'a Analytics) -> Self {
        Self {
            analytics,
            pending: Vec::with_capacity(BATCH),
            inserted: 0,
        }
    }

    pub fn push(&mut self, row: Row) -> Result<()> {
        self.pending.push(row);
        if self.pending.len() >= BATCH {
            self.flush()?;
        }
        Ok(())
    }

    /// Write the buffer out and fold it into `listens`.
    pub fn flush(&mut self) -> Result<u64> {
        if self.pending.is_empty() {
            return Ok(0);
        }

        {
            let mut appender = self.analytics.conn().appender("listens_staging")?;
            for row in &self.pending {
                // The appender is typed by column position, so the order here
                // must match `listens_staging` in the schema.
                match &row.played_at {
                    When::Epoch(seconds) => appender.append_row(duckdb::params![
                        row.origin,
                        row.origin_key,
                        // The appender has no "call this function" form, so a
                        // timestamp has to arrive as a value DuckDB accepts
                        // for the column type. Microseconds since the epoch is
                        // that value for TIMESTAMPTZ.
                        duckdb::types::Value::Timestamp(
                            duckdb::types::TimeUnit::Microsecond,
                            seconds * 1_000_000,
                        ),
                        row.track_id,
                        row.title,
                        row.artist,
                        row.album,
                        row.ms_played,
                        row.length_ms,
                        row.skipped,
                        row.source,
                    ])?,
                    When::Rfc3339(text) => appender.append_row(duckdb::params![
                        row.origin,
                        row.origin_key,
                        text,
                        row.track_id,
                        row.title,
                        row.artist,
                        row.album,
                        row.ms_played,
                        row.length_ms,
                        row.skipped,
                        row.source,
                    ])?,
                }
            }
            appender.flush()?;
        }
        self.pending.clear();

        let added = self.analytics.flush_staging()?;
        self.inserted += added;
        Ok(added)
    }
}
