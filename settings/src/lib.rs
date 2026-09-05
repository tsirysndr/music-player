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
    /// Where decoded audio is sent: "cpal" (system audio device, default),
    /// "stdout", "fifo:/path/to/pipe", "unix:/path/to/socket" or "tcp:host:port".
    pub audio_output: String,
    /// How often (in minutes) the music directory is rescanned in the
    /// background. 0 disables periodic refresh.
    pub library_refresh_interval: u64,
    /// Base URL of a Subsonic-compatible server (Navidrome, Airsonic, gonic, ...).
    /// Empty or absent means the Subsonic integration is disabled.
    pub subsonic_url: Option<String>,
    pub subsonic_username: Option<String>,
    pub subsonic_password: Option<String>,
    /// Base URL of a Jellyfin server. Empty or absent means the Jellyfin
    /// integration is disabled.
    pub jellyfin_url: Option<String>,
    pub jellyfin_username: Option<String>,
    pub jellyfin_password: Option<String>,
}

pub fn read_settings() -> Result<Config, ConfigError> {
    let path = match env::consts::OS {
        "android" => "/storage/emulated/0/Android/data/com.tsirysndr.songbird/files".to_owned(),
        _ => {
            let config_dir = dirs::config_dir().unwrap();
            format!("{}/music-player", config_dir.to_str().unwrap())
        }
    };

    let music_directory = match env::consts::OS {
        "android" => "/storage/emulated/0/Music".to_owned(),
        _ => {
            let mut tmp = PathBuf::new();
            tmp.push("/tmp");
            let music_dir = dirs::audio_dir().unwrap_or(tmp);
            music_dir.to_str().unwrap().to_owned()
        }
    };

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
            "local".to_string(),
            "chromecast".to_string(),
            "dlna".to_string(),
            "subsonic".to_string(),
            "jellyfin".to_string(),
        ]),
        music_directory,
        host: "0.0.0.0".to_string(),
        device_name: "Music Player".to_string(),
        device_id,
        http_port: 5053,
        tauri_enable_graphql_server: false,
        audio_output: "cpal".to_string(),
        library_refresh_interval: 30,
        subsonic_url: Some("".to_string()),
        subsonic_username: Some("".to_string()),
        subsonic_password: Some("".to_string()),
        jellyfin_url: Some("".to_string()),
        jellyfin_username: Some("".to_string()),
        jellyfin_password: Some("".to_string()),
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
        .set_default(
            "tauri_enable_graphql_server",
            default_settings.tauri_enable_graphql_server,
        )?
        .set_default("audio_output", default_settings.audio_output)?
        .set_default(
            "library_refresh_interval",
            default_settings.library_refresh_interval,
        )?
        .set_default("subsonic_url", "")?
        .set_default("subsonic_username", "")?
        .set_default("subsonic_password", "")?
        .set_default("jellyfin_url", "")?
        .set_default("jellyfin_username", "")?
        .set_default("jellyfin_password", "")?
        .build()
}

pub fn get_application_directory() -> String {
    let config_dir = match env::consts::OS {
        "android" => "/storage/emulated/0/Android/data/com.tsirysndr.songbird/files".to_owned(),
        _ => {
            let config_dir = dirs::config_dir().unwrap();
            format!("{}/music-player", config_dir.to_str().unwrap())
        }
    };
    let path = env::var("MUSIC_PLAYER_APPLICATION_DIRECTORY").unwrap_or_else(|_| config_dir);
    let albums = format!("{}/albums", path);
    let artists = format!("{}/artists", path);
    let playlists = format!("{}/playlists", path);
    let tracks = format!("{}/tracks", path);
    let covers = format!("{}/covers", path);
    let cache = format!("{}/cache", path);
    fs::create_dir_all(&albums).unwrap();
    fs::create_dir_all(&artists).unwrap();
    fs::create_dir_all(&playlists).unwrap();
    fs::create_dir_all(&tracks).unwrap();
    fs::create_dir_all(&covers).unwrap();
    fs::create_dir_all(&cache).unwrap();

    path
}
