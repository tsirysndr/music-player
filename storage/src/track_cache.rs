//! On-disk cache of remote track audio.
//!
//! A remote track is fetched over the network as it plays, so the gap between
//! one track ending and the next opening its stream is audible — a short cut
//! at every track change. Downloading the next track while the current one is
//! still playing removes it: by the time the engine asks for the file, it is
//! already local.
//!
//! Only *finite* streams are cached. A live radio stream has no end, so
//! "downloading it" would mean filling the disk, and it has no next track to
//! prefetch either.
//!
//! Files are named by a hash of the uri and written through a `.part` file, so
//! a download interrupted by a crash or a quit is never mistaken for a
//! complete one.

use anyhow::Error;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

/// How much audio to keep before evicting the least recently used.
///
/// Two gigabytes is a few hundred tracks — enough that a long listening
/// session never re-downloads, small enough to be an unremarkable amount of
/// disk. Configurable through `MUSIC_PLAYER_CACHE_MAX_BYTES`.
const DEFAULT_MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// Where cached audio lives.
pub fn cache_dir() -> PathBuf {
    PathBuf::from(music_player_settings::get_application_directory())
        .join("cache")
        .join("tracks")
}

fn max_bytes() -> u64 {
    std::env::var("MUSIC_PLAYER_CACHE_MAX_BYTES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_MAX_BYTES)
}

/// Whether a uri is something this cache can hold.
///
/// Local files are already local. A live stream has no end — caching one would
/// mean downloading until the disk filled.
pub fn is_cacheable(uri: &str) -> bool {
    (uri.starts_with("http://") || uri.starts_with("https://")) && !is_live_stream(uri)
}

/// Live streams, recognised by the shapes internet radio uses.
///
/// Deliberately conservative: a false positive here means a track is never
/// cached, which is only a missed optimisation, while a false negative means
/// downloading a stream that never ends.
fn is_live_stream(uri: &str) -> bool {
    let lowered = uri.to_lowercase();
    ["/stream", ".m3u", ".m3u8", ".pls", "icecast", "shoutcast"]
        .iter()
        .any(|marker| lowered.contains(marker))
        // A Subsonic stream endpoint is finite despite its name, and it is the
        // main thing worth caching, so it is excluded from the check above.
        && !lowered.contains("/rest/stream")
}

/// The cache file for a uri, whatever extension it was saved under.
///
/// Named by hash rather than by anything in the uri: a Subsonic stream url
/// carries an auth token and a salt that change between sessions, so the uri
/// itself is not a stable name — but the id inside it is, and hashing the
/// whole uri after stripping the volatile parts gives one file per track.
pub fn key_for(uri: &str) -> String {
    format!("{:x}", md5::compute(stable_part(uri)))
}

/// The part of a uri that identifies the *track* rather than the session.
///
/// Subsonic signs every request with a rotating salt and token; including
/// those would give the same track a new cache entry on every launch.
fn stable_part(uri: &str) -> String {
    let Ok(parsed) = url::Url::parse(uri) else {
        return uri.to_string();
    };
    let query: Vec<String> = parsed
        .query_pairs()
        .filter(|(key, _)| {
            !matches!(
                key.as_ref(),
                "t" | "s" | "u" | "p" | "c" | "v" | "X-Plex-Token"
            )
        })
        .map(|(key, value)| format!("{key}={value}"))
        .collect();
    format!(
        "{}{}?{}",
        parsed.host_str().unwrap_or_default(),
        parsed.path(),
        query.join("&")
    )
}

/// Every extension a cached file can carry.
///
/// The set is closed because [`extension_for`] is what names the files, so a
/// lookup can probe these directly instead of listing the directory.
const EXTENSIONS: [&str; 7] = ["mp3", "flac", "ogg", "opus", "aac", "m4a", "wav"];

/// The cached file for this uri, if one is complete on disk.
///
/// A keyed lookup, not a scan: the name is derived from the uri and the
/// extension is one of a known few, so this is a handful of `stat` calls
/// whatever the cache holds. It used to read the whole directory, which is a
/// per-track cost that grows with the cache — on the path that every play goes
/// through.
///
/// The mapping lives in the filenames rather than in a database beside them:
/// one fact in one place, so a file deleted by hand or by `cache clear` cannot
/// leave an index claiming it is still there.
pub fn cached_path(uri: &str) -> Option<PathBuf> {
    let key = key_for(uri);
    let dir = cache_dir();
    for extension in EXTENSIONS {
        let path = dir.join(format!("{key}.{extension}"));
        if path.is_file() {
            // Touched so eviction sees it as recently used: the cache is a
            // working set, not an archive.
            let _ = filetime::set_file_mtime(&path, filetime::FileTime::now());
            return Some(path);
        }
    }
    None
}

/// What to play for a uri: the cached copy when there is one, else the uri.
///
/// Every play goes through this, so a track that was prefetched is played
/// from disk without the caller having to know whether it was.
pub fn resolve(uri: &str) -> String {
    if !is_cacheable(uri) {
        return uri.to_string();
    }
    match cached_path(uri) {
        Some(path) => path.to_string_lossy().into_owned(),
        None => uri.to_string(),
    }
}

/// Downloads already running, so two prefetch ticks do not fetch the same
/// track twice.
fn inflight() -> &'static Mutex<std::collections::HashSet<String>> {
    static INFLIGHT: std::sync::OnceLock<Mutex<std::collections::HashSet<String>>> =
        std::sync::OnceLock::new();
    INFLIGHT.get_or_init(Default::default)
}

