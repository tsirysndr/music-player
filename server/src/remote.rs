//! Rocksky remote-player protocol for the daemon.
//!
//! Registers as a playback device on Rocksky's remote-control WebSocket
//! (mirrors rocksky's `playerd`), so this daemon shows up in the web/desktop
//! miniplayer device picker. Controllers get transport control (play, pause,
//! next, previous, seek, shuffle, repeat, volume) plus the audio-settings
//! surface our engine implements (EQ, tone, ReplayGain, crossfade, dither).
//!
//! Publishes queue metadata and supports jump/remove in the local queue.
//! Rocksky-library enqueue and queue moves report an explicit unsupported error.
//! Waveform analysis runs separately from the status/transport loop.
//!
//! Requires the access token written by `rocksky login` to
//! `~/.rocksky/token.json`; without it the remote player stays off. It can
//! also be disabled with `remote_player = false` in settings.toml.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use music_player_playback::player::PlayerCommand;
use music_player_settings::{read_settings, Settings};
use music_player_tracklist::Tracklist;
use rocksky_sdk::{
    RemoteCommand, RemoteNowPlaying, RemotePlayer, RemotePlayerConfig, RemoteQueueItem,
    RemoteRepeat, RemoteStatus,
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{info, warn};

fn token_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/"))
        .join(".rocksky")
        .join("token.json")
}

/// The access token written by `rocksky login`, if any.
fn read_token() -> Option<String> {
    let raw = std::fs::read_to_string(token_path()).ok()?;
    let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
    value
        .get("token")
        .and_then(|t| t.as_str())
        .filter(|t| !t.trim().is_empty())
        .map(String::from)
}

/// Start the remote player if it is enabled and a Rocksky token is available.
pub fn spawn(
    tracklist: Arc<Mutex<Tracklist>>,
    cmd_tx: Arc<Mutex<UnboundedSender<PlayerCommand>>>,
    providers: Arc<music_player_provider::ProviderState>,
) {
    let enabled = read_settings()
        .ok()
        .and_then(|config| config.get_bool("remote_player").ok())
        .unwrap_or(true);
    if !enabled {
        return;
    }
    let Some(token) = read_token() else {
        info!(
            "Rocksky remote player disabled: no token at {} (run `rocksky login`)",
            token_path().display()
        );
        return;
    };
    let name = read_settings()
        .ok()
        .and_then(|config| config.try_deserialize::<Settings>().ok())
        .map(|settings| settings.device_name)
        .unwrap_or_else(|| "Music Player".to_string());
    let ws_url = std::env::var("ROCKSKY_WS")
        .unwrap_or_else(|_| rocksky_sdk::remote_player::DEFAULT_REMOTE_WS.to_string());

    tokio::spawn(async move {
        info!("registering \"{name}\" on {ws_url}");
        let remote = Arc::new(RemotePlayer::connect(
            RemotePlayerConfig::new(token, name).url(ws_url),
        ));
        tokio::spawn(status_loop(remote.clone(), tracklist.clone(), providers));
        command_loop(remote, cmd_tx, tracklist).await;
    });
}

