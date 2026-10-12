//! Typed source-v2 boundary, not a scheduler or playback adapter.
//! All errors omit guest values and transport credentials. Busy is propagated
//! unchanged: that attempt did not execute. Other resolve errors must not be
//! blindly retried: execution/remote allocation may already have occurred.
//! The caller retains requestId cleanup ownership even for invalid responses or
//! an absent leaseId. Stale attempts still require cleanup on their original
//! instance; "still current" gates playback retries, not release obligations.
use super::Extension;
use crate::abi::source_v2::*;
use anyhow::{anyhow, Error};
use reqwest::header::{HeaderName, HeaderValue};
use std::collections::HashMap;

impl Extension {
    pub(crate) fn source_identity_v2(
        &self,
        request: &IdentityRequest,
    ) -> Result<SourceIdentity, Error> {
        let response: SourceIdentity = self.call_source_v2("source_identity_v2", request)?;
        identity(&response)?;
        Ok(response)
    }

    pub(crate) fn browse_media_v2(&self, request: &BrowseRequest) -> Result<BrowseResponse, Error> {
        identity(&request.identity)?;
        if let Some(parent) = &request.parent_id {
            nonempty(parent)?;
        }
        if request.limit == 0 {
            return Err(invalid("browse limit must be positive"));
        }
        let response: BrowseResponse = self.call_source_v2("browse_media_v2", request)?;
        browse_response(request, &response)?;
        Ok(response)
    }

    pub(crate) fn get_media_item_v2(&self, request: &ItemRequest) -> Result<ItemResponse, Error> {
        item_request(&request.identity, &request.item_id)?;
        let response: ItemResponse = self.call_source_v2("get_media_item_v2", request)?;
        media_item(&response.item)?;
        if response.item.item_id != request.item_id {
            return Err(invalid("item response identity mismatch"));
        }
        Ok(response)
    }

    pub(crate) fn get_audio_options_v2(
        &self,
        request: &AudioOptionsRequest,
    ) -> Result<AudioOptionsResponse, Error> {
        item_request(&request.identity, &request.item_id)?;
        let mut response: AudioOptionsResponse =
            self.call_source_v2("get_audio_options_v2", request)?;
        if response.item_id != request.item_id {
            return Err(invalid("options response identity mismatch"));
        }
        disable_ambiguous_candidates(&mut response);
        Ok(response)
    }

    /// No cleanup is performed here. Even a validation failure after execution
    /// leaves attempt-scoped cleanup with the caller (requestId, optional lease).
    pub(crate) fn resolve_audio_v2(
        &self,
        request: &ResolveRequest,
    ) -> Result<ResolveResponse, Error> {
        resolve_request(request)?;
        let response: ResolveResponse = self.call_source_v2("resolve_audio_v2", request)?;
        resolve_response(request, &response)?;
        Ok(response)
    }

    pub(crate) fn release_audio_v2(
        &self,
        request: &ReleaseRequest,
    ) -> Result<ReleaseResponse, Error> {
        item_request(&request.identity, &request.item_id)?;
        nonempty(&request.request_id)?;
        if let Some(lease) = &request.lease_id {
            nonempty(lease)?;
        }
        let response: ReleaseResponse = self.call_source_v2("release_audio_v2", request)?;
        if response.request_id != request.request_id || !response.released {
            return Err(invalid("release acknowledgement mismatch or refusal"));
        }
        Ok(response)
    }
}

