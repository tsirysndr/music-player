//! Track analysis and the automatic DJ, for clients that speak gRPC.

use anyhow::Error;
use music_player_server::api::{
    metadata::v1alpha1::Track,
    music::v1alpha1::{
        analysis_service_client::AnalysisServiceClient, AnalyzeLibraryRequest, AutoDjState,
        AutoDjTarget, GetAnalysisStatusRequest, GetAnalysisStatusResponse, GetAutoDjRequest,
        GetTrackAnalysisRequest, SetAutoDjRequest, SimilarTracksRequest, TrackAnalysis,
    },
};
use tonic::transport::Channel;

pub struct AnalysisClient {
    client: AnalysisServiceClient<Channel>,
}

impl AnalysisClient {
    pub async fn new(host: String, port: u16) -> Result<Self, Error> {
        let client = AnalysisServiceClient::connect(format!("http://{}:{}", host, port)).await?;
        Ok(Self { client })
    }

    /// What a track sounds like.
    ///
    /// `analyze_if_missing` decodes it now when it has not been analysed. Off
    /// is right for anything drawing a waveform — it should show what is ready
    /// rather than block on a decode.
    pub async fn track(
        &mut self,
        id: &str,
        analyze_if_missing: bool,
    ) -> Result<Option<TrackAnalysis>, Error> {
        let response = self
            .client
            .get_track_analysis(tonic::Request::new(GetTrackAnalysisRequest {
                track_id: id.to_string(),
                analyze_if_missing,
            }))
            .await?;
        // An un-analysed track comes back as a present-but-empty analysis; a
        // caller wants "nothing yet", not a row of zeroes.
        Ok(response
            .into_inner()
            .analysis
            .filter(|analysis| analysis.analyzed))
    }

    /// Start a background pass over tracks that have not been analysed.
    /// Returns how many were queued.
    pub async fn analyze_library(&mut self, limit: i32) -> Result<u64, Error> {
        let response = self
            .client
            .analyze_library(tonic::Request::new(AnalyzeLibraryRequest { limit }))
            .await?;
        Ok(response.into_inner().queued)
    }

    pub async fn status(&mut self) -> Result<GetAnalysisStatusResponse, Error> {
        let response = self
            .client
            .get_analysis_status(tonic::Request::new(GetAnalysisStatusRequest {}))
            .await?;
        Ok(response.into_inner())
    }

    /// Tracks that would sound good after this one, best first.
    pub async fn similar(&mut self, id: &str, limit: i32) -> Result<Vec<Track>, Error> {
        let response = self
            .client
            .similar_tracks(tonic::Request::new(SimilarTracksRequest {
                track_id: id.to_string(),
                limit,
            }))
            .await?;
        Ok(response.into_inner().tracks)
    }

    /// Turn auto-DJ on or off, and optionally steer it.
    ///
    /// `target` absent leaves the current one alone; an empty target clears it.
    /// Both are worth being able to say, so they are not collapsed.
    pub async fn set_auto_dj(
        &mut self,
        enabled: bool,
        target: Option<AutoDjTarget>,
    ) -> Result<AutoDjState, Error> {
        let response = self
            .client
            .set_auto_dj(tonic::Request::new(SetAutoDjRequest { enabled, target }))
            .await?;
        response
            .into_inner()
            .state
            .ok_or_else(|| Error::msg("the daemon returned no auto-dj state"))
    }

    pub async fn auto_dj(&mut self) -> Result<AutoDjState, Error> {
        let response = self
            .client
            .get_auto_dj(tonic::Request::new(GetAutoDjRequest {}))
            .await?;
        response
            .into_inner()
            .state
            .ok_or_else(|| Error::msg("the daemon returned no auto-dj state"))
    }
}
