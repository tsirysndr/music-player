//! Container navigation keeps the account and the original Emby metadata.
#[cfg(test)]
mod selection_tests;

use super::{objects::track::Track, provider};
use crate::source_resolver;
use async_graphql::{Context, Error, InputObject, Object, SimpleObject, ID};
use music_player_playback::player::PlayerCommand;
use music_player_provider::{ConnectedProvider, MediaEntry as ProviderEntry, Page};
use music_player_renderer::CurrentReceiverDevice;
use music_player_tracklist::{QueueEntry, Tracklist};
use music_player_types::{audio, source::SourceHandle};
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::{mpsc::UnboundedSender, oneshot, Mutex};

#[derive(SimpleObject)]
pub struct AudioOptions {
    pub source: ID,
    pub versions: Vec<AudioVersionOption>,
}

/// Omit both fields for automatic selection. Explicit choices always carry
/// both real identities; callers cannot submit an accepted playback pin.
#[derive(InputObject, Default)]
pub struct AudioChoiceInput {
    pub media_source_id: Option<String>,
    pub audio_stream_index: Option<i32>,
}

impl AudioChoiceInput {
    fn selection(self) -> Result<audio::AudioSelection, Error> {
        match (self.media_source_id, self.audio_stream_index) {
            (None, None) => Ok(audio::AudioSelection::Auto),
            (Some(id), Some(index)) if !id.trim().is_empty() && index >= 0 => {
                Ok(audio::AudioSelection::Explicit {
                    media_source_id: id,
                    audio_stream_index: index,
                })
            }
            _ => Err(Error::new(
                "An audio choice needs both a media version and a nonnegative stream index",
            )),
        }
    }
}

#[derive(InputObject)]
pub struct SelectedMediaInput {
    pub track: super::objects::track::TrackInput,
    pub choice: Option<AudioChoiceInput>,
}

#[derive(SimpleObject)]
pub struct AudioChoice {
    pub media_source_id: Option<String>,
    pub audio_stream_index: Option<i32>,
}

#[derive(SimpleObject)]
pub struct AcceptedAudio {
    pub media_source_id: String,
    pub audio_stream_index: i32,
    pub runtime_ticks: Option<String>,
    pub codec: Option<String>,
    pub channels: Option<u32>,
    pub sample_rate: Option<u32>,
}

#[derive(SimpleObject)]
pub struct MediaQueueEntry {
    pub occurrence_id: ID,
    pub track: Track,
    pub choice: AudioChoice,
    pub accepted_audio: Option<AcceptedAudio>,
}

impl From<QueueEntry> for MediaQueueEntry {
    fn from(entry: QueueEntry) -> Self {
        let (media_source_id, audio_stream_index) = match entry.selection {
            audio::AudioSelection::Explicit {
                media_source_id,
                audio_stream_index,
            } => (Some(media_source_id), Some(audio_stream_index)),
            _ => (None, None),
        };
        Self {
            occurrence_id: ID(entry.occurrence_id),
            track: entry.track.into(),
            choice: AudioChoice {
                media_source_id,
                audio_stream_index,
            },
            accepted_audio: entry.pin.map(|pin| AcceptedAudio {
                media_source_id: pin.media_source_id,
                audio_stream_index: pin.audio_stream_index,
                runtime_ticks: pin.runtime_ticks.map(|ticks| ticks.to_string()),
                codec: pin.codec,
                channels: pin.channels,
                sample_rate: pin.sample_rate,
            }),
        }
    }
}

#[derive(SimpleObject)]
pub struct MediaQueue {
    pub current: Option<MediaQueueEntry>,
    pub played: Vec<MediaQueueEntry>,
    pub upcoming: Vec<MediaQueueEntry>,
}

async fn command_reply<T>(reply: oneshot::Receiver<Result<T, String>>) -> Result<T, Error> {
    tokio::time::timeout(std::time::Duration::from_secs(5), reply)
        .await
        .map_err(|_| {
            Error::new("Player acknowledgement timed out; outcome uncertain. The delayed command may still apply. A refresh showing no change does not prove failure. Do not retry this mutation.")
        })?
        .map_err(|_| Error::new("Player acknowledgement channel closed; outcome uncertain. The command may have applied. A refresh showing no change does not prove failure. Do not retry this mutation."))?
        .map_err(Error::new)
}

#[derive(Default)]
pub struct MediaMutation;

