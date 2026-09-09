//! The drawn waveform, and how loud the track is.

use anyhow::{Error, Result};
use ebur128::{EbuR128, Mode};

use crate::decode::Decoded;

/// Peak envelope resampled to `count` display bars, 0–255.
///
/// Two things happen here, and both are needed for a waveform that looks like
/// the music rather than like a block:
///
/// *Normalising to percentiles, not to the maximum.* One stray peak — a click,
/// a snare hit twice as loud as anything else — would otherwise flatten the
/// whole track against the floor. The 5th and 99th percentiles ignore it.
///
/// *A power curve.* Amplitude is linear but hearing is not, so a straight
/// mapping renders most music as a thin line with occasional spikes. The
/// exponent pulls the quiet middle up to where it can be seen.
pub fn bins(chunk_peaks: &[f32], count: usize) -> Vec<u8> {
    if count == 0 || chunk_peaks.is_empty() {
        return Vec::new();
    }

    // Each bar takes the loudest moment it covers. Averaging would smooth away
    // exactly the transients that make a waveform recognisable.
    let mut peaks = vec![0.0f32; count];
    for (index, peak) in peaks.iter_mut().enumerate() {
        let start = index * chunk_peaks.len() / count;
        let end = ((index + 1) * chunk_peaks.len() / count)
            .max(start + 1)
            .min(chunk_peaks.len());
        *peak = chunk_peaks[start..end]
            .iter()
            .copied()
            .fold(0.0f32, f32::max);
    }

    let mut sorted = peaks.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let low = sorted[(sorted.len() * 5 / 100).min(sorted.len() - 1)];
    let high = sorted[(sorted.len() * 99 / 100).min(sorted.len() - 1)];
    let range = (high - low).max(1e-8);

    peaks
        .into_iter()
        .map(|peak| {
            let scaled = ((peak - low) / range).clamp(0.0, 1.0).powf(0.52);
            // Never quite zero: a silent passage should read as a quiet part of
            // the track, not as a gap where the waveform stopped.
            (8.0 + scaled * 247.0).min(255.0) as u8
        })
        .collect()
}

