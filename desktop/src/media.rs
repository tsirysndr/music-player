//! Media navigation owns its pending reads; playback remains in the daemon.
use std::{cell::RefCell, sync::Arc, time::Duration};

use music_player_server::api::{
    metadata::v1alpha1::Track,
    music::v1alpha1::{
        library_service_client::LibraryServiceClient,
        tracklist_service_client::TracklistServiceClient, AddSelectedMediaRequest,
        AddTracksRequest, BrowseMediaRequest, GetMediaAudioOptionsRequest,
        GetMediaAudioOptionsResponse, GetMediaBrowserRequest, GetMediaContainerTracksRequest,
        GetMediaQueueRequest, GetMediaQueueResponse, LoadTracksRequest, MediaEntry,
        MediaQueueChoice, MediaQueueOccurrence, SelectMediaAudioRequest, SelectedMediaEntry,
    },
};
use music_player_types::source::{ResourceKind, SourceRef};
use slint::{ComponentHandle, ModelRc, VecModel, Weak};
use tokio::sync::{mpsc::UnboundedSender, watch};
use tonic::{transport::Channel, Code, Status};

use crate::{rpc::Cmd, AppWindow, MediaCrumb, MediaItem};

const PAGE: i32 = 100;

#[derive(Clone, Copy, Debug, PartialEq)]
enum RequestPhase {
    Pending,
    Submitted,
    Cancelled,
}

#[derive(Clone, Debug)]
pub struct Ticket(Arc<watch::Sender<RequestPhase>>);

impl Ticket {
    pub(crate) fn cancel_pending(&self) -> bool {
        self.transition_pending(RequestPhase::Cancelled)
    }
    fn new() -> Self {
        Self(Arc::new(watch::channel(RequestPhase::Pending).0))
    }
    fn current(&self) -> bool {
        *self.0.borrow() != RequestPhase::Cancelled
    }
    fn cancel(&self) {
        self.0.send_replace(RequestPhase::Cancelled);
    }
    fn transition_pending(&self, next: RequestPhase) -> bool {
        self.0.send_if_modified(|phase| {
            if *phase != RequestPhase::Pending {
                return false;
            }
            *phase = next;
            true
        })
    }
    async fn cancelled(&self) {
        let mut receiver = self.0.subscribe();
        while *receiver.borrow_and_update() != RequestPhase::Cancelled {
            if receiver.changed().await.is_err() {
                return;
            }
        }
    }
}

#[derive(Clone, Debug)]
struct Location {
    server: Option<String>,
    // The root has no parent handle. Titles remain unmodified display data.
    path: Vec<(Option<String>, String)>,
    offset: i32,
}

impl Default for Location {
    fn default() -> Self {
        Self {
            server: None,
            path: vec![(None, "Libraries".into())],
            offset: 0,
        }
    }
}

#[derive(Default)]
struct State {
    location: Location,
    rows: Vec<MediaEntry>,
    next: Option<i32>,
    active: Option<Ticket>,
    audio: Option<AudioPage>,
    occurrences: Option<Vec<OccurrenceRow>>,
}

impl State {
    /// Retire UI ownership in both phases. Submitted work may still apply on
    /// the daemon, but neither its result nor pending reads may reopen this UI.
    fn leave(&mut self) {
        if let Some(ticket) = self.active.take() {
            ticket.cancel();
        }
        *self = Self::default();
    }
}

#[derive(Clone, Debug)]
struct OccurrenceRow {
    entry: MediaQueueOccurrence,
    section: &'static str,
}

impl OccurrenceRow {
    fn source(&self) -> Result<String, Status> {
        if self.entry.occurrence_id.is_empty() {
            return Err(Status::data_loss("Missing queue occurrence identity"));
        }
        let track = self
            .entry
            .track
            .as_ref()
            .ok_or_else(|| Status::data_loss("Missing queue metadata"))?;
        let source = SourceRef::parse(&track.id).map_err(|_| {
            Status::failed_precondition("Audio editing requires a saved-account media source")
        })?;
        if source.kind != ResourceKind::Item || track.uri != source.to_handle() {
            return Err(Status::failed_precondition(
                "Audio editing requires a playable source identity",
            ));
        }
        Ok(source.to_handle())
    }

    fn description(&self) -> String {
        let requested = match self.entry.choice.as_ref() {
            None => "Automatic".into(),
            Some(choice) => match (&choice.media_source_id, choice.audio_stream_index) {
                (None, None) => "Automatic".into(),
                (Some(id), Some(index)) => format!("{id} / stream {index}"),
                _ => "Invalid requested choice".into(),
            },
        };
        let accepted = self
            .entry
            .accepted_audio
            .as_ref()
            .map(|pin| {
                format!(
                    "{} / audio {} · {} · {} channels · {} Hz",
                    pin.media_source_id,
                    pin.audio_stream_index,
                    pin.codec.as_deref().unwrap_or("unknown codec"),
                    pin.channels
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "unknown".into()),
                    pin.sample_rate
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "unknown".into()),
                )
            })
            .unwrap_or_else(|| "Not yet played".into());
        format!(
            "{} · Selected: {} · Playback audio: {}",
            self.section, requested, accepted
        )
    }
}

fn occurrence_rows(snapshot: GetMediaQueueResponse) -> Vec<OccurrenceRow> {
    let current_id = snapshot.current.as_ref().map(|e| e.occurrence_id.clone());
    let mut rows = Vec::new();
    if let Some(entry) = snapshot.current {
        rows.push(OccurrenceRow {
            entry,
            section: "Current",
        });
    }
    for entry in snapshot.played.into_iter().rev() {
        if Some(&entry.occurrence_id) != current_id.as_ref() {
            rows.push(OccurrenceRow {
                entry,
                section: "History",
            });
        }
    }
    rows.extend(snapshot.upcoming.into_iter().map(|entry| OccurrenceRow {
        entry,
        section: "Upcoming",
    }));
    rows
}

