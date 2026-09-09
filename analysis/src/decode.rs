//! Decoding audio to mono samples.
//!
//! One pass over the file produces everything the analysis needs. Splitting it
//! into a pass per feature would decode the same track three times — and
//! decoding is nearly all of the cost, the feature extraction that follows is
//! cheap by comparison.

use anyhow::{Error, Result};
use std::io::Cursor;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// What one decode pass yields.
pub struct Decoded {
    /// Peak amplitude per short chunk, across the whole track. Resampled into
    /// display bins later — kept at this resolution here because the number of
    /// bins wanted is a presentation decision, and re-deciding it must not mean
    /// decoding the track again.
    pub chunk_peaks: Vec<f32>,
    /// Interleaved samples, for the loudness meter, which needs channels.
    pub loudness_frames: Vec<f32>,
    pub channels: u32,
    /// Mono samples from a window in the middle of the track, for tempo and
    /// mood. A window, not the whole track: those detectors are quadratic-ish
    /// in input length and a minute is more than enough to hear a tempo.
    pub window: Vec<f32>,
    pub sample_rate: f32,
    /// Seconds decoded. Zero when the container lied and nothing came out.
    pub duration: f32,
    /// Key and tempo as the file states them, when it does.
    pub tags: crate::tags::Tags,
}

/// How much audio the tempo and mood detectors get.
const WINDOW_SECONDS: f32 = 60.0;

/// How far in the window starts, when the track is long enough to allow it.
///
/// Intros are unrepresentative — a slow build, an a cappella opening, a spoken
/// sample — and a tempo read from one describes a part of the track nobody
/// would use to characterise it.
const WINDOW_SKIP_SECONDS: f32 = 30.0;

/// Chunks per second of the peak envelope. Twenty is finer than any waveform is
/// ever drawn, so the bins can be resampled down to whatever a client wants.
const CHUNKS_PER_SECOND: f32 = 20.0;

/// Decode audio bytes to everything the analysis needs.
///
/// `extension_hint` helps the prober when the bytes came from a url with no
/// content type; it is a hint only, and a wrong one costs nothing.
pub fn decode(bytes: &[u8], extension_hint: Option<&str>) -> Result<Decoded> {
    if bytes.is_empty() {
        return Err(Error::msg("no audio to analyse"));
    }

    let mut hint = Hint::new();
    if let Some(extension) = extension_hint {
        hint.with_extension(extension);
    }

    let stream = MediaSourceStream::new(Box::new(Cursor::new(bytes.to_vec())), Default::default());
    let mut probed = symphonia::default::get_probe().format(
        &hint,
        stream,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;
    // Read before the audio: a tag is a better answer than a re-derivation,
    // and the probe has already parsed the container to find it.
    let mut tags = probed
        .metadata
        .get()
        .and_then(|mut m| m.skip_to_latest().map(crate::tags::read))
        .unwrap_or_default();
    let mut format = probed.format;
    // Some containers carry metadata on the format reader rather than the
    // probe, so both are consulted before giving up on a tag.
    if tags.key.is_none() || tags.bpm.is_none() {
        if let Some(revision) = format.metadata().skip_to_latest() {
            let extra = crate::tags::read(revision);
            tags.key = tags.key.or(extra.key);
            tags.bpm = tags.bpm.or(extra.bpm);
        }
    }

    let track = format
        .default_track()
        .ok_or_else(|| Error::msg("the file has no audio track"))?;
    let track_id = track.id;
    // The container's own idea of how long the track is, when it has one. Only
    // used to place the analysis window, so a wrong answer costs nothing worse
    // than a window in the wrong part of the song.
    let claimed_duration = match (track.codec_params.n_frames, track.codec_params.sample_rate) {
        (Some(frames), Some(rate)) if rate > 0 => Some(frames as f32 / rate as f32),
        _ => None,
    };
    let mut decoder =
        symphonia::default::get_codecs().make(&track.codec_params, &DecoderOptions::default())?;

    // Not known until the first decoded packet for some codecs, so these start
    // empty and are filled in on the way.
    let mut sample_rate = track.codec_params.sample_rate.unwrap_or(0) as f32;
    let mut channels = track.codec_params.channels.map_or(0, |c| c.count()) as u32;

    let mut decoded = Decoded {
        chunk_peaks: Vec::new(),
        loudness_frames: Vec::new(),
        channels,
        window: Vec::new(),
        sample_rate,
        duration: 0.0,
        tags,
    };

    let mut buffer: Option<SampleBuffer<f32>> = None;
    let mut frames_seen: u64 = 0;
    let mut chunk_peak = 0.0f32;
    let mut chunk_len: u64 = 0;

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            // The end of the stream, and also what a truncated download looks
            // like. Either way what has been decoded so far is usable.
            Err(SymphoniaError::IoError(_)) => break,
            Err(SymphoniaError::ResetRequired) => break,
            Err(cause) => return Err(cause.into()),
        };
        if packet.track_id() != track_id {
            continue;
        }

        let audio = match decoder.decode(&packet) {
            Ok(audio) => audio,
            // A corrupt packet in the middle of a file is not a reason to
            // abandon the track; skipping it loses a few milliseconds.
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(SymphoniaError::IoError(_)) => break,
            Err(cause) => return Err(cause.into()),
        };

        let spec = *audio.spec();
        if sample_rate == 0.0 {
            sample_rate = spec.rate as f32;
        }
        if channels == 0 {
            channels = spec.channels.count() as u32;
        }

        let buffer =
            buffer.get_or_insert_with(|| SampleBuffer::<f32>::new(audio.capacity() as u64, spec));
        buffer.copy_interleaved_ref(audio);
        let samples = buffer.samples();
        let channel_count = spec.channels.count().max(1);

        // The loudness meter wants channels, so it gets the interleaved frames
        // untouched. Everything else works on the mono mixdown below.
        decoded.loudness_frames.extend_from_slice(samples);

        let chunk_frames = (sample_rate / CHUNKS_PER_SECOND).max(1.0) as u64;
        let window_start = window_start_frame(&claimed_duration, sample_rate);
        let window_end = window_start + (WINDOW_SECONDS * sample_rate) as u64;

        for frame in samples.chunks(channel_count) {
            let mono = frame.iter().sum::<f32>() / channel_count as f32;

            let magnitude = mono.abs();
            if magnitude > chunk_peak {
                chunk_peak = magnitude;
            }
            chunk_len += 1;
            if chunk_len >= chunk_frames {
                decoded.chunk_peaks.push(chunk_peak);
                chunk_peak = 0.0;
                chunk_len = 0;
            }

            if frames_seen >= window_start && frames_seen < window_end {
                decoded.window.push(mono);
            }
            frames_seen += 1;
        }
    }

    // Whatever was accumulating when the audio ended is still a chunk.
    if chunk_len > 0 {
        decoded.chunk_peaks.push(chunk_peak);
    }

    if frames_seen == 0 || sample_rate == 0.0 {
        return Err(Error::msg("nothing decoded"));
    }

    decoded.sample_rate = sample_rate;
    decoded.channels = channels.max(1);
    decoded.duration = frames_seen as f32 / sample_rate;

    Ok(decoded)
}