/// Integrated loudness in LUFS and true peak in dBTP.
///
/// EBU R128, which is what "as loud as" means when comparing two masters — an
/// average of amplitude would call a compressed track and a dynamic one equally
/// loud when one is plainly louder to listen to.
pub fn loudness(decoded: &Decoded) -> Result<(f32, f32)> {
    if decoded.loudness_frames.is_empty() || decoded.sample_rate <= 0.0 {
        return Err(Error::msg("no frames to measure"));
    }

    let mut meter = EbuR128::new(
        decoded.channels,
        decoded.sample_rate as u32,
        Mode::I | Mode::TRUE_PEAK,
    )?;
    meter.add_frames_f32(&decoded.loudness_frames)?;

    let lufs = meter.loudness_global()?;
    // Silence measures as negative infinity, which is true but useless: it
    // would be stored as a number and later used to compute a gain of infinity.
    if !lufs.is_finite() {
        return Err(Error::msg("silent"));
    }

    let mut peak = 0.0f64;
    for channel in 0..decoded.channels {
        peak = peak.max(meter.true_peak(channel).unwrap_or(0.0));
    }
    let peak_db = if peak > 0.0 {
        20.0 * peak.log10()
    } else {
        -120.0
    };

    Ok((lufs as f32, peak_db as f32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_audio_gives_no_bars() {
        assert!(bins(&[], 100).is_empty());
        assert!(bins(&[0.5, 0.5], 0).is_empty());
    }

    /// However many chunks the decode produced, the drawing gets the width it
    /// asked for — the client's layout does not depend on the track's length.
    #[test]
    fn the_bar_count_is_what_was_asked_for() {
        assert_eq!(bins(&[0.1; 10_000], 400).len(), 400);
        // Fewer chunks than bars: a very short track still fills the width.
        assert_eq!(bins(&[0.1, 0.9, 0.3], 400).len(), 400);
    }

    /// A single freak peak must not change how the rest of the track is drawn.
    ///
    /// This is exactly what normalising to the maximum gets wrong: one click
    /// twice as loud as the music squashes the whole waveform into the floor.
    /// Stated as "the drawing is the same with and without it", which is the
    /// property, rather than as a height, which is an implementation detail.
    #[test]
    fn one_stray_peak_does_not_flatten_the_rest() {
        let mut ordinary = vec![0.40f32; 1_000];
        ordinary.extend(vec![0.50f32; 1_000]);

        let mut with_click = ordinary.clone();
        with_click[7] = 1.0;

        let clean = bins(&ordinary, crate::WAVEFORM_BINS);
        let clicked = bins(&with_click, crate::WAVEFORM_BINS);

        // Every bar except the one holding the click is drawn identically.
        let clicked_bar = 7 * crate::WAVEFORM_BINS / ordinary.len();
        for (index, (a, b)) in clean.iter().zip(&clicked).enumerate() {
            if index == clicked_bar {
                continue;
            }
            assert_eq!(a, b, "bar {index} moved because of one distant peak");
        }
    }

    /// Louder must draw taller — the one property a waveform has to have.
    #[test]
    fn louder_passages_are_drawn_taller() {
        let mut peaks = vec![0.1f32; 100];
        peaks.extend(vec![0.9f32; 100]);
        let drawn = bins(&peaks, 20);
        assert!(drawn[0] < drawn[19], "{:?}", drawn);
    }

    /// Silence is still part of the track and gets a floor, not a hole.
    #[test]
    fn silence_is_drawn_as_quiet_rather_than_missing() {
        let drawn = bins(&[0.0; 100], 10);
        assert!(drawn.iter().all(|&bar| bar >= 8), "{drawn:?}");
    }

    #[test]
    fn loudness_needs_frames() {
        let empty = Decoded {
            chunk_peaks: vec![],
            loudness_frames: vec![],
            channels: 2,
            window: vec![],
            sample_rate: 44_100.0,
            duration: 0.0,
        };
        assert!(loudness(&empty).is_err());
    }

    /// Digital silence has no loudness. Storing negative infinity would make
    /// every later gain calculation infinite.
    #[test]
    fn silence_has_no_measurable_loudness() {
        let silent = Decoded {
            chunk_peaks: vec![],
            loudness_frames: vec![0.0; 44_100 * 2 * 4],
            channels: 2,
            window: vec![],
            sample_rate: 44_100.0,
            duration: 4.0,
        };
        assert!(loudness(&silent).is_err());
    }

    /// A real signal measures, and measures sanely: a full-scale sine is far
    /// louder than a quiet one, and neither reads as silence.
    #[test]
    fn a_tone_measures_louder_when_it_is_louder() {
        let tone = |amplitude: f32| {
            let mut frames = Vec::with_capacity(44_100 * 2 * 4);
            for i in 0..44_100 * 4 {
                let sample =
                    amplitude * (i as f32 * 440.0 * std::f32::consts::TAU / 44_100.0).sin();
                frames.push(sample);
                frames.push(sample);
            }
            Decoded {
                chunk_peaks: vec![],
                loudness_frames: frames,
                channels: 2,
                window: vec![],
                sample_rate: 44_100.0,
                duration: 4.0,
            }
        };

        let (loud_lufs, loud_peak) = loudness(&tone(0.9)).unwrap();
        let (quiet_lufs, quiet_peak) = loudness(&tone(0.09)).unwrap();

        assert!(loud_lufs > quiet_lufs, "{loud_lufs} vs {quiet_lufs}");
        // Ten times the amplitude is 20 dB, give or take the meter's filtering.
        assert!((loud_lufs - quiet_lufs - 20.0).abs() < 1.0);
        // A 0.9 peak is just under full scale; the quiet one is 20 dB down.
        assert!(loud_peak > -2.0 && loud_peak < 0.5, "{loud_peak}");
        assert!(quiet_peak < loud_peak);
    }
}
