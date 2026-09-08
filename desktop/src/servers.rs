//! Saved remote music servers (Subsonic/Navidrome + Jellyfin). The desktop
//! The desktop hands these to the daemon, which saves them and connects to
//! one as the current provider; every library screen then reads through it.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedServer {
    pub kind: String, // "subsonic" | "jellyfin"
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
}

fn file() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("music-player").join("desktop_servers.json"))
}

pub fn load() -> Vec<SavedServer> {
    let Some(path) = file() else { return vec![] };
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(list: &[SavedServer]) {
    if let Some(path) = file() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(list) {
            let _ = std::fs::write(path, json);
        }
    }
}