async fn command_loop(
    remote: Arc<RemotePlayer>,
    cmd_tx: Arc<Mutex<UnboundedSender<PlayerCommand>>>,
    tracklist: Arc<Mutex<Tracklist>>,
) {
    let send = |cmd: PlayerCommand| {
        if let Err(e) = cmd_tx.lock().unwrap().send(cmd) {
            warn!("remote command dropped: {e}");
        }
    };
    while let Some(cmd) = remote.next_command().await {
        match cmd {
            RemoteCommand::Play => send(PlayerCommand::Play),
            RemoteCommand::Pause => send(PlayerCommand::Pause),
            RemoteCommand::Next => send(PlayerCommand::Next),
            RemoteCommand::Previous => send(PlayerCommand::Previous),
            RemoteCommand::Seek { position_ms } => {
                send(PlayerCommand::Seek(position_ms.min(u32::MAX as u64) as u32))
            }
            RemoteCommand::SetShuffle { enabled } => send(PlayerCommand::SetShuffle(enabled)),
            RemoteCommand::SetRepeat { mode } => send(PlayerCommand::SetRepeat(match mode {
                RemoteRepeat::All => 1,
                RemoteRepeat::One => 2,
                _ => 0,
            })),
            RemoteCommand::SetVolume { volume } => {
                send(PlayerCommand::SetVolume(
                    (volume.clamp(0.0, 1.0) * 100.0).round() as u16,
                ));
            }
            RemoteCommand::SetAudioSettings(audio) => apply_audio(&send, &audio),
            RemoteCommand::QueueJump { index } | RemoteCommand::QueueRemove { index } => {
                let valid = {
                    let queue = tracklist.lock().unwrap();
                    let (played, upcoming) = queue.tracks();
                    (index as usize) < played.len() + upcoming.len()
                };
                if valid {
                    send(if matches!(cmd, RemoteCommand::QueueJump { .. }) {
                        PlayerCommand::PlayTrackAt(index as usize)
                    } else {
                        PlayerCommand::RemoveTrack(index as usize)
                    });
                } else {
                    remote.command_error("Queue index is out of range".into());
                }
            }
            RemoteCommand::Enqueue { .. } | RemoteCommand::QueueMove { .. } => {
                remote.command_error("Music Player does not support this queue operation".into());
            }
        }
    }
    info!("remote command loop ended");
}

/// Apply the sections of a partial audio-settings document our engine
/// implements (EQ, tone, ReplayGain, crossfade); the rest (crossfeed,
/// compressor, surround, PBE) is not read.
fn apply_audio(
    send: &impl Fn(PlayerCommand),
    audio: &rocksky_sdk::remote_audio::RemoteAudioSettings,
) {
    if let Some(eq) = &audio.equalizer {
        if let Some(enabled) = eq.enabled {
            send(PlayerCommand::SetEqEnabled(enabled));
        }
        if let Some(precut) = eq.precut {
            // Wire precut is ≤ 0 tenths-of-dB of attenuation; the engine
            // takes positive dB of headroom.
            send(PlayerCommand::SetEqPrecut((-precut).max(0) as f32 / 10.0));
        }
        if let Some(bands) = &eq.bands {
            for (i, band) in bands.iter().enumerate().take(10) {
                send(PlayerCommand::SetEqBandGain {
                    band: i,
                    gain_db: band.gain as f32 / 10.0,
                });
            }
        }
    }
    if let Some(tone) = &audio.tone {
        if let Some(bass) = tone.bass {
            send(PlayerCommand::SetBass(bass));
        }
        if let Some(treble) = tone.treble {
            send(PlayerCommand::SetTreble(treble));
        }
        if let Some(balance) = tone.balance {
            send(PlayerCommand::SetBalance(balance));
        }
    }
    if let Some(rg) = &audio.replay_gain {
        let mode = match rg.mode.as_deref() {
            Some("track") => 0,
            Some("album") => 1,
            Some("trackIfShuffling") => 2,
            _ => 3,
        };
        send(PlayerCommand::SetReplaygain {
            mode,
            preamp_db: rg.preamp.unwrap_or(0) as f32 / 10.0,
            prevent_clipping: rg.prevent_clipping.unwrap_or(false),
        });
    }
    if let Some(cf) = &audio.crossfade {
        let mode = match cf.mode.as_deref() {
            Some("enabled") => 5,
            Some("shuffle") => 3,
            Some("trackChange") => 2,
            Some("albumChange") => 1,
            _ => 0,
        };
        send(PlayerCommand::SetCrossfade {
            mode,
            fade_in_delay: cf.fade_in_delay.unwrap_or(0) / 1000,
            fade_in_duration: cf.fade_in_duration.unwrap_or(2000) / 1000,
            fade_out_delay: cf.fade_out_delay.unwrap_or(0) / 1000,
            fade_out_duration: cf.fade_out_duration.unwrap_or(2000) / 1000,
            mix_mode: if cf.fade_out_mix_mode.as_deref() == Some("mix") {
                2
            } else {
                0
            },
        });
    }
}

