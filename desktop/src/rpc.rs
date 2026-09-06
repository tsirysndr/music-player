//! gRPC worker: talks to the music-player daemon, pushes state into the
//! Slint UI. music-player has no server-streaming RPCs, so now-playing and
//! queue state are polled (1 s) instead of followed.

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::{LazyLock, RwLock as StdRwLock};
use std::time::Duration;

use slint::Weak;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;
use tonic::transport::{Channel, Endpoint};

use crate::AppWindow;

use music_player_addons::{jellyfin::Jellyfin, subsonic::Subsonic, Browsable};
use music_player_server::api::metadata::v1alpha1::Track as TrackProto;
use music_player_server::api::music::v1alpha1::{
    library_service_client::LibraryServiceClient, mixer_service_client::MixerServiceClient,
    playback_service_client::PlaybackServiceClient, playlist_service_client::PlaylistServiceClient,
    tracklist_service_client::TracklistServiceClient, AddItemRequest, AddTrackRequest,
    ClearTracklistRequest, CreateRequest, DeleteRequest, FindAllRequest, GetAlbumsRequest,
    GetArtistsRequest, GetAudioSettingsRequest, GetCurrentlyPlayingSongRequest,
    GetPlaylistDetailsRequest, GetTracklistTracksRequest, GetTracksRequest, GetVolumeRequest,
    LoadTracksRequest, NextRequest, PauseRequest, PlayNextRequest, PlayRequest, PlayTrackAtRequest,
    PreviousRequest, RemoveItemRequest, RemoveTrackAtRequest, RenameRequest, SeekRequest,
    SetAudioSettingRequest, SetEqBandGainRequest, SetRepeatRequest, SetVolumeRequest,
    ShuffleRequest,
};
use music_player_types::types as mp_types;

use crate::likes;
use crate::servers::SavedServer;

/// All albums/artists/tracks in one page — a limit of 0 means "none" on the
/// server side, so ask for effectively-everything instead.
const PAGE: i32 = 100_000;

// ── Commands from UI callbacks ──────────────────────────────────────────────

