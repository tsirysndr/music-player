use futures::future::FutureExt;
use music_player_entity::{album, artist, artist_tracks, track};
use music_player_storage::Database;
use sea_orm::{ActiveModelTrait, EntityTrait, PaginatorTrait};
use std::env;

#[tokio::test]
async fn scan_directory() {
    env::set_var("MUSIC_PLAYER_APPLICATION_DIRECTORY", "/tmp");
    env::set_var("MUSIC_PLAYER_MUSIC_DIRECTORY", "/tmp/audio");
    env::set_var(
        "MUSIC_PLAYER_DATABASE_URL",
        "sqlite:///tmp/music-player.sqlite3",
    );
    env::set_var("DATABASE_URL", "sqlite:///tmp/music-player.sqlite3");
    migration::run().await;
    super::scan_directory(move |song, db| {
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

    let db = Database::new().await;

    let conn = db.get_connection();
    assert_eq!(artist::Entity::find().count(conn).await.unwrap(), 1);
    assert_eq!(album::Entity::find().count(conn).await.unwrap(), 1);
    assert_eq!(track::Entity::find().count(conn).await.unwrap(), 2);
}