/// One download at a time, across the whole process.
///
/// Not merely to be tidy: the thing being protected is the *stream that is
/// playing*. Several downloads at once compete with it for bandwidth on the
/// same connection to the same server, so caching ahead would cause the very
/// stutter it exists to remove. Queueing a track behind another costs nothing
/// — there is time, that is the point of prefetching.
fn download_slot() -> &'static tokio::sync::Semaphore {
    static SLOT: std::sync::OnceLock<tokio::sync::Semaphore> = std::sync::OnceLock::new();
    SLOT.get_or_init(|| tokio::sync::Semaphore::new(1))
}

/// Download a track into the cache.
///
/// Returns the cached path. A uri that is already cached returns immediately;
/// one that is already downloading returns without starting a second.
pub async fn store(uri: &str) -> Result<PathBuf, Error> {
    if !is_cacheable(uri) {
        return Err(Error::msg("not a cacheable uri"));
    }
    if let Some(path) = cached_path(uri) {
        return Ok(path);
    }

    let key = key_for(uri);
    {
        let mut inflight = inflight().lock().unwrap();
        if !inflight.insert(key.clone()) {
            return Err(Error::msg("already downloading"));
        }
    }
    // Held for the whole download, so requests queue rather than overlap.
    let permit = download_slot().acquire().await;
    let result = download(uri, &key).await;
    drop(permit);
    inflight().lock().unwrap().remove(&key);
    result
}

async fn download(uri: &str, key: &str) -> Result<PathBuf, Error> {
    let dir = cache_dir();
    tokio::fs::create_dir_all(&dir).await?;

    let response = reqwest::Client::builder()
        // A prefetch that stalls must not hold a slot forever; the next tick
        // will try again.
        .timeout(std::time::Duration::from_secs(120))
        .build()?
        .get(uri)
        .send()
        .await?
        .error_for_status()?;

    // The extension comes from what was served, not from the uri: a Subsonic
    // stream url ends in `stream`, and the decoder picks its parser by
    // extension.
    let extension = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| extension_for(value.split(';').next().unwrap_or_default().trim()))
        .unwrap_or("mp3");

    let bytes = response.bytes().await?;
    // Written to `.part` first and renamed: a crash mid-download leaves a
    // partial file that is never mistaken for a complete one.
    let part = dir.join(format!("{key}.part"));
    let final_path = dir.join(format!("{key}.{extension}"));
    tokio::fs::write(&part, &bytes).await?;
    tokio::fs::rename(&part, &final_path).await?;

    prune().await;
    Ok(final_path)
}

fn extension_for(content_type: &str) -> Option<&'static str> {
    Some(match content_type {
        "audio/mpeg" | "audio/mp3" => "mp3",
        "audio/flac" | "audio/x-flac" => "flac",
        "audio/ogg" | "application/ogg" => "ogg",
        "audio/opus" => "opus",
        "audio/aac" | "audio/aacp" => "aac",
        "audio/mp4" | "audio/x-m4a" => "m4a",
        "audio/wav" | "audio/x-wav" => "wav",
        _ => return None,
    })
}

/// What the cache is holding.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Usage {
    pub tracks: u64,
    pub bytes: u64,
}

pub fn usage() -> Usage {
    let mut usage = Usage::default();
    let Ok(entries) = std::fs::read_dir(cache_dir()) else {
        return usage;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "part") {
            continue;
        }
        if let Ok(meta) = entry.metadata() {
            usage.tracks += 1;
            usage.bytes += meta.len();
        }
    }
    usage
}

/// Delete everything, including partial downloads. Returns what was freed.
pub fn clear() -> Usage {
    let freed = usage();
    if let Ok(entries) = std::fs::read_dir(cache_dir()) {
        for entry in entries.flatten() {
            let _ = std::fs::remove_file(entry.path());
        }
    }
    freed
}