struct AudioRow {
    title: String,
    subtitle: String,
    choice: Option<MediaQueueChoice>,
    enabled: bool,
}

struct AudioPage {
    track: Track,
    rows: Vec<AudioRow>,
    target: Option<OccurrenceRow>,
}

fn audio_page(track: Track, options: GetMediaAudioOptionsResponse) -> AudioPage {
    let mut rows = Vec::new();
    for version in options.versions {
        let version_label = version
            .name
            .as_deref()
            .filter(|v| !v.is_empty())
            .unwrap_or(&version.id);
        if version.audio_streams.is_empty() {
            rows.push(AudioRow {
                title: version_label.to_owned(),
                subtitle: version
                    .unavailable_reason
                    .clone()
                    .unwrap_or_else(|| "No audio streams".into()),
                choice: None,
                enabled: false,
            });
        }
        for stream in version.audio_streams {
            let mut labels = vec![format!("Version: {}", version.id)];
            if let Some(title) = stream.title {
                labels.push(title);
            }
            if let Some(display) = stream.display_title {
                labels.push(display);
            }
            labels.push(format!("Stream {}", stream.index));
            labels.push(match stream.language {
                Some(language) if language.is_empty() => "Language: (empty)".into(),
                Some(language) => format!("Language: {language}"),
                None => "Language: unspecified".into(),
            });
            if let Some(codec) = stream.codec {
                labels.push(codec);
            }
            if let Some(channels) = stream.channels {
                labels.push(format!("{channels} channels"));
            }
            if let Some(rate) = stream.sample_rate {
                labels.push(format!("{rate} Hz"));
            }
            if stream.is_default {
                labels.push("Stream default".into());
            }
            if version.default_audio_stream_index == Some(stream.index) {
                labels.push("Version default".into());
            }
            let reason = version
                .unavailable_reason
                .as_ref()
                .or(stream.unavailable_reason.as_ref());
            if let Some(reason) = reason {
                labels.push(reason.clone());
            }
            rows.push(AudioRow {
                title: version_label.to_owned(),
                subtitle: labels.join(" · "),
                enabled: reason.is_none() && !version.id.trim().is_empty() && stream.index >= 0,
                choice: Some(MediaQueueChoice {
                    media_source_id: Some(version.id.clone()),
                    audio_stream_index: Some(stream.index),
                }),
            });
        }
    }
    // Auto remains intent, not a fabricated media source or stream.
    if rows.iter().any(|row| row.enabled) {
        rows.insert(
            0,
            AudioRow {
                title: "Automatic choice".into(),
                subtitle: "Use the default audio when playback starts and keep it for resume."
                    .into(),
                choice: None,
                enabled: true,
            },
        );
    }
    AudioPage {
        track,
        rows,
        target: None,
    }
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

#[derive(Debug)]
enum Action {
    Occurrences,
    OccurrenceOptions(OccurrenceRow),
    SelectOccurrence {
        occurrence_id: String,
        choice: Option<MediaQueueChoice>,
    },
    AudioOptions(MediaEntry),
    AddSelected {
        track: Track,
        choice: Option<MediaQueueChoice>,
    },
    Browse(Location),
    Queue {
        server: String,
        entry: MediaEntry,
        play: bool,
    },
}

#[derive(Debug)]
pub struct Request {
    ticket: Ticket,
    action: Action,
    tx: UnboundedSender<Cmd>,
}

#[derive(Debug)]
pub struct Commit {
    ticket: Ticket,
    server: String,
    tracks: Vec<Track>,
    play: bool,
}

impl Commit {
    pub(crate) fn ticket(&self) -> Ticket {
        self.ticket.clone()
    }
}

impl Request {
    pub(crate) fn mutation_ticket(&self) -> Option<Ticket> {
        matches!(
            &self.action,
            Action::Queue { .. } | Action::AddSelected { .. } | Action::SelectOccurrence { .. }
        )
        .then(|| self.ticket.clone())
    }
}

pub(crate) fn show_superseded(weak: &Weak<AppWindow>, ticket: Ticket) {
    let _ = weak.upgrade_in_event_loop(move |app| {
        let owns_ui = STATE.with(|state| {
            state
                .borrow()
                .active
                .as_ref()
                .is_some_and(|active| Arc::ptr_eq(&active.0, &ticket.0))
        });
        if owns_ui {
            invalidate(&app);
            app.set_media_message(
                "The pending media request was cancelled by a newer playback command.".into(),
            );
        }
    });
}

enum Reply {
    Occurrences(GetMediaQueueResponse),
    SelectedOccurrence(Result<GetMediaQueueResponse, Status>),
    OccurrenceOptions {
        target: OccurrenceRow,
        options: GetMediaAudioOptionsResponse,
    },
    AudioOptions {
        entry: MediaEntry,
        options: GetMediaAudioOptionsResponse,
    },
    AddedSelected(Vec<String>),
    Page {
        location: Location,
        entries: Vec<MediaEntry>,
        next: Option<i32>,
    },
    Queue {
        server: String,
        tracks: Vec<Track>,
        play: bool,
    },
}

fn client(channel: &Channel) -> LibraryServiceClient<Channel> {
    LibraryServiceClient::new(channel.clone())
        .max_decoding_message_size(music_player_server::LIBRARY_MESSAGE_LIMIT)
}

fn message(error: Status) -> String {
    if error.code() == Code::Unimplemented {
        "This daemon does not support the requested media operation. Update it; no automatic playback fallback was used."
            .into()
    } else {
        error.message().to_owned()
    }
}

fn subtitle(entry: &MediaEntry) -> String {
    let mut parts = Vec::new();
    if let Some(kind) = &entry.item_type {
        parts.push(kind.clone());
    }
    if let Some(season) = entry.season_number {
        parts.push(format!("S{season}"));
    }
    if let Some(episode) = entry.episode_number {
        parts.push(format!("E{episode}"));
    }
    if !entry.is_container && entry.track.is_none() {
        parts.push("No audio track".into());
    }
    parts.join(" · ")
}

fn render(app: &AppWindow, state: &State) {
    app.set_media_editing_occurrence(state.audio.as_ref().is_some_and(|a| a.target.is_some()));
    if let Some(audio) = &state.audio {
        app.set_media_items(ModelRc::new(VecModel::from(
            audio
                .rows
                .iter()
                .map(|row| MediaItem {
                    id: "".into(),
                    title: row.title.clone().into(),
                    subtitle: row.subtitle.clone().into(),
                    is_container: false,
                    playable: row.enabled,
                    is_audio_option: true,
                    is_occurrence: false,
                })
                .collect::<Vec<_>>(),
        )));
        app.set_media_has_next(false);
        app.set_media_has_previous(false);
        app.set_media_offset_text(format!("Audio options · {}", audio.track.title).into());
        return;
    }
    if let Some(rows) = &state.occurrences {
        app.set_media_items(ModelRc::new(VecModel::from(
            rows.iter()
                .map(|row| {
                    let error = row.source().err();
                    MediaItem {
                        id: row.entry.occurrence_id.clone().into(),
                        title: row
                            .entry
                            .track
                            .as_ref()
                            .map(|t| t.title.clone())
                            .unwrap_or_else(|| "Missing track".into())
                            .into(),
                        subtitle: format!(
                            "{}{}",
                            row.description(),
                            error
                                .as_ref()
                                .map(|e| format!(" · {}", e.message()))
                                .unwrap_or_default()
                        )
                        .into(),
                        is_container: false,
                        playable: error.is_none(),
                        is_audio_option: false,
                        is_occurrence: true,
                    }
                })
                .collect::<Vec<_>>(),
        )));
        app.set_media_crumbs(ModelRc::new(VecModel::from(vec![MediaCrumb {
            title: "Libraries".into(),
        }])));
        app.set_media_has_next(false);
        app.set_media_has_previous(false);
        app.set_media_offset_text("Queue audio choices".into());
        return;
    }
    app.set_media_items(ModelRc::new(VecModel::from(
        state
            .rows
            .iter()
            .map(|entry| MediaItem {
                id: entry.id.clone().into(),
                title: entry.title.clone().into(),
                subtitle: subtitle(entry).into(),
                is_container: entry.is_container,
                playable: !entry.is_container && entry.track.is_some(),
                is_audio_option: false,
                is_occurrence: false,
            })
            .collect::<Vec<_>>(),
    )));
    app.set_media_crumbs(ModelRc::new(VecModel::from(
        state
            .location
            .path
            .iter()
            .map(|(_, title)| MediaCrumb {
                title: title.clone().into(),
            })
            .collect::<Vec<_>>(),
    )));
    app.set_media_has_next(state.next.is_some());
    app.set_media_has_previous(state.location.offset > 0);
    app.set_media_offset_text(if state.rows.is_empty() {
        "".into()
    } else {
        format!(
            "{}–{}",
            i64::from(state.location.offset) + 1,
            i64::from(state.location.offset) + state.rows.len() as i64
        )
        .into()
    });
}

/// Called synchronously by every provider/daemon switch callback, before
/// its command enters the worker queue. A -> B -> A still cancels the old A.
pub fn invalidate(app: &AppWindow) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.leave();
        render(app, &state);
    });
    app.set_media_loading(false);
    app.set_media_busy(false);
    app.set_media_cancellable(false);
    app.set_media_message("Refresh to browse the connected server's media libraries.".into());
}

