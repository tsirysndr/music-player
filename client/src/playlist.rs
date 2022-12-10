use music_player_server::api::v1alpha1::playlist_service_client::PlaylistServiceClient;
use music_player_settings::{read_settings, Settings};
use tonic::transport::Channel;

pub struct PlaylistClient {
    client: PlaylistServiceClient<Channel>,
}

impl PlaylistClient {
    pub async fn new(port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let config = read_settings().unwrap();
        let settings = config.try_deserialize::<Settings>().unwrap();
        let url = format!("http://{}:{}", settings.host, port);
        let client = PlaylistServiceClient::connect(url).await?;
        Ok(Self { client })
    }

    pub async fn add(&self, id: &str) {}

    pub async fn list_songs(&self) {}

    pub async fn clear(&self, id: &str) {}

    pub async fn list_all(&self) {}

    pub async fn play(&self, id: &str) {}

    pub async fn remove(&self, id: &str) {}

    pub async fn shuffle(&self) {}

    pub async fn create(&self, name: &str) {}

    pub async fn delete_playlist(&self, id: &str) {}
}
