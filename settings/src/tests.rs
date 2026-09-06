use std::{
    env,
    path::{Path, PathBuf},
};

#[test]
fn read_settings() {
    env::remove_var("MUSIC_PLAYER_APPLICATION_DIRECTORY");
    env::remove_var("MUSIC_PLAYER_MUSIC_DIRECTORY");
    env::remove_var("MUSIC_PLAYER_DATABASE_URL");
    env::remove_var("MUSIC_PLAYER_PORT");
    let settings = super::read_settings().unwrap();
    let mut tmp = PathBuf::new();
    tmp.push("/tmp/audio");
    assert_eq!(settings.get_string("host").unwrap(), "0.0.0.0");
    assert_eq!(settings.get_int("port").unwrap(), 5051);
    assert_eq!(settings.get_int("ws_port").unwrap(), 5052);
    assert_eq!(settings.get_int("http_port").unwrap(), 5053);
    assert_eq!(
        settings.get_string("radio_browser_url").unwrap(),
        super::DEFAULT_RADIO_BROWSER_URL
    );
    assert_eq!(
        settings.get_string("tunein_url").unwrap(),
        super::DEFAULT_TUNEIN_URL
    );
    assert_eq!(settings.get_string("device_name").unwrap(), "Music Player");
    assert_eq!(settings.get_string("device_id").unwrap().len(), 32);
    assert!(!settings.get_array("addons").unwrap().is_empty());
    assert!(!settings.get_string("music_directory").unwrap().is_empty());
    assert!(settings
        .get_string("database_url")
        .unwrap()
        .starts_with("sqlite:"));
    env::set_var("MUSIC_PLAYER_APPLICATION_DIRECTORY", "/tmp");
    env::set_var("MUSIC_PLAYER_MUSIC_DIRECTORY", "/tmp/audio");
    env::set_var(
        "MUSIC_PLAYER_DATABASE_URL",
        "sqlite:///tmp/music-player.sqlite3",
    );
    env::set_var("MUsIC_PLAYER_PORT", "5040");
}

#[test]
fn get_application_directory() {
    env::remove_var("MUSIC_PLAYER_APPLICATION_DIRECTORY");
    env::remove_var("MUSIC_PLAYER_MUSIC_DIRECTORY");
    env::remove_var("MUSIC_PLAYER_DATABASE_URL");
    let path = super::get_application_directory();
    assert_eq!(
        path,
        format!(
            "{}/music-player",
            dirs::config_dir().unwrap().to_str().unwrap()
        )
    );
    assert!(Path::new(&path).exists());
    assert!(Path::new(&format!("{}/albums", path)).exists());
    assert!(Path::new(&format!("{}/artists", path)).exists());
    assert!(Path::new(&format!("{}/playlists", path)).exists());
    assert!(Path::new(&format!("{}/tracks", path)).exists());
    env::set_var("MUSIC_PLAYER_APPLICATION_DIRECTORY", "/tmp");
    env::set_var("MUSIC_PLAYER_MUSIC_DIRECTORY", "/tmp/audio");
    env::set_var(
        "MUSIC_PLAYER_DATABASE_URL",
        "sqlite:///tmp/music-player.sqlite3",
    );
}