fn start(app: &AppWindow, tx: &UnboundedSender<Cmd>, action: Action) {
    let ticket = Ticket::new();
    let loading = matches!(
        &action,
        Action::Browse(_)
            | Action::AudioOptions(_)
            | Action::Occurrences
            | Action::OccurrenceOptions(_)
    );
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if let Some(previous) = state.active.replace(ticket.clone()) {
            previous.cancel();
        }
        if let Action::Browse(location) = &action {
            state.occurrences = None;
            state.audio = None;
            state.location = location.clone();
            state.rows.clear();
            state.next = None;
            render(app, &state);
        }
        if matches!(&action, Action::Occurrences) {
            state.audio = None;
            state.occurrences = Some(Vec::new());
            render(app, &state);
        }
    });
    app.set_media_loading(loading);
    app.set_media_busy(!loading);
    app.set_media_cancellable(true);
    app.set_media_message("".into());
    if tx
        .send(Cmd::Media(Request {
            ticket,
            action,
            tx: tx.clone(),
        }))
        .is_err()
    {
        app.set_media_loading(false);
        app.set_media_busy(false);
        app.set_media_cancellable(false);
        app.set_media_message("The connection worker is unavailable.".into());
    }
}

pub fn bind(app: &AppWindow, tx: &UnboundedSender<Cmd>) {
    {
        let weak = app.as_weak();
        app.on_media_leave(move || invalidate(&weak.unwrap()));
    }
    {
        let weak = app.as_weak();
        let tx = tx.clone();
        app.on_media_refresh(move || {
            let action = STATE.with(|s| {
                if s.borrow().occurrences.is_some() {
                    Action::Occurrences
                } else {
                    Action::Browse(Location::default())
                }
            });
            start(&weak.unwrap(), &tx, action)
        });
    }
    {
        let weak = app.as_weak();
        let tx = tx.clone();
        app.on_media_queue_choices(move || start(&weak.unwrap(), &tx, Action::Occurrences));
    }
    {
        let weak = app.as_weak();
        let tx = tx.clone();
        app.on_media_audio_options(move |index| {
            let action = STATE.with(|s| {
                let state = s.borrow();
                let index = usize::try_from(index).ok()?;
                if state.audio.is_some() {
                    return None;
                }
                if let Some(rows) = &state.occurrences {
                    return rows.get(index).cloned().map(Action::OccurrenceOptions);
                }
                state
                    .rows
                    .get(index)
                    .filter(|e| !e.is_container && e.track.is_some())
                    .cloned()
                    .map(Action::AudioOptions)
            });
            if let Some(action) = action {
                start(&weak.unwrap(), &tx, action);
            }
        });
    }
    {
        let weak = app.as_weak();
        let tx = tx.clone();
        app.on_media_open(move |index| {
            let location = STATE.with(|state| {
                let state = state.borrow();
                let row = state.rows.get(usize::try_from(index).ok()?)?;
                if !row.is_container {
                    return None;
                }
                let mut location = state.location.clone();
                location
                    .path
                    .push((Some(row.id.clone()), row.title.clone()));
                location.offset = 0;
                Some(location)
            });
            if let Some(location) = location {
                start(&weak.unwrap(), &tx, Action::Browse(location));
            }
        });
    }
    {
        let weak = app.as_weak();
        let tx = tx.clone();
        app.on_media_breadcrumb(move |index| {
            let location = STATE.with(|state| {
                let state = state.borrow();
                let index = usize::try_from(index).ok()?;
                if state.occurrences.is_some() {
                    return Some(Location::default());
                }
                if index >= state.location.path.len() {
                    return None;
                }
                let mut location = state.location.clone();
                location.path.truncate(index + 1);
                location.offset = 0;
                Some(location)
            });
            if let Some(location) = location {
                start(&weak.unwrap(), &tx, Action::Browse(location));
            }
        });
    }
    {
        let weak = app.as_weak();
        let tx = tx.clone();
        app.on_media_page(move |forward| {
            let location = STATE.with(|state| {
                let state = state.borrow();
                let mut location = state.location.clone();
                location.offset = if forward {
                    state.next?
                } else {
                    (location.offset - PAGE).max(0)
                };
                Some(location)
            });
            if let Some(location) = location {
                start(&weak.unwrap(), &tx, Action::Browse(location));
            }
        });
    }
    {
        let weak = app.as_weak();
        let tx = tx.clone();
        app.on_media_play(move |index| queue(&weak.unwrap(), &tx, index, true));
    }
    {
        let weak = app.as_weak();
        let tx = tx.clone();
        app.on_media_enqueue(move |index| queue(&weak.unwrap(), &tx, index, false));
    }
    {
        let weak = app.as_weak();
        app.on_media_cancel(move || {
            let cancelled = STATE.with(|state| {
                let mut state = state.borrow_mut();
                if !state
                    .active
                    .as_ref()
                    .is_some_and(|ticket| ticket.transition_pending(RequestPhase::Cancelled))
                {
                    return false;
                }
                state.active = None;
                true
            });
            let app = weak.unwrap();
            if !cancelled {
                app.set_media_message("The queue request has already been sent.".into());
                return;
            }
            app.set_media_loading(false);
            app.set_media_busy(false);
            app.set_media_cancellable(false);
            app.set_media_message("Cancelled.".into());
        });
    }
    invalidate(app);
}

