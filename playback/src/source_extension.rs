//! Conversion after account-bound v2 runtime dispatch. No transport state is persisted.
use crate::{
    managed::{PlaybackCapabilities, PlaybackLease, SeekPolicy, StartPosition, Timeline},
    source_descriptor::SourceDescriptor,
};
use music_player_extensions::{abi::source_v2 as wire, source_runtime::ResolvedSource};
use music_player_provider::ProviderError;
use music_player_transport::HttpRequest;
use music_player_types::{
    audio::{AudioPin, AudioSelection},
    source::{ResourceKind, SourceRef},
};
use std::time::Duration;

/// The caller binds the runtime to source.account_id and source.extension_id().
/// Output formats come from the active decoder, not the URL or source's MIME.
pub(crate) fn resolve_request(
    source: &SourceRef,
    selection: &AudioSelection,
    offset_ms: u64,
    format_exts: Vec<String>,
    follow_redirects: bool,
) -> Result<wire::ResolveRequest, ProviderError> {
    if SourceRef::parse(&source.to_handle()).is_err()
        || source.extension_id().is_none()
        || source.kind != ResourceKind::Item
        || format_exts.is_empty()
    {
        return Err(invalid());
    }
    Ok(wire::ResolveRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        identity: wire::SourceIdentity {
            server_id: source.remote.server_id.clone(),
            user_id: source.remote.user_id.clone(),
        },
        item_id: source.item_id.clone(),
        offset_ms,
        selection: selection_to_wire(selection),
        format_exts,
        follow_redirects,
    })
}

pub(crate) fn descriptor(
    resolved: ResolvedSource,
    follow_redirects: bool,
) -> Result<SourceDescriptor, ProviderError> {
    let ResolvedSource { response, lease } = resolved;
    // Every conversion failure also drops this lease on its original instance.
    from_response(
        response,
        PlaybackLease::new(move || drop(lease)),
        follow_redirects,
    )
}

fn from_response(
    response: wire::ResolveResponse,
    lease: PlaybackLease,
    follow_redirects: bool,
) -> Result<SourceDescriptor, ProviderError> {
    let mut request = HttpRequest::new(response.url);
    request.follow_redirects = follow_redirects;
    for header in response.headers {
        let name =
            http::header::HeaderName::from_bytes(header.name.as_bytes()).map_err(|_| invalid())?;
        let value = http::header::HeaderValue::from_str(&header.value).map_err(|_| invalid())?;
        // append preserves repeated values; insert would silently replace them.
        request.headers.append(name, value);
    }
    match response.cache_policy {
        wire::CachePolicy::NoPersistentCache => {}
    }
    let start = match response.start {
        wire::StartPosition::RequestedOnly {} => StartPosition::RequestedOnly,
        wire::StartPosition::Calibrated {
            offset_ms,
            max_error_ms,
        } => StartPosition::Calibrated {
            offset: Duration::from_millis(offset_ms),
            max_error: Duration::from_millis(max_error_ms),
        },
    };
    let timeline = match response.timeline {
        wire::Timeline::Finite { duration_ticks } => Timeline::Finite { duration_ticks },
        wire::Timeline::Live {} => Timeline::Live,
        wire::Timeline::Unknown {} => Timeline::Unknown,
    };
    let seek_policy = match response.seek_policy {
        wire::SeekPolicy::ServerOffset => SeekPolicy::ServerOffset,
        wire::SeekPolicy::None => SeekPolicy::None,
    };
    Ok(SourceDescriptor {
        request,
        format_ext: response.format_ext,
        start,
        capabilities: PlaybackCapabilities {
            timeline,
            seek_policy,
        },
        requested_offset_ms: response.requested_offset_ms,
        pin: response.actual_pin.map(pin_from_wire),
        lease,
    })
}

fn invalid() -> ProviderError {
    ProviderError::Other("invalid extension playback descriptor".into())
}

/// Browse metadata inherits host-owned account identity; guest IDs are only
/// resource IDs within it. No artist/album navigation ID is invented from a label.
pub(crate) fn media_entry(
    account_source: &SourceRef,
    item: wire::MediaItem,
) -> Result<music_player_provider::MediaEntry, ProviderError> {
    if account_source.extension_id().is_none()
        || SourceRef::parse(&account_source.to_handle()).is_err()
        || (item.is_container && item.playable)
    {
        return Err(invalid());
    }
    let source = SourceRef {
        kind: if item.is_container {
            ResourceKind::Container
        } else {
            ResourceKind::Item
        },
        item_id: item.item_id,
        ..account_source.clone()
    };
    let handle = source.to_handle();
    SourceRef::parse(&handle).map_err(|_| invalid())?;
    let music = item.media_type.as_deref() == Some("Audio");
    let track = item.playable.then(|| music_player_types::types::Track {
        id: handle.clone(),
        uri: handle.clone(),
        title: item.title.clone(),
        artist: item.artist.unwrap_or_default(),
        album: (item.album.is_some() || item.album_artist.is_some() || item.year.is_some()).then(
            || music_player_types::types::Album {
                title: item.album.unwrap_or_default(),
                artist: item.album_artist.unwrap_or_default(),
                year: item.year,
                ..Default::default()
            },
        ),
        // This legacy display field does not establish the playback timeline.
        // Exact ticks and seek capability come from the resolved descriptor.
        duration: item
            .runtime_ticks
            .map(|ticks| (ticks as f64 / 10_000_000.0) as f32),
        track_number: if music { item.track_number } else { None },
        disc_number: 0,
        ..Default::default()
    });
    Ok(music_player_provider::MediaEntry {
        id: handle,
        title: item.title,
        item_type: item.item_type,
        media_type: item.media_type,
        is_container: item.is_container,
        season_number: item.season_number,
        episode_number: item.episode_number,
        track,
    })
}

