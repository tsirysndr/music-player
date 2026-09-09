//! What a track sounds like, as numbers.
//!
//! Four things, from one decode pass: a **waveform** to draw, **loudness** to
//! level tracks against each other, a **tempo**, and a **mood** as a point in
//! valence/arousal space. Together they are enough to answer "what should play
//! after this?" without knowing anything about genres or what anyone has
//! listened to.
//!
//! Analysis is expensive and its inputs never change — the same bytes always
//! give the same answer — so results are stored and a track is analysed once.
//! That belongs to the caller; this crate only computes.

mod decode;
mod features;
mod waveform;

pub use decode::{decode, Decoded};

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// How many bars a stored waveform has.
///
/// Wide enough to look like the track rather than a smear, small enough that
/// storing one per track is unremarkable — 400 bytes, against several megabytes
/// of audio.
pub const WAVEFORM_BINS: usize = 400;

/// Everything analysis knows about one track.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Analysis {
    /// Peak amplitude per bar, 0–255, left to right. Drawn as-is: the shaping
    /// that makes a waveform readable is already applied.
    pub waveform: Vec<u8>,
    /// Beats per minute.
    pub bpm: Option<f32>,
    /// How much to believe the bpm, 0–1. Tempo detection is confident on a
    /// four-to-the-floor track and much less so on something rubato, and a
    /// caller matching tracks by tempo needs to know which it has.
    pub bpm_confidence: Option<f32>,
    /// Pleasantness, -1 (dark) to 1 (bright).
    pub valence: Option<f32>,
    /// Energy, 0 (calm) to 1 (driving).
    pub arousal: Option<f32>,
    /// Mood labels with confidences, as the detector named them.
    pub moods: Vec<(String, f32)>,
    /// Integrated loudness, LUFS. The EBU R128 measure — what "as loud as" means
    /// when comparing two tracks.
    pub lufs: Option<f32>,
    /// True peak, in dBTP. Needed alongside loudness: a track can be quiet on
    /// average and still clip when gained up.
    pub true_peak_db: Option<f32>,
    /// Seconds, as decoded rather than as the tags claim.
    pub duration: f32,
}

impl Analysis {
    /// Whether there is enough here to match tracks against each other.
    ///
    /// A waveform alone is worth storing — it is drawn — but it says nothing
    /// about what would sound good next.
    pub fn is_useful_for_matching(&self) -> bool {
        self.bpm.is_some() || (self.valence.is_some() && self.arousal.is_some())
    }

    /// The gain, in dB, that would bring this track to a target loudness.
    ///
    /// Limited so the true peak stays a decibel below clipping: turning a quiet
    /// master up to match a loud one is exactly when a track starts to clip.
    pub fn gain_for_target(&self, target_lufs: f32) -> Option<f32> {
        let lufs = self.lufs?;
        let mut gain = target_lufs - lufs;
        if let Some(peak) = self.true_peak_db {
            gain = gain.min(-1.0 - peak);
        }
        Some(gain.clamp(-24.0, 24.0))
    }
}

/// Analyse audio.
///
/// `extension_hint` is the file extension or codec name when it is known, to
/// help the prober; a wrong hint costs nothing.
///
/// Feature extraction that fails does not fail the analysis. A track whose
/// tempo cannot be found still has a waveform worth drawing and a loudness
/// worth knowing, and returning nothing would mean re-attempting all of it
/// every time.
pub fn analyze(bytes: &[u8], extension_hint: Option<&str>) -> Result<Analysis> {
    let decoded = decode(bytes, extension_hint)?;

    let mut analysis = Analysis {
        waveform: waveform::bins(&decoded.chunk_peaks, WAVEFORM_BINS),
        duration: decoded.duration,
        ..Default::default()
    };

    match waveform::loudness(&decoded) {
        Ok((lufs, true_peak_db)) => {
            analysis.lufs = Some(lufs);
            analysis.true_peak_db = Some(true_peak_db);
        }
        Err(cause) => tracing::debug!(%cause, "no loudness"),
    }

    match features::tempo(&decoded) {
        Ok((bpm, confidence)) => {
            analysis.bpm = Some(bpm);
            analysis.bpm_confidence = Some(confidence);
        }
        Err(cause) => tracing::debug!(%cause, "no tempo"),
    }

    match features::mood(&decoded) {
        Ok(mood) => {
            analysis.valence = Some(mood.valence);
            analysis.arousal = Some(mood.arousal);
            analysis.moods = mood.labels;
        }
        Err(cause) => tracing::debug!(%cause, "no mood"),
    }

    Ok(analysis)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analysed(lufs: Option<f32>, peak: Option<f32>) -> Analysis {
        Analysis {
            lufs,
            true_peak_db: peak,
            ..Default::default()
        }
    }

    /// The gain is the difference to the target, until the peak says otherwise.
    #[test]
    fn the_target_gain_never_lets_a_track_clip() {
        // Quiet master, plenty of headroom: the full correction applies.
        let quiet = analysed(Some(-20.0), Some(-6.0));
        assert_eq!(quiet.gain_for_target(-14.0), Some(5.0));

        // Same loudness, but already peaking near full scale. Turning it up by
        // 6 dB would clip, so it is held to a decibel below the peak.
        let hot = analysed(Some(-20.0), Some(-0.5));
        assert_eq!(hot.gain_for_target(-14.0), Some(-0.5));

        // Turning a loud track *down* is never limited by its peak.
        let loud = analysed(Some(-6.0), Some(-0.1));
        assert_eq!(loud.gain_for_target(-14.0), Some(-8.0));
    }

    #[test]
    fn a_track_with_no_loudness_has_no_gain() {
        assert_eq!(analysed(None, None).gain_for_target(-14.0), None);
        // No peak measurement is not a reason to refuse: it only means the
        // clipping limit cannot be applied.
        assert_eq!(
            analysed(Some(-20.0), None).gain_for_target(-14.0),
            Some(6.0)
        );
    }

    /// A waveform is worth storing on its own, but it cannot pick the next
    /// track — that needs a tempo or a mood.
    #[test]
    fn matching_needs_more_than_a_waveform() {
        let mut analysis = Analysis {
            waveform: vec![7; WAVEFORM_BINS],
            ..Default::default()
        };
        assert!(!analysis.is_useful_for_matching());

        analysis.bpm = Some(128.0);
        assert!(analysis.is_useful_for_matching());

        // Mood alone is enough too, but only when both halves are present:
        // valence without arousal is not a point in the space.
        let mood_only = Analysis {
            valence: Some(0.4),
            ..Default::default()
        };
        assert!(!mood_only.is_useful_for_matching());
        let mood_both = Analysis {
            valence: Some(0.4),
            arousal: Some(0.7),
            ..Default::default()
        };
        assert!(mood_both.is_useful_for_matching());
    }

    #[test]
    fn nothing_is_not_a_track() {
        assert!(analyze(&[], None).is_err());
    }
}
