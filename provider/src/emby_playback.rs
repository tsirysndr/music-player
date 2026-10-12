//! Resolve a stable Emby item into one transient, audio-only playback session.
//! The host commits `pin` only after accepting the resolve ticket.

use crate::{backends::emby::Emby, ProviderError};
use music_player_types::audio::{AudioOptions, AudioStreamOption, AudioVersionOption};
pub use music_player_types::audio::{AudioPin, AudioSelection};
use music_player_types::source::{ResourceKind, SourceRef};
use reqwest::{header::HeaderMap, Method};
use serde::Deserialize;
use std::sync::Arc;
use url::Url;

pub struct ResolvedAudio {
    pub url: Url,
    pub headers: HeaderMap,
    pub follow_redirects: bool,
    pub pin: AudioPin,
    /// Requested only: the server's actual start still needs calibration.
    pub requested_offset_ms: u64,
    pub lease: EncodingLease,
}

impl std::fmt::Debug for ResolvedAudio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResolvedAudio")
            .field("pin", &self.pin)
            .field("requested_offset_ms", &self.requested_offset_ms)
            .finish_non_exhaustive()
    }
}

/// Owns only this resolution's encoding. Drop schedules cleanup without
/// blocking stop/pause. A successful DELETE is not proof of encoder exit.
pub struct EncodingLease {
    client: Arc<Emby>,
    session_id: Option<String>,
    runtime: tokio::runtime::Handle,
}
impl EncodingLease {
    pub async fn release(mut self) -> Result<(), ProviderError> {
        let Some(session) = self.session_id.as_deref() else {
            return Ok(());
        };
        let result = cleanup(&self.client, session).await;
        self.session_id = None;
        result
    }
}
impl Drop for EncodingLease {
    fn drop(&mut self) {
        let Some(session) = self.session_id.take() else {
            return;
        };
        let client = self.client.clone();
        self.runtime.spawn(async move {
            if let Err(error) = cleanup(&client, &session).await {
                tracing::warn!(%error, "could not release Emby playback encoding");
            }
        });
    }
}

// Total per-attempt timeout includes all redirect hops. Cleanup is independent
// of the playback command loop and targets only this lease's session.
async fn cleanup(client: &Emby, session: &str) -> Result<(), ProviderError> {
    use std::time::Duration;
    let mut last = failure("Emby encoding cleanup timed out");
    for attempt in 0..3 {
        match tokio::time::timeout(Duration::from_secs(5), client.delete_encoding(session)).await {
            Ok(Ok(())) => return Ok(()),
            Ok(Err(error)) => last = error,
            Err(_) => last = failure("Emby encoding cleanup timed out"),
        }
        if attempt < 2 {
            tokio::time::sleep(Duration::from_millis(100 * (attempt + 1))).await;
        }
    }
    Err(last)
}

impl Emby {
    /// Read candidates without creating a playback session. Candidates describe
    /// this metadata snapshot; resolution checks the selected identities again.
    pub async fn audio_options(&self, source: &SourceRef) -> Result<AudioOptions, ProviderError> {
        self.check_reference(&source.to_handle(), ResourceKind::Item)?;
        let item = self.item(source).await?;
        if !item.is_media_leaf() {
            return Err(failure(
                "Emby audio options require an Audio or Video leaf item",
            ));
        }
        Ok(AudioOptions {
            source: source.to_handle(),
            versions: item
                .media_sources
                .iter()
                .map(|version| version.options(&item.media_sources))
                .collect(),
        })
    }

