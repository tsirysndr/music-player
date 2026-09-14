//! Per-server library snapshots, so reconnecting shows the library instantly.
//!
//! A remote library is downloaded whole on every connect — pages of albums,
//! artists and tracks, each a network round trip — and until that finishes
//! the screens show a loading skeleton. The listing rarely changes between
//! sessions, so the previous session's answer is worth showing immediately
//! while the fresh one is fetched; when it lands it replaces the snapshot on
//! screen and on disk.
//!
//! Keyed by the server's host — the same key the analytics use — and stored
//! as prost length-delimited messages: the listing already *is* three vectors
//! of protos, and re-encoding what the wire delivered beats inventing a
//! parallel serde schema that would drift from the .proto.

use prost::Message;
use std::path::PathBuf;

use crate::rpc::{AlbumProto, ArtistProto, TrackProto};

/// Bumped when the layout changes; a mismatched file is ignored, not migrated
/// — it is a cache, and the network refresh rebuilds it.
const VERSION: u8 = 1;

pub struct Snapshot {
    pub albums: Vec<AlbumProto>,
    pub artists: Vec<ArtistProto>,
    pub tracks: Vec<TrackProto>,
}

fn path(host: &str) -> Option<PathBuf> {
    // The host names a directory entry, so anything path-hostile is mapped
    // away. Collisions after mapping are theoretical (two hosts differing
    // only in a path-hostile character) and cost a shared cache slot, not
    // correctness — the refresh overwrites.
    let safe: String = host
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if safe.is_empty() {
        return None;
    }
    dirs::config_dir().map(|dir| {
        dir.join("music-player")
            .join("library-snapshots")
            .join(format!("{safe}.pb"))
    })
}

fn put_section<T: Message>(out: &mut Vec<u8>, items: &[T]) {
    out.extend_from_slice(&(items.len() as u32).to_le_bytes());
    for item in items {
        // Encoding into a Vec cannot fail: the only error is buffer space,
        // and a Vec grows.
        let _ = item.encode_length_delimited(out);
    }
}

fn take_section<T: Message + Default>(buf: &mut &[u8]) -> Option<Vec<T>> {
    let count = u32::from_le_bytes(buf.get(..4)?.try_into().ok()?);
    *buf = &buf[4..];
    let mut items = Vec::with_capacity(count.min(100_000) as usize);
    for _ in 0..count {
        items.push(T::decode_length_delimited(&mut *buf).ok()?);
    }
    Some(items)
}

/// Persist a listing. Best-effort: a snapshot that cannot be written only
/// means the next connect waits like this one did.
pub fn save(host: &str, albums: &[AlbumProto], artists: &[ArtistProto], tracks: &[TrackProto]) {
    let Some(path) = path(host) else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut out = vec![VERSION];
    put_section(&mut out, albums);
    put_section(&mut out, artists);
    put_section(&mut out, tracks);
    if let Err(e) = std::fs::write(&path, out) {
        tracing::debug!(host, "could not write the library snapshot: {e}");
    }
}

/// The previous session's listing for this server, if one was saved and
/// still parses. Empty listings stay `None`: publishing zero albums over the
/// skeleton would read as "this library is empty" while the truth is still
/// loading.
pub fn load(host: &str) -> Option<Snapshot> {
    let data = std::fs::read(path(host)?).ok()?;
    let mut buf: &[u8] = data.as_slice();
    if buf.first() != Some(&VERSION) {
        return None;
    }
    buf = &buf[1..];
    let snapshot = Snapshot {
        albums: take_section(&mut buf)?,
        artists: take_section(&mut buf)?,
        tracks: take_section(&mut buf)?,
    };
    if snapshot.tracks.is_empty() && snapshot.albums.is_empty() {
        return None;
    }
    Some(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A round trip through the wire format: what was saved is what loads.
    #[test]
    fn survives_a_round_trip() {
        let track = TrackProto {
            id: "sub-1".into(),
            title: "Beyond".into(),
            artist: "Daft Punk".into(),
            ..Default::default()
        };
        let album = AlbumProto {
            id: "al-1".into(),
            title: "Random Access Memories".into(),
            ..Default::default()
        };
        let artist = ArtistProto {
            id: "ar-1".into(),
            name: "Daft Punk".into(),
            ..Default::default()
        };

        let mut out = vec![VERSION];
        put_section(&mut out, &[album.clone()]);
        put_section(&mut out, &[artist.clone()]);
        put_section(&mut out, &[track.clone()]);

        let mut buf: &[u8] = &out[1..];
        let albums: Vec<AlbumProto> = take_section(&mut buf).unwrap();
        let artists: Vec<ArtistProto> = take_section(&mut buf).unwrap();
        let tracks: Vec<TrackProto> = take_section(&mut buf).unwrap();
        assert_eq!(albums, vec![album]);
        assert_eq!(artists, vec![artist]);
        assert_eq!(tracks, vec![track]);
    }

    /// A truncated or corrupt section must come back `None`, not panic or
    /// hand over half a listing.
    #[test]
    fn a_torn_file_is_rejected() {
        // Claims four items, delivers none.
        let torn: &[u8] = &[4, 0, 0, 0];
        let mut buf = torn;
        assert!(take_section::<TrackProto>(&mut buf).is_none());
    }
}
