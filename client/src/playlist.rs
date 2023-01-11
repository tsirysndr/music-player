use anyhow::{Error, Ok};
use music_player_server::api::music::v1alpha1::{
    playlist_service_client::PlaylistServiceClient, FindAllRequest, GetPlaylistDetailsRequest,
};
use music_player_settings::{read_settings, Settings};
use music_player_types::types::Playlist;
use tonic::transport::Channel;
pub struct PlaylistClient {
    client: PlaylistServiceClient<Channel>,
}

impl PlaylistClient {
    pub async fn new(host: String, port: u16) -> Result<Self, Error> {
        let url = format!("tcp://{}:{}", host, port);
        let client = PlaylistServiceClient::connect(url).await?;
        Ok(Self { client })
    }

    pub async fn find(&mut self, id: &str) -> Result<Playlist, Error> {
        let request = tonic::Request::new(GetPlaylistDetailsRequest { id: id.to_string() });
        let response = self.client.get_playlist_details(request).await?;
        Ok(response.into_inner().into())
    }

    pub async fn add(&mut self, id: &str) {
        todo!()
    }

    pub async fn list_songs(&mut self) {
        todo!()
    }

    pub async fn clear(&mut self, id: &str) {
        todo!()
    }

    pub async fn list_all(&mut self) -> Result<Vec<Playlist>, Error> {
        let request = tonic::Request::new(FindAllRequest {});
        let response = self.client.find_all(request).await?;
        let playlists = response.into_inner().playlists;
        Ok(playlists.into_iter().map(Into::into).collect())
    }

    pub async fn play(&mut self, id: &str) {
        todo!()
    }

    pub async fn remove(&mut self, id: &str) {
        todo!()
    }

    pub async fn shuffle(&mut self) {
        todo!()
    }

    pub async fn create(&mut self, name: &str) {
        todo!()
    }

    pub async fn delete_playlist(&mut self, id: &str) {
        todo!()
    }
}
