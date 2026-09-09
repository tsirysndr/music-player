//! Track analysis over gRPC, and the automatic DJ built on it.
//!
//! Analysis itself lives in `music-player-storage`; this is the part that knows
//! which library is connected, how to turn a track id into audio, and how to
//! put chosen tracks into the queue.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use music_player_playback::player::PlayerCommand;
use music_player_provider::{Page, ProviderState};
use music_player_storage::{auto_dj, track_analysis, track_analysis::TrackRef, Database};
use music_player_tracklist::Tracklist as TracklistState;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;

use crate::api::music::v1alpha1::{
    analysis_service_server::AnalysisService, AnalyzeLibraryRequest, AnalyzeLibraryResponse,
    AutoDjState, AutoDjTarget, GetAnalysisStatusRequest, GetAnalysisStatusResponse,
    GetAutoDjRequest, GetAutoDjResponse, GetTrackAnalysisRequest, GetTrackAnalysisResponse,
    MoodLabel, SetAutoDjRequest, SetAutoDjResponse, SimilarTracksRequest, SimilarTracksResponse,
    TrackAnalysis,
};
use crate::library::provider_status;

/// How many tracks auto-DJ keeps queued ahead of the one playing.
///
/// Enough that the next few are visible and a slow pick is never heard, few
/// enough that the queue still reacts to a change of target within a track or
/// two rather than being committed a half-hour in advance.
const QUEUE_AHEAD: usize = 5;

/// What auto-DJ is doing, shared between the rpc handlers and the loop.
#[derive(Default)]
pub struct AutoDj {
    enabled: AtomicBool,
    target: RwLock<auto_dj::Target>,
}

impl AutoDj {
    pub async fn target(&self) -> auto_dj::Target {
        *self.target.read().await
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
}

/// Progress of a background analysis pass.
#[derive(Default)]
pub struct AnalysisProgress {
    running: AtomicBool,
    remaining: AtomicU64,
}

pub struct Analysis {
    db: Database,
    providers: Arc<ProviderState>,
    tracklist: Arc<std::sync::Mutex<TracklistState>>,
    cmd_tx: Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>,
    auto_dj: Arc<AutoDj>,
    progress: Arc<AnalysisProgress>,
}

impl Analysis {
    pub fn new(
        db: Database,
        providers: Arc<ProviderState>,
        tracklist: Arc<std::sync::Mutex<TracklistState>>,
        cmd_tx: Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>,
        auto_dj: Arc<AutoDj>,
        progress: Arc<AnalysisProgress>,
    ) -> Self {
        Self {
            db,
            providers,
            tracklist,
            cmd_tx,
            auto_dj,
            progress,
        }
    }

    /// Which library the ids belong to. Empty is the daemon's own.
    ///
    /// Analysis is stored per library because ids are only unique within one:
    /// without this a Navidrome track and a local one could share a row and a
    /// waveform would be drawn for the wrong song.
    async fn source(&self) -> String {
        self.providers
            .config()
            .await
            .map(|config| config.id)
            .unwrap_or_default()
    }
}

#[tonic::async_trait]
impl AnalysisService for Analysis {
    async fn get_track_analysis(
        &self,
        request: tonic::Request<GetTrackAnalysisRequest>,
    ) -> Result<tonic::Response<GetTrackAnalysisResponse>, tonic::Status> {
        let request = request.into_inner();
        let source = self.source().await;
        let db = self.db.get_connection();

        let stored = track_analysis::get(db, &source, &request.track_id).await;
        let analysis = match (stored, request.analyze_if_missing) {
            (Some(analysis), _) => Some(analysis),
            (None, false) => None,
            (None, true) => {
                let track = self.resolve(&request.track_id).await?;
                // A failure here is the track's, not the request's: an
                // unplayable url or an unsupported codec is worth reporting
                // rather than returning an empty analysis that looks like
                // "not analysed yet" and invites an endless retry.
                Some(
                    track_analysis::ensure(db, &source, &track)
                        .await
                        .map_err(|e| tonic::Status::internal(e.to_string()))?,
                )
            }
        };

        Ok(tonic::Response::new(GetTrackAnalysisResponse {
            analysis: Some(to_proto(&request.track_id, analysis.as_ref())),
        }))
    }

