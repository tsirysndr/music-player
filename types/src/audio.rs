//! Durable audio choices and metadata candidates. No transport credentials or
//! playback session are represented here. A candidate is not an accepted pin.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioSelection {
    #[default]
    Auto,
    Explicit {
        media_source_id: String,
        audio_stream_index: i32,
    },
    Pinned(AudioPin),
}

/// Accepted choice for one queue occurrence. These fields detect known
/// metadata changes, not equality of the underlying media bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioPin {
    pub media_source_id: String,
    pub audio_stream_index: i32,
    pub runtime_ticks: Option<u64>,
    pub etag: Option<String>,
    pub codec: Option<String>,
    pub channels: Option<u32>,
    pub sample_rate: Option<u32>,
}

/// Metadata read under the account named in `source` (a stable SourceRef handle).
/// Loading still resolves and validates the chosen source/stream afresh.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioOptions {
    pub source: String,
    pub versions: Vec<AudioVersionOption>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioVersionOption {
    pub id: String,
    pub name: Option<String>,
    pub runtime_ticks: Option<u64>,
    /// The server's actual index, including zero, without array renumbering.
    pub default_audio_stream_index: Option<i32>,
    /// None means this version can be selected subject to stream availability.
    pub unavailable_reason: Option<String>,
    pub audio_streams: Vec<AudioStreamOption>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioStreamOption {
    pub index: i32,
    pub title: Option<String>,
    pub display_title: Option<String>,
    pub language: Option<String>,
    pub codec: Option<String>,
    pub channels: Option<u32>,
    pub sample_rate: Option<u32>,
    /// A per-stream flag; default_audio_stream_index takes precedence.
    pub is_default: bool,
    pub unavailable_reason: Option<String>,
}
