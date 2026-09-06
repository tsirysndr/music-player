//! Rocksky remote-player protocol for the daemon.
//!
//! Registers as a playback device on Rocksky's remote-control WebSocket
//! (mirrors rocksky's `playerd`), so this daemon shows up in the web/desktop
//! miniplayer device picker. Controllers get transport control (play, pause,
//! next, previous, seek, shuffle, repeat, volume) plus the audio-settings
//! surface our engine implements (EQ, tone, ReplayGain, crossfade, dither).
//!
//! Queue control (enqueue/jump/remove/move) is NOT enabled yet: the daemon
//! plays the local library and is not connected to the Rocksky library, so
//! queue commands are acknowledged-and-ignored and no queue is pushed.
//!
//! Requires the access token written by `rocksky login` to
//! `~/.rocksky/token.json`; without it the remote player stays off. It can
//! also be disabled with `remote_player = false` in settings.toml.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use music_player_playback::player::PlayerCommand;
use music_player_settings::{read_settings, Settings};
use music_player_tracklist::Tracklist;
use rocksky_sdk::{
    RemoteCommand, RemoteNowPlaying, RemotePlayer, RemotePlayerConfig, RemoteRepeat, RemoteStatus,
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{info, warn};

fn token_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/"))
        .join(".rocksky")
        .join("token.json")
}

/// The access token written by `rocksky login`, if any.
fn read_token() -> Option<String> {
    let raw = std::fs::read_to_string(token_path()).ok()?;
    let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
    value
        .get("token")
        .and_then(|t| t.as_str())
        .filter(|t| !t.trim().is_empty())
        .map(String::from)
}

/// Start the remote player if it is enabled and a Rocksky token is available.
pub fn spawn(tracklist: Arc<Mutex<Tracklist>>, cmd_tx: Arc<Mutex<UnboundedSender<PlayerCommand>>>) {
    let enabled = read_settings()
        .ok()
        .and_then(|config| config.get_bool("remote_player").ok())
        .unwrap_or(true);
    if !enabled {
        return;
    }
    let Some(token) = read_token() else {
        info!(
            "Rocksky remote player disabled: no token at {} (run `rocksky login`)",
            token_path().display()
        );
        return;
    };
    let name = read_settings()
        .ok()
        .and_then(|config| config.try_deserialize::<Settings>().ok())
        .map(|settings| settings.device_name)
        .unwrap_or_else(|| "Music Player".to_string());
    let ws_url = std::env::var("ROCKSKY_WS")
        .unwrap_or_else(|_| rocksky_sdk::remote_player::DEFAULT_REMOTE_WS.to_string());

    tokio::spawn(async move {
        info!("registering \"{name}\" on {ws_url}");
        let remote = Arc::new(RemotePlayer::connect(
            RemotePlayerConfig::new(token, name).url(ws_url),
        ));
        tokio::spawn(status_loop(remote.clone(), tracklist));
        command_loop(remote, cmd_tx).await;
    });
}

async fn command_loop(
    remote: Arc<RemotePlayer>,
    cmd_tx: Arc<Mutex<UnboundedSender<PlayerCommand>>>,
) {
    let send = |cmd: PlayerCommand| {
        if let Err(e) = cmd_tx.lock().unwrap().send(cmd) {
            warn!("remote command dropped: {e}");
        }
    };
    while let Some(cmd) = remote.next_command().await {
        match cmd {
            RemoteCommand::Play => send(PlayerCommand::Play),
            RemoteCommand::Pause => send(PlayerCommand::Pause),
            RemoteCommand::Next => send(PlayerCommand::Next),
            RemoteCommand::Previous => send(PlayerCommand::Previous),
            RemoteCommand::Seek { position_ms } => {
                send(PlayerCommand::Seek(position_ms.min(u32::MAX as u64) as u32))
            }
            RemoteCommand::SetShuffle { enabled } => send(PlayerCommand::SetShuffle(enabled)),
            RemoteCommand::SetRepeat { mode } => send(PlayerCommand::SetRepeat(match mode {
                RemoteRepeat::All => 1,
                RemoteRepeat::One => 2,
                _ => 0,
            })),
            RemoteCommand::SetVolume { volume } => {
                send(PlayerCommand::SetVolume(
                    (volume.clamp(0.0, 1.0) * 100.0).round() as u16,
                ));
            }
            RemoteCommand::SetAudioSettings(audio) => apply_audio(&send, &audio),
            // Queue control stays off until the daemon is connected to the
            // Rocksky library — the local library has no upload/track ids.
            RemoteCommand::Enqueue { .. }
            | RemoteCommand::QueueJump { .. }
            | RemoteCommand::QueueRemove { .. }
            | RemoteCommand::QueueMove { .. } => {
                warn!("remote queue control not enabled yet, ignoring");
            }
        }
    }
    info!("remote command loop ended");
}

