#[cfg(test)]
mod audio_tests;
mod entry;
#[cfg(test)]
mod tests;
pub use entry::QueueEntry;

use music_player_entity::track::Model as Track;
use music_player_types::audio::{AudioPin, AudioSelection};
use rand::seq::SliceRandom;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct PlaybackState {
    pub position_ms: u32,
    pub is_playing: bool,
}

/// Output levels for a meter, as the engine last reported them.
///
/// Kept here because the player writes them and the gRPC and GraphQL layers
/// read them, and all three already share this structure — a meter is not
/// worth a second channel.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Levels {
    pub left: f32,
    pub right: f32,
    /// The same signal below roughly 200 Hz, which is what makes a meter move
    /// with the bass rather than with whatever is loudest.
    pub low_left: f32,
    pub low_right: f32,
    /// Coarse spectrum, low band to high — what a bar visualiser draws.
    pub bands: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct Tracklist {
    tracks: Vec<QueueEntry>,
    played: Vec<QueueEntry>,
    current_track: Option<QueueEntry>,
    playback_state: PlaybackState,
    levels: Levels,
    /// The playback modes, so a client can *read* them rather than only set
    /// them. Held here because the player owns them and every API layer needs
    /// to report them — without this each client kept its own copy, which
    /// started at "off" on every launch however the session had ended.
    shuffle: bool,
    /// 0 off, 1 all, 2 one.
    repeat_mode: i32,
}

