use music_player_server::api::v1alpha1::tracklist_service_client::TracklistServiceClient;
use music_player_settings::{read_settings, Settings};
use tonic::transport::Channel;

pub struct TracklistClient {
    client: TracklistServiceClient<Channel>,
}

impl TracklistClient {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = read_settings().unwrap();
        let settings = config.try_deserialize::<Settings>().unwrap();
        let url = format!("http://[::1]:{}", settings.port);
        let client = TracklistServiceClient::connect(url).await?;
        Ok(Self { client })
    }

    pub async fn add(&self, id: &str) {}

    pub async fn clear(&self) {}

    pub async fn list(&self) {}

    pub async fn remove(&self, id: &str) {}
}
