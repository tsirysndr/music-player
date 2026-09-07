use anyhow::{Error, Ok};
use music_player_server::api::music::v1alpha1::{
    playlist_service_client::PlaylistServiceClient, AddItemRequest, CreateRequest, DeleteRequest,
    FindAllRequest, GetItemsRequest, GetPlaylistDetailsRequest, PreviewSmartPlaylistRequest,
    PreviewSmartPlaylistResponse, RegenerateSmartPlaylistRequest, RemoveItemRequest, RenameRequest,
    SmartPlaylist,
};
use music_player_types::types::{Playlist, Track};
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

    pub async fn find(&mut self, id: &str) -> Result<Playlist, Error> {
        let request = tonic::Request::new(GetPlaylistDetailsRequest { id: id.to_string() });
        let response = self.client.get_playlist_details(request).await?;
        Ok(response.into_inner().into())
    }

    pub async fn add(&mut self, id: &str, track_id: &str) -> Result<(), Error> {
        let request = tonic::Request::new(AddItemRequest {
            id: id.to_string(),
            track_id: track_id.to_string(),
        });
        self.client.add_item(request).await?;
        Ok(())
    }

    pub async fn list_songs(&mut self, id: &str) -> Result<Vec<Track>, Error> {
        let request = tonic::Request::new(GetItemsRequest { id: id.to_string() });
        let response = self.client.get_items(request).await?;
        Ok(response
            .into_inner()
            .tracks
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub async fn list_all(&mut self) -> Result<Vec<Playlist>, Error> {
        let request = tonic::Request::new(FindAllRequest {});
        let response = self.client.find_all(request).await?;
        let playlists = response.into_inner().playlists;
        Ok(playlists.into_iter().map(Into::into).collect())
    }

    pub async fn remove(&mut self, id: &str, track_id: &str) -> Result<(), Error> {
        let request = tonic::Request::new(RemoveItemRequest {
            id: id.to_string(),
            track_id: track_id.to_string(),
        });
        self.client.remove_item(request).await?;
        Ok(())
    }

    pub async fn create(&mut self, name: &str) -> Result<String, Error> {
        let request = tonic::Request::new(CreateRequest {
            name: name.to_string(),
            tracks: vec![],
            smart: None,
        });
        let response = self.client.create(request).await?;
        Ok(response.into_inner().id)
    }

    /// Create a smart playlist: the tracks come from `smart`'s RSQL filter and
    /// are regenerated whenever the library changes.
    pub async fn create_smart(
        &mut self,
        name: &str,
        smart: SmartPlaylist,
    ) -> Result<String, Error> {
        let request = tonic::Request::new(CreateRequest {
            name: name.to_string(),
            tracks: vec![],
            smart: Some(smart),
        });
        let response = self.client.create(request).await?;
        Ok(response.into_inner().id)
    }

    /// Re-run a smart playlist's filter against the library as it is now.
    /// Returns how many tracks it holds afterwards.
    pub async fn regenerate_smart(&mut self, id: &str) -> Result<u32, Error> {
        let request = tonic::Request::new(RegenerateSmartPlaylistRequest { id: id.to_string() });
        Ok(self
            .client
            .regenerate_smart_playlist(request)
            .await?
            .into_inner()
            .count)
    }

    /// What a filter would match, without saving anything.
    pub async fn preview_smart(
        &mut self,
        smart: SmartPlaylist,
    ) -> Result<PreviewSmartPlaylistResponse, Error> {
        let request = tonic::Request::new(PreviewSmartPlaylistRequest { smart: Some(smart) });
        Ok(self
            .client
            .preview_smart_playlist(request)
            .await?
            .into_inner())
    }

    pub async fn rename(&mut self, id: &str, name: &str) -> Result<(), Error> {
        let request = tonic::Request::new(RenameRequest {
            id: id.to_string(),
            name: name.to_string(),
        });
        self.client.rename(request).await?;
        Ok(())
    }

    pub async fn delete_playlist(&mut self, id: &str) -> Result<(), Error> {
        let request = tonic::Request::new(DeleteRequest { id: id.to_string() });
        self.client.delete(request).await?;
        Ok(())
    }
}