#[derive(Debug)]
pub enum Cmd {
    PlayPause,
    Next,
    Previous,
    SeekMs(u32),
    SetVolume(f32),
    PlayAlbum(String),
    PlayAlbumAt(String, i32),
    PlayArtist(String),
    PlayAllAt(i32),
    PlayLikedAt(i32),
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
    Browse {
        title: String,
        path: String,
        push: bool,
    },
    PlayDir(String),
    PlayDirAt(String, i32),
    OpenPlaylist(String),
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

#[derive(Clone, Debug)]
pub struct BrowseEntryData {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
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
    pub grpc: String,
    pub covers: String,
    pub display: String,
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
        grpc: format!("http://{host}:{grpc_port}"),
        covers: format!("http://{host}:{http_port}/covers/"),
        display: format!("{host}:{grpc_port}"),
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
    disc_number: i32,
    track_number: i32,
}

/// Live connection to a saved Subsonic/Jellyfin server.
enum RemoteSource {
    Subsonic(Subsonic),
    Jellyfin(Jellyfin),
}

#[derive(Default)]
struct WorkerState {
    tracks: Vec<FullTrack>,
    liked: HashSet<String>,
    playing: bool,
    remote: Option<RemoteSource>,
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
    let state = Arc::new(Mutex::new(WorkerState {
        liked: likes::load(),
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

    init_volume(channel, weak).await;
    init_audio_settings(channel, weak).await;
    load_library(channel, ep, weak, state).await?;
    load_playlists(channel, weak).await;

    // Poll now-playing + queue until the daemon (or the target) goes away.
    let mut tracklist = TracklistServiceClient::new(channel.clone());
    let mut last_art: Option<String> = None;
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

        let (title, artist, path, length_ms, art_file, track_id) = match &now.track {
            Some(t) => (
                if t.title.is_empty() {
                    filename_stem(&t.uri)
                } else {
                    t.title.clone()
                },
                t.artist.clone(),
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
                0,
                None,
                String::new(),
            ),
        };

        if art_file != last_art {
            last_art = art_file.clone();
            match &art_file {
                Some(file) => {
                    tokio::spawn(fetch_now_art(ep.covers.clone(), file.clone(), weak.clone()));
                }
                None => {
                    let _ = weak.upgrade_in_event_loop(|app| app.set_now_has_art(false));
                }
            }
        }

        let stopped = now.track.is_none();
        let elapsed = now.position_ms as f32 / 1000.0;
        let length = length_ms as f32 / 1000.0;
        let queue_pos = now.index;
        let _ = weak.upgrade_in_event_loop(move |app| {
            app.set_now_title(title.into());
            app.set_now_artist(artist.into());
            app.set_now_path(path.into());
            app.set_now_liked(crate::is_liked(&track_id));
            app.set_now_track_id(track_id.into());
            app.set_playing(playing);
            app.set_stopped(stopped);
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
            let vfd = if total > 0 {
                format!("TRK {:>2}/{:<2}", queue_pos + 1, total)
            } else {
                "--- / ---".to_string()
            };
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

fn track_data(t: &TrackProto, index: i32) -> TrackData {
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

/// Rebuilds the liked TrackData list from the worker cache (library order).
fn liked_list(state: &WorkerState) -> Vec<TrackData> {
    state
        .tracks
        .iter()
        .filter(|t| state.liked.contains(&t.proto.id))
        .enumerate()
        .map(|(i, t)| track_data(&t.proto, i as i32))
        .collect()
}

async fn load_library(
    channel: &Channel,
    ep: &Endpoints,
    weak: &Weak<AppWindow>,
    state: &Arc<Mutex<WorkerState>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
            disc_number: t.disc_number,
            track_number: t.track_number,
        })
        .collect();

    let liked = {
        let mut st = state.lock().await;
        st.tracks = full;
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
            })
            .collect(),
        tracks: tracks
            .iter()
            .enumerate()
            .map(|(i, t)| track_data(t, i as i32))
            .collect(),
        liked,
    };

    let art_files: Vec<(usize, String)> = data
        .albums
        .iter()
        .enumerate()
        .filter_map(|(i, a)| a.art_file.clone().map(|f| (i, f)))
        .collect();

    let _ = weak.upgrade_in_event_loop(move |app| crate::ui_set_library(&app, data));

    // Album art thumbnails, fetched sequentially so we don't hammer the server.
    let covers = ep.covers.clone();
    let weak2 = weak.clone();
    tokio::spawn(async move {
        for (idx, file) in art_files {
            if let Some((w, h, rgba)) = fetch_thumb(&covers, &file, 320).await {
                let _ = weak2.upgrade_in_event_loop(move |app| {
                    crate::ui_set_album_art(&app, idx, w, h, rgba);
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
                track_count: p.tracks.len() as i64,
            })
            .collect();
        let _ = weak.upgrade_in_event_loop(move |app| crate::ui_set_playlists(&app, data));
    }
}

async fn open_playlist(
    channel: &Channel,
    weak: &Weak<AppWindow>,
    id: String,
    open_picker: bool,
) -> Result<(), tonic::Status> {
    let mut playlists = PlaylistServiceClient::new(channel.clone());
    let resp = playlists
        .get_playlist_details(GetPlaylistDetailsRequest { id: id.clone() })
        .await?
        .into_inner();
    let ids: Vec<String> = resp.tracks.iter().map(|t| t.id.clone()).collect();
    let _ = weak.upgrade_in_event_loop(move |app| {
        crate::ui_show_playlist(&app, id, ids, open_picker);
    });
    Ok(())
}

/// A saved playlist's tracks with stream uris, for loading into the queue.
async fn playlist_tracks(channel: &Channel, id: &str) -> Result<Vec<TrackProto>, tonic::Status> {
    let mut playlists = PlaylistServiceClient::new(channel.clone());
    let resp = playlists
        .get_playlist_details(GetPlaylistDetailsRequest { id: id.to_string() })
        .await?
        .into_inner();
    Ok(resp.tracks)
}

async fn fetch_thumb(covers_base: &str, file: &str, max: u32) -> Option<(u32, u32, Vec<u8>)> {
    let url = format!("{covers_base}{file}");
    let bytes = reqwest::get(&url).await.ok()?.bytes().await.ok()?;
    tokio::task::spawn_blocking(move || {
        let img = image::load_from_memory(&bytes).ok()?;
        let thumb = img.thumbnail(max, max).to_rgba8();
        Some((thumb.width(), thumb.height(), thumb.into_raw()))
    })
    .await
    .ok()?
}

async fn fetch_now_art(covers_base: String, file: String, weak: Weak<AppWindow>) {
    if let Some((w, h, rgba)) = fetch_thumb(&covers_base, &file, 256).await {
        let _ = weak.upgrade_in_event_loop(move |app| {
            crate::ui_set_now_art(&app, w, h, rgba);
        });
    }
}

/// Sorted, deduped album tracks (disc, then track number) for detail/play.
fn album_tracks(state: &WorkerState, album_id: &str) -> Vec<TrackProto> {
    let mut with_order: Vec<(&FullTrack, i32, i32)> = state
        .tracks
        .iter()
        .filter(|t| t.album_id == album_id)
        .map(|t| (t, t.disc_number, t.track_number))
        .collect();
    with_order.sort_by_key(|(_, disc, num)| (*disc, *num));
    // The library can hold the same song twice (rescans, duplicate files);
    // don't show it twice within one album.
    with_order.dedup_by(|a, b| a.1 == b.1 && a.2 == b.2 && a.0.proto.title == b.0.proto.title);
    with_order
        .into_iter()
        .map(|(t, _, _)| t.proto.clone())
        .collect()
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

async fn open_album(state: &Arc<Mutex<WorkerState>>, weak: &Weak<AppWindow>, id: String) {
    let (title, artist, year, tracks) = {
        let st = state.lock().await;
        let tracks = album_tracks(&st, &id);
        let (title, artist, year) = tracks
            .first()
            .and_then(|t| t.album.as_ref())
            .map(|a| {
                (
                    a.title.clone(),
                    tracks.first().map(|t| t.artist.clone()).unwrap_or_default(),
                    if a.year > 0 {
                        a.year.to_string()
                    } else {
                        String::new()
                    },
                )
            })
            .unwrap_or_default();
        (title, artist, year, tracks)
    };
    if tracks.is_empty() {
        return;
    }
    let detail = AlbumDetailData {
        id,
        title,
        artist,
        year,
        label: String::new(),
        tracks: tracks
            .iter()
            .enumerate()
            .map(|(i, t)| track_data(t, i as i32))
            .collect(),
    };
    let _ = weak.upgrade_in_event_loop(move |app| crate::ui_show_album_detail(&app, detail));
}

// ── Remote server browsing (Subsonic / Jellyfin via music-player-addons) ────

impl RemoteSource {
    fn as_browsable(&mut self) -> &mut (dyn Browsable + Send) {
        match self {
            RemoteSource::Subsonic(c) => c,
            RemoteSource::Jellyfin(c) => c,
        }
    }
}

async fn connect_server(srv: &SavedServer) -> Result<RemoteSource, String> {
    match srv.kind.as_str() {
        "jellyfin" => {
            let mut client = Jellyfin::with_credentials(&srv.url, &srv.username, &srv.password);
            client
                .connect()
                .await
                .map_err(|e| format!("Jellyfin login failed: {e}"))?;
            Ok(RemoteSource::Jellyfin(client))
        }
        _ => {
            let mut client = Subsonic::with_credentials(&srv.url, &srv.username, &srv.password);
            client
                .connect()
                .await
                .map_err(|e| format!("Subsonic login failed: {e}"))?;
            Ok(RemoteSource::Subsonic(client))
        }
    }
}

fn browse_track_entry(i: usize, t: &mp_types::Track) -> BrowseEntryData {
    let number = t.track_number.unwrap_or((i + 1) as u32);
    BrowseEntryData {
        name: format!("{number:>2}  {}", t.title),
        path: format!("track:{i}"),
        is_dir: false,
    }
}

/// Lists one browse level. Paths: "root", "albums", "artists", "playlists",
/// "album:<id>", "artist:<id>", "playlist:<id>".
async fn browse_entries(
    source: &mut RemoteSource,
    path: &str,
) -> Result<Vec<BrowseEntryData>, String> {
    let client = source.as_browsable();
    let entries = match path {
        "root" => vec![
            BrowseEntryData {
                name: "Albums".into(),
                path: "albums".into(),
                is_dir: true,
            },
            BrowseEntryData {
                name: "Artists".into(),
                path: "artists".into(),
                is_dir: true,
            },
            BrowseEntryData {
                name: "Playlists".into(),
                path: "playlists".into(),
                is_dir: true,
            },
        ],
        "albums" => client
            .albums(None, 0, PAGE)
            .await
            .map_err(|e| e.to_string())?
            .iter()
            .map(|a| BrowseEntryData {
                name: if a.artist.is_empty() {
                    a.title.clone()
                } else {
                    format!("{} — {}", a.title, a.artist)
                },
                path: format!("album:{}", a.id),
                is_dir: true,
            })
            .collect(),
        "artists" => client
            .artists(None, 0, PAGE)
            .await
            .map_err(|e| e.to_string())?
            .iter()
            .map(|a| BrowseEntryData {
                name: a.name.clone(),
                path: format!("artist:{}", a.id),
                is_dir: true,
            })
            .collect(),
        "playlists" => client
            .playlists(0, PAGE)
            .await
            .map_err(|e| e.to_string())?
            .iter()
            .map(|p| BrowseEntryData {
                name: p.name.clone(),
                path: format!("playlist:{}", p.id),
                is_dir: true,
            })
            .collect(),
        _ if path.starts_with("album:") => client
            .album(&path["album:".len()..])
            .await
            .map_err(|e| e.to_string())?
            .tracks
            .iter()
            .enumerate()
            .map(|(i, t)| browse_track_entry(i, t))
            .collect(),
        _ if path.starts_with("artist:") => client
            .artist(&path["artist:".len()..])
            .await
            .map_err(|e| e.to_string())?
            .albums
            .iter()
            .map(|a| BrowseEntryData {
                name: a.title.clone(),
                path: format!("album:{}", a.id),
                is_dir: true,
            })
            .collect(),
        _ if path.starts_with("playlist:") => client
            .playlist(&path["playlist:".len()..])
            .await
            .map_err(|e| e.to_string())?
            .tracks
            .iter()
            .enumerate()
            .map(|(i, t)| browse_track_entry(i, t))
            .collect(),
        other => return Err(format!("unknown browse path: {other}")),
    };
    Ok(entries)
}

/// The playable tracks behind a browse dir (album:/playlist:/artist:).
async fn dir_tracks(source: &mut RemoteSource, path: &str) -> Result<Vec<mp_types::Track>, String> {
    let client = source.as_browsable();
    if let Some(id) = path.strip_prefix("album:") {
        return Ok(client.album(id).await.map_err(|e| e.to_string())?.tracks);
    }
    if let Some(id) = path.strip_prefix("playlist:") {
        return Ok(client.playlist(id).await.map_err(|e| e.to_string())?.tracks);
    }
    if let Some(id) = path.strip_prefix("artist:") {
        let albums = client.artist(id).await.map_err(|e| e.to_string())?.albums;
        let mut out = Vec::new();
        for album in albums {
            let client = source.as_browsable();
            out.extend(
                client
                    .album(&album.id)
                    .await
                    .map_err(|e| e.to_string())?
                    .tracks,
            );
        }
        return Ok(out);
    }
    Err(format!("not a playable dir: {path}"))
}

/// Stream-URL tracks → daemon tracklist. The album must be present — the
/// daemon's LoadTracks conversion requires it.
fn to_proto_tracks(tracks: Vec<mp_types::Track>) -> Vec<TrackProto> {
    tracks
        .into_iter()
        .map(|mut t| {
            if t.album.is_none() {
                t.album = Some(mp_types::Album {
                    id: format!("{:x}", md5::compute(&t.title)),
                    title: "Unknown".to_string(),
                    ..Default::default()
                });
            }
            t.into()
        })
        .collect()
}

async fn browse_into(
    state: &Arc<Mutex<WorkerState>>,
    weak: &Weak<AppWindow>,
    title: String,
    path: String,
    push: bool,
) -> Result<(), String> {
    let mut st = state.lock().await;
    let Some(source) = st.remote.as_mut() else {
        return Err("no server connected".into());
    };
    let mut entries = browse_entries(source, &path).await?;
    // Directory levels sort by name; track levels keep album order.
    if entries.iter().all(|e| e.is_dir) {
        entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    }
    let _ = weak.upgrade_in_event_loop(move |app| {
        crate::ui_browse_opened(&app, title, path, entries, push);
    });
    Ok(())
}

// ── Server discovery (mDNS) ─────────────────────────────────────────────────

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
        likes::save(&st.liked);
        liked_list(&st)
    };
    let _ = weak.upgrade_in_event_loop(move |app| {
        crate::ui_set_liked(&app, liked);
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
                Cmd::PlayPause => {
                    if state.lock().await.playing {
                        playback.pause(PauseRequest {}).await?;
                    } else {
                        playback.play(PlayRequest {}).await?;
                    }
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
                Cmd::PlayAlbum(id) => {
                    let tracks = album_tracks(&*state.lock().await, &id);
                    load_tracks(&channel, tracks, 0).await?;
                }
                Cmd::PlayAlbumAt(id, pos) => {
                    let tracks = album_tracks(&*state.lock().await, &id);
                    load_tracks(&channel, tracks, pos).await?;
                }
                Cmd::PlayAlbumShuffled(id) => {
                    let mut tracks = album_tracks(&*state.lock().await, &id);
                    // Fisher–Yates via fastrand: shuffle client-side.
                    for i in (1..tracks.len()).rev() {
                        tracks.swap(i, fastrand::usize(..=i));
                    }
                    load_tracks(&channel, tracks, 0).await?;
                }
                Cmd::PlayArtist(id) => {
                    let tracks: Vec<TrackProto> = {
                        let st = state.lock().await;
                        // Artist ids are md5(name) server-side; match either.
                        st.tracks
                            .iter()
                            .filter(|t| {
                                t.artist == id
                                    || format!("{:x}", md5::compute(t.artist.as_bytes())) == id
                            })
                            .map(|t| t.proto.clone())
                            .collect()
                    };
                    load_tracks(&channel, tracks, 0).await?;
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
                    open_album(&state, &weak, id).await;
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
                    open_playlist(&channel, &weak, id, false).await?;
                }
                Cmd::PlaySavedPlaylist(id) => {
                    let tracks = playlist_tracks(&channel, &id).await?;
                    load_tracks(&channel, tracks, 0).await?;
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
                        })
                        .await?
                        .into_inner();
                    load_playlists(&channel, &weak).await;
                    open_playlist(&channel, &weak, resp.id, true).await?;
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
                    open_playlist(&channel, &weak, id, false).await.ok();
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
                    open_playlist(&channel, &weak, playlist_id, false)
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
                    open_playlist(&channel, &weak, playlist_id, false)
                        .await
                        .ok();
                }
                Cmd::InsertTracks { position, tracks } => {
                    insert_track_ids(&channel, &state, position, tracks).await?;
                }
                Cmd::InsertAlbum { album_id, position } => {
                    let ids: Vec<String> = album_tracks(&*state.lock().await, &album_id)
                        .iter()
                        .map(|t| t.id.clone())
                        .collect();
                    insert_track_ids(&channel, &state, position, ids).await?;
                }
                Cmd::LikeTrack { id, like } => {
                    {
                        let mut st = state.lock().await;
                        if like {
                            st.liked.insert(id);
                        } else {
                            st.liked.remove(&id);
                        }
                    }
                    push_liked(&state, &weak).await;
                }
                Cmd::LikeAlbum(id) => {
                    {
                        let mut st = state.lock().await;
                        let ids: Vec<String> = st
                            .tracks
                            .iter()
                            .filter(|t| t.album_id == id)
                            .map(|t| t.proto.id.clone())
                            .collect();
                        st.liked.extend(ids);
                    }
                    push_liked(&state, &weak).await;
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
                Cmd::ConnectServer(srv) => {
                    let name = srv.name.clone();
                    match connect_server(&srv).await {
                        Ok(source) => {
                            state.lock().await.remote = Some(source);
                            if let Err(e) =
                                browse_into(&state, &weak, name, "root".into(), true).await
                            {
                                let _ = weak.upgrade_in_event_loop(move |app| {
                                    app.set_browse_loading(false);
                                    app.set_browse_error(e.into());
                                });
                            }
                        }
                        Err(e) => {
                            let _ = weak.upgrade_in_event_loop(move |app| {
                                app.set_browse_loading(false);
                                app.set_browse_error(e.into());
                            });
                        }
                    }
                }
                Cmd::Browse { title, path, push } => {
                    if let Err(e) = browse_into(&state, &weak, title, path, push).await {
                        let _ = weak.upgrade_in_event_loop(move |app| {
                            app.set_browse_loading(false);
                            app.set_browse_error(e.into());
                        });
                    }
                }
                Cmd::PlayDir(path) => {
                    let tracks = {
                        let mut st = state.lock().await;
                        match st.remote.as_mut() {
                            Some(source) => dir_tracks(source, &path).await,
                            None => Err("no server connected".into()),
                        }
                    };
                    match tracks {
                        Ok(tracks) => load_tracks(&channel, to_proto_tracks(tracks), 0).await?,
                        Err(e) => tracing::warn!("play dir failed: {e}"),
                    }
                }
                Cmd::PlayDirAt(path, idx) => {
                    let tracks = {
                        let mut st = state.lock().await;
                        match st.remote.as_mut() {
                            Some(source) => dir_tracks(source, &path).await,
                            None => Err("no server connected".into()),
                        }
                    };
                    match tracks {
                        Ok(tracks) => load_tracks(&channel, to_proto_tracks(tracks), idx).await?,
                        Err(e) => tracing::warn!("play dir failed: {e}"),
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
/// (decorative — the daemon does not export PCM levels over gRPC).
async fn ticker(weak: Weak<AppWindow>) {
    let mut phase: f64 = 0.0;
    loop {
        tokio::time::sleep(Duration::from_millis(250)).await;
        phase += 0.25;
        let t = phase;
        let _ = weak.upgrade_in_event_loop(move |app| {
            if app.get_playing() {
                let length = app.get_length_s();
                let elapsed = (app.get_elapsed_s() + 0.25).min(length.max(0.0));
                app.set_elapsed_s(elapsed);
                app.set_progress(if length > 0.0 { elapsed / length } else { 0.0 });
                app.set_elapsed_text(format_time(elapsed as f64).into());
                let l = 0.62 + 0.22 * (t * 5.9).sin() + 0.12 * (t * 13.7).sin();
                let r = 0.60 + 0.24 * (t * 5.1 + 1.3).sin() + 0.12 * (t * 11.3).sin();
                app.set_vu_left(l.clamp(0.05, 1.0) as f32);
                app.set_vu_right(r.clamp(0.05, 1.0) as f32);
            } else {
                app.set_vu_left((app.get_vu_left() * 0.6).max(0.0));
                app.set_vu_right((app.get_vu_right() * 0.6).max(0.0));
            }
        });
    }
}
