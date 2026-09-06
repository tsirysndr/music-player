//! Reachability check for a stream url typed into the "add station" form.
//!
//! A station is only worth saving if its url actually serves audio, and the
//! answer is one request away: an Icecast/SHOUTcast server replies to
//! `Icy-MetaData: 1` with `icy-name` / `icy-genre` / `icy-br` headers, which is
//! both the proof it is a radio stream and everything the form needs to fill
//! itself in. Nothing but headers is read — a live stream never ends.

use std::time::Duration;

use crate::atproto::http;

/// How long to wait for a station to answer. Long enough for a slow origin,
/// short enough that a dead url does not hang the form.
const PROBE_TIMEOUT: Duration = Duration::from_secs(8);

/// What a probe found. `ok` is the only field a caller must look at; the rest
/// prefills the form when the station announces itself.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StreamCheck {
    pub ok: bool,
    /// Why it failed, phrased for the form. Empty when `ok`.
    pub error: String,
    /// `icy-name` — the station's own name.
    pub name: String,
    /// `icy-genre`.
    pub genre: String,
    /// `icy-br`, in kbps (0 when not advertised).
    pub bitrate: u32,
    /// Decoded from the content type: "MP3", "AAC", "OGG"…
    pub codec: String,
    /// `icy-url` — the station's homepage.
    pub homepage: String,
}

impl StreamCheck {
    fn failed(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            ..Default::default()
        }
    }
}

/// Content types a station can legitimately serve: raw audio, or a playlist
/// pointing at it (a `.pls`/`.m3u` url is resolved at play time).
fn codec_of(content_type: &str) -> Option<&'static str> {
    let value = content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    match value.as_str() {
        "audio/mpeg" | "audio/mp3" | "audio/mpeg3" => Some("MP3"),
        "audio/aac" | "audio/aacp" | "audio/x-aac" => Some("AAC"),
        "audio/mp4" | "audio/m4a" | "audio/x-m4a" => Some("AAC"),
        "audio/ogg" | "application/ogg" | "audio/opus" => Some("OGG"),
        "audio/flac" | "audio/x-flac" => Some("FLAC"),
        "audio/wav" | "audio/x-wav" => Some("WAV"),
        "audio/x-mpegurl" | "audio/mpegurl" | "application/vnd.apple.mpegurl" => Some("HLS"),
        "audio/x-scpls" | "application/pls+xml" => Some("PLS"),
        // Some Icecast setups answer plain `audio/*` subtypes we do not know,
        // and a few send nothing useful at all; anything under audio/ counts.
        other if other.starts_with("audio/") => Some(""),
        _ => None,
    }
}

/// Ask `url` whether it is a playable radio stream.
///
/// Never returns an error: the whole point is to report the failure back to the
/// form, so an unreachable host is a `StreamCheck` with `ok: false`.
pub async fn probe(url: &str) -> StreamCheck {
    let url = url.trim();
    if url.is_empty() {
        return StreamCheck::failed("Enter a stream url.");
    }
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return StreamCheck::failed("The stream url must start with http:// or https://.");
    }
    let client = match http() {
        Ok(client) => client,
        Err(e) => return StreamCheck::failed(format!("Could not open a connection: {e}")),
    };
    // GET rather than HEAD: streaming servers routinely reject HEAD, and the
    // ICY headers only come back on the real request. The body is dropped
    // unread — a live stream has no end to read to.
    let response = client
        .get(url)
        .header("Icy-MetaData", "1")
        .header("Range", "bytes=0-0")
        .timeout(PROBE_TIMEOUT)
        .send()
        .await;
    let response = match response {
        Ok(response) => response,
        Err(e) if e.is_timeout() => {
            return StreamCheck::failed("The station did not answer in time.")
        }
        Err(e) if e.is_connect() => return StreamCheck::failed("Could not reach the station."),
        Err(e) => return StreamCheck::failed(format!("Could not reach the station: {e}")),
    };
    if !response.status().is_success() && response.status().as_u16() != 206 {
        return StreamCheck::failed(format!(
            "The station answered {}.",
            response.status().as_u16()
        ));
    }

    let header = |name: &str| {
        response
            .headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .unwrap_or_default()
    };
    let content_type = header("content-type");
    let icy_name = header("icy-name");
    let has_icy = !icy_name.is_empty() || !header("icy-metaint").is_empty();
    let codec = match codec_of(&content_type) {
        Some(codec) => codec.to_owned(),
        // No usable content type. An `icy-*` header still proves this is a
        // stream; anything else is a web page, not a station.
        None if has_icy => String::new(),
        None => {
            return StreamCheck::failed(if content_type.is_empty() {
                "That url does not serve audio.".to_owned()
            } else {
                format!("That url serves {content_type}, not audio.")
            })
        }
    };

    StreamCheck {
        ok: true,
        error: String::new(),
        name: icy_name,
        genre: header("icy-genre"),
        bitrate: header("icy-br").parse().unwrap_or_default(),
        codec,
        homepage: header("icy-url"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_urls_that_are_not_http() {
        assert_eq!(
            probe("example.com/stream").await.error,
            "The stream url must start with http:// or https://."
        );
        assert!(!probe("  ").await.ok);
    }

    #[test]
    fn content_types_map_onto_codecs() {
        assert_eq!(codec_of("audio/mpeg"), Some("MP3"));
        assert_eq!(codec_of("audio/aacp; charset=utf-8"), Some("AAC"));
        // Unknown audio subtypes still count as a stream, just unlabelled.
        assert_eq!(codec_of("audio/weird"), Some(""));
        assert_eq!(codec_of("text/html"), None);
    }

    /// Live check against a real station. Ignored by default: it needs the
    /// network. Run with `cargo test -p music-player-storage -- --ignored probe`.
    #[tokio::test]
    #[ignore]
    async fn probes_a_live_station() {
        let check = probe("https://mangoradio.stream.laut.fm/mangoradio").await;
        assert!(check.ok, "{}", check.error);
        assert_eq!(check.codec, "MP3");
        assert!(!check.name.is_empty(), "the station announces an icy-name");
    }
}
