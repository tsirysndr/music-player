use extism_pdk::*;
use music_player_pdk::{addon, Capability};

const ADDON_NAME: &str = "mopidy";

#[plugin_fn]
pub fn setup() -> FnResult<()> {
    addon()
        .register(ADDON_NAME)?
        .with_capabilities(vec![Capability::Player])?;
    Ok(())
}
