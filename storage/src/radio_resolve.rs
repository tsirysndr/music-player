//! Turn a station's advertised url into one the decoder can actually play.
//!
//! Directory entries rarely point straight at audio. A TuneIn station is a
//! `Tune.ashx` endpoint that answers with an M3U listing `.pls` files, and each
//! of those is another playlist — so unwrapping has to follow a *chain*, not a
//! single hop.
//!
//! And some origins will not talk to us at all: a station's playlist can sit
//! behind a bot wall that answers every non-browser request with an HTML
//! "verifying your request" page, whatever `User-Agent` is sent. Nothing at the
//! HTTP layer gets past that, so the last resort is the same one atradio.fm
//! uses — replay the request through a media proxy, which fetches the stream
//! from its own host and pipes it back.

use std::time::Duration;

use music_player_settings::{read_settings, Settings, DEFAULT_MEDIA_PROXY_URL};

use crate::atproto::http;

/// A playlist body is a small text file; a station that answers a fetch with
/// megabytes is not one, so the read is capped in time rather than size.
const FETCH_TIMEOUT: Duration = Duration::from_secs(10);

/// How many playlists to follow before giving up. TuneIn needs two
/// (`Tune.ashx` → `.pls` → stream); the rest is headroom against a loop.
const MAX_HOPS: usize = 5;

/// Base url of the media proxy, from settings. Empty disables the fallback.
fn media_proxy() -> String {
    read_settings()
        .ok()
        .and_then(|config| config.try_deserialize::<Settings>().ok())
        .map(|settings| settings.media_proxy_url)
        .unwrap_or_else(|| DEFAULT_MEDIA_PROXY_URL.to_owned())
        .trim_end_matches('/')
        .to_owned()
}

/// Route `url` through the media proxy's CORS/relay stream endpoint.
pub fn proxied(url: &str) -> Option<String> {
    let base = media_proxy();
    if base.is_empty() || url.starts_with(&base) {
        return None;
    }
    Some(format!("{base}/api/stream?url={}", encode(url)))
}

/// Percent-encode a url for use as a query-string value.
fn encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 16);
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// True for urls that name a playlist outright. TuneIn's `Tune.ashx` carries no
/// extension, so it is matched by name.
fn looks_like_playlist(url: &str) -> bool {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let lower = path.to_ascii_lowercase();
    // `.m3u8` is HLS: its segment uris resolve against the manifest, so it is
    // played as-is and never unwrapped.
    lower.ends_with(".pls")
        || lower.ends_with(".m3u")
        || lower.ends_with("tune.ashx")
        || url.to_ascii_lowercase().contains("tune.ashx?")
}

/// True for the content types a playlist body comes back as.
fn playlist_content_type(content_type: &str) -> bool {
    let value = content_type.to_ascii_lowercase();
    (value.contains("mpegurl") && !value.contains("apple")) || value.contains("scpls")
}

/// The first playable url in a `.pls` / `.m3u` / plain-text playlist body.
pub fn first_stream_url(body: &str) -> Option<String> {
    // .pls — `File1=http://…`
    for line in body.lines() {
        let line = line.trim();
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        if key.starts_with("file") && key[4..].chars().all(|c| c.is_ascii_digit()) {
            let value = value.trim();
            if value.starts_with("http://") || value.starts_with("https://") {
                return Some(value.to_owned());
            }
        }
    }
    // .m3u / plain text — the first non-comment line that is a url.
    body.lines()
        .map(str::trim)
        .find(|line| {
            !line.is_empty()
                && !line.starts_with('#')
                && (line.starts_with("http://") || line.starts_with("https://"))
        })
        .map(str::to_owned)
}

