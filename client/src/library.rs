use music_player_server::{
    api::v1alpha1::{
        library_service_client::LibraryServiceClient, GetAlbumsRequest, GetArtistsRequest,
        GetTracksRequest,
    },
    metadata::v1alpha1::{Album, Artist, Track},
};
use music_player_settings::{read_settings, Settings};
use tonic::transport::Channel;

pub struct LibraryClient {
    client: LibraryServiceClient<Channel>,
}

impl LibraryClient {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = read_settings().unwrap();
        let settings = config.try_deserialize::<Settings>().unwrap();
        let url = format!("http://[::1]:{}", settings.port);
        let client = LibraryServiceClient::connect(url).await?;
        Ok(Self { client })
    }

    pub async fn albums(&mut self) -> Result<Vec<Album>, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(GetAlbumsRequest {});
        let response = self.client.get_albums(request).await?;
        Ok(response.into_inner().albums.into_iter().collect())
    }

    pub async fn artists(&mut self) -> Result<Vec<Artist>, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(GetArtistsRequest {});
        let response = self.client.get_artists(request).await?;
        Ok(response.into_inner().artists.into_iter().collect())
    }

    pub async fn songs(&mut self) -> Result<Vec<Track>, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(GetTracksRequest {});
        let response = self.client.get_tracks(request).await?;
        Ok(response.into_inner().tracks.into_iter().collect())
    }

    pub async fn search(&mut self, query: &str) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}
