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

#[test]
fn insert_tracks() {
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
    tracklist.insert_tracks(0, vec![track2.clone()]);
    assert_eq!(tracklist.len(), 2);
    assert_eq!(tracklist.tracks[0], track2);
}

#[test]
fn insert_next() {
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
        id: "2a81ab806a5d2bf9cad8917e7f89f1a5".to_owned(),
        track: Some(6),
        title: "Wet Dreamz".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.insert_next(track2.clone());
    assert_eq!(tracklist.len(), 2);
    assert_eq!(tracklist.tracks[0], track2);
}

#[test]
fn queue() {
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
        id: "2a81ab806a5d2bf9cad8917e7f89f1a5".to_owned(),
        track: Some(6),
        title: "Wet Dreamz".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.queue(vec![track2.clone()]);
    assert_eq!(tracklist.len(), 2);
    assert_eq!(tracklist.tracks[1], track2);
}

#[test]
fn shuffle() {
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
        id: "2a81ab806a5d2bf9cad8917e7f89f1a5".to_owned(),
        track: Some(6),
        title: "Wet Dreamz".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track2.clone());
    assert_eq!(tracklist.len(), 2);
    let track3 = Track {
        id: "5d7f5f595064177eb70e4c57a5e3ef45".to_owned(),
        track: Some(9),
        title: "No Role Modelz".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track3.clone());
    let track4 = Track {
        id: "e2f90d9be6548928fb55875b9b42f8d8".to_owned(),
        track: Some(7),
        title: "St. Tropez".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track4.clone());
    assert_eq!(tracklist.len(), 4);
    let track5 = Track {
        id: "53fc91928a293df6cd9765a5938446eb".to_owned(),
        track: Some(12),
        title: "Love Yourz".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track5.clone());
    assert_eq!(tracklist.len(), 5);
    tracklist.shuffle();
    assert_eq!(tracklist.len(), 5);
}

#[test]
fn play_track_at() {
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
        id: "2a81ab806a5d2bf9cad8917e7f89f1a5".to_owned(),
        track: Some(6),
        title: "Wet Dreamz".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track2.clone());
    assert_eq!(tracklist.len(), 2);
    let track3 = Track {
        id: "5d7f5f595064177eb70e4c57a5e3ef45".to_owned(),
        track: Some(9),
        title: "No Role Modelz".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track3.clone());
    let track4 = Track {
        id: "e2f90d9be6548928fb55875b9b42f8d8".to_owned(),
        track: Some(7),
        title: "St. Tropez".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track4.clone());
    assert_eq!(tracklist.len(), 4);
    let track5 = Track {
        id: "53fc91928a293df6cd9765a5938446eb".to_owned(),
        track: Some(12),
        title: "Love Yourz".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track5.clone());
    assert_eq!(tracklist.len(), 5);
    tracklist.play_track_at(2);
    assert_eq!(tracklist.current_track(), (Some(track3), 3));
}

#[test]
fn playback_state() {
    let tracklist = Tracklist::new_empty();
    assert_eq!(
        tracklist.playback_state(),
        PlaybackState {
            position_ms: 0,
            is_playing: false,
        }
    );
}

#[test]
fn set_playback_state() {
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
        id: "2a81ab806a5d2bf9cad8917e7f89f1a5".to_owned(),
        track: Some(6),
        title: "Wet Dreamz".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track2.clone());
    assert_eq!(tracklist.len(), 2);
    let track3 = Track {
        id: "5d7f5f595064177eb70e4c57a5e3ef45".to_owned(),
        track: Some(9),
        title: "No Role Modelz".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track3.clone());
    let track4 = Track {
        id: "e2f90d9be6548928fb55875b9b42f8d8".to_owned(),
        track: Some(7),
        title: "St. Tropez".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track4.clone());
    assert_eq!(tracklist.len(), 4);
    let track5 = Track {
        id: "53fc91928a293df6cd9765a5938446eb".to_owned(),
        track: Some(12),
        title: "Love Yourz".to_owned(),
        artist: "J. Cole".to_owned(),
        ..Default::default()
    };
    tracklist.add_track(track5.clone());
    assert_eq!(tracklist.len(), 5);
    tracklist.play_track_at(2);
    let playback_state = PlaybackState {
        position_ms: 0,
        is_playing: true,
    };
    tracklist.set_playback_state(playback_state.clone());
    assert_eq!(tracklist.current_track(), (Some(track3), 3));
    assert_eq!(tracklist.playback_state(), playback_state.clone());
}
