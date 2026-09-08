use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::vec;

use music_player_entity::album::Model as Album;
use music_player_entity::track::Model as Track;
use music_player_tracklist::Tracklist;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::player::{Player, PlayerCommand};

#[tokio::test]
async fn load_tracklist() {
    let (cmd_tx, cmd_rx, tracklist) = setup_new_params();

    let (_, _) = Player::new(
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let tracks = vec![
        Track {
            id: "2a81ab806a5d2bf9cad8917e7f89f1a5".to_owned(),
            title: "Wet Dreamz".to_owned(),
            artist: "J. Cole".to_owned(),
            track: Some(1),
            duration: Some(239.381),
            uri: "/tmp/audio/03 - J. Cole - Wet Dreamz(Explicit).m4a".to_owned(),
            album: Album {
                id: "ecd3fb5214ef4faf77a0eba4eedb2638".to_owned(),
                title: "2014 Forest Hills Drive".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        },
        Track {
            id: "d078aab608b47743781027a8881bf3cb".to_owned(),
            title: "Fire Squad".to_owned(),
            artist: "J. Cole".to_owned(),
            track: Some(2),
            duration: Some(288.238),
            uri: "/tmp/audio/06 - J. Cole - Fire Squad(Explicit).m4a".to_owned(),
            album: Album {
                id: "ecd3fb5214ef4faf77a0eba4eedb2638".to_owned(),
                title: "2014 Forest Hills Drive".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        },
    ];

    cmd_tx
        .lock()
        .unwrap()
        .send(PlayerCommand::LoadTracklist {
            tracks,
            start_index: None,
        })
        .unwrap();

    sleep(Duration::from_millis(1000));

    let (previous_tracks, next_tracks) = tracklist.lock().unwrap().tracks();

    assert_eq!(previous_tracks.len(), 1);
    assert_eq!(next_tracks.len(), 1);
    assert_eq!(previous_tracks[0].id, "2a81ab806a5d2bf9cad8917e7f89f1a5");
    assert_eq!(next_tracks[0].id, "d078aab608b47743781027a8881bf3cb");
}

#[test]
fn play() {
    let (cmd_tx, cmd_rx, tracklist) = setup_new_params();

    let (_, _) = Player::new(
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let tracks = vec![
        Track {
            id: "2a81ab806a5d2bf9cad8917e7f89f1a5".to_owned(),
            title: "Wet Dreamz".to_owned(),
            artist: "J. Cole".to_owned(),
            track: Some(1),
            duration: Some(239.381),
            uri: "/tmp/audio/03 - J. Cole - Wet Dreamz(Explicit).m4a".to_owned(),
            album: Album {
                id: "ecd3fb5214ef4faf77a0eba4eedb2638".to_owned(),
                title: "2014 Forest Hills Drive".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        },
        Track {
            id: "d078aab608b47743781027a8881bf3cb".to_owned(),
            title: "Fire Squad".to_owned(),
            artist: "J. Cole".to_owned(),
            track: Some(2),
            duration: Some(288.238),
            uri: "/tmp/audio/06 - J. Cole - Fire Squad(Explicit).m4a".to_owned(),
            album: Album {
                id: "ecd3fb5214ef4faf77a0eba4eedb2638".to_owned(),
                title: "2014 Forest Hills Drive".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        },
    ];

    cmd_tx
        .lock()
        .unwrap()
        .send(PlayerCommand::LoadTracklist {
            tracks,
            start_index: None,
        })
        .unwrap();

    sleep(Duration::from_millis(1000));

    let state = tracklist.lock().unwrap().playback_state();

    assert!(state.is_playing);

    cmd_tx.lock().unwrap().send(PlayerCommand::Pause).unwrap();

    sleep(Duration::from_millis(1000));

    let state = tracklist.lock().unwrap().playback_state();

    assert!(!state.is_playing);

    cmd_tx.lock().unwrap().send(PlayerCommand::Play).unwrap();

    sleep(Duration::from_millis(1000));

    let state = tracklist.lock().unwrap().playback_state();

    assert!(state.is_playing);
}

#[test]
fn pause() {
    let (cmd_tx, cmd_rx, tracklist) = setup_new_params();

    let (_, _) = Player::new(
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let tracks = vec![
        Track {
            id: "2a81ab806a5d2bf9cad8917e7f89f1a5".to_owned(),
            title: "Wet Dreamz".to_owned(),
            artist: "J. Cole".to_owned(),
            track: Some(1),
            duration: Some(239.381),
            uri: "/tmp/audio/03 - J. Cole - Wet Dreamz(Explicit).m4a".to_owned(),
            album: Album {
                id: "ecd3fb5214ef4faf77a0eba4eedb2638".to_owned(),
                title: "2014 Forest Hills Drive".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        },
        Track {
            id: "d078aab608b47743781027a8881bf3cb".to_owned(),
            title: "Fire Squad".to_owned(),
            artist: "J. Cole".to_owned(),
            track: Some(2),
            duration: Some(288.238),
            uri: "/tmp/audio/06 - J. Cole - Fire Squad(Explicit).m4a".to_owned(),
            album: Album {
                id: "ecd3fb5214ef4faf77a0eba4eedb2638".to_owned(),
                title: "2014 Forest Hills Drive".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        },
    ];

    cmd_tx
        .lock()
        .unwrap()
        .send(PlayerCommand::LoadTracklist {
            tracks,
            start_index: None,
        })
        .unwrap();

    sleep(Duration::from_millis(1000));

    let state = tracklist.lock().unwrap().playback_state();

    assert!(state.is_playing);

    cmd_tx.lock().unwrap().send(PlayerCommand::Pause).unwrap();

    sleep(Duration::from_millis(1000));

    let state = tracklist.lock().unwrap().playback_state();

    assert!(!state.is_playing);
}

#[test]
fn stop() {
    let (cmd_tx, cmd_rx, tracklist) = setup_new_params();

    let (_, _) = Player::new(
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let tracks = vec![
        Track {
            id: "2a81ab806a5d2bf9cad8917e7f89f1a5".to_owned(),
            title: "Wet Dreamz".to_owned(),
            artist: "J. Cole".to_owned(),
            track: Some(1),
            duration: Some(239.381),
            uri: "/tmp/audio/03 - J. Cole - Wet Dreamz(Explicit).m4a".to_owned(),
            album: Album {
                id: "ecd3fb5214ef4faf77a0eba4eedb2638".to_owned(),
                title: "2014 Forest Hills Drive".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        },
        Track {
            id: "d078aab608b47743781027a8881bf3cb".to_owned(),
            title: "Fire Squad".to_owned(),
            artist: "J. Cole".to_owned(),
            track: Some(2),
            duration: Some(288.238),
            uri: "/tmp/audio/06 - J. Cole - Fire Squad(Explicit).m4a".to_owned(),
            album: Album {
                id: "ecd3fb5214ef4faf77a0eba4eedb2638".to_owned(),
                title: "2014 Forest Hills Drive".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        },
    ];

    cmd_tx
        .lock()
        .unwrap()
        .send(PlayerCommand::LoadTracklist {
            tracks,
            start_index: None,
        })
        .unwrap();

    sleep(Duration::from_millis(1000));

    let state = tracklist.lock().unwrap().playback_state();

    assert!(state.is_playing);

    cmd_tx.lock().unwrap().send(PlayerCommand::Stop).unwrap();

    sleep(Duration::from_millis(2000));

    let state = tracklist.lock().unwrap().playback_state();

    assert!(!state.is_playing);
}

#[test]
fn next() {
    let (cmd_tx, cmd_rx, tracklist) = setup_new_params();

    let (_, _) = Player::new(
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let tracks = vec![
        Track {
            id: "2a81ab806a5d2bf9cad8917e7f89f1a5".to_owned(),
            title: "Wet Dreamz".to_owned(),
            artist: "J. Cole".to_owned(),
            track: Some(1),
            duration: Some(239.381),
            uri: "/tmp/audio/03 - J. Cole - Wet Dreamz(Explicit).m4a".to_owned(),
            album: Album {
                id: "ecd3fb5214ef4faf77a0eba4eedb2638".to_owned(),
                title: "2014 Forest Hills Drive".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        },
        Track {
            id: "d078aab608b47743781027a8881bf3cb".to_owned(),
            title: "Fire Squad".to_owned(),
            artist: "J. Cole".to_owned(),
            track: Some(2),
            duration: Some(288.238),
            uri: "/tmp/audio/06 - J. Cole - Fire Squad(Explicit).m4a".to_owned(),
            album: Album {
                id: "ecd3fb5214ef4faf77a0eba4eedb2638".to_owned(),
                title: "2014 Forest Hills Drive".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        },
    ];

    cmd_tx
        .lock()
        .unwrap()
        .send(PlayerCommand::LoadTracklist {
            tracks,
            start_index: None,
        })
        .unwrap();

    sleep(Duration::from_millis(1000));

    let (current, index) = tracklist.lock().unwrap().current_track();
    assert_eq!(current.unwrap().id, "2a81ab806a5d2bf9cad8917e7f89f1a5");
    assert_eq!(index, 1);

    cmd_tx.lock().unwrap().send(PlayerCommand::Next).unwrap();

    sleep(Duration::from_millis(1000));

    let (current, index) = tracklist.lock().unwrap().current_track();
    assert_eq!(current.unwrap().id, "d078aab608b47743781027a8881bf3cb");
    assert_eq!(index, 2);
}

#[test]
fn previous() {
    let (cmd_tx, cmd_rx, tracklist) = setup_new_params();

    let (_, _) = Player::new(
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let tracks = vec![
        Track {
            id: "2a81ab806a5d2bf9cad8917e7f89f1a5".to_owned(),
            title: "Wet Dreamz".to_owned(),
            artist: "J. Cole".to_owned(),
            track: Some(1),
            duration: Some(239.381),
            uri: "/tmp/audio/03 - J. Cole - Wet Dreamz(Explicit).m4a".to_owned(),
            album: Album {
                id: "ecd3fb5214ef4faf77a0eba4eedb2638".to_owned(),
                title: "2014 Forest Hills Drive".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        },
        Track {
            id: "d078aab608b47743781027a8881bf3cb".to_owned(),
            title: "Fire Squad".to_owned(),
            artist: "J. Cole".to_owned(),
            track: Some(2),
            duration: Some(288.238),
            uri: "/tmp/audio/06 - J. Cole - Fire Squad(Explicit).m4a".to_owned(),
            album: Album {
                id: "ecd3fb5214ef4faf77a0eba4eedb2638".to_owned(),
                title: "2014 Forest Hills Drive".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        },
    ];

    cmd_tx
        .lock()
        .unwrap()
        .send(PlayerCommand::LoadTracklist {
            tracks,
            start_index: None,
        })
        .unwrap();

    sleep(Duration::from_millis(1000));

    let (current, index) = tracklist.lock().unwrap().current_track();
    assert_eq!(current.unwrap().id, "2a81ab806a5d2bf9cad8917e7f89f1a5");
    assert_eq!(index, 1);

    cmd_tx.lock().unwrap().send(PlayerCommand::Next).unwrap();

    sleep(Duration::from_millis(1000));

    let (current, index) = tracklist.lock().unwrap().current_track();
    assert_eq!(current.unwrap().id, "d078aab608b47743781027a8881bf3cb");
    assert_eq!(index, 2);

    cmd_tx
        .lock()
        .unwrap()
        .send(PlayerCommand::Previous)
        .unwrap();

    sleep(Duration::from_millis(1000));

    let (current, index) = tracklist.lock().unwrap().current_track();
    assert_eq!(current.unwrap().id, "2a81ab806a5d2bf9cad8917e7f89f1a5");
    assert_eq!(index, 1);
}

#[test]
fn clear() {
    let (cmd_tx, cmd_rx, tracklist) = setup_new_params();

    let (_, _) = Player::new(
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let tracks = vec![
        Track {
            id: "2a81ab806a5d2bf9cad8917e7f89f1a5".to_owned(),
            title: "Wet Dreamz".to_owned(),
            artist: "J. Cole".to_owned(),
            track: Some(1),
            duration: Some(239.381),
            uri: "/tmp/audio/03 - J. Cole - Wet Dreamz(Explicit).m4a".to_owned(),
            album: Album {
                id: "ecd3fb5214ef4faf77a0eba4eedb2638".to_owned(),
                title: "2014 Forest Hills Drive".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        },
        Track {
            id: "d078aab608b47743781027a8881bf3cb".to_owned(),
            title: "Fire Squad".to_owned(),
            artist: "J. Cole".to_owned(),
            track: Some(2),
            duration: Some(288.238),
            uri: "/tmp/audio/06 - J. Cole - Fire Squad(Explicit).m4a".to_owned(),
            album: Album {
                id: "ecd3fb5214ef4faf77a0eba4eedb2638".to_owned(),
                title: "2014 Forest Hills Drive".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        },
    ];

    cmd_tx
        .lock()
        .unwrap()
        .send(PlayerCommand::LoadTracklist {
            tracks,
            start_index: None,
        })
        .unwrap();

    sleep(Duration::from_millis(1000));

    let (previous_tracks, next_tracks) = tracklist.lock().unwrap().tracks();

    assert_eq!(previous_tracks.len(), 1);
    assert_eq!(next_tracks.len(), 1);

    cmd_tx.lock().unwrap().send(PlayerCommand::Clear).unwrap();

    sleep(Duration::from_millis(1000));

    let (previous_tracks, next_tracks) = tracklist.lock().unwrap().tracks();

    assert_eq!(previous_tracks.len(), 0);
    assert_eq!(next_tracks.len(), 0);
}

#[test]
fn play_track_at() {
    let (cmd_tx, cmd_rx, tracklist) = setup_new_params();

    let (_, _) = Player::new(
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let tracks = vec![
        Track {
            id: "2a81ab806a5d2bf9cad8917e7f89f1a5".to_owned(),
            title: "Wet Dreamz".to_owned(),
            artist: "J. Cole".to_owned(),
            track: Some(1),
            duration: Some(239.381),
            uri: "/tmp/audio/03 - J. Cole - Wet Dreamz(Explicit).m4a".to_owned(),
            album: Album {
                id: "ecd3fb5214ef4faf77a0eba4eedb2638".to_owned(),
                title: "2014 Forest Hills Drive".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        },
        Track {
            id: "d078aab608b47743781027a8881bf3cb".to_owned(),
            title: "Fire Squad".to_owned(),
            artist: "J. Cole".to_owned(),
            track: Some(2),
            duration: Some(288.238),
            uri: "/tmp/audio/06 - J. Cole - Fire Squad(Explicit).m4a".to_owned(),
            album: Album {
                id: "ecd3fb5214ef4faf77a0eba4eedb2638".to_owned(),
                title: "2014 Forest Hills Drive".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        },
    ];

    cmd_tx
        .lock()
        .unwrap()
        .send(PlayerCommand::LoadTracklist {
            tracks,
            start_index: None,
        })
        .unwrap();

    sleep(Duration::from_millis(1000));

    let (current, index) = tracklist.lock().unwrap().current_track();
    assert_eq!(current.unwrap().id, "2a81ab806a5d2bf9cad8917e7f89f1a5");
    assert_eq!(index, 1);

    cmd_tx
        .lock()
        .unwrap()
        .send(PlayerCommand::PlayTrackAt(1))
        .unwrap();

    sleep(Duration::from_millis(1000));

    let (current, index) = tracklist.lock().unwrap().current_track();
    assert_eq!(current.unwrap().id, "d078aab608b47743781027a8881bf3cb");
    assert_eq!(index, 2);
}

#[test]
fn play_next() {
    let (cmd_tx, cmd_rx, tracklist) = setup_new_params();

    let (_, _) = Player::new(
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let tracks = vec![
        Track {
            id: "2a81ab806a5d2bf9cad8917e7f89f1a5".to_owned(),
            title: "Wet Dreamz".to_owned(),
            artist: "J. Cole".to_owned(),
            track: Some(1),
            duration: Some(239.381),
            uri: "/tmp/audio/03 - J. Cole - Wet Dreamz(Explicit).m4a".to_owned(),
            album: Album {
                id: "ecd3fb5214ef4faf77a0eba4eedb2638".to_owned(),
                title: "2014 Forest Hills Drive".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        },
        Track {
            id: "d078aab608b47743781027a8881bf3cb".to_owned(),
            title: "Fire Squad".to_owned(),
            artist: "J. Cole".to_owned(),
            track: Some(2),
            duration: Some(288.238),
            uri: "/tmp/audio/06 - J. Cole - Fire Squad(Explicit).m4a".to_owned(),
            album: Album {
                id: "ecd3fb5214ef4faf77a0eba4eedb2638".to_owned(),
                title: "2014 Forest Hills Drive".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        },
    ];

    cmd_tx
        .lock()
        .unwrap()
        .send(PlayerCommand::LoadTracklist {
            tracks,
            start_index: None,
        })
        .unwrap();

    sleep(Duration::from_millis(1000));

    let (current, index) = tracklist.lock().unwrap().current_track();
    assert_eq!(current.unwrap().id, "2a81ab806a5d2bf9cad8917e7f89f1a5");
    assert_eq!(index, 1);
    let (previous_tracks, next_tracks) = tracklist.lock().unwrap().tracks();
    assert_eq!(previous_tracks.len(), 1);
    assert_eq!(next_tracks.len(), 1);

    cmd_tx
        .lock()
        .unwrap()
        .send(PlayerCommand::PlayNext(Track {
            id: "d078aab608b47743781027a8881bf3cb".to_owned(),
            title: "Fire Squad".to_owned(),
            artist: "J. Cole".to_owned(),
            track: Some(2),
            duration: Some(288.238),
            uri: "/tmp/audio/06 - J. Cole - Fire Squad(Explicit).m4a".to_owned(),
            album: Album {
                id: "ecd3fb5214ef4faf77a0eba4eedb2638".to_owned(),
                title: "2014 Forest Hills Drive".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        }))
        .unwrap();

    sleep(Duration::from_millis(1000));

    let (previous_tracks, next_tracks) = tracklist.lock().unwrap().tracks();
    assert_eq!(previous_tracks.len(), 1);
    assert_eq!(next_tracks.len(), 2);
    assert_eq!(next_tracks[0].id, "d078aab608b47743781027a8881bf3cb");
}

#[test]
fn current_track() {
    let (cmd_tx, cmd_rx, tracklist) = setup_new_params();

    let (_, _) = Player::new(
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let tracks = vec![
        Track {
            id: "2a81ab806a5d2bf9cad8917e7f89f1a5".to_owned(),
            title: "Wet Dreamz".to_owned(),
            artist: "J. Cole".to_owned(),
            track: Some(1),
            duration: Some(239.381),
            uri: "/tmp/audio/03 - J. Cole - Wet Dreamz(Explicit).m4a".to_owned(),
            album: Album {
                id: "ecd3fb5214ef4faf77a0eba4eedb2638".to_owned(),
                title: "2014 Forest Hills Drive".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        },
        Track {
            id: "d078aab608b47743781027a8881bf3cb".to_owned(),
            title: "Fire Squad".to_owned(),
            artist: "J. Cole".to_owned(),
            track: Some(2),
            duration: Some(288.238),
            uri: "/tmp/audio/06 - J. Cole - Fire Squad(Explicit).m4a".to_owned(),
            album: Album {
                id: "ecd3fb5214ef4faf77a0eba4eedb2638".to_owned(),
                title: "2014 Forest Hills Drive".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        },
    ];

    cmd_tx
        .lock()
        .unwrap()
        .send(PlayerCommand::LoadTracklist {
            tracks,
            start_index: None,
        })
        .unwrap();

    sleep(Duration::from_millis(1000));

    let (current, index) = tracklist.lock().unwrap().current_track();
    assert_eq!(current.unwrap().id, "2a81ab806a5d2bf9cad8917e7f89f1a5");
    assert_eq!(index, 1);
}

#[test]
fn playback_state() {
    let (cmd_tx, cmd_rx, tracklist) = setup_new_params();

    let (_, _) = Player::new(
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let state = tracklist.lock().unwrap().playback_state();
    assert!(!state.is_playing);
}

/// What `Player::new` needs: the command channel's two ends, and the shared
/// tracklist.
type NewParams = (
    Arc<Mutex<UnboundedSender<PlayerCommand>>>,
    Arc<Mutex<UnboundedReceiver<PlayerCommand>>>,
    Arc<Mutex<Tracklist>>,
);

fn setup_new_params() -> NewParams {
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cmd_tx = Arc::new(Mutex::new(cmd_tx));
    let cmd_rx = Arc::new(Mutex::new(cmd_rx));
    let tracklist = Arc::new(Mutex::new(Tracklist::new_empty()));
    (cmd_tx, cmd_rx, tracklist)
}
