use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use config::{Config, ConfigError};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub database_url: String,
    pub port: u16,
    pub addons: Option<Vec<String>>,
    pub music_directory: String,
}

pub fn read_settings() -> Result<Config, ConfigError> {
    let path = format!(
        "{}/music-player",
        dirs::config_dir().unwrap().to_str().unwrap()
    );
    fs::create_dir_all(&path).unwrap();

    let default_settings = Settings {
        database_url: format!("sqlite:/{}/music-player.sqlite3", path),
        port: 50051,
        addons: None,
        music_directory: dirs::audio_dir().unwrap().to_str().unwrap().to_string(),
    };

    let settings_path = format!("{}/settings.toml", path);

    if !Path::new(&settings_path).exists() {
        let mut file = File::create(format!("{}/settings.toml", path)).unwrap();
        file.write_all(
            toml::to_string_pretty(&default_settings)
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
    }

    Config::builder()
        .add_source(config::File::with_name(&settings_path))
        .add_source(config::Environment::with_prefix("MUSIC_PLAYER"))
        .build()
}
