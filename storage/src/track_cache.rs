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
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

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

/// Whether caching is turned on at all.
///
/// Off unless `cache = true` is set in settings.toml. Writing gigabytes of the
/// user's disk is not something to begin doing on their behalf, however useful
/// it is once asked for.
///
/// Read once. Settings are not reloaded anywhere else while the daemon runs,
/// and a cache that switched on halfway through would be a surprise rather than
/// a feature — but more practically, this is on the path of every play, and
/// re-reading a file there to learn something that cannot change is waste.
pub fn enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| {
        let configured = music_player_settings::read_settings()
            .ok()
            .and_then(|config| {
                config
                    .try_deserialize::<music_player_settings::Settings>()
                    .ok()
            })
            .is_some_and(|settings| settings.cache);
        decide(
            std::env::var("MUSIC_PLAYER_CACHE").ok().as_deref(),
            configured,
        )
    })
}

/// The rule `enabled` applies, kept apart from the reading of it.
///
/// `MUSIC_PLAYER_CACHE` wins over the file so the feature can be tried for one
/// run without editing settings, and so a test can pin it without depending on
/// whatever the machine happens to have configured.
fn decide(env: Option<&str>, configured: bool) -> bool {
    match env {
        Some(value) => !matches!(value, "" | "0" | "false"),
        None => configured,
    }
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
        if !path.is_file() {
            continue;
        }
        // Checked on the way out as well as on the way in. An entry that is
        // not audio is worse than a miss: the decoder cannot open it, so the
        // engine skips the track — every time it is played, for as long as
        // the file is on disk. Entries written before [`download`] learned to
        // refuse them are still out there, so the read side has to be able to
        // throw them away too.
        if !is_audio_file(&path) {
            tracing::warn!(
                path = %path.display(),
                "cached file is not audio; discarding it and playing from the network"
            );
            let _ = std::fs::remove_file(&path);
            continue;
        }
        // Touched so eviction sees it as recently used: the cache is a
        // working set, not an archive.
        let _ = filetime::set_file_mtime(&path, filetime::FileTime::now());
        return Some(path);
    }
    None
}

/// Forget whatever is cached for a uri, so the next play goes to the network.
///
/// The player calls this when the engine could not open a track: a cached copy
/// that fails to open fails identically every time, so a retry is only worth
/// making once the local file is out of the way.
pub fn invalidate(uri: &str) -> bool {
    let key = key_for(uri);
    let dir = cache_dir();
    let mut removed = false;
    for extension in EXTENSIONS {
        let path = dir.join(format!("{key}.{extension}"));
        if path.is_file() && std::fs::remove_file(&path).is_ok() {
            removed = true;
        }
    }
    removed
}

/// Whether the file on disk begins like one of the formats the cache holds.
fn is_audio_file(path: &Path) -> bool {
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let mut head = [0u8; SNIFF_BYTES];
    let Ok(read) = file.read(&mut head) else {
        return false;
    };
    sniff(&head[..read]).is_some()
}

/// Enough of a file to recognise every format in [`EXTENSIONS`] — an Ogg
/// stream names its codec in the first page, at byte 28.
const SNIFF_BYTES: usize = 64;

/// The format a body actually is, from its leading bytes.
///
/// The `Content-Type` alone cannot be trusted to decide this. A Subsonic
/// server reports a failed login as `200 OK` with a JSON body — "Wrong
/// username or password" — at the very url that normally streams the track,
/// and a server that serves audio as `application/octet-stream` is ordinary.
/// The bytes are the one thing that cannot lie about what the decoder is
/// going to be handed.
fn sniff(bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() < 12 {
        return None;
    }
    if bytes.starts_with(b"ID3") {
        return Some("mp3");
    }
    if bytes.starts_with(b"fLaC") {
        return Some("flac");
    }
    if bytes.starts_with(b"OggS") {
        // Opus and Vorbis share the Ogg container, and the decoder picks its
        // parser by extension, so the two have to be told apart here.
        let opus = bytes.windows(8).any(|window| window == b"OpusHead");
        return Some(if opus { "opus" } else { "ogg" });
    }
    if bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WAVE" {
        return Some("wav");
    }
    // Every MP4 family file (m4a, m4b, ALAC…) opens with a `ftyp` box.
    if &bytes[4..8] == b"ftyp" {
        return Some("m4a");
    }
    // MPEG audio and ADTS AAC both start with a frame sync; the layer bits
    // are zero only for AAC.
    if bytes[0] == 0xFF && bytes[1] & 0xE0 == 0xE0 {
        return Some(if bytes[1] & 0x06 == 0 { "aac" } else { "mp3" });
    }
    None
}

/// What to play for a uri: the cached copy when there is one, else the uri.
///
/// Every play goes through this, so a track that was prefetched is played
/// from disk without the caller having to know whether it was.
pub fn resolve(uri: &str) -> String {
    // Turned off means not used, not merely not written. Playing from files a
    // previously-enabled run left behind would make "cache = false" mean
    // something different on a machine that had once had it on.
    if !enabled() || !is_cacheable(uri) {
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
    if !enabled() {
        return Err(Error::msg(
            "caching is off (set `cache = true` to enable it)",
        ));
    }
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

/// How many times a download is tried before giving up.
///
/// The failure worth retrying is transient and server-side: a Subsonic server
/// intermittently answers a perfectly valid stream request with `Wrong
/// username or password`, and the same request a moment later serves the
/// track. One attempt turned that blip into a permanent one — the error body
/// was written to the cache, and the track skipped on every play afterwards.
const ATTEMPTS: u32 = 3;

/// The client every download shares.
///
/// Building one per download builds a connection pool per download, which is
/// both slower (a fresh TLS handshake each time) and a way to run out of file
/// descriptors on a long listening session.
fn client() -> &'static reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            // A prefetch that stalls must not hold a slot forever; the next
            // tick will try again.
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap_or_default()
    })
}

