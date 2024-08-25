use std::sync::{Arc, Mutex};

use music_player_pdk::Capability;
use music_player_playback::player::PlayerCommand;
use music_player_storage::Database;
use music_player_tracklist::Tracklist;
use tokio::sync::mpsc::UnboundedSender;

pub struct State {
    pub player_cmd_tx: Arc<Mutex<UnboundedSender<PlayerCommand>>>,
    pub tracklist: Arc<Mutex<Tracklist>>,
    pub db: Database,
    pub addons: Vec<String>,
    pub addon_capabilities: Vec<(String, Capability)>,
}
