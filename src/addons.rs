use anyhow::Error;
use extism::{convert::Json, host_fn, Manifest, PluginBuilder, UserData, Wasm, PTR};
use music_player_addons::load_plugin;
use music_player_host_fn::state::State;
use music_player_playback::player::PlayerCommand;
use music_player_settings::{get_application_directory, read_settings, Settings};
use music_player_storage::Database;
use music_player_tracklist::Tracklist;
use music_player_types::types::Module;
use std::fs;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;

pub fn register_addons(user_data: UserData<State>) -> Result<(), Error> {
    let config = read_settings()?;
    let settings = config.try_deserialize::<Settings>()?;
    let app_dir = get_application_directory();

    let addons = settings
        .addons
        .unwrap_or(vec![])
        .iter()
        .map(
            |addon| match fs::metadata(addon).is_ok() || addon.starts_with("http") {
                true => addon.clone(),
                false => format!("{}/addons/{}.wasm", app_dir, addon),
            },
        )
        .collect::<Vec<String>>();

    for addon in addons {
        setup_addon(&addon, &user_data)?;
    }

    Ok(())
}

pub fn setup_addon(module: &str, user_data: &UserData<State>) -> Result<(), Error> {
    let mut plugin = load_plugin(module, user_data)?;
    let result = plugin.call::<&str, &str>("setup", "")?;
    Ok(())
}
