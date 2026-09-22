//! Importing listening history from outside the player.
//!
//! Three sources, one destination. The player's own listen log only goes back
//! as far as the player has been installed; a Spotify export goes back to
//! whenever the account was opened, which is where the interesting questions
//! live ("what was I listening to five summers ago", "when did I stop playing
//! this").
//!
//! Everything lands in `listens` keyed by `(origin, origin_key)`, so importing
//! the same export twice is a no-op rather than a doubling. Cross-origin
//! duplicates — the same listen arriving from both Spotify and Last.fm — are
//! *not* removed here; they are resolved by [`crate::sql::DEDUP`], which can
//! see all the origins at once and work out how their clocks differ.

pub mod lastfm;
pub mod rocksky;
pub mod spotify;

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use anyhow::{bail, Result};

/// What an import did.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ImportReport {
    pub origin: String,
    /// Files actually read.
    pub files: Vec<String>,
    /// Records present in the source.
    pub read: u64,
    /// Rows now in `listens` that were not there before.
    pub inserted: u64,
    /// Records deliberately not imported, by reason. Reported rather than
    /// silently dropped: "29,224 records, 28,744 imported" invites the
    /// question of where the other 480 went, and the answer is usually
    /// "podcasts", not "a bug".
    pub skipped: BTreeMap<String, u64>,
}

impl ImportReport {
    pub fn skip(&mut self, reason: &str, n: u64) {
        if n > 0 {
            *self.skipped.entry(reason.to_owned()).or_default() += n;
        }
    }

    pub fn total_skipped(&self) -> u64 {
        self.skipped.values().sum()
    }
}

/// The export formats that can be read from disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Spotify "Extended Streaming History" — a directory of
    /// `Streaming_History_Audio_*.json`.
    Spotify,
    /// A Last.fm export, CSV or JSON.
    Lastfm,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Format::Spotify => "spotify",
            Format::Lastfm => "lastfm",
        })
    }
}

impl std::str::FromStr for Format {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "spotify" => Ok(Format::Spotify),
            "lastfm" | "last.fm" => Ok(Format::Lastfm),
            other => bail!("unknown import format '{other}' (expected spotify or lastfm)"),
        }
    }
}

/// Work out what `path` is, from its shape and its first few kilobytes.
///
/// Mirrors the detection in the Rocksky CLI's importer, so the same export is
/// recognised the same way by both tools.
pub fn detect(path: &Path) -> Result<Format> {
    if path.is_dir() {
        // A directory is only ever a Spotify Extended Streaming History export.
        return Ok(Format::Spotify);
    }

    let head = read_head(path, 64 * 1024)?;
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if extension == "json"
        || head.trim_start().starts_with('[')
        || head.trim_start().starts_with('{')
    {
        // Spotify records are unmistakable.
        if head.contains("master_metadata_track_name")
            || head.contains("spotify_track_uri")
            || (head.contains("\"ts\"") && head.contains("\"ms_played\""))
        {
            return Ok(Format::Spotify);
        }
        return Ok(Format::Lastfm);
    }

    let header = head.lines().next().unwrap_or_default().to_ascii_lowercase();
    if header.contains("uts") || (header.contains("track") && header.contains("artist")) {
        return Ok(Format::Lastfm);
    }
    if header.contains("master_metadata_track_name") || header.contains("spotify_track_uri") {
        return Ok(Format::Spotify);
    }
    bail!(
        "could not tell what kind of export {} is — pass --format spotify or --format lastfm",
        path.display()
    )
}

fn read_head(path: &Path, bytes: usize) -> Result<String> {
    use std::io::Read;

    let mut file = std::fs::File::open(path)?;
    let mut buffer = vec![0u8; bytes];
    let read = file.read(&mut buffer)?;
    buffer.truncate(read);
    // An export is UTF-8, but a truncated read can land mid-codepoint.
    Ok(String::from_utf8_lossy(&buffer).into_owned())
}

/// Quote a path for embedding in SQL.
///
/// The import paths come from the command line and go into `read_json_auto('…')`
/// because DuckDB's file-reading functions take a literal, not a bind
/// parameter. Doubling single quotes is what keeps a filename containing one
/// from ending the string early.
pub(crate) fn sql_literal(path: &Path) -> String {
    path.to_string_lossy().replace('\'', "''")
}
