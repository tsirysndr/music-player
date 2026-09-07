//! Parse and compile errors, each pointing at the offending character.

use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

/// A filter that could not be understood, and where it went wrong.
///
/// The byte offset matters: these errors are shown under a filter box the user
/// is typing into, so "unknown field 'artistt' at 0" beats "invalid filter".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub message: String,
    /// Byte offset into the filter where the problem starts.
    pub at: usize,
}

impl Error {
    pub fn new(message: impl Into<String>, at: usize) -> Self {
        Self {
            message: message.into(),
            at,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (at {})", self.message, self.at)
    }
}

impl std::error::Error for Error {}
