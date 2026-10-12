//! Legacy source ABI conversion. Protocol selection belongs to the loader;
//! a failed v2 request must never enter this adapter.
use crate::{
    managed::{PlaybackCapabilities, PlaybackLease, SeekPolicy, StartPosition, Timeline},
    source_descriptor::SourceDescriptor,
};
use music_player_extensions::abi::{StreamRequest, StreamResponse};
use music_player_extensions::source_runtime::ResolvedLegacySource;
use music_player_provider::ProviderError;
use music_player_transport::HttpRequest;
use music_player_types::audio::AudioSelection;
use std::collections::BTreeMap;

/// Carry actual browse tags to the queue, replacing guest transport URIs with
/// the host's persistent local reference. V1 has no independent get-item API.
pub(crate) fn browse_track(
    source: &music_player_types::source::LocalSourceRef,
    item: music_player_extensions::abi::TrackInfo,
) -> Result<music_player_types::types::Track, ProviderError> {
    use music_player_types::{
        source::{LocalSourceRef, ResourceKind},
        types::{Album, Track},
    };
    LocalSourceRef::parse(&source.to_handle()).map_err(|_| invalid())?;
    if source.kind != ResourceKind::Item
        || source.item_id != item.id
        || !item.duration.is_finite()
        || item.duration < 0.0
    {
        return Err(invalid());
    }
    Ok(Track {
        id: source.to_handle(),
        uri: source.to_handle(),
        title: item.title,
        artist: item.artist,
        // V1 has album tags but no album ID on a track. Keep display metadata
        // without turning the title into a navigable resource identity.
        album: (!item.album.is_empty() || !item.album_artist.is_empty() || item.year.is_some())
            .then_some(Album {
                title: item.album,
                artist: item.album_artist,
                year: item.year,
                ..Default::default()
            }),
        duration: (item.duration > 0.0).then_some(item.duration),
        track_number: item.track_number,
        ..Default::default()
    })
}

/// Validate before calling the guest: v1 has no offset or selection argument.
pub(crate) fn request(
    item_id: &str,
    selection: &AudioSelection,
    offset_ms: u64,
) -> Result<StreamRequest, ProviderError> {
    if item_id.trim().is_empty() {
        return Err(invalid());
    }
    if offset_ms != 0 || !matches!(selection, AudioSelection::Auto) {
        return Err(ProviderError::Other(
            "source v1 does not support offsets or audio selection".into(),
        ));
    }
    Ok(StreamRequest {
        track_id: item_id.to_owned(),
    })
}

/// MIME is the legacy format hint. A URL suffix cannot override it, and an
/// empty/unknown MIME must not silently become MP3. Decoder formats are host
/// capabilities, separate from this MIME mapping.
pub(crate) fn descriptor(
    resolved: ResolvedLegacySource,
    format_exts: &[String],
    follow_redirects: bool,
) -> Result<SourceDescriptor, ProviderError> {
    let ResolvedLegacySource { response, lease } = resolved;
    convert(
        response,
        format_exts,
        follow_redirects,
        PlaybackLease::new(move || drop(lease)),
    )
}

fn convert(
    response: StreamResponse,
    format_exts: &[String],
    follow_redirects: bool,
    lease: PlaybackLease,
) -> Result<SourceDescriptor, ProviderError> {
    let format = format_hint(&response.mime_type)?;
    if !format_exts.iter().any(|supported| supported == format) {
        return Err(ProviderError::Other(
            "source v1 audio format is unavailable for streaming".into(),
        ));
    }
    let url = url::Url::parse(&response.url).map_err(|_| invalid())?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(invalid());
    }
    let headers: BTreeMap<String, String> = if response.headers.trim().is_empty() {
        BTreeMap::new()
    } else {
        serde_json::from_str(&response.headers).map_err(|_| invalid())?
    };
    let mut request = HttpRequest::new(response.url);
    request.follow_redirects = follow_redirects;
    for (name, value) in headers {
        request.headers.append(
            http::header::HeaderName::from_bytes(name.as_bytes()).map_err(|_| invalid())?,
            http::header::HeaderValue::from_str(&value).map_err(|_| invalid())?,
        );
    }
    Ok(SourceDescriptor {
        request,
        format_ext: format.to_owned(),
        start: StartPosition::RequestedOnly,
        capabilities: PlaybackCapabilities {
            timeline: Timeline::Unknown,
            seek_policy: SeekPolicy::None,
        },
        requested_offset_ms: 0,
        pin: None,
        lease,
    })
}

fn format_hint(value: &str) -> Result<&'static str, ProviderError> {
    let mime: mime::Mime = value.trim().parse().map_err(|_| invalid())?;
    let mut codecs = mime
        .params()
        .filter(|(name, _)| name.as_str().eq_ignore_ascii_case("codecs"));
    let codec = codecs
        .next()
        .map(|(_, value)| value.as_str().to_ascii_lowercase());
    if codecs.next().is_some() {
        return Err(invalid());
    }
    let format = match mime.essence_str() {
        "audio/mpeg" | "audio/mp3" => "mp3",
        "audio/flac" | "audio/x-flac" => "flac",
        // The engine chooses Vorbis for `ogg` and Opus for `opus`. The Ogg
        // container alone cannot identify which decoder to use.
        "audio/ogg" | "application/ogg" => match codec.as_deref() {
            Some("opus") => "opus",
            Some("vorbis") => "ogg",
            _ => {
                return Err(ProviderError::Other(
                    "source v1 Ogg MIME requires an audio codec hint".into(),
                ))
            }
        },
        "audio/vorbis" | "audio/x-vorbis+ogg" => "ogg",
        "audio/opus" => "opus",
        "audio/aac" | "audio/aacp" | "audio/x-aac" => "aac",
        "audio/wav" | "audio/wave" | "audio/x-wav" => "wav",
        // MP4 is not offered by this forward-only adapter.
        _ => {
            return Err(ProviderError::Other(
                "unsupported source v1 audio MIME type".into(),
            ))
        }
    };
    Ok(format)
}

