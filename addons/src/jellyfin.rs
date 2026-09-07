//! Moved to `music-player-provider`.
//!
//! Re-exported here so the Slint desktop's browse stack keeps compiling until
//! it is removed; the [`Browsable`](crate::Browsable) blanket impl in `lib.rs`
//! adapts the new trait to the old one.

pub use music_player_provider::backends::jellyfin::Jellyfin;
