use extism_pdk::*;
use music_player_pdk::{addon, player::*, types::Track, Capability};

const ADDON_NAME: &str = "local";

#[plugin_fn]
pub fn setup() -> FnResult<()> {
    addon()
        .register(ADDON_NAME)?
        .with_capabilities(vec![Capability::Player, Capability::Browse])?;
    Ok(())
}

#[plugin_fn]
pub fn play() -> FnResult<()> {
    player().play()?;
    Ok(())
}

#[plugin_fn]
pub fn pause() -> FnResult<()> {
    player().pause()?;
    Ok(())
}

#[plugin_fn]
pub fn stop() -> FnResult<()> {
    player().stop()?;
    Ok(())
}

#[plugin_fn]
pub fn next() -> FnResult<()> {
    player().next()?;
    Ok(())
}

#[plugin_fn]
pub fn previous() -> FnResult<()> {
    player().previous()?;
    Ok(())
}

#[plugin_fn]
pub fn seek(position: u32) -> FnResult<()> {
    player().seek(position)?;
    Ok(())
}

#[plugin_fn]
pub fn load_tracks(tracks: Json<Vec<Track>>) -> FnResult<()> {
    player().load_tracks(tracks)?;
    Ok(())
}

#[plugin_fn]
pub fn play_next() -> FnResult<()> {
    player().play_next()?;
    Ok(())
}

#[plugin_fn]
pub fn load(track: Json<Track>) -> FnResult<()> {
    player().load(track)?;
    Ok(())
}

#[plugin_fn]
pub fn get_current_playback() -> FnResult<()> {
    player().get_current_playback()?;
    Ok(())
}

#[plugin_fn]
pub fn get_current_tracklist() -> FnResult<()> {
    player().get_current_tracklist()?;
    Ok(())
}

#[plugin_fn]
pub fn play_track_at(index: u32) -> FnResult<()> {
    player().play_track_at(index)?;
    Ok(())
}

#[plugin_fn]
pub fn remove_track_at(index: u32) -> FnResult<()> {
    player().remove_track_at(index)?;
    Ok(())
}

#[plugin_fn]
pub fn connect_to() -> FnResult<()> {
    Ok(())
}
