use std::sync::{Arc, Mutex};

use music_player_playback::player::PlayerCommand;
use music_player_settings::{read_settings, save_audio_settings, AudioSettings, Settings};
use tokio::sync::mpsc::UnboundedSender;

use crate::api::music::v1alpha1::{
    mixer_service_server::MixerService, EqBand, GetAudioSettingsRequest, GetAudioSettingsResponse,
    GetMuteRequest, GetMuteResponse, GetVolumeRequest, GetVolumeResponse, SetAudioSettingRequest,
    SetAudioSettingResponse, SetEqBandGainRequest, SetEqBandGainResponse, SetMuteRequest,
    SetMuteResponse, SetVolumeRequest, SetVolumeResponse,
};

/// Standard EQ band center frequencies (Hz), one per band — mirrors
/// rockbox_playback::EQ_BAND_FREQUENCIES.
const EQ_BAND_FREQUENCIES: [i32; 10] = [32, 64, 125, 250, 500, 1_000, 2_000, 4_000, 8_000, 16_000];
const TONE_MIN_DB: i32 = -24;
const TONE_MAX_DB: i32 = 24;

pub struct Mixer {
    cmd_tx: Arc<Mutex<UnboundedSender<PlayerCommand>>>,
    volume: Arc<Mutex<u16>>,
    mute: Arc<Mutex<bool>>,
    audio: Arc<Mutex<AudioSettings>>,
}

impl Mixer {
    pub fn new(cmd_tx: Arc<Mutex<UnboundedSender<PlayerCommand>>>) -> Self {
        let audio = read_settings()
            .ok()
            .and_then(|config| config.try_deserialize::<Settings>().ok())
            .map(|settings| settings.audio)
            .unwrap_or_default();
        Self {
            cmd_tx,
            volume: Arc::new(Mutex::new(100)),
            mute: Arc::new(Mutex::new(false)),
            audio: Arc::new(Mutex::new(audio)),
        }
    }

    fn send(&self, cmd: PlayerCommand) -> Result<(), tonic::Status> {
        self.cmd_tx
            .lock()
            .unwrap()
            .send(cmd)
            .map_err(|e| tonic::Status::internal(e.to_string()))
    }

    fn persist(&self, audio: &AudioSettings) {
        if let Err(e) = save_audio_settings(audio) {
            tracing::warn!("failed to persist audio settings: {e}");
        }
    }

    /// Translate the current crossfade snapshot into a player command.
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
}

#[tonic::async_trait]
impl MixerService for Mixer {
    async fn get_mute(
        &self,
        _request: tonic::Request<GetMuteRequest>,
    ) -> Result<tonic::Response<GetMuteResponse>, tonic::Status> {
        let response = GetMuteResponse {
            mute: *self.mute.lock().unwrap(),
        };
        Ok(tonic::Response::new(response))
    }
    async fn set_mute(
        &self,
        request: tonic::Request<SetMuteRequest>,
    ) -> Result<tonic::Response<SetMuteResponse>, tonic::Status> {
        let mute = request.into_inner().mute;
        let volume = *self.volume.lock().unwrap();
        self.send(PlayerCommand::SetVolume(if mute { 0 } else { volume }))?;
        *self.mute.lock().unwrap() = mute;
        Ok(tonic::Response::new(SetMuteResponse {}))
    }
    async fn get_volume(
        &self,
        _request: tonic::Request<GetVolumeRequest>,
    ) -> Result<tonic::Response<GetVolumeResponse>, tonic::Status> {
        let response = GetVolumeResponse {
            volume: *self.volume.lock().unwrap() as u32,
        };
        Ok(tonic::Response::new(response))
    }
    async fn set_volume(
        &self,
        request: tonic::Request<SetVolumeRequest>,
    ) -> Result<tonic::Response<SetVolumeResponse>, tonic::Status> {
        let volume = request.into_inner().volume.min(100) as u16;
        self.send(PlayerCommand::SetVolume(volume))?;
        *self.volume.lock().unwrap() = volume;
        *self.mute.lock().unwrap() = false;
        Ok(tonic::Response::new(SetVolumeResponse {}))
    }

    async fn get_audio_settings(
        &self,
        _request: tonic::Request<GetAudioSettingsRequest>,
    ) -> Result<tonic::Response<GetAudioSettingsResponse>, tonic::Status> {
        let audio = self.audio.lock().unwrap().clone();
        let response = GetAudioSettingsResponse {
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
        };
        Ok(tonic::Response::new(response))
    }

    async fn set_audio_setting(
        &self,
        request: tonic::Request<SetAudioSettingRequest>,
    ) -> Result<tonic::Response<SetAudioSettingResponse>, tonic::Status> {
        let request = request.into_inner();
        let value = request.value;
        let mut audio = self.audio.lock().unwrap().clone();
        let cmd = match request.name.as_str() {
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
                Self::crossfade_command(&audio)
            }
            "fade_in_delay" => {
                audio.fade_in_delay = value.clamp(0, 7) as u64;
                Self::crossfade_command(&audio)
            }
            "fade_in_duration" => {
                audio.fade_in_duration = value.clamp(0, 15) as u64;
                Self::crossfade_command(&audio)
            }
            "fade_out_delay" => {
                audio.fade_out_delay = value.clamp(0, 7) as u64;
                Self::crossfade_command(&audio)
            }
            "fade_out_duration" => {
                audio.fade_out_duration = value.clamp(0, 15) as u64;
                Self::crossfade_command(&audio)
            }
            "fade_out_mixmode" => {
                audio.fade_out_mixmode = if value == 2 { 2 } else { 0 };
                Self::crossfade_command(&audio)
            }
            "dithering" => {
                audio.dithering = value != 0;
                PlayerCommand::SetDither(audio.dithering)
            }
            other => {
                return Err(tonic::Status::invalid_argument(format!(
                    "unknown audio setting: {other}"
                )))
            }
        };
        self.send(cmd)?;
        *self.audio.lock().unwrap() = audio.clone();
        self.persist(&audio);
        Ok(tonic::Response::new(SetAudioSettingResponse {}))
    }

    async fn set_eq_band_gain(
        &self,
        request: tonic::Request<SetEqBandGainRequest>,
    ) -> Result<tonic::Response<SetEqBandGainResponse>, tonic::Status> {
        let request = request.into_inner();
        let band = request.band as usize;
        if band >= EQ_BAND_FREQUENCIES.len() {
            return Err(tonic::Status::invalid_argument("EQ band out of range"));
        }
        let gain_db = (request.gain.clamp(-240, 240) as f32) / 10.0;
        let mut audio = self.audio.lock().unwrap().clone();
        if audio.eq_band_gains.len() < EQ_BAND_FREQUENCIES.len() {
            audio.eq_band_gains.resize(EQ_BAND_FREQUENCIES.len(), 0.0);
        }
        audio.eq_band_gains[band] = gain_db;
        self.send(PlayerCommand::SetEqBandGain { band, gain_db })?;
        *self.audio.lock().unwrap() = audio.clone();
        self.persist(&audio);
        Ok(tonic::Response::new(SetEqBandGainResponse {}))
    }
}
