//! Client-side liked-tracks store. The music-player daemon has no favorites
//! concept, so likes live with the desktop app: a JSON list of track ids in
//! the music-player config directory, resolved against the library cache.

use std::collections::HashSet;
use std::path::PathBuf;

fn file() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("music-player").join("desktop_liked.json"))
}

pub fn load() -> HashSet<String> {
    let Some(path) = file() else {
        return HashSet::new();
    };
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .map(|v| v.into_iter().collect())
        .unwrap_or_default()
}

pub fn save(ids: &HashSet<String>) {
    if let Some(path) = file() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut list: Vec<&String> = ids.iter().collect();
        list.sort();
        if let Ok(json) = serde_json::to_string_pretty(&list) {
            let _ = std::fs::write(path, json);
        }
    }
}
