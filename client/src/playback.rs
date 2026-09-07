use anyhow::Error;
use music_player_server::api::{
    metadata::v1alpha1::Track,
    music::v1alpha1::{
        mixer_service_client::MixerServiceClient, playback_service_client::PlaybackServiceClient,
        GetCurrentlyPlayingSongRequest, GetMuteRequest, GetVolumeRequest, NextRequest,
        PauseRequest, PlayRequest, PreviousRequest, SeekRequest, SetMuteRequest, SetVolumeRequest,
        StopRequest,
    },
};
use tonic::transport::Channel;
pub struct PlaybackClient {
    client: PlaybackServiceClient<Channel>,
    mixer: MixerServiceClient<Channel>,
}

impl PlaybackClient {
    pub async fn new(host: String, port: u16) -> Result<Self, Error> {
        let url = format!("http://{}:{}", host, port);
        let client = PlaybackServiceClient::connect(url.clone()).await?;
        let mixer = MixerServiceClient::connect(url).await?;
        Ok(Self { client, mixer })
    }

    pub async fn play(&mut self) -> Result<(), Error> {
        let request = tonic::Request::new(PlayRequest {});
        self.client.play(request).await?;
        Ok(())
    }

    pub async fn pause(&mut self) -> Result<(), Error> {
        let request = tonic::Request::new(PauseRequest {});
        self.client.pause(request).await?;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), Error> {
        let request = tonic::Request::new(StopRequest {});
        self.client.stop(request).await?;
        Ok(())
    }

    pub async fn next(&mut self) -> Result<(), Error> {
        let request = tonic::Request::new(NextRequest {});
        self.client.next(request).await?;
        Ok(())
    }

    pub async fn prev(&mut self) -> Result<(), Error> {
        let request = tonic::Request::new(PreviousRequest {});
        self.client.previous(request).await?;
        Ok(())
    }

    pub async fn seek(&mut self, position_ms: u32) -> Result<(), Error> {
        let request = tonic::Request::new(SeekRequest { position_ms });
        self.client.seek(request).await?;
        Ok(())
    }

    pub async fn set_volume(&mut self, volume: u32) -> Result<(), Error> {
        let request = tonic::Request::new(SetVolumeRequest { volume });
        self.mixer.set_volume(request).await?;
        Ok(())
    }

    pub async fn get_volume(&mut self) -> Result<u32, Error> {
        let request = tonic::Request::new(GetVolumeRequest {});
        let response = self.mixer.get_volume(request).await?;
        Ok(response.into_inner().volume)
    }

    /// Mute is separate from volume on purpose: the daemon keeps the level it
    /// was at, so unmuting restores it rather than guessing.
    pub async fn set_mute(&mut self, mute: bool) -> Result<(), Error> {
        let request = tonic::Request::new(SetMuteRequest { mute });
        self.mixer.set_mute(request).await?;
        Ok(())
    }

    pub async fn get_mute(&mut self) -> Result<bool, Error> {
        let request = tonic::Request::new(GetMuteRequest {});
        let response = self.mixer.get_mute(request).await?;
        Ok(response.into_inner().mute)
    }

    pub async fn current(&mut self) -> Result<(Option<Track>, u32, u32, bool), Error> {
        let request = tonic::Request::new(GetCurrentlyPlayingSongRequest {});
        let response = self.client.get_currently_playing_song(request).await?;
        let response = response.into_inner();
        Ok((
            response.track,
            response.index,
            response.position_ms,
            response.is_playing,
        ))
    }
}
