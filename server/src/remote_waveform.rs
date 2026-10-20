// RMS waveform worker shared in behavior with Rocksky playerd.
//! On-demand waveform analysis on a separate worker, never the playback decoder.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex, OnceLock,
    },
    time::{Duration, Instant},
};
use symphonia::core::{
    audio::SampleBuffer, codecs::DecoderOptions, errors::Error, io::MediaSourceStream, probe::Hint,
};

const BINS: usize = 256;
const MAX_BYTES: u64 = 256 * 1024 * 1024;
static GENERATION: AtomicU64 = AtomicU64::new(0);
static WORKER: Mutex<()> = Mutex::new(());
static STATE: OnceLock<Mutex<(String, Value)>> = OnceLock::new();

/// Returns immediately. Repeated calls poll the same job; a new source cancels
/// obsolete analysis. Only one worker decodes/downloads at a time.
pub fn request(source: &str, directory: &Path) -> Value {
    let state = STATE.get_or_init(|| Mutex::new((String::new(), json!({"status":"idle"}))));
    let mut state = state.lock().unwrap();
    if source.is_empty() {
        GENERATION.fetch_add(1, Ordering::Relaxed);
        *state = (String::new(), json!({"status":"idle"}));
        return state.1.clone();
    }
    let key = cache_key(source);
    if state.0 == key {
        return state.1.clone();
    }
    let generation = GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
    *state = (key.clone(), json!({"status":"pending"}));
    let source = source.to_owned();
    let directory = directory.to_owned();
    std::thread::spawn(move || {
        let _worker = WORKER.lock().unwrap();
        if GENERATION.load(Ordering::Relaxed) != generation {
            return;
        }
        let result = generate(&source, &directory, &key, generation);
        let mut state = STATE.get().unwrap().lock().unwrap();
        if state.0 == key && GENERATION.load(Ordering::Relaxed) == generation {
            state.1 = match result {
                Ok(peaks) => json!({"status":"ready","peaks":peaks}),
                Err(_) => json!({"status":"unavailable"}),
            };
        }
    });
    state.1.clone()
}

fn cache_key(source: &str) -> String {
    let stamp = fs::metadata(source)
        .ok()
        .map(|m| format!("{}:{:?}", m.len(), m.modified().ok()))
        .unwrap_or_default();
    format!(
        "{:x}",
        Sha256::digest(format!("waveform-v2-rms:{source}:{stamp}"))
    )
}

fn generate(source: &str, directory: &Path, key: &str, generation: u64) -> Result<Vec<u8>, String> {
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let cache = directory.join(format!("{key}.json"));
    if let Ok(bytes) = fs::read(&cache) {
        if let Ok(peaks) = serde_json::from_slice::<Vec<u8>>(&bytes) {
            if peaks.len() == BINS
                && fs::metadata(&cache)
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.elapsed().ok())
                    .is_some_and(|age| age < Duration::from_secs(86400 * 7))
            {
                return Ok(peaks);
            }
        }
    }
    let started = Instant::now();
    let check = || {
        if GENERATION.load(Ordering::Relaxed) != generation
            || started.elapsed() > Duration::from_secs(120)
        {
            Err("Analysis cancelled".to_string())
        } else {
            Ok(())
        }
    };
    let mut temporary = None;
    let path = if source.starts_with("http://") || source.starts_with("https://") {
        let client = reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(90))
            .build()
            .map_err(|e| e.to_string())?;
        let mut response = client
            .get(source)
            .send()
            .and_then(|r| r.error_for_status())
            .map_err(|e| e.to_string())?;
        let length = response
            .content_length()
            .filter(|n| *n > 0 && *n <= MAX_BYTES)
            .ok_or("Not a bounded audio file")?;
        let mut file = tempfile::NamedTempFile::new_in(directory).map_err(|e| e.to_string())?;
        let mut buffer = [0u8; 64 * 1024];
        let mut total = 0;
        loop {
            check()?;
            let n = response.read(&mut buffer).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            total += n as u64;
            if total > length || total > MAX_BYTES {
                return Err("Audio too large".into());
            }
            file.write_all(&buffer[..n]).map_err(|e| e.to_string())?;
        }
        if total != length {
            return Err("Incomplete audio download".into());
        }
        let path = file.path().to_owned();
        temporary = Some(file);
        path
    } else {
        Path::new(source.strip_prefix("file://").unwrap_or(source)).to_owned()
    };
    let file = fs::File::open(&path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > MAX_BYTES {
        return Err("Audio too large".into());
    }
    let peaks = decode(file, check)?;
    drop(temporary);
    // Bound the small derived cache as well as temporary audio downloads.
    if let Ok(entries) = fs::read_dir(directory) {
        let mut files: Vec<_> = entries
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
            .collect();
        files.sort_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()));
        let excess = files.len().saturating_sub(127);
        for entry in files.into_iter().take(excess) {
            let _ = fs::remove_file(entry.path());
        }
    }
    fs::write(cache, serde_json::to_vec(&peaks).unwrap()).map_err(|e| e.to_string())?;
    Ok(peaks)
}

