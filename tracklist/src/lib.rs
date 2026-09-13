#[cfg(test)]
mod tests;

use music_player_entity::track::Model as Track;
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
    tracks: Vec<Track>,
    played: Vec<Track>,
    current_track: Option<Track>,
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
            tracks,
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
        self.tracks.push(track);
    }

    pub fn next_track(&mut self) -> Option<Track> {
        if self.tracks.is_empty() {
            return None;
        }

        let next_track = self.tracks.remove(0);
        self.current_track = Some(next_track.clone());
        self.played.push(next_track.clone());
        Some(next_track)
    }

    pub fn previous_track(&mut self) -> Option<Track> {
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

        Some(previous_track)
    }

    pub fn current_track(&self) -> (Option<Track>, usize) {
        (self.current_track.clone(), self.played.len())
    }

    /// Replace the current track in place, keeping the queue split intact.
    /// Used to fold a live stream's ICY metadata (the song playing right now)
    /// onto the station entry — the history copy is updated too so the queue
    /// drawer and the now-playing bar keep showing the same thing.
    pub fn update_current_track(&mut self, track: Track) {
        if self.current_track.as_ref().map(|t| t.id.as_str()) != Some(track.id.as_str()) {
            return;
        }
        if let Some(last) = self.played.last_mut() {
            if last.id == track.id {
                *last = track.clone();
            }
        }
        self.current_track = Some(track);
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
            if track.id == id {
                track.liked = Some(liked);
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
            if track.liked.is_some() {
                track.liked = Some(starred.contains(&track.id));
            }
        }
    }

    pub fn tracks(&self) -> (Vec<Track>, Vec<Track>) {
        (self.played.clone(), self.tracks.clone())
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
        self.tracks.retain(|t| t.id != track.id);
        self.played.retain(|t| t.id != track.id);
    }

    pub fn remove_track_at(&mut self, index: usize) {
        if index >= self.played.len() {
            self.tracks.remove(index - self.played.len());
            return;
        }
        self.played.remove(index);
    }

    pub fn insert(&mut self, index: usize, track: Track) {
        self.tracks.insert(index, track);
    }

    pub fn insert_tracks(&mut self, index: usize, tracks: Vec<Track>) {
        self.tracks.splice(index..index, tracks);
    }

    pub fn insert_next(&mut self, track: Track) {
        self.tracks.insert(0, track);
    }

    pub fn queue(&mut self, tracks: Vec<Track>) {
        self.tracks.extend(tracks);
    }

    pub fn shuffle(&mut self) {
        self.tracks.shuffle(&mut rand::thread_rng());
    }

    /// The upcoming track, without advancing.
    pub fn peek_next(&self) -> Option<Track> {
        self.tracks.first().cloned()
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
        self.tracks = tracks;
    }

    /// Rebuild the exact queue split from a persisted snapshot. The current
    /// track is the last `played` entry — the same invariant `next_track`
    /// maintains — and playback starts out paused at `position_ms`.
    pub fn restore(&mut self, played: Vec<Track>, tracks: Vec<Track>, position_ms: u32) {
        self.current_track = played.last().cloned();
        self.played = played;
        self.tracks = tracks;
        self.playback_state = PlaybackState {
            position_ms,
            is_playing: false,
        };
    }
}
