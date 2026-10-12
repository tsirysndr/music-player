use music_player_entity::track::Model as Track;
use music_player_types::{
    audio::{AudioPin, AudioSelection},
    source::{ResourceKind, SourceHandle},
};
use serde::{Deserialize, Serialize};

/// One occurrence, including when an identical track appears elsewhere in the
/// queue. The flattened metadata keeps old queue snapshots readable.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueueEntry {
    #[serde(default = "new_occurrence_id")]
    pub occurrence_id: String,
    #[serde(flatten)]
    pub track: Track,
    #[serde(default)]
    pub selection: AudioSelection,
    #[serde(default)]
    pub pin: Option<AudioPin>,
}

fn new_occurrence_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

impl QueueEntry {
    pub fn new(track: Track) -> Self {
        Self {
            occurrence_id: new_occurrence_id(),
            track,
            selection: AudioSelection::Auto,
            pin: None,
        }
    }

    pub fn effective_selection(&self) -> AudioSelection {
        self.pin
            .clone()
            .map(AudioSelection::Pinned)
            .unwrap_or_else(|| self.selection.clone())
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if uuid::Uuid::parse_str(&self.occurrence_id).is_err() {
            return Err("queue occurrence identity is invalid");
        }
        if matches!(self.selection, AudioSelection::Pinned(_)) {
            return Err("an accepted pin is not a requested audio choice");
        }
        if self.selection != AudioSelection::Auto
            || self.pin.is_some()
            || SourceHandle::is_handle(&self.track.id)
            || SourceHandle::is_handle(&self.track.uri)
        {
            let source = SourceHandle::parse(&self.track.uri)
                .map_err(|_| "audio choices require a stable media source")?;
            if source.kind() != ResourceKind::Item || self.track.id != source.to_handle() {
                return Err("audio choices require a playable source identity");
            }
            if matches!(source, SourceHandle::Local(_))
                && (self.selection != AudioSelection::Auto || self.pin.is_some())
            {
                return Err("source v1 does not support audio selection");
            }
        }
        if let AudioSelection::Explicit {
            media_source_id,
            audio_stream_index,
        } = &self.selection
        {
            if media_source_id.is_empty() || *audio_stream_index < 0 {
                return Err("audio choice has an invalid version or stream index");
            }
        }
        if let Some(pin) = &self.pin {
            if pin.media_source_id.is_empty() || pin.audio_stream_index < 0 {
                return Err("accepted audio pin is invalid");
            }
            if let AudioSelection::Explicit {
                media_source_id,
                audio_stream_index,
            } = &self.selection
            {
                if &pin.media_source_id != media_source_id
                    || pin.audio_stream_index != *audio_stream_index
                {
                    return Err("accepted audio pin differs from the requested choice");
                }
            }
        }
        Ok(())
    }
}
