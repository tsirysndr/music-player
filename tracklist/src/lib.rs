use std::sync::{Arc, Mutex};

use music_player_entity::track::Model as Track;
use rand::seq::SliceRandom;

#[derive(Debug, Clone)]
pub struct Tracklist {
    tracks: Arc<Mutex<Vec<Track>>>,
    played: Arc<Mutex<Vec<Track>>>,
    current_track: Option<Track>,
}

impl Tracklist {
    pub fn new(tracks: Vec<Track>) -> Self {
        Self {
            tracks: Arc::new(Mutex::new(tracks)),
            played: Arc::new(Mutex::new(Vec::new())),
            current_track: None,
        }
    }
    pub fn new_empty() -> Self {
        Self {
            tracks: Arc::new(Mutex::new(Vec::new())),
            played: Arc::new(Mutex::new(Vec::new())),
            current_track: None,
        }
    }

    pub fn add_track(&mut self, track: Track) {
        self.tracks.lock().unwrap().push(track);
    }

    pub fn next_track(&mut self) -> Option<Track> {
        if self.tracks.lock().unwrap().is_empty() {
            return None;
        }

        let next_track = self.tracks.lock().unwrap().remove(0);
        self.current_track = Some(next_track.clone());
        self.played.lock().unwrap().push(next_track.clone());
        Some(next_track)
    }

    pub fn previous_track(&mut self) -> Option<Track> {
        if self.played.lock().unwrap().len() < 2 {
            return None;
        }

        let previous_track = self.played.lock().unwrap().pop().unwrap();
        self.tracks
            .lock()
            .unwrap()
            .insert(0, previous_track.clone());

        if self.played.lock().unwrap().is_empty() {
            self.current_track = None;
            return None;
        }

        let previous_track = self.played.lock().unwrap().pop().unwrap();
        self.current_track = Some(previous_track.clone());

        self.played.lock().unwrap().push(previous_track.clone());

        Some(previous_track)
    }

    pub fn current_track(&self) -> (Option<Track>, usize) {
        (
            self.current_track.clone(),
            self.played.lock().unwrap().len(),
        )
    }

    pub fn tracks(&self) -> (Vec<Track>, Vec<Track>) {
        (
            self.played.lock().unwrap().clone(),
            self.tracks.lock().unwrap().clone(),
        )
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.lock().unwrap().is_empty()
    }

    pub fn len(&self) -> usize {
        self.tracks.lock().unwrap().len()
    }

    pub fn clear(&mut self) {
        self.tracks.lock().unwrap().clear();
        self.played.lock().unwrap().clear();
    }

    pub fn remove_track(&mut self, track: Track) {
        self.tracks.lock().unwrap().retain(|t| t.id != track.id);
    }

    pub fn remove_track_at(&mut self, index: usize) {
        self.tracks.lock().unwrap().remove(index);
    }

    pub fn insert(&mut self, index: usize, track: Track) {
        self.tracks.lock().unwrap().insert(index, track);
    }

    pub fn insert_tracks(&mut self, index: usize, tracks: Vec<Track>) {
        self.tracks.lock().unwrap().splice(index..index, tracks);
    }

    pub fn insert_next(&mut self, track: Track) {
        self.tracks.lock().unwrap().insert(0, track);
    }

    pub fn queue(&mut self, tracks: Vec<Track>) {
        self.tracks.lock().unwrap().extend(tracks);
    }

    pub fn shuffle(&mut self) {
        self.tracks.lock().unwrap().shuffle(&mut rand::thread_rng());
    }

    pub fn play_track_at(&mut self, index: usize) -> (Option<Track>, usize) {
        if index >= (self.tracks.lock().unwrap().len() + self.played.lock().unwrap().len()) {
            return (None, 0);
        }
        let played = self.played.lock().unwrap().clone();
        let tracks = self.tracks.lock().unwrap().clone();

        self.played = Arc::new(Mutex::new([played, tracks].concat()));

        self.tracks = Arc::new(Mutex::new(self.played.lock().unwrap().split_off(index)));

        if index > 1 && index < self.played.lock().unwrap().len() - 1 {
            self.next_track();
        }
        self.next_track();
        self.current_track()
    }
}