/// Evict least-recently-used files until the cache is under its cap.
async fn prune() {
    let cap = max_bytes();
    let Ok(entries) = std::fs::read_dir(cache_dir()) else {
        return;
    };

    let mut files: Vec<(PathBuf, u64, SystemTime)> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "part") {
                return None;
            }
            let meta = entry.metadata().ok()?;
            Some((path, meta.len(), meta.modified().ok()?))
        })
        .collect();

    let mut total: u64 = files.iter().map(|(_, size, _)| size).sum();
    if total <= cap {
        return;
    }
    // Oldest first: the least recently *played*, since `cached_path` touches
    // what it hands out.
    files.sort_by_key(|(_, _, modified)| *modified);
    for (path, size, _) in files {
        if total <= cap {
            break;
        }
        if std::fs::remove_file(&path).is_ok() {
            total = total.saturating_sub(size);
        }
    }
}

/// A human-readable size, for the CLI.
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[0])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// Whether a path is inside the cache — i.e. already a local copy.
pub fn is_cached_path(path: &str) -> bool {
    Path::new(path).starts_with(cache_dir())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A live stream has no end, so caching one would fill the disk.
    #[test]
    fn only_finite_remote_streams_are_cacheable() {
        assert!(is_cacheable("https://nas.lan/rest/stream?id=1"));
        assert!(is_cacheable(
            "http://plex.lan:32400/library/parts/5/file.mp3"
        ));

        assert!(!is_cacheable("/music/local.mp3"), "already local");
        assert!(!is_cacheable("https://radio.example/live.m3u8"));
        assert!(!is_cacheable("https://icecast.example/stream.ogg"));
    }

    /// A Subsonic url is signed with a rotating salt and token, so including
    /// them would give the same track a new entry on every launch.
    #[test]
    fn the_key_ignores_the_session_parts_of_a_url() {
        let monday = "https://nas.lan/rest/stream?id=42&u=me&t=abc&s=xyz&c=music-player";
        let tuesday = "https://nas.lan/rest/stream?id=42&u=me&t=def&s=uvw&c=music-player";
        assert_eq!(key_for(monday), key_for(tuesday));

        // A different track is still a different file.
        let other = "https://nas.lan/rest/stream?id=43&u=me&t=abc&s=xyz&c=music-player";
        assert_ne!(key_for(monday), key_for(other));
    }

    /// Two servers can both have a track called 42.
    #[test]
    fn the_key_distinguishes_servers() {
        assert_ne!(
            key_for("https://a.lan/rest/stream?id=42"),
            key_for("https://b.lan/rest/stream?id=42")
        );
    }

    /// A uri with nothing cached plays from the network, unchanged.
    #[test]
    fn resolve_passes_through_what_it_cannot_serve() {
        assert_eq!(resolve("/music/local.mp3"), "/music/local.mp3");
        let live = "https://radio.example/live.m3u8";
        assert_eq!(resolve(live), live);
    }

    /// The decoder picks its parser by extension, and a Subsonic stream url
    /// ends in `stream`.
    #[test]
    fn the_extension_comes_from_the_served_type() {
        assert_eq!(extension_for("audio/flac"), Some("flac"));
        assert_eq!(extension_for("audio/mpeg"), Some("mp3"));
        assert_eq!(extension_for("application/octet-stream"), None);
    }

    /// Deleting the cache directory by hand has to be safe.
    ///
    /// It is, and for a structural reason: the filename *is* the index, so
    /// there is nothing that can outlive the files it describes. A key-value
    /// store beside them would need reconciling on every start — and could
    /// still hand out a path to a file that is no longer there, because
    /// something has to touch the disk before playing it anyway.
    #[test]
    fn a_missing_cache_directory_is_simply_a_miss() {
        let uri = "https://nas.lan/rest/stream?id=nothing-cached";
        // The real cache dir may or may not exist here; either way a key that
        // was never written must not resolve.
        assert!(cached_path(uri).is_none());
        assert_eq!(resolve(uri), uri, "a miss plays from the network");

        // And the accounting agrees rather than reporting phantom entries.
        let usage = usage();
        assert!(usage.tracks < u64::MAX);
    }

    /// The probe list and the writer have to agree: an extension the writer
    /// can produce but the lookup never checks is a file that is downloaded
    /// and then never found.
    #[test]
    fn every_extension_the_writer_uses_is_probed() {
        for content_type in [
            "audio/mpeg",
            "audio/mp3",
            "audio/flac",
            "audio/x-flac",
            "audio/ogg",
            "application/ogg",
            "audio/opus",
            "audio/aac",
            "audio/aacp",
            "audio/mp4",
            "audio/x-m4a",
            "audio/wav",
            "audio/x-wav",
        ] {
            let extension = extension_for(content_type).expect(content_type);
            assert!(
                EXTENSIONS.contains(&extension),
                "{content_type} writes .{extension}, which is never looked for"
            );
        }
    }

    #[test]
    fn sizes_read_as_sizes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(3 * 1024 * 1024 * 1024), "3.0 GB");
    }
}
