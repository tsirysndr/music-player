//! Progress reporting for long imports.
//!
//! An import can read nine years of JSON or walk thousands of API pages, and a
//! command that prints nothing for two minutes is indistinguishable from one
//! that has hung. The engine reports what it is doing through this trait; the
//! CLI renders bars, the daemon logs, the tests ignore it.
//!
//! Deliberately tiny and free of any progress-bar dependency, so that this
//! crate stays a library and the choice of renderer belongs to the caller.

/// Something that wants to be told how an import is going.
pub trait Reporter: Send + Sync {
    /// A new phase began — "reading Streaming_History_Audio_2025.json",
    /// "resolving metadata". `total` is the number of units expected when it
    /// is known ahead of time.
    fn stage(&self, name: &str, total: Option<u64>);
    /// `n` more units of the current stage are done.
    fn advance(&self, n: u64);
    /// The current stage finished, with a one-line summary.
    fn finish(&self, summary: &str);
    /// Something went wrong but the import carried on — a file that could not
    /// be parsed, a track the catalogue did not recognise.
    fn warn(&self, message: &str);
}

/// The default: report nothing. Used by the tests and by any caller that just
/// wants the result.
pub struct Silent;

impl Reporter for Silent {
    fn stage(&self, _name: &str, _total: Option<u64>) {}
    fn advance(&self, _n: u64) {}
    fn finish(&self, _summary: &str) {}
    fn warn(&self, _message: &str) {}
}

/// A reporter that forwards to `tracing`, for the daemon and for background
/// imports where there is no terminal to draw on.
pub struct Tracing;

impl Reporter for Tracing {
    fn stage(&self, name: &str, total: Option<u64>) {
        match total {
            Some(total) => tracing::info!(total, "{name}"),
            None => tracing::info!("{name}"),
        }
    }
    fn advance(&self, _n: u64) {}
    fn finish(&self, summary: &str) {
        tracing::info!("{summary}");
    }
    fn warn(&self, message: &str) {
        tracing::warn!("{message}");
    }
}