fn decode(file: fs::File, check: impl Fn() -> Result<(), String>) -> Result<Vec<u8>, String> {
    let source = MediaSourceStream::new(Box::new(file), Default::default());
    let mut format = symphonia::default::get_probe()
        .format(
            &Hint::new(),
            source,
            &Default::default(),
            &Default::default(),
        )
        .map_err(|e| e.to_string())?
        .format;
    let track = format.default_track().ok_or("No audio track")?;
    let id = track.id;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| e.to_string())?;
    let mut envelope = Vec::<Energy>::new();
    let mut time = 0.0;
    loop {
        check()?;
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.to_string()),
        };
        if packet.track_id() != id {
            continue;
        }
        let decoded = decoder.decode(&packet).map_err(|e| e.to_string())?;
        let spec = *decoded.spec();
        let channels = spec.channels.count();
        if spec.rate == 0 || channels == 0 {
            return Err("Invalid audio format".into());
        }
        let mut samples = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        samples.copy_interleaved_ref(decoded);
        for frame in samples.samples().chunks(channels) {
            let bin = (time * 20.0) as usize;
            if bin >= 144_000 {
                return Err("Audio longer than two hours".into());
            }
            if envelope.len() <= bin {
                envelope.resize(bin + 1, Energy::default());
            }
            for sample in frame {
                if sample.is_finite() {
                    envelope[bin].squares += f64::from(*sample).powi(2);
                    envelope[bin].samples += 1;
                }
            }
            time += 1.0 / spec.rate as f64;
        }
    }
    if envelope.is_empty() {
        return Err("No audio decoded".into());
    }
    Ok(bins(&envelope))
}

// Accumulate energy rather than maxima: a short drum transient must not make
// the whole surrounding second look full-scale. Counts also weight partial
// windows correctly. Fixed full-scale calibration preserves dynamics without
// stretching every track's loudest passage to 255.
#[derive(Clone, Copy, Default)]
struct Energy {
    squares: f64,
    samples: u64,
}