    async fn analyze_library(
        &self,
        request: tonic::Request<AnalyzeLibraryRequest>,
    ) -> Result<tonic::Response<AnalyzeLibraryResponse>, tonic::Status> {
        let limit = request.into_inner().limit.clamp(1, 5_000) as usize;

        if self.progress.running.swap(true, Ordering::SeqCst) {
            return Err(tonic::Status::failed_precondition(
                "an analysis pass is already running",
            ));
        }

        let tracks = self.library_tracks(limit).await?;
        let source = self.source().await;
        let db = self.db.get_connection().clone();
        let progress = Arc::clone(&self.progress);

        // Filtering out what is already analysed happens in the task, not here:
        // it is a query per track, and this request should return as soon as
        // the work is accepted.
        progress
            .remaining
            .store(tracks.len() as u64, Ordering::Relaxed);
        let queued = tracks.len() as u64;

        tokio::spawn(async move {
            for track in tracks {
                if let Err(cause) = track_analysis::ensure(&db, &source, &track).await {
                    // One track that cannot be decoded must not end the pass.
                    tracing::debug!(track = %track.title, %cause, "could not analyse");
                }
                progress.remaining.fetch_sub(1, Ordering::Relaxed);
            }
            progress.running.store(false, Ordering::SeqCst);
        });

        Ok(tonic::Response::new(AnalyzeLibraryResponse { queued }))
    }

    async fn get_analysis_status(
        &self,
        _request: tonic::Request<GetAnalysisStatusRequest>,
    ) -> Result<tonic::Response<GetAnalysisStatusResponse>, tonic::Status> {
        let source = self.source().await;
        Ok(tonic::Response::new(GetAnalysisStatusResponse {
            analyzed: track_analysis::coverage(self.db.get_connection(), &source).await,
            running: self.progress.running.load(Ordering::Relaxed),
            remaining: self.progress.remaining.load(Ordering::Relaxed),
        }))
    }

    async fn similar_tracks(
        &self,
        request: tonic::Request<SimilarTracksRequest>,
    ) -> Result<tonic::Response<SimilarTracksResponse>, tonic::Status> {
        let request = request.into_inner();
        let limit = request.limit.clamp(1, 100) as usize;
        let source = self.source().await;
        let db = self.db.get_connection();

        let seed = track_analysis::get(db, &source, &request.track_id).await;
        if seed.is_none() {
            return Err(tonic::Status::failed_precondition(format!(
                "{} has not been analysed yet",
                request.track_id
            )));
        }

        // The seed itself is never a suggestion for what to play after it.
        let exclude: HashSet<String> = [request.track_id.clone()].into();
        let ids = auto_dj::plan(
            &self.candidates(&source).await,
            seed.as_ref(),
            &auto_dj::Target::default(),
            &exclude,
            limit,
        );

        Ok(tonic::Response::new(SimilarTracksResponse {
            tracks: self.hydrate(&ids).await,
        }))
    }

    async fn set_auto_dj(
        &self,
        request: tonic::Request<SetAutoDjRequest>,
    ) -> Result<tonic::Response<SetAutoDjResponse>, tonic::Status> {
        let request = request.into_inner();

        // A target sent as absent leaves the current one alone; sent as an
        // empty message it clears it. Both are things a caller wants to say,
        // and collapsing them would make "stop steering" impossible to express.
        if let Some(target) = request.target {
            *self.auto_dj.target.write().await = auto_dj::Target {
                bpm: target.bpm,
                valence: target.valence,
                arousal: target.arousal,
            };
        }
        self.auto_dj
            .enabled
            .store(request.enabled, Ordering::Relaxed);

        Ok(tonic::Response::new(SetAutoDjResponse {
            state: Some(self.state().await),
        }))
    }

