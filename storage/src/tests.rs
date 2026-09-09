use super::*;
use crate::repo::track::TrackRepository;
use crate::searcher::Searcher;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use std::env;

#[tokio::test]
async fn new_database() {
    env::set_var("MUSIC_PLAYER_DATABASE_URL", "sqlite::memory:");

    let db = Database::new().await;

    let conn = db.get_connection();
    assert_eq!(conn.get_database_backend(), DbBackend::Sqlite);
}

async fn setup_searcher() -> (tempfile::TempDir, Searcher) {
    // These test the built-in index. Without this the searcher would answer
    // from whatever Typesense the developer's settings point at, describing a
    // different library than the fixture below.
    env::set_var(music_player_settings::LOCAL_SEARCH_ONLY, "1");
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("music-player.sqlite3");
    let url = format!("sqlite://{}?mode=rwc", db_path.display());
    let conn = sea_orm::Database::connect(&url).await.unwrap();
    Migrator::up(&conn, None).await.unwrap();

    let statements = [
        "INSERT INTO artist (id, name) VALUES ('0afe1226a5a75408acb57e97bd5feca1', 'Lil Uzi Vert')",
        r#"INSERT INTO album (id, title, artist, artist_id, year, cover)
            VALUES ('27234641d4f5f9e0832affa79b9f62d8', 'Eternal Atake', 'Lil Uzi Vert', '0afe1226a5a75408acb57e97bd5feca1', 2020, 'cover.jpg')"#,
        r#"INSERT INTO track (id, title, artist, genre, duration, uri, album_id, artist_id)
            VALUES ('3ac1f226a5a75408acb57e97bd5feca2', 'Futsal Shuffle 2020', 'Lil Uzi Vert', 'Hip Hop', 192.5, '/tmp/audio/futsal.mp3', '27234641d4f5f9e0832affa79b9f62d8', '0afe1226a5a75408acb57e97bd5feca1')"#,
    ];
    for sql in statements {
        conn.execute(Statement::from_string(DbBackend::Sqlite, sql.to_string()))
            .await
            .unwrap();
    }

    (dir, Searcher::new(conn))
}

/// A genre listing has to reach tracks by *either* route: the file's own tag
/// or the artist's. Most files carry no usable genre tag, so a listing built
/// on tags alone is nearly empty, and one built on artists alone mis-files
/// every compilation.
#[tokio::test]
async fn genres_reach_tracks_by_tag_and_by_artist() {
    use crate::repo::genre::GenreRepository;

    let (_dir, searcher) = setup_searcher().await;
    let conn = searcher.get_connection();

    // "Hip Hop" reaches the track through its own tag; "Trap" only through
    // the artist, which is the case the artist link exists for.
    for sql in [
        "INSERT INTO genre (id, name) VALUES ('g1', 'Hip Hop')",
        "INSERT INTO genre (id, name) VALUES ('g2', 'Trap')",
        "INSERT INTO genre (id, name) VALUES ('g3', 'Polka')",
        r#"INSERT INTO track_genres (id, track_id, genre_id)
            VALUES ('tg1', '3ac1f226a5a75408acb57e97bd5feca2', 'g1')"#,
        r#"INSERT INTO artist_genres (id, artist_id, genre_id)
            VALUES ('ag1', '0afe1226a5a75408acb57e97bd5feca1', 'g2')"#,
    ] {
        conn.execute(Statement::from_string(DbBackend::Sqlite, sql.to_string()))
            .await
            .unwrap();
    }

    let genres = GenreRepository::new(conn)
        .find_all(None, None)
        .await
        .unwrap();
    let names: Vec<&str> = genres.iter().map(|g| g.name.as_str()).collect();
    assert!(names.contains(&"Hip Hop"), "{names:?}");
    assert!(names.contains(&"Trap"), "reached via the artist: {names:?}");
    // A genre nothing reaches is omitted rather than shown as empty.
    assert!(!names.contains(&"Polka"), "{names:?}");

    for id in ["g1", "g2"] {
        let tracks = GenreRepository::new(conn)
            .tracks(id, None, None)
            .await
            .unwrap();
        assert_eq!(tracks.len(), 1, "{id} should reach the track");
        assert_eq!(tracks[0].title, "Futsal Shuffle 2020");
    }
}

#[tokio::test]
async fn search_song_with_fts5() {
    let (_dir, searcher) = setup_searcher().await;

    let tracks = searcher.search_song("futsal").await.unwrap();
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].id, "3ac1f226a5a75408acb57e97bd5feca2");
    assert_eq!(tracks[0].title, "Futsal Shuffle 2020");
    assert_eq!(tracks[0].artist, "Lil Uzi Vert");
    assert_eq!(tracks[0].album, "Eternal Atake");
    assert_eq!(tracks[0].album_id, "27234641d4f5f9e0832affa79b9f62d8");
    assert_eq!(tracks[0].artist_id, "0afe1226a5a75408acb57e97bd5feca1");
    assert_eq!(tracks[0].cover, Some("cover.jpg".to_string()));
    assert_eq!(tracks[0].duration.as_secs(), 192);

    // prefix match
    let tracks = searcher.search_song("fut").await.unwrap();
    assert_eq!(tracks.len(), 1);

    // match on album title via the FTS index
    let tracks = searcher.search_song("eternal").await.unwrap();
    assert_eq!(tracks.len(), 1);
}

