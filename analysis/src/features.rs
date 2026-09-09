//! Tempo and mood, from the decoded window.

use anyhow::{Error, Result};
use oximedia_mir::{key::KeyDetector, mood::MoodDetector, tempo::TempoDetector, MirConfig};

use crate::decode::Decoded;

/// The shortest window worth running a detector on.
///
/// Tempo is found by autocorrelation, so it needs several bars to correlate;
/// under a few seconds the answer is noise dressed up as a number.
const MINIMUM_SECONDS: f32 = 5.0;

/// Beats per minute, with a confidence.
///
/// Bounded to the range music is actually written in. Without bounds the
/// detector happily returns half or double the real tempo, which is worse than
/// no answer: a matcher would put a drum-and-bass track next to a ballad on the
/// grounds that 174 is close to 87 doubled.
pub fn tempo(decoded: &Decoded) -> Result<(f32, f32)> {
    let window = usable_window(decoded)?;
    let config = MirConfig::default();
    let detector = TempoDetector::new(decoded.sample_rate, config.min_tempo, config.max_tempo);
    let result = detector
        .detect(window)
        .map_err(|cause| Error::msg(format!("tempo: {cause}")))?;

    if !result.bpm.is_finite() || result.bpm <= 0.0 {
        return Err(Error::msg("tempo: no usable estimate"));
    }
    Ok((result.bpm, result.confidence.clamp(0.0, 1.0)))
}

/// Where a track sits emotionally.
pub struct Mood {
    /// -1 dark to 1 bright.
    pub valence: f32,
    /// 0 calm to 1 driving.
    pub arousal: f32,
    /// Named moods with confidences, strongest first.
    pub labels: Vec<(String, f32)>,
}

pub fn mood(decoded: &Decoded) -> Result<Mood> {
    let window = usable_window(decoded)?;
    let detector = MoodDetector::new(decoded.sample_rate);
    let result = detector
        .detect(window)
        .map_err(|cause| Error::msg(format!("mood: {cause}")))?;

    if !result.valence.is_finite() || !result.arousal.is_finite() {
        return Err(Error::msg("mood: no usable estimate"));
    }

    // The detector returns a map, whose order is arbitrary. Sorting makes the
    // stored value stable for the same audio, and puts the label a client would
    // show first at the front. Ties break by name for the same reason.
    let mut labels: Vec<(String, f32)> = result
        .moods
        .iter()
        .map(|(name, score)| (name.clone(), *score))
        .collect();
    labels.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    labels.truncate(4);

    Ok(Mood {
        valence: result.valence.clamp(-1.0, 1.0),
        arousal: result.arousal.clamp(0.0, 1.0),
        labels,
    })
}

/// The musical key, in traditional notation.
pub fn key(decoded: &Decoded) -> Result<(String, f32)> {
    let window = usable_window(decoded)?;
    // A chromagram needs a long window to resolve pitch classes; 4096 is the
    // detector's own default and a reasonable balance at any sample rate.
    let detector = KeyDetector::new(decoded.sample_rate, 4096);
    let result = detector
        .detect(window)
        .map_err(|cause| Error::msg(format!("key: {cause}")))?;

    let key = crate::key::Key {
        root: result.root % 12,
        is_major: result.is_major,
    };
    Ok((key.name(), result.confidence.clamp(0.0, 1.0)))
}

/// The window, if there is enough of it to mean anything.
fn usable_window(decoded: &Decoded) -> Result<&[f32]> {
    if decoded.sample_rate <= 0.0 {
        return Err(Error::msg("no sample rate"));
    }
    let seconds = decoded.window.len() as f32 / decoded.sample_rate;
    if seconds < MINIMUM_SECONDS {
        return Err(Error::msg(format!(
            "only {seconds:.1}s of audio, too little to analyse"
        )));
    }
    Ok(&decoded.window)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decoded(window: Vec<f32>, sample_rate: f32) -> Decoded {
        Decoded {
            chunk_peaks: vec![],
            loudness_frames: vec![],
            channels: 1,
            window,
            sample_rate,
            duration: 0.0,
            tags: Default::default(),
        }
    }

    /// Too short to hear a tempo in is an error, not a number. A bpm from two
    /// seconds of audio would be stored and later trusted.
    #[test]
    fn a_window_too_short_to_analyse_is_refused() {
        let short = decoded(vec![0.0; 44_100 * 2], 44_100.0);
        assert!(usable_window(&short).is_err());
        assert!(tempo(&short).is_err());
        assert!(mood(&short).is_err());

        let enough = decoded(vec![0.0; 44_100 * 10], 44_100.0);
        assert!(usable_window(&enough).is_ok());
    }

    #[test]
    fn a_missing_sample_rate_is_refused() {
        assert!(usable_window(&decoded(vec![0.0; 500_000], 0.0)).is_err());
    }

    /// A steady pulse has a tempo, and the detector should find roughly the one
    /// it was given. This is the whole feature working end to end on synthetic
    /// audio, with no file needed.
    #[test]
    fn a_steady_pulse_reads_as_its_own_tempo() {
        let rate = 22_050.0;
        let bpm = 120.0;
        let samples_per_beat = (rate * 60.0 / bpm) as usize;
        let mut window = vec![0.0f32; (rate * 20.0) as usize];
        for beat in 0..(window.len() / samples_per_beat) {
            // A short decaying click on each beat.
            for i in 0..(rate * 0.03) as usize {
                let at = beat * samples_per_beat + i;
                if at < window.len() {
                    let decay = 1.0 - (i as f32 / (rate * 0.03));
                    window[at] = decay * (i as f32 * 0.35).sin();
                }
            }
        }

        let (detected, confidence) = tempo(&decoded(window, rate)).expect("a pulse has a tempo");
        // Allowing the octave error every tempo detector makes: what must not
        // happen is an unrelated number.
        let plausible = [bpm, bpm / 2.0, bpm * 2.0]
            .iter()
            .any(|candidate| (detected - candidate).abs() < 6.0);
        assert!(plausible, "detected {detected} bpm, expected around {bpm}");
        assert!((0.0..=1.0).contains(&confidence));
    }
}