impl Tracklist {
    pub fn new(tracks: Vec<Track>) -> Self {
        Self {
            tracks: tracks.into_iter().map(QueueEntry::new).collect(),
            played: Vec::new(),
            current_track: None,
            playback_state: PlaybackState::default(),
            levels: Levels::default(),
            shuffle: false,
            repeat_mode: 0,
        }
    }
    pub fn new_empty() -> Self {
        Self {
            tracks: Vec::new(),
            played: Vec::new(),
            current_track: None,
            playback_state: PlaybackState::default(),
            levels: Levels::default(),
            shuffle: false,
            repeat_mode: 0,
        }
    }

    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(QueueEntry::new(track));
    }

    pub fn next_track(&mut self) -> Option<Track> {
        if self.tracks.is_empty() {
            return None;
        }

        let next_track = self.tracks.remove(0);
        self.current_track = Some(next_track.clone());
        self.played.push(next_track.clone());
        Some(next_track.track)
    }

    pub fn previous_track(&mut self) -> Option<Track> {
        // Removing the currently sounding occurrence does not stop playback.
        // In that case the last retained history entry is already Previous;
        // do not pop it as if it were still the current occurrence.
        if self.current_track.as_ref().is_some_and(|current| {
            self.played
                .last()
                .is_none_or(|last| last.occurrence_id != current.occurrence_id)
        }) {
            let previous = self.played.last()?.clone();
            self.current_track = Some(previous.clone());
            return Some(previous.track);
        }
        if self.played.len() < 2 {
            return None;
        }

        let previous_track = self.played.pop().unwrap();
        self.tracks.insert(0, previous_track.clone());

        if self.played.is_empty() {
            self.current_track = None;
            return None;
        }

        let previous_track = self.played.pop().unwrap();
        self.current_track = Some(previous_track.clone());

        self.played.push(previous_track.clone());

        Some(previous_track.track)
    }

    pub fn current_track(&self) -> (Option<Track>, usize) {
        (
            self.current_track.as_ref().map(|entry| entry.track.clone()),
            self.played.len(),
        )
    }

    /// Replace the current track in place, keeping the queue split intact.
    /// Used to fold a live stream's ICY metadata (the song playing right now)
    /// onto the station entry — the history copy is updated too so the queue
    /// drawer and the now-playing bar keep showing the same thing.
    pub fn update_current_track(&mut self, track: Track) {
        let Some(current) = self.current_track.as_mut() else {
            return;
        };
        if current.track.id != track.id {
            return;
        }
        if let Some(last) = self.played.last_mut() {
            if last.occurrence_id == current.occurrence_id {
                last.track = track.clone();
            }
        }
        current.track = track;
    }

    /// Record a like/unlike on every queued copy of the track — up-next,
    /// history and current. The now-playing readout serves `liked` from these
    /// copies, so without this a toggle only reaches the remote server and the
    /// heart keeps showing the state from when the track was queued.
    pub fn set_track_liked(&mut self, id: &str, liked: bool) {
        for track in self
            .tracks
            .iter_mut()
            .chain(self.played.iter_mut())
            .chain(self.current_track.iter_mut())
        {
            if track.track.id == id {
                track.track.liked = Some(liked);
            }
        }
    }

    /// Overwrite the queued copies' `liked` with the provider's own starred
    /// set: every id in it is starred, every id missing from it is not. The
    /// queue can outlive a session — it is restored from disk with whatever
    /// `liked` each track had when it was queued — so a freshly connected
    /// provider's answer must replace those snapshots, both ways.
    ///
    /// Tracks whose `liked` was never known (`None` — a local file) are left
    /// alone: they are not the provider's to answer for, and the clients fall
    /// back to the local like store for them.
    pub fn restamp_liked(&mut self, starred: &std::collections::HashSet<String>) {
        for track in self
            .tracks
            .iter_mut()
            .chain(self.played.iter_mut())
            .chain(self.current_track.iter_mut())
        {
            if track.track.liked.is_some() {
                track.track.liked = Some(starred.contains(&track.track.id));
            }
        }
    }

    pub fn tracks(&self) -> (Vec<Track>, Vec<Track>) {
        (
            self.played
                .iter()
                .map(|entry| entry.track.clone())
                .collect(),
            self.tracks
                .iter()
                .map(|entry| entry.track.clone())
                .collect(),
        )
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
        self.played.clear();
    }

    pub fn remove_track(&mut self, track: Track) {
        self.tracks.retain(|t| t.track.id != track.id);
        self.played.retain(|t| t.track.id != track.id);
    }

    pub fn remove_track_at(&mut self, index: usize) {
        if index >= self.played.len() {
            self.tracks.remove(index - self.played.len());
            return;
        }
        self.played.remove(index);
    }

    pub fn insert(&mut self, index: usize, track: Track) {
        self.tracks.insert(index, QueueEntry::new(track));
    }

    pub fn insert_tracks(&mut self, index: usize, tracks: Vec<Track>) {
        self.tracks
            .splice(index..index, tracks.into_iter().map(QueueEntry::new));
    }

    pub fn insert_next(&mut self, track: Track) {
        self.tracks.insert(0, QueueEntry::new(track));
    }

    /// Preserve the complete browse metadata and requested audio choice.
    /// Validation precedes mutation; each insertion is a new occurrence.
    pub fn insert_next_with_selection(
        &mut self,
        track: Track,
        selection: AudioSelection,
    ) -> Result<String, &'static str> {
        let mut entry = QueueEntry::new(track);
        entry.selection = selection;
        entry.validate()?;
        let id = entry.occurrence_id.clone();
        self.tracks.insert(0, entry);
        Ok(id)
    }

    pub fn queue(&mut self, tracks: Vec<Track>) {
        self.tracks.extend(tracks.into_iter().map(QueueEntry::new));
    }

    pub fn shuffle(&mut self) {
        self.tracks.shuffle(&mut rand::thread_rng());
    }

    /// The upcoming track, without advancing.
    pub fn peek_next(&self) -> Option<Track> {
        self.tracks.first().map(|entry| entry.track.clone())
    }

    pub fn play_track_at(&mut self, index: usize) -> (Option<Track>, usize) {
        if index >= (self.tracks.len() + self.played.len()) {
            return (None, 0);
        }

        self.played = [self.played.clone(), self.tracks.clone()].concat();
        self.tracks = self.played.split_off(index);

        if index > 1 && index < self.played.len() - 1 {
            self.next_track();
        }
        self.next_track();
        self.current_track()
    }

    pub fn playback_state(&self) -> PlaybackState {
        self.playback_state.clone()
    }

    pub fn levels(&self) -> Levels {
        self.levels.clone()
    }

    pub fn set_levels(&mut self, levels: Levels) {
        self.levels = levels;
    }

    pub fn shuffle_enabled(&self) -> bool {
        self.shuffle
    }

    pub fn repeat_mode(&self) -> i32 {
        self.repeat_mode
    }

    pub fn set_modes(&mut self, shuffle: bool, repeat_mode: i32) {
        self.shuffle = shuffle;
        self.repeat_mode = repeat_mode;
    }

    pub fn set_playback_state(&mut self, playback_state: PlaybackState) {
        self.playback_state = playback_state;
    }

    pub fn stop(&mut self) {
        self.current_track = None;
        self.playback_state.is_playing = false;
    }

    pub fn load_tracks(&mut self, tracks: Vec<Track>) {
        self.clear();
        self.tracks = tracks.into_iter().map(QueueEntry::new).collect();
    }

    /// Rebuild the exact queue split from a persisted snapshot. The current
    /// track is the last `played` entry — the same invariant `next_track`
    /// maintains — and playback starts out paused at `position_ms`.
    pub fn restore(&mut self, played: Vec<Track>, tracks: Vec<Track>, position_ms: u32) {
        self.restore_entries(
            played.into_iter().map(QueueEntry::new).collect(),
            tracks.into_iter().map(QueueEntry::new).collect(),
            position_ms,
        )
        .expect("new queue entries have unique identities and no audio choices");
    }

    /// Preserve the identities and choices of both history and upcoming entries.
    /// Validate the complete snapshot before replacing any existing queue.
    pub fn restore_entries(
        &mut self,
        played: Vec<QueueEntry>,
        tracks: Vec<QueueEntry>,
        position_ms: u32,
    ) -> Result<(), &'static str> {
        validate_entries(played.iter().chain(tracks.iter()))?;
        self.current_track = played.last().cloned();
        self.played = played;
        self.tracks = tracks;
        self.playback_state = PlaybackState {
            position_ms,
            is_playing: false,
        };
        Ok(())
    }

    pub fn entries(&self) -> (Vec<QueueEntry>, Vec<QueueEntry>) {
        (self.played.clone(), self.tracks.clone())
    }

    pub fn current_entry(&self) -> Option<QueueEntry> {
        self.current_track.clone()
    }

    /// New incoming items always receive fresh occurrence identities. A pin is
    /// committed only after playback has accepted a resolution.
    pub fn queue_with_selection(
        &mut self,
        tracks: Vec<(Track, AudioSelection)>,
    ) -> Result<Vec<String>, &'static str> {
        let mut entries = Vec::with_capacity(tracks.len());
        for (track, selection) in tracks {
            let mut entry = QueueEntry::new(track);
            entry.selection = selection;
            entry.validate()?;
            entries.push(entry);
        }
        let ids = entries
            .iter()
            .map(|entry| entry.occurrence_id.clone())
            .collect();
        self.tracks.extend(entries);
        Ok(ids)
    }

    /// The current occurrence can outlive its removal from the visible queue.
    /// Match by occurrence identity, never by the media item's shared ID.
    pub fn accept_current_pin(
        &mut self,
        occurrence_id: &str,
        pin: AudioPin,
    ) -> Result<(), &'static str> {
        let current = self
            .current_track
            .as_ref()
            .filter(|entry| entry.occurrence_id == occurrence_id)
            .ok_or("audio resolution belongs to a different queue occurrence")?;
        let mut candidate = current.clone();
        candidate.pin = Some(pin);
        candidate.validate()?;
        for entry in self.played.iter_mut().chain(self.tracks.iter_mut()) {
            if entry.occurrence_id == occurrence_id {
                entry.pin = candidate.pin.clone();
            }
        }
        self.current_track = Some(candidate);
        Ok(())
    }

    /// Editing an occurrence's choice invalidates its old pin. The player must
    /// restart a current occurrence at zero, so another version cannot inherit
    /// a position on an unverified timeline.
    pub fn select_audio(
        &mut self,
        occurrence_id: &str,
        selection: AudioSelection,
    ) -> Result<bool, &'static str> {
        let existing = self
            .current_track
            .iter()
            .chain(self.played.iter())
            .chain(self.tracks.iter())
            .find(|entry| entry.occurrence_id == occurrence_id)
            .ok_or("queue occurrence no longer exists")?;
        let mut candidate = existing.clone();
        let source = music_player_types::source::SourceHandle::parse(&candidate.track.uri)
            .map_err(|_| "audio choices require a stable media source")?;
        if source.kind() != music_player_types::source::ResourceKind::Item {
            return Err("audio choices require a playable media source");
        }
        if matches!(source, music_player_types::source::SourceHandle::Local(_)) {
            return Err("source v1 does not support audio selection");
        }
        candidate.selection = selection;
        candidate.pin = None;
        candidate.validate()?;
        let is_current = self
            .current_track
            .as_ref()
            .is_some_and(|entry| entry.occurrence_id == occurrence_id);
        for entry in self
            .current_track
            .iter_mut()
            .chain(self.played.iter_mut())
            .chain(self.tracks.iter_mut())
        {
            if entry.occurrence_id == occurrence_id {
                entry.selection = candidate.selection.clone();
                entry.pin = None;
            }
        }
        Ok(is_current)
    }
}

pub fn validate_entries<'a>(
    entries: impl IntoIterator<Item = &'a QueueEntry>,
) -> Result<(), &'static str> {
    let mut ids = std::collections::HashSet::new();
    for entry in entries {
        entry.validate()?;
        if !ids.insert(entry.occurrence_id.as_str()) {
            return Err("duplicate queue occurrence identity");
        }
    }
    Ok(())
}
