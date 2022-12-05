use futures_util::FutureExt;
use music_player_entity::{album, artist, artist_tracks, track};
use music_player_playback::{
    audio_backend::{self, rodio::RodioSink},
    config::AudioFormat,
    player::Player,
};
use music_player_scanner::scan_directory;
use music_player_tracklist::Tracklist;
use sea_orm::ActiveModelTrait;
use std::{env, sync::Arc, thread, time::Duration};
use surf::{Client, Config, Url};
use tokio::runtime;

#[tokio::test]
async fn start_webui() {
    env::set_var("MUSIC_PLAYER_APPLICATION_DIRECTORY", "/tmp");
    env::set_var("MUSIC_PLAYER_MUSIC_DIRECTORY", "/tmp/audio");
    env::set_var(
        "MUSIC_PLAYER_DATABASE_URL",
        "sqlite:///tmp/music-player.sqlite3",
    );

    scan_music_directory().await;

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

    thread::sleep(Duration::from_secs(2));

    let client: Client = Config::new()
        .set_base_url(Url::parse("http://localhost:5051").unwrap())
        .try_into()
        .unwrap();

    let res = client.get("/").await.unwrap();
    assert_eq!(res.status(), 200);

    let res = client.get("/graphiql").await.unwrap();
    assert_eq!(res.status(), 200);

    let payload = r#"{
      "operationName": "IntrospectionQuery"
      "query": "\n    query IntrospectionQuery {\n      __schema {\n        \n        queryType { name }\n        mutationType { name }\n        subscriptionType { name }\n        types {\n          ...FullType\n        }\n        directives {\n          name\n          description\n          \n          locations\n          args {\n            ...InputValue\n          }\n        }\n      }\n    }\n\n    fragment FullType on __Type {\n      kind\n      name\n      description\n      \n      fields(includeDeprecated: true) {\n        name\n        description\n        args {\n          ...InputValue\n        }\n        type {\n          ...TypeRef\n        }\n        isDeprecated\n        deprecationReason\n      }\n      inputFields {\n        ...InputValue\n      }\n      interfaces {\n        ...TypeRef\n      }\n      enumValues(includeDeprecated: true) {\n        name\n        description\n        isDeprecated\n        deprecationReason\n      }\n      possibleTypes {\n        ...TypeRef\n      }\n    }\n\n    fragment InputValue on __InputValue {\n      name\n      description\n      type { ...TypeRef }\n      defaultValue\n      \n      \n    }\n\n    fragment TypeRef on __Type {\n      kind\n      name\n      ofType {\n        kind\n        name\n        ofType {\n          kind\n          name\n          ofType {\n            kind\n            name\n            ofType {\n              kind\n              name\n              ofType {\n                kind\n                name\n                ofType {\n                  kind\n                  name\n                  ofType {\n                    kind\n                    name\n                  }\n                }\n              }\n            }\n          }\n        }\n      }\n    }\n  "
    }"#;

    let res = surf::post("http://localhost:5051/graphql")
        .header("Accept", "application/json")
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
        .get("/albums/216ccc791352fbbffc11268b984db19a")
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let res = client
        .get("/tracks/dd77dd0ea2de5208e4987001a59ba8e4")
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

    let res = client
        .get("/covers/216ccc791352fbbffc11268b984db19a.jpg")
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
}

async fn scan_music_directory() {
    scan_directory(move |song, db| {
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
    })
    .await
    .unwrap_or_default();
}