fn queue(app: &AppWindow, tx: &UnboundedSender<Cmd>, index: i32, play: bool) {
    let action = STATE.with(|state| {
        let state = state.borrow();
        if let Some(audio) = &state.audio {
            if play {
                return None;
            }
            return selected_action(audio, index);
        }
        if state.occurrences.is_some() {
            return None;
        }
        let entry = state.rows.get(usize::try_from(index).ok()?)?.clone();
        browse_action(entry, state.location.server.clone(), play)
    });
    if let Some(action) = action {
        start(app, tx, action);
    }
}

fn browse_action(entry: MediaEntry, server: Option<String>, play: bool) -> Option<Action> {
    if (entry.is_container && play) || (!entry.is_container && entry.track.is_none()) {
        return None;
    }
    if !entry.is_container && !play {
        return Some(Action::AudioOptions(entry));
    }
    Some(Action::Queue {
        server: server?,
        entry,
        play,
    })
}

fn selected_action(audio: &AudioPage, index: i32) -> Option<Action> {
    let row = audio.rows.get(usize::try_from(index).ok()?)?;
    if !row.enabled {
        return None;
    }
    Some(match &audio.target {
        Some(target) => Action::SelectOccurrence {
            occurrence_id: target.entry.occurrence_id.clone(),
            choice: row.choice.clone(),
        },
        None => Action::AddSelected {
            track: audio.track.clone(),
            choice: row.choice.clone(),
        },
    })
}