/// Codec name for the format badge, from the file extension.
///
/// Lowercased, and only for URIs that look like files — an HTTP stream's
/// extension (or lack of one) says nothing about what is inside it.
fn codec_of(uri: &str) -> Option<String> {
    let path = uri.strip_prefix("file://").unwrap_or(uri);
    if path.starts_with("http://") || path.starts_with("https://") {
        return None;
    }
    std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
}

/// Push now-playing + transport state to controllers every couple of
/// seconds so this daemon stays live in the miniplayer device picker.
/// No queue push — see the module docs.
type Track = music_player_entity::track::Model;

/// Peak-preserving resampling. Even a narrow transient must survive 400 -> 256.
fn waveform_peaks(peaks: &[u8]) -> Option<Vec<u8>> {
    if peaks.is_empty() {
        return None;
    }
    Some(
        (0..256)
            .map(|i| {
                let start = i * peaks.len() / 256;
                let end = ((i + 1) * peaks.len()).div_ceil(256).min(peaks.len());
                peaks[start..end.max(start + 1)]
                    .iter()
                    .copied()
                    .max()
                    .unwrap_or(0)
            })
            .collect(),
    )
}

fn repeat(mode: i32) -> RemoteRepeat {
    match mode {
        1 => RemoteRepeat::All,
        2 => RemoteRepeat::One,
        _ => RemoteRepeat::Off,
    }
}

fn duration_ms(seconds: Option<f32>) -> u64 {
    seconds
        .filter(|s| s.is_finite() && *s > 0.0)
        .map(|s| (s as f64 * 1000.0) as u64)
        .unwrap_or(0)
}

// Only advertise artwork that a controller can actually open. Local paths are
// not URLs on the controller; the enrichment worker resolves local covers.
fn artwork(track: &Track) -> String {
    track
        .album
        .cover
        .as_deref()
        .filter(|s| {
            s.starts_with("https://") || s.starts_with("http://") || s.starts_with("data:image/")
        })
        .unwrap_or_default()
        .to_owned()
}

fn queue_item(track: &Track) -> RemoteQueueItem {
    RemoteQueueItem {
        title: track.title.clone(),
        artist: track.artist.clone(),
        album: track.album.title.clone(),
        album_artist: track.album.artist.clone(),
        album_art: artwork(track),
        duration_ms: duration_ms(track.duration),
        song_uri: track.aturi.clone().unwrap_or_default(),
        album_uri: track.album.aturi.clone().unwrap_or_default(),
        track_number: track.track.unwrap_or(0).min(i32::MAX as u32) as i32,
        // Local paths, provider credentials and non-Rocksky IDs are not valid
        // Rocksky enqueue identities. Positional queue commands still work.
        ..Default::default()
    }
}

fn now_playing(track: Option<&Track>, queue: &Tracklist) -> RemoteNowPlaying {
    let state = queue.playback_state();
    let mut np = RemoteNowPlaying {
        elapsed_ms: if track.is_some() {
            state.position_ms as u64
        } else {
            0
        },
        is_playing: track.is_some() && state.is_playing,
        shuffle: Some(queue.shuffle_enabled()),
        repeat: Some(repeat(queue.repeat_mode())),
        volume: queue.volume(),
        waveform_status: Some("unavailable".into()),
        ..Default::default()
    };
    if let Some(track) = track {
        np.title = track.title.clone();
        np.artist = track.artist.clone();
        np.album = track.album.title.clone();
        np.album_artist = track.album.artist.clone();
        np.album_art = artwork(track);
        np.duration_ms = duration_ms(track.duration);
        np.codec = queue.codec().or_else(|| codec_of(&track.uri));
        np.sample_rate = track.sample_rate.filter(|n| *n > 0);
        np.bitrate = track.bitrate.filter(|n| *n > 0);
    }
    np
}

#[derive(Clone, Default)]
struct Enrichment {
    sample_rate: Option<u32>,
    bitrate: Option<u32>,
    waveform: Option<Vec<u8>>,
    album_art: String,
}