/// Item lookup must match the requested leaf, unlike a browse result.
pub(crate) fn media_track(
    source: &SourceRef,
    response: wire::ItemResponse,
) -> Result<music_player_types::types::Track, ProviderError> {
    if source.kind != ResourceKind::Item || source.item_id != response.item.item_id {
        return Err(invalid());
    }
    media_entry(source, response.item)?
        .track
        .ok_or_else(invalid)
}

/// Called with the host-bound source after the runtime validates candidates.
/// The guest never chooses the persistent account/source handle.
pub(crate) fn audio_options(
    source: &SourceRef,
    response: wire::AudioOptionsResponse,
) -> Result<music_player_types::audio::AudioOptions, ProviderError> {
    use music_player_types::audio::{AudioOptions, AudioStreamOption, AudioVersionOption};
    if source.extension_id().is_none()
        || SourceRef::parse(&source.to_handle()).is_err()
        || source.kind != ResourceKind::Item
        || response.item_id != source.item_id
    {
        return Err(invalid());
    }
    Ok(AudioOptions {
        source: source.to_handle(),
        versions: response
            .versions
            .into_iter()
            .map(|version| AudioVersionOption {
                id: version.id,
                name: version.name,
                runtime_ticks: version.runtime_ticks,
                default_audio_stream_index: version.default_audio_stream_index,
                unavailable_reason: version.unavailable_reason,
                audio_streams: version
                    .audio_streams
                    .into_iter()
                    .map(|stream| AudioStreamOption {
                        index: stream.index,
                        title: stream.title,
                        display_title: stream.display_title,
                        language: stream.language,
                        codec: stream.codec,
                        channels: stream.channels,
                        sample_rate: stream.sample_rate,
                        is_default: stream.is_default,
                        unavailable_reason: stream.unavailable_reason,
                    })
                    .collect(),
            })
            .collect(),
    })
}

