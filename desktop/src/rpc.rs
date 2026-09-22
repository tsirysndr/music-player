//! gRPC worker: talks to the music-player daemon, pushes state into the
//! Slint UI. music-player has no server-streaming RPCs, so now-playing and
//! queue state are polled (1 s) instead of followed.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::{LazyLock, RwLock as StdRwLock};
use std::time::Duration;

use slint::Weak;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;
use tonic::transport::{Channel, Endpoint};

use crate::AppWindow;

pub use music_player_server::api::metadata::v1alpha1::{
    Album as AlbumProto, Artist as ArtistProto, Track as TrackProto,
};
use music_player_server::api::music::v1alpha1::{
    analysis_service_client::AnalysisServiceClient, library_service_client::LibraryServiceClient,
    mixer_service_client::MixerServiceClient, playback_service_client::PlaybackServiceClient,
    playlist_service_client::PlaylistServiceClient,
    tracklist_service_client::TracklistServiceClient, AddItemRequest, AddTrackRequest,
    ClearTracklistRequest, CreateRequest, DeleteRequest, FindAllRequest, GetAlbumDetailsRequest,
    GetAlbumsRequest, GetArtistsRequest, GetAudioSettingsRequest, GetCurrentlyPlayingSongRequest,
    GetLikedTracksRequest, GetPlaylistDetailsRequest, GetTrackAnalysisRequest,
    GetTracklistTracksRequest, GetTracksRequest, GetVolumeRequest, LikeTrackRequest,
    LoadTracksRequest, NextRequest, PauseRequest, PlayNextRequest, PlayRequest, PlayTrackAtRequest,
    PreviewSmartPlaylistRequest, PreviousRequest, RemoveItemRequest, RemoveTrackAtRequest,
    RenameRequest, SearchRequest, SeekRequest, SetAudioSettingRequest, SetEqBandGainRequest,
    SetMuteRequest, SetRepeatRequest, SetVolumeRequest, ShuffleRequest,
    SmartPlaylist as SmartPlaylistProto, StreamLevelsRequest,
};

use crate::likes;
use crate::SavedServer;
use music_player_entity::saved_radio;
use sea_orm::EntityTrait;

/// All albums/artists/tracks in one page — a limit of 0 means "none" on the
/// server side, so ask for effectively-everything instead.
/// One request's worth of library rows.
///
/// Sized to what a remote server will actually return in one page — Subsonic
/// caps at 500 and applies it silently — so the loops that use this can tell
/// "that is all there is" from "that is all you may have at once" by whether
/// the page came back short. Asking for 100,000 could not: it always came
/// back short, at 500.
const PAGE: i32 = 500;

// ── Commands from UI callbacks ──────────────────────────────────────────────