fn bins(envelope: &[Energy]) -> Vec<u8> {
    (0..BINS)
        .map(|i| {
            let start = i * envelope.len() / BINS;
            let end = ((i + 1) * envelope.len() / BINS)
                .max(start + 1)
                .min(envelope.len());
            let energy = envelope[start..end]
                .iter()
                .fold(Energy::default(), |mut sum, e| {
                    sum.squares += e.squares;
                    sum.samples += e.samples;
                    sum
                });
            let rms = (energy.squares / energy.samples.max(1) as f64).sqrt();
            (rms.clamp(0.0, 1.0) * 255.0).round() as u8
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decodes_audio_into_time_aligned_waveform() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        let frames = 32_000_u32;
        let bytes = frames * 2;
        file.write_all(b"RIFF").unwrap();
        file.write_all(&(36 + bytes).to_le_bytes()).unwrap();
        file.write_all(b"WAVEfmt ").unwrap();
        file.write_all(&16_u32.to_le_bytes()).unwrap();
        file.write_all(&1_u16.to_le_bytes()).unwrap();
        file.write_all(&1_u16.to_le_bytes()).unwrap();
        file.write_all(&8_000_u32.to_le_bytes()).unwrap();
        file.write_all(&16_000_u32.to_le_bytes()).unwrap();
        file.write_all(&2_u16.to_le_bytes()).unwrap();
        file.write_all(&16_u16.to_le_bytes()).unwrap();
        file.write_all(b"data").unwrap();
        file.write_all(&bytes.to_le_bytes()).unwrap();
        for i in 0..frames {
            // Silence, sparse transients, denser transients, then sustained
            // audio. All non-silent sections have exactly the same peak.
            let audible = match i / 8000 {
                0 => false,
                1 => i % 100 == 0,
                2 => i % 100 < 25,
                _ => true,
            };
            let sample: i16 = if !audible {
                0
            } else if i % 2 == 0 {
                16000
            } else {
                -16000
            };
            file.write_all(&sample.to_le_bytes()).unwrap();
        }
        let peaks = decode(fs::File::open(file.path()).unwrap(), || Ok(())).unwrap();
        let directory = tempfile::tempdir().unwrap();
        let source = file.path().to_str().unwrap();
        let key = cache_key(source);
        assert_eq!(generate(source, directory.path(), &key, 0).unwrap(), peaks);
        assert_eq!(
            generate("/missing", directory.path(), &key, 0).unwrap(),
            peaks
        );
        request(source, directory.path());
        let deadline = Instant::now() + Duration::from_secs(5);
        while request(source, directory.path())["status"] == "pending" {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(request(source, directory.path())["peaks"], json!(peaks));
        assert_eq!(request("", directory.path())["status"], "idle");
        assert_eq!(peaks.len(), BINS);
        assert!(peaks[..60].iter().all(|&p| p == 0));
        assert!(peaks[70..120].iter().all(|&p| (11..=14).contains(&p)));
        assert!(peaks[140..185].iter().all(|&p| (61..=64).contains(&p)));
        assert!(peaks[205..].iter().all(|&p| p == 125));
        assert!(decode(fs::File::open(file.path()).unwrap(), || Err(
            "cancelled".into()
        ))
        .is_err());
    }

    #[test]
    fn equal_peaks_with_different_energy_produce_different_heights() {
        // Every window has a full-scale peak, but the sustained energy differs.
        // The old max-based algorithm produced 255 for all three passages.
        let mut envelope = vec![Energy::default(); BINS];
        envelope[80] = Energy {
            squares: 1.0,
            samples: 100,
        };
        envelope[120] = Energy {
            squares: 25.0,
            samples: 100,
        };
        envelope[190] = Energy {
            squares: 100.0,
            samples: 100,
        };
        let levels = bins(&envelope);
        assert_eq!(levels[0], 0);
        assert_eq!(levels[80], 26);
        assert_eq!(levels[120], 128);
        assert_eq!(levels[190], 255);
        assert_eq!(bins(&[Energy::default()]), vec![0; BINS]);
    }

    #[test]
    fn quiet_tracks_are_not_stretched_to_full_height() {
        let levels = bins(&vec![
            Energy {
                squares: 1.0,
                samples: 100
            };
            BINS
        ]);
        assert!(levels.iter().all(|&level| level == 26));
    }

    #[test]
    fn downsampling_weights_partial_windows_by_sample_count() {
        let mut envelope = vec![
            Energy {
                squares: 0.0,
                samples: 99
            };
            BINS * 2
        ];
        for i in (0..envelope.len()).step_by(2) {
            envelope[i] = Energy {
                squares: 1.0,
                samples: 1,
            };
        }
        assert!(bins(&envelope).iter().all(|&level| level == 26));
    }
}
