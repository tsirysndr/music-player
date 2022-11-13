use std::{
    io::Write,
    sync::{Arc, Mutex},
    thread,
};

use futures::future::BoxFuture;
use music_player_storage::{
    searcher::{self, Searcher},
    Database,
};
use music_player_types::types::{Album, Artist, Song};

use lofty::{Accessor, AudioFile, ItemKey, LoftyError, Probe, Tag};
use music_player_settings::{read_settings, Settings};
use walkdir::WalkDir;

pub async fn scan_directory(
    save: impl for<'a> Fn(&'a Song, &'a Database) -> BoxFuture<'a, ()> + 'static,
) -> Result<Vec<Song>, LoftyError> {
    let db = Database::new().await;
    let searcher = Arc::new(Mutex::new(Searcher::new()));
    let config = read_settings().unwrap();
    let settings = config.try_deserialize::<Settings>().unwrap();

    let mut songs: Vec<Song> = Vec::new();

    for entry in WalkDir::new(settings.music_directory)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = format!("{}", entry.path().display());
        let guess = mime_guess::from_path(&path);
        let mime = guess.first_or_octet_stream();

        if mime == "audio/mpeg"
            || mime == "audio/mp4"
           // || mime == "audio/ogg"
            || mime == "audio/m4a"
            || mime == "audio/aac"
        {
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

                    let mut song = Song {
                        title: tag.title().unwrap_or("None").to_string(),
                        artist: tag.artist().unwrap_or("None").to_string(),
                        album: tag.album().unwrap_or("None").to_string(),
                        genre: tag.genre().unwrap_or("None").to_string(),
                        year: tag.year(),
                        track: tag.track(),
                        bitrate: properties.audio_bitrate(),
                        sample_rate: properties.sample_rate(),
                        bit_depth: properties.bit_depth(),
                        channels: properties.channels(),
                        duration: properties.duration(),
                        uri: Some(path.clone()),
                        cover: None,
                        album_artist: tag
                            .get_string(&ItemKey::AlbumArtist)
                            .unwrap_or(tag.artist().unwrap_or("None"))
                            .to_string(),
                    };
                    let album = song.album.clone();
                    song.cover = extract_and_save_album_cover(tag, &album);
                    save(&song, &db).await;
                    songs.push(song);

                    let track = Song {
                        artist: tag
                            .get_string(&ItemKey::AlbumArtist)
                            .unwrap_or(tag.artist().unwrap_or("None"))
                            .to_string(),
                        title: tag.title().unwrap_or("None").to_string(),
                        album: tag.album().unwrap_or("None").to_string(),
                        genre: tag.genre().unwrap_or("None").to_string(),
                        uri: Some(path.clone()),
                        ..Default::default()
                    };
                    let artist_id = format!(
                        "{:x}",
                        md5::compute(
                            tag.get_string(&ItemKey::AlbumArtist)
                                .unwrap_or(tag.artist().unwrap_or("None"))
                                .to_string()
                        )
                    );

                    let artist = Artist {
                        name: tag
                            .get_string(&ItemKey::AlbumArtist)
                            .unwrap_or(tag.artist().unwrap_or("None"))
                            .to_string(),
                        id: artist_id.clone(),
                        picture: Some("".to_string()),
                    };
                    let album_id = format!(
                        "{:x}",
                        md5::compute(tag.album().unwrap_or("None").to_string())
                    );
                    let album = Album {
                        title: tag.album().unwrap_or("None").to_string(),
                        artist: tag
                            .get_string(&ItemKey::AlbumArtist)
                            .unwrap_or(tag.artist().unwrap_or("None"))
                            .to_string(),
                        cover: extract_and_save_album_cover(tag, &track.album),
                        id: album_id.clone(),
                        artist_id: Some(artist_id),
                        year: tag.year(),
                    };
                    let artist_id = format!("{:x}", md5::compute(artist.name.clone()));
                    let id = format!("{:x}", md5::compute(track.uri.as_ref().unwrap()));
                    let searcher = Arc::clone(&searcher);
                    thread::spawn(move || {
                        match searcher.lock() {
                            Ok(searcher) => {
                                searcher.insert_artist(artist, &artist_id).unwrap();
                                searcher.insert_album(album, &album_id).unwrap();
                                searcher.insert_song(track, &id).unwrap();
                            }
                            Err(e) => println!("Error: {:?}", e),
                        };
                    });
                }
                Err(e) => println!("ERROR: {}", e),
            }
        }
    }
    Ok(songs)
}

fn extract_and_save_album_cover(tag: &Tag, album: &str) -> Option<String> {
    let pictures = tag.pictures();
    if pictures.len() > 0 {
        let covers_path = format!(
            "{}/music-player/covers",
            dirs::config_dir().unwrap().to_str().unwrap()
        );
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
