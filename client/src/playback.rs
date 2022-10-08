use music_player_server::{
    api::v1alpha1::{
        playback_service_client::PlaybackServiceClient, GetCurrentlyPlayingSongRequest,
        NextRequest, PauseRequest, PlayRequest, PreviousRequest, StopRequest,
    },
    metadata::v1alpha1::Track,
};
use music_player_settings::{read_settings, Settings};
use tonic::{codegen::http::request, transport::Channel};

pub struct PlaybackClient {
    client: PlaybackServiceClient<Channel>,
}

impl PlaybackClient {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = read_settings().unwrap();
        let settings = config.try_deserialize::<Settings>().unwrap();
        let url = format!("http://{}:{}", settings.host, settings.port);
        let client = PlaybackServiceClient::connect(url).await?;
        Ok(Self { client })
    }

    pub async fn play(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(PlayRequest {});
        self.client.play(request).await?;
        Ok(())
    }

    pub async fn pause(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(PauseRequest {});
        self.client.pause(request).await?;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(StopRequest {});
        self.client.stop(request).await?;
        Ok(())
    }

    pub async fn next(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(NextRequest {});
        self.client.next(request).await?;
        Ok(())
    }

    pub async fn prev(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(PreviousRequest {});
        self.client.previous(request).await?;
        Ok(())
    }

    pub async fn seek(&mut self, position: u32) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn set_volume(&mut self, volume: u32) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn status(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn current(
        &mut self,
    ) -> Result<(Option<Track>, u32, u32, bool), Box<dyn std::error::Error>> {
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