/// The frame the analysis window starts at.
///
/// Skips the intro when there is enough track to skip it and still get a full
/// window; on anything shorter it starts at the beginning, because half a
/// window from the middle is worse than a whole one from the start.
fn window_start_frame(duration_seconds: &Option<f32>, sample_rate: f32) -> u64 {
    let Some(duration) = duration_seconds else {
        return (WINDOW_SKIP_SECONDS * sample_rate) as u64;
    };
    if *duration < WINDOW_SKIP_SECONDS + WINDOW_SECONDS {
        return 0;
    }
    (WINDOW_SKIP_SECONDS * sample_rate) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A short track has no middle to skip to. Starting 30 seconds into a
    /// two-minute song would analyse a quarter of it and miss the rest.
    #[test]
    fn the_window_only_skips_an_intro_when_there_is_room() {
        let rate = 44_100.0;
        // Long enough for a skip and a full window.
        assert_eq!(
            window_start_frame(&Some(300.0), rate),
            (WINDOW_SKIP_SECONDS * rate) as u64
        );
        // Exactly enough, to the second.
        assert_eq!(window_start_frame(&Some(90.0), rate), (30.0 * rate) as u64);
        // Too short: start at the beginning rather than take a partial window.
        assert_eq!(window_start_frame(&Some(89.0), rate), 0);
        assert_eq!(window_start_frame(&Some(10.0), rate), 0);
    }

    /// An unknown duration is the common case for a stream, and must not mean
    /// no window at all.
    #[test]
    fn an_unknown_duration_still_gets_a_window() {
        assert_eq!(
            window_start_frame(&None, 44_100.0),
            (WINDOW_SKIP_SECONDS * 44_100.0) as u64
        );
    }

    #[test]
    fn empty_bytes_are_not_audio() {
        assert!(decode(&[], None).is_err());
    }

    /// Bytes that are not audio at all must be an error, not a panic — this is
    /// what an html error page from a server looks like.
    #[test]
    fn html_is_not_audio() {
        assert!(decode(b"<html><body>404</body></html>", Some("mp3")).is_err());
    }
}
