#[cfg(test)]
mod tests;

use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use config::{Config, ConfigError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub database_url: String,
    pub port: u16,
    pub ws_port: u16,
    pub addons: Option<Vec<String>>,
    pub music_directory: String,
    pub host: String,
    pub device_name: String,
    pub device_id: String,
    pub http_port: u16,
    pub tauri_enable_graphql_server: bool,
}

pub fn read_settings() -> Result<Config, ConfigError> {
    let path = format!(
        "{}/music-player",
        dirs::config_dir().unwrap().to_str().unwrap()
    );
    let covers_path = format!("{}/covers", path);

    fs::create_dir_all(&covers_path).unwrap();

    let device_id = format!("{:x}", md5::compute(Uuid::new_v4().to_string()));

    let mut tmp = PathBuf::new();
    tmp.push("/tmp");

    let default_settings = Settings {
        database_url: format!("sqlite:{}/music-player.sqlite3", path),
        port: 5051,
        ws_port: 5052,
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
        music_directory: dirs::audio_dir()
            .unwrap_or(tmp)
            .to_str()
            .unwrap()
            .to_string(),
        host: "0.0.0.0".to_string(),
        device_name: "Music Player".to_string(),
        device_id,
        http_port: 5053,
        tauri_enable_graphql_server: false,
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
        .set_default("device_name", default_settings.device_name)?
        .set_default("device_id", default_settings.device_id)?
        .set_default("http_port", default_settings.http_port)?
        .set_default("tauri_enable_graphql_server", default_settings.tauri_enable_graphql_server)?
        .build()
}

pub fn get_application_directory() -> String {
    let path = env::var("MUSIC_PLAYER_APPLICATION_DIRECTORY").unwrap_or_else(|_| {
        format!(
            "{}/music-player",
            dirs::config_dir().unwrap().to_str().unwrap()
        )
    });
    let albums = format!("{}/albums", path);
    let artists = format!("{}/artists", path);
    let playlists = format!("{}/playlists", path);
    let tracks = format!("{}/tracks", path);
    fs::create_dir_all(&albums).unwrap();
    fs::create_dir_all(&artists).unwrap();
    fs::create_dir_all(&playlists).unwrap();
    fs::create_dir_all(&tracks).unwrap();

    path
}
