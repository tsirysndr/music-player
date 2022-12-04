use async_graphql::value;
use music_player_playback::player::Player;
use std::{sync::Arc, thread, time::Duration};

use super::{play_album, setup_schema};

#[tokio::test]
async fn tracklist_tracks() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
    let resp = schema
        .execute(
            r#"
              query TracklistTracks {
                tracklistTracks {
                    previousTracks {
                        id
                    }
                    nextTracks {
                        id
                    }
                } 
              }
            "#,
        )
        .await;
    assert_eq!(resp.errors.len(), 0);
    assert_eq!(
        resp.data,
        value!({ "tracklistTracks": {
            "previousTracks": [],
            "nextTracks": [],
        }})
    );

    play_album(schema.clone()).await;

    thread::sleep(Duration::from_secs(1));

    let resp = schema
        .execute(
            r#"
              query TracklistTracks {
                tracklistTracks {
                    previousTracks {
                        id
                        title
                        artist
                        duration
                    }
                    nextTracks {
                        id
                        title
                        artist
                        duration
                    }
                } 
              }
            "#,
        )
        .await;
    assert_eq!(resp.errors.len(), 0);
    assert_eq!(
        resp.data,
        value!({
            "tracklistTracks": {
                "previousTracks": [
                      {
                        "id": "3ac1f1651b6ef6d5f3f55b711e3bfcd1",
                        "title": "Wet Dreamz",
                        "artist": "J. Cole",
                        "duration": 239.38099670410156
                      },
                ],
                "nextTracks": [
                    {
                        "id": "dd77dd0ea2de5208e4987001a59ba8e4",
                        "title": "Fire Squad",
                        "artist": "J. Cole",
                        "duration": 288.2380065917969
                      },
                ]
            }
        })
    );
}

#[tokio::test]
async fn add_track() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    play_album(schema.clone()).await;

    thread::sleep(Duration::from_secs(1));

    let resp = schema
        .execute(
            r#"
              mutation AddTrack {
                addTrack( 
                    track: {
                        id: "3ac1f1651b6ef6d5f3f55b711e3bfcd1",
                        title: "Wet Dreamz",
                        duration: 239.38099670410156,
                        discNumber: 1,
                        uri: ""
                    }) {
                    id
                    title
                    artist
                    duration
                }
              }
            "#,
        )
        .await;
    assert_eq!(resp.errors.len(), 0);

    thread::sleep(Duration::from_secs(1));

    let resp = schema
        .execute(
            r#"
              query TracklistTracks {
                tracklistTracks {
                    previousTracks {
                        id
                        title
                        artist
                        duration
                    }
                    nextTracks {
                        id
                        title
                        artist
                        duration
                    }
                } 
              }
            "#,
        )
        .await;

    assert_eq!(
        resp.data,
        value!({
            "tracklistTracks": {
                "previousTracks": [
                      {
                        "id": "3ac1f1651b6ef6d5f3f55b711e3bfcd1",
                        "title": "Wet Dreamz",
                        "artist": "J. Cole",
                        "duration": 239.38099670410156
                      },
                ],
                "nextTracks": [
                    {
                        "id": "dd77dd0ea2de5208e4987001a59ba8e4",
                        "title": "Fire Squad",
                        "artist": "J. Cole",
                        "duration": 288.2380065917969
                      },
                    {
                        "id": "3ac1f1651b6ef6d5f3f55b711e3bfcd1",
                        "title": "Wet Dreamz",
                        "artist": "J. Cole",
                        "duration": 239.38099670410156
                      },
                ]
            }
        })
    );
}

