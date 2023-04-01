use futures_util::FutureExt;
use music_player_entity::{album, artist, artist_tracks, track};
use music_player_playback::{
    audio_backend::{self, rodio::RodioSink},
    config::AudioFormat,
    player::Player,
};
use music_player_scanner::scan_directory;
use music_player_storage::{searcher::Searcher, Database};
use music_player_tracklist::Tracklist;
use sea_orm::ActiveModelTrait;
use std::{env, sync::Arc, thread, time::Duration};
use surf::{Client, Config, Url};
use tokio::{runtime, sync::Mutex};

#[tokio::test]
async fn start_webui() {
    env::set_var("MUSIC_PLAYER_APPLICATION_DIRECTORY", "/tmp");
    env::set_var("MUSIC_PLAYER_MUSIC_DIRECTORY", "/tmp/audio");
    env::set_var(
        "MUSIC_PLAYER_DATABASE_URL",
        "sqlite:///tmp/music-player.sqlite3",
    );
    env::set_var("MUSIC_PLAYER_HTTP_PORT", "5054");

    let db_conn = Database::new().await;
    let searcher = Searcher::new();
    scan_music_directory(db_conn.clone(), searcher).await;

    let audio_format = AudioFormat::default();
    let backend = audio_backend::find(Some(RodioSink::NAME.to_string())).unwrap();
    let tracklist = Arc::new(std::sync::Mutex::new(Tracklist::new_empty()));
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cmd_tx = Arc::new(std::sync::Mutex::new(cmd_tx));
    let cmd_rx = Arc::new(std::sync::Mutex::new(cmd_rx));
    let cloned_tracklist = tracklist.clone();
    let cloned_cmd_tx = Arc::clone(&cmd_tx);
    let cloned_cmd_rx = Arc::clone(&cmd_rx);
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        move |_| {},
        cloned_cmd_tx,
        cloned_cmd_rx,
        cloned_tracklist,
    );

    thread::spawn(move || {
        let rt = runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async { super::start_webui(cmd_tx, tracklist).await })
            .unwrap();
    });

    thread::sleep(Duration::from_secs(5));

    let client: Client = Config::new()
        .set_base_url(Url::parse("http://localhost:5054").unwrap())
        .try_into()
        .unwrap();

    let res = client.get("/").await.unwrap();
    assert_eq!(res.status(), 200);

    let res = client.get("/graphiql").await.unwrap();
    assert_eq!(res.status(), 200);

    let payload = "{\"query\":\"\\n    query IntrospectionQuery {\\n      __schema {\\n        \\n        queryType { name }\\n        mutationType { name }\\n        subscriptionType { name }\\n        types {\\n          ...FullType\\n        }\\n        directives {\\n          name\\n          description\\n          \\n          locations\\n          args {\\n            ...InputValue\\n          }\\n        }\\n      }\\n    }\\n\\n    fragment FullType on __Type {\\n      kind\\n      name\\n      description\\n      \\n      fields(includeDeprecated: true) {\\n        name\\n        description\\n        args {\\n          ...InputValue\\n        }\\n        type {\\n          ...TypeRef\\n        }\\n        isDeprecated\\n        deprecationReason\\n      }\\n      inputFields {\\n        ...InputValue\\n      }\\n      interfaces {\\n        ...TypeRef\\n      }\\n      enumValues(includeDeprecated: true) {\\n        name\\n        description\\n        isDeprecated\\n        deprecationReason\\n      }\\n      possibleTypes {\\n        ...TypeRef\\n      }\\n    }\\n\\n    fragment InputValue on __InputValue {\\n      name\\n      description\\n      type { ...TypeRef }\\n      defaultValue\\n      \\n      \\n    }\\n\\n    fragment TypeRef on __Type {\\n      kind\\n      name\\n      ofType {\\n        kind\\n        name\\n        ofType {\\n          kind\\n          name\\n          ofType {\\n            kind\\n            name\\n            ofType {\\n              kind\\n              name\\n              ofType {\\n                kind\\n                name\\n                ofType {\\n                  kind\\n                  name\\n                  ofType {\\n                    kind\\n                    name\\n                  }\\n                }\\n              }\\n            }\\n          }\\n        }\\n      }\\n    }\\n  \",\"operationName\":\"IntrospectionQuery\"}";
    let res = surf::post("http://localhost:5054/graphql")
        .header("Content-Type", "application/json")
        .body(payload)
        .await
        .unwrap();

    assert_eq!(res.status(), 200);

    let res = client.get("/tracks").await.unwrap();
    assert_eq!(res.status(), 200);

    let res = client.get("/artists").await.unwrap();
    assert_eq!(res.status(), 200);

    let res = client.get("/albums").await.unwrap();
    assert_eq!(res.status(), 200);

    let res = client.get("/artists/").await.unwrap();
    assert_eq!(res.status(), 200);

    let res = client
        .get("/artists/f16ccc791352fbbffc11268b984db19a")
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let res = client
        .get("/albums/216ccc791352fbbffc11268b984db19a")
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let res = client
        .get("/folders/e2d0c9e2b0b4f8e4b7e4e4e4e4e4e4e4")
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let res = client
        .get("/playlists/f2d0c9e2b0b4f8e4b7e4e4e4e4e4e4e")
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let res = client.get("/search").await.unwrap();
    assert_eq!(res.status(), 200);

    let _res = client
        .get("/covers/216ccc791352fbbffc11268b984db19a.jpg")
        .await
        .unwrap();
    // assert_eq!(res.status(), 200);
}

async fn scan_music_directory(db: Database, searcher: Searcher) {
    scan_directory(
        move |song, db| {
            async move {
                let item: artist::ActiveModel = song.try_into().unwrap();
                match item.insert(db.get_connection()).await {
                    Ok(_) => (),
                    Err(_) => (),
                }

                let item: album::ActiveModel = song.try_into().unwrap();
                match item.insert(db.get_connection()).await {
                    Ok(_) => (),
                    Err(_) => (),
                }

                let item: track::ActiveModel = song.try_into().unwrap();

                match item.insert(db.get_connection()).await {
                    Ok(_) => (),
                    Err(_) => (),
                }

                let item: artist_tracks::ActiveModel = song.try_into().unwrap();
                match item.insert(db.get_connection()).await {
                    Ok(_) => (),
                    Err(_) => (),
                }
            }
            .boxed()
        },
        &db,
        &searcher,
    )
    .await
    .unwrap_or_default();
}
