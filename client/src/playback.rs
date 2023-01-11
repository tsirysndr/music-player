use anyhow::Error;
use music_player_server::api::{
    metadata::v1alpha1::Track,
    music::v1alpha1::{
        playback_service_client::PlaybackServiceClient, GetCurrentlyPlayingSongRequest,
        NextRequest, PauseRequest, PlayRequest, PreviousRequest, StopRequest,
    },
};
use tonic::transport::Channel;
pub struct PlaybackClient {
    client: PlaybackServiceClient<Channel>,
}

impl PlaybackClient {
    pub async fn new(host: String, port: u16) -> Result<Self, Error> {
        let url = format!("tcp://{}:{}", host, port);
        let client = PlaybackServiceClient::connect(url).await?;
        Ok(Self { client })
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

    pub async fn seek(&mut self, position: u32) -> Result<(), Error> {
        Ok(())
    }

    pub async fn set_volume(&mut self, volume: u32) -> Result<(), Error> {
        Ok(())
    }

    pub async fn status(&mut self) -> Result<(), Error> {
        Ok(())
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
