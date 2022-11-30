use super::*;
use music_player_entity::track::Model as Track;

#[test]
fn new() {
    let tracklist = Tracklist::new(vec![Track {
        id: "d078aab608b47743781027a8881bf3cb".to_owned(),
        track: Some(6),
        title: "Fire Squad".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    }]);
    assert_eq!(tracklist.tracks.len(), 1);
}

#[test]
fn new_empty() {
    let tracklist = Tracklist::new_empty();
    assert_eq!(tracklist.tracks.len(), 0);
    assert_eq!(tracklist.played.len(), 0);
    assert_eq!(tracklist.current_track, None);
    assert_eq!(tracklist.playback_state.position_ms, 0);
    assert_eq!(tracklist.playback_state.is_playing, false);
}

#[test]
fn next_track() {
    let mut tracklist = Tracklist::new_empty();
    let track = Track {
        id: "d078aab608b47743781027a8881bf3cb".to_owned(),
        track: Some(6),
        title: "Fire Squad".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track.clone());
    assert_eq!(tracklist.next_track(), Some(track));
}

#[test]
fn add_track() {
    let mut tracklist = Tracklist::new_empty();
    let track = Track {
        id: "d078aab608b47743781027a8881bf3cb".to_owned(),
        track: Some(6),
        title: "Fire Squad".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    assert!(tracklist.tracks.is_empty());
    tracklist.add_track(track.clone());
    assert_eq!(tracklist.tracks.len(), 1);
    assert_eq!(tracklist.tracks[0], track);
}

#[test]
fn previous_track() {
    let mut tracklist = Tracklist::new_empty();
    let track = Track {
        id: "d078aab608b47743781027a8881bf3cb".to_owned(),
        track: Some(6),
        title: "Fire Squad".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track.clone());
    assert_eq!(tracklist.next_track(), Some(track.clone()));
    assert_eq!(tracklist.previous_track(), None);
}

#[test]
fn current_track() {
    let mut tracklist = Tracklist::new_empty();
    let track = Track {
        id: "d078aab608b47743781027a8881bf3cb".to_owned(),
        track: Some(6),
        title: "Fire Squad".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track.clone());
    assert_eq!(tracklist.next_track(), Some(track.clone()));
    assert_eq!(tracklist.current_track(), (Some(track), 1));
}

#[test]
fn tracks() {
    let mut tracklist = Tracklist::new_empty();
    let track = Track {
        id: "d078aab608b47743781027a8881bf3cb".to_owned(),
        track: Some(6),
        title: "Fire Squad".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track.clone());
    assert_eq!(tracklist.tracks(), (vec![], vec![track]));
}

#[test]
fn is_empty() {
    let mut tracklist = Tracklist::new_empty();
    assert!(tracklist.is_empty());
    let track = Track {
        id: "d078aab608b47743781027a8881bf3cb".to_owned(),
        track: Some(6),
        title: "Fire Squad".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track.clone());
    assert!(!tracklist.is_empty());
}

#[test]
fn len() {
    let mut tracklist = Tracklist::new_empty();
    assert_eq!(tracklist.len(), 0);
    let track = Track {
        id: "d078aab608b47743781027a8881bf3cb".to_owned(),
        track: Some(6),
        title: "Fire Squad".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track.clone());
    assert_eq!(tracklist.len(), 1);
}

#[test]
fn clear() {
    let mut tracklist = Tracklist::new_empty();
    let track = Track {
        id: "d078aab608b47743781027a8881bf3cb".to_owned(),
        track: Some(6),
        title: "Fire Squad".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track.clone());
    assert_eq!(tracklist.len(), 1);
    tracklist.clear();
    assert_eq!(tracklist.len(), 0);
}

#[test]
fn remove_track() {
    let mut tracklist = Tracklist::new_empty();
    let track = Track {
        id: "d078aab608b47743781027a8881bf3cb".to_owned(),
        track: Some(6),
        title: "Fire Squad".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track.clone());
    assert_eq!(tracklist.len(), 1);
    tracklist.remove_track(track);
    assert_eq!(tracklist.len(), 0);
}

#[test]
fn remove_track_at() {
    let mut tracklist = Tracklist::new_empty();
    let track = Track {
        id: "d078aab608b47743781027a8881bf3cb".to_owned(),
        track: Some(6),
        title: "Fire Squad".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track.clone());
    assert_eq!(tracklist.len(), 1);
    tracklist.remove_track_at(0);
    assert_eq!(tracklist.len(), 0);
}

#[test]
fn insert() {
    let mut tracklist = Tracklist::new_empty();
    let track = Track {
        id: "d078aab608b47743781027a8881bf3cb".to_owned(),
        track: Some(6),
        title: "Fire Squad".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track.clone());
    assert_eq!(tracklist.len(), 1);
    let track2 = Track {
        id: "d078aab608b47743781027a8881bf3cb".to_owned(),
        track: Some(6),
        title: "Fire Squad".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.insert(0, track2.clone());
    assert_eq!(tracklist.len(), 2);
    assert_eq!(tracklist.tracks[0], track2);
}
