use extism_pdk::*;
use music_player_pdk::{addon, Capability};

#[plugin_fn]
pub fn setup() -> FnResult<()> {
    addon()
        .register("mopidy")?
        .with_capabilities(vec![Capability::Player])?;
    Ok(())
}
