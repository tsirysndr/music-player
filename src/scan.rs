use anyhow::Error;
use futures::future::FutureExt;
use music_player_entity::{album, artist, artist_tracks, track};
use music_player_scanner::scan_directory;
use music_player_storage::{searcher::Searcher, Database};
use music_player_types::types::Song;
use owo_colors::OwoColorize;
use sea_orm::{ActiveModelTrait, EntityTrait};

pub async fn auto_scan_music_library(db: Database, searcher: Searcher) {
    match track::Entity::find().all(db.clone().get_connection()).await {
        Ok(result) => {
            if result.len() == 0 {
                scan_music_library(false, db, searcher)
                    .await
                    .unwrap_or_default();
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

pub async fn scan_music_library(
    enable_log: bool,
    db: Database,
    searcher: Searcher,
) -> Result<Vec<Song>, Error> {
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

                if enable_log {
                    let filename = song.uri.as_ref().unwrap().split("/").last().unwrap();
                    let path = song.uri.as_ref().unwrap().replace(filename, "");
                    println!("{}{}", path, filename.magenta());
                }

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
}