async fn fetch(
    channel: &Channel,
    action: Action,
    ticket: &Ticket,
    weak: Weak<AppWindow>,
) -> Result<Reply, Status> {
    let mut library = client(channel);
    match action {
        Action::Occurrences => read_occurrences(channel).await.map(Reply::Occurrences),
        Action::OccurrenceOptions(target) => {
            let source = target.source()?;
            let options = tokio::time::timeout(
                Duration::from_secs(35),
                library.get_media_audio_options(GetMediaAudioOptionsRequest {
                    source: source.clone(),
                }),
            )
            .await
            .map_err(|_| Status::deadline_exceeded("Reading audio options timed out."))??
            .into_inner();
            if options.source != source {
                return Err(Status::failed_precondition(
                    "Audio options belong to another source.",
                ));
            }
            Ok(Reply::OccurrenceOptions { target, options })
        }
        Action::SelectOccurrence {
            occurrence_id,
            choice,
        } => {
            if !ticket.transition_pending(RequestPhase::Submitted) {
                return Err(Status::cancelled("Cancelled"));
            }
            let submitted = ticket.clone();
            let _ = weak.upgrade_in_event_loop(move |app| {
                if submitted.current() {
                    app.set_media_cancellable(false);
                }
            });
            let mut tracklist = TracklistServiceClient::new(channel.clone());
            tokio::time::timeout(Duration::from_secs(15), tracklist.select_media_audio(
                SelectMediaAudioRequest { occurrence_id, choice }
            )).await.map_err(|_| Status::deadline_exceeded(
                "The queue result is unknown. The delayed command may still apply. A refresh showing no change does not prove failure. Do not retry this mutation."
            ))??;
            // The mutation was acknowledged. A later read failure must never
            // look like a failed mutation or invite submitting it again.
            Ok(Reply::SelectedOccurrence(read_occurrences(channel).await))
        }
        Action::AudioOptions(entry) => {
            let options = tokio::time::timeout(
                Duration::from_secs(35),
                library.get_media_audio_options(GetMediaAudioOptionsRequest {
                    source: entry.id.clone(),
                }),
            )
            .await
            .map_err(|_| Status::deadline_exceeded("Reading audio options timed out."))??
            .into_inner();
            if options.source != entry.id {
                return Err(Status::failed_precondition(
                    "Audio options belong to another source.",
                ));
            }
            Ok(Reply::AudioOptions { entry, options })
        }
        Action::AddSelected { track, choice } => {
            if !ticket.transition_pending(RequestPhase::Submitted) {
                return Err(Status::cancelled("Cancelled"));
            }
            let submitted = ticket.clone();
            let _ = weak.upgrade_in_event_loop(move |app| {
                if submitted.current() {
                    app.set_media_cancellable(false);
                }
            });
            let mut tracklist = TracklistServiceClient::new(channel.clone());
            let response = tokio::time::timeout(
                Duration::from_secs(15),
                tracklist.add_selected_media(AddSelectedMediaRequest {
                    entries: vec![SelectedMediaEntry {
                        track: Some(track),
                        choice,
                    }],
                }),
            )
            .await
            .map_err(|_| {
                Status::deadline_exceeded(
                    "The queue result is unknown. The delayed command may still apply. A refresh showing no change does not prove failure. Do not retry this mutation.",
                )
            })??
            .into_inner();
            if response.occurrence_ids.len() != 1 {
                return Err(Status::data_loss(
                    "Unexpected queue acknowledgement; outcome uncertain. The command may have applied. A refresh showing no change does not prove failure. Do not retry this mutation.",
                ));
            }
            Ok(Reply::AddedSelected(response.occurrence_ids))
        }
        Action::Browse(mut location) => {
            let browser = library
                .get_media_browser(GetMediaBrowserRequest {})
                .await?
                .into_inner()
                .browser
                .ok_or_else(|| {
                    Status::failed_precondition(
                        "Connect an Emby server to browse its media libraries.",
                    )
                })?;
            if !browser.supported {
                return Err(Status::failed_precondition(
                    "This server does not support media library navigation.",
                ));
            }
            if location
                .server
                .as_ref()
                .is_some_and(|id| id != &browser.server_id)
            {
                return Err(Status::failed_precondition(
                    "The browsing server changed. Refresh its libraries.",
                ));
            }
            let response = library
                .browse_media(BrowseMediaRequest {
                    server_id: browser.server_id.clone(),
                    parent: location.path.last().and_then(|(id, _)| id.clone()),
                    offset: location.offset,
                    limit: Some(PAGE),
                })
                .await?
                .into_inner();
            if response.server_id != browser.server_id {
                return Err(Status::failed_precondition(
                    "The browsing server changed. Refresh its libraries.",
                ));
            }
            location.server = Some(response.server_id);
            Ok(Reply::Page {
                location,
                entries: response.entries,
                next: response.next_offset,
            })
        }
        Action::Queue {
            server,
            entry,
            play,
        } => {
            let tracks = if entry.is_container {
                let response = library
                    .get_media_container_tracks(GetMediaContainerTracksRequest {
                        server_id: server.clone(),
                        parent: entry.id,
                        offset: 0,
                        limit: Some(0),
                    })
                    .await?
                    .into_inner();
                if response.server_id != server {
                    return Err(Status::failed_precondition(
                        "The browsing server changed. Refresh its libraries.",
                    ));
                }
                response.tracks
            } else {
                vec![entry
                    .track
                    .ok_or_else(|| Status::failed_precondition("This item has no audio track."))?]
            };
            Ok(Reply::Queue {
                server,
                tracks,
                play,
            })
        }
    }
}

async fn read_occurrences(channel: &Channel) -> Result<GetMediaQueueResponse, Status> {
    let mut tracklist = TracklistServiceClient::new(channel.clone())
        .max_decoding_message_size(music_player_server::LIBRARY_MESSAGE_LIMIT);
    Ok(tokio::time::timeout(
        Duration::from_secs(15),
        tracklist.get_media_queue(GetMediaQueueRequest {}),
    )
    .await
    .map_err(|_| Status::deadline_exceeded("Reading the media queue timed out."))??
    .into_inner())
}

fn show_occurrences(app: &AppWindow, snapshot: GetMediaQueueResponse) {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.audio = None;
        state.occurrences = Some(occurrence_rows(snapshot));
        render(app, &state);
    });
}

