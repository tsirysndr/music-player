//! Client-side liked-tracks store. The music-player daemon has no favorites
//! concept, so likes live with the desktop app: a JSON list of track ids in
//! the music-player config directory, resolved against the library cache.
//!
//! The list is ORDERED — most recently liked first — and that order is the
//! order the Liked screen shows and the play-liked queue loads. (Files written
//! by older builds were sorted alphabetically; their order is kept as found
//! and corrects itself as new likes land on top.)

use std::path::PathBuf;

fn file() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("music-player").join("desktop_liked.json"))
}

/// The liked track ids, most recently liked first.
pub fn load() -> Vec<String> {
    let Some(path) = file() else {
        return Vec::new();
    };
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .unwrap_or_default()
}

/// Persist the ids exactly as ordered — the order IS the data.
pub fn save(ids: &[String]) {
    if let Some(path) = file() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(ids) {
            let _ = std::fs::write(path, json);
        }
    }
}