async fn local_artwork(track: &Track) -> String {
    use tokio::io::AsyncReadExt;
    let Some(name) = track.album.cover.as_deref() else {
        return String::new();
    };
    // Scanner covers are single filenames. Never interpret a provider path as
    // an arbitrary file on the daemon, or send a controller an unusable file URL.
    if name.is_empty() || name.contains('/') || name.contains('\\') || name == ".." {
        return String::new();
    }
    let path = std::path::Path::new(&music_player_settings::get_application_directory())
        .join("covers")
        .join(name);
    let Ok(file) = tokio::fs::File::open(path).await else {
        return String::new();
    };
    let mut bytes = Vec::new();
    if file
        .take(8 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .await
        .is_err()
        || bytes.len() > 8 * 1024 * 1024
    {
        return String::new();
    }
    tokio::task::spawn_blocking(move || thumbnail(&bytes))
        .await
        .unwrap_or_default()
}

fn thumbnail(bytes: &[u8]) -> String {
    use base64::Engine as _;
    let Ok(mut reader) = image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format()
    else {
        return String::new();
    };
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(64 * 1024 * 1024);
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    reader.limits(limits);
    let Ok(image) = reader.decode() else {
        return String::new();
    };
    let image = image.thumbnail(256, 256).to_rgb8();
    let mut jpeg = Vec::new();
    if image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 80)
        .encode_image(&image)
        .is_err()
    {
        return String::new();
    }
    format!(
        "data:image/jpeg;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(jpeg)
    )
}

async fn enrich(
    track: Track,
    source: String,
    progress: tokio::sync::watch::Sender<Enrichment>,
) -> Enrichment {
    use music_player_storage::track_analysis::{self, TrackRef};
    let db = music_player_storage::shared().await;
    let conn = db.get_connection();
    let mut result = Enrichment::default();
    result.album_art = local_artwork(&track).await;
    if source.is_empty() {
        use sea_orm::EntityTrait;
        if let Ok(Some(row)) = music_player_entity::track::Entity::find_by_id(&track.id)
            .one(conn)
            .await
        {
            result.sample_rate = row.sample_rate.filter(|n| *n > 0);
            result.bitrate = row.bitrate.filter(|n| *n > 0);
        }
    }
    let _ = progress.send(result.clone());
    if duration_ms(track.duration) == 0 || track.id.starts_with("radio:") {
        return result;
    }
    let reference = TrackRef {
        id: track.id,
        uri: track.uri,
        artist: track.artist,
        title: track.title,
    };
    if let Ok(Ok(analysis)) = tokio::time::timeout(
        Duration::from_secs(120),
        track_analysis::ensure(conn, &source, &reference),
    )
    .await
    {
        result.waveform = waveform_peaks(&analysis.waveform);
    }
    result
}

