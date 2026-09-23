//! Acoustic fingerprints — what a recording *is*, rather than what it sounds
//! like.
//!
//! The rest of this crate measures a track: how loud, how fast, how bright.
//! A fingerprint answers a different question. It is a compact summary of the
//! audio itself, robust to the encoder, the bitrate and the volume it was
//! ripped at, so two files of the same recording produce fingerprints that
//! match even when nothing about their tags or their bytes does. That is what
//! makes it an identity: a badly tagged file can be looked up by its sound.
//!
//! The algorithm is [Chromaprint], the one AcoustID indexes, computed here by
//! a pure-Rust port — no C library to find, build or ship, and the same
//! answer on every platform we build for. What comes out is byte-for-byte what
//! `fpcalc` would produce, which is the whole point: a fingerprint nobody else
//! can read is not an identity, it is a checksum.
//!
//! [Chromaprint]: https://acoustid.org/chromaprint

use anyhow::{Error, Result};
use base64::Engine;
use rusty_chromaprint::{Configuration, FingerprintCompressor, Fingerprinter};
use std::io::Cursor;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// How much of a track is fingerprinted.
///
/// Two minutes, which is what `fpcalc` uses by default and therefore what the
/// AcoustID index was built from. Fingerprinting more would not be *wrong* —
/// a longer fingerprint still matches a shorter one over the part they share —
/// but it would decode the rest of every track to no purpose, and this pass
/// runs over a whole library.
pub const FINGERPRINT_SECONDS: f32 = 120.0;

/// A track's acoustic identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fingerprint {
    /// The compressed fingerprint, base64url with no padding — exactly the
    /// encoding AcoustID's `fingerprint` parameter expects, and exactly what
    /// `fpcalc` prints.
    pub fingerprint: String,
    /// The length of the **whole** track in seconds, not of the fingerprinted
    /// part. AcoustID matches on duration as well as audio, so telling it two
    /// minutes for a six-minute track would lose the match.
    pub duration: u32,
}

/// Fingerprint audio already in memory.
///
/// `extension_hint` helps the prober when the bytes came from a url with no
/// content type; it is a hint only, and a wrong one costs nothing.
pub fn fingerprint(bytes: &[u8], extension_hint: Option<&str>) -> Result<Fingerprint> {
    if bytes.is_empty() {
        return Err(Error::msg("no audio to fingerprint"));
    }
    let stream = MediaSourceStream::new(Box::new(Cursor::new(bytes.to_vec())), Default::default());
    compress(compute(stream, extension_hint)?)
}

/// Fingerprint a file on disk, without reading it into memory first.
///
/// The path this pass actually uses. Only the first two minutes are decoded
/// and the rest is walked packet-header by packet-header, so a library of
/// lossless files is fingerprinted without ever holding one — which is what
/// makes it safe to run several at once.
pub fn fingerprint_file(path: impl AsRef<std::path::Path>) -> Result<Fingerprint> {
    let path = path.as_ref();
    let file = std::fs::File::open(path)?;
    let extension = path.extension().and_then(|e| e.to_str());
    let stream = MediaSourceStream::new(Box::new(file), Default::default());
    compress(compute(stream, extension)?)
}

fn compress((raw, duration): (Vec<u32>, u32)) -> Result<Fingerprint> {
    let compressed = FingerprintCompressor::from(&*CONFIG).compress(&raw);
    Ok(Fingerprint {
        fingerprint: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(compressed),
        duration,
    })
}

/// Chromaprint's default algorithm — the one AcoustID indexes. Built once:
/// the preset computes a filter bank and a set of coefficients, and every
/// track in the library would otherwise rebuild the same ones.
static CONFIG: std::sync::LazyLock<Configuration> =
    std::sync::LazyLock::new(Configuration::preset_test2);

