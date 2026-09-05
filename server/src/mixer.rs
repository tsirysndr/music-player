use std::sync::{Arc, Mutex};

use music_player_playback::player::PlayerCommand;
use tokio::sync::mpsc::UnboundedSender;

use crate::api::music::v1alpha1::{
    mixer_service_server::MixerService, GetMuteRequest, GetMuteResponse, GetVolumeRequest,
    GetVolumeResponse, SetMuteRequest, SetMuteResponse, SetVolumeRequest, SetVolumeResponse,
};

pub struct Mixer {
    cmd_tx: Arc<Mutex<UnboundedSender<PlayerCommand>>>,
    volume: Arc<Mutex<u16>>,
    mute: Arc<Mutex<bool>>,
}

impl Mixer {
    pub fn new(cmd_tx: Arc<Mutex<UnboundedSender<PlayerCommand>>>) -> Self {
        Self {
            cmd_tx,
            volume: Arc::new(Mutex::new(100)),
            mute: Arc::new(Mutex::new(false)),
        }
    }

    fn send(&self, cmd: PlayerCommand) -> Result<(), tonic::Status> {
        self.cmd_tx
            .lock()
            .unwrap()
            .send(cmd)
            .map_err(|e| tonic::Status::internal(e.to_string()))
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
}
