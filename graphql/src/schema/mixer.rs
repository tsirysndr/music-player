use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc, Mutex,
};

use async_graphql::*;
use music_player_playback::player::PlayerCommand;
use music_player_settings::{read_settings, save_audio_settings, AudioSettings, Settings};
use tokio::sync::mpsc::UnboundedSender;

// The playback engine has no volume read-back over the command channel, so the
// last value set through this API is remembered here.
static VOLUME: AtomicU32 = AtomicU32::new(100);
static MUTED: AtomicBool = AtomicBool::new(false);

/// EQ band centre frequencies (Hz) — mirrors `rockbox_playback::EQ_BAND_FREQUENCIES`
/// and the same list in `server/src/mixer.rs`.
const EQ_BAND_FREQUENCIES: [i32; 10] = [32, 64, 125, 250, 500, 1_000, 2_000, 4_000, 8_000, 16_000];
const TONE_MIN_DB: i32 = -24;
const TONE_MAX_DB: i32 = 24;

/// One EQ band, in the firmware's units.
#[derive(SimpleObject, Clone, Debug)]
pub struct EqBand {
    /// Centre frequency in Hz.
    pub cutoff: i32,
    /// Q, times ten.
    pub q: i32,
    /// Gain in dB, times ten — the engine's own unit, so a UI never has to
    /// round-trip a float through a slider.
    pub gain: i32,
}

/// The DSP chain's state, in the firmware's units.
///
/// Mirrors the `GetAudioSettingsResponse` of the gRPC `MixerService`, field for
/// field, so the web client and the Slint desktop show the same numbers. The
/// ranges come back with the values because they are what a slider needs and
/// they are not obvious from the field alone.
#[derive(SimpleObject, Clone, Debug)]
pub struct AudioSettingsState {
    pub eq_enabled: bool,
    /// EQ pre-gain in dB × 10, 0..=240.
    pub eq_precut: i32,
    pub eq_bands: Vec<EqBand>,
    /// Bass shelf gain in dB.
    pub bass: i32,
    pub bass_min: i32,
    pub bass_max: i32,
    /// Treble shelf gain in dB.
    pub treble: i32,
    pub treble_min: i32,
    pub treble_max: i32,
    /// Stereo balance, -100 (full left)..=100 (full right).
    pub balance: i32,
    /// 0 track, 1 album, 2 track (shuffle), 3 off.
    pub replaygain_type: i32,
    /// ReplayGain pre-amp in dB × 10, -120..=120.
    pub replaygain_preamp: i32,
    pub replaygain_noclip: bool,
    /// 0 off … 5 always.
    pub crossfade: i32,
    pub fade_in_delay: i32,
    pub fade_in_duration: i32,
    pub fade_out_delay: i32,
    pub fade_out_duration: i32,
    /// 0 crossfade, 2 mix.
    pub fade_out_mixmode: i32,
    pub dithering: bool,
}

/// Read the persisted settings. A missing or unreadable file is the defaults,
/// not an error: the DSP has to have *some* state to start from.
fn audio_settings() -> AudioSettings {
    read_settings()
        .ok()
        .and_then(|config| config.try_deserialize::<Settings>().ok())
        .map(|settings| settings.audio)
        .unwrap_or_default()
}

impl From<AudioSettings> for AudioSettingsState {
    fn from(audio: AudioSettings) -> Self {
        Self {
            eq_enabled: audio.eq_enabled,
            eq_precut: (audio.eq_precut * 10.0).round() as i32,
            eq_bands: EQ_BAND_FREQUENCIES
                .iter()
                .enumerate()
                .map(|(i, &cutoff)| EqBand {
                    cutoff,
                    q: 10,
                    gain: (audio.eq_band_gains.get(i).copied().unwrap_or(0.0) * 10.0).round()
                        as i32,
                })
                .collect(),
            bass: audio.bass,
            bass_min: TONE_MIN_DB,
            bass_max: TONE_MAX_DB,
            treble: audio.treble,
            treble_min: TONE_MIN_DB,
            treble_max: TONE_MAX_DB,
            balance: audio.balance,
            replaygain_type: audio.replaygain_mode,
            replaygain_preamp: (audio.replaygain_preamp * 10.0).round() as i32,
            replaygain_noclip: audio.replaygain_noclip,
            crossfade: audio.crossfade,
            fade_in_delay: audio.fade_in_delay as i32,
            fade_in_duration: audio.fade_in_duration as i32,
            fade_out_delay: audio.fade_out_delay as i32,
            fade_out_duration: audio.fade_out_duration as i32,
            fade_out_mixmode: audio.fade_out_mixmode,
            dithering: audio.dithering,
        }
    }
}

