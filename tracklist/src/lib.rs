use music_player_entity::track::Model as Track;
use rand::seq::SliceRandom;

#[derive(Default, Debug, Clone)]
pub struct PlaybackState {
    pub position_ms: u32,
    pub is_playing: bool,
}

#[derive(Debug, Clone)]
pub struct Tracklist {
    tracks: Vec<Track>,
    played: Vec<Track>,
    current_track: Option<Track>,
    playback_state: PlaybackState,
}

impl Tracklist {
    pub fn new(tracks: Vec<Track>) -> Self {
        Self {
            tracks,
            played: Vec::new(),
            current_track: None,
            playback_state: PlaybackState::default(),
        }
    }
    pub fn new_empty() -> Self {
        Self {
            tracks: Vec::new(),
            played: Vec::new(),
            current_track: None,
            playback_state: PlaybackState::default(),
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
    }

    pub fn remove_track_at(&mut self, index: usize) {
        self.tracks.remove(index);
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

    pub fn set_playback_state(&mut self, playback_state: PlaybackState) {
        self.playback_state = playback_state;
    }
}
