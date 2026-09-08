//! gRPC worker: talks to the music-player daemon, pushes state into the
//! Slint UI. music-player has no server-streaming RPCs, so now-playing and
//! queue state are polled (1 s) instead of followed.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::{LazyLock, RwLock as StdRwLock};
use std::time::Duration;

use slint::Weak;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;
use tonic::transport::{Channel, Endpoint};

use crate::AppWindow;

pub use music_player_server::api::metadata::v1alpha1::Track as TrackProto;
use music_player_server::api::music::v1alpha1::{
    library_service_client::LibraryServiceClient, mixer_service_client::MixerServiceClient,
    playback_service_client::PlaybackServiceClient, playlist_service_client::PlaylistServiceClient,
    tracklist_service_client::TracklistServiceClient, AddItemRequest, AddTrackRequest,
    ClearTracklistRequest, CreateRequest, DeleteRequest, FindAllRequest, GetAlbumDetailsRequest,
    GetAlbumsRequest, GetArtistsRequest, GetAudioSettingsRequest, GetCurrentlyPlayingSongRequest,
    GetLikedTracksRequest, GetPlaylistDetailsRequest, GetTracklistTracksRequest, GetTracksRequest,
    GetVolumeRequest, LikeTrackRequest, LoadTracksRequest, NextRequest, PauseRequest,
    PlayNextRequest, PlayRequest, PlayTrackAtRequest, PreviewSmartPlaylistRequest, PreviousRequest,
    RemoveItemRequest, RemoveTrackAtRequest, RenameRequest, SeekRequest, SetAudioSettingRequest,
    SetEqBandGainRequest, SetMuteRequest, SetRepeatRequest, SetVolumeRequest, ShuffleRequest,
    SmartPlaylist as SmartPlaylistProto,
};

use crate::likes;
use crate::SavedServer;
use music_player_entity::saved_radio;
use sea_orm::EntityTrait;

/// All albums/artists/tracks in one page — a limit of 0 means "none" on the
/// server side, so ask for effectively-everything instead.
const PAGE: i32 = 100_000;

// ── Commands from UI callbacks ──────────────────────────────────────────────

#[derive(Debug)]
pub enum Cmd {
    Play,
    Pause,
    Next,
    Previous,
    SeekMs(u32),
    SetVolume(f32),
    SetMute(bool),
    PlayAlbum(String),
    PlayAlbumAt(String, i32),
    PlayArtist(String),
    PlayArtistShuffled(String),
    /// Artist id and a position within that artist's track list.
    PlayArtistAt(String, i32),
    PlayAllAt(i32),
    PlayLikedAt(i32),
    PlayLikedShuffled,
    QueueJump(i32),
    QueueClear,
    QueueRemove(i32),
    OpenAlbum(String),
    PlayAlbumShuffled(String),
    SetShuffle(bool),
    SetRepeat(i32),
    AudioSet(String, i32),
    EqBandSet(usize, i32),
    ConnectServer(SavedServer),
    /// Read the library from the daemon's own files again.
    DisconnectProvider,
    /// Fetch the saved servers from the daemon.
    LoadServers,
    AddServer {
        kind: String,
        name: String,
        url: String,
        username: String,
        password: String,
    },
    DeleteServer(String),
    /// Fetch the places the audio could come out.
    LoadRenderers,
    /// id, is-cast. An empty id means "play here".
    ActivateRenderer(String, bool),
    OpenPlaylist(String),
    /// Create a smart playlist, or convert nothing — the form only offers this
    /// when creating.
    SmartPlaylistCreate {
        name: String,
        rsql: String,
        sort_by: String,
        sort_order: String,
        limit: u32,
    },
    /// Count what a filter would match, for the form's live preview.
    SmartPlaylistPreview {
        rsql: String,
        sort_by: String,
        limit: u32,
    },
    PlaylistCreate {
        name: String,
        description: String,
    },
    PlaylistUpdate {
        id: String,
        name: String,
        description: String,
    },
    PlaylistDelete(String),
    PlaylistAddTrack {
        playlist_id: String,
        track_id: String,
    },
    PlaylistRemoveTrack {
        playlist_id: String,
        track_id: String,
    },
    PlaySavedPlaylist(String),
    /// Same, in a random order.
    ShufflePlaylist(String),
    /// Rockbox insert positions: -2 play next, -3 add last. `tracks` are ids.
    InsertTracks {
        position: i32,
        tracks: Vec<String>,
    },
    InsertAlbum {
        album_id: String,
        position: i32,
    },
    LikeTrack {
        id: String,
        like: bool,
    },
    LikeAlbum(String),
    SwitchServer {
        host: String,
        grpc_port: u16,
    },
    DiscoverServers,
    /// Re-read the extension directories, pruning flags for anything gone.
    ExtensionsRescan,
    /// Switch an extension on or off.
    ExtensionSetEnabled {
        id: String,
        enabled: bool,
    },
    RadioSearch(String),
    RadioBrowse(String),
    RadioBookmarks,
    RadioPlay(String),
    RadioBookmark(String),
    /// The user's own stations.
    RadioStations,
    /// Check the stream url in the add-station form.
    RadioCheckStream(String),
    RadioAddStation(crate::radio::NewStation),
}

// ── Plain data handed to the UI thread ──────────────────────────────────────

#[derive(Clone, Debug)]
pub struct AlbumData {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub year: String,
    pub art_file: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ArtistData {
    pub id: String,
    pub name: String,
    /// Artist image URL (Rocksky), empty when unknown.
    pub picture: String,
}

#[derive(Clone, Debug)]
pub struct StationData {
    pub id: String,
    pub name: String,
    pub subtitle: String,
    pub source: String,
    pub logo: String,
    pub bookmarked: bool,
}

#[derive(Clone, Debug)]
pub struct TrackData {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub length_ms: u64,
    pub index: i32,
    pub disc: i32,
    pub track_no: i32,
    pub album_id: String,
}

#[derive(Clone, Debug)]
pub struct PlaylistData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub track_count: i64,
}

#[derive(Clone, Debug, Default)]
pub struct LibraryData {
    pub albums: Vec<AlbumData>,
    pub artists: Vec<ArtistData>,
    pub tracks: Vec<TrackData>,
    pub liked: Vec<TrackData>,
}

#[derive(Clone, Debug)]
pub struct AlbumDetailData {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub year: String,
    pub label: String,
    pub tracks: Vec<TrackData>,
}

/// Firmware-unit snapshot of the daemon's audio settings for the UI.
#[derive(Clone, Debug, Default)]
pub struct AudioSettingsData {
    pub eq_enabled: bool,
    pub eq_precut: i32,
    pub eq_bands: Vec<(i32, i32, i32)>, // (cutoff Hz, Q x10, gain dB x10)
    pub bass: i32,
    pub bass_min: i32,
    pub bass_max: i32,
    pub treble: i32,
    pub treble_min: i32,
    pub treble_max: i32,
    pub balance: i32,
    pub rg_type: i32,
    pub rg_preamp: i32,
    pub rg_noclip: bool,
    pub crossfade: i32,
    pub fade_in_delay: i32,
    pub fade_in_duration: i32,
    pub fade_out_delay: i32,
    pub fade_out_duration: i32,
    pub fade_out_mixmode: i32,
    pub dithering: bool,
}

pub struct Endpoints {
    pub covers: String,
    pub display: String,
    /// The daemon's GraphQL endpoint. Saved servers and the current provider
    /// live there, not over gRPC.
    pub graphql: String,
}