/// Runs as a separate task so a slow container read cannot block pause/stop.
pub async fn run(channel: Channel, weak: Weak<AppWindow>, request: Request) {
    let Request { ticket, action, tx } = request;
    let result = tokio::select! {
        biased;
        _ = ticket.cancelled() => return,
        result = tokio::time::timeout(Duration::from_secs(180), fetch(&channel, action, &ticket, weak.clone())) => {
            result.unwrap_or_else(|_| Err(Status::deadline_exceeded("The media request timed out.")))
        }
    };
    let _ = weak.upgrade_in_event_loop(move |app| {
        // Recheck on the UI thread: cancellation may follow network completion
        // but precede this callback (including A -> B -> A).
        if !ticket.current() {
            return;
        }
        app.set_media_loading(false);
        match result {
            Ok(Reply::Occurrences(snapshot)) => {
                let empty = snapshot.current.is_none() && snapshot.played.is_empty() && snapshot.upcoming.is_empty();
                show_occurrences(&app, snapshot);
                app.set_media_busy(false);
                app.set_media_cancellable(false);
                app.set_media_message(if empty { "The queue is empty." } else { "Choose a queued item to change its audio. Changing the current item resets its position to the beginning, even when paused. A paused item stays paused." }.into());
            }
            Ok(Reply::OccurrenceOptions { target, options }) => {
                let description = target.description();
                // source() already required metadata before the request.
                if let Some(track) = target.entry.track.clone() {
                    let selectable = STATE.with(|s| {
                        let mut state = s.borrow_mut();
                        let mut page = audio_page(track, options);
                        page.target = Some(target);
                        let selectable = page.rows.iter().any(|row| row.enabled);
                        state.audio = Some(page);
                        render(&app, &state);
                        selectable
                    });
                    let availability = if selectable { "" } else { "No selectable audio streams. " };
                    app.set_media_message(format!("{availability}{description}. If this is the current item when you apply the change, its position resets to the beginning, even when paused. A paused item stays paused.").into());
                }
                app.set_media_busy(false);
                app.set_media_cancellable(false);
            }
            Ok(Reply::SelectedOccurrence(snapshot)) => {
                match snapshot {
                    Ok(snapshot) => {
                        show_occurrences(&app, snapshot);
                        app.set_media_message("Audio choice saved for this queued item.".into());
                    }
                    Err(error) => {
                        STATE.with(|s| {
                            let mut state = s.borrow_mut();
                            state.audio = None;
                            state.occurrences = Some(Vec::new());
                            render(&app, &state);
                        });
                        app.set_media_message(format!("Audio choice was applied, but reading the queue failed: {}. Refresh the snapshot; do not resubmit the change.", message(error)).into());
                    }
                }
                app.set_media_busy(false);
                app.set_media_cancellable(false);
            }
            Ok(Reply::AudioOptions { entry, options }) => {
                if let Some(track) = entry.track {
                    STATE.with(|state| {
                        let mut state = state.borrow_mut();
                        state.audio = Some(audio_page(track, options));
                        render(&app, &state);
                        let selectable = state.audio.as_ref().is_some_and(|page| page.rows.iter().any(|row| row.enabled));
                        app.set_media_message(if selectable {
                            "Choose an audio track to add to the queue. Use a breadcrumb to return.".into()
                        } else { "No selectable audio streams. Use a breadcrumb to return.".into() });
                    });
                } else {
                    app.set_media_message("This item has no audio track.".into());
                }
                app.set_media_busy(false);
                app.set_media_cancellable(false);
            }
            Ok(Reply::AddedSelected(_ids)) => {
                app.set_media_busy(false);
                app.set_media_cancellable(false);
                app.set_media_message("Added to the queue with your audio choice.".into());
            }
            Ok(Reply::Page {
                location,
                entries,
                next,
            }) => {
                STATE.with(|state| {
                    let mut state = state.borrow_mut();
                    state.location = location;
                    state.rows = entries;
                    state.next = next;
                    render(&app, &state);
                    app.set_media_message(if state.rows.is_empty() {
                        "No media in this folder.".into()
                    } else {
                        "".into()
                    });
                });
                app.set_media_busy(false);
                app.set_media_cancellable(false);
            }
            Ok(Reply::Queue {
                server,
                tracks,
                play,
            }) => {
                if tracks.is_empty() {
                    app.set_media_busy(false);
                    app.set_media_cancellable(false);
                    app.set_media_message("This container has no audio tracks.".into());
                } else if tx
                    .send(Cmd::MediaCommit(Commit {
                        ticket,
                        server,
                        tracks,
                        play,
                    }))
                    .is_err()
                {
                    app.set_media_busy(false);
                    app.set_media_cancellable(false);
                    app.set_media_message("The connection worker is unavailable.".into());
                }
            }
            Err(error) => {
                app.set_media_busy(false);
                app.set_media_cancellable(false);
                app.set_media_message(message(error).into());
            }
        }
    });
}

