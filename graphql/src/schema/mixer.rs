use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc, Mutex,
};

use async_graphql::*;
use music_player_playback::player::PlayerCommand;
use tokio::sync::mpsc::UnboundedSender;

// The playback engine has no volume read-back over the command channel, so the
// last value set through this API is remembered here.
static VOLUME: AtomicU32 = AtomicU32::new(100);
static MUTED: AtomicBool = AtomicBool::new(false);

#[derive(Default)]
pub struct MixerQuery;

#[Object]
impl MixerQuery {
    async fn get_volume(&self, _ctx: &Context<'_>) -> Result<i32, Error> {
        Ok(VOLUME.load(Ordering::Relaxed) as i32)
    }

    async fn get_mute(&self, _ctx: &Context<'_>) -> Result<bool, Error> {
        Ok(MUTED.load(Ordering::Relaxed))
    }
}

#[derive(Default)]
pub struct MixerMutation;

#[Object]
impl MixerMutation {
    async fn set_volume(&self, ctx: &Context<'_>, volume: i32) -> Result<bool, Error> {
        let volume = volume.clamp(0, 100) as u16;
        let cmd_tx = ctx
            .data::<Arc<Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        cmd_tx
            .lock()
            .unwrap()
            .send(PlayerCommand::SetVolume(volume))?;
        VOLUME.store(volume as u32, Ordering::Relaxed);
        MUTED.store(false, Ordering::Relaxed);
        Ok(true)
    }

    async fn set_mute(&self, ctx: &Context<'_>, mute: bool) -> Result<bool, Error> {
        let cmd_tx = ctx
            .data::<Arc<Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        let volume = if mute {
            0
        } else {
            VOLUME.load(Ordering::Relaxed) as u16
        };
        cmd_tx
            .lock()
            .unwrap()
            .send(PlayerCommand::SetVolume(volume))?;
        MUTED.store(mute, Ordering::Relaxed);
        Ok(mute)
    }
}
