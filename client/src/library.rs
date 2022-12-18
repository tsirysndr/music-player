use music_player_server::api::{
    metadata::v1alpha1::{Album, Artist, Track},
    music::v1alpha1::{
        library_service_client::LibraryServiceClient, GetAlbumDetailsRequest, GetAlbumsRequest,
        GetArtistDetailsRequest, GetArtistsRequest, GetTracksRequest,
    },
};
use tonic::transport::Channel;
use anyhow::Error;

pub struct LibraryClient {
    client: LibraryServiceClient<Channel>,
}

impl LibraryClient {
    pub async fn new(host: String, port: u16) -> Result<Self, Error> {
        let url = format!("http://{}:{}", host, port);
        let client = LibraryServiceClient::connect(url).await?;
        Ok(Self { client })
    }

    pub async fn album(&mut self, id: &str) -> Result<Option<Album>, Error> {
        let request = tonic::Request::new(GetAlbumDetailsRequest { id: id.to_string() });
        let response = self.client.get_album_details(request).await?;
        Ok(response.into_inner().album)
    }

    pub async fn albums(&mut self) -> Result<Vec<Album>, Error> {
        let request = tonic::Request::new(GetAlbumsRequest {});
        let response = self.client.get_albums(request).await?;
        Ok(response.into_inner().albums.into_iter().collect())
    }

    pub async fn artist(&mut self, id: &str) -> Result<Option<Artist>, Error> {
        let request = tonic::Request::new(GetArtistDetailsRequest { id: id.to_string() });
        let response = self.client.get_artist_details(request).await?;
        Ok(response.into_inner().artist)
    }

    pub async fn artists(&mut self) -> Result<Vec<Artist>, Error> {
        let request = tonic::Request::new(GetArtistsRequest {});
        let response = self.client.get_artists(request).await?;
        Ok(response.into_inner().artists.into_iter().collect())
    }

    pub async fn songs(&mut self) -> Result<Vec<Track>, Error> {
        let request = tonic::Request::new(GetTracksRequest {});
        let response = self.client.get_tracks(request).await?;
        Ok(response.into_inner().tracks.into_iter().collect())
    }

    pub async fn search(&mut self, query: &str) -> Result<(), Error> {
        todo!()
    }
}
