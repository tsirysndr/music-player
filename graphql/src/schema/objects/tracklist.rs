use async_graphql::Object;
use music_player_types::types;

use super::track::Track;

#[derive(Default, Clone)]
pub struct Tracklist {
    pub next_tracks: Vec<Track>,
    pub previous_tracks: Vec<Track>,
}

#[Object]
impl Tracklist {
    async fn next_tracks(&self) -> Vec<Track> {
        self.next_tracks.clone()
    }

    async fn previous_tracks(&self) -> Vec<Track> {
        self.previous_tracks.clone()
    }
}

impl From<types::Playback> for Tracklist {
    fn from(playback: types::Playback) -> Self {
        let mut previous_tracks: Vec<Track> = vec![];
        let mut next_tracks: Vec<Track> = vec![];
        let mut next = false;
        for (track, item_id) in playback.items {
            match next {
                true => next_tracks.push(track.into()),
                false => previous_tracks.push(track.into()),
            };
            if playback.current_item_id == Some(item_id) {
                next = true;
            }
        }
        Tracklist {
            next_tracks,
            previous_tracks,
        }
    }
}
