use anyhow::Error;
use music_player_scanner::refresh_music_library as refresh;
use music_player_settings::{read_settings, Settings};
use music_player_storage::Database;
use music_player_types::types::Song;
use sea_orm::EntityTrait;
use tracing::error;

use music_player_entity::track;

pub async fn auto_scan_music_library(db: Database) {
    match track::Entity::find().all(db.clone().get_connection()).await {
        Ok(result) => {
            if result.is_empty() {
                scan_music_library(false, db).await.unwrap_or_default();
            }
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    }
}

pub async fn scan_music_library(enable_log: bool, db: Database) -> Result<Vec<Song>, Error> {
    refresh(enable_log, db).await
}

/// Periodically rescan the music directory so new files show up without a
/// manual `music-player scan`. Controlled by `library_refresh_interval` in
/// settings.toml (minutes; 0 disables). Runs forever on its own runtime.
pub async fn periodic_scan_music_library() {
    let interval_minutes = read_settings()
        .ok()
        .and_then(|config| config.try_deserialize::<Settings>().ok())
        .map(|settings| settings.library_refresh_interval)
        .unwrap_or(0);
    if interval_minutes == 0 {
        return;
    }
    let interval = std::time::Duration::from_secs(interval_minutes as u64 * 60);
    loop {
        tokio::time::sleep(interval).await;
        let db = Database::new().await;
        if let Err(e) = scan_music_library(false, db.clone()).await {
            error!("Library refresh failed: {}", e);
        }
        // Detached here, unlike the `scan` command: this loop runs inside a
        // daemon that outlives it, and the next refresh is a long way off.
        music_player_scanner::spawn_key_and_bpm_analysis(db);
    }
}
