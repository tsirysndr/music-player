#[cfg(test)]
mod tests;

use anyhow::Error;
use futures::future::BoxFuture;
use music_player_storage::{searcher::Searcher, Database};
use music_player_types::types::{Album, Artist, Song};
use std::{io::Write, thread};

use lofty::{AudioFile, Probe, Tag};
use music_player_settings::{get_application_directory, read_settings, Settings};
use walkdir::WalkDir;

pub async fn scan_directory(
    save: impl for<'a> Fn(&'a Song, &'a Database) -> BoxFuture<'a, ()> + 'static,
    db: &Database,
    searcher: &Searcher,
) -> Result<Vec<Song>, Error> {
    let config = read_settings().unwrap();
    let settings = config.try_deserialize::<Settings>().unwrap();

    let mut songs: Vec<Song> = Vec::new();

    let supported_formats = vec![
        "audio/mpeg",
        "audio/mp4",
        // "audio/ogg",
        "audio/m4a",
        "audio/aac",
    ];

    let total = WalkDir::new(&settings.music_directory)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .count();

    let cloned_db = db.clone();
    let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
    let (tx, rx) = std::sync::mpsc::channel::<(Album, Song, Artist, usize)>();
    let searcher = searcher.clone();

    thread::spawn(move || {
        while let Ok((album, track, artist, index)) = rx.recv() {
            let id = format!("{:x}", md5::compute(track.uri.as_ref().unwrap()));
            match searcher.insert_artist(artist) {
                Ok(_) => {}
                Err(e) => println!("Error inserting artist: {}", e),
            };
            match searcher.insert_album(album) {
                Ok(_) => {}
                Err(e) => println!("Error inserting album: {}", e),
            };
            match searcher.insert_song(track, &id) {
                Ok(_) => {}
                Err(e) => println!("Error inserting song: {}", e),
            };

            if index + 1 == total {
                done_tx.send(()).unwrap();
                break;
            }
        }
    });

    for (index, entry) in WalkDir::new(&settings.music_directory)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .enumerate()
    {
        let path = format!("{}", entry.path().display());
        let guess = mime_guess::from_path(&path);
        let mime = guess.first_or_octet_stream();

        if supported_formats.iter().any(|x| x.to_owned() == mime) {
            match Probe::open(&path)
                .expect("ERROR: Bad path provided!")
                .read()
            {
                Ok(tagged_file) => {
                    let tag = match tagged_file.primary_tag() {
                        Some(primary_tag) => primary_tag,
                        // If the "primary" tag doesn't exist, we just grab the
                        // first tag we can find. Realistically, a tag reader would likely
                        // iterate through the tags to find a suitable one.
                        None => tagged_file.first_tag().expect("ERROR: No tags found!"),
                    };

                    let properties = tagged_file.properties();
                    let mut song: Song = tag.try_into().unwrap();
                    song.with_properties(properties);
                    song.uri = Some(path.clone());

                    let album = song.album.clone();
                    let cover = extract_and_save_album_cover(tag, &album);
                    song.cover = cover.clone();
                    save(&song, &cloned_db).await;
                    songs.push(song);

                    let mut track: Song = tag.try_into().unwrap();
                    track.uri = Some(path.clone());
                    track.cover = cover.clone();
                    track.with_properties(properties);

                    let artist: Artist = tag.try_into().unwrap();
                    let mut album: Album = tag.try_into().unwrap();
                    album.cover = cover.clone();

                    tx.send((album, track, artist, index)).unwrap();
                }
                Err(e) => println!("ERROR: {}, {}", e, path),
            }
        }
    }
    done_rx.recv().unwrap();
    Ok(songs)
}

fn extract_and_save_album_cover(tag: &Tag, album: &str) -> Option<String> {
    let pictures = tag.pictures();
    if pictures.len() > 0 {
        let covers_path = format!("{}/covers", get_application_directory());
        let picture = &pictures[0];
        let album = md5::compute(album.as_bytes());
        let filename = format!("{}/{:x}", covers_path, album);
        match picture.mime_type() {
            lofty::MimeType::Jpeg => {
                let filename = format!("{}.jpg", filename);
                let mut file = std::fs::File::create(filename).unwrap();
                file.write_all(picture.data()).unwrap();
                Some(format!("{:x}.jpg", album))
            }
            lofty::MimeType::Png => {
                let filename = format!("{}.png", filename);
                let mut file = std::fs::File::create(filename).unwrap();
                file.write_all(picture.data()).unwrap();
                Some(format!("{:x}.png", album))
            }
            _ => {
                println!("Unsupported picture format");
                None
            }
        }
    } else {
        None
    }
}