/// The daemon the app is currently pointed at. Starts from the env overrides
/// (falling back to settings.toml) and can be switched at runtime via
/// `switch_server` (server switcher UI).
static TARGET: LazyLock<StdRwLock<(String, u16)>> = LazyLock::new(|| {
    let settings = crate::daemon::settings();
    let host = std::env::var("MUSIC_PLAYER_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let grpc = std::env::var("MUSIC_PLAYER_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(settings.port);
    StdRwLock::new((host, grpc))
});

static CHAN: LazyLock<StdRwLock<Channel>> = LazyLock::new(|| StdRwLock::new(make_channel()));

/// Bumped on every server switch; polling loops compare generations and
/// reconnect when it moved.
static SWITCH_GEN: AtomicU64 = AtomicU64::new(0);

fn make_channel() -> Channel {
    let (host, grpc) = TARGET.read().unwrap().clone();
    Endpoint::from_shared(format!("http://{host}:{grpc}"))
        .expect("grpc url")
        .connect_timeout(Duration::from_secs(3))
        .connect_lazy()
}

/// Current channel — cheap to clone; loops fetch a fresh one per iteration so
/// a server switch takes effect everywhere.
fn chan() -> Channel {
    CHAN.read().unwrap().clone()
}

/// Repoints the app at another music-player daemon.
pub fn switch_server(host: &str, grpc_port: u16) {
    *TARGET.write().unwrap() = (host.to_string(), grpc_port);
    *CHAN.write().unwrap() = make_channel();
    SWITCH_GEN.fetch_add(1, Ordering::SeqCst);
}

pub fn endpoints() -> Endpoints {
    let (host, grpc_port) = TARGET.read().unwrap().clone();
    // Default layout is 5051 gRPC / 5053 http, so http = grpc + 2.
    let http_port = std::env::var("MUSIC_PLAYER_HTTP_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(grpc_port + 2);
    Endpoints {
        covers: format!("http://{host}:{http_port}/covers/"),
        display: format!("{host}:{grpc_port}"),
        graphql: format!("http://{host}:{http_port}/graphql"),
    }
}

pub fn format_time(secs: f64) -> String {
    let s = secs.max(0.0) as u64;
    format!("{:02}:{:02}", s / 60, s % 60)
}

/// Full track record kept on the worker so albums/artists/likes/inserts can
/// be resolved without re-fetching (LoadTracks needs uri + album).
struct FullTrack {
    proto: TrackProto,
    album_id: String,
    artist: String,
}

#[derive(Default)]
struct WorkerState {
    tracks: Vec<FullTrack>,
    liked: HashSet<String>,
    playing: bool,
    /// Album and playlist detail responses, keyed by id.
    ///
    /// Navigating back to one it has already fetched should not re-ask the
    /// server — that is a visible pause against a remote library, and the
    /// answer does not change between two clicks. Dropped on a server switch,
    /// where every id belongs to a different library.
    /// The connected provider's own liked tracks, when there is one.
    ///
    /// A provider's likes are its own — Subsonic's stars, Jellyfin's
    /// favourites — and their ids mean nothing to the local store, so they
    /// cannot be derived by filtering the cached library. `None` means the
    /// local library, where they can.
    remote_liked: Option<Vec<TrackProto>>,
    album_cache: HashMap<String, Vec<TrackProto>>,
    playlist_cache: HashMap<String, (Vec<TrackProto>, u32)>,
    radios: HashMap<String, crate::radio::Station>,
}

/// Spawns the background runtime; returns immediately.
pub fn start(weak: Weak<AppWindow>, rx: UnboundedReceiver<Cmd>) {
    std::thread::Builder::new()
        .name("music-player-rpc".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("tokio runtime");
            rt.block_on(run(weak, rx));
        })
        .expect("spawn rpc thread");
}

async fn run(weak: Weak<AppWindow>, rx: UnboundedReceiver<Cmd>) {
    // Likes restored from the user's atproto repo join the locally-stored ones,
    // so a fresh install shows the account's likes once the library is scanned.
    let mut liked = likes::load();
    let db = music_player_storage::shared().await;
    match music_player_storage::rocksky_likes::matched_track_ids(db.get_connection()).await {
        Ok(ids) => liked.extend(ids),
        Err(e) => tracing::debug!("could not read restored likes: {e}"),
    }
    let state = Arc::new(Mutex::new(WorkerState {
        liked,
        ..WorkerState::default()
    }));

    tokio::spawn(cmd_loop(rx, state.clone(), weak.clone()));
    tokio::spawn(ticker(weak.clone()));

    // Main loop: connect → init volume/audio → load library → poll status.
    // Each pass fetches the CURRENT channel, so a server switch reconnects.
    loop {
        let channel = chan();
        let ep = endpoints();
        match session(&channel, &ep, &weak, &state).await {
            Ok(()) => {}
            Err(e) => tracing::debug!("session ended: {e}"),
        }
        let display = endpoints().display;
        let _ = weak.upgrade_in_event_loop(move |app| {
            app.set_connected(false);
            app.set_status_text(format!("{display} (retrying…)").into());
        });
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

async fn session(
    channel: &Channel,
    ep: &Endpoints,
    weak: &Weak<AppWindow>,
    state: &Arc<Mutex<WorkerState>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let session_gen = SWITCH_GEN.load(Ordering::SeqCst);
    let mut playback = PlaybackServiceClient::new(channel.clone());

    // Probe with a unary call so we only report connected when it works.
    playback
        .get_currently_playing_song(GetCurrentlyPlayingSongRequest {})
        .await?;
    let display = ep.display.clone();
    let _ = weak.upgrade_in_event_loop(move |app| {
        app.set_connected(true);
        app.set_status_text(display.into());
    });

    // Whatever the daemon is already reading from — a restart must not claim
    // the local library when a provider is connected.
    load_connected_provider(weak).await;
    let _ = load_servers(weak).await;
    init_volume(channel, weak).await;
    init_audio_settings(channel, weak).await;
    load_library(channel, ep, weak, state).await?;
    load_playlists(channel, weak).await;

    // Poll now-playing + queue until the daemon (or the target) goes away.
    let mut tracklist = TracklistServiceClient::new(channel.clone());
    let mut last_art: Option<String> = None;
    let mut format_cache: HashMap<String, (u32, u32)> = HashMap::new();
    loop {
        if SWITCH_GEN.load(Ordering::SeqCst) != session_gen {
            return Err("server switched".into());
        }
        let now = playback
            .get_currently_playing_song(GetCurrentlyPlayingSongRequest {})
            .await?
            .into_inner();

        let playing = now.is_playing;
        state.lock().await.playing = playing;

        let (title, artist, album, path, length_ms, art_file, track_id) = match &now.track {
            Some(t) => (
                if t.title.is_empty() {
                    filename_stem(&t.uri)
                } else {
                    t.title.clone()
                },
                t.artist.clone(),
                t.album
                    .as_ref()
                    .map(|a| a.title.clone())
                    .unwrap_or_default(),
                t.uri.clone(),
                (t.duration * 1000.0) as u64,
                t.album.as_ref().and_then(|a| {
                    if a.cover.is_empty() {
                        None
                    } else {
                        Some(a.cover.clone())
                    }
                }),
                t.id.clone(),
            ),
            None => (
                "Nothing playing".to_string(),
                String::new(),
                String::new(),
                String::new(),
                0,
                None,
                String::new(),
            ),
        };
        // VFD readout: codec (from the uri extension), bitrate and sample
        // rate — "FLAC 986 kbps 44.1 kHz". The queue position is prepended once
        // the queue snapshot below is in (compact units keep it on one line).
        let format_info = match &now.track {
            Some(t) => {
                let codec = codec_of(&t.uri);
                let (probed_bitrate, probed_sample_rate) = if t.bitrate == 0 || t.sample_rate == 0 {
                    if let Some(values) = format_cache.get(&t.id) {
                        *values
                    } else {
                        let values = probe_audio_format(&t.uri).await.unwrap_or_default();
                        format_cache.insert(t.id.clone(), values);
                        values
                    }
                } else {
                    (0, 0)
                };
                let bitrate = if t.bitrate > 0 {
                    t.bitrate
                } else {
                    probed_bitrate
                };
                let sample_rate = if t.sample_rate > 0 {
                    t.sample_rate
                } else {
                    probed_sample_rate
                };
                let mut parts: Vec<String> = Vec::with_capacity(3);
                if !codec.is_empty() {
                    parts.push(codec);
                }
                parts.push(if bitrate > 0 {
                    format!("{} kbps", bitrate)
                } else {
                    "--- kbps".to_string()
                });
                parts.push(if sample_rate > 0 {
                    format!("{:.1} kHz", sample_rate as f64 / 1000.0)
                } else {
                    "--.- kHz".to_string()
                });
                parts.join(" ")
            }
            None => String::new(),
        };

        if art_file != last_art {
            last_art = art_file.clone();
            match &art_file {
                Some(file) => {
                    let _ = weak.upgrade_in_event_loop(|app| {
                        app.set_now_has_art(false);
                        app.set_now_cover_url("".into());
                    });
                    tokio::spawn(fetch_now_art(
                        ep.covers.clone(),
                        file.clone(),
                        track_id.clone(),
                        weak.clone(),
                    ));
                }
                None => {
                    let _ = weak.upgrade_in_event_loop(|app| {
                        app.set_now_has_art(false);
                        app.set_now_cover_url("".into());
                    });
                }
            }
        }

        let stopped = now.track.is_none();
        let is_radio = track_id.starts_with("radio:");
        let station_id = track_id
            .strip_prefix("radio:")
            .unwrap_or_default()
            .to_owned();
        // Only hit the database when the station changes; the poll runs every
        // second and the answer only moves when the user tunes or bookmarks.
        let bookmarked = if is_radio && !station_id.is_empty() {
            saved_radio::Entity::find_by_id(station_id.clone())
                .one(music_player_storage::shared().await.get_connection())
                .await
                .ok()
                .flatten()
                .is_some()
        } else {
            false
        };
        let elapsed = now.position_ms as f32 / 1000.0;
        let length = length_ms as f32 / 1000.0;
        let _ = weak.upgrade_in_event_loop(move |app| {
            let station_changed = app.get_now_track_id().as_str() != track_id;
            app.set_now_title(title.into());
            app.set_now_artist(artist.into());
            app.set_now_album(album.into());
            app.set_now_path(path.into());
            app.set_now_liked(crate::is_liked(&track_id));
            app.set_now_track_id(track_id.into());
            app.set_now_is_radio(is_radio);
            if station_changed || !is_radio {
                app.set_now_station_id(station_id.into());
                app.set_now_radio_bookmarked(bookmarked);
            }
            app.set_playing(playing);
            app.set_stopped(stopped);
            // The daemon reports no position for a live stream, so the local
            // ticker owns the radio clock; only tuning a new station resets it.
            let elapsed = if is_radio && !station_changed && elapsed <= 0.0 {
                app.get_elapsed_s()
            } else {
                elapsed
            };
            app.set_elapsed_s(elapsed);
            app.set_length_s(length);
            app.set_progress(if length > 0.0 { elapsed / length } else { 0.0 });
            app.set_elapsed_text(format_time(elapsed as f64).into());
            app.set_duration_text(format_time(length as f64).into());
        });

        // Queue drawer: previous/next around the current track.
        if let Ok(resp) = tracklist
            .get_tracklist_tracks(GetTracklistTracksRequest {})
            .await
        {
            let resp = resp.into_inner();
            let history_len = resp.previous_tracks.len();
            // previous_tracks includes the current track as its last entry.
            let current_abs = history_len.saturating_sub(if now.track.is_some() { 1 } else { 0 });
            let total = history_len + resp.next_tracks.len();
            let upnext: Vec<TrackData> = resp
                .next_tracks
                .iter()
                .enumerate()
                .map(|(i, t)| track_data(t, (history_len + i) as i32))
                .collect();
            let mut history: Vec<TrackData> = resp
                .previous_tracks
                .iter()
                .take(current_abs)
                .enumerate()
                .map(|(i, t)| track_data(t, i as i32))
                .collect();
            history.reverse(); // most recent first
                               // "TRK  3/12  FLAC 986 kbps 44.1 kHz"
            let mut vfd = if total > 0 {
                format!("TRK {:>2}/{:<2}", now.index + 1, total)
            } else {
                String::new()
            };
            if !format_info.is_empty() {
                if !vfd.is_empty() {
                    vfd.push_str("  ");
                }
                vfd.push_str(&format_info);
            }
            if vfd.is_empty() {
                vfd = "--- kbps   --.- kHz".to_string();
            }
            let _ = weak.upgrade_in_event_loop(move |app| {
                crate::ui_set_queue(&app, total, upnext, history);
                app.set_vfd_info(vfd.into());
            });
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

fn filename_stem(path: &str) -> String {
    std::path::Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

/// Codec label for the VFD, derived from the uri's file extension
/// (query strings stripped for stream urls). Empty when unknown.
fn codec_of(uri: &str) -> String {
    let path = uri.split(['?', '#']).next().unwrap_or(uri);
    std::path::Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().to_uppercase())
        .filter(|e| e.len() <= 4 && e.chars().all(|c| c.is_ascii_alphanumeric()))
        .unwrap_or_default()
}

/// Read technical audio properties directly from a local current-track file
/// when an older database row (or client) did not provide them over gRPC.
/// Network streams are left to their server metadata rather than downloaded.
async fn probe_audio_format(uri: &str) -> Option<(u32, u32)> {
    if uri.starts_with("http://") || uri.starts_with("https://") {
        return probe_remote_audio_format(uri).await;
    }
    let path = uri.strip_prefix("file://").unwrap_or(uri).to_owned();
    probe_local_audio_format(path.into()).await
}

/// Probe a finite remote file without downloading the audio payload. Rockbox
/// metadata opens paths directly, so HTTP ranges are written into a sparse
/// file at their original offsets. The head covers stream headers/ID3; the
/// tail covers formats such as MP4/M4A whose `moov` atom may be at EOF.
async fn probe_remote_audio_format(uri: &str) -> Option<(u32, u32)> {
    use reqwest::header::{CONTENT_RANGE, RANGE};
    use reqwest::StatusCode;
    use tokio::io::{AsyncSeekExt, AsyncWriteExt};

    const RANGE_BYTES: u64 = 2 * 1024 * 1024;

    let client = reqwest::Client::new();
    let head = client
        .get(uri)
        .header(RANGE, format!("bytes=0-{}", RANGE_BYTES - 1))
        .send()
        .await
        .ok()?;
    if head.status() != StatusCode::PARTIAL_CONTENT {
        return None;
    }
    let total = content_range_total(head.headers().get(CONTENT_RANGE)?.to_str().ok()?)?;
    if total == 0 {
        return None;
    }
    let head_bytes = head.bytes().await.ok()?;

    let parsed_url = reqwest::Url::parse(uri).ok()?;
    let extension = std::path::Path::new(parsed_url.path())
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| value.len() <= 8 && value.chars().all(|c| c.is_ascii_alphanumeric()))
        .unwrap_or("audio");
    let cache_dir = std::env::temp_dir().join("music-player-remote-metadata");
    tokio::fs::create_dir_all(&cache_dir).await.ok()?;
    let cache_path = cache_dir.join(format!("{:x}.{extension}", md5::compute(uri)));
    let mut file = tokio::fs::File::create(&cache_path).await.ok()?;
    file.set_len(total).await.ok()?;
    file.write_all(&head_bytes).await.ok()?;
    file.flush().await.ok()?;

    if let Some(values) = probe_local_audio_format(cache_path.clone()).await {
        if values.0 > 0 && values.1 > 0 {
            return Some(values);
        }
    }

    if total > RANGE_BYTES {
        let tail_start = total.saturating_sub(RANGE_BYTES);
        let tail = client
            .get(uri)
            .header(RANGE, format!("bytes={tail_start}-{}", total - 1))
            .send()
            .await
            .ok()?;
        if tail.status() != StatusCode::PARTIAL_CONTENT {
            return None;
        }
        let tail_bytes = tail.bytes().await.ok()?;
        file.seek(std::io::SeekFrom::Start(tail_start)).await.ok()?;
        file.write_all(&tail_bytes).await.ok()?;
        file.flush().await.ok()?;
    }
    drop(file);
    probe_local_audio_format(cache_path).await
}

fn content_range_total(value: &str) -> Option<u64> {
    let (_, total) = value.rsplit_once('/')?;
    (total != "*").then(|| total.parse().ok()).flatten()
}

async fn probe_local_audio_format(path: std::path::PathBuf) -> Option<(u32, u32)> {
    tokio::task::spawn_blocking(move || {
        let metadata = rockbox_metadata::read(path).ok()?;
        Some((metadata.bitrate, metadata.sample_rate))
    })
    .await
    .ok()?
}

pub fn track_data(t: &TrackProto, index: i32) -> TrackData {
    TrackData {
        id: t.id.clone(),
        title: if t.title.is_empty() {
            filename_stem(&t.uri)
        } else {
            t.title.clone()
        },
        artist: t.artist.clone(),
        album: t
            .album
            .as_ref()
            .map(|a| a.title.clone())
            .unwrap_or_default(),
        length_ms: (t.duration * 1000.0) as u64,
        index,
        disc: t.disc_number,
        track_no: t.track_number,
        album_id: t.album.as_ref().map(|a| a.id.clone()).unwrap_or_default(),
    }
}

async fn init_volume(channel: &Channel, weak: &Weak<AppWindow>) {
    let mut mixer = MixerServiceClient::new(channel.clone());
    if let Ok(resp) = mixer.get_volume(GetVolumeRequest {}).await {
        let volume = resp.into_inner().volume.min(100) as f32 / 100.0;
        let _ = weak.upgrade_in_event_loop(move |app| app.set_volume(volume));
    }
}

async fn init_audio_settings(channel: &Channel, weak: &Weak<AppWindow>) {
    let mut mixer = MixerServiceClient::new(channel.clone());
    if let Ok(resp) = mixer.get_audio_settings(GetAudioSettingsRequest {}).await {
        let s = resp.into_inner();
        let data = AudioSettingsData {
            eq_enabled: s.eq_enabled,
            eq_precut: s.eq_precut,
            eq_bands: s.eq_bands.iter().map(|b| (b.cutoff, b.q, b.gain)).collect(),
            bass: s.bass,
            bass_min: s.bass_min,
            bass_max: s.bass_max,
            treble: s.treble,
            treble_min: s.treble_min,
            treble_max: s.treble_max,
            balance: s.balance,
            rg_type: s.replaygain_type,
            rg_preamp: s.replaygain_preamp,
            rg_noclip: s.replaygain_noclip,
            crossfade: s.crossfade,
            fade_in_delay: s.fade_in_delay,
            fade_in_duration: s.fade_in_duration,
            fade_out_delay: s.fade_out_delay,
            fade_out_duration: s.fade_out_duration,
            fade_out_mixmode: s.fade_out_mixmode,
            dithering: s.dithering,
        };
        let _ = weak.upgrade_in_event_loop(move |app| {
            crate::ui_set_audio_settings(&app, data);
        });
    }
}

/// The liked tracks to show.
///
/// The daemon's list when a provider is connected — it is the only thing that
/// knows that server's stars — else the local set filtered over the cached
/// library.
fn liked_list(state: &WorkerState) -> Vec<TrackData> {
    if let Some(remote) = &state.remote_liked {
        return remote
            .iter()
            .enumerate()
            .map(|(i, track)| track_data(track, i as i32))
            .collect();
    }
    state
        .tracks
        .iter()
        .filter(|t| state.liked.contains(&t.proto.id))
        .enumerate()
        .map(|(i, t)| track_data(&t.proto, i as i32))
        .collect()
}

/// Fetch albums, artists and tracks, flipping `library-loading` around it so
/// every list screen shows placeholders rather than an empty page — including
/// after a server switch, which re-reads the lot.
async fn load_library(
    channel: &Channel,
    ep: &Endpoints,
    weak: &Weak<AppWindow>,
    state: &Arc<Mutex<WorkerState>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _ = weak.upgrade_in_event_loop(|app| app.set_library_loading(true));
    {
        // Every cached id belongs to whichever library was current when it was
        // fetched, so a reload — which is what a server switch triggers —
        // drops the lot.
        let mut st = state.lock().await;
        st.album_cache.clear();
        st.playlist_cache.clear();
    }
    let mut lib = LibraryServiceClient::new(channel.clone());

    let albums = lib
        .get_albums(GetAlbumsRequest {
            limit: PAGE,
            offset: 0,
            filter: String::new(),
        })
        .await?
        .into_inner()
        .albums;
    let artists = lib
        .get_artists(GetArtistsRequest {
            limit: PAGE,
            offset: 0,
            filter: String::new(),
        })
        .await?
        .into_inner()
        .artists;
    let tracks = lib
        .get_tracks(GetTracksRequest {
            limit: PAGE,
            offset: 0,
            filter: String::new(),
        })
        .await?
        .into_inner()
        .tracks;

    let full: Vec<FullTrack> = tracks
        .iter()
        .map(|t| FullTrack {
            proto: t.clone(),
            album_id: t.album.as_ref().map(|a| a.id.clone()).unwrap_or_default(),
            artist: t.artist.clone(),
        })
        .collect();

    // The daemon knows whose likes these are: with a provider connected they
    // are *its* stars, and their ids mean nothing to the local like store —
    // which is why filtering the cache locally left the screen empty.
    let remote_liked = lib
        .get_liked_tracks(GetLikedTracksRequest {
            offset: 0,
            limit: 500,
        })
        .await
        .ok()
        .map(|response| response.into_inner().tracks);

    let liked = {
        let mut st = state.lock().await;
        st.tracks = full;
        match remote_liked {
            Some(tracks) if !tracks.is_empty() => {
                // The ids go into the same set the heart icon consults, so a
                // remote star lights it just as a local like does.
                for track in &tracks {
                    st.liked.insert(track.id.clone());
                }
                st.remote_liked = Some(tracks);
            }
            // Nothing from the daemon: the local set, which is what a purely
            // local library has always used.
            _ => st.remote_liked = None,
        }
        liked_list(&st)
    };

    let data = LibraryData {
        albums: albums
            .iter()
            .map(|a| AlbumData {
                id: a.id.clone(),
                title: a.title.clone(),
                artist: a.artist.clone(),
                year: if a.year > 0 {
                    a.year.to_string()
                } else {
                    String::new()
                },
                art_file: if a.cover.is_empty() {
                    None
                } else {
                    Some(a.cover.clone())
                },
            })
            .collect(),
        artists: artists
            .iter()
            .map(|a| ArtistData {
                id: a.id.clone(),
                name: a.name.clone(),
                picture: a.picture.clone(),
            })
            .collect(),
        tracks: tracks
            .iter()
            .enumerate()
            .map(|(i, t)| track_data(t, i as i32))
            .collect(),
        liked,
    };

    // Keyed by album id, not by position: `ui_set_library` de-duplicates the
    // list it is given, so an index taken from this side can address a
    // different album — or none at all — by the time the art arrives.
    let art_files: Vec<(String, String)> = data
        .albums
        .iter()
        .filter_map(|a| a.art_file.clone().map(|f| (a.id.clone(), f)))
        .collect();

    let _ = weak.upgrade_in_event_loop(move |app| crate::ui_set_library(&app, data));

    // Album art thumbnails, fetched sequentially so we don't hammer the server.
    let covers = ep.covers.clone();
    let weak2 = weak.clone();
    tokio::spawn(async move {
        if !await_covers(&covers).await {
            return;
        }
        let total = art_files.len();
        let mut loaded = 0usize;
        let mut failed = 0usize;
        for (album_id, file) in art_files {
            match fetch_thumb(&covers, &file, 320).await {
                Some((w, h, rgba)) => {
                    loaded += 1;
                    let _ = weak2.upgrade_in_event_loop(move |app| {
                        crate::ui_set_album_art(&app, &album_id, w, h, rgba);
                    });
                }
                None => {
                    failed += 1;
                    // Silence here is what made a broken art pipeline look like
                    // "every album shows the placeholder" with nothing to go on.
                    tracing::warn!(album = %album_id, url = %format!("{covers}{file}"),
                        "could not load album art");
                }
            }
        }
        tracing::info!(total, loaded, failed, "album art loaded");
    });

    // Artist pictures are full URLs (Rocksky CDN), fetched the same way.
    let artist_pics: Vec<(usize, String)> = artists
        .iter()
        .enumerate()
        .filter(|(_, a)| !a.picture.is_empty())
        .map(|(i, a)| (i, a.picture.clone()))
        .collect();
    let weak3 = weak.clone();
    tokio::spawn(async move {
        for (idx, url) in artist_pics {
            if let Some((w, h, rgba)) = fetch_thumb("", &url, 96).await {
                let _ = weak3.upgrade_in_event_loop(move |app| {
                    crate::ui_set_artist_art(&app, idx, w, h, rgba);
                });
            }
        }
    });
    Ok(())
}

async fn load_playlists(channel: &Channel, weak: &Weak<AppWindow>) {
    let mut playlists = PlaylistServiceClient::new(channel.clone());
    if let Ok(resp) = playlists.find_all(FindAllRequest {}).await {
        let data: Vec<PlaylistData> = resp
            .into_inner()
            .playlists
            .into_iter()
            .map(|p| PlaylistData {
                id: p.id,
                name: p.name,
                description: p.description,
                // The server's own count: a listing reports one without
                // sending the entries, so counting `tracks` here gave zero for
                // every row.
                track_count: p.track_count.max(p.tracks.len() as u32) as i64,
            })
            .collect();
        let _ = weak.upgrade_in_event_loop(move |app| crate::ui_set_playlists(&app, data));
    }
}

async fn open_playlist(
    channel: &Channel,
    state: &Arc<Mutex<WorkerState>>,
    weak: &Weak<AppWindow>,
    id: String,
    open_picker: bool,
) -> Result<(), tonic::Status> {
    if let Some((tracks, count)) = state.lock().await.playlist_cache.get(&id).cloned() {
        let id = id.clone();
        let _ = weak.upgrade_in_event_loop(move |app| {
            crate::ui_show_playlist(&app, id, tracks, count, open_picker);
        });
        return Ok(());
    }

    let mut playlists = PlaylistServiceClient::new(channel.clone());
    let resp = playlists
        .get_playlist_details(GetPlaylistDetailsRequest { id: id.clone() })
        .await?
        .into_inner();
    // The full tracks, not just their ids: the daemon has already sent
    // everything a row needs, and looking each one up in the cached library
    // dropped whatever was not cached — most of a remote playlist.
    let count = resp.track_count.max(resp.tracks.len() as u32);
    let tracks = resp.tracks;
    state
        .lock()
        .await
        .playlist_cache
        .insert(id.clone(), (tracks.clone(), count));
    let _ = weak.upgrade_in_event_loop(move |app| {
        crate::ui_show_playlist(&app, id, tracks, count, open_picker);
    });
    Ok(())
}

/// A saved playlist's tracks with stream uris, for loading into the queue.
async fn playlist_tracks(
    channel: &Channel,
    state: &Arc<Mutex<WorkerState>>,
    id: &str,
) -> Result<Vec<TrackProto>, tonic::Status> {
    // Pressing play on a playlist you are looking at should not re-ask for
    // what the detail view already fetched.
    if let Some((tracks, _)) = state.lock().await.playlist_cache.get(id) {
        return Ok(tracks.clone());
    }
    let mut playlists = PlaylistServiceClient::new(channel.clone());
    let resp = playlists
        .get_playlist_details(GetPlaylistDetailsRequest { id: id.to_string() })
        .await?
        .into_inner();
    let count = resp.track_count.max(resp.tracks.len() as u32);
    state
        .lock()
        .await
        .playlist_cache
        .insert(id.to_string(), (resp.tracks.clone(), count));
    Ok(resp.tracks)
}

/// The shared HTTP client for artwork.
///
/// One client, cloned per call: each `reqwest::Client` owns a connection pool,
/// so building one per cover (a library is hundreds) burns a file descriptor
/// apiece and reuses nothing.
fn http() -> reqwest::Client {
    static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);
    CLIENT.clone()
}

/// Block until the daemon's HTTP port answers, or give up.
///
/// When the desktop boots the daemon in-process, gRPC comes up before the HTTP
/// server that serves `/covers/`. The library — and with it every cover fetch —
/// lands in that window, so without this the whole grid fails in one burst of
/// connection-refused and stays on placeholders for the rest of the session.
/// Any response counts, including a 404: it proves something is listening.
async fn await_covers(covers_base: &str) -> bool {
    const ATTEMPTS: usize = 40;
    const DELAY: Duration = Duration::from_millis(250);
    for attempt in 0..ATTEMPTS {
        if http().get(covers_base).send().await.is_ok() {
            if attempt > 0 {
                tracing::info!(attempt, "cover server is up");
            }
            return true;
        }
        tokio::time::sleep(DELAY).await;
    }
    tracing::warn!(
        covers_base,
        "cover server never came up; album art is unavailable"
    );
    false
}

async fn fetch_thumb(covers_base: &str, file: &str, max: u32) -> Option<(u32, u32, Vec<u8>)> {
    let url = if file.starts_with("http://") || file.starts_with("https://") {
        file.to_owned()
    } else {
        format!("{covers_base}{file}")
    };
    let bytes = http().get(&url).send().await.ok()?.bytes().await.ok()?;
    tokio::task::spawn_blocking(move || {
        let img = image::load_from_memory(&bytes).ok()?;
        let thumb = img.thumbnail(max, max).to_rgba8();
        Some((thumb.width(), thumb.height(), thumb.into_raw()))
    })
    .await
    .ok()?
}

async fn fetch_now_art(covers_base: String, file: String, track_id: String, weak: Weak<AppWindow>) {
    let url = if file.starts_with("http://") || file.starts_with("https://") {
        file.clone()
    } else {
        format!("{covers_base}{file}")
    };
    let Ok(bytes) = reqwest::get(&url)
        .await
        .and_then(|response| response.error_for_status())
    else {
        return;
    };
    let Ok(bytes) = bytes.bytes().await else {
        return;
    };

    // Souvlaki/macOS loads artwork from a URL. Keep a local copy so artwork
    // works with the embedded HTTP server, remote servers, and macOS App
    // Transport Security alike.
    let cache_dir = std::env::temp_dir().join("music-player-now-playing");
    let cache_name = std::path::Path::new(&file)
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("cover"));
    let cache_path = cache_dir.join(cache_name);
    let cover_url = if tokio::fs::create_dir_all(&cache_dir).await.is_ok()
        && tokio::fs::write(&cache_path, &bytes).await.is_ok()
    {
        format!("file://{}", cache_path.to_string_lossy())
    } else {
        url
    };

    let decoded = tokio::task::spawn_blocking(move || {
        let img = image::load_from_memory(&bytes).ok()?;
        let thumb = img.thumbnail(256, 256).to_rgba8();
        Some((thumb.width(), thumb.height(), thumb.into_raw()))
    })
    .await
    .ok()
    .flatten();
    if let Some((w, h, rgba)) = decoded {
        let _ = weak.upgrade_in_event_loop(move |app| {
            if app.get_now_track_id().as_str() == track_id {
                crate::ui_set_now_art(&app, w, h, rgba);
                app.set_now_cover_url(cover_url.into());
            }
        });
    }
}

async fn load_tracks(
    channel: &Channel,
    tracks: Vec<TrackProto>,
    start_index: i32,
) -> Result<(), tonic::Status> {
    if tracks.is_empty() {
        return Ok(());
    }
    let mut tracklist = TracklistServiceClient::new(channel.clone());
    tracklist
        .load_tracks(LoadTracksRequest {
            tracks,
            start_index,
        })
        .await?;
    Ok(())
}

async fn fetch_album(
    channel: &Channel,
    id: &str,
) -> Result<Option<music_player_server::api::metadata::v1alpha1::Album>, tonic::Status> {
    let mut library = LibraryServiceClient::new(channel.clone());
    Ok(library
        .get_album_details(GetAlbumDetailsRequest { id: id.to_owned() })
        .await?
        .into_inner()
        .album)
}

/// An album's tracks, fetching the album only if it is not already cached.
async fn fetch_album_tracks(
    channel: &Channel,
    state: &Arc<Mutex<WorkerState>>,
    id: &str,
) -> Result<Vec<TrackProto>, tonic::Status> {
    // Before the request, not after it: the point is to skip the round trip.
    if let Some(cached) = state.lock().await.album_cache.get(id) {
        return Ok(cached.clone());
    }
    let Some(album) = fetch_album(channel, id).await? else {
        return Ok(Vec::new());
    };
    Ok(album_tracks_from(state, id, album).await)
}

/// Turn an already-fetched album into playable rows, and remember them.
///
/// The listing carries every field a row needs, uri included, so this makes no
/// request of its own. It used to look each track up in the cached library,
/// which dropped anything uncached — most of a remote album — and, once that
/// was fixed by fetching the misses, opening an album became a dozen
/// sequential round trips.
async fn album_tracks_from(
    state: &Arc<Mutex<WorkerState>>,
    id: &str,
    mut album: music_player_server::api::metadata::v1alpha1::Album,
) -> Vec<TrackProto> {
    let songs = std::mem::take(&mut album.tracks);
    let by_id: HashMap<String, TrackProto> = state
        .lock()
        .await
        .tracks
        .iter()
        .map(|track| (track.proto.id.clone(), track.proto.clone()))
        .collect();

    let tracks: Vec<TrackProto> = songs
        .into_iter()
        .map(|song| {
            // The cached copy is preferred only for what the listing does not
            // repeat per song.
            let mut track = by_id.get(&song.id).cloned().unwrap_or_else(|| TrackProto {
                id: song.id.clone(),
                title: song.title.clone(),
                artist: song.artist.clone(),
                duration: song.duration,
                uri: song.uri.clone(),
                ..Default::default()
            });
            if track.uri.is_empty() {
                track.uri = song.uri.clone();
            }
            track.album = Some(album.clone());
            track.track_number = song.track_number;
            track.disc_number = song.disc_number;
            track
        })
        .collect();

    state
        .lock()
        .await
        .album_cache
        .insert(id.to_string(), tracks.clone());
    tracks
}

/// Every track by one artist, in library order.
///
/// Artist ids are `md5(name)` server-side, so both the id and the bare name
/// are accepted — the sidebar and the artist page reach this by different
/// routes and it is not worth making them agree first.
async fn artist_tracks(state: &Arc<Mutex<WorkerState>>, id: &str) -> Vec<TrackProto> {
    let st = state.lock().await;
    st.tracks
        .iter()
        .filter(|t| t.artist == id || format!("{:x}", md5::compute(t.artist.as_bytes())) == id)
        .map(|t| t.proto.clone())
        .collect()
}

async fn open_album(
    channel: &Channel,
    state: &Arc<Mutex<WorkerState>>,
    weak: &Weak<AppWindow>,
    id: String,
) {
    // The page is already open with a placeholder list, so a failure has to
    // put it back rather than leaving it loading forever.
    let failed = |weak: &Weak<AppWindow>| {
        let _ = weak.upgrade_in_event_loop(|app| app.set_detail_loading(false));
    };
    // One `GetAlbumDetails` for both the header and the rows. This used to
    // call it twice — once here and once inside `fetch_album_tracks` — which
    // doubled the wait on every album against a remote server.
    let Ok(Some(album)) = fetch_album(channel, &id).await else {
        failed(weak);
        return;
    };
    let tracks = album_tracks_from(state, &id, album.clone()).await;
    let detail = AlbumDetailData {
        id,
        title: album.title,
        artist: album.artist,
        year: if album.year > 0 {
            album.year.to_string()
        } else {
            String::new()
        },
        label: String::new(),
        tracks: tracks
            .iter()
            .enumerate()
            .map(|(i, t)| track_data(t, i as i32))
            .collect(),
    };
    let _ = weak.upgrade_in_event_loop(move |app| crate::ui_show_album_detail(&app, detail));
}

// ── Providers (remote servers the library is read from) ─────────────────────

/// Save a server with the daemon and make it the current provider.
///
/// Connecting is all this does. Every library screen reads through whatever
/// the daemon has current, so there is nothing to navigate to — and nothing
/// here touches playback, which keeps going across a switch.
async fn connect_provider(
    weak: &Weak<AppWindow>,
    server: &SavedServer,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Already saved — the list comes from the daemon, so connecting is just
    // asking it to use one of its own rows.
    const CONNECT: &str = r#"mutation($id: ID!) { connectToServer(id: $id) { id name url kind } }"#;

    let connected = graphql(CONNECT, serde_json::json!({ "id": &server.id })).await?;
    let name = connected["connectToServer"]["name"]
        .as_str()
        .unwrap_or(&server.name)
        .to_string();
    let url = connected["connectToServer"]["url"]
        .as_str()
        .unwrap_or(&server.url)
        .to_string();

    let _ = weak.upgrade_in_event_loop(move |app| {
        app.set_server_error("".into());
        app.set_provider_name(name.into());
        app.set_provider_url(url.clone().into());
        crate::set_active_provider(&url);
        crate::refresh_servers_model(&app);
    });
    Ok(())
}

/// The saved servers, as the daemon has them.
async fn load_servers(
    weak: &Weak<AppWindow>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    const QUERY: &str = r#"query { savedServers { id kind name url } }"#;
    let data = graphql(QUERY, serde_json::json!({})).await?;
    let servers: Vec<SavedServer> = data["savedServers"]
        .as_array()
        .map(|rows| {
            rows.iter()
                .map(|row| SavedServer {
                    id: row["id"].as_str().unwrap_or_default().to_string(),
                    kind: row["kind"].as_str().unwrap_or_default().to_string(),
                    name: row["name"].as_str().unwrap_or_default().to_string(),
                    url: row["url"].as_str().unwrap_or_default().to_string(),
                })
                .collect()
        })
        .unwrap_or_default();
    let _ = weak.upgrade_in_event_loop(move |app| crate::ui_set_servers(&app, servers));
    Ok(())
}

/// The places the audio could come out, as the daemon sees them.
///
/// The daemon does the discovering — Chromecast, UPnP/DLNA and mDNS peers all
/// arrive through it — so this is a read rather than a scan.
async fn load_renderers(
    weak: &Weak<AppWindow>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Two calls, not one: `connectedCastDevice` *errors* when nothing is
    // connected rather than returning null, and one error in a combined query
    // failed the whole thing — which is why this list came back empty until
    // something was already casting.
    const LIST: &str = r#"query { listCastDevices { id name app } }"#;
    const CONNECTED: &str = r#"query { connectedCastDevice { id } }"#;

    let data = graphql(LIST, serde_json::json!({})).await?;
    let devices: Vec<(String, String, String)> = data["listCastDevices"]
        .as_array()
        .map(|rows| {
            rows.iter()
                .map(|row| {
                    (
                        row["id"].as_str().unwrap_or_default().to_string(),
                        row["name"].as_str().unwrap_or_default().to_string(),
                        row["app"].as_str().unwrap_or_default().to_string(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    // Nothing connected is an error here, not a null, and it is the ordinary
    // case — so a failure means "playing on this machine".
    let current = graphql(CONNECTED, serde_json::json!({}))
        .await
        .ok()
        .and_then(|data| {
            data["connectedCastDevice"]["id"]
                .as_str()
                .map(str::to_owned)
        })
        .unwrap_or_default();

    let _ = weak.upgrade_in_event_loop(move |app| {
        crate::ui_set_renderers(&app, devices, current);
    });
    Ok(())
}

async fn activate_renderer(
    weak: &Weak<AppWindow>,
    id: &str,
    cast: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if id.is_empty() {
        const OFF: &str = r#"mutation { disconnectFromCastDevice { id } }"#;
        // Failing here just means nothing was connected.
        let _ = graphql(OFF, serde_json::json!({})).await;
    } else if cast {
        const ON: &str = r#"mutation($id: ID!) { connectToCastDevice(id: $id) { id } }"#;
        graphql(ON, serde_json::json!({ "id": id })).await?;
    } else {
        const ON: &str = r#"mutation($id: ID!) { connectToDevice(id: $id) { id } }"#;
        graphql(ON, serde_json::json!({ "id": id })).await?;
    }
    load_renderers(weak).await
}

/// Ask the daemon which provider it is reading from, if any.
async fn load_connected_provider(weak: &Weak<AppWindow>) {
    const QUERY: &str = r#"query { connectedServer { name url } }"#;
    let (name, url) = match graphql(QUERY, serde_json::json!({})).await {
        Ok(data) => {
            let server = &data["connectedServer"];
            (
                server["name"].as_str().unwrap_or_default().to_string(),
                server["url"].as_str().unwrap_or_default().to_string(),
            )
        }
        // The daemon may not be up yet; the next connect will set it.
        Err(_) => (String::new(), String::new()),
    };
    let _ = weak.upgrade_in_event_loop(move |app| {
        app.set_provider_name(name.into());
        app.set_provider_url(url.clone().into());
        crate::set_active_provider(&url);
        crate::refresh_servers_model(&app);
    });
}

/// One GraphQL round trip to the daemon, with its in-body errors surfaced.
async fn graphql(
    query: &str,
    variables: serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let response = http()
        .post(endpoints().graphql)
        .json(&serde_json::json!({ "query": query, "variables": variables }))
        .send()
        .await?;
    let body: serde_json::Value = response.json().await?;
    // GraphQL answers 200 even when it failed; the errors array is the only
    // thing that says so.
    if let Some(errors) = body.get("errors").and_then(|e| e.as_array()) {
        if let Some(first) = errors.first() {
            let message = first
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("the daemon refused the request");
            return Err(message.into());
        }
    }
    body.get("data")
        .cloned()
        .ok_or_else(|| "the daemon returned no data".into())
}

/// Browses `_music-player._tcp.local.` for ~2.5 s and returns
/// (name, host, port) per resolved gRPC peer (deduped). Peers that resolve
/// to one of THIS machine's addresses are dropped — the embedded daemon
/// already has its own switcher row.
async fn discover_music_player_servers() -> Vec<(String, String, u16)> {
    let own_addrs: std::collections::HashSet<String> = if_addrs::get_if_addrs()
        .map(|ifs| ifs.into_iter().map(|i| i.addr.ip().to_string()).collect())
        .unwrap_or_default();
    let found = Arc::new(Mutex::new(Vec::<(String, String, u16)>::new()));
    let found2 = found.clone();
    let handle = tokio::task::spawn_blocking(move || {
        let Ok(daemon) = mdns_sd::ServiceDaemon::new() else {
            return;
        };
        let Ok(receiver) = daemon.browse("_music-player._tcp.local.") else {
            return;
        };
        let deadline = std::time::Instant::now() + Duration::from_millis(2500);
        while let Ok(event) =
            receiver.recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
        {
            if let mdns_sd::ServiceEvent::ServiceResolved(info) = event {
                let name = info
                    .get_fullname()
                    .trim_end_matches("._music-player._tcp.local.")
                    .to_string();
                // The daemon advertises http-/websocket-/grpc- instances per
                // host; only the gRPC one is a valid switch target.
                if !name.starts_with("grpc-") {
                    continue;
                }
                // TXT carries device_name=… — nicer than the instance id.
                let display = info
                    .get_property_val_str("device_name")
                    .filter(|d| !d.is_empty())
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| name.trim_start_matches("grpc-").to_string());
                let port = info.get_port();
                for addr in info.get_addresses() {
                    if addr.is_ipv4() {
                        let host = addr.to_string();
                        if own_addrs.contains(&host) {
                            continue;
                        }
                        let mut list = found2.blocking_lock();
                        if !list.iter().any(|(_, h, p)| h == &host && *p == port) {
                            list.push((display.clone(), host, port));
                        }
                    }
                }
            }
            if std::time::Instant::now() >= deadline {
                break;
            }
        }
        let _ = daemon.shutdown();
    });
    let _ = handle.await;
    let list = found.lock().await.clone();
    list
}

// ── Command loop ────────────────────────────────────────────────────────────

/// Push the rebuilt liked list to the UI after a store mutation.
async fn push_liked(state: &Arc<Mutex<WorkerState>>, weak: &Weak<AppWindow>) {
    let liked = {
        let st = state.lock().await;
        // Only the local store is a file; a provider's stars live on it.
        if st.remote_liked.is_none() {
            likes::save(&st.liked);
        }
        liked_list(&st)
    };
    let _ = weak.upgrade_in_event_loop(move |app| {
        crate::ui_set_liked(&app, liked);
    });
}

async fn push_radios(
    state: &Arc<Mutex<WorkerState>>,
    weak: &Weak<AppWindow>,
    stations: Vec<crate::radio::Station>,
) {
    let saved: HashSet<String> = crate::radio::load_bookmarks()
        .await
        .into_iter()
        .map(|station| station.id)
        .collect();
    let rows: Vec<StationData> = {
        let mut state = state.lock().await;
        for station in &stations {
            state.radios.insert(station.id.clone(), station.clone());
        }
        stations
            .into_iter()
            .map(|station| StationData {
                id: station.id.clone(),
                name: station.name,
                subtitle: [station.genre, station.country]
                    .into_iter()
                    .filter(|value| !value.is_empty())
                    .collect::<Vec<_>>()
                    .join(" · "),
                source: station.source,
                logo: station.logo,
                bookmarked: saved.contains(&station.id),
            })
            .collect()
    };
    let logos: Vec<(String, String)> = rows
        .iter()
        .filter(|row| !row.logo.is_empty())
        .map(|row| (row.id.clone(), row.logo.clone()))
        .collect();
    let _ = weak.upgrade_in_event_loop(move |app| crate::ui_set_radios(&app, rows));
    fetch_radio_logos(weak.clone(), logos);
}

/// Station logos are fetched after the rows are on screen and pushed in as they
/// decode, a few at a time so a 100-station result doesn't open 100 sockets.
fn fetch_radio_logos(weak: Weak<AppWindow>, logos: Vec<(String, String)>) {
    const PARALLEL: usize = 8;
    if logos.is_empty() {
        return;
    }
    tokio::spawn(async move {
        let mut inflight = tokio::task::JoinSet::new();
        for (id, url) in logos {
            if inflight.len() >= PARALLEL {
                let _ = inflight.join_next().await;
            }
            inflight.spawn(fetch_radio_logo(weak.clone(), id, url));
        }
        while inflight.join_next().await.is_some() {}
    });
}

async fn fetch_radio_logo(weak: Weak<AppWindow>, id: String, url: String) {
    let Some(bytes) = crate::radio::logo_bytes(&url).await else {
        return;
    };
    let decoded = tokio::task::spawn_blocking(move || {
        let img = image::load_from_memory(&bytes).ok()?;
        let thumb = img.thumbnail(96, 96).to_rgba8();
        Some((thumb.width(), thumb.height(), thumb.into_raw()))
    })
    .await
    .ok()
    .flatten();
    let Some((w, h, rgba)) = decoded else {
        tracing::debug!("could not decode radio logo {url}");
        return;
    };
    let _ = weak.upgrade_in_event_loop(move |app| {
        crate::ui_set_radio_logo(&app, &id, w, h, rgba);
    });
}

async fn cmd_loop(
    mut rx: UnboundedReceiver<Cmd>,
    state: Arc<Mutex<WorkerState>>,
    weak: Weak<AppWindow>,
) {
    while let Some(cmd) = rx.recv().await {
        let channel = chan();
        let mut playback = PlaybackServiceClient::new(channel.clone());
        let mut mixer = MixerServiceClient::new(channel.clone());
        let mut tracklist = TracklistServiceClient::new(channel.clone());
        let res: Result<(), tonic::Status> = async {
            match cmd {
                Cmd::Play => {
                    playback.play(PlayRequest {}).await?;
                    state.lock().await.playing = true;
                }
                Cmd::Pause => {
                    playback.pause(PauseRequest {}).await?;
                    state.lock().await.playing = false;
                }
                Cmd::Next => {
                    playback.next(NextRequest {}).await?;
                }
                Cmd::Previous => {
                    playback.previous(PreviousRequest {}).await?;
                }
                Cmd::SeekMs(position_ms) => {
                    playback.seek(SeekRequest { position_ms }).await?;
                }
                Cmd::SetVolume(pct) => {
                    let volume = (pct.clamp(0.0, 1.0) * 100.0).round() as u32;
                    mixer.set_volume(SetVolumeRequest { volume }).await?;
                }
                Cmd::SetMute(mute) => {
                    // The daemon keeps the level while muted, so unmuting
                    // restores it — nothing to remember on this side.
                    mixer.set_mute(SetMuteRequest { mute }).await?;
                }
                Cmd::PlayAlbum(id) => {
                    let tracks = fetch_album_tracks(&channel, &state, &id).await?;
                    load_tracks(&channel, tracks, 0).await?;
                }
                Cmd::PlayAlbumAt(id, pos) => {
                    let tracks = fetch_album_tracks(&channel, &state, &id).await?;
                    load_tracks(&channel, tracks, pos).await?;
                }
                Cmd::PlayAlbumShuffled(id) => {
                    let mut tracks = fetch_album_tracks(&channel, &state, &id).await?;
                    // Fisher–Yates via fastrand: shuffle client-side.
                    for i in (1..tracks.len()).rev() {
                        tracks.swap(i, fastrand::usize(..=i));
                    }
                    load_tracks(&channel, tracks, 0).await?;
                }
                Cmd::PlayArtist(id) => {
                    let tracks = artist_tracks(&state, &id).await;
                    load_tracks(&channel, tracks, 0).await?;
                }
                Cmd::PlayArtistShuffled(id) => {
                    let mut tracks = artist_tracks(&state, &id).await;
                    // Fisher–Yates via fastrand: shuffle client-side.
                    for i in (1..tracks.len()).rev() {
                        tracks.swap(i, fastrand::usize(..=i));
                    }
                    load_tracks(&channel, tracks, 0).await?;
                }
                Cmd::PlayArtistAt(id, pos) => {
                    let tracks = artist_tracks(&state, &id).await;
                    load_tracks(&channel, tracks, pos).await?;
                }
                Cmd::PlayAllAt(pos) => {
                    let tracks: Vec<TrackProto> = state
                        .lock()
                        .await
                        .tracks
                        .iter()
                        .map(|t| t.proto.clone())
                        .collect();
                    load_tracks(&channel, tracks, pos).await?;
                }
                Cmd::PlayLikedShuffled => {
                    let mut tracks: Vec<TrackProto> = {
                        let st = state.lock().await;
                        st.tracks
                            .iter()
                            .filter(|t| st.liked.contains(&t.proto.id))
                            .map(|t| t.proto.clone())
                            .collect()
                    };
                    // Fisher–Yates via fastrand: shuffle client-side.
                    for i in (1..tracks.len()).rev() {
                        tracks.swap(i, fastrand::usize(..=i));
                    }
                    load_tracks(&channel, tracks, 0).await?;
                }
                Cmd::PlayLikedAt(pos) => {
                    let tracks: Vec<TrackProto> = {
                        let st = state.lock().await;
                        st.tracks
                            .iter()
                            .filter(|t| st.liked.contains(&t.proto.id))
                            .map(|t| t.proto.clone())
                            .collect()
                    };
                    load_tracks(&channel, tracks, pos).await?;
                }
                Cmd::QueueJump(idx) => {
                    tracklist
                        .play_track_at(PlayTrackAtRequest { index: idx as u32 })
                        .await?;
                }
                Cmd::QueueClear => {
                    tracklist.clear_tracklist(ClearTracklistRequest {}).await?;
                }
                Cmd::QueueRemove(idx) => {
                    tracklist
                        .remove_track_at(RemoveTrackAtRequest {
                            position: idx as u32,
                        })
                        .await?;
                }
                Cmd::OpenAlbum(id) => {
                    open_album(&channel, &state, &weak, id).await;
                }
                Cmd::SetShuffle(enabled) => {
                    tracklist.shuffle(ShuffleRequest { enabled }).await?;
                }
                Cmd::SetRepeat(mode) => {
                    tracklist.set_repeat(SetRepeatRequest { mode }).await?;
                }
                Cmd::AudioSet(name, value) => {
                    mixer
                        .set_audio_setting(SetAudioSettingRequest { name, value })
                        .await?;
                }
                Cmd::EqBandSet(band, gain) => {
                    mixer
                        .set_eq_band_gain(SetEqBandGainRequest {
                            band: band as u32,
                            gain,
                        })
                        .await?;
                }
                Cmd::OpenPlaylist(id) => {
                    open_playlist(&channel, &state, &weak, id, false).await?;
                }
                Cmd::PlaySavedPlaylist(id) => {
                    let tracks = playlist_tracks(&channel, &state, &id).await?;
                    load_tracks(&channel, tracks, 0).await?;
                }
                Cmd::ShufflePlaylist(id) => {
                    let mut tracks = playlist_tracks(&channel, &state, &id).await?;
                    // Shuffled here rather than by turning the player's own
                    // shuffle on: that is a mode the user set, and starting a
                    // playlist should not silently change it. Fisher–Yates via
                    // fastrand, as the album path does.
                    for i in (1..tracks.len()).rev() {
                        tracks.swap(i, fastrand::usize(..=i));
                    }
                    load_tracks(&channel, tracks, 0).await?;
                }
                Cmd::SmartPlaylistCreate {
                    name,
                    rsql,
                    sort_by,
                    sort_order,
                    limit,
                } => {
                    let mut playlists = PlaylistServiceClient::new(channel.clone());
                    let smart = SmartPlaylistProto {
                        filter: rsql,
                        sort_by,
                        sort_order,
                        limit,
                    };
                    match playlists
                        .create(CreateRequest {
                            name,
                            tracks: vec![],
                            smart: Some(smart),
                        })
                        .await
                    {
                        Ok(resp) => {
                            let id = resp.into_inner().id;
                            load_playlists(&channel, &weak).await;
                            // A smart playlist arrives full, so it opens on its
                            // tracks rather than on the picker.
                            open_playlist(&channel, &state, &weak, id, false).await?;
                        }
                        Err(status) => {
                            let message = status.message().to_owned();
                            let _ = weak.upgrade_in_event_loop(move |app| {
                                app.set_pl_form_preview_error(message.into());
                                app.set_show_playlist_form(true);
                            });
                        }
                    }
                }
                Cmd::SmartPlaylistPreview {
                    rsql,
                    sort_by,
                    limit,
                } => {
                    let mut playlists = PlaylistServiceClient::new(channel.clone());
                    let preview = playlists
                        .preview_smart_playlist(PreviewSmartPlaylistRequest {
                            smart: Some(SmartPlaylistProto {
                                filter: rsql,
                                sort_by,
                                sort_order: String::new(),
                                limit,
                            }),
                        })
                        .await;
                    let (count, error) = match preview {
                        Ok(resp) => {
                            let resp = resp.into_inner();
                            (resp.count as i32, resp.error)
                        }
                        Err(status) => (-1, status.message().to_owned()),
                    };
                    let _ = weak.upgrade_in_event_loop(move |app| {
                        app.set_pl_form_previewing(false);
                        app.set_pl_form_preview_error(error.clone().into());
                        // A filter that did not compile has no count to show.
                        app.set_pl_form_preview_count(if error.is_empty() { count } else { -1 });
                    });
                }
                Cmd::PlaylistCreate { name, description } => {
                    // The daemon's playlists have no description field; the
                    // name is what identifies them everywhere.
                    let _ = description;
                    let mut playlists = PlaylistServiceClient::new(channel.clone());
                    let resp = playlists
                        .create(CreateRequest {
                            name,
                            tracks: vec![],
                            smart: None,
                        })
                        .await?
                        .into_inner();
                    load_playlists(&channel, &weak).await;
                    open_playlist(&channel, &state, &weak, resp.id, true).await?;
                }
                Cmd::PlaylistUpdate {
                    id,
                    name,
                    description,
                } => {
                    let _ = description;
                    let mut playlists = PlaylistServiceClient::new(channel.clone());
                    playlists
                        .rename(RenameRequest {
                            id: id.clone(),
                            name,
                        })
                        .await?;
                    load_playlists(&channel, &weak).await;
                    state.lock().await.playlist_cache.remove(&id);
                    open_playlist(&channel, &state, &weak, id, false).await.ok();
                }
                Cmd::PlaylistDelete(id) => {
                    let mut playlists = PlaylistServiceClient::new(channel.clone());
                    playlists.delete(DeleteRequest { id }).await?;
                    load_playlists(&channel, &weak).await;
                }
                Cmd::PlaylistAddTrack {
                    playlist_id,
                    track_id,
                } => {
                    let mut playlists = PlaylistServiceClient::new(channel.clone());
                    playlists
                        .add_item(AddItemRequest {
                            id: playlist_id.clone(),
                            track_id,
                        })
                        .await?;
                    load_playlists(&channel, &weak).await;
                    // The cached copy is now wrong.
                    state.lock().await.playlist_cache.remove(&playlist_id);
                    open_playlist(&channel, &state, &weak, playlist_id, false)
                        .await
                        .ok();
                }
                Cmd::PlaylistRemoveTrack {
                    playlist_id,
                    track_id,
                } => {
                    let mut playlists = PlaylistServiceClient::new(channel.clone());
                    playlists
                        .remove_item(RemoveItemRequest {
                            id: playlist_id.clone(),
                            track_id,
                        })
                        .await?;
                    load_playlists(&channel, &weak).await;
                    // The cached copy is now wrong.
                    state.lock().await.playlist_cache.remove(&playlist_id);
                    open_playlist(&channel, &state, &weak, playlist_id, false)
                        .await
                        .ok();
                }
                Cmd::InsertTracks { position, tracks } => {
                    insert_track_ids(&channel, &state, position, tracks).await?;
                }
                Cmd::InsertAlbum { album_id, position } => {
                    let ids: Vec<String> = fetch_album_tracks(&channel, &state, &album_id)
                        .await?
                        .iter()
                        .map(|t| t.id.clone())
                        .collect();
                    insert_track_ids(&channel, &state, position, ids).await?;
                }
                Cmd::LikeTrack { id, like } => {
                    {
                        let mut st = state.lock().await;
                        if like {
                            st.liked.insert(id.clone());
                        } else {
                            st.liked.remove(&id);
                        }
                        // Keep the provider's list in step so the Liked screen
                        // reflects the toggle without waiting for a reload.
                        // Looked up before the mutable borrow.
                        let newly_liked = st
                            .tracks
                            .iter()
                            .find(|t| t.proto.id == id)
                            .map(|t| t.proto.clone());
                        if let Some(remote) = st.remote_liked.as_mut() {
                            match (like, newly_liked) {
                                (true, Some(track))
                                    if !remote.iter().any(|t| t.id == id) =>
                                {
                                    remote.insert(0, track)
                                }
                                (false, _) => remote.retain(|track| track.id != id),
                                _ => {}
                            }
                        }
                    }
                    push_liked(&state, &weak).await;
                    // The daemon forwards the like to Rocksky.
                    let mut lib = LibraryServiceClient::new(channel.clone());
                    lib.like_track(LikeTrackRequest { id, like }).await?;
                }
                Cmd::LikeAlbum(id) => {
                    let ids: Vec<String> = {
                        let mut st = state.lock().await;
                        let ids: Vec<String> = st
                            .tracks
                            .iter()
                            .filter(|t| t.album_id == id)
                            .map(|t| t.proto.id.clone())
                            .collect();
                        st.liked.extend(ids.iter().cloned());
                        ids
                    };
                    push_liked(&state, &weak).await;
                    let mut lib = LibraryServiceClient::new(channel.clone());
                    for id in ids {
                        lib.like_track(LikeTrackRequest { id, like: true }).await?;
                    }
                }
                Cmd::SwitchServer { host, grpc_port } => {
                    switch_server(&host, grpc_port);
                    let display = format!("{host}:{grpc_port}");
                    let _ = weak.upgrade_in_event_loop(move |app| {
                        app.set_connected(false);
                        app.set_status_text(display.into());
                    });
                }
                Cmd::DiscoverServers => {
                    let weak2 = weak.clone();
                    tokio::spawn(async move {
                        let found = discover_music_player_servers().await;
                        let _ = weak2.upgrade_in_event_loop(move |app| {
                            crate::ui_set_discovered(&app, found);
                        });
                    });
                }
                Cmd::RadioSearch(query) => {
                    push_radios(&state, &weak, crate::radio::search(&query).await).await;
                }
                Cmd::RadioBrowse(tag) => {
                    let stations = crate::radio::browse(&tag).await.unwrap_or_default();
                    push_radios(&state, &weak, stations).await;
                }
                Cmd::RadioBookmarks => {
                    let stations = crate::radio::load_bookmarks().await;
                    push_radios(&state, &weak, stations).await;
                }
                Cmd::ExtensionsRescan => {
                    let extensions = crate::extensions::rescan().await;
                    let _ = weak.upgrade_in_event_loop(move |app| {
                        crate::ui_set_extensions(&app, extensions);
                    });
                }
                Cmd::ExtensionSetEnabled { id, enabled } => {
                    match crate::extensions::set_enabled(&id, enabled).await {
                        Ok(extensions) => {
                            let _ = weak.upgrade_in_event_loop(move |app| {
                                crate::ui_set_extensions(&app, extensions);
                            });
                        }
                        Err(e) => {
                            tracing::warn!(extension = %id, "could not switch it: {e}");
                            // Put the row back the way it was rather than
                            // leaving the switch showing a state that did not
                            // stick.
                            let extensions = crate::extensions::load().await;
                            let _ = weak.upgrade_in_event_loop(move |app| {
                                crate::ui_set_extensions(&app, extensions);
                            });
                        }
                    }
                }
                Cmd::RadioStations => {
                    let stations = crate::radio::load_stations().await;
                    push_radios(&state, &weak, stations).await;
                }
                Cmd::RadioCheckStream(url) => {
                    let check = music_player_storage::radio_stream::probe(&url).await;
                    let _ = weak.upgrade_in_event_loop(move |app| {
                        crate::ui_set_stream_check(&app, &check);
                    });
                }
                Cmd::RadioAddStation(draft) => match crate::radio::add_station(draft).await {
                    Ok(_) => {
                        let stations = crate::radio::load_stations().await;
                        let _ = weak.upgrade_in_event_loop(|app| {
                            app.set_add_station_saving(false);
                            app.invoke_close_add_station();
                        });
                        push_radios(&state, &weak, stations).await;
                    }
                    Err(e) => {
                        let _ = weak.upgrade_in_event_loop(move |app| {
                            app.set_add_station_saving(false);
                            app.set_add_station_status(e.into());
                            app.set_add_station_ok(false);
                        });
                    }
                },
                Cmd::RadioBookmark(id) => {
                    let station = state.lock().await.radios.get(&id).cloned();
                    if let Some(station) = station {
                        crate::radio::toggle_bookmark(&station).await;
                        let stations = state.lock().await.radios.values().cloned().collect();
                        push_radios(&state, &weak, stations).await;
                    }
                }
                Cmd::RadioPlay(id) => {
                    let station = state.lock().await.radios.get(&id).cloned();
                    if let Some(station) = station {
                        let uri = crate::radio::resolve_stream(&station).await;
                        let track = TrackProto {
                            id: format!("radio:{}", station.id),
                            title: station.name.clone(),
                            artist: station.source,
                            uri,
                            bitrate: station.bitrate,
                            album: Some(music_player_server::api::metadata::v1alpha1::Album {
                                id: "internet-radio".into(),
                                // The station name doubles as the "album": once
                                // ICY metadata arrives the title/artist become
                                // the song on the air, and this is what still
                                // says which station it came from.
                                title: station.name,
                                cover: station.logo,
                                ..Default::default()
                            }),
                            ..Default::default()
                        };
                        load_tracks(&channel, vec![track], 0).await?;
                    }
                }
                Cmd::LoadServers => {
                    if let Err(e) = load_servers(&weak).await {
                        tracing::warn!("could not list servers: {e}");
                    }
                    // Which one is current, so the switcher marks the right
                    // row rather than falling back to "this machine".
                    load_connected_provider(&weak).await;
                }
                Cmd::AddServer {
                    kind,
                    name,
                    url,
                    username,
                    password,
                } => {
                    const ADD: &str =
                        r#"mutation($input: ServerInput!) { addServer(input: $input) { id } }"#;
                    let input = serde_json::json!({
                        "input": {
                            "kind": kind,
                            "name": if name.is_empty() { url.clone() } else { name },
                            "url": url,
                            "username": username,
                            "password": password,
                        }
                    });
                    match graphql(ADD, input).await {
                        Ok(_) => {
                            let _ = load_servers(&weak).await;
                        }
                        Err(e) => {
                            let message = e.to_string();
                            let _ = weak.upgrade_in_event_loop(move |app| {
                                app.set_server_error(message.into());
                            });
                        }
                    }
                }
                Cmd::DeleteServer(id) => {
                    const DELETE: &str = r#"mutation($id: ID!) { deleteServer(id: $id) }"#;
                    if let Err(e) = graphql(DELETE, serde_json::json!({ "id": id })).await {
                        tracing::warn!("could not delete the server: {e}");
                    }
                    let _ = load_servers(&weak).await;
                }
                Cmd::LoadRenderers => {
                    if let Err(e) = load_renderers(&weak).await {
                        tracing::warn!("could not list renderers: {e}");
                    }
                }
                Cmd::ActivateRenderer(id, cast) => {
                    if let Err(e) = activate_renderer(&weak, &id, cast).await {
                        tracing::warn!("could not switch renderer: {e}");
                    }
                }
                Cmd::DisconnectProvider => {
                    const DISCONNECT: &str = r#"mutation { disconnectFromServer { id } }"#;
                    // A failure here just means nothing was connected.
                    let _ = graphql(DISCONNECT, serde_json::json!({})).await;
                    let ep = endpoints();
                    if let Err(e) = load_library(&channel, &ep, &weak, &state).await {
                        tracing::warn!("reloading the library failed: {e}");
                    }
                    load_playlists(&channel, &weak).await;
                }
                Cmd::ConnectServer(srv) => {
                    // Connecting is now the daemon's job: it makes the server
                    // the current provider and every library screen follows,
                    // rather than the desktop opening a browse view of its own.
                    match connect_provider(&weak, &srv).await {
                        Ok(()) => {
                            // The daemon is pointed somewhere else now, and
                            // every library screen reads through it — so they
                            // all need re-fetching. Playback is untouched.
                            let ep = endpoints();
                            if let Err(e) = load_library(&channel, &ep, &weak, &state).await {
                                tracing::warn!("reloading the library failed: {e}");
                            }
                            load_playlists(&channel, &weak).await;
                        }
                        Err(e) => {
                            let message = e.to_string();
                            let _ = weak.upgrade_in_event_loop(move |app| {
                                app.set_server_error(message.into());
                            });
                        }
                    }
                }
            }
            Ok(())
        }
        .await;
        if let Err(e) = res {
            tracing::warn!("command failed: {e}");
        }
    }
}

/// Queue inserts by track id. Rockbox positions: -2 = play next (insert
/// right after the current track), -3 = add last (append).
async fn insert_track_ids(
    channel: &Channel,
    state: &Arc<Mutex<WorkerState>>,
    position: i32,
    ids: Vec<String>,
) -> Result<(), tonic::Status> {
    let mut tracklist = TracklistServiceClient::new(channel.clone());
    match position {
        -2 => {
            // PlayNext inserts right after the current track, so pushing in
            // reverse keeps the batch in its original order.
            let protos: Vec<TrackProto> = {
                let st = state.lock().await;
                ids.iter()
                    .filter_map(|id| {
                        st.tracks
                            .iter()
                            .find(|t| &t.proto.id == id)
                            .map(|t| t.proto.clone())
                    })
                    .collect()
            };
            for track in protos.into_iter().rev() {
                tracklist
                    .play_next(PlayNextRequest { track: Some(track) })
                    .await?;
            }
        }
        _ => {
            // Append: the daemon resolves AddTrack ids from its own library.
            for id in ids {
                tracklist
                    .add_track(AddTrackRequest {
                        track: Some(TrackProto {
                            id,
                            ..Default::default()
                        }),
                    })
                    .await?;
            }
        }
    }
    Ok(())
}

/// Local clock: advances elapsed between polls and animates the VU meters
/// (decorative — the daemon does not export PCM levels over gRPC). Runs at
/// 60 ms so the meters bounce like meters instead of crawling.
async fn ticker(weak: Weak<AppWindow>) {
    const TICK_S: f64 = 0.06;
    let mut phase: f64 = 0.0;
    loop {
        tokio::time::sleep(Duration::from_millis((TICK_S * 1000.0) as u64)).await;
        phase += TICK_S;
        let t = phase;
        let _ = weak.upgrade_in_event_loop(move |app| {
            if app.get_playing() {
                let length = app.get_length_s();
                let elapsed = app.get_elapsed_s() + TICK_S as f32;
                // Live streams have no length; clamping to it would pin the
                // readout at 00:00 instead of counting up.
                let elapsed = if length > 0.0 {
                    elapsed.min(length)
                } else {
                    elapsed
                };
                app.set_elapsed_s(elapsed);
                app.set_progress(if length > 0.0 { elapsed / length } else { 0.0 });
                app.set_elapsed_text(format_time(elapsed as f64).into());
                let l = 0.62 + 0.22 * (t * 5.9).sin() + 0.12 * (t * 13.7).sin();
                let r = 0.60 + 0.24 * (t * 5.1 + 1.3).sin() + 0.12 * (t * 11.3).sin();
                app.set_vu_left(l.clamp(0.05, 1.0) as f32);
                app.set_vu_right(r.clamp(0.05, 1.0) as f32);
            } else {
                app.set_vu_left((app.get_vu_left() * 0.8).max(0.0));
                app.set_vu_right((app.get_vu_right() * 0.8).max(0.0));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What the albums screen is actually given: the gRPC library response must
    /// carry a cover filename, or `art_file` is None and every card falls back
    /// to the disc icon. Ignored by default — it needs the daemon up.
    /// `cargo test -p music-player-desktop -- --ignored grpc_albums`
    #[tokio::test]
    #[ignore]
    async fn grpc_albums_carry_covers() {
        let mut lib = LibraryServiceClient::new(chan());
        let albums = lib
            .get_albums(GetAlbumsRequest {
                limit: 10,
                offset: 0,
                filter: String::new(),
            })
            .await
            .expect("get_albums")
            .into_inner()
            .albums;
        assert!(!albums.is_empty(), "the daemon returned no albums");
        for album in albums.iter().take(5) {
            println!("album={:?} cover={:?}", album.title, album.cover);
        }
        let with_cover = albums.iter().filter(|a| !a.cover.is_empty()).count();
        assert!(
            with_cover > 0,
            "no album in the gRPC response carries a cover filename"
        );
    }

    /// End-to-end check of the album-art pipeline against a running daemon:
    /// fetch a cover over HTTP and decode it to a thumbnail, exactly as the
    /// albums screen does. Ignored by default — it needs the daemon up.
    /// `cargo test -p music-player-desktop -- --ignored fetch_thumb`
    #[tokio::test]
    #[ignore]
    async fn fetch_thumb_decodes_a_real_cover() {
        let covers = "http://127.0.0.1:5053/covers/";
        let file = "376c5380f8b5ac9666f3775b9abd3774.jpg";
        let thumb = fetch_thumb(covers, file, 320).await;
        let (w, h, rgba) = thumb.expect("cover fetch/decode returned None");
        assert!(w > 0 && h > 0, "decoded a {w}x{h} thumbnail");
        assert_eq!(rgba.len(), (w * h * 4) as usize);
    }
}
