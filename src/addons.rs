use std::sync::{Arc, Mutex};

use anyhow::Error;
use extism::{convert::Json, host_fn, Manifest, PluginBuilder, UserData, Wasm, PTR};
use music_player_host_fn::{
    chromecast::*, player::*, register_addon, state::State, upnp::*, with_capabilities,
};
use music_player_playback::player::PlayerCommand;
use music_player_settings::{get_application_directory, read_settings, Settings};
use music_player_storage::Database;
use music_player_tracklist::Tracklist;
use music_player_types::types::Module;
use tokio::sync::mpsc::UnboundedSender;

pub fn register_addons(
    player_cmd_tx: Arc<Mutex<UnboundedSender<PlayerCommand>>>,
    tracklist: Arc<Mutex<Tracklist>>,
    db: Database,
) -> Result<(), Error> {
    let user_data = UserData::new(State {
        player_cmd_tx,
        tracklist,
        db,
    });

    let config = read_settings()?;
    let settings = config.try_deserialize::<Settings>()?;
    let app_dir = get_application_directory();

    let addons = settings
        .addons
        .unwrap_or(vec![])
        .iter()
        .map(|addon| format!("{}/addons/{}.wasm", app_dir, addon))
        .collect::<Vec<String>>();

    for addon in addons {
        setup_addon(&addon, &user_data)?;
    }

    Ok(())
}

pub fn setup_addon(module: &str, user_data: &UserData<State>) -> Result<(), Error> {
    let module = match module.starts_with("http") {
        true => Wasm::url(module),
        false => Wasm::file(module),
    };

    let manifest = Manifest::new([module]);
    let mut plugin = PluginBuilder::new(manifest.clone())
        .with_wasi(true)
        .with_function(
            "register_addon",
            [PTR],
            [],
            user_data.clone(),
            register_addon,
        )
        .with_function(
            "with_capabilities",
            [PTR],
            [],
            user_data.clone(),
            with_capabilities,
        )
        .with_function(
            "connect_to_chromecast",
            [PTR],
            [],
            user_data.clone(),
            connect_to_chromecast,
        )
        .with_function(
            "reconnect_to_chromecast",
            [],
            [],
            user_data.clone(),
            reconnect_to_chromecast,
        )
        .with_function(
            "send_command_to_chromecast",
            [PTR],
            [],
            user_data.clone(),
            send_command_to_chromecast,
        )
        .with_function(
            "chromecast_queue_load",
            [PTR],
            [],
            user_data.clone(),
            chromecast_queue_load,
        )
        .with_function(
            "load_track_to_chromecast",
            [PTR],
            [],
            user_data.clone(),
            load_track_to_chromecast,
        )
        .with_function(
            "get_chromecast_current_playback",
            [],
            [PTR],
            user_data.clone(),
            get_chromecast_current_playback,
        )
        .with_function(
            "disconnect_from_chromecast",
            [],
            [],
            user_data.clone(),
            disconnect_from_chromecast,
        )
        .with_function("load", [PTR], [], user_data.clone(), load)
        .with_function(
            "load_tracklist",
            [PTR],
            [],
            user_data.clone(),
            load_tracklist,
        )
        .with_function("preload", [PTR], [], user_data.clone(), preload)
        .with_function("play", [], [], user_data.clone(), play)
        .with_function("pause", [], [], user_data.clone(), pause)
        .with_function("stop", [], [], user_data.clone(), stop)
        .with_function("seek", [PTR], [], user_data.clone(), seek)
        .with_function("play_track_at", [PTR], [], user_data.clone(), play_track_at)
        .with_function("clear", [], [], user_data.clone(), clear)
        .with_function("get_tracks", [], [PTR], user_data.clone(), get_tracks)
        .with_function(
            "get_current_track",
            [],
            [PTR],
            user_data.clone(),
            get_current_track,
        )
        .with_function("play_next", [], [], user_data.clone(), play_next)
        .with_function("remove_track", [PTR], [], user_data.clone(), remove_track)
        .with_function(
            "connect_to_upnp_media_renderer",
            [],
            [],
            user_data.clone(),
            connect_to_upnp_media_renderer,
        )
        .with_function(
            "connect_to_upnp_media_server",
            [],
            [],
            user_data.clone(),
            connect_to_upnp_media_server,
        )
        .with_function(
            "browse_upnp_media_server",
            [],
            [],
            user_data.clone(),
            browse_upnp_media_server,
        )
        .with_function(
            "send_command_to_upnp_player",
            [PTR],
            [],
            user_data.clone(),
            send_command_to_upnp_player,
        )
        .with_function("get_settings", [], [PTR], user_data.clone(), get_settings)
        .build()?;

    let result = plugin.call::<&str, &str>("setup", "")?;

    Ok(())
}