/// The crossfade snapshot as one player command.
fn crossfade_command(audio: &AudioSettings) -> PlayerCommand {
    PlayerCommand::SetCrossfade {
        mode: audio.crossfade,
        fade_in_delay: audio.fade_in_delay,
        fade_in_duration: audio.fade_in_duration,
        fade_out_delay: audio.fade_out_delay,
        fade_out_duration: audio.fade_out_duration,
        mix_mode: audio.fade_out_mixmode,
    }
}

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

    /// The DSP chain's state: EQ, tone, ReplayGain, crossfade, dithering.
    async fn audio_settings(&self, _ctx: &Context<'_>) -> Result<AudioSettingsState, Error> {
        Ok(audio_settings().into())
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

    /// Change one audio setting, by the same names the gRPC service takes —
    /// `eq_enabled`, `eq_precut`, `bass`, `treble`, `balance`, `rg_type`,
    /// `rg_preamp`, `rg_noclip`, `crossfade`, `fade_in_delay`,
    /// `fade_in_duration`, `fade_out_delay`, `fade_out_duration`,
    /// `fade_out_mixmode`, `dithering`.
    ///
    /// Everything is an integer in the firmware's own units (dB × 10 where a
    /// fraction is meaningful, 0/1 for a flag), which is what keeps a slider
    /// from having to round-trip a float.
    ///
    /// Returns the whole state, so a client renders what was stored rather
    /// than what it asked for — the two differ wherever a value was clamped.
    async fn set_audio_setting(
        &self,
        ctx: &Context<'_>,
        name: String,
        value: i32,
    ) -> Result<AudioSettingsState, Error> {
        let mut audio = audio_settings();
        let cmd = match name.as_str() {
            "eq_enabled" => {
                audio.eq_enabled = value != 0;
                PlayerCommand::SetEqEnabled(audio.eq_enabled)
            }
            "eq_precut" => {
                audio.eq_precut = (value.clamp(0, 240) as f32) / 10.0;
                PlayerCommand::SetEqPrecut(audio.eq_precut)
            }
            "bass" => {
                audio.bass = value.clamp(TONE_MIN_DB, TONE_MAX_DB);
                PlayerCommand::SetBass(audio.bass)
            }
            "treble" => {
                audio.treble = value.clamp(TONE_MIN_DB, TONE_MAX_DB);
                PlayerCommand::SetTreble(audio.treble)
            }
            "balance" => {
                audio.balance = value.clamp(-100, 100);
                PlayerCommand::SetBalance(audio.balance)
            }
            "rg_type" => {
                audio.replaygain_mode = value.clamp(0, 3);
                PlayerCommand::SetReplaygain {
                    mode: audio.replaygain_mode,
                    preamp_db: audio.replaygain_preamp,
                    prevent_clipping: audio.replaygain_noclip,
                }
            }
            "rg_preamp" => {
                audio.replaygain_preamp = (value.clamp(-120, 120) as f32) / 10.0;
                PlayerCommand::SetReplaygain {
                    mode: audio.replaygain_mode,
                    preamp_db: audio.replaygain_preamp,
                    prevent_clipping: audio.replaygain_noclip,
                }
            }
            "rg_noclip" => {
                audio.replaygain_noclip = value != 0;
                PlayerCommand::SetReplaygain {
                    mode: audio.replaygain_mode,
                    preamp_db: audio.replaygain_preamp,
                    prevent_clipping: audio.replaygain_noclip,
                }
            }
            "crossfade" => {
                audio.crossfade = value.clamp(0, 5);
                crossfade_command(&audio)
            }
            "fade_in_delay" => {
                audio.fade_in_delay = value.clamp(0, 7) as u64;
                crossfade_command(&audio)
            }
            "fade_in_duration" => {
                audio.fade_in_duration = value.clamp(0, 15) as u64;
                crossfade_command(&audio)
            }
            "fade_out_delay" => {
                audio.fade_out_delay = value.clamp(0, 7) as u64;
                crossfade_command(&audio)
            }
            "fade_out_duration" => {
                audio.fade_out_duration = value.clamp(0, 15) as u64;
                crossfade_command(&audio)
            }
            "fade_out_mixmode" => {
                audio.fade_out_mixmode = if value == 2 { 2 } else { 0 };
                crossfade_command(&audio)
            }
            "dithering" => {
                audio.dithering = value != 0;
                PlayerCommand::SetDither(audio.dithering)
            }
            other => return Err(Error::new(format!("unknown audio setting: {other}"))),
        };

        let cmd_tx = ctx
            .data::<Arc<Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        cmd_tx.lock().unwrap().send(cmd)?;

        // Persisted after the engine accepted it, so a rejected command does
        // not leave settings.toml describing a state the DSP is not in.
        if let Err(e) = save_audio_settings(&audio) {
            tracing::warn!("failed to persist audio settings: {e}");
        }
        Ok(audio.into())
    }

    /// Set one EQ band's gain, in dB × 10 (-240..=240).
    async fn set_eq_band_gain(
        &self,
        ctx: &Context<'_>,
        band: i32,
        gain: i32,
    ) -> Result<AudioSettingsState, Error> {
        let band = band as usize;
        if band >= EQ_BAND_FREQUENCIES.len() {
            return Err(Error::new("EQ band out of range"));
        }
        let gain_db = (gain.clamp(-240, 240) as f32) / 10.0;

        let mut audio = audio_settings();
        // A settings file written before a band was added would be short.
        if audio.eq_band_gains.len() < EQ_BAND_FREQUENCIES.len() {
            audio.eq_band_gains.resize(EQ_BAND_FREQUENCIES.len(), 0.0);
        }
        audio.eq_band_gains[band] = gain_db;

        let cmd_tx = ctx
            .data::<Arc<Mutex<UnboundedSender<PlayerCommand>>>>()
            .unwrap();
        cmd_tx
            .lock()
            .unwrap()
            .send(PlayerCommand::SetEqBandGain { band, gain_db })?;

        if let Err(e) = save_audio_settings(&audio) {
            tracing::warn!("failed to persist audio settings: {e}");
        }
        Ok(audio.into())
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
