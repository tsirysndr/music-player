use music_player_server::api::{
    metadata::v1alpha1::Track,
    music::v1alpha1::{
        tracklist_service_client::TracklistServiceClient, AddTrackRequest, ClearTracklistRequest,
        GetTracklistTracksRequest, PlayTrackAtRequest, RemoveTrackRequest,
    },
};
use music_player_settings::{read_settings, Settings};
use tonic::transport::Channel;

pub struct TracklistClient {
    client: TracklistServiceClient<Channel>,
}

impl TracklistClient {
    pub async fn new(port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let config = read_settings().unwrap();
        let settings = config.try_deserialize::<Settings>().unwrap();
        let url = format!("http://{}:{}", settings.host, port);
        let client = TracklistServiceClient::connect(url).await?;
        Ok(Self { client })
    }

    pub async fn add(&mut self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(AddTrackRequest {
            track: Some(Track {
                id: id.to_string(),
                ..Default::default()
            }),
            ..Default::default()
        });
        let response = self.client.add_track(request).await?;
        Ok(())
    }

    pub async fn add_tracks(&mut self, ids: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(AddTrackRequest {
            ..Default::default()
        });
        let response = self.client.add_track(request).await?;
        Ok(())
    }

    pub async fn clear(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(ClearTracklistRequest {
            ..Default::default()
        });
        let response = self.client.clear_tracklist(request).await?;
        Ok(())
    }

    pub async fn list(&mut self) -> Result<(Vec<Track>, Vec<Track>), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(GetTracklistTracksRequest {
            ..Default::default()
        });
        let response = self.client.get_tracklist_tracks(request).await?;
        let response = response.into_inner();
        Ok((response.previous_tracks, response.next_tracks))
    }

    pub async fn remove(&mut self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(RemoveTrackRequest {
            ..Default::default()
        });
        let response = self.client.remove_track(request).await?;
        Ok(())
    }

    pub async fn play_track_at(&mut self, index: usize) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(PlayTrackAtRequest {
            index: index as u32,
        });
        let response = self.client.play_track_at(request).await?;
        Ok(())
    }
}
