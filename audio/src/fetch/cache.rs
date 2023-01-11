use anyhow::Error;
use music_player_settings::get_application_directory;
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};
pub struct Cache {
    cache_dir: String,
}

impl Cache {
    pub fn new() -> Self {
        let cache_dir = format!("{}/cache", get_application_directory());
        Self { cache_dir }
    }

    pub fn save_file<F: Read>(&self, name: &str, contents: &mut F) -> Result<(), Error> {
        if self.is_file_cached(name) {
            return Ok(());
        }
        let mut file = File::create(format!("{}/{}", self.cache_dir, name))?;
        io::copy(contents, &mut file)?;
        Ok(())
    }

    pub fn is_file_cached(&self, name: &str) -> bool {
        Path::new(&format!("{}/{}", self.cache_dir, name)).exists()
    }

    pub fn open_file(&self, name: &str) -> Result<File, Error> {
        Ok(File::open(format!("{}/{}", self.cache_dir, name))?)
    }
}
