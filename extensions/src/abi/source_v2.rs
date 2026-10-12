//! Source v2 JSON wire contract. No saved-account ID or authoritative SourceRef
//! is supplied by the guest. The host binds remote IDs to its saved account.
//! Semantic validation (identity, selected pin, offsets, URLs and capabilities)
//! belongs to the adapter; successful deserialization alone does not authorize play.
//! No implicit Auto, zero start, seek capability, or persistent cache fallback.
use serde::{Deserialize, Deserializer, Serialize};

// Unlike ordinary Option fields, these nullable fields must occur on the wire.
fn required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::deserialize(deserializer)
}
fn default_follow_redirects() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IdentityRequest {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceIdentity {
    pub server_id: String,
    pub user_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BrowseRequest {
    pub identity: SourceIdentity,
    pub parent_id: Option<String>,
    pub query: Option<String>,
    pub offset: u64,
    /// Host supplies a positive bounded page size; no implicit unlimited request.
    pub limit: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BrowseResponse {
    pub items: Vec<MediaItem>,
    /// Explicit null means exhausted. Missing is an invalid response.
    #[serde(deserialize_with = "required_nullable")]
    pub next_offset: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemRequest {
    pub identity: SourceIdentity,
    pub item_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemResponse {
    pub item: MediaItem,
}

/// Persistent metadata only: never a transport URL, headers, lease or host handle.
/// Containers are not playable; a non-container may still have no audio.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaItem {
    pub item_id: String,
    pub title: String,
    pub item_type: Option<String>,
    pub media_type: Option<String>,
    pub is_container: bool,
    pub playable: bool,
    pub season_number: Option<u64>,
    pub episode_number: Option<u64>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub year: Option<u32>,
    pub track_number: Option<u32>,
    /// 100ns ticks, never floating-point seconds.
    pub runtime_ticks: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioOptionsRequest {
    pub identity: SourceIdentity,
    pub item_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioOptionsResponse {
    pub item_id: String,
    /// Empty is meaningful; do not manufacture a default version or stream.
    pub versions: Vec<AudioVersionOption>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioVersionOption {
    pub id: String,
    pub name: Option<String>,
    pub runtime_ticks: Option<u64>,
    pub default_audio_stream_index: Option<i32>,
    pub unavailable_reason: Option<String>,
    pub audio_streams: Vec<AudioStreamOption>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioStreamOption {
    pub index: i32,
    pub title: Option<String>,
    pub display_title: Option<String>,
    pub language: Option<String>,
    pub codec: Option<String>,
    pub channels: Option<u32>,
    pub sample_rate: Option<u32>,
    pub is_default: bool,
    pub unavailable_reason: Option<String>,
}

/// Same evidence fields as types::audio::AudioPin, independently camelCase encoded.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioPin {
    pub media_source_id: String,
    pub audio_stream_index: i32,
    pub runtime_ticks: Option<u64>,
    pub etag: Option<String>,
    pub codec: Option<String>,
    pub channels: Option<u32>,
    pub sample_rate: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AudioSelection {
    // Empty struct variants enforce deny_unknown_fields for tag-only objects.
    Auto {},
    Explicit {
        media_source_id: String,
        audio_stream_index: i32,
    },
    Pinned {
        pin: AudioPin,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolveRequest {
    /// Unique per resolve attempt; also identifies cleanup when no lease returned.
    pub request_id: String,
    pub identity: SourceIdentity,
    pub item_id: String,
    pub offset_ms: u64,
    pub selection: AudioSelection,
    /// Decoder extension hints (e.g. mp3), not MIME types or URL suffix guesses.
    pub format_exts: Vec<String>,
    #[serde(default = "default_follow_redirects")]
    pub follow_redirects: bool,
}

/// Deliberately no Debug: transient credential-bearing transport data.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

/// Kept in memory only, never persisted in queue metadata or diagnostic Debug.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolveResponse {
    pub request_id: String,
    pub requested_offset_ms: u64,
    pub url: String,
    /// Ordered structured headers; not a JSON-encoded string.
    pub headers: Vec<HttpHeader>,
    pub mime_type: String,
    pub format_ext: String,
    /// Null is explicit and only valid for Auto with SeekPolicy::None.
    /// ServerOffset always requires a pin so reopening cannot change selection;
    /// Explicit/Pinned requests require matching evidence in the host adapter.
    #[serde(deserialize_with = "required_nullable")]
    pub actual_pin: Option<AudioPin>,
    pub start: StartPosition,
    pub timeline: Timeline,
    pub seek_policy: SeekPolicy,
    pub cache_policy: CachePolicy,
    pub lease_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum StartPosition {
    /// Only the requested position is known; this is not calibrated evidence.
    RequestedOnly {},
    Calibrated {
        offset_ms: u64,
        max_error_ms: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Timeline {
    Finite { duration_ticks: u64 },
    Live {},
    Unknown {},
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SeekPolicy {
    ServerOffset,
    None,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CachePolicy {
    NoPersistentCache,
}

/// Release only resources owned by this attempt; idempotent even without leaseId.
/// Deliberately no Debug: opaque lease identifiers may themselves be sensitive.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReleaseRequest {
    pub request_id: String,
    pub identity: SourceIdentity,
    pub item_id: String,
    pub lease_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReleaseResponse {
    pub request_id: String,
    /// A required affirmative ack (including already released). Errors must fail
    /// the call; the adapter must reject false or a mismatched requestId.
    pub released: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn tag_only<T>(tag: &str, expected: T, conflicting_field: &str)
    where
        T: serde::de::DeserializeOwned + Serialize + PartialEq + std::fmt::Debug,
    {
        let value = json!({"type": tag});
        assert_eq!(
            serde_json::from_value::<T>(value.clone()).unwrap(),
            expected
        );
        assert_eq!(serde_json::to_value(expected).unwrap(), value);
        for field in [conflicting_field, "unexpected"] {
            let mut bad = value.clone();
            bad[field] = json!(0);
            assert!(serde_json::from_value::<T>(bad).is_err(), "{tag}: {field}");
        }
    }

    #[test]
    fn tag_only_variants_reject_extra_evidence_and_keep_wire_shape() {
        tag_only("auto", AudioSelection::Auto {}, "audioStreamIndex");
        tag_only("requestedOnly", StartPosition::RequestedOnly {}, "offsetMs");
        tag_only(
            "requestedOnly",
            StartPosition::RequestedOnly {},
            "maxErrorMs",
        );
        tag_only("live", Timeline::Live {}, "durationTicks");
        tag_only("unknown", Timeline::Unknown {}, "durationTicks");
    }

    fn request() -> Value {
        json!({"requestId":"attempt-1", "identity":{"serverId":"s","userId":"u"},
            "itemId":"i", "offsetMs":u64::MAX, "formatExts":["mp3"],
            "selection":{"type":"explicit","mediaSourceId":"version","audioStreamIndex":0}})
    }

    #[test]
    fn selection_zero_and_u64_max_survive_wire_without_defaults() {
        let input = request();
        let decoded: ResolveRequest = serde_json::from_value(input.clone()).unwrap();
        assert!(decoded.follow_redirects);
        assert_eq!(decoded.offset_ms, u64::MAX);
        assert_eq!(
            serde_json::to_value(&decoded).unwrap()["selection"],
            input["selection"]
        );
        for key in ["offsetMs", "selection", "formatExts", "identity", "itemId"] {
            let mut bad = input.clone();
            bad.as_object_mut().unwrap().remove(key);
            assert!(
                serde_json::from_value::<ResolveRequest>(bad).is_err(),
                "{key}"
            );
        }
        for invalid in [
            json!({}),
            json!({"type":"auto","audioStreamIndex":0}),
            json!({"type":"explicit","mediaSourceId":"v"}),
            json!({"type":"future"}),
        ] {
            assert!(serde_json::from_value::<AudioSelection>(invalid).is_err());
        }
        for offset in [json!(-1), json!(1.5), json!(18446744073709551616.0)] {
            let mut bad = input.clone();
            bad["offsetMs"] = offset;
            assert!(serde_json::from_value::<ResolveRequest>(bad).is_err());
        }
        let mut disabled = input;
        disabled["followRedirects"] = json!(false);
        assert!(
            !serde_json::from_value::<ResolveRequest>(disabled)
                .unwrap()
                .follow_redirects
        );
    }

    #[test]
    fn pin_and_candidates_keep_all_evidence_and_empty_language() {
        let pin = json!({"mediaSourceId":"v", "audioStreamIndex":0,"runtimeTicks":u64::MAX,
            "etag":"e","codec":"aac","channels":2,"sampleRate":44100});
        let selection = json!({"type":"pinned","pin":pin});
        let decoded: AudioSelection = serde_json::from_value(selection.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), selection);
        let candidates = json!({"itemId":"i", "versions":[{"id":"v","name":"Original",
            "runtimeTicks":u64::MAX,"defaultAudioStreamIndex":0,"unavailableReason":null,
            "audioStreams":[{"index":0,"title":"Original title","displayTitle":"Stereo",
            "language":"","codec":"aac","channels":2,"sampleRate":44100,
            "isDefault":true,"unavailableReason":null}]}]});
        let decoded: AudioOptionsResponse = serde_json::from_value(candidates.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), candidates);
        assert!(serde_json::from_value::<AudioOptionsResponse>(
            json!({"itemId":"i","versions":[],"source":"mp-source:guest"})
        )
        .is_err());
    }

    #[test]
    fn pagination_must_explicitly_end_and_metadata_cannot_carry_transport() {
        assert!(serde_json::from_value::<BrowseResponse>(json!({"items":[]})).is_err());
        assert!(
            serde_json::from_value::<BrowseResponse>(json!({"items":[],"nextOffset":null}))
                .unwrap()
                .next_offset
                .is_none()
        );
        let page: BrowseResponse = serde_json::from_value(json!({"nextOffset":u64::MAX,
            "items":[{"itemId":"season","title":"Original","isContainer":true,
            "playable":false,"seasonNumber":u64::MAX}]}))
        .unwrap();
        assert_eq!(page.items[0].season_number, Some(u64::MAX));
        let mut bad = serde_json::to_value(&page.items[0]).unwrap();
        bad["url"] = json!("https://fixture.invalid/temporary");
        assert!(serde_json::from_value::<MediaItem>(bad).is_err());
    }

    #[test]
    fn response_requires_start_policy_and_structured_headers() {
        let response = json!({"requestId":"attempt-1","requestedOffsetMs":u64::MAX,
            "url":"https://fixture.invalid/audio", "headers":[{"name":"Authorization","value":"synthetic"}],
            "mimeType":"audio/mpeg","formatExt":"mp3","actualPin":null,
            "start":{"type":"calibrated","offsetMs":u64::MAX,"maxErrorMs":1},
            "timeline":{"type":"finite","durationTicks":u64::MAX},
            "seekPolicy":"serverOffset","cachePolicy":"noPersistentCache","leaseId":"opaque"});
        let decoded: ResolveResponse = serde_json::from_value(response.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), response);
        for key in [
            "start",
            "timeline",
            "actualPin",
            "requestedOffsetMs",
            "headers",
            "seekPolicy",
            "cachePolicy",
        ] {
            let mut bad = response.clone();
            bad.as_object_mut().unwrap().remove(key);
            assert!(
                serde_json::from_value::<ResolveResponse>(bad).is_err(),
                "{key}"
            );
        }
        let mut bad = response;
        bad["headers"] = json!("{}");
        assert!(serde_json::from_value::<ResolveResponse>(bad).is_err());
        assert!(
            serde_json::from_value::<StartPosition>(json!({"type":"calibrated","offsetMs":0}))
                .is_err()
        );
        assert!(serde_json::from_value::<SeekPolicy>(json!("randomAccess")).is_err());
        assert!(serde_json::from_value::<CachePolicy>(json!("persistent")).is_err());
    }

    #[test]
    fn cleanup_without_returned_lease_still_identifies_attempt_and_requires_ack() {
        let request: ReleaseRequest = serde_json::from_value(json!({"requestId":"attempt-1",
            "identity":{"serverId":"s","userId":"u"},"itemId":"i"}))
        .unwrap();
        assert!(request.lease_id.is_none());
        for empty in ["", "null", "{}", r#"{"requestId":"attempt-1"}"#] {
            assert!(serde_json::from_str::<ReleaseResponse>(empty).is_err());
        }
        let ack: ReleaseResponse =
            serde_json::from_value(json!({"requestId":"attempt-1","released":true})).unwrap();
        assert!(ack.released);
    }
}
