use music_player_entity::{album, artist, track};
use music_player_storage::Database;
use sea_orm::{EntityTrait, PaginatorTrait};
use std::env;

#[tokio::test]
async fn scan_music_library() {
    env::set_var("MUSIC_PLAYER_APPLICATION_DIRECTORY", "/tmp");
    env::set_var("MUSIC_PLAYER_MUSIC_DIRECTORY", "/tmp/audio");
    env::set_var(
        "MUSIC_PLAYER_DATABASE_URL",
        "sqlite:///tmp/music-player.sqlite3",
    );
    migration::apply().await;
    let db = Database::new().await;
    let songs = super::scan_music_library(false, db.clone())
        .await
        .unwrap_or_default();
    assert_eq!(songs.len(), 2);

    let conn = db.get_connection();
    assert_eq!(artist::Entity::find().count(conn).await.unwrap(), 1);
    assert_eq!(album::Entity::find().count(conn).await.unwrap(), 1);
    assert_eq!(track::Entity::find().count(conn).await.unwrap(), 2);
}