    pub async fn resolve_audio(
        self: &Arc<Self>,
        source: &SourceRef,
        selection: &AudioSelection,
        offset_ms: u64,
    ) -> Result<ResolvedAudio, ProviderError> {
        self.check_reference(&source.to_handle(), ResourceKind::Item)?;
        let offset_ticks = offset_ms
            .checked_mul(10_000)
            .and_then(|ticks| i64::try_from(ticks).ok())
            .ok_or_else(|| failure("Emby seek offset is out of range"))?;
        let item = self.item(source).await?;
        if !item.is_media_leaf() {
            return Err(failure("Emby containers cannot be played as audio"));
        }
        let runtime = tokio::runtime::Handle::try_current()
            .map_err(|_| failure("Emby playback requires an active async runtime"))?;
        let info: PlaybackInfo = self
            .json(
                Method::GET,
                &["Items", &source.item_id, "PlaybackInfo"],
                &[
                    ("UserId", self.identity().user_id.clone()),
                    ("DeviceId", self.device_id().to_owned()),
                ],
                None,
            )
            .await?;
        // Establish ownership before fallible selection and URL construction.
        let session_id = info
            .play_session_id
            .filter(|id| !id.is_empty())
            .ok_or_else(|| failure("Emby did not issue a playback session identity"))?;
        let lease = EncodingLease {
            client: self.clone(),
            session_id: Some(session_id.clone()),
            runtime,
        };
        if info.error_code.is_some() {
            return Err(failure("Emby refused audio playback resolution"));
        }
        let pin = select(&info.media_sources, selection)?;
        if offset_ticks > 0 && pin.runtime_ticks.is_none() {
            return Err(failure(
                "this Emby source has no confirmed finite seek timeline",
            ));
        }
        if pin
            .runtime_ticks
            .is_some_and(|duration| offset_ticks as u64 >= duration)
        {
            return Err(failure("Emby seek target is outside this source"));
        }
        let url = self.endpoint(
            &["Audio", &source.item_id, "stream.mp3"],
            &[
                ("UserId", self.identity().user_id.clone()),
                ("MediaSourceId", pin.media_source_id.clone()),
                ("AudioStreamIndex", pin.audio_stream_index.to_string()),
                ("AudioCodec", "mp3".into()),
                ("EnableAutoStreamCopy", "false".into()),
                ("StartTimeTicks", offset_ticks.to_string()),
                ("DeviceId", self.device_id().to_owned()),
                ("PlaySessionId", session_id),
            ],
        )?;
        Ok(ResolvedAudio {
            url,
            headers: self.playback_headers(),
            follow_redirects: self.follows_redirects(),
            pin,
            requested_offset_ms: offset_ms,
            lease,
        })
    }
}

fn select(sources: &[MediaSource], selection: &AudioSelection) -> Result<AudioPin, ProviderError> {
    let wanted = match selection {
        AudioSelection::Auto => None,
        AudioSelection::Explicit {
            media_source_id,
            audio_stream_index,
        } => Some((media_source_id.as_str(), *audio_stream_index)),
        AudioSelection::Pinned(pin) => Some((pin.media_source_id.as_str(), pin.audio_stream_index)),
    };
    let source = match wanted {
        Some((id, _)) => sources.iter().find(|source| source.id == id),
        None => sources
            .iter()
            .find(|source| {
                !source.requires_opening
                    && !source.requires_closing
                    && !source.is_infinite_stream
                    && source.media_streams.iter().any(|stream| stream.is_audio())
            })
            .or_else(|| {
                sources
                    .iter()
                    .find(|source| source.media_streams.iter().any(|stream| stream.is_audio()))
            }),
    }
    .ok_or_else(|| failure("the selected Emby media source is unavailable"))?;
    source.check_source(sources)?;
    let index = wanted
        .map(|(_, index)| index)
        .or(source.default_audio_stream_index);
    let audio = if let Some(index) = index {
        source
            .media_streams
            .iter()
            .find(|stream| stream.is_audio() && stream.index == index)
    } else {
        source
            .media_streams
            .iter()
            .find(|stream| stream.is_audio() && stream.is_default)
            .or_else(|| source.media_streams.iter().find(|stream| stream.is_audio()))
    }
    .ok_or_else(|| failure("the selected Emby audio stream is unavailable"))?;
    source.check_stream(audio)?;
    let pin = AudioPin {
        media_source_id: source.id.clone(),
        audio_stream_index: audio.index,
        runtime_ticks: source.run_time_ticks,
        etag: source.etag.clone(),
        codec: audio.codec.clone(),
        channels: audio.channels,
        sample_rate: audio.sample_rate,
    };
    if let AudioSelection::Pinned(previous) = selection {
        if previous != &pin {
            return Err(failure(
                "Emby media changed; the saved playback position requires review",
            ));
        }
    }
    Ok(pin)
}
fn failure(message: &str) -> ProviderError {
    ProviderError::Other(message.into())
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PlaybackInfo {
    #[serde(default)]
    media_sources: Vec<MediaSource>,
    play_session_id: Option<String>,
    error_code: Option<serde_json::Value>,
}
#[derive(Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct MediaSource {
    // Required in JSON: missing/null identity must not become a selectable ID.
    id: String,
    name: Option<String>,
    #[serde(default)]
    media_streams: Vec<MediaStream>,
    default_audio_stream_index: Option<i32>,
    run_time_ticks: Option<u64>,
    #[serde(rename = "ETag")]
    etag: Option<String>,
    #[serde(default)]
    requires_opening: bool,
    #[serde(default)]
    requires_closing: bool,
    #[serde(default)]
    is_infinite_stream: bool,
}
#[derive(Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MediaStream {
    // Required in JSON: absent Index must never silently mean audio index zero.
    index: i32,
    #[serde(rename = "Type")]
    stream_type: String,
    title: Option<String>,
    display_title: Option<String>,
    language: Option<String>,
    codec: Option<String>,
    channels: Option<u32>,
    sample_rate: Option<u32>,
    #[serde(default)]
    is_default: bool,
}
impl MediaStream {
    fn is_audio(&self) -> bool {
        self.index >= 0 && self.stream_type == "Audio"
    }
}

