use std::path::{Path, PathBuf};

#[test]
fn read_settings() {
    let settings = super::read_settings().unwrap();
    let mut tmp = PathBuf::new();
    tmp.push("/tmp");
    assert_eq!(settings.get_string("host").unwrap(), "0.0.0.0");
    assert_eq!(settings.get_int("port").unwrap(), 5051);
    assert_eq!(settings.get_int("ws_port").unwrap(), 5052);
    assert_eq!(settings.get_int("http_port").unwrap(), 5053);
    assert_eq!(settings.get_string("device_name").unwrap(), "Music Player");
    assert_eq!(settings.get_string("device_id").unwrap().len(), 32);
    assert_eq!(settings.get_array("addons").unwrap().len(), 8);
    assert_eq!(
        settings.get_string("music_directory").unwrap(),
        dirs::audio_dir()
            .unwrap_or(tmp)
            .to_str()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        settings.get_string("database_url").unwrap(),
        format!(
            "sqlite:{}/music-player/music-player.sqlite3",
            dirs::config_dir().unwrap().to_str().unwrap()
        )
    );
}

#[test]
fn get_application_directory() {
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
}