#[tokio::test]
async fn search_album_with_fts5() {
    let (_dir, searcher) = setup_searcher().await;

    let albums = searcher.search_album("eternal").await.unwrap();
    assert_eq!(albums.len(), 1);
    assert_eq!(albums[0].id, "27234641d4f5f9e0832affa79b9f62d8");
    assert_eq!(albums[0].title, "Eternal Atake");
    assert_eq!(albums[0].artist, "Lil Uzi Vert");
    assert_eq!(albums[0].year, Some(2020));
    assert_eq!(albums[0].cover, Some("cover.jpg".to_string()));
}

#[tokio::test]
async fn search_artist_with_fts5() {
    let (_dir, searcher) = setup_searcher().await;

    let artists = searcher.search_artist("uzi").await.unwrap();
    assert_eq!(artists.len(), 1);
    assert_eq!(artists[0].id, "0afe1226a5a75408acb57e97bd5feca1");
    assert_eq!(artists[0].name, "Lil Uzi Vert");
}

#[tokio::test]
async fn search_handles_empty_and_hostile_input() {
    let (_dir, searcher) = setup_searcher().await;

    assert!(searcher.search_song("").await.unwrap().is_empty());
    assert!(searcher.search_song("   ").await.unwrap().is_empty());
    assert!(searcher
        .search_song("zzzznomatch")
        .await
        .unwrap()
        .is_empty());

    // FTS5 query syntax must never be interpreted
    assert!(searcher
        .search_song("\" OR 1; DROP TABLE track --")
        .await
        .is_ok());
    assert!(searcher.search_album("NEAR(a b)").await.is_ok());
    assert!(searcher.search_artist("col:value*").await.is_ok());
}

#[tokio::test]
async fn delete_removes_row_from_index() {
    let (_dir, searcher) = setup_searcher().await;

    searcher
        .get_connection()
        .execute(Statement::from_string(
            DbBackend::Sqlite,
            "DELETE FROM track WHERE id = '3ac1f226a5a75408acb57e97bd5feca2'".to_string(),
        ))
        .await
        .unwrap();

    assert!(searcher.search_song("futsal").await.unwrap().is_empty());
}

#[tokio::test]
async fn track_repository_matches_albums_by_id_when_titles_are_equal() {
    let dir = tempfile::tempdir().unwrap();
    let url = format!(
        "sqlite://{}?mode=rwc",
        dir.path().join("albums.sqlite3").display()
    );
    let conn = sea_orm::Database::connect(&url).await.unwrap();
    Migrator::up(&conn, None).await.unwrap();

    for sql in [
        "INSERT INTO artist (id, name) VALUES ('muse', 'Muse'), ('chris', 'Chris Brown')",
        "INSERT INTO album (id, title, artist, artist_id) VALUES ('muse-album', 'The 2nd Law', 'Muse', 'muse'), ('brown-album', 'BROWN (The Chocolate Edition)', 'Chris Brown', 'chris')",
        "INSERT INTO track (id, title, artist, genre, uri, album_id, artist_id) VALUES ('muse-track', 'Save Me', 'Muse', 'Rock', '/music/muse.flac', 'muse-album', 'muse'), ('brown-track', 'Save Me', 'Chris Brown', 'R&B', '/music/brown.flac', 'brown-album', 'chris')",
    ] {
        conn.execute(Statement::from_string(DbBackend::Sqlite, sql.to_owned()))
            .await
            .unwrap();
    }

    let tracks = TrackRepository::new(&conn)
        .find_all(None, None, 100)
        .await
        .unwrap();
    let muse = tracks
        .iter()
        .find(|track| track.id == "muse-track")
        .unwrap();
    let brown = tracks
        .iter()
        .find(|track| track.id == "brown-track")
        .unwrap();

    assert_eq!(muse.album.id, "muse-album");
    assert_eq!(muse.album.artist, "Muse");
    assert_eq!(brown.album.id, "brown-album");
    assert_eq!(brown.album.artist, "Chris Brown");
}

/// `shared()` must hand back one pool, not a new one per call: a pool costs a
/// file descriptor per connection, and anything calling it on a timer would
/// otherwise run the process out of descriptors ("Too many open files").
#[tokio::test]
async fn shared_database_is_opened_once() {
    env::set_var("MUSIC_PLAYER_DATABASE_URL", "sqlite::memory:");

    let first = crate::shared().await;
    let second = crate::shared().await;

    assert!(
        std::ptr::eq(first, second),
        "shared() opened a second pool instead of reusing the first"
    );
}
