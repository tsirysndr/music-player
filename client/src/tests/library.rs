use std::env;

use crate::library::LibraryClient;

#[tokio::test]
async fn album() -> Result<(), Box<dyn std::error::Error>> {
    let port = env::var("MUSIC_PLAYER_PORT").unwrap_or_else(|_| "50051".to_string());
    let host = env::var("MUSIC_PLAYER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let mut client = LibraryClient::new(host, port.parse().unwrap()).await?;
    let response = client.album("216ccc791352fbbffc11268b984db19a").await?;
    let response = response.unwrap();
    assert_eq!(response.id, "216ccc791352fbbffc11268b984db19a");
    assert_eq!(response.title, "2014 Forest Hills Drive");
    assert_eq!(response.artist, "J. Cole");
    assert_eq!(response.year, 2014);
    Ok(())
}

#[tokio::test]
async fn albums() -> Result<(), Box<dyn std::error::Error>> {
    let port = env::var("MUSIC_PLAYER_PORT").unwrap_or_else(|_| "50051".to_string());
    let host = "0.0.0.0".to_owned();
    let mut client = LibraryClient::new(host, port.parse().unwrap()).await?;
    let response = client.albums(0, 100).await?;
    assert_eq!(response.len(), 1);
    assert_eq!(response[0].id, "216ccc791352fbbffc11268b984db19a");
    assert_eq!(response[0].title, "2014 Forest Hills Drive");
    assert_eq!(response[0].artist, "J. Cole");
    assert_eq!(response[0].year, 2014);
    Ok(())
}

#[tokio::test]
async fn artist() -> Result<(), Box<dyn std::error::Error>> {
    let port = env::var("MUSIC_PLAYER_PORT").unwrap_or_else(|_| "50051".to_string());
    let host = env::var("MUSIC_PLAYER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let mut client = LibraryClient::new(host, port.parse().unwrap()).await?;
    let response = client.artist("b03cc90c455d92d8e9a0ce331e6de54d").await?;
    let response = response.unwrap();
    assert_eq!(response.id, "b03cc90c455d92d8e9a0ce331e6de54d");
    assert_eq!(response.name, "J. Cole");
    Ok(())
}

#[tokio::test]
async fn artists() -> Result<(), Box<dyn std::error::Error>> {
    let port = env::var("MUSIC_PLAYER_PORT").unwrap_or_else(|_| "50051".to_string());
    let host = env::var("MUSIC_PLAYER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let mut client = LibraryClient::new(host, port.parse().unwrap()).await?;
    let response = client.artists(0, 100).await?;
    assert_eq!(response.len(), 1);
    assert_eq!(response[0].id, "b03cc90c455d92d8e9a0ce331e6de54d");
    assert_eq!(response[0].name, "J. Cole");
    Ok(())
}

#[tokio::test]
async fn songs() -> Result<(), Box<dyn std::error::Error>> {
    let port = env::var("MUSIC_PLAYER_PORT").unwrap_or_else(|_| "50051".to_string());
    let host = env::var("MUSIC_PLAYER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let mut client = LibraryClient::new(host, port.parse().unwrap()).await?;
    let response = client.songs(0, 100).await?;
    assert_eq!(response.len(), 2);
    assert_eq!(response[0].id, "dd77dd0ea2de5208e4987001a59ba8e4");
    assert_eq!(response[0].title, "Fire Squad");
    assert_eq!(response[0].artist, "J. Cole");
    assert_eq!(
        response[0].album.as_ref().unwrap().title,
        "2014 Forest Hills Drive"
    );
    assert_eq!(response[1].id, "3ac1f1651b6ef6d5f3f55b711e3bfcd1");
    assert_eq!(response[1].title, "Wet Dreamz");
    assert_eq!(response[1].artist, "J. Cole");
    assert_eq!(
        response[1].album.as_ref().unwrap().title,
        "2014 Forest Hills Drive"
    );
    Ok(())
}
