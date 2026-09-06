use async_graphql::*;
use futures_util::FutureExt;
use music_player_entity::{album, artist, artist_tracks, track};
use music_player_playback::player::Player;
use music_player_storage::Database;
use sea_orm::ActiveModelTrait;
use std::{sync::Arc, thread, time::Duration};

use super::setup_schema;

#[tokio::test]
async fn tracks() {
    let (schema, cmd_tx, cmd_rx, tracklist) = setup_schema().await;
    let (_, _) = Player::new(
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
    let resp = schema
        .execute(
            r#"
              query Tracks {
                tracks {
                    id
                    title
                    artist
                    trackNumber
                }
              }
            "#,
        )
        .await;
    assert_eq!(resp.errors.len(), 0);
    assert_eq!(
        resp.data,
        value!({
          "tracks": [
            {
              "id": "dd77dd0ea2de5208e4987001a59ba8e4",
              "title": "Fire Squad",
              "artist": "J. Cole",
              "trackNumber": 6
            },
            {
              "id": "3ac1f1651b6ef6d5f3f55b711e3bfcd1",
              "title": "Wet Dreamz",
              "artist": "J. Cole",
              "trackNumber": 3
            },
          ]
        })
    );
}

#[tokio::test]
async fn artists() {
    let (schema, cmd_tx, cmd_rx, tracklist) = setup_schema().await;
    let (_, _) = Player::new(
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
    let resp = schema
        .execute(
            r#"
              query Artists {
                artists {
                  id
                  name
                }
              }
            "#,
        )
        .await;
    assert_eq!(resp.errors.len(), 0);
    assert_eq!(
        resp.data,
        value!({
          "artists": [
            {
              "id": "b03cc90c455d92d8e9a0ce331e6de54d",
              "name": "J. Cole"
            },
          ]
        })
    );
}

#[tokio::test]
async fn albums() {
    let (schema, cmd_tx, cmd_rx, tracklist) = setup_schema().await;
    let (_, _) = Player::new(
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
    let resp = schema
        .execute(
            r#"
              query Albums {
                albums {
                    id
                    title
                    artist
                    year
                }
              }
            "#,
        )
        .await;
    assert_eq!(resp.errors.len(), 0);
    assert_eq!(
        resp.data,
        value!({
          "albums": [
            {
              "id": "ecd3fb5214ef4faf77a0eba4eedb2638",
              "title": "2014 Forest Hills Drive",
              "artist": "J. Cole",
              "year": 2014
            },
          ]
        })
    );
}

#[tokio::test]
async fn track() {
    let (schema, cmd_tx, cmd_rx, tracklist) = setup_schema().await;
    let (_, _) = Player::new(
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
    let resp = schema
        .execute(
            r#"
              query Track {
                track(id: "3ac1f1651b6ef6d5f3f55b711e3bfcd1") {
                    id
                    title
                    artist
                }
              }
            "#,
        )
        .await;
    assert_eq!(resp.errors.len(), 0);
    assert_eq!(
        resp.data,
        value!({
          "track": {
              "id": "3ac1f1651b6ef6d5f3f55b711e3bfcd1",
              "title": "Wet Dreamz",
              "artist": "J. Cole",
            }
        })
    );
    let resp = schema
        .execute(
            r#"
              query Track {
                track(id: "x") {
                    id
                    title
                    artist
                    trackNumber
                }
              }
            "#,
        )
        .await;
    assert_eq!(resp.errors.len(), 1);
}

#[tokio::test]
async fn artist() {
    let (schema, cmd_tx, cmd_rx, tracklist) = setup_schema().await;
    let (_, _) = Player::new(
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
    let resp = schema
        .execute(
            r#"
              query Artist {
                artist(id: "b03cc90c455d92d8e9a0ce331e6de54d") {
                  id
                  name
                }
              }
            "#,
        )
        .await;
    assert_eq!(resp.errors.len(), 0);
    assert_eq!(
        resp.data,
        value!({
          "artist":
            {
              "id": "b03cc90c455d92d8e9a0ce331e6de54d",
              "name": "J. Cole"
            }
        })
    );
    let resp = schema
        .execute(
            r#"
              query Artist {
                artist(id: "x") {
                  id
                  name
                }
              }
            "#,
        )
        .await;
    assert_eq!(resp.errors.len(), 1);
}

#[tokio::test]
async fn album() {
    let (schema, cmd_tx, cmd_rx, tracklist) = setup_schema().await;
    let (_, _) = Player::new(
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
    let resp = schema
        .execute(
            r#"
              query Album {
                album(id: "ecd3fb5214ef4faf77a0eba4eedb2638") {
                    id
                    title
                    artist
                    year
                }
              }
            "#,
        )
        .await;
    assert_eq!(resp.errors.len(), 0);
    assert_eq!(
        resp.data,
        value!({
          "album": {
            "id": "ecd3fb5214ef4faf77a0eba4eedb2638",
            "title": "2014 Forest Hills Drive",
            "artist": "J. Cole",
            "year": 2014
          }
        })
    );
    let resp = schema
        .execute(
            r#"
              query Album {
                album(id: "x") {
                    id
                    title
                    artist
                    year
                }
              }
            "#,
        )
        .await;
    assert_eq!(resp.errors.len(), 1);
}

#[tokio::test]
async fn search() {
    let (schema, cmd_tx, cmd_rx, tracklist) = setup_schema().await;
    let (_, _) = Player::new(
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let db = Database::new().await;
    music_player_scanner::scan_music_library(false, db)
        .await
        .unwrap_or_default();

    let resp = schema
        .execute(
            r#"
              query Search {
                search(keyword: "fire") {
                    artists {
                        id
                        name
                      }
                      tracks {
                        id
                        title
                        artist
                        duration
                      }
                      albums {
                        id
                        title
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
            "search": {
                "artists": [],
                "tracks": [
                  {
                    "id": "dd77dd0ea2de5208e4987001a59ba8e4",
                    "title": "Fire Squad",
                    "artist": "J. Cole",
                    "duration": 288.2380065917969
                  }
                ],
                "albums": []
              }
        })
    );
}