/// Commits a container expansion or an explicit default leaf Play.
/// Selected leaves use their own bounded AddSelectedMedia acknowledgement.
/// Once the mutation is sent, cancellation cannot roll it back.
pub async fn commit(channel: Channel, weak: Weak<AppWindow>, request: Commit) {
    let Commit {
        ticket,
        server,
        tracks,
        play,
    } = request;
    // Own the Slint Weak in the spawned future; do not require a shared Weak
    // reference to be Sync across network awaits.
    let operation_ticket = ticket.clone();
    let observer = weak.clone();
    let result = async move {
        let ticket = operation_ticket;
        let weak = observer;
        let mut library = client(&channel);
        let browser = tokio::select! {
            biased;
            _ = ticket.cancelled() => return Err(Status::cancelled("Cancelled")),
            result = tokio::time::timeout(Duration::from_secs(15), library.get_media_browser(GetMediaBrowserRequest {})) => {
                result.map_err(|_| Status::deadline_exceeded("Checking the connected server timed out."))??.into_inner().browser
            }
        };
        if browser.as_ref().map(|value| value.server_id.as_str()) != Some(server.as_str()) {
            return Err(Status::failed_precondition("The browsing server changed. Refresh its libraries."));
        }
        if !ticket.transition_pending(RequestPhase::Submitted) { return Err(Status::cancelled("Cancelled")); }
        let submitted = ticket.clone();
        let _ = weak.upgrade_in_event_loop(move |app| {
            if submitted.current() { app.set_media_cancellable(false); }
        });
        let mut tracklist = TracklistServiceClient::new(channel.clone());
        let mutation = async {
            if play {
                tracklist.load_tracks(LoadTracksRequest { tracks, start_index: 0 }).await?;
            } else {
                tracklist.add_tracks(AddTracksRequest { tracks }).await?;
            }
            Ok::<(), Status>(())
        };
        tokio::time::timeout(Duration::from_secs(15), mutation).await
            .map_err(|_| Status::deadline_exceeded("The queue request timed out; its result is unknown. The delayed command may still apply. A refresh showing no change does not prove failure. Do not retry this mutation."))?
    }.await;
    let _ = weak.upgrade_in_event_loop(move |app| {
        if !ticket.current() {
            return;
        }
        app.set_media_busy(false);
        app.set_media_cancellable(false);
        app.set_media_message(match result {
            Ok(()) => {
                if play {
                    "Playback requested.".into()
                } else {
                    "Added to queue.".into()
                }
            }
            Err(error) => message(error).into(),
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn newer_control_cancels_pending_before_delayed_preflight_can_submit() {
        tokio::time::timeout(Duration::from_secs(5), async {
            for control in [Cmd::Pause, Cmd::Next, Cmd::SeekMs(700)] {
                let ticket = Ticket::new();
                let mut pending = Some(ticket.clone());
                let (ready, waiting) = tokio::sync::oneshot::channel();
                let (finish, gate) = tokio::sync::oneshot::channel();
                let worker = tokio::spawn(async move {
                    ready.send(()).unwrap();
                    gate.await.unwrap();
                    ticket.transition_pending(RequestPhase::Submitted)
                });
                waiting.await.unwrap();
                // This is the actual worker's pre-await arbitration helper.
                assert!(crate::rpc::supersede_pending_media(&control, &mut pending).is_some());
                finish.send(()).unwrap();
                assert!(!worker.await.unwrap(), "late preflight must not load/play");
                assert!(pending.is_none());
            }
        })
        .await
        .unwrap();
    }

    #[test]
    fn controls_do_not_claim_to_revoke_submitted_media_or_cancel_for_volume() {
        let ticket = Ticket::new();
        let mut pending = Some(ticket.clone());
        assert!(crate::rpc::supersede_pending_media(&Cmd::SetVolume(0.5), &mut pending).is_none());
        assert!(pending.is_some());
        assert!(ticket.transition_pending(RequestPhase::Submitted));
        assert!(crate::rpc::supersede_pending_media(&Cmd::Pause, &mut pending).is_none());
        assert!(ticket.current());
        assert_eq!(*ticket.0.borrow(), RequestPhase::Submitted);
    }

    #[tokio::test]
    async fn leaving_page_prevents_pending_commit_after_preflight_finishes() {
        tokio::time::timeout(Duration::from_secs(5), async {
            let ticket = Ticket::new();
            let pending = ticket.clone();
            let mut state = State {
                active: Some(ticket),
                ..Default::default()
            };
            let (ready, waiting) = tokio::sync::oneshot::channel();
            let (finish, gate) = tokio::sync::oneshot::channel();
            let worker = tokio::spawn(async move {
                ready.send(()).unwrap();
                gate.await.unwrap();
                pending.transition_pending(RequestPhase::Submitted)
            });
            waiting.await.unwrap();
            state.leave();
            // A fresh drawer Audio entry owns a different ticket, even when
            // returning to the same media page/account.
            let drawer = Ticket::new();
            state.active = Some(drawer.clone());
            finish.send(()).unwrap();
            assert!(!worker.await.unwrap());
            assert!(drawer.current());
            assert!(drawer.transition_pending(RequestPhase::Submitted));
        })
        .await
        .unwrap();
    }

    #[test]
    fn leaving_page_retires_submitted_callback_without_reusing_its_identity() {
        let submitted = Ticket::new();
        assert!(submitted.transition_pending(RequestPhase::Submitted));
        let mut state = State {
            active: Some(submitted.clone()),
            occurrences: Some(Vec::new()),
            ..Default::default()
        };
        // Cancel only owns Pending; navigation must also retire Submitted UI.
        assert!(!submitted.transition_pending(RequestPhase::Cancelled));
        state.leave();
        assert!(!submitted.current()); // late acknowledgement cannot update UI
        assert!(state.active.is_none());
        assert!(state.occurrences.is_none());
        let reopened = Ticket::new();
        state.active = Some(reopened.clone());
        submitted.cancel();
        assert!(reopened.current());
        // This tests local ownership, not rollback of a daemon mutation.
    }

    #[test]
    fn leaf_play_is_independent_of_options_and_containers_cannot_play() {
        let leaf = MediaEntry {
            track: Some(Track::default()),
            ..Default::default()
        };
        assert!(matches!(
            browse_action(leaf.clone(), Some("account".into()), true),
            Some(Action::Queue { play: true, .. })
        ));
        assert!(matches!(
            browse_action(leaf, Some("account".into()), false),
            Some(Action::AudioOptions(_))
        ));
        assert!(browse_action(
            MediaEntry {
                is_container: true,
                ..Default::default()
            },
            Some("account".into()),
            true
        )
        .is_none());
        assert!(browse_action(MediaEntry::default(), Some("account".into()), true).is_none());
    }

    fn occurrence(id: &str, account: &str) -> MediaQueueOccurrence {
        let source = SourceRef {
            resolver: "emby".into(),
            account_id: account.into(),
            remote: music_player_types::source::RemoteIdentity {
                server_id: "server".into(),
                user_id: account.into(),
            },
            kind: ResourceKind::Item,
            item_id: "same-item".into(),
        }
        .to_handle();
        MediaQueueOccurrence {
            occurrence_id: id.into(),
            track: Some(Track {
                id: source.clone(),
                uri: source,
                title: "Same title".into(),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn occurrence_snapshot_preserves_repeated_tracks_and_only_deduplicates_current_identity() {
        let current = occurrence("current", "a");
        let history = occurrence("previous-copy", "a");
        let upcoming = occurrence("next-copy", "a");
        let rows = occurrence_rows(GetMediaQueueResponse {
            current: Some(current.clone()),
            played: vec![history, current],
            upcoming: vec![upcoming],
        });
        assert_eq!(
            rows.iter()
                .map(|r| r.entry.occurrence_id.as_str())
                .collect::<Vec<_>>(),
            vec!["current", "previous-copy", "next-copy"]
        );
        assert_eq!(
            rows.iter().map(|r| r.section).collect::<Vec<_>>(),
            vec!["Current", "History", "Upcoming"]
        );
        assert_eq!(rows[0].source().unwrap(), rows[2].source().unwrap());
    }

    #[test]
    fn queue_source_and_edit_target_remain_the_saved_occurrence_not_browsing_account() {
        let target = OccurrenceRow {
            entry: occurrence("second-copy", "account-a"),
            section: "Upcoming",
        };
        let browsing = occurrence("unrelated", "account-b");
        assert_ne!(target.source().unwrap(), browsing.track.unwrap().id);
        assert_eq!(
            SourceRef::parse(&target.source().unwrap())
                .unwrap()
                .account_id,
            "account-a"
        );
        let mut page = AudioPage {
            track: target.entry.track.clone().unwrap(),
            target: Some(target),
            rows: vec![AudioRow {
                title: "Original".into(),
                subtitle: String::new(),
                choice: Some(MediaQueueChoice {
                    media_source_id: Some("version".into()),
                    audio_stream_index: Some(0),
                }),
                enabled: true,
            }],
        };
        match selected_action(&page, 0).unwrap() {
            Action::SelectOccurrence {
                occurrence_id,
                choice,
            } => {
                assert_eq!(occurrence_id, "second-copy");
                assert_eq!(choice.unwrap().audio_stream_index, Some(0));
            }
            _ => panic!("must edit, not append or play by index"),
        }
        page.rows[0].enabled = false;
        assert!(selected_action(&page, 0).is_none());
        assert!(selected_action(&page, -1).is_none());
        page.target
            .as_mut()
            .unwrap()
            .entry
            .track
            .as_mut()
            .unwrap()
            .uri = "http://example.invalid/temporary".into();
        assert!(page.target.as_ref().unwrap().source().is_err());
    }

    #[test]
    fn requested_auto_and_accepted_stream_zero_are_distinct() {
        let mut row = OccurrenceRow {
            entry: occurrence("current", "a"),
            section: "Current",
        };
        row.entry.accepted_audio = Some(
            music_player_server::api::music::v1alpha1::MediaQueueAcceptedAudio {
                media_source_id: "original".into(),
                audio_stream_index: 0,
                runtime_ticks: Some(u64::MAX),
                ..Default::default()
            },
        );
        let description = row.description();
        assert!(description.contains("Selected: Automatic"));
        assert!(description.contains("Playback audio: original / audio 0"));
    }

    #[test]
    fn audio_candidates_preserve_zero_empty_language_defaults_and_disabled_versions() {
        use music_player_server::api::music::v1alpha1::{AudioStreamOption, AudioVersionOption};
        let page = audio_page(
            Track::default(),
            GetMediaAudioOptionsResponse {
                source: "stable-source".into(),
                versions: vec![
                    AudioVersionOption {
                        id: "original".into(),
                        name: Some("Original cut".into()),
                        default_audio_stream_index: Some(0),
                        audio_streams: vec![AudioStreamOption {
                            index: 0,
                            title: Some("Original audio".into()),
                            display_title: Some("AAC stereo".into()),
                            language: Some(String::new()),
                            is_default: true,
                            ..Default::default()
                        }],
                        ..Default::default()
                    },
                    AudioVersionOption {
                        id: "live".into(),
                        unavailable_reason: Some("Requires opening".into()),
                        audio_streams: vec![AudioStreamOption {
                            index: 1,
                            ..Default::default()
                        }],
                        ..Default::default()
                    },
                ],
            },
        );
        assert_eq!(page.rows.len(), 3);
        assert!(page.rows[0].choice.is_none()); // Auto is intent, not a version.
        let original = &page.rows[1];
        assert!(original.enabled);
        assert_eq!(
            original.choice.as_ref().unwrap().audio_stream_index,
            Some(0)
        );
        assert_eq!(
            original.choice.as_ref().unwrap().media_source_id.as_deref(),
            Some("original")
        );
        for label in [
            "Original audio",
            "AAC stereo",
            "Language: (empty)",
            "Stream default",
            "Version default",
        ] {
            assert!(original.subtitle.contains(label));
        }
        assert!(!page.rows[2].enabled);
        assert!(page.rows[2].subtitle.contains("Requires opening"));
    }

    #[test]
    fn empty_metadata_never_fabricates_an_automatic_playable_candidate() {
        let page = audio_page(Track::default(), GetMediaAudioOptionsResponse::default());
        assert!(page.rows.is_empty());
    }

    #[tokio::test]
    async fn cancelled_before_or_during_subscription_wakes_and_never_reactivates() {
        let old_a = Ticket::new();
        old_a.cancel();
        tokio::time::timeout(Duration::from_secs(1), old_a.cancelled())
            .await
            .unwrap();
        let b = Ticket::new();
        let waiting = b.clone();
        let pending = tokio::spawn(async move { waiting.cancelled().await });
        b.cancel();
        tokio::time::timeout(Duration::from_secs(1), pending)
            .await
            .unwrap()
            .unwrap();
        let new_a = Ticket::new();
        old_a.cancel();
        assert!(!old_a.current());
        assert!(!b.current());
        assert!(new_a.current());
    }

    #[test]
    fn cancellation_and_submission_are_exclusive_but_a_source_switch_invalidates_both() {
        let cancelled = Ticket::new();
        assert!(cancelled.transition_pending(RequestPhase::Cancelled));
        assert!(!cancelled.transition_pending(RequestPhase::Submitted));
        let submitted = Ticket::new();
        assert!(submitted.transition_pending(RequestPhase::Submitted));
        assert!(!submitted.transition_pending(RequestPhase::Cancelled));
        assert!(!submitted.transition_pending(RequestPhase::Submitted));
        assert!(submitted.current());
        submitted.cancel();
        assert!(!submitted.current());
    }

    #[test]
    fn media_indices_are_exact_and_do_not_change_the_original_title() {
        let entry = MediaEntry {
            title: "My First Ever Ender Dragon Fight! [First Ever Minecraft Playthrough Ep.40]"
                .into(),
            item_type: Some("Episode".into()),
            season_number: Some(u64::MAX),
            episode_number: Some(0),
            track: Some(Track::default()),
            ..Default::default()
        };
        assert_eq!(subtitle(&entry), "Episode · S18446744073709551615 · E0");
        assert!(entry.title.ends_with("Ep.40]"));
    }
}
