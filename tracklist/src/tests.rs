use super::*;
use music_player_entity::track::Model as Track;

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