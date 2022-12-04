use async_graphql::value;
use music_player_playback::player::Player;

use super::{new_folder, new_playlist, setup_schema};
use std::sync::Arc;

#[tokio::test]
async fn playlist() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let id = new_playlist(schema.clone()).await;

    let resp = schema
        .execute(format!(
            r#"
              query Playlist {{
                playlist(id: {}) {{
                  id
                  name
                }}
              }}
            "#,
            id
        ))
        .await;

    assert_eq!(resp.errors.len(), 0);
    assert_eq!(
        resp.data,
        value!({
          "playlist":
            {
              "id": id.replace("\"", ""),
              "name": "New Playlist"
            }
        })
    );
}

#[tokio::test]
async fn playlists() {
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
              query Playlists {
                playlists {
                  name
                }
              }
            "#,
        )
        .await;
    assert_eq!(resp.errors.len(), 0);
}

#[tokio::test]
async fn folder() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let id = new_folder(schema.clone()).await;

    let resp = schema
        .execute(format!(
            r#"
              query Folder {{
                folder(id: {}) {{
                  id
                  name
                }}
            }}"#,
            id
        ))
        .await;
    assert_eq!(resp.errors.len(), 0);
    assert_eq!(
        resp.data,
        value!({
          "folder":
            {
              "id": id.replace("\"", ""),
              "name": "New Folder"
            }
        })
    );
}

#[tokio::test]
async fn folders() {
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
              query Folders {
                folders {
                  name
                }
            }"#,
        )
        .await;
    assert_eq!(resp.errors.len(), 0);
}

#[tokio::test]
async fn create_playlist() {
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
              mutation CreatePlaylist {
                createPlaylist(name: "New Playlist") {
                  name
                }
            }"#,
        )
        .await;
    assert_eq!(resp.errors.len(), 0);
    assert_eq!(
        resp.data,
        value!({
          "createPlaylist":
            {
              "name": "New Playlist"
            }
        })
    );
}

#[tokio::test]
async fn delete_playlist() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
    let id = new_playlist(schema.clone()).await;
    let resp = schema
        .execute(format!(
            r#"
              mutation DeletePlaylist {{
                deletePlaylist(id: {}) {{
                  name
                }}
            }}"#,
            id
        ))
        .await;
    assert_eq!(resp.errors.len(), 0);
}

#[tokio::test]
async fn add_track_to_playlist() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}

#[tokio::test]
async fn remove_track_from_playlist() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
}

#[tokio::test]
async fn rename_playlist() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let id = new_playlist(schema.clone()).await;

    let resp = schema
        .execute(format!(
            r#"
            mutation RenamePlaylist {{
                renamePlaylist(id: {}, name: "New South") {{
                    id
                    name
                  }}
            }}
            "#,
            id
        ))
        .await;
    assert_eq!(resp.errors.len(), 0);
    assert_eq!(
        resp.data,
        value!({
          "renamePlaylist":
            {
              "id": id.replace("\"", ""),
              "name": "New South"
            }
        })
    );
}

#[tokio::test]
async fn create_folder() {
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
              mutation CreateFolder {
                createFolder(name: "New Folder") {
                  name
                }
            }"#,
        )
        .await;
    assert_eq!(resp.errors.len(), 0);
    assert_eq!(
        resp.data,
        value!({
          "createFolder":
            {
              "name": "New Folder"
            }
        })
    );
}

#[tokio::test]
async fn delete_folder() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
    let id = new_folder(schema.clone()).await;
    let resp = schema
        .execute(format!(
            r#"
              mutation DeleteFolder {{
                deleteFolder(id: {}) {{
                  id
                }}
            }}"#,
            id
        ))
        .await;
    assert_eq!(resp.errors.len(), 0);
    assert_eq!(
        resp.data,
        value!({
          "deleteFolder":
            {
              "id": id.replace("\"", "")
            }
        })
    );
}

#[tokio::test]
async fn rename_folder() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let id = new_folder(schema.clone()).await;

    let resp = schema
        .execute(format!(
            r#"
              mutation RenameFolder {{
                renameFolder(id: {}, name: "New South") {{
                  id
                  name
                }}
            }}"#,
            id
        ))
        .await;
    assert_eq!(resp.errors.len(), 0);
    assert_eq!(
        resp.data,
        value!({
          "renameFolder":
            {
              "id": id.replace("\"", ""),
              "name": "New South"
            }
        })
    );
}

#[tokio::test]
async fn move_playlist_to_folder() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );
    let playlist_id = new_playlist(schema.clone()).await;
    let folder_id = new_folder(schema.clone()).await;
    let resp = schema
        .execute(format!(
            r#"
              mutation MovePlaylistToFolder {{
                movePlaylistToFolder(id: {}, folderId: {}) {{
                  id
                  name
                }}
            }}"#,
            playlist_id, folder_id
        ))
        .await;
    assert_eq!(resp.errors.len(), 0);

    let resp = schema
        .execute(format!(
            r#"
          query {{
            folder(id: {}) {{
              id
              name
              playlists {{
                id
                name
              }}
            }}
          }}"#,
            folder_id
        ))
        .await;
    assert_eq!(resp.errors.len(), 0);
    assert_eq!(
        resp.data,
        value!({
          "folder":
            {
              "id": folder_id.replace("\"", ""),
              "name": "New Folder",
              "playlists":
                [{
                  "id": playlist_id.replace("\"", ""),
                  "name": "New Playlist"
                }]
            }
        })
    );
}

#[tokio::test]
async fn move_playlists_to_folder() {
    let (schema, cmd_tx, cmd_rx, tracklist, backend, audio_format) = setup_schema().await;
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        Arc::clone(&tracklist),
    );

    let playlist_id = new_playlist(schema.clone()).await;
    let folder_id = new_folder(schema.clone()).await;
    let resp = schema
        .execute(format!(
            r#"
              mutation MovePlaylistsToFolder {{
                movePlaylistsToFolder(ids: [{}], folderId: {}) {{
                  id
                  name
                }}
            }}"#,
            playlist_id, folder_id
        ))
        .await;
    assert_eq!(resp.errors.len(), 0);

    let resp = schema
        .execute(format!(
            r#"
          query {{
            folder(id: {}) {{
              id
              name
              playlists {{
                id
                name
              }}
            }}
          }}"#,
            folder_id
        ))
        .await;
    assert_eq!(resp.errors.len(), 0);
    assert_eq!(
        resp.data,
        value!({
          "folder":
            {
              "id": folder_id.replace("\"", ""),
              "name": "New Folder",
              "playlists":
                [{
                  "id": playlist_id.replace("\"", ""),
                  "name": "New Playlist"
                }]
            }
        })
    );
}