#[Object]
impl MediaMutation {
    /// Insert a fresh occurrence and play it now, or place it next. Full browse
    /// metadata supports legacy sources that cannot resolve an item by ID.
    async fn play_selected_media(
        &self,
        ctx: &Context<'_>,
        entry: SelectedMediaInput,
        #[graphql(default = false)] next: bool,
    ) -> Result<ID, Error> {
        let selection = entry.choice.unwrap_or_default().selection()?;
        let mut track = entry.track.into();
        let device = ctx
            .data::<Arc<Mutex<CurrentReceiverDevice>>>()?
            .lock()
            .await;
        if device.client.is_some() {
            return Err(Error::new(
                "Selected media playback is unavailable on this remote receiver",
            ));
        }
        crate::validate_queue(source_resolver(ctx)?, std::slice::from_mut(&mut track)).await?;
        let mut candidate = QueueEntry::new(track.clone());
        candidate.selection = selection.clone();
        candidate.validate().map_err(Error::new)?;
        let (reply, received) = oneshot::channel();
        ctx.data::<Arc<StdMutex<UnboundedSender<PlayerCommand>>>>()?
            .lock()
            .unwrap()
            .send(PlayerCommand::PlaySelectedMedia {
                track,
                selection,
                next,
                reply,
            })
            .map_err(|_| Error::new("Player command channel closed"))?;
        command_reply(received).await.map(ID)
    }

    /// Returns newly allocated occurrence IDs in input order.
    async fn add_selected_media(
        &self,
        ctx: &Context<'_>,
        entries: Vec<SelectedMediaInput>,
    ) -> Result<Vec<ID>, Error> {
        let mut tracks = Vec::with_capacity(entries.len());
        let mut choices = Vec::with_capacity(entries.len());
        for entry in entries {
            tracks.push(entry.track.into());
            choices.push(entry.choice.unwrap_or_default().selection()?);
        }
        let device = ctx
            .data::<Arc<Mutex<CurrentReceiverDevice>>>()?
            .lock()
            .await;
        if device.client.is_some() {
            return Err(Error::new(
                "Audio choices are unavailable on this remote receiver",
            ));
        }
        crate::validate_queue(source_resolver(ctx)?, &mut tracks)
            .await
            .map_err(|error| Error::new(error.to_string()))?;
        // Check the entire batch before handing it to the command thread.
        for (track, choice) in tracks.iter().zip(&choices) {
            let mut entry = QueueEntry::new(track.clone());
            entry.selection = choice.clone();
            entry.validate().map_err(Error::new)?;
        }
        if tracks.is_empty() {
            return Ok(Vec::new());
        }
        let (reply, received) = oneshot::channel();
        ctx.data::<Arc<StdMutex<UnboundedSender<PlayerCommand>>>>()?
            .lock()
            .unwrap()
            .send(PlayerCommand::LoadSelectedTracks {
                tracks: tracks.into_iter().zip(choices).collect(),
                start_index: None,
                reply,
            })
            .map_err(|_| Error::new("Player command channel closed"))?;
        command_reply(received)
            .await
            .map(|ids| ids.into_iter().map(ID).collect())
    }

    async fn select_media_audio(
        &self,
        ctx: &Context<'_>,
        occurrence_id: ID,
        choice: AudioChoiceInput,
    ) -> Result<bool, Error> {
        let selection = choice.selection()?;
        let device = ctx
            .data::<Arc<Mutex<CurrentReceiverDevice>>>()?
            .lock()
            .await;
        if device.client.is_some() {
            return Err(Error::new(
                "Audio choices are unavailable on this remote receiver",
            ));
        }
        let (reply, received) = oneshot::channel();
        ctx.data::<Arc<StdMutex<UnboundedSender<PlayerCommand>>>>()?
            .lock()
            .unwrap()
            .send(PlayerCommand::SelectAudio {
                occurrence_id: occurrence_id.to_string(),
                selection,
                reply,
            })
            .map_err(|_| Error::new("Player command channel closed"))?;
        command_reply(received).await?;
        Ok(true)
    }
}

#[derive(SimpleObject)]
pub struct AudioVersionOption {
    pub id: String,
    pub name: Option<String>,
    /// Decimal text preserves exact Emby ticks.
    pub runtime_ticks: Option<String>,
    pub default_audio_stream_index: Option<i32>,
    pub unavailable_reason: Option<String>,
    pub audio_streams: Vec<AudioStreamOption>,
}

#[derive(SimpleObject)]
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