fn invalid() -> ProviderError {
    ProviderError::Other("invalid source v1 playback response".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    fn response(mime: &str, headers: &str) -> StreamResponse {
        StreamResponse {
            url: "http://localhost/misleading.mp3?token=private".into(),
            mime_type: mime.into(),
            headers: headers.into(),
        }
    }

    #[test]
    fn browse_tags_keep_host_identity_and_discard_guest_transport_uri() {
        use music_player_types::source::{LocalSourceRef, ResourceKind};
        let source = LocalSourceRef::for_extension(
            "legacy.fixture",
            "account-a".into(),
            9,
            ResourceKind::Item,
            "track-id".into(),
        )
        .unwrap();
        let mut item = music_player_extensions::abi::TrackInfo {
            id: "track-id".into(),
            title: "Guest title".into(),
            artist: "Guest artist".into(),
            album: "Guest album".into(),
            album_artist: "Album artist".into(),
            year: Some(2024),
            duration: 301.5,
            track_number: Some(4),
            uri: "http://fixture/audio?token=temporary".into(),
            ..Default::default()
        };
        let track = browse_track(&source, item.clone()).unwrap();
        assert_eq!(track.id, source.to_handle());
        assert_eq!(track.uri, track.id);
        assert_eq!(track.title, item.title);
        assert_eq!(track.artist, item.artist);
        assert_eq!(track.duration, Some(301.5));
        assert_eq!(track.track_number, Some(4));
        let album = track.album.as_ref().unwrap();
        assert_eq!(album.title, item.album);
        assert_eq!(album.artist, item.album_artist);
        assert_eq!(album.year, item.year);
        assert!(album.id.is_empty() && album.artist_id.is_none());
        let queued: music_player_entity::track::Model = track.into();
        assert!(queued.album_id.is_none());
        let restored: music_player_types::types::Track = queued.into();
        assert_eq!(restored.album.unwrap().title, item.album);
        item.duration = 0.0;
        assert_eq!(browse_track(&source, item.clone()).unwrap().duration, None);
        item.id = "other-item".into();
        assert!(browse_track(&source, item).is_err());
    }

    #[test]
    fn mime_and_headers_drive_legacy_descriptor_without_inventing_seek() {
        let result = convert(
            response(
                "Audio/FLAC; charset=binary",
                r#"{"Authorization":"Bearer fixture"}"#,
            ),
            &["flac".into()],
            false,
            PlaybackLease::new(|| {}),
        )
        .unwrap();
        assert_eq!(result.format_ext, "flac");
        assert_eq!(result.request.headers["authorization"], "Bearer fixture");
        assert!(!result.request.follow_redirects);
        assert_eq!(result.capabilities.seek_policy, SeekPolicy::None);
        assert_eq!(result.capabilities.timeline, Timeline::Unknown);
        assert!(result.pin.is_none());
        assert!(request("item", &AudioSelection::Auto, 1).is_err());
    }

    #[test]
    fn invalid_legacy_results_drop_the_owned_instance_lease_without_exposing_input() {
        let released = Arc::new(AtomicUsize::new(0));
        for (mime, headers) in [
            ("", ""),
            ("audio/mp4", ""),
            ("audio/flac", ""),
            ("audio/mpeg", r#"{"Authorization":123}"#),
        ] {
            let count = released.clone();
            let error = convert(
                response(mime, headers),
                &["mp3".into()],
                true,
                PlaybackLease::new(move || {
                    count.fetch_add(1, Ordering::SeqCst);
                }),
            )
            .err()
            .unwrap();
            assert!(!error.to_string().contains("private"));
        }
        assert_eq!(released.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn ogg_codec_selects_decoder_and_explicit_selection_is_not_silently_ignored() {
        let result = convert(
            response("audio/ogg; codecs=\"opus\"", ""),
            &["opus".into()],
            true,
            PlaybackLease::new(|| {}),
        )
        .unwrap();
        assert_eq!(result.format_ext, "opus");
        for mime in [
            "audio/ogg",
            "audio/ogg; codecs=theora",
            "audio/ogg; codecs=opus; codecs=vorbis",
        ] {
            assert!(convert(
                response(mime, ""),
                &["ogg".into(), "opus".into()],
                true,
                PlaybackLease::new(|| {})
            )
            .is_err());
        }
        assert!(request(
            "item",
            &AudioSelection::Explicit {
                media_source_id: "version".into(),
                audio_stream_index: 1
            },
            0
        )
        .is_err());
        assert_eq!(
            request("item", &AudioSelection::Auto, 0).unwrap().track_id,
            "item"
        );
    }
}
