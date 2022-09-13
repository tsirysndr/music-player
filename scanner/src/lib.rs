use lofty::{Accessor, AudioFile, LoftyError, Probe};
use types::Song;
use walkdir::WalkDir;

pub mod types;

pub fn scan_directory<F>(save: F) -> Result<Vec<Song>, LoftyError>
where
    F: Fn(&Song),
{
    let mut songs: Vec<Song> = Vec::new();
    for entry in WalkDir::new(dirs::audio_dir().unwrap())
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = format!("{}", entry.path().display());
        let guess = mime_guess::from_path(&path);

        if guess.first_or_octet_stream() == "audio/mpeg" {
            match Probe::open(&path)
                .expect("ERROR: Bad path provided!")
                .read(true)
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

                    let song = Song {
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
                    };
                    save(&song);
                    songs.push(song);
                }
                Err(e) => println!("ERROR: {}", e),
            }
        }
    }
    Ok(songs)
}