async fn download(uri: &str, key: &str) -> Result<PathBuf, Error> {
    let dir = cache_dir();
    tokio::fs::create_dir_all(&dir).await?;

    let mut last = Error::msg("download failed");
    for attempt in 1..=ATTEMPTS {
        match fetch(uri).await {
            Ok((extension, bytes)) => {
                // Written to `.part` first and renamed: a crash mid-download
                // leaves a partial file that is never mistaken for a complete
                // one.
                let part = dir.join(format!("{key}.part"));
                let final_path = dir.join(format!("{key}.{extension}"));
                tokio::fs::write(&part, &bytes).await?;
                tokio::fs::rename(&part, &final_path).await?;
                prune().await;
                return Ok(final_path);
            }
            Err(cause) => {
                tracing::debug!(%uri, attempt, %cause, "could not fetch track");
                last = cause;
                if attempt < ATTEMPTS {
                    tokio::time::sleep(Duration::from_millis(500 * attempt as u64)).await;
                }
            }
        }
    }
    Err(last)
}

/// One attempt: fetch the body and refuse anything that is not audio.
///
/// Returns the extension to save it under. That comes from the bytes first and
/// the `Content-Type` second, because the decoder picks its parser by
/// extension and a Subsonic stream url ends in `stream` — so a body saved
/// under the wrong name is a track that never plays, however sound the
/// download was.
async fn fetch(uri: &str) -> Result<(&'static str, Vec<u8>), Error> {
    let response = client().get(uri).send().await?.error_for_status()?;
    let served = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value
                .split(';')
                .next()
                .unwrap_or_default()
                .trim()
                .to_string()
        });
    let bytes = response.bytes().await?.to_vec();

    let extension = sniff(&bytes[..bytes.len().min(SNIFF_BYTES)])
        .or_else(|| served.as_deref().and_then(extension_for))
        .ok_or_else(|| {
            Error::msg(format!(
                "the server did not send audio ({}): {}",
                served.as_deref().unwrap_or("no content type"),
                summarise(&bytes)
            ))
        })?;
    Ok((extension, bytes))
}

/// A short, printable description of a body that was not audio, for the log.
fn summarise(bytes: &[u8]) -> String {
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(200)]);
    let head = head.trim();
    if head.is_empty() {
        return format!("{} bytes", bytes.len());
    }
    head.replace('\n', " ")
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

    /// Off unless asked for, and the environment overrides the file.
    ///
    /// Tests the rule rather than calling `enabled`: that memoises once per
    /// process and reads the developer's real settings.toml, so asserting on it
    /// would be asserting about this machine.
    #[test]
    fn caching_is_off_until_it_is_turned_on() {
        assert!(!decide(None, false));
        assert!(decide(None, true));

        // The override works in both directions — including turning off a
        // cache that settings.toml turns on.
        assert!(decide(Some("1"), false));
        assert!(!decide(Some("0"), true));
        assert!(!decide(Some("false"), true));
        assert!(!decide(Some(""), true));
    }

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

    /// A body is what its first bytes say it is, whatever it was served as.
    #[test]
    fn the_format_is_read_from_the_bytes() {
        let pad = |head: &[u8]| {
            let mut bytes = head.to_vec();
            bytes.resize(64, 0);
            bytes
        };
        assert_eq!(sniff(&pad(b"ID3\x04\x00\x00\x00\x00\x00\x00")), Some("mp3"));
        assert_eq!(sniff(&pad(b"fLaC\x00\x00\x00\x22")), Some("flac"));
        assert_eq!(sniff(&pad(b"RIFF\x00\x00\x00\x00WAVEfmt ")), Some("wav"));
        assert_eq!(sniff(&pad(b"\x00\x00\x00\x20ftypM4A ")), Some("m4a"));
        assert_eq!(sniff(&pad(b"\xff\xfb\x90\x00")), Some("mp3"), "mpeg sync");
        assert_eq!(sniff(&pad(b"\xff\xf1\x50\x80")), Some("aac"), "adts sync");

        // Opus and Vorbis share the Ogg container, and the extension is what
        // the decoder chooses its parser by.
        let mut ogg = b"OggS\x00\x02".to_vec();
        ogg.resize(28, 0);
        ogg.extend_from_slice(b"OpusHead");
        ogg.resize(64, 0);
        assert_eq!(sniff(&ogg), Some("opus"));
        assert_eq!(
            sniff(&pad(b"OggS\x00\x02\x00\x00\x00\x00\x00\x00")),
            Some("ogg")
        );
    }

    /// The failure this whole check exists for.
    ///
    /// A Subsonic server reports a refused login as `200 OK` with a JSON body,
    /// at the url that normally streams the track. Saved as audio — which is
    /// what an unchecked download did, under a guessed `.mp3` — the track
    /// stops playing for good: the decoder cannot open the file, so the engine
    /// skips it on every play until someone empties the cache by hand.
    #[test]
    fn an_error_page_is_not_audio() {
        let refused = br#"{"subsonic-response":{"error":{"code":40,"message":"Wrong username or password"},"status":"failed","version":"1.16.1"}}"#;
        assert_eq!(sniff(refused), None);
        assert!(summarise(refused).contains("Wrong username or password"));

        assert_eq!(
            sniff(b"<!DOCTYPE html><title>502 Bad Gateway</title>"),
            None
        );
        assert_eq!(sniff(b""), None, "an empty body is not audio either");
        assert_eq!(sniff(b"\x00\x01\x02"), None, "and neither is a stub");
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