impl MediaSource {
    // These checks serve both the candidate UI and the final resolver.
    fn check_source(&self, sources: &[Self]) -> Result<(), ProviderError> {
        if self.id.trim().is_empty()
            || sources.iter().filter(|other| other.id == self.id).count() != 1
        {
            return Err(failure(
                "Emby media source identity is missing or ambiguous",
            ));
        }
        if self.requires_opening || self.requires_closing || self.is_infinite_stream {
            return Err(ProviderError::Unsupported {
                kind: "emby",
                feature: "live or open/close playback sessions",
            });
        }
        Ok(())
    }

    fn check_stream(&self, audio: &MediaStream) -> Result<(), ProviderError> {
        if !audio.is_audio() {
            return Err(failure("Emby audio stream index is invalid"));
        }
        if self
            .media_streams
            .iter()
            .filter(|stream| stream.stream_type == "Audio" && stream.index == audio.index)
            .count()
            != 1
        {
            return Err(failure("Emby audio stream identity is ambiguous"));
        }
        Ok(())
    }

    fn options(&self, sources: &[Self]) -> AudioVersionOption {
        let mut unavailable_reason = self.check_source(sources).err().map(|e| e.to_string());
        let audio_streams: Vec<_> = self
            .media_streams
            .iter()
            // Invalid audio indices remain visible but disabled, never renumbered.
            .filter(|stream| stream.stream_type == "Audio")
            .map(|stream| AudioStreamOption {
                index: stream.index,
                title: stream.title.clone(),
                display_title: stream.display_title.clone(),
                language: stream.language.clone(),
                codec: stream.codec.clone(),
                channels: stream.channels,
                sample_rate: stream.sample_rate,
                is_default: stream.is_default,
                unavailable_reason: unavailable_reason
                    .clone()
                    .or_else(|| self.check_stream(stream).err().map(|e| e.to_string())),
            })
            .collect();
        if unavailable_reason.is_none()
            && !audio_streams
                .iter()
                .any(|stream| stream.unavailable_reason.is_none())
        {
            unavailable_reason = Some("Emby media source has no selectable audio streams".into());
        }
        AudioVersionOption {
            id: self.id.clone(),
            name: self.name.clone(),
            runtime_ticks: self.run_time_ticks,
            default_audio_stream_index: self.default_audio_stream_index,
            unavailable_reason,
            audio_streams,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target() -> MediaSource {
        serde_json::from_str(r#"{
            "Id":"mediasource_55508", "RunTimeTicks":191429666670,
            "DefaultAudioStreamIndex":1,
            "MediaStreams":[
                {"Index":0,"Type":"Video","Codec":"h264"},
                {"Index":1,"Type":"Audio","Codec":"aac","IsDefault":false,"Channels":2,"SampleRate":44100}
            ]
        }"#).unwrap()
    }

    #[test]
    fn target_uses_actual_default_audio_index_even_when_stream_is_not_default() {
        let pin = select(&[target()], &AudioSelection::Auto).unwrap();
        assert_eq!(pin.audio_stream_index, 1);
        assert_eq!(pin.media_source_id, "mediasource_55508");
        assert_eq!(pin.runtime_ticks, Some(191429666670));
    }

    #[test]
    fn resume_keeps_pin_when_defaults_change_and_rejects_changed_timeline() {
        let mut source = target();
        let pin = select(&[source.clone()], &AudioSelection::Auto).unwrap();
        let mut other_audio = source.media_streams[1].clone();
        other_audio.index = 2;
        other_audio.is_default = true;
        source.media_streams.push(other_audio);
        source.default_audio_stream_index = Some(2);
        assert_eq!(
            select(&[source.clone()], &AudioSelection::Pinned(pin.clone())).unwrap(),
            pin
        );
        source.run_time_ticks = Some(123);
        assert!(select(&[source], &AudioSelection::Pinned(pin)).is_err());
    }

    #[test]
    fn index_zero_is_audio_when_metadata_says_audio_and_missing_pin_never_falls_back() {
        let mut source = target();
        source.media_streams.remove(0);
        source.media_streams[0].index = 0;
        source.default_audio_stream_index = Some(0);
        let pin = select(&[source.clone()], &AudioSelection::Auto).unwrap();
        assert_eq!(pin.audio_stream_index, 0);
        source.media_streams[0].index = 1;
        source.default_audio_stream_index = Some(1);
        assert!(select(&[source], &AudioSelection::Pinned(pin)).is_err());
    }

    #[test]
    fn unsupported_sessions_and_ambiguous_selection_are_explicit() {
        let original = target();
        let mut opening = original.clone();
        opening.requires_opening = true;
        assert!(select(&[opening], &AudioSelection::Auto).is_err());
        assert!(select(&[original.clone(), original.clone()], &AudioSelection::Auto).is_err());
        let mut duplicate = original.clone();
        duplicate
            .media_streams
            .push(original.media_streams[1].clone());
        assert!(select(&[duplicate], &AudioSelection::Auto).is_err());
        assert!(select(
            &[original],
            &AudioSelection::Explicit {
                media_source_id: "missing".into(),
                audio_stream_index: 1
            }
        )
        .is_err());
    }

    #[test]
    fn automatic_selection_can_use_a_supported_version_but_explicit_selection_never_substitutes() {
        let supported = target();
        let mut unsupported = supported.clone();
        unsupported.id = "version-needing-open".into();
        unsupported.requires_opening = true;
        let sources = [unsupported, supported];
        assert_eq!(
            select(&sources, &AudioSelection::Auto)
                .unwrap()
                .media_source_id,
            "mediasource_55508"
        );
        assert!(select(
            &sources,
            &AudioSelection::Explicit {
                media_source_id: "version-needing-open".into(),
                audio_stream_index: 1,
            }
        )
        .is_err());
    }

    #[test]
    fn candidate_availability_agrees_with_explicit_resolution() {
        let sources: Vec<MediaSource> = serde_json::from_value(serde_json::json!([
            {"Id":"mixed","DefaultAudioStreamIndex":9,"MediaStreams":[
                {"Type":"Video","Index":0}, {"Type":"Audio","Index":0},
                {"Type":"Audio","Index":-1},
                {"Type":"Audio","Index":2}, {"Type":"Audio","Index":2}
            ]},
            {"Id":"live","IsInfiniteStream":true,"MediaStreams":[{"Type":"Audio","Index":0}]},
            {"Id":"open","RequiresOpening":true,"MediaStreams":[{"Type":"Audio","Index":0}]},
            {"Id":"close","RequiresClosing":true,"MediaStreams":[{"Type":"Audio","Index":0}]},
            {"Id":"duplicate","MediaStreams":[{"Type":"Audio","Index":0}]},
            {"Id":"duplicate","MediaStreams":[{"Type":"Audio","Index":1}]},
            {"Id":"","MediaStreams":[{"Type":"Audio","Index":0}]},
            {"Id":" \t","MediaStreams":[{"Type":"Audio","Index":0}]},
            {"Id":"empty","MediaStreams":[]},
            {"Id":"invalid-only","MediaStreams":[{"Type":"Audio","Index":-1}]}
        ]))
        .unwrap();
        let mut selectable = 0;
        for source in &sources {
            let option = source.options(&sources);
            for stream in &option.audio_streams {
                let advertised =
                    option.unavailable_reason.is_none() && stream.unavailable_reason.is_none();
                let result = select(
                    &sources,
                    &AudioSelection::Explicit {
                        media_source_id: option.id.clone(),
                        audio_stream_index: stream.index,
                    },
                );
                assert_eq!(advertised, result.is_ok());
                selectable += usize::from(advertised);
            }
            if option.audio_streams.is_empty() {
                assert!(option.unavailable_reason.is_some());
            }
        }
        assert_eq!(selectable, 1);
        // An invalid advertised default is preserved, not silently replaced;
        // an explicit valid index remains selectable independently of it.
        assert_eq!(
            sources[0].options(&sources).default_audio_stream_index,
            Some(9)
        );
    }
}