/// The sub-fingerprints themselves, and the track's length in seconds.
fn compute(stream: MediaSourceStream, extension_hint: Option<&str>) -> Result<(Vec<u32>, u32)> {
    let mut hint = Hint::new();
    if let Some(extension) = extension_hint {
        hint.with_extension(extension);
    }

    let probed = symphonia::default::get_probe().format(
        &hint,
        stream,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;
    let mut format = probed.format;

    let track = format
        .default_track()
        .ok_or_else(|| Error::msg("the file has no audio track"))?;
    let track_id = track.id;
    let params = track.codec_params.clone();
    // The container's own frame count, when it has one. Preferred over
    // counting packets: a lossy stream's last packet is padded, so summing
    // packet durations overshoots by a fraction of a second, and a duration
    // that disagrees with everyone else's by a second is a lookup that misses.
    let claimed_frames = params.n_frames;
    let mut decoder = symphonia::default::get_codecs().make(&params, &DecoderOptions::default())?;

    let mut printer = Fingerprinter::new(&CONFIG);
    let mut started = false;

    let mut sample_rate = params.sample_rate.unwrap_or(0);
    let mut frames_seen: u64 = 0;
    // Filled in once the sample rate is known, which for some codecs is not
    // until the first decoded packet.
    let mut frame_limit = u64::MAX;
    let mut buffer: Option<SampleBuffer<i16>> = None;

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            // The end of the stream, and also what a truncated download looks
            // like. Either way what has been read so far is usable.
            Err(SymphoniaError::IoError(_)) => break,
            Err(SymphoniaError::ResetRequired) => break,
            Err(cause) => return Err(cause.into()),
        };
        if packet.track_id() != track_id {
            continue;
        }

        // Past the fingerprinted window there is still a duration to finish
        // measuring, and a packet says how long it is without being decoded.
        // Skipping the decode is what keeps a ten-minute track from costing
        // ten minutes of audio to fingerprint two of them.
        if frames_seen >= frame_limit {
            frames_seen += packet.dur();
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
        if sample_rate == 0 {
            sample_rate = spec.rate;
        }
        if !started {
            printer
                .start(spec.rate, spec.channels.count() as u32)
                .map_err(|cause| Error::msg(format!("cannot fingerprint this audio: {cause}")))?;
            frame_limit = (FINGERPRINT_SECONDS * spec.rate as f32) as u64;
            started = true;
        }

        let buffer =
            buffer.get_or_insert_with(|| SampleBuffer::<i16>::new(audio.capacity() as u64, spec));
        buffer.copy_interleaved_ref(audio);
        let samples = buffer.samples();
        // Chromaprint wants interleaved 16-bit samples at the source rate and
        // does its own mixdown and resampling to 11025 Hz.
        printer.consume(samples);
        frames_seen += (samples.len() / spec.channels.count().max(1)) as u64;
    }

    if !started || sample_rate == 0 {
        return Err(Error::msg("nothing decoded"));
    }

    printer.finish();
    let raw = printer.fingerprint();
    if raw.is_empty() {
        // Chromaprint needs a few seconds of audio before it produces a single
        // sub-fingerprint. A shorter file — a gap track, a sound effect — has
        // no acoustic identity to give, and an empty string is not one.
        return Err(Error::msg("too short to fingerprint"));
    }

    let frames = claimed_frames.unwrap_or(frames_seen);
    Ok((
        raw.to_vec(),
        // Rounded, not truncated: AcoustID compares durations as whole
        // seconds and a track is as likely to be just over as just under.
        ((frames as f64 / sample_rate as f64).round() as i64).max(0) as u32,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 16-bit mono wav of a tone sweep, which is enough audio for
    /// Chromaprint to have something to say about it.
    fn wav(seconds: u32, start_hz: f32) -> Vec<u8> {
        const RATE: u32 = 44_100;
        let frames = RATE * seconds;
        let mut samples = Vec::with_capacity(frames as usize * 2);
        let mut phase = 0.0f32;
        for frame in 0..frames {
            let progress = frame as f32 / frames as f32;
            phase += std::f32::consts::TAU * (start_hz + progress * 600.0) / RATE as f32;
            let value = (phase.sin() * 12_000.0) as i16;
            samples.extend_from_slice(&value.to_le_bytes());
        }

        let mut out = Vec::with_capacity(44 + samples.len());
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(36 + samples.len() as u32).to_le_bytes());
        out.extend_from_slice(b"WAVEfmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes()); // pcm
        out.extend_from_slice(&1u16.to_le_bytes()); // mono
        out.extend_from_slice(&RATE.to_le_bytes());
        out.extend_from_slice(&(RATE * 2).to_le_bytes());
        out.extend_from_slice(&2u16.to_le_bytes());
        out.extend_from_slice(&16u16.to_le_bytes());
        out.extend_from_slice(b"data");
        out.extend_from_slice(&(samples.len() as u32).to_le_bytes());
        out.extend_from_slice(&samples);
        out
    }

    /// The point of a fingerprint: the same audio is the same identity, every
    /// time and on every machine. A fingerprint that drifted between runs
    /// would match nothing, including itself.
    #[test]
    fn the_same_audio_fingerprints_the_same() {
        let audio = wav(20, 440.0);
        let first = fingerprint(&audio, Some("wav")).unwrap();
        let second = fingerprint(&audio, Some("wav")).unwrap();
        assert_eq!(first, second);
        assert!(!first.fingerprint.is_empty());
        assert_eq!(first.duration, 20);
    }

    /// And different audio is a different identity.
    #[test]
    fn different_audio_fingerprints_differently() {
        let one = fingerprint(&wav(20, 440.0), Some("wav")).unwrap();
        let other = fingerprint(&wav(20, 110.0), Some("wav")).unwrap();
        assert_ne!(one.fingerprint, other.fingerprint);
    }

    /// The duration reported is the whole track's, not the fingerprinted
    /// window's — AcoustID matches on it, and saying 120 for a longer track
    /// is how a good fingerprint loses its match.
    #[test]
    fn the_duration_is_the_whole_track() {
        let long = (FINGERPRINT_SECONDS + 30.0) as u32;
        let fingerprinted = fingerprint(&wav(long, 220.0), Some("wav")).unwrap();
        assert_eq!(fingerprinted.duration, long);
    }

    /// Too short to have an acoustic identity is an error, not an empty one.
    #[test]
    fn a_fraction_of_a_second_has_no_identity() {
        assert!(fingerprint(&wav(0, 440.0), Some("wav")).is_err());
    }

    #[test]
    fn empty_bytes_are_not_audio() {
        assert!(fingerprint(&[], None).is_err());
    }

    /// What an html error page from a server looks like: an error, not a panic.
    #[test]
    fn html_is_not_audio() {
        assert!(fingerprint(b"<html><body>404</body></html>", Some("mp3")).is_err());
    }
}
