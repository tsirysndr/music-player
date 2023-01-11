use crate::schema::objects::track::Track;

use super::setup_schema;
use async_graphql::value;
use music_player_playback::player::Player;
use std::{sync::Arc, thread, time::Duration};

#[tokio::test]
async fn currently_playing_song() {
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
                "track": None::<Track>,
                "index": 0,
                "isPlaying": false
            }
        })
    );

    schema
        .execute(
            r#"
              mutation PlayAlbum {
                playAlbum(id: "216ccc791352fbbffc11268b984db19a", shuffle: false) 
              }
            "#,
        )
        .await;

    thread::sleep(Duration::from_secs(2));

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
                    "id": "3ac1f1651b6ef6d5f3f55b711e3bfcd1",
                    "title": "Wet Dreamz",
                    "artist": "J. Cole",
                    "duration": 239.38099670410156,
                    "album": {
                        "id": "216ccc791352fbbffc11268b984db19a",
                        "title": "2014 Forest Hills Drive",
                        "artist": "J. Cole",
                        "year": 2014,
                        "cover": "216ccc791352fbbffc11268b984db19a.jpg",
                    },
                },
                "index": 1,
                "isPlaying": true
            }
        })
    );
}

#[tokio::test]
async fn next() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    schema
        .execute(
            r#"
              mutation PlayAlbum {
                playAlbum(id: "216ccc791352fbbffc11268b984db19a", shuffle: false) 
              }
            "#,
        )
        .await;

    thread::sleep(Duration::from_secs(1));

    schema
        .execute(
            r#"
          mutation Next {
            next
          }
        "#,
        )
        .await;

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

#[tokio::test]
async fn previous() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    schema
        .execute(
            r#"
              mutation PlayAlbum {
                playAlbum(id: "216ccc791352fbbffc11268b984db19a", shuffle: false) 
              }
            "#,
        )
        .await;

    thread::sleep(Duration::from_secs(1));

    schema
        .execute(
            r#"
          mutation Next {
            next
          }
        "#,
        )
        .await;

    thread::sleep(Duration::from_secs(1));

    schema
        .execute(
            r#"
          mutation Previous {
            previous
          }
        "#,
        )
        .await;

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
                    "id": "3ac1f1651b6ef6d5f3f55b711e3bfcd1",
                    "title": "Wet Dreamz",
                    "artist": "J. Cole",
                    "duration": 239.38099670410156,
                    "album": {
                        "id": "216ccc791352fbbffc11268b984db19a",
                        "title": "2014 Forest Hills Drive",
                        "artist": "J. Cole",
                        "year": 2014,
                        "cover": "216ccc791352fbbffc11268b984db19a.jpg",
                    },
                },
                "index": 1,
                "isPlaying": true
            }
        })
    );
}

#[tokio::test]
async fn play() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    schema
        .execute(
            r#"
              mutation PlayAlbum {
                playAlbum(id: "216ccc791352fbbffc11268b984db19a", shuffle: false) 
              }
            "#,
        )
        .await;

    thread::sleep(Duration::from_secs(1));

    schema
        .execute(
            r#"
          mutation Pause {
            pause
          }
        "#,
        )
        .await;

    thread::sleep(Duration::from_secs(1));

    schema
        .execute(
            r#"
          mutation Play {
            play
          }
        "#,
        )
        .await;

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
                    "id": "3ac1f1651b6ef6d5f3f55b711e3bfcd1",
                    "title": "Wet Dreamz",
                    "artist": "J. Cole",
                    "duration": 239.38099670410156,
                    "album": {
                        "id": "216ccc791352fbbffc11268b984db19a",
                        "title": "2014 Forest Hills Drive",
                        "artist": "J. Cole",
                        "year": 2014,
                        "cover": "216ccc791352fbbffc11268b984db19a.jpg",
                    },
                },
                "index": 1,
                "isPlaying": true
            }
        })
    );
}

#[tokio::test]
async fn pause() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    schema
        .execute(
            r#"
              mutation PlayAlbum {
                playAlbum(id: "216ccc791352fbbffc11268b984db19a", shuffle: false) 
              }
            "#,
        )
        .await;

    thread::sleep(Duration::from_secs(1));

    schema
        .execute(
            r#"
          mutation Pause {
            pause
          }
        "#,
        )
        .await;

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
                    "id": "3ac1f1651b6ef6d5f3f55b711e3bfcd1",
                    "title": "Wet Dreamz",
                    "artist": "J. Cole",
                    "duration": 239.38099670410156,
                    "album": {
                        "id": "216ccc791352fbbffc11268b984db19a",
                        "title": "2014 Forest Hills Drive",
                        "artist": "J. Cole",
                        "year": 2014,
                        "cover": "216ccc791352fbbffc11268b984db19a.jpg",
                    },
                },
                "index": 1,
                "isPlaying": false
            }
        })
    );
}

#[tokio::test]
async fn stop() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    schema
        .execute(
            r#"
          mutation Stop {
            stop
          }
        "#,
        )
        .await;

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
                "track": None::<Track>,
                "index": 0,
                "isPlaying": false
            }
        })
    );
}