async fn status_loop(
    remote: Arc<RemotePlayer>,
    tracklist: Arc<Mutex<Tracklist>>,
    providers: Arc<music_player_provider::ProviderState>,
) {
    let mut key = None;
    let mut work: Option<tokio::task::JoinHandle<Enrichment>> = None;
    let mut enriched = None;
    let mut partial: Option<tokio::sync::watch::Receiver<Enrichment>> = None;
    let mut last_queue = String::new();
    loop {
        let (track, mut np, mut items, index) = {
            let queue = tracklist.lock().unwrap();
            let (track, played_count) = queue.current_track();
            let (played, upcoming) = queue.tracks();
            let items = played
                .iter()
                .chain(upcoming.iter())
                .map(queue_item)
                .collect::<Vec<_>>();
            let np = now_playing(track.as_ref(), &queue);
            (track, np, items, played_count.saturating_sub(1) as u32)
        };
        let source = if let Some(track) = &track {
            if track.uri.starts_with("http://") || track.uri.starts_with("https://") {
                providers
                    .config()
                    .await
                    .filter(|c| {
                        track
                            .uri
                            .starts_with(&format!("{}/", c.url.trim_end_matches('/')))
                    })
                    .map(|c| c.id)
                    .unwrap_or_else(|| format!("remote:{:x}", md5::compute(&track.uri)))
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        let next_key = track.as_ref().map(|t| {
            (
                source.clone(),
                t.id.clone(),
                t.uri.clone(),
                duration_ms(t.duration) > 0,
            )
        });
        if next_key != key {
            if let Some(task) = work.take() {
                task.abort();
            }
            enriched = None;
            partial = None;
            key = next_key;
            if let Some(track) = &track {
                let (tx, rx) = tokio::sync::watch::channel(Enrichment::default());
                partial = Some(rx);
                work = Some(tokio::spawn(enrich(track.clone(), source, tx)));
            }
        }
        if work.as_ref().is_some_and(|task| task.is_finished()) {
            enriched = work.take().unwrap().await.ok();
        }
        let data = enriched
            .clone()
            .or_else(|| partial.as_ref().map(|rx| rx.borrow().clone()));
        if let Some(data) = &data {
            if np.album_art.is_empty() {
                np.album_art = data.album_art.clone();
            }
            np.sample_rate = np.sample_rate.or(data.sample_rate);
            np.bitrate = np.bitrate.or(data.bitrate);
            np.waveform = data.waveform.clone();
            if np.waveform.is_some() {
                np.waveform_status = Some("ready".into());
            }
        }
        if np.waveform.is_none()
            && work.is_some()
            && np.duration_ms > 0
            && !track.as_ref().is_some_and(|t| t.id.starts_with("radio:"))
        {
            np.waveform_status = Some("pending".into());
        }
        if let Some(item) = items.get_mut(index as usize) {
            item.album_art = np.album_art.clone();
        }
        // RemoteQueueItem has no Serialize; compare the full metadata snapshot.
        let signature = format!("{index}:{items:?}");
        if signature != last_queue {
            remote.set_queue(items, index);
            last_queue = signature;
        }
        let status = if track.is_none() {
            RemoteStatus::Stopped
        } else if np.is_playing {
            RemoteStatus::Playing
        } else {
            RemoteStatus::Paused
        };
        remote.set_now_playing(np);
        remote.set_status(status);
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn websocket_payload_carries_waveform_metadata_queue_and_track_reset() {
        use futures_util::{SinkExt, StreamExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}", listener.local_addr().unwrap());
        let player = RemotePlayer::connect(RemotePlayerConfig::new("test-only", "Test").url(url));
        let mut track = Track {
            title: "First".into(),
            artist: "Artist".into(),
            uri: "/song.flac".into(),
            duration: Some(100.0),
            sample_rate: Some(48000),
            bitrate: Some(800),
            ..Default::default()
        };
        track.album.title = "Album".into();
        track.album.artist = "Album artist".into();
        track.album.cover = Some("https://example.test/cover.jpg".into());
        let mut queue = Tracklist::new_empty();
        queue.set_volume(0.5);
        queue.set_modes(true, 1);
        let mut np = now_playing(Some(&track), &queue);
        np.waveform = waveform_peaks(&vec![42; 400]);
        np.waveform_status = Some("ready".into());
        player.set_now_playing(np);
        player.set_queue(vec![queue_item(&track)], 0);
        player.set_now_playing(now_playing(None, &queue));
        let observed = tokio::time::timeout(Duration::from_secs(5), async {
            let (stream, _) = listener.accept().await.unwrap();
            let mut socket = tokio_tungstenite::accept_async(stream).await.unwrap();
            let mut frames = Vec::new();
            while frames.len() < 3 {
                let message = socket.next().await.unwrap().unwrap();
                if let tokio_tungstenite::tungstenite::Message::Ping(bytes) = message {
                    socket
                        .send(tokio_tungstenite::tungstenite::Message::Pong(bytes))
                        .await
                        .unwrap();
                    continue;
                }
                let Ok(text) = message.to_text() else {
                    continue;
                };
                let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
                    continue;
                };
                if value["type"] == "message" {
                    frames.push(value["data"].clone());
                }
            }
            frames
        })
        .await
        .unwrap();
        player.disconnect();
        let first = &observed[0];
        assert_eq!(first["waveform"], serde_json::json!(vec![42; 256]));
        assert_eq!(first["waveform_status"], "ready");
        assert_eq!(first["codec"], "flac");
        assert_eq!(first["sample_rate"], 48000);
        assert_eq!(first["bitrate"], 800);
        assert_eq!(first["volume"], 0.5);
        assert_eq!(first["shuffle"], true);
        assert_eq!(first["repeat"], "all");
        assert_eq!(first["album_art"], "https://example.test/cover.jpg");
        assert_eq!(observed[1]["queue"][0]["album_artist"], "Album artist");
        assert_eq!(observed[1]["index"], 0);
        assert_eq!(observed[2]["title"], "");
        assert!(observed[2].get("waveform").is_none());
        assert_eq!(observed[2]["waveform_status"], "unavailable");
    }

    #[test]
    fn artwork_is_portable_and_bounded() {
        assert!(thumbnail(b"not an image").is_empty());
        let image = image::RgbImage::from_pixel(512, 512, image::Rgb([20, 40, 60]));
        let mut bytes = Vec::new();
        image::codecs::jpeg::JpegEncoder::new(&mut bytes)
            .encode_image(&image)
            .unwrap();
        let art = thumbnail(&bytes);
        assert!(art.starts_with("data:image/jpeg;base64,"));
        assert!(art.len() < 128 * 1024);
    }

    #[test]
    fn queue_current_index_matches_played_and_upcoming_order() {
        let tracks = (0..3)
            .map(|n| Track {
                id: n.to_string(),
                title: n.to_string(),
                ..Default::default()
            })
            .collect();
        let mut queue = Tracklist::new(tracks);
        queue.next_track();
        queue.next_track();
        let (current, count) = queue.current_track();
        let (played, upcoming) = queue.tracks();
        let items: Vec<_> = played
            .iter()
            .chain(upcoming.iter())
            .map(queue_item)
            .collect();
        assert_eq!(items.len(), 3);
        assert_eq!(items[count - 1].title, current.unwrap().title);
    }

    #[test]
    fn waveform_resampling_preserves_silence_transients_and_endpoints() {
        assert!(waveform_peaks(&[]).is_none());
        assert_eq!(waveform_peaks(&[0; 400]), Some(vec![0; 256]));
        assert_eq!(waveform_peaks(&[83]), Some(vec![83; 256]));
        for at in 0..400 {
            let mut input = vec![0; 400];
            input[at] = 255;
            let out = waveform_peaks(&input).unwrap();
            assert_eq!(out.len(), 256);
            assert!(out.contains(&255));
        }
    }
    #[test]
    fn publishes_metadata_and_real_transport_and_clears_on_stop() {
        let mut track = Track {
            title: "Song".into(),
            artist: "Artist".into(),
            uri: "/music/song.flac".into(),
            duration: Some(123.5),
            bitrate: Some(900),
            sample_rate: Some(96000),
            track: Some(7),
            aturi: Some("at://did/song/1".into()),
            ..Default::default()
        };
        track.album.title = "Album".into();
        track.album.artist = "Album artist".into();
        track.album.cover = Some("https://example.test/art.jpg".into());
        let mut queue = Tracklist::new_empty();
        queue.set_modes(true, 2);
        queue.set_volume(0.42);
        let np = now_playing(Some(&track), &queue);
        assert_eq!(np.bitrate, Some(900));
        assert_eq!(np.sample_rate, Some(96000));
        assert_eq!(np.shuffle, Some(true));
        assert!(matches!(np.repeat, Some(RemoteRepeat::One)));
        assert_eq!(np.volume, Some(0.42));
        assert_eq!(np.duration_ms, 123500);
        assert_eq!(np.album_art, "https://example.test/art.jpg");
        let item = queue_item(&track);
        assert_eq!(item.track_number, 7);
        assert_eq!(item.song_uri, "at://did/song/1");
        assert!(item.url.is_empty());
        assert!(item.track_id.is_empty());
        let stopped = now_playing(None, &queue);
        assert!(stopped.title.is_empty());
        assert!(stopped.waveform.is_none());
        assert!(!stopped.is_playing);
        assert_eq!(stopped.waveform_status.as_deref(), Some("unavailable"));
    }
}
