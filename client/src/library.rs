use anyhow::Error;
use music_player_server::api::{
    metadata::v1alpha1::{Album, Artist, Track},
    music::v1alpha1::{
        library_service_client::LibraryServiceClient, GetAlbumDetailsRequest, GetAlbumsRequest,
        GetArtistDetailsRequest, GetArtistsRequest, GetTrackDetailsRequest, GetTracksRequest,
    },
};
use tonic::transport::Channel;

pub struct LibraryClient {
    client: LibraryServiceClient<Channel>,
}

impl LibraryClient {
    pub async fn new(host: String, port: u16) -> Result<Self, Error> {
        let url = format!("tcp://{}:{}", host, port);
        let client = LibraryServiceClient::connect(url).await?;
        Ok(Self { client })
    }

    pub async fn album(&mut self, id: &str) -> Result<Option<Album>, Error> {
        let request = tonic::Request::new(GetAlbumDetailsRequest { id: id.to_string() });
        let response = self.client.get_album_details(request).await?;
        Ok(response.into_inner().album)
    }

    pub async fn albums(
        &mut self,
        filter: Option<String>,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<Album>, Error> {
        let filter = match filter {
            Some(filter) => filter,
            None => "".to_string(),
        };
        let request = tonic::Request::new(GetAlbumsRequest {
            offset,
            limit,
            filter,
        });
        let response = self.client.get_albums(request).await?;
        Ok(response.into_inner().albums.into_iter().collect())
    }

    pub async fn artist(&mut self, id: &str) -> Result<Option<Artist>, Error> {
        let request = tonic::Request::new(GetArtistDetailsRequest { id: id.to_string() });
        let response = self.client.get_artist_details(request).await?;
        Ok(response.into_inner().artist)
    }

    pub async fn artists(
        &mut self,
        filter: Option<String>,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<Artist>, Error> {
        let filter = match filter {
            Some(filter) => filter,
            None => "".to_string(),
        };
        let request = tonic::Request::new(GetArtistsRequest {
            offset,
            limit,
            filter,
        });
        let response = self.client.get_artists(request).await?;
        Ok(response.into_inner().artists.into_iter().collect())
    }

    pub async fn songs(
        &mut self,
        filter: Option<String>,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<Track>, Error> {
        let filter = match filter {
            Some(filter) => filter,
            None => "".to_string(),
        };
        let request = tonic::Request::new(GetTracksRequest {
            offset,
            limit,
            filter,
        });
        let response = self.client.get_tracks(request).await?;
        Ok(response.into_inner().tracks.into_iter().collect())
    }

    pub async fn song(&mut self, id: &str) -> Result<Option<Track>, Error> {
        let request = tonic::Request::new(GetTrackDetailsRequest { id: id.to_string() });
        let response = self.client.get_track_details(request).await?;
        Ok(response.into_inner().track)
    }

    pub async fn search(&mut self, query: &str) -> Result<(), Error> {
        todo!()
    }
}
