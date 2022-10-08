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
    pub ws_port: u16,
    pub addons: Option<Vec<String>>,
    pub music_directory: String,
    pub host: String,
}

pub fn read_settings() -> Result<Config, ConfigError> {
    let path = format!(
        "{}/music-player",
        dirs::config_dir().unwrap().to_str().unwrap()
    );
    fs::create_dir_all(&path).unwrap();

    let default_settings = Settings {
        database_url: format!("sqlite:{}/music-player.sqlite3", path),
        port: 50051,
        ws_port: 50052,
        addons: Some(vec![
            "deezer".to_string(),
            "datpiff".to_string(),
            "genius".to_string(),
            "local".to_string(),
            "myvazo".to_string(),
            "tononkira".to_string(),
            "musicbrainz".to_string(),
            "lastfm".to_string(),
        ]),
        music_directory: dirs::audio_dir().unwrap().to_str().unwrap().to_string(),
        host: "0.0.0.0".to_string(),
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
        .set_default("database_url", default_settings.database_url)?
        .set_default("port", default_settings.port)?
        .set_default("addons", default_settings.addons)?
        .set_default("ws_port", default_settings.ws_port)?
        .set_default("music_directory", default_settings.music_directory)?
        .set_default("host", default_settings.host)?
        .build()
}