/// Resolve `url` to a playable stream.
///
/// Follows the playlist chain as far as the origin allows. When a hop cannot be
/// read — a bot wall, a dead host, a body that is not a playlist — that hop is
/// handed to the media proxy instead, which is what makes stations like
/// AlternativeRadio.us (whose `.pls` sits behind such a wall) play at all.
/// Falls back to the url it was given when nothing better can be worked out.
pub async fn resolve(url: &str) -> String {
    let original = url.trim().to_owned();
    if !original.starts_with("http://") && !original.starts_with("https://") {
        return original;
    }
    let Ok(client) = http() else {
        return original;
    };

    let mut current = original.clone();
    for _ in 0..MAX_HOPS {
        if !looks_like_playlist(&current) {
            return current;
        }
        let response = client
            .get(&current)
            .header("Icy-MetaData", "1")
            .timeout(FETCH_TIMEOUT)
            .send()
            .await;
        let Ok(response) = response.and_then(|r| r.error_for_status()) else {
            // Unreachable by us; the proxy may still get there.
            return proxied(&current).unwrap_or(original);
        };
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        // A url that *looked* like a playlist but answers with audio is the
        // stream itself — some hosts serve `/x.pls` as a live mount.
        if content_type.starts_with("audio/") && !playlist_content_type(&content_type) {
            return current;
        }
        let Ok(body) = response.text().await else {
            return proxied(&current).unwrap_or(original);
        };
        match first_stream_url(&body) {
            Some(next) if next != current => current = next,
            // A playlist url that yields no urls is a wall or an error page.
            _ => return proxied(&current).unwrap_or(original),
        }
    }
    proxied(&current).unwrap_or(original)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_playlist_urls() {
        assert!(looks_like_playlist("http://h/x.pls"));
        assert!(looks_like_playlist("http://h/x.m3u"));
        assert!(looks_like_playlist(
            "https://api.atradio.fm/api/tunein/Tune.ashx?id=s221580&formats=mp3,aac"
        ));
        // HLS resolves its segments against the manifest, so it is played whole.
        assert!(!looks_like_playlist("http://h/master.m3u8"));
        assert!(!looks_like_playlist("http://h/stream"));
    }

    #[test]
    fn picks_the_first_url_out_of_a_playlist() {
        // The M3U body TuneIn's Tune.ashx returns for AlternativeRadio.us.
        let m3u = "http://avbhost.com/listen/alternativeradio/alt64.pls\n\
                   https://avbhost.com/listen/alternativeradio/alt64.pls\n";
        assert_eq!(
            first_stream_url(m3u).as_deref(),
            Some("http://avbhost.com/listen/alternativeradio/alt64.pls")
        );

        let pls = "[playlist]\nNumberOfEntries=1\nFile1=http://ice.example/stream\nTitle1=X\n";
        assert_eq!(
            first_stream_url(pls).as_deref(),
            Some("http://ice.example/stream")
        );

        // A bot wall answers with HTML, which has no urls of its own to follow.
        assert_eq!(first_stream_url("<!DOCTYPE html><html></html>"), None);
    }

    #[test]
    fn proxies_through_the_media_proxy() {
        let proxied = proxied("http://avbhost.com/listen/alternativeradio/alt64.pls").unwrap();
        assert!(proxied.ends_with(
            "/api/stream?url=http%3A%2F%2Favbhost.com%2Flisten%2Falternativeradio%2Falt64.pls"
        ));
        // Already proxied — never wrap twice.
        assert_eq!(proxied.clone(), proxied);
        assert!(super::proxied(&proxied).is_none());
    }

    /// Live end-to-end check against the station that prompted all this: its
    /// `.pls` is behind a bot wall, so resolution has to end at the proxy.
    /// Ignored by default (network). Run with
    /// `cargo test -p music-player-storage -- --ignored resolves_a_tunein`.
    #[tokio::test]
    #[ignore]
    async fn resolves_a_tunein_station_behind_a_bot_wall() {
        let resolved =
            resolve("https://api.atradio.fm/api/tunein/Tune.ashx?id=s221580&formats=mp3,aac").await;
        assert!(
            resolved.contains("/api/stream?url="),
            "expected a proxied url, got {resolved}"
        );
        // And the proxied url must actually serve audio.
        let response = http()
            .unwrap()
            .get(&resolved)
            .header("Icy-MetaData", "1")
            .timeout(FETCH_TIMEOUT)
            .send()
            .await
            .unwrap();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        assert!(content_type.starts_with("audio/"), "got {content_type}");
    }
}
