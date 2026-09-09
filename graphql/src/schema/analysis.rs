//! What a track sounds like, and the automatic DJ built on it.
//!
//! Read-mostly: a client asks for a waveform to draw and gets whatever has been
//! computed. Computing it is deliberately not the default — decoding a track
//! takes seconds, and a screen full of rows must not turn into a screen full of
//! decodes.

use std::sync::Arc;

use async_graphql::*;
use music_player_provider::ProviderState;
use music_player_storage::{track_analysis, Database};

/// A named mood with how strongly it was detected.
#[derive(SimpleObject, Clone, Debug)]
pub struct Mood {
    pub name: String,
    pub confidence: f32,
}

/// How a track sounds.
#[derive(SimpleObject, Clone, Debug, Default)]
pub struct TrackAnalysis {
    pub track_id: String,
    /// Peak per bar, 0–255, left to right, ready to draw. Empty when the track
    /// has not been analysed.
    ///
    /// A list of integers rather than a base64 blob: it is read by a canvas
    /// that wants numbers, and 400 of them is a few kilobytes either way.
    pub waveform: Vec<u32>,
    pub bpm: Option<f32>,
    /// How much to believe the bpm, 0–1.
    pub bpm_confidence: Option<f32>,
    /// -1 dark to 1 bright.
    pub valence: Option<f32>,
    /// 0 calm to 1 driving.
    pub arousal: Option<f32>,
    pub moods: Vec<Mood>,
    /// Integrated loudness, LUFS.
    pub lufs: Option<f32>,
    pub true_peak_db: Option<f32>,
    /// Seconds, as decoded rather than as the tags claim.
    pub duration: f32,
    /// False means nothing has been computed; every other field is then empty
    /// rather than zero.
    pub analyzed: bool,
}

/// Which library the ids belong to. Empty is the daemon's own.
async fn source(ctx: &Context<'_>) -> String {
    let Ok(providers) = ctx.data::<Arc<ProviderState>>() else {
        return String::new();
    };
    providers
        .config()
        .await
        .map(|config| config.id)
        .unwrap_or_default()
}

#[derive(Default)]
pub struct AnalysisQuery;

#[Object]
impl AnalysisQuery {
    /// How a track sounds, if it has been analysed.
    ///
    /// Never analyses on demand. A waveform under a player should appear or not
    /// appear; it should not make opening the player cost a decode, and a
    /// client that wants one computed asks the daemon to analyse the library.
    async fn track_analysis(&self, ctx: &Context<'_>, track_id: String) -> Result<TrackAnalysis> {
        let db = ctx.data::<Database>()?;
        let source = source(ctx).await;

        let Some(analysis) = track_analysis::get(db.get_connection(), &source, &track_id).await
        else {
            return Ok(TrackAnalysis {
                track_id,
                ..Default::default()
            });
        };

        Ok(TrackAnalysis {
            track_id,
            waveform: analysis.waveform.iter().map(|bar| *bar as u32).collect(),
            bpm: analysis.bpm,
            bpm_confidence: analysis.bpm_confidence,
            valence: analysis.valence,
            arousal: analysis.arousal,
            moods: analysis
                .moods
                .iter()
                .map(|(name, confidence)| Mood {
                    name: name.clone(),
                    confidence: *confidence,
                })
                .collect(),
            lufs: analysis.lufs,
            true_peak_db: analysis.true_peak_db,
            duration: analysis.duration,
            analyzed: true,
        })
    }

    /// How many tracks in the connected library have been analysed.
    async fn analyzed_tracks(&self, ctx: &Context<'_>) -> Result<u64> {
        let db = ctx.data::<Database>()?;
        Ok(track_analysis::coverage(db.get_connection(), &source(ctx).await).await)
    }
}
