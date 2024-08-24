use extism_pdk::*;
use music_player_pdk::{addon, Capability};

const ADDON_NAME: &str = "squeezebox";

#[plugin_fn]
pub fn setup() -> FnResult<()> {
    addon()
        .register(ADDON_NAME)?
        .with_capabilities(vec![Capability::Player])?;
    Ok(())
}

#[plugin_fn]
pub fn play() -> FnResult<()> {
    Ok(())
}

#[plugin_fn]
pub fn pause() -> FnResult<()> {
    Ok(())
}

#[plugin_fn]
pub fn stop() -> FnResult<()> {
    Ok(())
}

#[plugin_fn]
pub fn next() -> FnResult<()> {
    Ok(())
}

#[plugin_fn]
pub fn previous() -> FnResult<()> {
    Ok(())
}

#[plugin_fn]
pub fn seek() -> FnResult<()> {
    Ok(())
}

#[plugin_fn]
pub fn load_tracks() -> FnResult<()> {
    Ok(())
}

#[plugin_fn]
pub fn play_next() -> FnResult<()> {
    Ok(())
}

#[plugin_fn]
pub fn load() -> FnResult<()> {
    Ok(())
}

#[plugin_fn]
pub fn get_current_playback() -> FnResult<()> {
    Ok(())
}

#[plugin_fn]
pub fn get_current_tracklist() -> FnResult<()> {
    Ok(())
}

#[plugin_fn]
pub fn play_track_at() -> FnResult<()> {
    Ok(())
}

#[plugin_fn]
pub fn remove_track_at() -> FnResult<()> {
    Ok(())
}

#[plugin_fn]
pub fn connect_to() -> FnResult<()> {
    Ok(())
}