    async fn get_auto_dj(
        &self,
        _request: tonic::Request<GetAutoDjRequest>,
    ) -> Result<tonic::Response<GetAutoDjResponse>, tonic::Status> {
        Ok(tonic::Response::new(GetAutoDjResponse {
            state: Some(self.state().await),
        }))
    }
}

impl Analysis {
    async fn state(&self) -> AutoDjState {
        let source = self.source().await;
        let target = self.auto_dj.target().await;
        let queued_ahead = self.tracklist.lock().unwrap().tracks().1.len() as u32;

        AutoDjState {
            enabled: self.auto_dj.is_enabled(),
            target: Some(AutoDjTarget {
                bpm: target.bpm,
                valence: target.valence,
                arousal: target.arousal,
            }),
            queued_ahead,
            candidates: track_analysis::matchable(self.db.get_connection(), &source)
                .await
                .len() as u64,
        }
    }

    /// Every analysed track that could be played.
    ///
    /// No library lookups: the artist each candidate needs was copied into its
    /// row when it was analysed, precisely so that ranking a whole library does
    /// not become a request per track.
    async fn candidates(&self, source: &str) -> Vec<auto_dj::Candidate> {
        track_analysis::matchable(self.db.get_connection(), source)
            .await
            .into_iter()
            .map(|row| auto_dj::Candidate {
                track_id: row.track_id,
                artist: row.artist,
                analysis: row.analysis,
            })
            .collect()
    }

    /// One track, from whichever library is connected.
    async fn resolve(&self, id: &str) -> Result<TrackRef, tonic::Status> {
        if let Some(current) = self.providers.current().await {
            let track = current.provider.track(id).await.map_err(provider_status)?;
            let track = music_player_provider::url::decorate(track, &current.config);
            return Ok(TrackRef {
                id: track.id.clone(),
                uri: track.uri.clone(),
                artist: track.artist.clone(),
                title: track.title.clone(),
            });
        }

        let track =
            music_player_storage::repo::track::TrackRepository::new(self.db.get_connection())
                .find(id)
                .await
                .map_err(|_| tonic::Status::not_found(format!("no track {id}")))?;

        Ok(TrackRef {
            id: track.id,
            uri: track.uri,
            artist: track.artist,
            title: track.title,
        })
    }

    /// A page of the connected library, for a background analysis pass.
    async fn library_tracks(&self, limit: usize) -> Result<Vec<TrackRef>, tonic::Status> {
        if let Some(current) = self.providers.current().await {
            let tracks = current
                .provider
                .tracks(None, Page::new(0, limit as i32))
                .await
                .map_err(provider_status)?;
            let tracks = music_player_provider::url::decorate_all(tracks, &current.config);
            return Ok(tracks
                .into_iter()
                .map(|track| TrackRef {
                    id: track.id,
                    uri: track.uri,
                    artist: track.artist,
                    title: track.title,
                })
                .collect());
        }

        let tracks =
            music_player_storage::repo::track::TrackRepository::new(self.db.get_connection())
                .find_all(None, Some(0), limit as u64)
                .await
                .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tracks
            .into_iter()
            .map(|track| TrackRef {
                id: track.id,
                uri: track.uri,
                artist: track.artist,
                title: track.title,
            })
            .collect())
    }

    /// Turn planned ids into playable tracks, in the order planned.
    ///
    /// A lookup per track, which is why only the handful actually chosen ever
    /// reaches here — never the candidate pool.
    async fn hydrate(&self, ids: &[String]) -> Vec<crate::api::metadata::v1alpha1::Track> {
        let mut tracks = Vec::with_capacity(ids.len());
        for id in ids {
            match self.full_track(id).await {
                Ok(track) => tracks.push(track),
                // A track that has been analysed but has since gone from the
                // library is simply not offered.
                Err(cause) => tracing::debug!(%id, %cause, "planned track is gone"),
            }
        }
        tracks
    }

    async fn full_track(
        &self,
        id: &str,
    ) -> Result<crate::api::metadata::v1alpha1::Track, tonic::Status> {
        if let Some(current) = self.providers.current().await {
            let track = current.provider.track(id).await.map_err(provider_status)?;
            return Ok(music_player_provider::url::decorate(track, &current.config).into());
        }
        let track =
            music_player_storage::repo::track::TrackRepository::new(self.db.get_connection())
                .find(id)
                .await
                .map_err(|_| tonic::Status::not_found(format!("no track {id}")))?;
        Ok(track.into())
    }
}

