use music_player_server::api::v1alpha1::playback_service_client::PlaybackServiceClient;
use music_player_settings::{read_settings, Settings};
use tonic::transport::Channel;

pub struct PlaybackClient {
    client: PlaybackServiceClient<Channel>,
}

impl PlaybackClient {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = read_settings().unwrap();
        let settings = config.try_deserialize::<Settings>().unwrap();
        let url = format!("http://[::1]:{}", settings.port);
        let client = PlaybackServiceClient::connect(url).await?;
        Ok(Self { client })
    }

    pub async fn play(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn pause(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn next(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn prev(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn seek(&self, position: u32) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn set_volume(&self, volume: u32) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn status(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn current(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