impl From<audio::AudioOptions> for AudioOptions {
    fn from(options: audio::AudioOptions) -> Self {
        Self {
            source: ID(options.source),
            versions: options
                .versions
                .into_iter()
                .map(|version| AudioVersionOption {
                    id: version.id,
                    name: version.name,
                    runtime_ticks: version.runtime_ticks.map(|ticks| ticks.to_string()),
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
        }
    }
}

#[derive(SimpleObject)]
pub struct MediaBrowser {
    pub server_id: ID,
    pub supported: bool,
}

#[derive(SimpleObject)]
pub struct MediaEntry {
    pub id: ID,
    pub title: String,
    pub item_type: Option<String>,
    pub media_type: Option<String>,
    pub is_container: bool,
    /// Decimal strings preserve indices larger than GraphQL Int / JS Number.
    pub season_number: Option<String>,
    pub episode_number: Option<String>,
    pub track: Option<Track>,
}

#[derive(SimpleObject)]
pub struct MediaPage {
    pub server_id: ID,
    pub entries: Vec<MediaEntry>,
    pub next_offset: Option<i32>,
}

impl From<ProviderEntry> for MediaEntry {
    fn from(entry: ProviderEntry) -> Self {
        Self {
            id: ID(entry.id),
            title: entry.title,
            item_type: entry.item_type,
            media_type: entry.media_type,
            is_container: entry.is_container,
            season_number: entry.season_number.map(|number| number.to_string()),
            episode_number: entry.episode_number.map(|number| number.to_string()),
            track: entry.track.map(Into::into),
        }
    }
}

// Capture one provider before awaiting HTTP. The response names that account;
// a client switching sources can discard the old response by its query key.
async fn current_for(ctx: &Context<'_>, server_id: &str) -> Result<ConnectedProvider, Error> {
    let current = provider::connected(ctx)
        .await
        .ok_or_else(|| Error::new("Connect a media server to browse its libraries"))?;
    if current.config.id != server_id {
        return Err(Error::new(
            "The browsing server changed; refresh its libraries",
        ));
    }
    if !current.provider.capabilities().media_browse {
        return Err(Error::new(
            "This server does not support media library navigation",
        ));
    }
    Ok(current)
}

#[derive(Default)]
pub struct MediaQuery;

#[Object]
impl MediaQuery {
    /// Queue identity is per occurrence, including duplicate media items.
    async fn media_queue(&self, ctx: &Context<'_>) -> Result<MediaQueue, Error> {
        let device = ctx
            .data::<Arc<Mutex<CurrentReceiverDevice>>>()?
            .lock()
            .await;
        if device.client.is_some() {
            return Err(Error::new(
                "Audio choices are unavailable on this remote receiver",
            ));
        }
        let queue = ctx.data::<Arc<StdMutex<Tracklist>>>()?.lock().unwrap();
        let (played, upcoming) = queue.entries();
        Ok(MediaQueue {
            current: queue.current_entry().map(Into::into),
            played: played.into_iter().map(Into::into).collect(),
            upcoming: upcoming.into_iter().map(Into::into).collect(),
        })
    }

    /// Resolve the account in the stable handle, independently of navigation.
    async fn media_audio_options(
        &self,
        ctx: &Context<'_>,
        source: ID,
    ) -> Result<AudioOptions, Error> {
        let source = SourceHandle::parse(&source).map_err(|error| Error::new(error.to_string()))?;
        source_resolver(ctx)?
            .audio_options_handle(&source)
            .await
            .map(Into::into)
            .map_err(provider::err)
    }

    async fn media_browser(&self, ctx: &Context<'_>) -> Option<MediaBrowser> {
        provider::connected(ctx).await.map(|current| MediaBrowser {
            server_id: ID(current.config.id),
            supported: current.provider.capabilities().media_browse,
        })
    }

    /// Root libraries or direct children. Containers are never playable rows.
    async fn browse_media(
        &self,
        ctx: &Context<'_>,
        server_id: ID,
        parent: Option<ID>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<MediaPage, Error> {
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(100);
        if offset < 0 || !(1..=500).contains(&limit) {
            return Err(Error::new(
                "Media pages need a nonnegative offset and a limit from 1 to 500",
            ));
        }
        let next = offset
            .checked_add(limit)
            .ok_or_else(|| Error::new("Media page offset is too large"))?;
        let current = current_for(ctx, &server_id).await?;
        let mut entries = current
            .provider
            .browse(
                parent.as_ref().map(|id| id.as_str()),
                Page::new(offset, limit + 1),
            )
            .await
            .map_err(provider::err)?;
        let next_offset = (entries.len() > limit as usize).then_some(next);
        entries.truncate(limit as usize);
        for entry in &mut entries {
            entry.track = entry
                .track
                .take()
                .map(|track| provider::decorate(track, &current.config));
        }
        Ok(MediaPage {
            server_id: ID(current.config.id),
            entries: entries.into_iter().map(Into::into).collect(),
            next_offset,
        })
    }

    /// Flatten a container for queueing. limit <= 0 requests every leaf; the
    /// caller receives the complete result or an error before changing a queue.
    async fn media_container_tracks(
        &self,
        ctx: &Context<'_>,
        server_id: ID,
        parent: ID,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<Track>, Error> {
        let current = current_for(ctx, &server_id).await?;
        let tracks = current
            .provider
            .container_tracks(&parent, provider::page(offset, limit))
            .await
            .map_err(provider::err)?;
        Ok(provider::decorate_all(tracks, &current.config)
            .into_iter()
            .map(Into::into)
            .collect())
    }
}