fn invalid(message: &'static str) -> Error {
    anyhow!("source v2: {message}")
}
fn nonempty(value: &str) -> Result<(), Error> {
    if value.trim().is_empty() {
        Err(invalid("empty identity or request field"))
    } else {
        Ok(())
    }
}
fn identity(value: &SourceIdentity) -> Result<(), Error> {
    nonempty(&value.server_id)?;
    nonempty(&value.user_id)
}
fn item_request(remote: &SourceIdentity, item: &str) -> Result<(), Error> {
    identity(remote)?;
    nonempty(item)
}
fn media_item(item: &MediaItem) -> Result<(), Error> {
    nonempty(&item.item_id)?;
    if item.is_container && item.playable {
        return Err(invalid("container cannot be playable"));
    }
    Ok(())
}
fn browse_response(request: &BrowseRequest, response: &BrowseResponse) -> Result<(), Error> {
    if response.items.len() as u128 > u128::from(request.limit) {
        return Err(invalid("browse response exceeds requested limit"));
    }
    if response
        .next_offset
        .is_some_and(|next| next <= request.offset)
    {
        return Err(invalid("browse cursor did not advance"));
    }
    for item in &response.items {
        media_item(item)?;
    }
    Ok(())
}

/// Preserve original IDs, indices, labels and plugin reasons. Mark every member
/// of an ambiguous group, including when its duplicate was already unavailable.
/// Invalid advertised defaults are retained, as in native Emby; they do not make
/// an independently addressable explicit stream invalid.
fn disable_ambiguous_candidates(response: &mut AudioOptionsResponse) {
    let mut versions = HashMap::new();
    for version in &response.versions {
        *versions.entry(version.id.clone()).or_insert(0usize) += 1;
    }
    for version in &mut response.versions {
        if version.id.trim().is_empty() || versions[&version.id] > 1 {
            version
                .unavailable_reason
                .get_or_insert_with(|| "Invalid or ambiguous media source identity".into());
        }
        let mut indices = HashMap::new();
        for stream in &version.audio_streams {
            *indices.entry(stream.index).or_insert(0usize) += 1;
        }
        for stream in &mut version.audio_streams {
            if stream.index < 0 || indices[&stream.index] > 1 {
                stream
                    .unavailable_reason
                    .get_or_insert_with(|| "Invalid or ambiguous audio stream index".into());
            }
        }
        if !version
            .audio_streams
            .iter()
            .any(|s| s.unavailable_reason.is_none())
        {
            version
                .unavailable_reason
                .get_or_insert_with(|| "No selectable audio streams".into());
        }
    }
}
fn pin(value: &AudioPin) -> Result<(), Error> {
    nonempty(&value.media_source_id)?;
    if value.audio_stream_index < 0 {
        return Err(invalid("invalid selected audio index"));
    }
    Ok(())
}
fn resolve_request(request: &ResolveRequest) -> Result<(), Error> {
    item_request(&request.identity, &request.item_id)?;
    nonempty(&request.request_id)?;
    if request.format_exts.is_empty() || request.format_exts.iter().any(|s| s.trim().is_empty()) {
        return Err(invalid("no usable requested decoder format"));
    }
    match &request.selection {
        AudioSelection::Auto {} => (),
        AudioSelection::Explicit {
            media_source_id,
            audio_stream_index,
        } => {
            nonempty(media_source_id)?;
            if *audio_stream_index < 0 {
                return Err(invalid("invalid requested audio index"));
            }
        }
        AudioSelection::Pinned { pin: previous } => pin(previous)?,
    }
    Ok(())
}
fn resolve_response(request: &ResolveRequest, response: &ResolveResponse) -> Result<(), Error> {
    if response.request_id != request.request_id
        || response.requested_offset_ms != request.offset_ms
    {
        return Err(invalid("resolve attempt or requested offset mismatch"));
    }
    if !request.format_exts.contains(&response.format_ext) {
        return Err(invalid("unrequested decoder format"));
    }
    if response.seek_policy == SeekPolicy::None && request.offset_ms != 0 {
        return Err(invalid("source does not support the requested offset"));
    }
    // Reopening must preserve the first actual choice, not rerun Auto against
    // a possibly changed default. Timeline alone says nothing about selection.
    if response.seek_policy == SeekPolicy::ServerOffset && response.actual_pin.is_none() {
        return Err(invalid("server-offset source requires an actual audio pin"));
    }
    if let Some(actual) = &response.actual_pin {
        pin(actual)?;
    }
    match &request.selection {
        AudioSelection::Auto {} => (),
        AudioSelection::Explicit {
            media_source_id,
            audio_stream_index,
        } => {
            if !response.actual_pin.as_ref().is_some_and(|p| {
                p.media_source_id == *media_source_id && p.audio_stream_index == *audio_stream_index
            }) {
                return Err(invalid("actual selection does not match explicit choice"));
            }
        }
        AudioSelection::Pinned { pin: previous } => {
            // Native select compares the entire shared AudioPin, including
            // loss/addition of evidence (Some/None), not just source/index.
            if response.actual_pin.as_ref() != Some(previous) {
                return Err(invalid("pinned media evidence changed"));
            }
        }
    }
    if let Some(lease) = &response.lease_id {
        nonempty(lease)?;
    }
    // Syntax only. No origin allowlist, host/account policy or redirect arbitration.
    // Keep the original URL and ordered headers; parsing is not normalization.
    if response
        .url
        .chars()
        .any(|c| c.is_control() || c.is_whitespace())
    {
        return Err(invalid("invalid HTTP source URL"));
    }
    let url = reqwest::Url::parse(&response.url).map_err(|_| invalid("invalid HTTP source URL"))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(invalid("invalid HTTP source URL"));
    }
    for header in &response.headers {
        HeaderName::from_bytes(header.name.as_bytes())
            .map_err(|_| invalid("invalid HTTP header name"))?;
        HeaderValue::from_str(&header.value).map_err(|_| invalid("invalid HTTP header value"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::{HostContext, SourceInstanceBusy};
    use serde_json::{json, Value};
    use std::collections::BTreeMap;

    // Real Extism fixture: fixed JSON output for all six exports. No network,
    // decoder or scheduling claims; CI performs WAT compilation and execution.
    fn fixture(response: Value) -> (tempfile::TempDir, Extension) {
        let dir = tempfile::tempdir().unwrap();
        let bytes = serde_json::to_vec(&response).unwrap();
        let mut wat = String::from(
            r#"(module
          (import "extism:host/env" "alloc" (func $alloc (param i64) (result i64)))
          (import "extism:host/env" "store_u8" (func $store (param i64 i32)))
          (import "extism:host/env" "output_set" (func $output (param i64 i64)))
          (func $reply (result i32) (local $p i64)"#,
        );
        wat.push_str(&format!(
            "(local.set $p (call $alloc (i64.const {})))",
            bytes.len()
        ));
        for (i, byte) in bytes.iter().enumerate() {
            wat.push_str(&format!(
                "(call $store (i64.add (local.get $p) (i64.const {i})) (i32.const {byte}))"
            ));
        }
        wat.push_str(&format!(
            "(call $output (local.get $p) (i64.const {})) (i32.const 0))",
            bytes.len()
        ));
        for name in [
            "source_identity_v2",
            "browse_media_v2",
            "get_media_item_v2",
            "get_audio_options_v2",
            "resolve_audio_v2",
            "release_audio_v2",
        ] {
            wat.push_str(&format!(
                "(func (export {name:?}) (result i32) (call $reply))"
            ));
        }
        wat.push(')');
        std::fs::write(dir.path().join("plugin.wat"), wat).unwrap();
        std::fs::write(
            dir.path().join("plugin.json"),
            json!({"id":"fixture.typed","name":"Typed fixture",
            "capabilities":["source"],"sourceApiVersion":2,"entry":"plugin.wat"})
            .to_string(),
        )
        .unwrap();
        let extension =
            Extension::load(dir.path(), &BTreeMap::new(), HostContext::default()).unwrap();
        (dir, extension)
    }
    fn remote() -> SourceIdentity {
        SourceIdentity {
            server_id: "server".into(),
            user_id: "user".into(),
        }
    }
    fn request() -> ResolveRequest {
        ResolveRequest {
            request_id: "attempt".into(),
            identity: remote(),
            item_id: "item".into(),
            offset_ms: 7_200_123,
            selection: AudioSelection::Explicit {
                media_source_id: "version".into(),
                audio_stream_index: 0,
            },
            format_exts: vec!["mp3".into()],
            follow_redirects: true,
        }
    }
    fn response() -> Value {
        json!({"requestId":"attempt","requestedOffsetMs":7200123,
            "url":"https://fixture.invalid/audio?signature=synthetic%2Bvalue",
            "headers":[{"name":"Authorization","value":"synthetic-secret"}],
            "mimeType":"audio/mpeg","formatExt":"mp3",
            "actualPin":{"mediaSourceId":"version","audioStreamIndex":0,"runtimeTicks":u64::MAX,
                "etag":"revision","codec":"aac","channels":2,"sampleRate":44100},
            "start":{"type":"requestedOnly"},"timeline":{"type":"finite","durationTicks":u64::MAX},
            "seekPolicy":"serverOffset","cachePolicy":"noPersistentCache","leaseId":"synthetic-lease"})
    }

    #[test]
    fn typed_calls_validate_metadata_and_preserve_remote_identity() {
        let (_dir, extension) = fixture(json!({"serverId":"server","userId":"user"}));
        assert_eq!(
            extension.source_identity_v2(&IdentityRequest {}).unwrap(),
            remote()
        );
        let (_dir2, invalid) = fixture(json!({"serverId":"server","userId":" "}));
        assert!(invalid.source_identity_v2(&IdentityRequest {}).is_err());
        let page = json!({"items":[{"itemId":"season","title":"Original","seasonNumber":u64::MAX,
            "isContainer":true,"playable":false}],"nextOffset":12});
        let (_dir3, browser) = fixture(page);
        let req = BrowseRequest {
            identity: remote(),
            parent_id: None,
            query: None,
            offset: 10,
            limit: 1,
        };
        assert_eq!(
            browser.browse_media_v2(&req).unwrap().items[0].season_number,
            Some(u64::MAX)
        );
        let (_dir4, item) = fixture(
            json!({"item":{"itemId":"other","title":"Episode","isContainer":false,"playable":true}}),
        );
        assert!(item
            .get_media_item_v2(&ItemRequest {
                identity: remote(),
                item_id: "item".into()
            })
            .is_err());
        let (_dir5, options) = fixture(json!({"itemId":"other","versions":[]}));
        assert!(options
            .get_audio_options_v2(&AudioOptionsRequest {
                identity: remote(),
                item_id: "item".into()
            })
            .is_err());
    }

    #[test]
    fn rejects_overfull_nonadvancing_and_playable_container_pages() {
        let req = BrowseRequest {
            identity: remote(),
            parent_id: None,
            query: None,
            offset: 5,
            limit: 1,
        };
        let item = json!({"itemId":"i","title":"Title","isContainer":false,"playable":true});
        for page in [
            json!({"items":[item.clone(),item],"nextOffset":7}),
            json!({"items":[],"nextOffset":5}),
            json!({"items":[{"itemId":"c","title":"Series","isContainer":true,"playable":true}],"nextOffset":null}),
            json!({"items":[{"itemId":"","title":"bad","isContainer":false,"playable":false}],"nextOffset":null}),
        ] {
            let (_dir, ext) = fixture(page);
            assert!(ext.browse_media_v2(&req).is_err());
        }
    }

    #[test]
    fn candidate_ambiguity_disables_every_duplicate_without_rewriting_evidence() {
        let value = json!({"itemId":"item","versions":[
            {"id":"duplicate","audioStreams":[{"index":0,"isDefault":true}]},
            {"id":"duplicate","unavailableReason":"Plugin reason","audioStreams":[]},
            {"id":"valid","name":"Original","defaultAudioStreamIndex":0,"audioStreams":[
                {"index":0,"isDefault":true,"language":""},
                {"index":1,"isDefault":false}, {"index":1,"isDefault":false},
                {"index":-1,"isDefault":false}]}, {"id":"empty","audioStreams":[]}]});
        let (_dir, ext) = fixture(value);
        let options = ext
            .get_audio_options_v2(&AudioOptionsRequest {
                identity: remote(),
                item_id: "item".into(),
            })
            .unwrap();
        assert!(options.versions[0].unavailable_reason.is_some());
        assert_eq!(
            options.versions[1].unavailable_reason.as_deref(),
            Some("Plugin reason")
        );
        let valid = &options.versions[2];
        assert!(valid.unavailable_reason.is_none());
        assert_eq!(valid.default_audio_stream_index, Some(0));
        assert_eq!(valid.audio_streams[0].language.as_deref(), Some(""));
        assert!(valid.audio_streams[0].unavailable_reason.is_none());
        assert_eq!(
            valid
                .audio_streams
                .iter()
                .map(|s| s.index)
                .collect::<Vec<_>>(),
            vec![0, 1, 1, -1]
        );
        assert!(valid.audio_streams[1..]
            .iter()
            .all(|s| s.unavailable_reason.is_some()));
        assert!(options.versions[3].unavailable_reason.is_some());
    }

    #[test]
    fn explicit_zero_and_all_pinned_evidence_are_enforced() {
        let req = request();
        let (_dir, ext) = fixture(response());
        let actual = ext.resolve_audio_v2(&req).unwrap();
        assert_eq!(actual.actual_pin.as_ref().unwrap().audio_stream_index, 0);
        assert_eq!(actual.headers[0].value, "synthetic-secret");
        let pinned = ResolveRequest {
            selection: AudioSelection::Pinned {
                pin: actual.actual_pin.unwrap(),
            },
            ..req
        };
        assert!(ext.resolve_audio_v2(&pinned).is_ok());
        for (field, changed) in [
            ("runtimeTicks", json!(5)),
            ("etag", Value::Null),
            ("codec", json!("flac")),
            ("channels", json!(1)),
            ("sampleRate", json!(48000)),
            ("mediaSourceId", json!("other")),
            ("audioStreamIndex", json!(1)),
        ] {
            let mut value = response();
            value["actualPin"][field] = changed;
            let (_d, changed) = fixture(value);
            assert!(changed.resolve_audio_v2(&pinned).is_err(), "{field}");
        }
        let mut value = response();
        value["actualPin"] = Value::Null;
        let (_d, missing) = fixture(value);
        assert!(missing.resolve_audio_v2(&request()).is_err());
    }

    #[test]
    fn auto_server_offset_requires_a_pin_for_every_timeline() {
        for timeline in [
            json!({"type":"finite","durationTicks":191429666670u64}),
            json!({"type":"unknown"}),
            json!({"type":"live"}),
        ] {
            for offset_ms in [0, 7_200_123] {
                let req = ResolveRequest {
                    offset_ms,
                    selection: AudioSelection::Auto {},
                    ..request()
                };
                let mut value = response();
                value["requestedOffsetMs"] = json!(offset_ms);
                value["timeline"] = timeline.clone();
                value["actualPin"] = Value::Null;
                let (_dir, ext) = fixture(value);
                assert_eq!(
                    ext.resolve_audio_v2(&req).err().unwrap().to_string(),
                    "source v2: server-offset source requires an actual audio pin"
                );
            }
        }
    }

    #[test]
    fn auto_nonseekable_source_can_omit_pin() {
        for timeline in [
            json!({"type":"finite","durationTicks":191429666670u64}),
            json!({"type":"unknown"}),
            json!({"type":"live"}),
        ] {
            let req = ResolveRequest {
                offset_ms: 0,
                selection: AudioSelection::Auto {},
                ..request()
            };
            let mut value = response();
            value["requestedOffsetMs"] = json!(0);
            value["timeline"] = timeline;
            value["seekPolicy"] = json!("none");
            value["actualPin"] = Value::Null;
            let (_dir, ext) = fixture(value);
            assert!(ext.resolve_audio_v2(&req).unwrap().actual_pin.is_none());
        }
    }

    #[test]
    fn rejects_mismatched_attempt_offset_format_seek_and_bad_http_without_secrets() {
        for (field, changed) in [
            ("requestId", json!("other")),
            ("requestedOffsetMs", json!(0)),
            ("formatExt", json!("flac")),
            ("seekPolicy", json!("none")),
            ("url", json!("file:///synthetic-secret")),
            (
                "headers",
                json!([{"name":"bad name","value":"synthetic-secret"}]),
            ),
            (
                "headers",
                json!([{"name":"Authorization","value":"synthetic-secret\r\ninjected: value"}]),
            ),
        ] {
            let mut value = response();
            value[field] = changed;
            let (_dir, ext) = fixture(value);
            let error = ext.resolve_audio_v2(&request()).err().unwrap().to_string();
            assert!(!error.contains("synthetic-secret"));
        }
        let mut value = response();
        value["requestedOffsetMs"] = json!(0);
        value["seekPolicy"] = json!("none");
        let (_dir, ext) = fixture(value);
        let req = ResolveRequest {
            offset_ms: 0,
            ..request()
        };
        assert!(ext.resolve_audio_v2(&req).is_ok());
    }

    #[test]
    fn preflight_precedes_guest_admission_and_busy_retains_type() {
        // A regression to blocking lock acquisition must fail rather than hang
        // the test thread. Completion includes fixture/instance teardown.
        let (done, completed) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            preflight_scenario();
            done.send(()).unwrap();
        });
        completed
            .recv_timeout(std::time::Duration::from_secs(30))
            .expect("typed source admission or teardown exceeded watchdog");
        worker.join().unwrap();
    }

    fn preflight_scenario() {
        let (_dir, ext) = fixture(response());
        let held = ext.plugin.lock().unwrap();
        let err = ext.resolve_audio_v2(&request()).err().unwrap();
        assert!(err.downcast_ref::<SourceInstanceBusy>().is_some());
        let mut req = request();
        req.identity.user_id.clear();
        let err = ext.resolve_audio_v2(&req).err().unwrap();
        assert!(err.downcast_ref::<SourceInstanceBusy>().is_none());
        req = request();
        req.request_id.clear();
        assert!(ext
            .resolve_audio_v2(&req)
            .err()
            .unwrap()
            .downcast_ref::<SourceInstanceBusy>()
            .is_none());
        req = request();
        req.selection = AudioSelection::Explicit {
            media_source_id: "v".into(),
            audio_stream_index: -1,
        };
        assert!(ext
            .resolve_audio_v2(&req)
            .err()
            .unwrap()
            .downcast_ref::<SourceInstanceBusy>()
            .is_none());
        let page = BrowseRequest {
            identity: remote(),
            parent_id: None,
            query: None,
            offset: 0,
            limit: 0,
        };
        assert!(ext
            .browse_media_v2(&page)
            .unwrap_err()
            .downcast_ref::<SourceInstanceBusy>()
            .is_none());
        drop(held);
    }

    #[test]
    fn release_requires_matching_affirmative_ack_even_without_lease() {
        let req = ReleaseRequest {
            request_id: "attempt".into(),
            identity: remote(),
            item_id: "item".into(),
            lease_id: None,
        };
        for ack in [
            json!({"requestId":"other","released":true}),
            json!({"requestId":"attempt","released":false}),
        ] {
            let (_dir, ext) = fixture(ack);
            assert!(ext.release_audio_v2(&req).is_err());
        }
        let (_dir, ext) = fixture(json!({"requestId":"attempt","released":true}));
        assert!(ext.release_audio_v2(&req).unwrap().released);
    }
}