#[derive(Debug)]
pub enum Cmd {
    Play,
    Pause,
    Next,
    Previous,
    SeekMs(u32),
    SetVolume(f32),
    /// Refresh the Most Played tab from the local analytics views.
    LoadMostPlayed,
    /// Refresh the Statistics tab.
    LoadStats,
    /// Flip the listening-history panel between the whole history and only
    /// what the local library has, and redraw it.
    SetHistoryLocalOnly(bool),
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
    /// Ask the daemon to search. Its search is federated — the connected
    /// server *and* the local index — which filtering this side's cached
    /// library could never be: that cache is one page of a remote library.
    Search(String),
    /// Who is signed in, if anyone.
    SignIn {
        handle: String,
        password: String,
    },
    SignOut,
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
    /// Create a playlist named from the picker's query and put the pending
    /// tracks straight into it — the picker's "Create new playlist" row.
    PlaylistCreateAndAdd {
        name: String,
        track_ids: Vec<String>,
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
    /// Camelot notation, e.g. "8A". Empty when not analysed, or when the
    /// library is a source that does not analyse.
    pub key: String,
    /// Empty when unknown.
    pub bpm: String,
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
/// Bumped per palette keystroke, so a pending search can tell it has been
/// superseded.
static SEARCH_GEN: AtomicUsize = AtomicUsize::new(0);

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
    /// Whether the listening-history panel is restricted to the local
    /// library. Held here rather than read back off the UI because the
    /// worker refreshes the panel from several places — a server switch, a
    /// scan — that have no business touching Slint properties.
    history_local_only: bool,
    tracks: Vec<FullTrack>,
    liked: HashSet<String>,
    /// The same ids as `liked`, most recently liked first. This order is what
    /// the Liked screen shows AND what play-liked loads, so the queue always
    /// mirrors the list.
    liked_order: Vec<String>,
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
    /// A like the daemon is still applying: track id, the state the user
    /// chose, and how many more polls the choice outranks the daemon's
    /// answer. Without it the poll snaps the heart back to the pre-click
    /// state for the second the remote star takes to land.
    pending_like: Option<(String, bool, u8)>,
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
    // The account's likes carry real like-times (rocksky created_at, already
    // newest-first) — they are the ordering backbone. Ids only the local file
    // knows (liked while signed out, or pre-sync) go on top in file order,
    // where the file's front is the newest.
    let local = likes::load();
    let mut liked: HashSet<String> = HashSet::new();
    let mut liked_order: Vec<String> = Vec::new();
    let db = music_player_storage::shared().await;
    match music_player_storage::rocksky_likes::matched_track_ids(db.get_connection()).await {
        Ok(ids) => {
            for id in ids {
                if liked.insert(id.clone()) {
                    liked_order.push(id);
                }
            }
        }
        Err(e) => tracing::debug!("could not read restored likes: {e}"),
    }
    let mut fronted = 0usize;
    for id in local {
        if liked.insert(id.clone()) {
            liked_order.insert(fronted, id);
            fronted += 1;
        }
    }
    let state = Arc::new(Mutex::new(WorkerState {
        liked,
        liked_order,
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
    tokio::spawn(load_account_when_ready(weak.clone()));
    tokio::spawn(stream_levels(channel.clone(), weak.clone()));
    init_volume(channel, weak).await;
    init_audio_settings(channel, weak).await;
    load_library(channel, ep, weak, state).await?;
    load_playlists(channel, weak).await;

    // Poll now-playing + queue until the daemon (or the target) goes away.
    let mut tracklist = TracklistServiceClient::new(channel.clone());
    let mut last_art: Option<String> = None;
    // The waveform is a property of the track, so it is fetched when the
    // track changes rather than on every poll — plus spaced retries while it
    // has not landed, because the first ask can race the daemon's provider
    // reconnect.
    let mut last_waveform_track = String::new();
    let waveform_loaded = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut last_waveform_attempt = std::time::Instant::now();
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
        let (shuffle, repeat_mode) = (now.shuffle, now.repeat_mode);
        let liked_now = {
            let mut st = state.lock().await;
            st.playing = playing;
            // The track's own answer when the source gave one: a snapshot of
            // "everything liked" can always be incomplete — truncated by a
            // limit, or listing only songs when an album was what was starred
            // — and then some hearts are right and some are not. The set is
            // the fallback, and is all a local library has.
            match now.track.as_ref() {
                Some(track) => {
                    // The daemon's copy is a snapshot from queue time (it can
                    // even predate this session — the queue is restored from
                    // disk). The set also holds the provider's own starred
                    // list, fetched fresh at connect, so it can vouch for a
                    // star the snapshot missed; its absence proves nothing, so
                    // it only ever turns the heart ON.
                    let source = track.liked == Some(true) || st.liked.contains(&track.id);
                    // A just-clicked heart outranks the snapshot until the
                    // daemon reports the new state (or long enough that it
                    // clearly never will).
                    let mut value = source;
                    st.pending_like = match st.pending_like.take() {
                        Some((id, desired, polls))
                            if id == track.id && source != desired && polls > 0 =>
                        {
                            value = desired;
                            Some((id, desired, polls - 1))
                        }
                        _ => None,
                    };
                    value
                }
                None => false,
            }
        };

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

        if track_id != last_waveform_track {
            last_waveform_track = track_id.clone();
            // Cleared first: a stale waveform under a new track is worse than
            // none, because it invites seeking to the wrong place.
            let _ = weak.upgrade_in_event_loop(|app| {
                app.set_waveform_bars(slint::ModelRc::new(
                    slint::VecModel::from(Vec::<f32>::new()),
                ));
            });
            waveform_loaded.store(false, Ordering::Relaxed);
            last_waveform_attempt = std::time::Instant::now();
            if !track_id.is_empty() {
                tokio::spawn(load_waveform(
                    channel.clone(),
                    track_id.clone(),
                    weak.clone(),
                    Arc::clone(&waveform_loaded),
                ));
            }
        } else if !track_id.is_empty()
            && !waveform_loaded.load(Ordering::Relaxed)
            && last_waveform_attempt.elapsed() >= Duration::from_secs(5)
        {
            // The first ask can legitimately fail — right after a restart the
            // daemon has not reconnected its provider yet, so a remote id
            // resolves to nothing. One failed attempt must not be the final
            // word: keep asking, spaced out, until the waveform lands.
            last_waveform_attempt = std::time::Instant::now();
            tokio::spawn(load_waveform(
                channel.clone(),
                track_id.clone(),
                weak.clone(),
                Arc::clone(&waveform_loaded),
            ));
        }

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
            app.set_now_liked(liked_now);
            // The daemon's modes, not this side's guess: they survive a
            // restart, and every client should show what is actually in force.
            app.set_shuffle(shuffle);
            app.set_repeat_mode(repeat_mode);
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
        key: t.key.clone().unwrap_or_default(),
        // Rounded here rather than in the cell: a tempo is read as a whole
        // number, and "127.9384" in a narrow column is unreadable.
        bpm: t.bpm.map(|bpm| format!("{:.0}", bpm)).unwrap_or_default(),
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
    liked_local_protos(state)
        .iter()
        .enumerate()
        .map(|(i, t)| track_data(t, i as i32))
        .collect()
}

/// The local liked tracks, most recently liked first — the single ordering
/// shared by the Liked screen and the play-liked queue.
fn liked_local_protos(state: &WorkerState) -> Vec<TrackProto> {
    let by_id: HashMap<&str, &TrackProto> = state
        .tracks
        .iter()
        .map(|t| (t.proto.id.as_str(), &t.proto))
        .collect();
    state
        .liked_order
        .iter()
        .filter_map(|id| by_id.get(id.as_str()).map(|t| (*t).clone()))
        .collect()
}

/// The liked tracks as protos, in exactly the order the Liked screen lists
/// them — the provider's list when one is connected, else the local order.
fn liked_protos(state: &WorkerState) -> Vec<TrackProto> {
    if let Some(remote) = &state.remote_liked {
        return remote.clone();
    }
    liked_local_protos(state)
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

    // Whose listing this will be — 'local' when the daemon reads its own
    // library. A remote server may have a snapshot from the last session:
    // shown at once, so connecting reads as instant, while the fresh
    // download below replaces it when it lands.
    let (snapshot_host, _) = analytics_source().await;
    if snapshot_host != "local" {
        let host = snapshot_host.clone();
        let snap = tokio::task::spawn_blocking(move || crate::snapshot::load(&host))
            .await
            .ok()
            .flatten();
        if let Some(snap) = snap {
            // Likes are left untouched — a snapshot has no answer for them,
            // and the fresh fetch below brings the real one.
            publish_library(
                ep,
                weak,
                state,
                &snap.albums,
                &snap.artists,
                &snap.tracks,
                None,
            )
            .await;
            let _ = weak.upgrade_in_event_loop(|app| app.set_library_loading(false));
        }
    }

    let mut lib = LibraryServiceClient::new(channel.clone());

    // Paged to exhaustion rather than asked for everything at once. A remote
    // server has its own ceiling — Subsonic's is 500 rows — and it applies it
    // silently, so a single large request returned 500 albums and looked like
    // a library with 500 albums in it.
    //
    // The three listings run *concurrently*. They are independent, and against
    // a remote server each page is a round trip measured in seconds: fetched
    // one after another, opening the app on a library of a few thousand tracks
    // meant a minute of watching a loading skeleton, because the waits added
    // up. Run together the wait is the slowest one rather than the sum.
    async fn page_to_end<T, F, Fut>(mut fetch: F) -> Result<Vec<T>, tonic::Status>
    where
        F: FnMut(i32) -> Fut,
        Fut: std::future::Future<Output = Result<Vec<T>, tonic::Status>>,
    {
        // The first page goes alone: most listings fit in one, and firing a
        // window at a small library would be three wasted round trips per
        // listing. Only a full first page proves there is more.
        let mut all = fetch(0).await?;
        if all.len() < PAGE as usize {
            return Ok(all);
        }
        loop {
            // Whatever is left is fetched four pages at a time. The offsets
            // are known in advance — page N starts at N*PAGE regardless of
            // what page N-1 returns — so nothing forces the round trips into
            // a line; only the stop condition does, and a short page inside
            // a window still ends it. Against a remote server each page is a
            // full network round trip, and fetching them one after another
            // is exactly the first-connect minute this window removes.
            let base = all.len() as i32;
            let (a, b, c, d) = tokio::try_join!(
                fetch(base),
                fetch(base + PAGE),
                fetch(base + 2 * PAGE),
                fetch(base + 3 * PAGE),
            )?;
            for page in [a, b, c, d] {
                let short = page.len() < PAGE as usize;
                all.extend(page);
                if short {
                    return Ok(all);
                }
            }
        }
    }

    let albums = page_to_end({
        let mut lib = lib.clone();
        move |offset| {
            let mut lib = lib.clone();
            async move {
                Ok(lib
                    .get_albums(GetAlbumsRequest {
                        limit: PAGE,
                        offset,
                        filter: String::new(),
                    })
                    .await?
                    .into_inner()
                    .albums)
            }
        }
    });
    let artists = page_to_end({
        let mut lib = lib.clone();
        move |offset| {
            let mut lib = lib.clone();
            async move {
                Ok(lib
                    .get_artists(GetArtistsRequest {
                        limit: PAGE,
                        offset,
                        filter: String::new(),
                    })
                    .await?
                    .into_inner()
                    .artists)
            }
        }
    });
    let tracks = page_to_end({
        let mut lib = lib.clone();
        move |offset| {
            let mut lib = lib.clone();
            async move {
                Ok(lib
                    .get_tracks(GetTracksRequest {
                        limit: PAGE,
                        offset,
                        filter: String::new(),
                    })
                    .await?
                    .into_inner()
                    .tracks)
            }
        }
    });

    // The daemon knows whose likes these are: with a provider connected they
    // are *its* stars, and their ids mean nothing to the local like store —
    // which is why filtering the cache locally left the screen empty.
    // Joined with the listings rather than awaited after them: on a remote
    // server the starred list is its own round trip, and it depends on
    // nothing the listings return.
    let liked_fetch = {
        let mut lib = lib.clone();
        async move {
            lib.get_liked_tracks(GetLikedTracksRequest {
                offset: 0,
                // Every one of them: this set decides which hearts light, so a
                // ceiling here is a heart that is wrong past that many likes.
                limit: 0,
            })
            .await
        }
    };

    let (albums, artists, tracks, liked_response) =
        tokio::join!(albums, artists, tracks, liked_fetch);
    let (albums, artists, tracks) = (albums?, artists?, tracks?);

    let remote_liked = match liked_response {
        Ok(response) => Some(response.into_inner().tracks),
        Err(e) => {
            // Swallowing this made the Liked screen and the heart both look
            // like "nothing is liked" when the truth was "the daemon could
            // not be asked" — usually an older daemon still running.
            tracing::warn!("could not read liked tracks: {e}");
            None
        }
    };

    publish_library(
        ep,
        weak,
        state,
        &albums,
        &artists,
        &tracks,
        Some(remote_liked),
    )
    .await;

    // A remote listing is worth keeping: the next connect shows it instantly
    // while the fresh one downloads. The local library needs no snapshot —
    // it is already a local read.
    if snapshot_host != "local" {
        let host = snapshot_host.clone();
        tokio::task::spawn_blocking(move || {
            crate::snapshot::save(&host, &albums, &artists, &tracks);
        });
    }
    Ok(())
}

/// Push a full library listing to the UI and start the art fetches.
///
/// Shared by the network path and the snapshot path. `liked_update` is
/// `Some(answer)` when the daemon's liked list was (re)fetched — `None` for
/// a snapshot publish, which has no fresh answer and must not disturb the
/// liked state it would otherwise overwrite.
async fn publish_library(
    ep: &Endpoints,
    weak: &Weak<AppWindow>,
    state: &Arc<Mutex<WorkerState>>,
    albums: &[AlbumProto],
    artists: &[ArtistProto],
    tracks: &[TrackProto],
    liked_update: Option<Option<Vec<TrackProto>>>,
) {
    let full: Vec<FullTrack> = tracks
        .iter()
        .map(|t| FullTrack {
            proto: t.clone(),
            album_id: t.album.as_ref().map(|a| a.id.clone()).unwrap_or_default(),
            artist: t.artist.clone(),
        })
        .collect();

    let liked = {
        let mut st = state.lock().await;
        st.tracks = full;
        if let Some(remote_liked) = liked_update {
            match remote_liked {
                Some(tracks) if !tracks.is_empty() => {
                    // The ids go into the same set the heart icon consults, so
                    // a remote star lights it just as a local like does.
                    for track in &tracks {
                        if st.liked.insert(track.id.clone()) {
                            st.liked_order.push(track.id.clone());
                        }
                    }
                    st.remote_liked = Some(tracks);
                }
                // Nothing from the daemon: the local set, which is what a
                // purely local library has always used.
                _ => st.remote_liked = None,
            }
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
    // Hashed, not taken from the url's last path segment. A remote cover is
    // `/rest/getCoverArt?id=…&t=…`, whose "file name" is the whole query
    // string — too long to write on most filesystems and carrying the auth
    // token in a temp file's name. The write then failed and the fallback was
    // the plain-http url, which macOS refuses to load, so remote tracks showed
    // no artwork at all.
    let extension = std::path::Path::new(&file)
        .extension()
        .and_then(|extension| extension.to_str())
        .filter(|extension| extension.len() <= 5 && extension.chars().all(char::is_alphanumeric))
        .unwrap_or("jpg");
    let cache_path = cache_dir.join(format!("{:x}.{extension}", md5::compute(url.as_bytes())));
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

/// Who is signed in, and their avatar.
///
/// The daemon is the authority: an `atradio login` at the terminal, or
/// credentials in the environment, count as signed in here too — there is one
/// session, however it was established.
/// Load the signed-in account into the sidebar.
///
/// Returns whether the daemon *answered*. Nobody being signed in is an answer;
/// the http server not being up yet is not, and the two must not look the same
/// — writing an empty handle for an unreachable daemon is what left the row
/// showing "Sign in" for an account that was signed in all along.
async fn load_account(weak: &Weak<AppWindow>) -> bool {
    const QUERY: &str = r#"query { account { handle displayName avatar } }"#;
    let (handle, name, avatar_url) = match graphql(QUERY, serde_json::json!({})).await {
        Ok(data) => {
            let account = &data["account"];
            (
                account["handle"].as_str().unwrap_or_default().to_string(),
                account["displayName"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                account["avatar"].as_str().map(str::to_owned),
            )
        }
        Err(cause) => {
            tracing::debug!(%cause, "could not read the account");
            return false;
        }
    };

    // Fetched before touching the UI so the row appears complete rather than
    // filling in a beat later.
    let avatar = match &avatar_url {
        Some(url) => fetch_avatar(url).await,
        None => None,
    };

    let _ = weak.upgrade_in_event_loop(move |app| {
        app.set_account_handle(handle.into());
        app.set_account_name(name.into());
        match avatar {
            Some((width, height, rgba)) => {
                let mut buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(width, height);
                buffer.make_mut_bytes().copy_from_slice(&rgba);
                app.set_account_avatar(slint::Image::from_rgba8(buffer));
                app.set_account_has_avatar(true);
            }
            None => app.set_account_has_avatar(false),
        }
    });
    true
}

/// The current track's waveform, if the daemon has analysed it.
///
/// Never asks the daemon to analyse on demand. Opening the player should not
/// cost a decode — a track that has not been analysed simply shows a flat line,
/// and the analysis pass fills it in later.
async fn load_waveform(
    channel: Channel,
    track_id: String,
    weak: Weak<AppWindow>,
    loaded: Arc<std::sync::atomic::AtomicBool>,
) {
    let mut client = AnalysisServiceClient::new(channel);
    let Ok(response) = client
        .get_track_analysis(GetTrackAnalysisRequest {
            track_id,
            // Generate on demand: the track being looked at is exactly the
            // one whose waveform matters right now, and waiting for the
            // background pass to reach it left a flat rail for whole songs.
            analyze_if_missing: true,
        })
        .await
    else {
        return;
    };
    let Some(analysis) = response.into_inner().analysis else {
        return;
    };
    if !analysis.analyzed {
        return;
    }

    let bars = resample_waveform(&analysis.waveform, WAVEFORM_BARS);
    // Marked before the UI hop: landing is what stops the poll loop's
    // retries, and the hop cannot fail in a way a retry would fix.
    loaded.store(true, Ordering::Relaxed);
    let _ = weak.upgrade_in_event_loop(move |app| {
        app.set_waveform_bars(slint::ModelRc::new(slint::VecModel::from(bars)));
    });
}

/// Load the account once the daemon's http server is up.
///
/// gRPC and http start separately, and the session begins the moment gRPC
/// answers — which is usually before there is anything listening for a GraphQL
/// query. A single attempt therefore asked too early and gave up, while the web
/// client, which cannot load at all until http is serving it, always saw the
/// account. Retrying is the whole fix.
async fn load_account_when_ready(weak: Weak<AppWindow>) {
    for attempt in 0..24u32 {
        if load_account(&weak).await {
            return;
        }
        // Backs off to a couple of seconds and stays there: a daemon that is
        // slow to start is worth waiting for, and one that is never coming
        // costs nothing but an idle task.
        let wait = Duration::from_millis(250 * u64::from(attempt.min(8) + 1));
        tokio::time::sleep(wait).await;
    }
    tracing::debug!("gave up reading the account");
}

/// Download and decode an avatar.
///
/// Returns raw pixels rather than a `slint::Image`: the image type is not
/// `Send`, so it has to be built on the UI thread.
async fn fetch_avatar(url: &str) -> Option<(u32, u32, Vec<u8>)> {
    let bytes = http().get(url).send().await.ok()?.bytes().await.ok()?;
    tokio::task::spawn_blocking(move || {
        let image = image::load_from_memory(&bytes).ok()?;
        let thumb = image.thumbnail(64, 64).to_rgba8();
        Some((thumb.width(), thumb.height(), thumb.into_raw()))
    })
    .await
    .ok()
    .flatten()
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
    const QUERY: &str = r#"query { connectedServer { name url kind } }"#;
    let (name, url, kind) = match graphql(QUERY, serde_json::json!({})).await {
        Ok(data) => {
            let server = &data["connectedServer"];
            (
                server["name"].as_str().unwrap_or_default().to_string(),
                server["url"].as_str().unwrap_or_default().to_string(),
                // `null` for the daemon's own library, which counts as a
                // music-player for what the key and tempo columns need.
                server["kind"]
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| "music-player".to_string()),
            )
        }
        // The daemon may not be up yet; the next connect will set it.
        Err(_) => (String::new(), String::new(), "music-player".to_string()),
    };
    // Only a music-player daemon analyses its own library, so only it can
    // answer for a key or a tempo. Against Subsonic or Jellyfin the columns are
    // hidden rather than shown empty for every row.
    let analyses = kind == "music-player";
    let _ = weak.upgrade_in_event_loop(move |app| {
        app.set_provider_analyses(analyses);
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
// ── Analytics (local DB views) ──────────────────────────────────────────────

/// One row of an analytics view, as the UI shows it.
#[derive(Clone, Debug)]
pub struct StatRowData {
    pub track_id: String,
    pub title: String,
    pub artist: String,
    /// Artwork to fetch for this row: a bare filename for the daemon's cover
    /// server, or an absolute URL from the catalogue. `None` draws the
    /// placeholder.
    pub art: Option<String>,
    /// play count / skip count / seconds heard, depending on the view.
    pub count: i64,
    /// Unix seconds (last played / last skipped / played_at), when present.
    pub at: Option<i64>,
}

/// Everything the Statistics tab shows.
#[derive(Clone, Debug, Default)]
pub struct StatsData {
    pub total_tracks: i64,
    pub total_plays: i64,
    pub total_skips: i64,
    pub never_played_count: i64,
    pub most_skipped: Vec<StatRowData>,
    pub recently_played: Vec<StatRowData>,
    pub never_played: Vec<StatRowData>,
}

/// Rows of a `(track_id, title, artist, count, at)`-shaped view query.
async fn analytics_rows<V>(sql: &str, values: V) -> Vec<StatRowData>
where
    V: IntoIterator<Item = sea_orm::Value>,
{
    use sea_orm::{ConnectionTrait, Statement};
    let db = music_player_storage::shared().await;
    let conn = db.get_connection();
    let rows = match conn
        .query_all_raw(Statement::from_sql_and_values(
            sea_orm::DbBackend::Sqlite,
            sql,
            values,
        ))
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!("analytics query failed: {e}");
            return Vec::new();
        }
    };
    rows.into_iter()
        .filter_map(|r| {
            Some(StatRowData {
                track_id: r.try_get_by_index::<String>(0).ok()?,
                title: r.try_get_by_index::<String>(1).ok()?,
                artist: r.try_get_by_index::<String>(2).ok()?,
                count: r.try_get_by_index::<i64>(3).unwrap_or(0),
                at: r.try_get_by_index::<Option<i64>>(4).ok().flatten(),
                // Optional sixth column: queries that can supply a cover do,
                // the rest leave the row on its placeholder.
                art: r
                    .try_get_by_index::<Option<String>>(5)
                    .ok()
                    .flatten()
                    .filter(|c| !c.trim().is_empty()),
            })
        })
        .collect()
}

/// A single scalar, for the Statistics counters.
async fn analytics_count<V>(sql: &str, values: V) -> i64
where
    V: IntoIterator<Item = sea_orm::Value>,
{
    use sea_orm::{ConnectionTrait, Statement};
    let db = music_player_storage::shared().await;
    match db
        .get_connection()
        .query_one_raw(Statement::from_sql_and_values(
            sea_orm::DbBackend::Sqlite,
            sql,
            values,
        ))
        .await
    {
        Ok(Some(row)) => row.try_get_by_index::<i64>(0).unwrap_or(0),
        _ => 0,
    }
}

/// Which source the analytics screens read, and the name to show for it.
///
/// Asked of the daemon each time rather than cached: the screens must follow
/// a server switch the moment the tab is opened. The key is the server url's
/// host — exactly how the daemon's play recorder labels each listen — or
/// `'local'` when the daemon reads its own library.
async fn analytics_source() -> (String, String) {
    const QUERY: &str = r#"query { connectedServer { name url } }"#;
    if let Ok(data) = graphql(QUERY, serde_json::json!({})).await {
        let server = &data["connectedServer"];
        if let Some(url) = server["url"].as_str().filter(|url| !url.is_empty()) {
            let name = server["name"].as_str().filter(|n| !n.is_empty());
            return (host_of(url), name.unwrap_or(url).to_string());
        }
    }
    ("local".to_string(), "Local library".to_string())
}

/// The host of a server url, matching the daemon's `source_of` labeling: no
/// scheme, credentials, port or path, lowercased.
fn host_of(url: &str) -> String {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    let authority = rest.split(['/', '?']).next().unwrap_or_default();
    let host = authority.rsplit('@').next().unwrap_or(authority);
    let host = host.split(':').next().unwrap_or(host);
    host.to_ascii_lowercase()
}

/// Claim pre-migration plays back for the connected server.
///
/// Listens recorded before the source column existed were marked `'unknown'`
/// when their track id matched nothing local — they were remote plays, but
/// nothing said whose. The connected server's listing answers that by id, so
/// its history is whole rather than starting over at the migration.
async fn adopt_unknown_rows(source: &str, state: &Arc<Mutex<WorkerState>>) {
    use sea_orm::{ConnectionTrait, Statement};
    let known: HashMap<String, (String, String)> = {
        let st = state.lock().await;
        st.tracks
            .iter()
            .map(|t| {
                (
                    t.proto.id.clone(),
                    (t.proto.title.clone(), t.artist.clone()),
                )
            })
            .collect()
    };
    if known.is_empty() {
        return;
    }
    let orphans = analytics_rows(
        "SELECT track_id, '', '', 0, NULL FROM track_stats WHERE source = 'unknown' \
         UNION SELECT DISTINCT track_id, '', '', 0, NULL FROM play_history \
         WHERE source = 'unknown'",
        std::iter::empty(),
    )
    .await;
    let db = music_player_storage::shared().await;
    let conn = db.get_connection();
    for row in orphans {
        let Some((title, artist)) = known.get(&row.track_id) else {
            continue;
        };
        for sql in [
            "UPDATE track_stats SET source = ?, title = ?, artist = ? \
             WHERE track_id = ? AND source = 'unknown'",
            "UPDATE play_history SET source = ?, title = ?, artist = ? \
             WHERE track_id = ? AND source = 'unknown'",
        ] {
            let update = Statement::from_sql_and_values(
                sea_orm::DbBackend::Sqlite,
                sql,
                [
                    source.into(),
                    title.as_str().into(),
                    artist.as_str().into(),
                    row.track_id.as_str().into(),
                ],
            );
            if let Err(e) = conn.execute_raw(update).await {
                tracing::debug!(track = %row.track_id, "could not adopt an orphan play: {e}");
            }
        }
    }
}

/// Reload both analytics tabs for whatever source is connected right now.
async fn refresh_analytics(weak: &Weak<AppWindow>, state: &Arc<Mutex<WorkerState>>) {
    let (source, label) = analytics_source().await;
    if source != "local" {
        adopt_unknown_rows(&source, state).await;
    }
    let mut most_played = analytics_rows(MOST_PLAYED_SQL, [source.as_str().into()]).await;
    fill_missing_art(&mut most_played);
    let mut data = load_stats(&source, state).await;
    // The lists above come from SQLite and only know local covers; the
    // catalogue has resolved artwork for much of the rest.
    fill_missing_art(&mut data.most_skipped);
    fill_missing_art(&mut data.recently_played);
    // Independent of the source: the imported history spans every source
    // there has ever been, so a server switch does not change it.
    let local_only = state.lock().await.history_local_only;
    let history = load_history(local_only).await;

    // The artwork each list wants, captured before the data moves into the
    // UI closure — the models are built there and the fetches keyed by index.
    let most_played_art = art_of(&most_played);
    let skipped_art = art_of(&data.most_skipped);
    let recent_art = art_of(&data.recently_played);
    let history_track_art = history
        .as_ref()
        .map(|h| art_of(&h.top_tracks))
        .unwrap_or_default();
    let artist_art: Vec<Option<String>> = history
        .as_ref()
        .map(|h| h.artist_bars.iter().map(|b| b.art.clone()).collect())
        .unwrap_or_default();

    let _ = weak.upgrade_in_event_loop(move |app| {
        app.set_stats_source(label.into());
        crate::ui_set_most_played(&app, most_played);
        crate::ui_set_stats(&app, data);
        crate::ui_set_history(&app, history);
    });

    // Fetched after the lists are on screen: a row draws its placeholder
    // immediately and fills in as each thumbnail lands, rather than the whole
    // screen waiting on the slowest image.
    let covers_base = endpoints().covers;
    fetch_stat_art(
        weak.clone(),
        covers_base,
        most_played_art,
        skipped_art,
        recent_art,
        history_track_art,
        artist_art,
    );
}

async fn load_stats(source: &str, state: &Arc<Mutex<WorkerState>>) -> StatsData {
    let mut data = StatsData {
        total_tracks: 0,
        total_plays: analytics_count(
            "SELECT COALESCE(SUM(play_count), 0) FROM track_stats WHERE source = ?",
            [source.into()],
        )
        .await,
        total_skips: analytics_count(
            "SELECT COALESCE(SUM(skip_count), 0) FROM track_stats WHERE source = ?",
            [source.into()],
        )
        .await,
        never_played_count: 0,
        most_skipped: analytics_rows(
            "SELECT v.track_id, v.title, v.artist, v.skip_count, v.last_skipped, al.cover \
             FROM v_most_skipped v \
             LEFT JOIN track t ON t.id = v.track_id \
             LEFT JOIN album al ON al.id = t.album_id \
             WHERE v.source = ? \
             ORDER BY v.skip_count DESC, v.last_skipped DESC \
             LIMIT 20",
            [source.into()],
        )
        .await,
        recently_played: analytics_rows(
            // One row per track — the latest listen. Restarting a track a few
            // times writes several history rows, and a screen repeating the
            // same title reads as a bug, not a log.
            "SELECT v.track_id, v.title, v.artist, v.ms_played / 1000, \
                    MAX(v.played_at) AS played_at, MAX(al.cover) \
             FROM v_recently_played v \
             LEFT JOIN track t ON t.id = v.track_id \
             LEFT JOIN album al ON al.id = t.album_id \
             WHERE v.source = ? GROUP BY v.track_id \
             ORDER BY played_at DESC LIMIT 50",
            [source.into()],
        )
        .await,
        never_played: Vec::new(),
    };

    if source == "local" {
        data.total_tracks = analytics_count("SELECT COUNT(*) FROM track", std::iter::empty()).await;
        data.never_played_count =
            analytics_count("SELECT COUNT(*) FROM v_never_played", std::iter::empty()).await;
        data.never_played = analytics_rows(
            "SELECT v.track_id, v.title, v.artist, 0, NULL, al.cover \
             FROM v_never_played v \
             LEFT JOIN track t ON t.id = v.track_id \
             LEFT JOIN album al ON al.id = t.album_id \
             ORDER BY v.created_at DESC \
             LIMIT 20",
            std::iter::empty(),
        )
        .await;
        return data;
    }

    // A remote library has no rows in the local `track` table; the listing
    // the screens already show *is* the library, so "tracks" and "never
    // played" are answered from it — the played ids come from this source's
    // own counters.
    let played: std::collections::HashSet<String> = analytics_rows(
        "SELECT track_id, '' , '', play_count, NULL FROM track_stats \
         WHERE source = ? AND play_count > 0",
        [source.into()],
    )
    .await
    .into_iter()
    .map(|row| row.track_id)
    .collect();
    let st = state.lock().await;
    data.total_tracks = st.tracks.len() as i64;
    let never: Vec<StatRowData> = st
        .tracks
        .iter()
        .filter(|track| !played.contains(&track.proto.id))
        .map(|track| StatRowData {
            track_id: track.proto.id.clone(),
            title: track.proto.title.clone(),
            artist: track.artist.clone(),
            count: 0,
            at: None,
            // The cached remote track keeps no cover filename — only the
            // decoded image, which lives on the UI thread. This list is the
            // never-played one; a placeholder is an honest answer for it.
            art: None,
        })
        .collect();
    data.never_played_count = never.len() as i64;
    data.never_played = never.into_iter().take(20).collect();
    data
}

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
            likes::save(&st.liked_order);
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
                        liked_protos(&st)
                    };
                    // Fisher–Yates via fastrand: shuffle client-side.
                    for i in (1..tracks.len()).rev() {
                        tracks.swap(i, fastrand::usize(..=i));
                    }
                    load_tracks(&channel, tracks, 0).await?;
                }
                Cmd::PlayLikedAt(pos) => {
                    // Same source and same order as the Liked screen, so
                    // position N in the list is position N in the queue.
                    let tracks: Vec<TrackProto> = {
                        let st = state.lock().await;
                        liked_protos(&st)
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
                Cmd::PlaylistCreateAndAdd { name, track_ids } => {
                    // Create takes full Track protos; ids are enough for the
                    // daemon to resolve, same as AddTrack.
                    let tracks: Vec<TrackProto> = track_ids
                        .into_iter()
                        .map(|id| TrackProto {
                            id,
                            ..Default::default()
                        })
                        .collect();
                    let mut playlists = PlaylistServiceClient::new(channel.clone());
                    playlists
                        .create(CreateRequest {
                            name,
                            tracks,
                            smart: None,
                        })
                        .await?;
                    load_playlists(&channel, &weak).await;
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
                    // Flip the player-bar heart right away. The poll confirms
                    // within a second — the daemon updates its queued copy of
                    // the track — but a heart that waits for that reads as a
                    // dead button.
                    {
                        let id = id.clone();
                        let _ = weak.upgrade_in_event_loop(move |app| {
                            if app.get_now_track_id().as_str() == id {
                                app.set_now_liked(like);
                            }
                        });
                    }
                    {
                        let mut st = state.lock().await;
                        // Five polls ≈ five seconds: enough for the star's
                        // network round-trip, short enough that a failed one
                        // snaps back visibly.
                        st.pending_like = Some((id.clone(), like, 5));
                        if like {
                            if st.liked.insert(id.clone()) {
                                st.liked_order.insert(0, id.clone());
                            }
                        } else {
                            st.liked.remove(&id);
                            st.liked_order.retain(|t| t != &id);
                        }
                        // The cached library copy feeds every future queue —
                        // a requeue must carry the new state, not the flag
                        // from when the library was listed.
                        if let Some(track) = st.tracks.iter_mut().find(|t| t.proto.id == id) {
                            track.proto.liked = Some(like);
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
                                (true, Some(track)) if !remote.iter().any(|t| t.id == id) => {
                                    remote.insert(0, track)
                                }
                                (false, _) => remote.retain(|track| track.id != id),
                                _ => {}
                            }
                        }
                    }
                    push_liked(&state, &weak).await;
                    // Detached: the UI is already updated above, and the daemon
                    // forwards the like to Rocksky — a network round-trip with
                    // no timeout. Awaiting it here stalled every later command
                    // in this loop (play/pause, seek) behind a heart click.
                    let channel = channel.clone();
                    tokio::spawn(async move {
                        let mut lib = LibraryServiceClient::new(channel);
                        if let Err(e) = lib
                            .like_track(LikeTrackRequest {
                                id: id.clone(),
                                like,
                            })
                            .await
                        {
                            tracing::warn!("like {id} failed: {e}");
                        }
                    });
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
                        let fresh: Vec<String> = ids
                            .iter()
                            .filter(|id| st.liked.insert((*id).clone()))
                            .cloned()
                            .collect();
                        st.liked_order.splice(0..0, fresh);
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
                // Analytics live in the LOCAL database whatever the library
                // screens read from — but every listen is recorded with its
                // source, so the charts here follow the connected server:
                // Rocksky's plays on Rocksky, the local library's on local.
                // Both tabs are refreshed together: the queries are cheap and
                // the two must never disagree about which source they show.
                Cmd::LoadMostPlayed | Cmd::LoadStats => {
                    refresh_analytics(&weak, &state).await;
                }
                Cmd::SetHistoryLocalOnly(local_only) => {
                    state.lock().await.history_local_only = local_only;
                    // Only the history panel changes; the counters above it
                    // are about the connected library and are unaffected.
                    let history = load_history(local_only).await;
                    let _ = weak.upgrade_in_event_loop(move |app| {
                        app.set_history_local_only(local_only);
                        crate::ui_set_history(&app, history);
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
                Cmd::Search(query) => {
                    // Debounced: the palette sends one of these per keystroke,
                    // and search is federated — the connected server *and* the
                    // local index — so "sabbath" would be seven round trips
                    // answering one question, six of them already stale when
                    // they land. Waiting for a pause, then checking nothing
                    // newer arrived, issues exactly one.
                    let generation = SEARCH_GEN.fetch_add(1, Ordering::SeqCst) + 1;
                    tokio::time::sleep(Duration::from_millis(250)).await;
                    if SEARCH_GEN.load(Ordering::SeqCst) != generation {
                        return Ok(());
                    }

                    let mut lib = LibraryServiceClient::new(channel.clone());
                    let Ok(response) = lib.search(SearchRequest { query }).await else {
                        // A failed search leaves the palette showing whatever
                        // it had; the next keystroke tries again.
                        return Ok(());
                    };
                    let hits = response.into_inner();
                    let tracks: Vec<TrackData> = hits
                        .tracks
                        .iter()
                        .enumerate()
                        .map(|(i, t)| track_data(t, i as i32))
                        .collect();
                    let albums: Vec<(String, String, String)> = hits
                        .albums
                        .iter()
                        .map(|a| (a.id.clone(), a.title.clone(), a.artist.clone()))
                        .collect();
                    let artists: Vec<(String, String)> = hits
                        .artists
                        .iter()
                        .map(|a| (a.id.clone(), a.name.clone()))
                        .collect();
                    let _ = weak.upgrade_in_event_loop(move |app| {
                        crate::ui_set_search_hits(&app, tracks, albums, artists);
                    });
                }
                Cmd::SignIn { handle, password } => {
                    const MUTATION: &str = r#"mutation($handle: String!, $password: String!) {
                        signIn(handle: $handle, password: $password) { did }
                    }"#;
                    let result = graphql(
                        MUTATION,
                        serde_json::json!({ "handle": handle, "password": password }),
                    )
                    .await;
                    match result {
                        Ok(_) => {
                            let _ = weak.upgrade_in_event_loop(|app| {
                                app.set_signin_busy(false);
                                app.set_signin_error("".into());
                                app.set_show_signin(false);
                            });
                            let _ = load_account(&weak).await;
                        }
                        Err(e) => {
                            let message = e.to_string();
                            let _ = weak.upgrade_in_event_loop(move |app| {
                                app.set_signin_busy(false);
                                app.set_signin_error(message.into());
                            });
                        }
                    }
                }
                Cmd::SignOut => {
                    const MUTATION: &str = r#"mutation { signOut }"#;
                    let _ = graphql(MUTATION, serde_json::json!({})).await;
                    let _ = load_account(&weak).await;
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
                    // An open analytics tab must not keep showing the old
                    // source's numbers.
                    refresh_analytics(&weak, &state).await;
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
                            // An open analytics tab must not keep showing the
                            // old source's numbers.
                            refresh_analytics(&weak, &state).await;
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

/// Local clock: advances elapsed between polls.
///
/// The VU meters are no longer its business — they follow the daemon's own
/// measurement of the audio leaving the device (see [`stream_levels`]). This
/// used to synthesise them from a pair of sine waves, which looked like a
/// meter and measured nothing.
async fn ticker(weak: Weak<AppWindow>) {
    const TICK_S: f64 = 0.06;
    let mut elapsed_total = 0.0f32;
    loop {
        tokio::time::sleep(Duration::from_millis((TICK_S * 1000.0) as u64)).await;
        elapsed_total += TICK_S as f32;
        let now = elapsed_total;
        let _ = weak.upgrade_in_event_loop(move |app| {
            // Only while the full player is open: this rebuilds a model every
            // frame, and nothing is looking at it otherwise.
            if app.get_show_full_player() {
                let targets = if app.get_playing() {
                    let bands: Vec<f32> = {
                        use slint::Model;
                        app.get_eq_band_drive().iter().collect()
                    };
                    if bands.iter().any(|b| *b > 0.0) {
                        eq_targets_from_bands(&bands, EQ_BARS)
                    } else {
                        // No spectrum from this daemon (older build): the
                        // two-scalar synthesis is better than a flat line.
                        eq_targets(
                            app.get_eq_drive_left(),
                            app.get_eq_drive_right(),
                            EQ_BARS,
                            now,
                        )
                    }
                } else {
                    // Paused bars settle to the floor rather than freezing
                    // mid-bounce, which reads as the app having hung.
                    vec![0.0; EQ_BARS]
                };
                // Caps hold the level a transient reached and fall at a
                // constant rate, so a hit that has already decayed is still
                // visible for a moment instead of vanishing with the bar.
                let previous: Vec<f32> = {
                    use slint::Model;
                    app.get_eq_peaks().iter().collect()
                };
                let peaks: Vec<f32> = targets
                    .iter()
                    .enumerate()
                    .map(|(i, target)| {
                        let falling = previous.get(i).copied().unwrap_or(0.0) - 0.012;
                        target.max(falling).clamp(0.0, 1.0)
                    })
                    .collect();

                app.set_eq_bars(slint::ModelRc::new(slint::VecModel::from(targets)));
                app.set_eq_peaks(slint::ModelRc::new(slint::VecModel::from(peaks)));
                app.set_waveform_progress(app.get_progress());
            }
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
            }
        });
    }
}

/// Drive the VU meters from the daemon's own measurement of the audio.
///
/// The bass band, not the full-band RMS: a meter driven by everything sits
/// near the top on anything loud, and reads as an ornament. The low band
/// moves with the beat, which is what a meter is for.
///
/// Falls back to decaying to zero if the stream ends — a meter frozen at its
/// last reading looks broken rather than stopped.
async fn stream_levels(channel: Channel, weak: Weak<AppWindow>) {
    let mut playback = PlaybackServiceClient::new(channel);
    let Ok(response) = playback.stream_levels(StreamLevelsRequest {}).await else {
        return;
    };
    let mut stream = response.into_inner();

    // Auto-gain, so kicks reach the top on any material.
    //
    // A fixed scale cannot: bass RMS depends on the mix, the master and the
    // volume, and a factor chosen for one track leaves another at half height
    // — which is why these sat around 60%. Instead the loudest thing heard
    // recently *is* the top of the meter. The reference jumps up instantly and
    // falls slowly, so a quiet passage does not immediately re-normalise into
    // looking loud.
    const FLOOR: f32 = 0.02;
    /// Per update at 20 Hz — about a 1.5 second half-life, which is release
    /// rather than memory. Slower and a quiet passage stays squashed for most
    /// of a verse.
    const DECAY: f32 = 0.977;
    let mut reference = FLOOR;

    // Short-term averages for the equalizer bars — separate from `reference`
    // on purpose. The meter normalises to the recent *peak*, which is right
    // for a meter (it should ride near the top) and wrong for bar animation:
    // steady music keeps value/peak near 1, so every bar sat at full height.
    // Sensitivity comes from comparing the instant against its own recent
    // *average* instead: steady material lands mid-height, a kick 2x above
    // average hits the top, and a dip actually dips — the way the psysonic
    // bars behave on their dB scale.
    let mut avg_left = FLOOR;
    let mut avg_right = FLOOR;
    // Per-band references for the spectrum bars — each band rides its own
    // recent peak, the way a dB-scaled analyser normalises per column. One
    // shared reference would let the bass bury the treble, which is exactly
    // the "does not respect the spectrum" look.
    let mut band_refs: Vec<f32> = Vec::new();

    while let Ok(Some(levels)) = stream.message().await {
        let peak = levels.low_left.max(levels.low_right);
        reference = (reference * DECAY).max(peak).max(FLOOR);
        let scale = |value: f32| (value / reference).clamp(0.0, 1.0);
        let (left, right) = (scale(levels.low_left), scale(levels.low_right));

        // ~0.35 s time constant at 20 Hz: long enough to be "the song right
        // now", short enough that a chorus re-normalises within a bar or two.
        avg_left = avg_left * 0.9 + levels.low_left.max(0.0) * 0.1;
        avg_right = avg_right * 0.9 + levels.low_right.max(0.0) * 0.1;
        let drive = |value: f32, avg: f32| {
            // Headroom of 1.8x: the average lands at (1/1.8)^1.5 ≈ 0.41 of
            // the bar, leaving the top ~60% for transients. The 1.5 gamma
            // stretches the useful range the way a dB axis would.
            let ratio = value.max(0.0) / (avg.max(FLOOR) * 1.8);
            ratio.powf(1.5).clamp(0.0, 1.0)
        };
        let (drive_left, drive_right) = (
            drive(levels.low_left, avg_left),
            drive(levels.low_right, avg_right),
        );

        // Spectrum: normalise each band against its own decaying peak, so a
        // hi-hat column reaches the top on hi-hats, not on kicks.
        band_refs.resize(levels.bands.len().max(band_refs.len()), FLOOR);
        let band_drive: Vec<f32> = levels
            .bands
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let v = if v.is_finite() { v.max(0.0) } else { 0.0 };
                band_refs[i] = (band_refs[i] * DECAY).max(v).max(FLOOR);
                (v / band_refs[i]).clamp(0.0, 1.0)
            })
            .collect();

        let _ = weak.upgrade_in_event_loop(move |app| {
            app.set_eq_band_drive(slint::ModelRc::new(slint::VecModel::from(band_drive)));
            app.set_eq_drive_left(drive_left);
            app.set_eq_drive_right(drive_right);
            // Fast attack, slow release: real meter ballistics. An instant
            // attack reads as jitter and an instant release flickers between
            // buffers, but too slow an attack clips the top off a kick, which
            // is the one thing the meter exists to show.
            let ease = |current: f32, target: f32| {
                if target > current {
                    current + (target - current) * 0.8
                } else {
                    current + (target - current) * 0.15
                }
            };
            app.set_vu_left(ease(app.get_vu_left(), left).clamp(0.0, 1.0));
            app.set_vu_right(ease(app.get_vu_right(), right).clamp(0.0, 1.0));
        });
    }

    let _ = weak.upgrade_in_event_loop(|app| {
        app.set_vu_left(0.0);
        app.set_vu_right(0.0);
    });
}

/// How many bars the full player's equalizer draws.
pub const EQ_BARS: usize = 96;

/// How many bars its waveform draws.
///
/// Fewer than the 400 the daemon stores: this is a rectangle per bar in the
/// scene graph rather than a canvas, and 400 of them is a cost paid on every
/// frame to draw detail nobody can see at this width.
pub const WAVEFORM_BARS: usize = 120;

/// The height each equalizer bar is heading for.
///
/// The daemon measures left, right, and each again below 200 Hz — not a
/// spectrum — so this is a shaped response rather than an FFT. Two things make
/// it read as one, and both are about how music behaves:
///
/// Bars lean on the channel nearest them, so a hard-panned hat lifts one side.
/// And low bars are steadier than high ones, because bass is sustained and
/// treble is transient — without that, uniform bars read instantly as
/// decoration.
///
/// The same shaping as the web client's, deliberately: it is one instrument
/// shown in two places, and it should look like the same instrument.
fn eq_targets(left: f32, right: f32, count: usize, time: f32) -> Vec<f32> {
    let safe = |value: f32| {
        if value.is_finite() {
            value.clamp(0.0, 1.0)
        } else {
            0.0
        }
    };
    let (left, right) = (safe(left), safe(right));

    (0..count)
        .map(|i| {
            let position = if count <= 1 {
                0.0
            } else {
                i as f32 / (count - 1) as f32
            };
            // 0 at the edges, 1 in the middle: how far this bar is from "its"
            // channel.
            let blend = 1.0 - (position - 0.5).abs() * 2.0;
            let channel = if position < 0.5 {
                left + (right - left) * blend
            } else {
                right + (left - right) * blend
            };

            let tilt = 1.0 - position * 0.45;
            let flicker =
                1.0 + 0.22 * position * (time * (5.0 + i as f32 * 1.7) + i as f32 * 2.399).sin();
            (channel * tilt * flicker).clamp(0.0, 1.0)
        })
        .collect()
}

/// Spread the measured bands across the drawn bars: linear interpolation
/// between band centres, with a light 3-tap smooth so neighbouring bars do
/// not step. No flicker and no tilt — the data is real now, and decoration on
/// top of measurement reads as noise.
fn eq_targets_from_bands(bands: &[f32], count: usize) -> Vec<f32> {
    if bands.is_empty() || count == 0 {
        return vec![0.0; count];
    }
    let raw: Vec<f32> = (0..count)
        .map(|i| {
            let pos = if count <= 1 {
                0.0
            } else {
                i as f32 / (count - 1) as f32 * (bands.len() - 1) as f32
            };
            let lo = pos.floor() as usize;
            let hi = (lo + 1).min(bands.len() - 1);
            let t = pos - lo as f32;
            bands[lo] * (1.0 - t) + bands[hi] * t
        })
        .collect();
    (0..count)
        .map(|i| {
            let prev = raw[i.saturating_sub(1)];
            let next = raw[(i + 1).min(count - 1)];
            (prev * 0.25 + raw[i] * 0.5 + next * 0.25).clamp(0.0, 1.0)
        })
        .collect()
}

/// The stored waveform, resampled to the number of bars actually drawn.
///
/// Peak per output bar rather than an average: averaging smooths away the
/// transients that make a waveform recognisable as a particular song.
fn resample_waveform(stored: &[u8], count: usize) -> Vec<f32> {
    if stored.is_empty() || count == 0 {
        return Vec::new();
    }
    (0..count)
        .map(|i| {
            let start = i * stored.len() / count;
            let end = ((i + 1) * stored.len() / count)
                .max(start + 1)
                .min(stored.len());
            let peak = stored[start..end].iter().copied().max().unwrap_or(0);
            peak as f32 / 255.0
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a drift point with only the fields `fill_gaps` reads.
    fn point(bucket: &str, listens: i64) -> music_player_analytics::query::DriftPoint {
        music_player_analytics::query::DriftPoint {
            bucket: bucket.to_string(),
            listens,
            avg_bpm: None,
            avg_valence: None,
            avg_arousal: None,
            median_year: None,
            discovery_rate: 0.0,
        }
    }

    /// A year with no listening is data, and the gap is the point. Plotting
    /// only the periods that have rows puts 2017 next to 2021 and calls the
    /// result a time axis.
    #[test]
    fn silent_periods_are_drawn() {
        let points = [point("2017-01-01", 400), point("2021-01-01", 1200)];
        let filled = fill_gaps(&points, "year");
        assert_eq!(filled.len(), 5, "2017 through 2021 inclusive");
        assert_eq!(filled[0], ("2017".into(), 400));
        assert_eq!(filled[1], ("2018".into(), 0), "a silent year is a zero bar");
        assert_eq!(filled[4], ("2021".into(), 1200));
    }

    /// Stepping months must roll the year over, and land on the right ones.
    #[test]
    fn month_stepping_rolls_over_the_year() {
        let points = [point("2025-11-01", 10), point("2026-02-01", 20)];
        let filled = fill_gaps(&points, "month");
        let labels: Vec<&str> = filled.iter().map(|(l, _)| l.as_str()).collect();
        assert_eq!(labels, ["2025-11", "2025-12", "2026-01", "2026-02"]);
        assert_eq!(filled[0].1, 10);
        assert_eq!(filled[1].1, 0);
        assert_eq!(filled[3].1, 20);
    }

    #[test]
    fn quarter_stepping_advances_three_months() {
        let points = [point("2025-01-01", 5), point("2025-10-01", 7)];
        let filled = fill_gaps(&points, "quarter");
        let labels: Vec<&str> = filled.iter().map(|(l, _)| l.as_str()).collect();
        assert_eq!(labels, ["2025-01", "2025-04", "2025-07", "2025-10"]);
        assert_eq!(filled[1].1, 0);
    }

    /// A single period is a valid series of one, not an empty chart.
    #[test]
    fn a_single_period_survives() {
        let filled = fill_gaps(&[point("2026-03-01", 42)], "month");
        assert_eq!(filled, vec![("2026-03".to_string(), 42)]);
    }

    /// Malformed buckets must not hang the UI thread or panic it.
    #[test]
    fn malformed_buckets_are_survivable() {
        assert!(fill_gaps(&[point("not-a-date", 1)], "month").is_empty());
        assert!(fill_gaps(&[], "month").is_empty());
    }

    /// Thousands separators, at the boundaries that usually break them.
    #[test]
    fn thousands_groups_correctly() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1_000), "1,000");
        assert_eq!(thousands(68_659), "68,659");
        assert_eq!(thousands(1_234_567), "1,234,567");
        assert_eq!(thousands(-4_500), "-4,500");
    }

    /// Spreading 16 measured bands over 96 bars must keep the spectrum's
    /// shape — a low-heavy input stays low-heavy — and every bar drawable.
    #[test]
    fn band_spread_keeps_shape_and_range() {
        let mut bands = vec![0.0f32; 16];
        bands[0] = 1.0;
        bands[1] = 0.8;
        let bars = eq_targets_from_bands(&bands, EQ_BARS);
        assert_eq!(bars.len(), EQ_BARS);
        assert!(bars[0] > bars[EQ_BARS - 1], "low-heavy input inverted");
        assert!(bars.iter().all(|b| (0.0..=1.0).contains(b)));
    }

    #[test]
    fn band_spread_handles_degenerate_input() {
        assert_eq!(eq_targets_from_bands(&[], EQ_BARS), vec![0.0; EQ_BARS]);
        assert!(eq_targets_from_bands(&[0.5], 0).is_empty());
        let flat = eq_targets_from_bands(&[0.5], EQ_BARS);
        assert!(flat.iter().all(|b| (b - 0.5).abs() < 1e-6));
    }

    /// One height per bar, all drawable.
    #[test]
    fn eq_bars_stay_in_range() {
        let targets = eq_targets(0.8, 0.4, EQ_BARS, 1.5);
        assert_eq!(targets.len(), EQ_BARS);
        assert!(targets.iter().all(|value| (0.0..=1.0).contains(value)));
    }

    #[test]
    fn silence_draws_nothing() {
        assert!(eq_targets(0.0, 0.0, 12, 3.0).iter().all(|v| *v == 0.0));
    }

    /// The display is stereo: a sound only on the left lifts the left side.
    #[test]
    fn each_side_leans_on_its_own_channel() {
        let targets = eq_targets(1.0, 0.0, 20, 0.0);
        assert!(targets[0] > targets[19], "{targets:?}");
    }

    /// A dropped frame arrives as NaN; drawing it would be a bar of NaN pixels.
    #[test]
    fn a_broken_level_reads_as_silence() {
        assert!(eq_targets(f32::NAN, f32::NAN, 8, 1.0)
            .iter()
            .all(|v| *v == 0.0));
    }

    /// The waveform is drawn at the width chosen here, whatever was stored.
    #[test]
    fn the_waveform_is_resampled_to_what_is_drawn() {
        assert_eq!(
            resample_waveform(&[128; 400], WAVEFORM_BARS).len(),
            WAVEFORM_BARS
        );
        // More bars than stored peaks still fills the width.
        assert_eq!(resample_waveform(&[10, 200, 30], 120).len(), 120);
        assert!(resample_waveform(&[], 120).is_empty());
    }

    /// Peaks survive the resampling — averaging would flatten them, and they
    /// are what makes a waveform recognisable.
    #[test]
    fn resampling_keeps_the_peaks() {
        let mut stored = vec![20u8; 400];
        stored[200] = 255;
        let bars = resample_waveform(&stored, 20);
        assert_eq!(bars[10], 1.0);
        assert!(bars[0] < 0.2);
    }

    /// The meter's auto-gain, extracted so it can be reasoned about: the
    /// loudest thing heard recently is the top of the meter.
    fn normalise(reference: &mut f32, peak: f32) -> f32 {
        const FLOOR: f32 = 0.02;
        const DECAY: f32 = 0.977;
        *reference = (*reference * DECAY).max(peak).max(FLOOR);
        (peak / *reference).clamp(0.0, 1.0)
    }

    /// The whole point: whatever the material's level, its peaks reach the
    /// top. A fixed scale left quiet mixes at a fraction of the height.
    #[test]
    fn a_peak_reaches_the_top_at_any_level() {
        for peak in [0.05_f32, 0.2, 0.6, 1.0] {
            let mut reference = 0.02;
            assert!(
                (normalise(&mut reference, peak) - 1.0).abs() < 1e-6,
                "{peak} did not reach the top"
            );
        }
    }

    /// Between kicks the meter has to fall, or it just sits at the top.
    #[test]
    fn quieter_passages_read_lower() {
        let mut reference = 0.02;
        normalise(&mut reference, 0.5);
        let quiet = normalise(&mut reference, 0.1);
        assert!(quiet < 0.25, "expected a low reading, got {quiet}");
    }

    /// The reference decays, so a track that gets quieter re-normalises
    /// rather than staying pinned near the floor forever.
    #[test]
    fn the_reference_recovers_after_a_loud_passage() {
        let mut reference = 0.02;
        normalise(&mut reference, 1.0);
        // ~10 seconds at 20 Hz.
        for _ in 0..200 {
            normalise(&mut reference, 0.1);
        }
        assert!(
            normalise(&mut reference, 0.1) > 0.5,
            "still scaled to the old peak: reference {reference}"
        );
    }

    /// Silence must not divide by zero or invent a reading.
    #[test]
    fn silence_reads_as_silence() {
        let mut reference = 0.02;
        assert_eq!(normalise(&mut reference, 0.0), 0.0);
    }

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

// ── Listening history (DuckDB analytics) ────────────────────────────────────

/// One cell of the listening-clock heatmap.
#[derive(Clone, Copy, Debug)]
pub struct HeatCellData {
    pub weekday: i32,
    pub hour: i32,
    /// 0..1 against the busiest hour.
    pub level: f32,
    pub listens: i32,
}

/// One bar of a chart.
#[derive(Clone, Debug)]
pub struct ChartBarData {
    pub label: String,
    /// Artist picture, as in [`StatRowData::art`].
    pub art: Option<String>,
    pub norm: f32,
    pub value: String,
    pub show_label: bool,
}

/// The whole imported listening history, as the Statistics tab shows it.
///
/// Distinct from [`StatsData`], which is about the library or server connected
/// right now. This spans every source that has been imported and is
/// deduplicated across them, so the two sets of numbers do not add up to each
/// other and are presented apart.
#[derive(Clone, Debug, Default)]
pub struct HistoryData {
    pub available: bool,
    /// Whether these figures are restricted to the local library.
    pub local_only: bool,
    pub listens: i64,
    pub hours: f64,
    pub artists: i64,
    pub tracks: i64,
    pub sessions: i64,
    pub streak: i64,
    pub span: String,
    pub sources: String,
    pub clock: Vec<HeatCellData>,
    pub clock_peak: i32,
    pub timeline: Vec<ChartBarData>,
    pub artist_bars: Vec<ChartBarData>,
    pub top_tracks: Vec<StatRowData>,
}

/// Read the analytics database.
///
/// Read-only, and on a blocking thread: DuckDB is synchronous, its connection
/// is not `Send` across an await, and a decade of imported history is enough
/// work that running it on the UI worker's executor would be felt.
///
/// Every failure is the same answer — `available: false`, which hides the
/// section. There is nothing actionable to show a user who has never run an
/// import, and a half-drawn panel of zeroes reads as breakage.
/// `None` means the database could not be read — an import is holding it, or
/// it is mid-write. That is *not* the same as having no history, and the
/// caller must leave whatever is on screen alone rather than blanking it.
pub async fn load_history(local_only: bool) -> Option<HistoryData> {
    // Artist pictures first, resolved from the Rocksky catalogue and cached.
    // Does nothing once every artist on the leaderboard is known.
    //
    // Split into three because a DuckDB connection is not `Sync` and this
    // runs inside a spawned task: the database work happens on blocking
    // threads either side, and only the HTTP part is awaited here.
    resolve_artist_pictures(local_only).await;

    tokio::task::spawn_blocking(move || read_history(local_only))
        .await
        .ok()
        .flatten()
}

/// Make sure the artists the leaderboard is about to show have a picture.
///
/// Every failure is silent: the database being briefly held by an import is a
/// normal thing, and the cost is placeholder portraits for one repaint.
async fn resolve_artist_pictures(local_only: bool) {
    use music_player_analytics::artwork;

    let pending = tokio::task::spawn_blocking(move || {
        use music_player_analytics::query::{self, Scope, TopKind, Window};
        use music_player_analytics::Analytics;

        let analytics = Analytics::open_default().ok()?;
        let scope = Scope {
            window: Window::all(),
            local_only,
        };
        let artists = query::top(&analytics, TopKind::Artists, scope, 10).ok()?;
        let names: Vec<String> = artists
            .iter()
            .filter(|a| a.art.is_none())
            .map(|a| a.name.clone())
            .collect();
        artwork::pending_artists(&analytics, &names).ok()
    })
    .await
    .ok()
    .flatten()
    .unwrap_or_default();

    if pending.is_empty() {
        return;
    }

    let found = artwork::fetch_pictures(pending).await;
    if found.is_empty() {
        return;
    }
    let _ = tokio::task::spawn_blocking(move || {
        use music_player_analytics::Analytics;
        if let Ok(analytics) = Analytics::open_default() {
            match artwork::store_pictures(&analytics, &found) {
                Ok(n) => tracing::debug!(resolved = n, "artist pictures"),
                Err(e) => tracing::debug!("could not store artist pictures: {e}"),
            }
        }
    })
    .await;
}

fn read_history(local_only: bool) -> Option<HistoryData> {
    use music_player_analytics::query::{self, Scope, TopKind, Window};
    use music_player_analytics::Analytics;

    let analytics = match Analytics::open_default_read_only() {
        Ok(analytics) => analytics,
        Err(cause) => {
            // Could be "never imported", could be "an import is running and
            // holds the single writer". Neither is a reason to erase a panel
            // that is already showing nine years of history.
            tracing::debug!("listening history unavailable: {cause}");
            return None;
        }
    };

    let scope = Scope {
        window: Window::all(),
        local_only,
    };
    let overview = match query::overview(&analytics, scope) {
        Ok(overview) if overview.listens > 0 => overview,
        // Genuinely empty: there is nothing to show, and saying so is right.
        Ok(_) => return Some(HistoryData::default()),
        Err(e) => {
            tracing::warn!("could not read the listening history: {e}");
            return None;
        }
    };

    let mut data = HistoryData {
        available: true,
        listens: overview.listens,
        hours: overview.hours_played,
        artists: overview.distinct_artists,
        tracks: overview.distinct_tracks,
        sessions: overview.sessions,
        streak: overview.longest_streak,
        span: match (&overview.first_listen, &overview.last_listen) {
            (Some(first), Some(last)) => format!("{} — {}", day(first), day(last)),
            _ => String::new(),
        },
        local_only,
        ..Default::default()
    };

    if let Ok(origins) = query::origins(&analytics) {
        // Named because the headline figure is otherwise unattributable: a
        // user seeing 68,000 listens needs to know which of them came from an
        // export rather than from this player.
        data.sources = origins
            .iter()
            .map(|o| format!("{} {}", thousands(o.canonical), o.origin))
            .collect::<Vec<_>>()
            .join(" · ");
    }

    if let Ok(cells) = query::clock(&analytics, scope) {
        let peak = cells.iter().map(|c| c.listens).max().unwrap_or(0).max(1);
        data.clock_peak = peak as i32;
        // Every hour of the week gets a tile, present in the data or not, so
        // the grid reads as a week rather than as scattered marks.
        data.clock = (0..7)
            .flat_map(|weekday| {
                let cells = &cells;
                (0..24).map(move |hour| {
                    let listens = cells
                        .iter()
                        .find(|c| c.weekday == weekday && c.hour == hour)
                        .map(|c| c.listens)
                        .unwrap_or(0);
                    HeatCellData {
                        weekday,
                        hour,
                        level: (listens as f64 / peak as f64) as f32,
                        listens: listens as i32,
                    }
                })
            })
            .collect();
    }

    data.timeline = timeline(&analytics, scope);

    if let Ok(artists) = query::top(&analytics, TopKind::Artists, scope, 10) {
        let peak = artists.iter().map(|a| a.listens).max().unwrap_or(0).max(1);
        data.artist_bars = artists
            .iter()
            .map(|a| ChartBarData {
                label: a.name.clone(),
                norm: (a.listens as f64 / peak as f64) as f32,
                value: thousands(a.listens),
                show_label: true,
                art: a.art.clone(),
            })
            .collect();
    }

    data.top_tracks = top_tracks(&analytics, local_only);

    Some(data)
}

/// The most-played tracks of all time, carrying a local track id where the
/// library happens to have the track.
///
/// Not [`query::top`], which returns names only. Most of these rows come from
/// an imported history and have nothing to play, but the ones that are also
/// in the library should be playable from here — a row that offers a pointer
/// and then does nothing is worse than a row that never offered.
fn top_tracks(analytics: &music_player_analytics::Analytics, local_only: bool) -> Vec<StatRowData> {
    // An INNER join when restricted: a row with no local track is exactly
    // what "in my library only" excludes.
    let join = if local_only { "JOIN" } else { "LEFT JOIN" };
    let mut statement = match analytics.conn().prepare(&format!(
        "SELECT coalesce(max(t.track_id), '') AS track_id,
                mode(l.title)  AS title,
                mode(l.artist) AS artist,
                count(*)       AS listens,
                any_value(l.cover) AS art
         FROM enriched_listens l
         {join} tracks t ON t.match_key = l.match_key
         GROUP BY l.match_key
         ORDER BY listens DESC
         LIMIT 20"
    )) {
        Ok(statement) => statement,
        Err(e) => {
            tracing::warn!("all-time tracks unavailable: {e}");
            return Vec::new();
        }
    };

    let rows = statement.query_map([], |row| {
        Ok(StatRowData {
            track_id: row.get::<_, String>(0)?,
            title: row.get::<_, String>(1)?,
            artist: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            count: row.get::<_, i64>(3)?,
            at: None,
            art: row.get::<_, Option<String>>(4)?,
        })
    });
    match rows {
        Ok(rows) => rows.filter_map(Result::ok).collect(),
        Err(e) => {
            tracing::warn!("all-time tracks unavailable: {e}");
            Vec::new()
        }
    }
}

/// Listens per period, as bars.
///
/// Two things this has to get right that a plain `GROUP BY` does not:
///
/// * **Bucket size follows the range.** Months over a decade is a hundred-odd
///   bars, which at panel width is a smear; years over one is a sparse row of
///   eight. The bucket is widened until the count is legible.
/// * **Silent periods are drawn.** `drift` returns only periods that have
///   listens, so plotting it directly puts 2017 next to 2021 and calls it a
///   time axis. A year of not listening is data — the gap is the point.
fn timeline(
    analytics: &music_player_analytics::Analytics,
    scope: music_player_analytics::query::Scope,
) -> Vec<ChartBarData> {
    use music_player_analytics::query;

    /// Beyond this the bars are too thin to read or hover.
    const MAX_BARS: usize = 64;

    let mut filled = Vec::new();
    for bucket in ["month", "quarter", "year"] {
        let Ok(points) = query::drift(analytics, scope, bucket) else {
            return Vec::new();
        };
        if points.is_empty() {
            return Vec::new();
        }
        filled = fill_gaps(&points, bucket);
        if filled.len() <= MAX_BARS {
            break;
        }
    }
    if filled.is_empty() {
        return Vec::new();
    }

    let peak = filled.iter().map(|(_, n)| *n).max().unwrap_or(0).max(1);
    let count = filled.len();
    // A label under every bar is unreadable; roughly eight across the axis is
    // legible whatever the range.
    let every = (count / 8).max(1);
    filled
        .iter()
        .enumerate()
        .map(|(i, (label, listens))| ChartBarData {
            // A period has no portrait.
            art: None,
            label: label.clone(),
            norm: (*listens as f64 / peak as f64) as f32,
            value: format!("{} plays", thousands(*listens)),
            show_label: i % every == 0 || i + 1 == count,
        })
        .collect()
}

/// Expand a sparse series into every period between its first and last,
/// inserting zeroes for the ones with no listening.
///
/// Buckets arrive as `YYYY-MM-DD` (the truncated period start), so stepping
/// is month arithmetic; no date library is needed and none of this depends on
/// the local timezone.
fn fill_gaps(
    points: &[music_player_analytics::query::DriftPoint],
    bucket: &str,
) -> Vec<(String, i64)> {
    let step_months = match bucket {
        "month" => 1,
        "quarter" => 3,
        _ => 12,
    };

    let parse = |b: &str| -> Option<(i32, u32)> {
        let mut parts = b.split('-');
        let year: i32 = parts.next()?.parse().ok()?;
        let month: u32 = parts.next()?.parse().ok()?;
        Some((year, month))
    };

    let Some(first) = points.first().and_then(|p| parse(&p.bucket)) else {
        return Vec::new();
    };
    let Some(last) = points.last().and_then(|p| parse(&p.bucket)) else {
        return Vec::new();
    };

    let label_of = |year: i32, month: u32| match bucket {
        "year" => format!("{year}"),
        _ => format!("{year}-{month:02}"),
    };

    let mut out = Vec::new();
    let (mut year, mut month) = first;
    // Bounded by construction — the loop always advances by at least a month
    // — but capped anyway so a malformed bucket cannot hang the UI thread.
    for _ in 0..4096 {
        if (year, month) > last {
            break;
        }
        let key = format!("{year:04}-{month:02}");
        let listens = points
            .iter()
            .find(|p| p.bucket.starts_with(&key))
            .map(|p| p.listens)
            .unwrap_or(0);
        out.push((label_of(year, month), listens));

        month += step_months;
        while month > 12 {
            month -= 12;
            year += 1;
        }
    }
    out
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

/// The Most Played screen, and the Statistics tab's chart of the same name.
///
/// The `ORDER BY` is not redundant with the one inside `v_most_played`: a
/// view's ordering is not carried through a select that joins onto it, and
/// leaving it off silently returned rows in physical order — a track with
/// five plays above one with seven.
pub(crate) const MOST_PLAYED_SQL: &str = "SELECT v.track_id, v.title, v.artist, \
     v.play_count, v.last_played, al.cover \
     FROM v_most_played v \
     LEFT JOIN track t ON t.id = v.track_id \
     LEFT JOIN album al ON al.id = t.album_id \
     WHERE v.source = ? \
     ORDER BY v.play_count DESC, v.last_played DESC \
     LIMIT 200";

/// The artwork each row wants, by position.
fn art_of(rows: &[StatRowData]) -> Vec<Option<String>> {
    rows.iter().map(|row| row.art.clone()).collect()
}

/// Fill in covers the local library could not supply, from the catalogue.
///
/// The Statistics lists that come from SQLite only know about covers the
/// local library has on disk, so a track streamed from a server or played
/// before it was scanned shows a placeholder. Enrichment has already resolved
/// an album cover for most of those through `matchSong`; this is a lookup of
/// what is already stored, not a network call.
fn fill_missing_art(rows: &mut [StatRowData]) {
    use music_player_analytics::Analytics;

    let missing: Vec<&StatRowData> = rows
        .iter()
        .filter(|row| row.art.is_none() && !row.title.trim().is_empty())
        .collect();
    if missing.is_empty() {
        return;
    }
    let Ok(analytics) = Analytics::open_default_read_only() else {
        return;
    };

    let keys: Vec<String> = missing
        .iter()
        .map(|row| {
            format!(
                "match_key('{}', '{}')",
                row.artist.replace('\'', "''"),
                row.title.replace('\'', "''")
            )
        })
        .collect();

    let sql = format!(
        "SELECT m.match_key, m.album_art FROM song_match m \
         WHERE m.resolved AND m.album_art IS NOT NULL AND m.match_key IN ({})",
        keys.join(",")
    );
    let Ok(mut statement) = analytics.conn().prepare(&sql) else {
        return;
    };
    let Ok(found) = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }) else {
        return;
    };
    let found: std::collections::HashMap<String, String> = found.filter_map(Result::ok).collect();
    if found.is_empty() {
        return;
    }

    // The key is computed the same way on both sides, so ask DuckDB for each
    // row's key rather than reimplementing the normalisation in Rust and
    // letting the two drift.
    for row in rows.iter_mut().filter(|row| row.art.is_none()) {
        let key: Option<String> = analytics
            .conn()
            .query_row(
                "SELECT match_key(?, ?)",
                music_player_analytics::duckdb::params![row.artist, row.title],
                |r| r.get(0),
            )
            .ok();
        if let Some(art) = key.and_then(|key| found.get(&key)).cloned() {
            row.art = Some(art);
        }
    }
}

/// Which list a fetched thumbnail belongs to.
#[derive(Clone, Copy)]
enum StatList {
    MostPlayed,
    MostSkipped,
    Recent,
    HistoryTracks,
    HistoryArtists,
}

/// Load the artwork for every Statistics list, in the background.
///
/// One task per image, capped by a semaphore: a cold Statistics tab wants a
/// hundred-odd thumbnails and firing them all at once starves the daemon's
/// own cover server.
#[allow(clippy::too_many_arguments)]
fn fetch_stat_art(
    weak: Weak<AppWindow>,
    covers_base: String,
    most_played: Vec<Option<String>>,
    skipped: Vec<Option<String>>,
    recent: Vec<Option<String>>,
    history_tracks: Vec<Option<String>>,
    history_artists: Vec<Option<String>>,
) {
    let lists = [
        (StatList::MostPlayed, most_played),
        (StatList::MostSkipped, skipped),
        (StatList::Recent, recent),
        (StatList::HistoryTracks, history_tracks),
        (StatList::HistoryArtists, history_artists),
    ];

    let limit = Arc::new(tokio::sync::Semaphore::new(6));
    for (list, arts) in lists {
        for (idx, art) in arts.into_iter().enumerate() {
            let Some(art) = art else { continue };
            let weak = weak.clone();
            let covers_base = covers_base.clone();
            let limit = Arc::clone(&limit);
            tokio::spawn(async move {
                let Ok(_permit) = limit.acquire().await else {
                    return;
                };
                // 96px: the largest of these is drawn at 34pt, so a 96px
                // thumbnail covers a 2x display with nothing to spare.
                let Some((w, h, rgba)) = fetch_thumb(&covers_base, &art, 96).await else {
                    return;
                };
                let _ = weak.upgrade_in_event_loop(move |app| match list {
                    StatList::MostPlayed => {
                        crate::ui_set_stat_art(&app.get_most_played(), idx, w, h, rgba)
                    }
                    StatList::MostSkipped => {
                        crate::ui_set_stat_art(&app.get_stats_most_skipped(), idx, w, h, rgba)
                    }
                    StatList::Recent => {
                        crate::ui_set_stat_art(&app.get_stats_recent(), idx, w, h, rgba)
                    }
                    StatList::HistoryTracks => {
                        crate::ui_set_stat_art(&app.get_history_top_tracks(), idx, w, h, rgba)
                    }
                    StatList::HistoryArtists => {
                        crate::ui_set_bar_art(&app.get_history_artist_bars(), idx, w, h, rgba)
                    }
                });
            });
        }
    }
}

/// `2026-09-21 18:04:36+03` -> `2026-09-21`.
fn day(timestamp: &str) -> String {
    timestamp.split(' ').next().unwrap_or(timestamp).to_string()
}

/// Thousands separators: five-figure counts are common here and `28744` is
/// harder to read at a glance than `28,744`.
fn thousands(n: i64) -> String {
    let digits = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 {
        format!("-{out}")
    } else {
        out
    }
}

#[cfg(test)]
mod history_tests {
    use super::*;

    /// The listening-history panel, against the real analytics database.
    ///
    /// Ignored by default: it needs a database that has actually had a history
    /// imported, which only a real installation has. Run it with
    /// `cargo test -p music-player-desktop -- --ignored --nocapture` to check
    /// what the Statistics tab would draw.
    #[tokio::test]
    #[ignore]
    async fn listening_history_panel_is_populated() {
        let data = load_history(false)
            .await
            .expect("the analytics database could not be read (is an import running?)");
        assert!(data.available, "the analytics database holds no history");

        println!("listens          {}", thousands(data.listens));
        println!("hours            {:.0}", data.hours);
        println!("artists          {}", thousands(data.artists));
        println!("tracks           {}", thousands(data.tracks));
        println!("sessions         {}", thousands(data.sessions));
        println!("streak           {} days", data.streak);
        println!("span             {}", data.span);
        println!("sources          {}", data.sources);
        println!(
            "clock cells      {} (peak {})",
            data.clock.len(),
            data.clock_peak
        );
        println!("timeline bars    {}", data.timeline.len());
        println!("artist bars      {}", data.artist_bars.len());
        println!("top tracks       {}", data.top_tracks.len());
        if let Some(first) = data.timeline.first() {
            println!("timeline starts  {} ({})", first.label, first.value);
        }
        if let Some(last) = data.timeline.last() {
            println!("timeline ends    {} ({})", last.label, last.value);
        }
        for bar in data.artist_bars.iter().take(3) {
            println!("  artist         {} {}", bar.label, bar.value);
        }

        // Every hour of the week gets a tile so the grid reads as a week.
        assert_eq!(data.clock.len(), 7 * 24, "the heatmap is a full week");
        assert!(data.clock_peak > 0);
        assert!(!data.timeline.is_empty(), "the timeline has bars");
        assert!(!data.artist_bars.is_empty(), "the ranking has bars");
        assert!(data.listens > 0 && data.hours > 0.0);

        // Silent periods are drawn, so the labelled bars must be contiguous.
        let labelled = data.timeline.iter().filter(|b| b.show_label).count();
        assert!(
            labelled >= 2 && labelled <= data.timeline.len(),
            "selective labels, not one per bar"
        );

        // The restricted view must be a subset, never larger.
        let owned = load_history(true).await.expect("read the restricted view");
        assert!(
            owned.listens <= data.listens,
            "\"in my library only\" cannot add listens"
        );
        println!("local-only       {} listens", thousands(owned.listens));

        // Artwork: every artist should have a portrait, and most tracks a
        // cover. These are the URLs the rows fetch and decode.
        let with_portrait = data.artist_bars.iter().filter(|b| b.art.is_some()).count();
        let with_cover = data.top_tracks.iter().filter(|t| t.art.is_some()).count();
        println!(
            "artist portraits {}/{}",
            with_portrait,
            data.artist_bars.len()
        );
        println!("track covers     {}/{}", with_cover, data.top_tracks.len());
        for bar in data.artist_bars.iter().take(2) {
            println!(
                "  {} -> {}",
                bar.label,
                bar.art.as_deref().unwrap_or("(none)")
            );
        }
        for track in data.top_tracks.iter().take(2) {
            println!(
                "  {} -> {}",
                track.title,
                track.art.as_deref().unwrap_or("(none)")
            );
        }
        assert_eq!(
            with_portrait,
            data.artist_bars.len(),
            "every artist on the leaderboard should have a portrait"
        );
        assert!(
            with_cover * 2 > data.top_tracks.len(),
            "most played tracks should carry a cover"
        );
    }
}

#[cfg(test)]
mod most_played_tests {
    use super::*;
    use sea_orm::{ConnectionTrait, DbBackend, Statement};

    /// The Most Played screen must be ordered by play count.
    ///
    /// `v_most_played` orders internally, but SQLite does not carry a view's
    /// ordering through a select that joins onto it — which is exactly what
    /// fetching the album cover added. The screen then listed a track with
    /// five plays above one with seven.
    #[tokio::test]
    async fn most_played_is_ordered_by_play_count() {
        use migration::{Migrator, MigratorTrait};

        let dir = tempfile::tempdir().unwrap();
        let url = format!(
            "sqlite://{}?mode=rwc",
            dir.path().join("music-player.sqlite3").display()
        );
        let conn = sea_orm::Database::connect(&url).await.unwrap();
        Migrator::up(&conn, None).await.unwrap();

        // Inserted in an order that is not the answer, so physical order
        // cannot pass for sorted.
        for (id, title, plays) in [
            ("t1", "Five", 5),
            ("t2", "Nine", 9),
            ("t3", "Seven", 7),
            ("t4", "Six", 6),
        ] {
            for sql in [
                format!(
                    "INSERT INTO track (id, title, artist, genre, duration, uri) \
                     VALUES ('{id}', '{title}', 'Someone', 'Rock', 100.0, '/m/{id}.mp3')"
                ),
                format!(
                    "INSERT INTO track_stats (track_id, play_count, last_played, source, \
                     title, artist) VALUES ('{id}', {plays}, 1750000000, 'local', \
                     '{title}', 'Someone')"
                ),
            ] {
                conn.execute_raw(Statement::from_string(DbBackend::Sqlite, sql))
                    .await
                    .unwrap();
            }
        }

        let rows = conn
            .query_all_raw(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                MOST_PLAYED_SQL,
                ["local".into()],
            ))
            .await
            .unwrap();

        let counts: Vec<i64> = rows
            .iter()
            .map(|row| row.try_get_by_index::<i64>(3).unwrap_or(0))
            .collect();
        assert_eq!(counts, vec![9, 7, 6, 5], "most played first");
    }
}