fn to_proto(track_id: &str, analysis: Option<&music_player_analysis::Analysis>) -> TrackAnalysis {
    let Some(analysis) = analysis else {
        return TrackAnalysis {
            track_id: track_id.to_string(),
            analyzed: false,
            ..Default::default()
        };
    };

    TrackAnalysis {
        track_id: track_id.to_string(),
        waveform: analysis.waveform.clone(),
        bpm: analysis.bpm,
        bpm_confidence: analysis.bpm_confidence,
        valence: analysis.valence,
        arousal: analysis.arousal,
        moods: analysis
            .moods
            .iter()
            .map(|(name, confidence)| MoodLabel {
                name: name.clone(),
                confidence: *confidence,
            })
            .collect(),
        lufs: analysis.lufs,
        true_peak_db: analysis.true_peak_db,
        duration: analysis.duration,
        analyzed: true,
    }
}

/// Keep the queue full while auto-DJ is on.
///
/// A loop rather than a hook on track change, because the queue is changed by
/// more than the player: a user clearing it, a client queueing something, an
/// agent building a set. Polling reacts to all of them without every one of
/// them having to remember to say so.
pub async fn run_auto_dj(dj: Analysis) {
    // Slow enough to be free, fast enough that the queue is never seen empty:
    // it only ever has to top up before the *current* track ends.
    let mut ticker = tokio::time::interval(std::time::Duration::from_secs(3));

    loop {
        ticker.tick().await;
        if !dj.auto_dj.is_enabled() {
            continue;
        }
        if let Err(cause) = dj.top_up().await {
            tracing::debug!(%cause, "auto-dj could not top up the queue");
        }
    }
}

impl Analysis {
    /// Add tracks until the queue is `QUEUE_AHEAD` deep.
    async fn top_up(&self) -> Result<(), tonic::Status> {
        let (played, upcoming, current) = {
            let tracklist = self.tracklist.lock().unwrap();
            let (played, upcoming) = tracklist.tracks();
            let (current, _) = tracklist.current_track();
            (played, upcoming, current)
        };

        let wanted = QUEUE_AHEAD.saturating_sub(upcoming.len());
        if wanted == 0 {
            return Ok(());
        }

        let source = self.source().await;
        let db = self.db.get_connection();

        // What to follow: the last thing queued if there is one, else what is
        // playing. Following the current track when five are already queued
        // would plan a set that starts from the wrong place.
        let seed_id = upcoming
            .last()
            .or(current.as_ref())
            .or(played.last())
            .map(|track| track.id.clone());
        let seed = match &seed_id {
            Some(id) => track_analysis::get(db, &source, id).await,
            None => None,
        };

        // Nothing in the queue is a candidate, and neither is anything already
        // played — an automatic set that loops back after four tracks is worse
        // than one that stops.
        let exclude: HashSet<String> = played
            .iter()
            .chain(upcoming.iter())
            .map(|track| track.id.clone())
            .chain(seed_id)
            .collect();

        let ids = auto_dj::plan(
            &self.candidates(&source).await,
            seed.as_ref(),
            &self.auto_dj.target().await,
            &exclude,
            wanted,
        );
        if ids.is_empty() {
            return Ok(());
        }

        let tracks: Vec<music_player_entity::track::Model> = self
            .hydrate(&ids)
            .await
            .into_iter()
            .map(Into::into)
            .collect();
        if tracks.is_empty() {
            return Ok(());
        }

        tracing::debug!(added = tracks.len(), "auto-dj topped up the queue");
        self.cmd_tx
            .lock()
            .unwrap()
            .send(PlayerCommand::LoadTracklist {
                tracks,
                // Appending. Auto-DJ must never interrupt what is playing —
                // that is the whole difference between it and a shuffle.
                start_index: None,
            })
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(())
    }
}