#[tokio::test]
async fn add_tracks() {
    let (_schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}

#[tokio::test]
async fn clear_tracklist() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    play_album(schema.clone()).await;

    thread::sleep(Duration::from_secs(1));

    let resp = schema
        .execute(
            r#"
              query TracklistTracks {
                tracklistTracks {
                    previousTracks {
                        id
                        title
                        artist
                        duration
                    }
                    nextTracks {
                        id
                        title
                        artist
                        duration
                    }
                } 
              }
            "#,
        )
        .await;
    assert_eq!(resp.errors.len(), 0);
    assert_eq!(
        resp.data,
        value!({
            "tracklistTracks": {
                "previousTracks": [
                      {
                        "id": "3ac1f1651b6ef6d5f3f55b711e3bfcd1",
                        "title": "Wet Dreamz",
                        "artist": "J. Cole",
                        "duration": 239.38099670410156
                      },
                ],
                "nextTracks": [
                    {
                        "id": "dd77dd0ea2de5208e4987001a59ba8e4",
                        "title": "Fire Squad",
                        "artist": "J. Cole",
                        "duration": 288.2380065917969
                      },
                ]
            }
        })
    );

    let resp = schema
        .execute(
            r#"
              mutation ClearTracklist {
                clearTracklist
              }
            "#,
        )
        .await;

    assert_eq!(resp.errors.len(), 0);

    thread::sleep(Duration::from_secs(1));

    let resp = schema
        .execute(
            r#"
              query TracklistTracks {
                tracklistTracks {
                    previousTracks {
                        id
                        title
                        artist
                        duration
                    }
                    nextTracks {
                        id
                        title
                        artist
                        duration
                    }
                } 
              }
            "#,
        )
        .await;

    assert_eq!(
        resp.data,
        value!({
            "tracklistTracks": {
                "previousTracks": [],
                "nextTracks": []
            }
        })
    );
}

#[tokio::test]
async fn remove_track() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
    play_album(schema.clone()).await;
    thread::sleep(Duration::from_secs(1));
    let resp = schema
        .execute(
            r#"
              mutation RemoveTrack {
                removeTrack(position: 1)
              }
            "#,
        )
        .await;
    assert_eq!(resp.errors.len(), 0);

    thread::sleep(Duration::from_secs(1));

    let resp = schema
        .execute(
            r#"
              query TracklistTracks {
                tracklistTracks {
                    previousTracks {
                        id
                        title
                        artist
                        duration
                    }
                    nextTracks {
                        id
                        title
                        artist
                        duration
                    }
                } 
              }
            "#,
        )
        .await;

    assert_eq!(
        resp.data,
        value!({
            "tracklistTracks": {
                "previousTracks": [
                      {
                        "id": "3ac1f1651b6ef6d5f3f55b711e3bfcd1",
                        "title": "Wet Dreamz",
                        "artist": "J. Cole",
                        "duration": 239.38099670410156
                      },
                ],
                "nextTracks": []
            }
        })
    );
}

#[tokio::test]
async fn remove_tracks() {
    let (_schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}

#[tokio::test]
async fn play_track_at() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
    play_album(schema.clone()).await;
    thread::sleep(Duration::from_secs(1));
    let resp = schema
        .execute(
            r#"
              mutation PlayTrackAt {
                playTrackAt(position: 1)
              }
            "#,
        )
        .await;
    assert_eq!(resp.errors.len(), 0);

    thread::sleep(Duration::from_secs(1));

    let resp = schema
        .execute(
            r#"
              query CurrentlyPlayingSong {
                currentlyPlayingSong {
                    track {
                        id
                        title
                        artist
                        duration
                        album {
                          id
                          title
                          artist
                          year
                          cover
                        }
                        artist
                      }
                      index
                      isPlaying
                }
              }
            "#,
        )
        .await;

    assert_eq!(resp.errors.len(), 0);
    assert_eq!(
        resp.data,
        value!({
            "currentlyPlayingSong": {
                "track": {
                    "id": "dd77dd0ea2de5208e4987001a59ba8e4",
                    "title": "Fire Squad",
                    "artist": "J. Cole",
                    "duration": 288.2380065917969,
                    "album": {
                        "id": "216ccc791352fbbffc11268b984db19a",
                        "title": "2014 Forest Hills Drive",
                        "artist": "J. Cole",
                        "year": 2014,
                        "cover": "216ccc791352fbbffc11268b984db19a.jpg",
                    },
                },
                "index": 2,
                "isPlaying": true
            }
        })
    );
}
