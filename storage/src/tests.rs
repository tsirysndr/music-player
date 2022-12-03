use super::*;
use music_player_types::types::{Album, Artist, Song};
use sea_orm::{ConnectionTrait, DbBackend};
use std::{env, time::Duration};
use tokio::time::sleep;

#[tokio::test]
async fn new_database() {
    env::set_var("MUSIC_PLAYER_DATABASE_URL", "sqlite::memory:");

    let db = Database::new().await;

    let conn = db.get_connection();
    assert_eq!(conn.get_database_backend(), DbBackend::Sqlite);
}

#[test]
fn insert_album() {
    env::set_var("MUSIC_PLAYER_APPLICATION_DIRECTORY", "/tmp");
    let searcher = Searcher::new();
    let album = Album {
        id: "27234641d4f5f9e0832affa79b9f62d8".to_owned(),
        title: "Eternal Atake".to_owned(),
        artist: "Lil Uzi Vert".to_owned(),
        artist_id: Some("0afe1226a5a75408acb57e97bd5feca1".to_owned()),
        ..Default::default()
    };

    searcher.insert_album(album).unwrap();
    assert!(searcher.search_album("Eternal").is_ok());
}

#[test]
fn insert_artist() {
    env::set_var("MUSIC_PLAYER_APPLICATION_DIRECTORY", "/tmp");
    let searcher = Searcher::new();
    let artist = Artist {
        id: "0afe1226a5a75408acb57e97bd5feca1".to_owned(),
        name: "Lil Uzi Vert".to_owned(),
        ..Default::default()
    };

    assert!(searcher.insert_artist(artist).is_ok());
}

#[test]
fn insert_track() {
    env::set_var("MUSIC_PLAYER_APPLICATION_DIRECTORY", "/tmp");
    let searcher = Searcher::new();
    let song = Song {
        title: "Futsal Shuffle 2020".to_owned(),
        album: "Eternal Atake".to_owned(),
        artist: "Lil Uzi Vert".to_owned(),
        ..Default::default()
    };

    assert!(searcher
        .insert_song(song, "27234641d4f5f9e0832affa79b9f62d8")
        .is_ok())
}

#[tokio::test]
async fn search_album() {
    env::set_var("MUSIC_PLAYER_APPLICATION_DIRECTORY", "/tmp/search_album");
    let searcher = Searcher::new();
    let album = Album {
        id: "27234641d4f5f9e0832affa79b9f62d8".to_owned(),
        title: "Eternal Atake".to_owned(),
        artist: "Lil Uzi Vert".to_owned(),
        artist_id: Some("0afe1226a5a75408acb57e97bd5feca1".to_owned()),
        ..Default::default()
    };
    searcher.insert_album(album).unwrap();

    sleep(Duration::from_secs(1)).await;

    let albums = searcher.search_album("eternal").unwrap();
    assert_eq!(albums.len(), 1);
}

#[tokio::test]
async fn search_artist() {
    env::set_var("MUSIC_PLAYER_APPLICATION_DIRECTORY", "/tmp/search_artist");
    let searcher = Searcher::new();
    let artist = Artist {
        id: "0afe1226a5a75408acb57e97bd5feca1".to_owned(),
        name: "Lil Uzi Vert".to_owned(),
        ..Default::default()
    };
    searcher.insert_artist(artist).unwrap();

    sleep(Duration::from_secs(1)).await;

    let artists = searcher.search_artist("uzi").unwrap();
    assert_eq!(artists.len(), 1);
}

#[tokio::test]
async fn search_track() {
    env::set_var("MUSIC_PLAYER_APPLICATION_DIRECTORY", "/tmp/search_track");
    let searcher = Searcher::new();
    let song = Song {
        title: "Futsal Shuffle 2020".to_owned(),
        album: "Eternal Atake".to_owned(),
        artist: "Lil Uzi Vert".to_owned(),
        ..Default::default()
    };

    searcher
        .insert_song(song, "27234641d4f5f9e0832affa79b9f62d8")
        .unwrap();

    sleep(Duration::from_secs(1)).await;

    let tracks = searcher.search_song("futsal").unwrap();
    assert_eq!(tracks.len(), 1);
}