pub(crate) fn selection_to_wire(selection: &AudioSelection) -> wire::AudioSelection {
    match selection {
        AudioSelection::Auto => wire::AudioSelection::Auto {},
        AudioSelection::Explicit {
            media_source_id,
            audio_stream_index,
        } => wire::AudioSelection::Explicit {
            media_source_id: media_source_id.clone(),
            audio_stream_index: *audio_stream_index,
        },
        AudioSelection::Pinned(pin) => wire::AudioSelection::Pinned {
            pin: wire::AudioPin {
                media_source_id: pin.media_source_id.clone(),
                audio_stream_index: pin.audio_stream_index,
                runtime_ticks: pin.runtime_ticks,
                etag: pin.etag.clone(),
                codec: pin.codec.clone(),
                channels: pin.channels,
                sample_rate: pin.sample_rate,
            },
        },
    }
}
fn pin_from_wire(pin: wire::AudioPin) -> AudioPin {
    AudioPin {
        media_source_id: pin.media_source_id,
        audio_stream_index: pin.audio_stream_index,
        runtime_ticks: pin.runtime_ticks,
        etag: pin.etag,
        codec: pin.codec,
        channels: pin.channels,
        sample_rate: pin.sample_rate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use music_player_types::source::RemoteIdentity;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    #[test]
    fn options_keep_host_account_and_reject_wrong_item_or_container() {
        let source = SourceRef {
            resolver: SourceRef::extension_resolver("fixture.source").unwrap(),
            account_id: "account-a".into(),
            remote: RemoteIdentity {
                server_id: "server".into(),
                user_id: "user".into(),
            },
            kind: ResourceKind::Item,
            item_id: "55508".into(),
        };
        let response = wire::AudioOptionsResponse {
            item_id: "55508".into(),
            versions: vec![],
        };
        let options = audio_options(&source, response.clone()).unwrap();
        assert_eq!(options.source, source.to_handle());
        assert!(options.versions.is_empty());
        let mut other_account = source.clone();
        other_account.account_id = "account-b".into();
        let other = audio_options(&other_account, response.clone()).unwrap();
        assert_ne!(options.source, other.source);
        let mut wrong_item = response.clone();
        wrong_item.item_id = "different-item".into();
        assert!(audio_options(&source, wrong_item).is_err());
        let mut container = source.clone();
        container.kind = ResourceKind::Container;
        assert!(audio_options(&container, response.clone()).is_err());
        let mut native = source;
        native.resolver = "emby".into();
        assert!(audio_options(&native, response).is_err());
    }
    fn response() -> wire::ResolveResponse {
        serde_json::from_value(serde_json::json!({
            "requestId":"fixture", "requestedOffsetMs":u64::MAX,
            "url":"http://fixture.invalid/audio",
            "headers":[{"name":"x-fixture","value":"first"},{"name":"x-fixture","value":"second"}],
            "mimeType":"audio/flac","formatExt":"flac","actualPin":null,
            "start":{"type":"calibrated","offsetMs":7200123u64,"maxErrorMs":17},
            "timeline":{"type":"unknown"},"seekPolicy":"serverOffset",
            "cachePolicy":"noPersistentCache","leaseId":"opaque"
        }))
        .unwrap()
    }
    #[test]
    fn episode_metadata_preserves_host_identity_and_large_season_without_music_numbering() {
        let source = SourceRef {
            resolver: SourceRef::extension_resolver("fixture.source").unwrap(),
            account_id: "family-account".into(),
            remote: RemoteIdentity {
                server_id: "emby-server".into(),
                user_id: "family".into(),
            },
            kind: ResourceKind::Item,
            item_id: "55508".into(),
        };
        let item: wire::MediaItem = serde_json::from_value(serde_json::json!({
            "itemId":"55508",
            "title":"My First Ever Ender Dragon Fight! [First Ever Minecraft Playthrough Ep.40]",
            "itemType":"Episode", "mediaType":"Video",
            "isContainer":false, "playable":true,
            "seasonNumber":2025050473u64, "episodeNumber":41,
            "runtimeTicks":191429666670u64, "trackNumber":40
        }))
        .unwrap();
        let entry = media_entry(&source, item.clone()).unwrap();
        assert_eq!(entry.season_number, Some(2025050473));
        assert_eq!(entry.episode_number, Some(41));
        let track = entry.track.unwrap();
        assert_eq!(track.id, source.to_handle());
        assert_eq!(track.uri, track.id);
        assert_eq!(track.disc_number, 0);
        assert_eq!(track.track_number, None);
        assert!(track.album.is_none());
        let mut tagged = item.clone();
        tagged.album = Some("Playthrough".into());
        tagged.album_artist = Some("EvieGaming".into());
        tagged.year = Some(2025);
        let tagged_track = media_track(&source, wire::ItemResponse { item: tagged }).unwrap();
        let album = tagged_track.album.unwrap();
        assert_eq!(album.title, "Playthrough");
        assert_eq!(album.artist, "EvieGaming");
        assert_eq!(album.year, Some(2025));
        assert!(album.id.is_empty());
        let mut other = source.clone();
        other.account_id = "other-account".into();
        assert_ne!(media_entry(&other, item.clone()).unwrap().id, entry.id);
        let mut container = item.clone();
        container.is_container = true;
        container.playable = false;
        assert!(media_entry(&source, container.clone())
            .unwrap()
            .track
            .is_none());
        assert!(media_track(
            &source,
            wire::ItemResponse {
                item: container.clone()
            }
        )
        .is_err());
        container.playable = true;
        assert!(media_entry(&source, container).is_err());
        let mut wrong = item;
        wrong.item_id = "different-episode".into();
        assert!(media_track(&source, wire::ItemResponse { item: wrong }).is_err());
    }
    #[test]
    fn conversion_keeps_headers_offsets_capabilities_and_lease_ownership() {
        let releases = Arc::new(AtomicUsize::new(0));
        let count = releases.clone();
        let result = from_response(
            response(),
            PlaybackLease::new(move || {
                count.fetch_add(1, Ordering::SeqCst);
            }),
            false,
        )
        .unwrap();
        let values: Vec<_> = result
            .request
            .headers
            .get_all("x-fixture")
            .iter()
            .map(|value| value.to_str().unwrap())
            .collect();
        assert_eq!(values, ["first", "second"]);
        assert!(!result.request.follow_redirects);
        assert_eq!(result.format_ext, "flac");
        assert_eq!(result.requested_offset_ms, u64::MAX);
        assert_eq!(
            result.start,
            StartPosition::Calibrated {
                offset: Duration::from_millis(7200123),
                max_error: Duration::from_millis(17)
            }
        );
        assert_eq!(
            result.capabilities,
            PlaybackCapabilities {
                timeline: Timeline::Unknown,
                seek_policy: SeekPolicy::ServerOffset
            }
        );
        assert!(result.pin.is_none());
        assert_eq!(releases.load(Ordering::SeqCst), 0);
        drop(result);
        assert_eq!(releases.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn conversion_failure_releases_once_without_echoing_header_content() {
        let releases = Arc::new(AtomicUsize::new(0));
        let count = releases.clone();
        let mut response = response();
        response.headers[0].value = "synthetic-secret\r\n".into();
        let result = from_response(
            response,
            PlaybackLease::new(move || {
                count.fetch_add(1, Ordering::SeqCst);
            }),
            true,
        );
        assert_eq!(
            result.err().unwrap().to_string(),
            "invalid extension playback descriptor"
        );
        assert_eq!(releases.load(Ordering::SeqCst), 1);
    }
}
