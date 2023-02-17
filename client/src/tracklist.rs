use anyhow::Error;
use music_player_server::api::{
    metadata::v1alpha1::Track,
    music::v1alpha1::{
        tracklist_service_client::TracklistServiceClient, AddTrackRequest, AddTracksRequest,
        ClearTracklistRequest, GetTracklistTracksRequest, LoadTracksRequest, PlayNextRequest,
        PlayTrackAtRequest, RemoveTrackAtRequest,
    },
};
use music_player_types::types;
use tonic::transport::Channel;
pub struct TracklistClient {
    client: TracklistServiceClient<Channel>,
}

impl TracklistClient {
    pub async fn new(host: String, port: u16) -> Result<Self, Error> {
        let url = format!("tcp://{}:{}", host, port);
        let client = TracklistServiceClient::connect(url).await?;
        Ok(Self { client })
    }

    pub async fn add(&mut self, id: &str) -> Result<(), Error> {
        let request = tonic::Request::new(AddTrackRequest {
            track: Some(Track {
                id: id.to_string(),
                ..Default::default()
            }),
            ..Default::default()
        });
        self.client.add_track(request).await?;
        Ok(())
    }

    pub async fn add_tracks(&mut self, ids: &[&str]) -> Result<(), Error> {
        let request = tonic::Request::new(AddTracksRequest {
            tracks: ids
                .iter()
                .map(|id| Track {
                    id: id.to_string(),
                    ..Default::default()
                })
                .collect(),
        });
        self.client.add_tracks(request).await?;
        Ok(())
    }

    pub async fn clear(&mut self) -> Result<(), Error> {
        let request = tonic::Request::new(ClearTracklistRequest {
            ..Default::default()
        });
        self.client.clear_tracklist(request).await?;
        Ok(())
    }

    pub async fn list(&mut self) -> Result<(Vec<Track>, Vec<Track>), Error> {
        let request = tonic::Request::new(GetTracklistTracksRequest {
            ..Default::default()
        });
        let response = self.client.get_tracklist_tracks(request).await?;
        let response = response.into_inner();
        Ok((response.previous_tracks, response.next_tracks))
    }

    pub async fn remove(&mut self, position: u32) -> Result<(), Error> {
        let request = tonic::Request::new(RemoveTrackAtRequest {
            position,
            ..Default::default()
        });
        self.client.remove_track_at(request).await?;
        Ok(())
    }

    pub async fn play_track_at(&mut self, index: usize) -> Result<(), Error> {
        let request = tonic::Request::new(PlayTrackAtRequest {
            index: index as u32,
        });
        self.client.play_track_at(request).await?;
        Ok(())
    }

    pub async fn play_next(&mut self, track: types::Track) -> Result<(), Error> {
        let request = tonic::Request::new(PlayNextRequest {
            track: Some(track.into()),
        });
        self.client.play_next(request).await?;
        Ok(())
    }

    pub async fn load_tracks(
        &mut self,
        tracks: Vec<types::Track>,
        start_index: i32,
    ) -> Result<(), Error> {
        let request = tonic::Request::new(LoadTracksRequest {
            tracks: tracks.into_iter().map(Into::into).collect(),
            start_index,
        });
        self.client.load_tracks(request).await?;
        Ok(())
    }
}
