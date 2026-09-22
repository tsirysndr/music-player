//! The listening-analytics engine.
//!
//! Everything the player knows about *what was listened to* lives in SQLite:
//! `play_history` is the append-only listen log, `track_stats` its running
//! counters. That is the right home for it — it is written on every tick by
//! the player itself, and [`music_player_storage`] owns the schema.
//!
//! What SQLite is not good at is the other half: nine years of Spotify
//! streaming history arriving as a directory of JSON, a Last.fm CSV export,
//! and the questions that only make sense once those are combined —
//! sessionisation, transition graphs, feature drift over time. This crate is a
//! DuckDB **lens** over that data: it reads SQLite, never writes it.
//!
//! The invariant worth stating plainly, because breaking it would be subtle
//! and painful: `migration::apply` remains the single owner of the SQLite
//! schema, and DuckDB is never a second write path for anything the player
//! itself records. If the DuckDB file is deleted it rebuilds from SQLite plus
//! the import sources; nothing is lost that did not come from outside.

pub mod artwork;
pub mod db;
pub mod enrich;
pub mod import;
pub mod progress;
pub mod query;
pub mod sql;
pub mod staging;
pub mod sync;

/// Re-exported so callers can render arbitrary query results (the `analytics
/// query` escape hatch) without taking their own DuckDB dependency, which
/// would have to be kept in version lockstep with this one.
pub use duckdb;

pub use db::Analytics;
pub use enrich::EnrichReport;
pub use import::{Format, ImportReport};
pub use progress::Reporter;
pub use query::Window;
pub use sync::SyncReport;

#[cfg(test)]
mod tests;
