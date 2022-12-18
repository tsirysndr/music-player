use anyhow::Error;
use music_player_server::api::music::v1alpha1::playlist_service_client::PlaylistServiceClient;
use music_player_settings::{read_settings, Settings};
use tonic::transport::Channel;
pub struct PlaylistClient {
    client: PlaylistServiceClient<Channel>,
}

impl PlaylistClient {
    pub async fn new(host: String, port: u16) -> Result<Self, Error> {
        let url = format!("http://{}:{}", host, port);
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
