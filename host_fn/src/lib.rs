use std::fs;

use chromecast::*;
use extism::{convert::Json, host_fn, Manifest, PluginBuilder, Wasm, PTR};
use music_player_settings::{get_application_directory, read_settings, Settings};
use music_player_types::types::Module;
use player::*;
use state::State;
use tracklist::*;
use upnp::*;

pub mod chromecast;
pub mod http;
pub mod library;
pub mod player;
pub mod playlist;
pub mod state;
pub mod tracklist;
pub mod upnp;

host_fn!(pub register_addon(app_data: State; name: String) {
  let state = app_data.get()?;
  let mut state = state.lock().unwrap();
  state.addons.push(name);
  Ok(())
});

host_fn!(pub with_capabilities(app_data: State; capabilities: Json<Vec<String>>) {
  let state = app_data.get()?;
  let mut state = state.lock().unwrap();
  let addon = state.addons.last().unwrap().clone();
  for capability in capabilities.into_inner() {
    state.addon_capabilities.push((addon.clone(), capability.into()));
  }
  Ok(())
});

host_fn!(pub get_settings(app_data: State;) -> Json<Settings> {
  let config = read_settings().unwrap();
  let settings = config.try_deserialize::<Settings>().unwrap();
  Ok(Json(settings))
});

host_fn!(pub get_addons(app_data: State;) -> Json<Vec<String>> {
  let state = app_data.get()?;
  let state = state.lock().unwrap();
  Ok(Json(state.addons.clone()))
});

host_fn!(pub call(app_data: State; opts: Json<Module>) -> String {
  let opts = opts.into_inner();
  let module = opts.url.clone();
  let app_dir = get_application_directory();
  let module = match fs::metadata(module.clone()).is_ok() || module.starts_with("http") {
      true => module,
      false => format!("{}/addons/{}.wasm", app_dir, module),
  };
  let module = match module.starts_with("http") {
      true => Wasm::url(module),
      false => Wasm::file(module),
  };

  let manifest = Manifest::new([module]);
  let mut plugin = PluginBuilder::new(manifest.clone())
    .with_wasi(true)
    .with_function("register_addon", [PTR], [], app_data.clone(), register_addon)
    .with_function("with_capabilities", [PTR], [], app_data.clone(), with_capabilities)
    .with_function("connect_to_chromecast", [PTR], [], app_data.clone(), connect_to_chromecast)
    .with_function("reconnect_to_chromecast", [], [], app_data.clone(), reconnect_to_chromecast)
    .with_function("send_command_to_chromecast", [PTR], [], app_data.clone(), send_command_to_chromecast)
    .with_function("chromecast_queue_load", [PTR], [], app_data.clone(), chromecast_queue_load)
    .with_function("load_track_to_chromecast", [PTR], [], app_data.clone(), load_track_to_chromecast)
    .with_function("get_chromecast_current_playback", [], [PTR], app_data.clone(), get_chromecast_current_playback)
    .with_function("disconnect_from_chromecast", [], [], app_data.clone(), disconnect_from_chromecast)
    .with_function("load", [PTR], [], app_data.clone(), load)
    .with_function("load_tracklist", [PTR], [], app_data.clone(), load_tracklist)
    .with_function("preload", [PTR], [], app_data.clone(), preload)
    .with_function("play", [], [], app_data.clone(), play)
    .with_function("pause", [], [], app_data.clone(), pause)
    .with_function("stop", [], [], app_data.clone(), stop)
    .with_function("next", [], [], app_data.clone(), next)
    .with_function("previous", [], [], app_data.clone(), previous)
    .with_function("seek", [PTR], [], app_data.clone(), seek)
    .with_function("play_track_at", [PTR], [], app_data.clone(), play_track_at)
    .with_function("clear", [], [], app_data.clone(), clear)
    .with_function("get_current_tracklist", [], [PTR], app_data.clone(), get_current_tracklist)
    .with_function("get_current_track", [], [PTR], app_data.clone(), get_current_track)
    .with_function("play_next", [], [], app_data.clone(), play_next)
    .with_function("remove_track", [PTR], [], app_data.clone(), remove_track)
    .with_function("connect_to_upnp_media_renderer", [], [], app_data.clone(), connect_to_upnp_media_renderer)
    .with_function("connect_to_upnp_media_server", [], [], app_data.clone(), connect_to_upnp_media_server)
    .with_function("browse_upnp_media_server", [], [], app_data.clone(), browse_upnp_media_server)
    .with_function("send_command_to_upnp_player", [PTR], [], app_data.clone(), send_command_to_upnp_player)
    .with_function("get_settings", [], [PTR], app_data.clone(), get_settings)
    .with_function("get_addons", [], [PTR], app_data.clone(), get_addons)
    .build()?;

  let func = opts.function.clone();
  let args = opts.args.clone();
  let result = plugin.call::<&str, &str>(func, &args)?;
  Ok(result.to_string())
});