/// Apply the sections of a partial audio-settings document our engine
/// implements (EQ, tone, ReplayGain, crossfade); the rest (crossfeed,
/// compressor, surround, PBE) is not read.
fn apply_audio(
    send: &impl Fn(PlayerCommand),
    audio: &rocksky_sdk::remote_audio::RemoteAudioSettings,
) {
    if let Some(eq) = &audio.equalizer {
        if let Some(enabled) = eq.enabled {
            send(PlayerCommand::SetEqEnabled(enabled));
        }
        if let Some(precut) = eq.precut {
            // Wire precut is ≤ 0 tenths-of-dB of attenuation; the engine
            // takes positive dB of headroom.
            send(PlayerCommand::SetEqPrecut((-precut).max(0) as f32 / 10.0));
        }
        if let Some(bands) = &eq.bands {
            for (i, band) in bands.iter().enumerate().take(10) {
                send(PlayerCommand::SetEqBandGain {
                    band: i,
                    gain_db: band.gain as f32 / 10.0,
                });
            }
        }
    }
    if let Some(tone) = &audio.tone {
        if let Some(bass) = tone.bass {
            send(PlayerCommand::SetBass(bass));
        }
        if let Some(treble) = tone.treble {
            send(PlayerCommand::SetTreble(treble));
        }
        if let Some(balance) = tone.balance {
            send(PlayerCommand::SetBalance(balance));
        }
    }
    if let Some(rg) = &audio.replay_gain {
        let mode = match rg.mode.as_deref() {
            Some("track") => 0,
            Some("album") => 1,
            Some("trackIfShuffling") => 2,
            _ => 3,
        };
        send(PlayerCommand::SetReplaygain {
            mode,
            preamp_db: rg.preamp.unwrap_or(0) as f32 / 10.0,
            prevent_clipping: rg.prevent_clipping.unwrap_or(false),
        });
    }
    if let Some(cf) = &audio.crossfade {
        let mode = match cf.mode.as_deref() {
            Some("enabled") => 5,
            Some("shuffle") => 3,
            Some("trackChange") => 2,
            Some("albumChange") => 1,
            _ => 0,
        };
        send(PlayerCommand::SetCrossfade {
            mode,
            fade_in_delay: cf.fade_in_delay.unwrap_or(0) / 1000,
            fade_in_duration: cf.fade_in_duration.unwrap_or(2000) / 1000,
            fade_out_delay: cf.fade_out_delay.unwrap_or(0) / 1000,
            fade_out_duration: cf.fade_out_duration.unwrap_or(2000) / 1000,
            mix_mode: if cf.fade_out_mix_mode.as_deref() == Some("mix") {
                2
            } else {
                0
            },
        });
    }
}

/// Push now-playing + transport state to controllers every couple of
/// seconds so this daemon stays live in the miniplayer device picker.
/// No queue push — see the module docs.
async fn status_loop(remote: Arc<RemotePlayer>, tracklist: Arc<Mutex<Tracklist>>) {
    loop {
        let (track, position_ms, is_playing, stopped) = {
            let tracklist = tracklist.lock().unwrap();
            let (track, _) = tracklist.current_track();
            let state = tracklist.playback_state();
            let stopped = track.is_none();
            (track, state.position_ms, state.is_playing, stopped)
        };

        let mut np = RemoteNowPlaying {
            elapsed_ms: position_ms as u64,
            is_playing,
            shuffle: Some(false),
            repeat: None,
            volume: None,
            ..Default::default()
        };
        if let Some(track) = &track {
            np.title = track.title.clone();
            np.artist = track.artist.clone();
            np.album = track.album.title.clone();
            np.album_artist = track.album.artist.clone();
            np.duration_ms = (track.duration.unwrap_or(0.0) * 1000.0) as u64;
            np.sample_rate = track.sample_rate;
        }
        remote.set_now_playing(np);
        remote.set_status(if stopped {
            RemoteStatus::Stopped
        } else if is_playing {
            RemoteStatus::Playing
        } else {
            RemoteStatus::Paused
        });

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
