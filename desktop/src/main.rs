//! music-player-desktop — skinnable Slint client for the music-player daemon.
//!
//! Talks to a running daemon over gRPC (127.0.0.1:5051 by default; override
//! with MUSIC_PLAYER_HOST / MUSIC_PLAYER_PORT / MUSIC_PLAYER_HTTP_PORT).
//! When nothing is listening the full daemon is booted in-process
//! (src/daemon.rs). All library state lives on the UI thread in a
//! thread_local; the tokio worker in rpc.rs pushes plain data across via
//! upgrade_in_event_loop.

mod daemon;
mod likes;
mod rpc;
mod servers;
mod skin;

use std::cell::RefCell;
use std::rc::Rc;

use slint::{ComponentHandle, Model, ModelRc, SharedPixelBuffer, VecModel};

slint::include_modules!();

struct AlbumEntry {
    data: rpc::AlbumData,
    image: Option<slint::Image>,
}

struct ArtistEntry {
    data: rpc::ArtistData,
    image: Option<slint::Image>,
}

#[derive(Default)]
struct UiState {
    albums: Vec<AlbumEntry>,
    artists: Vec<ArtistEntry>,
    tracks: Vec<rpc::TrackData>,
    liked: Vec<rpc::TrackData>,
    servers: Vec<servers::SavedServer>,
    /// Browse navigation stack of (title, path); top = current level.
    browse_stack: Vec<(String, String)>,
    playlists: Vec<rpc::PlaylistData>,
    discovered: Vec<(String, String, u16)>,
    switcher_query: String,
    active_server: String,
    pl_detail_id: Option<String>,
    pl_detail_track_ids: Vec<String>,
    picker_query: String,
}

thread_local! {
    static STATE: RefCell<UiState> = RefCell::new(UiState::default());
}

fn track_item_with(t: &rpc::TrackData, liked_ids: &std::collections::HashSet<String>) -> TrackItem {
    TrackItem {
        id: t.id.clone().into(),
        title: t.title.clone().into(),
        artist: t.artist.clone().into(),
        album: t.album.clone().into(),
        duration: rpc::format_time(t.length_ms as f64 / 1000.0).into(),
        index: t.index,
        track_no: if t.track_no > 0 {
            t.track_no
        } else {
            t.index + 1
        },
        liked: liked_ids.contains(&t.id),
        album_id: t.album_id.clone().into(),
    }
}

fn liked_ids_of(liked: &[rpc::TrackData]) -> std::collections::HashSet<String> {
    liked.iter().map(|t| t.id.clone()).collect()
}

fn artist_item(a: &ArtistEntry) -> ArtistItem {
    ArtistItem {
        id: a.data.id.clone().into(),
        name: a.data.name.clone().into(),
        art: a.image.clone().unwrap_or_default(),
        has_art: a.image.is_some(),
    }
}

fn album_item(a: &AlbumEntry) -> AlbumItem {
    AlbumItem {
        id: a.data.id.clone().into(),
        title: a.data.title.clone().into(),
        artist: a.data.artist.clone().into(),
        year: a.data.year.clone().into(),
        art: a.image.clone().unwrap_or_default(),
        has_art: a.image.is_some(),
    }
}

/// Called from the rpc worker (on the UI thread) when the library loads.
pub fn ui_set_library(app: &AppWindow, data: rpc::LibraryData) {
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.albums = data
            .albums
            .into_iter()
            .map(|a| AlbumEntry {
                data: a,
                image: None,
            })
            .collect();
        st.artists = data
            .artists
            .into_iter()
            .map(|a| ArtistEntry {
                data: a,
                image: None,
            })
            .collect();
        st.tracks = data.tracks;
        st.liked = data.liked;

        let albums: Vec<AlbumItem> = st.albums.iter().map(album_item).collect();
        let artists: Vec<ArtistItem> = st.artists.iter().map(artist_item).collect();
        let ids = liked_ids_of(&st.liked);
        let tracks: Vec<TrackItem> = st.tracks.iter().map(|t| track_item_with(t, &ids)).collect();
        let liked: Vec<TrackItem> = st.liked.iter().map(|t| track_item_with(t, &ids)).collect();

        app.set_albums(ModelRc::new(VecModel::from(albums)));
        app.set_artists(ModelRc::new(VecModel::from(artists)));
        app.set_tracks(ModelRc::new(VecModel::from(tracks)));
        app.set_liked(ModelRc::new(VecModel::from(liked)));
    });
}

/// Called per decoded album-art thumbnail.
pub fn ui_set_album_art(app: &AppWindow, idx: usize, w: u32, h: u32, rgba: Vec<u8>) {
    let image = slint::Image::from_rgba8(SharedPixelBuffer::clone_from_slice(&rgba, w, h));
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        if let Some(entry) = st.albums.get_mut(idx) {
            entry.image = Some(image.clone());
        }
    });
    let model = app.get_albums();
    if let Some(mut row) = model.row_data(idx) {
        row.art = image.clone();
        row.has_art = true;
        model.set_row_data(idx, row);
    }
    // Keep an open detail view in sync with the freshly loaded art.
    let mut detail = app.get_detail_album();
    let matches = STATE.with(|s| {
        s.borrow()
            .albums
            .get(idx)
            .map(|a| a.data.id == detail.id.as_str())
            .unwrap_or(false)
    });
    if matches {
        detail.art = image;
        detail.has_art = true;
        app.set_detail_album(detail);
    }
}

/// Called per fetched artist picture (Rocksky CDN).
pub fn ui_set_artist_art(app: &AppWindow, idx: usize, w: u32, h: u32, rgba: Vec<u8>) {
    let image = slint::Image::from_rgba8(SharedPixelBuffer::clone_from_slice(&rgba, w, h));
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        if let Some(entry) = st.artists.get_mut(idx) {
            entry.image = Some(image.clone());
        }
    });
    let model = app.get_artists();
    if let Some(mut row) = model.row_data(idx) {
        row.art = image;
        row.has_art = true;
        model.set_row_data(idx, row);
    }
}

pub fn ui_set_now_art(app: &AppWindow, w: u32, h: u32, rgba: Vec<u8>) {
    let image = slint::Image::from_rgba8(SharedPixelBuffer::clone_from_slice(&rgba, w, h));
    app.set_now_art(image);
    app.set_now_has_art(true);
}

pub fn ui_set_queue(
    app: &AppWindow,
    total: usize,
    upnext: Vec<rpc::TrackData>,
    history: Vec<rpc::TrackData>,
) {
    let ids = STATE.with(|s| liked_ids_of(&s.borrow().liked));
    let upnext: Vec<TrackItem> = upnext.iter().map(|t| track_item_with(t, &ids)).collect();
    let history: Vec<TrackItem> = history.iter().map(|t| track_item_with(t, &ids)).collect();
    app.set_queue_total(total as i32);
    app.set_queue_upnext(ModelRc::new(VecModel::from(upnext)));
    app.set_queue_history(ModelRc::new(VecModel::from(history)));
}

pub fn ui_show_album_detail(app: &AppWindow, detail: rpc::AlbumDetailData) {
    let (image, has_art) = STATE.with(|s| {
        s.borrow()
            .albums
            .iter()
            .find(|a| a.data.id == detail.id)
            .and_then(|a| a.image.clone())
            .map(|img| (img, true))
            .unwrap_or_default()
    });
    let total_s: u64 = detail.tracks.iter().map(|t| t.length_ms / 1000).sum();
    let duration = if total_s >= 3600 {
        format!("{} hr {} min", total_s / 3600, (total_s % 3600) / 60)
    } else {
        format!("{} min", (total_s / 60).max(1))
    };
    let mut meta = String::new();
    if !detail.year.is_empty() {
        meta.push_str(&detail.year);
        meta.push_str(" · ");
    }
    meta.push_str(&format!("{} tracks · {duration}", detail.tracks.len()));

    let ids = STATE.with(|s| liked_ids_of(&s.borrow().liked));
    // Group by disc when the album spans more than one.
    let discs: std::collections::BTreeSet<i32> =
        detail.tracks.iter().map(|t| t.disc.max(1)).collect();
    let mut rows: Vec<DetailRow> = Vec::new();
    if discs.len() > 1 {
        for disc in discs {
            rows.push(DetailRow {
                is_header: true,
                header: format!("DISC {disc}").into(),
                track: TrackItem::default(),
            });
            rows.extend(
                detail
                    .tracks
                    .iter()
                    .filter(|t| t.disc.max(1) == disc)
                    .map(|t| DetailRow {
                        is_header: false,
                        header: "".into(),
                        track: track_item_with(t, &ids),
                    }),
            );
        }
    } else {
        rows.extend(detail.tracks.iter().map(|t| DetailRow {
            is_header: false,
            header: "".into(),
            track: track_item_with(t, &ids),
        }));
    }
    app.set_detail_album(AlbumItem {
        id: detail.id.into(),
        title: detail.title.into(),
        artist: detail.artist.into(),
        year: detail.year.into(),
        art: image,
        has_art,
    });
    app.set_detail_meta(meta.into());
    app.set_detail_label(detail.label.into());
    app.set_detail_tracks(ModelRc::new(VecModel::from(rows)));
    app.set_show_detail(true);
}

// ── Remote servers / browsing ───────────────────────────────────────────────

fn server_item(s: &servers::SavedServer) -> ServerItem {
    ServerItem {
        kind: s.kind.clone().into(),
        name: s.name.clone().into(),
        url: s.url.clone().into(),
    }
}

fn refresh_servers_model(app: &AppWindow) {
    let items: Vec<ServerItem> =
        STATE.with(|s| s.borrow().servers.iter().map(server_item).collect());
    app.set_servers(ModelRc::new(VecModel::from(items)));
}

/// Called from the rpc worker when a browse level has been fetched.
pub fn ui_browse_opened(
    app: &AppWindow,
    title: String,
    path: String,
    entries: Vec<rpc::BrowseEntryData>,
    push: bool,
) {
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        if push {
            st.browse_stack.push((title, path));
        }
        let crumbs: Vec<&str> = st.browse_stack.iter().map(|(t, _)| t.as_str()).collect();
        app.set_browse_title(crumbs.join("  ›  ").into());
    });
    let items: Vec<BrowseItem> = entries
        .iter()
        .map(|e| BrowseItem {
            name: e.name.clone().into(),
            path: e.path.clone().into(),
            is_dir: e.is_dir,
        })
        .collect();
    app.set_browse_entries(ModelRc::new(VecModel::from(items)));
    app.set_browse_loading(false);
    app.set_browse_error("".into());
    app.set_browsing(true);
}

/// Called from the rpc worker with the daemon's audio settings snapshot.
pub fn ui_set_audio_settings(app: &AppWindow, a: rpc::AudioSettingsData) {
    app.set_bass(a.bass);
    app.set_bass_min(a.bass_min);
    app.set_bass_max(a.bass_max);
    app.set_treble(a.treble);
    app.set_treble_min(a.treble_min);
    app.set_treble_max(a.treble_max);
    app.set_balance(a.balance);
    app.set_eq_enabled(a.eq_enabled);
    app.set_eq_precut(a.eq_precut);
    app.set_rg_type(a.rg_type);
    app.set_rg_preamp(a.rg_preamp);
    app.set_rg_noclip(a.rg_noclip);
    app.set_crossfade(a.crossfade);
    app.set_fade_in_delay(a.fade_in_delay);
    app.set_fade_in_duration(a.fade_in_duration);
    app.set_fade_out_delay(a.fade_out_delay);
    app.set_fade_out_duration(a.fade_out_duration);
    app.set_fade_out_mixmode(a.fade_out_mixmode);
    app.set_dithering(a.dithering);
    let bands: Vec<EqBand> = a
        .eq_bands
        .iter()
        .map(|&(cutoff, q, gain)| EqBand { cutoff, q, gain })
        .collect();
    app.set_eq_bands(ModelRc::new(VecModel::from(bands)));
}

// ── Playlists ───────────────────────────────────────────────────────────────

fn playlist_item(p: &rpc::PlaylistData) -> PlaylistItem {
    PlaylistItem {
        id: p.id.clone().into(),
        name: p.name.clone().into(),
        description: p.description.clone().into(),
        count: p.track_count as i32,
    }
}

pub fn ui_set_playlists(app: &AppWindow, data: Vec<rpc::PlaylistData>) {
    let items: Vec<PlaylistItem> = data.iter().map(playlist_item).collect();
    STATE.with(|s| s.borrow_mut().playlists = data);
    app.set_playlists(ModelRc::new(VecModel::from(items)));
    // Refresh an open detail header (name/description/count may have changed).
    let id = app.get_pl_detail().id.to_string();
    if !id.is_empty() {
        if let Some(p) = STATE.with(|s| s.borrow().playlists.iter().find(|p| p.id == id).cloned()) {
            app.set_pl_detail(playlist_item(&p));
        }
    }
}

/// Called from the rpc worker with a playlist's track ids.
pub fn ui_show_playlist(app: &AppWindow, id: String, track_ids: Vec<String>, open_picker: bool) {
    let tracks: Vec<TrackItem> = STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.pl_detail_id = Some(id.clone());
        st.pl_detail_track_ids = track_ids.clone();
        let ids = liked_ids_of(&st.liked);
        track_ids
            .iter()
            .filter_map(|tid| st.tracks.iter().find(|t| &t.id == tid))
            .map(|t| track_item_with(t, &ids))
            .collect()
    });
    if let Some(p) = STATE.with(|s| s.borrow().playlists.iter().find(|p| p.id == id).cloned()) {
        app.set_pl_detail(playlist_item(&p));
    }
    app.set_pl_detail_tracks(ModelRc::new(VecModel::from(tracks)));
    app.set_pl_detail_open(true);
    app.set_current_tab(5);
    if open_picker {
        app.invoke_open_track_picker();
    } else if app.get_show_track_picker() {
        // Keep the picker results in sync after an add.
        let query = STATE.with(|s| s.borrow().picker_query.clone());
        app.set_picker_results(ModelRc::new(VecModel::from(picker_results(&query))));
    }
}

/// Track-only palette results for the add-to-playlist picker; tracks already
/// in the playlist are hidden.
fn picker_results(query: &str) -> Vec<PaletteItem> {
    let q = query.trim().to_lowercase();
    STATE.with(|s| {
        let st = s.borrow();
        st.tracks
            .iter()
            .filter(|t| !st.pl_detail_track_ids.contains(&t.id))
            .filter(|t| {
                q.is_empty()
                    || [&t.title, &t.artist, &t.album]
                        .iter()
                        .any(|f| f.to_lowercase().contains(&q))
            })
            .take(12)
            .map(|t| {
                let art = st
                    .albums
                    .iter()
                    .find(|a| a.data.title == t.album && a.data.artist == t.artist)
                    .and_then(|a| a.image.clone());
                PaletteItem {
                    kind: "track".into(),
                    id: t.id.clone().into(),
                    title: t.title.clone().into(),
                    subtitle: format!("{} · {}", t.artist, t.album).into(),
                    index: t.index,
                    has_art: art.is_some(),
                    art: art.unwrap_or_default(),
                }
            })
            .collect()
    })
}

/// Called after a like/unlike: refresh the liked list + hearts everywhere.
pub fn ui_set_liked(app: &AppWindow, liked: Vec<rpc::TrackData>) {
    STATE.with(|s| s.borrow_mut().liked = liked);
    STATE.with(|s| {
        let st = s.borrow();
        let ids = liked_ids_of(&st.liked);
        let tracks: Vec<TrackItem> = st.tracks.iter().map(|t| track_item_with(t, &ids)).collect();
        let liked_items: Vec<TrackItem> =
            st.liked.iter().map(|t| track_item_with(t, &ids)).collect();
        app.set_tracks(ModelRc::new(VecModel::from(tracks)));
        app.set_liked(ModelRc::new(VecModel::from(liked_items)));
        app.set_now_liked(ids.contains(app.get_now_track_id().as_str()));
    });
}

pub fn is_liked(id: &str) -> bool {
    STATE.with(|s| s.borrow().liked.iter().any(|t| t.id == id))
}

// ── Server switcher ─────────────────────────────────────────────────────────

pub fn ui_set_discovered(app: &AppWindow, found: Vec<(String, String, u16)>) {
    STATE.with(|s| s.borrow_mut().discovered = found);
    let q = STATE.with(|s| s.borrow().switcher_query.clone());
    app.set_switcher_results(ModelRc::new(VecModel::from(switcher_results(&q))));
}

fn server_row(
    kind: &str,
    title: String,
    mut subtitle: String,
    id: String,
    active: &str,
) -> PaletteItem {
    if subtitle == active || id == active {
        subtitle = format!("{subtitle} · connected");
    }
    PaletteItem {
        kind: kind.into(),
        id: id.into(),
        title: title.into(),
        subtitle: subtitle.into(),
        index: -1,
        has_art: false,
        art: slint::Image::default(),
    }
}

fn switcher_results(query: &str) -> Vec<PaletteItem> {
    let q = query.trim().to_lowercase();
    let hit =
        |fields: &[&str]| q.is_empty() || fields.iter().any(|f| f.to_lowercase().contains(&q));
    STATE.with(|s| {
        let st = s.borrow();
        let active = st.active_server.clone();
        let mut out: Vec<PaletteItem> = Vec::new();

        if hit(&["this machine", "localhost", "127.0.0.1:5051", "embedded"]) {
            out.push(server_row(
                "server",
                "This machine (embedded)".into(),
                "127.0.0.1:5051".into(),
                "127.0.0.1:5051".into(),
                &active,
            ));
        }
        for (name, host, port) in &st.discovered {
            if host == "127.0.0.1" {
                continue;
            }
            let addr = format!("{host}:{port}");
            if hit(&[name, &addr]) {
                out.push(server_row(
                    "server",
                    name.clone(),
                    addr.clone(),
                    addr,
                    &active,
                ));
            }
        }
        for (i, srv) in st.servers.iter().enumerate() {
            if hit(&[&srv.name, &srv.url]) {
                out.push(server_row(
                    if srv.kind == "jellyfin" {
                        "jellyfin"
                    } else {
                        "subsonic"
                    },
                    srv.name.clone(),
                    srv.url.clone(),
                    format!("srv:{i}"),
                    &active,
                ));
            }
        }
        // Free-form "connect to host[:port]" when the query looks like one.
        let raw = query.trim();
        if (raw.contains('.') || raw.contains(':')) && !out.iter().any(|r| r.id.as_str() == raw) {
            out.push(server_row(
                "server",
                format!("Connect to {raw}"),
                "remote music-player".into(),
                raw.to_string(),
                &active,
            ));
        }
        out
    })
}

fn palette_results(query: &str) -> Vec<PaletteItem> {
    let q = query.trim().to_lowercase();
    let hit =
        |fields: &[&str]| q.is_empty() || fields.iter().any(|f| f.to_lowercase().contains(&q));
    STATE.with(|s| {
        let st = s.borrow();
        let mut out: Vec<PaletteItem> = Vec::new();
        out.extend(
            st.tracks
                .iter()
                .filter(|t| hit(&[&t.title, &t.artist, &t.album]))
                .take(8)
                .map(|t| {
                    // Reuse the album grid's thumbnail for the track's album.
                    let art = st
                        .albums
                        .iter()
                        .find(|a| a.data.title == t.album && a.data.artist == t.artist)
                        .and_then(|a| a.image.clone());
                    PaletteItem {
                        kind: "track".into(),
                        id: t.id.clone().into(),
                        title: t.title.clone().into(),
                        subtitle: format!("{} · {}", t.artist, t.album).into(),
                        index: t.index,
                        has_art: art.is_some(),
                        art: art.unwrap_or_default(),
                    }
                }),
        );
        out.extend(
            st.albums
                .iter()
                .filter(|a| hit(&[&a.data.title, &a.data.artist]))
                .take(5)
                .map(|a| PaletteItem {
                    kind: "album".into(),
                    id: a.data.id.clone().into(),
                    title: a.data.title.clone().into(),
                    subtitle: a.data.artist.clone().into(),
                    index: -1,
                    has_art: a.image.is_some(),
                    art: a.image.clone().unwrap_or_default(),
                }),
        );
        out.extend(
            st.playlists
                .iter()
                .filter(|p| hit(&[&p.name, &p.description]))
                .take(4)
                .map(|p| PaletteItem {
                    kind: "playlist".into(),
                    id: p.id.clone().into(),
                    title: p.name.clone().into(),
                    subtitle: if p.track_count == 1 {
                        "1 track".into()
                    } else {
                        format!("{} tracks", p.track_count).into()
                    },
                    index: -1,
                    has_art: false,
                    art: slint::Image::default(),
                }),
        );
        out.extend(
            st.artists
                .iter()
                .filter(|a| hit(&[&a.data.name]))
                .take(4)
                .map(|a| PaletteItem {
                    kind: "artist".into(),
                    id: a.data.id.clone().into(),
                    title: a.data.name.clone().into(),
                    subtitle: "".into(),
                    index: -1,
                    has_art: a.image.is_some(),
                    art: a.image.clone().unwrap_or_default(),
                }),
        );
        out
    })
}

/// macOS Now Playing integration: publishes the current track + playback
/// state to the system Now Playing center (Control Center, media keys,
/// AirPods controls) and routes remote commands back into the app. Returns
/// the repeating update timer — keep it alive for the app's lifetime.
#[cfg(target_os = "macos")]
fn setup_media_controls(
    app: &AppWindow,
    tx: tokio::sync::mpsc::UnboundedSender<rpc::Cmd>,
) -> Option<slint::Timer> {
    use souvlaki::{
        MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition,
        PlatformConfig, SeekDirection,
    };
    use std::time::Duration;

    let mut controls = match MediaControls::new(PlatformConfig {
        dbus_name: "music_player_desktop",
        display_name: "Music Player",
        hwnd: None,
    }) {
        Ok(controls) => controls,
        Err(e) => {
            tracing::warn!("Now Playing integration disabled: {e:?}");
            return None;
        }
    };

    {
        let tx = tx.clone();
        let app_weak = app.as_weak();
        if let Err(e) = controls.attach(move |event| {
            let elapsed_ms = app_weak
                .upgrade()
                .map(|app| (app.get_elapsed_s() * 1000.0) as u32)
                .unwrap_or(0);
            let cmd = match event {
                MediaControlEvent::Play | MediaControlEvent::Pause | MediaControlEvent::Toggle => {
                    Some(rpc::Cmd::PlayPause)
                }
                MediaControlEvent::Next => Some(rpc::Cmd::Next),
                MediaControlEvent::Previous => Some(rpc::Cmd::Previous),
                MediaControlEvent::SetPosition(MediaPosition(pos)) => {
                    Some(rpc::Cmd::SeekMs(pos.as_millis() as u32))
                }
                MediaControlEvent::Seek(direction) => Some(rpc::Cmd::SeekMs(match direction {
                    SeekDirection::Forward => elapsed_ms.saturating_add(5_000),
                    SeekDirection::Backward => elapsed_ms.saturating_sub(5_000),
                })),
                MediaControlEvent::SeekBy(direction, by) => {
                    let by = by.as_millis() as u32;
                    Some(rpc::Cmd::SeekMs(match direction {
                        SeekDirection::Forward => elapsed_ms.saturating_add(by),
                        SeekDirection::Backward => elapsed_ms.saturating_sub(by),
                    }))
                }
                _ => None,
            };
            if let Some(cmd) = cmd {
                let _ = tx.send(cmd);
            }
        }) {
            tracing::warn!("Now Playing integration disabled: {e:?}");
            return None;
        }
    }

    // Mirror the UI's now-playing state into the system center once a second.
    let app_weak = app.as_weak();
    let controls = std::cell::RefCell::new(controls);
    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(1000),
        move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let mut controls = controls.borrow_mut();
            if app.get_stopped() {
                let _ = controls.set_playback(MediaPlayback::Stopped);
                return;
            }
            let _ = controls.set_metadata(MediaMetadata {
                title: Some(&app.get_now_title()),
                artist: Some(&app.get_now_artist()),
                album: None,
                duration: Some(Duration::from_secs_f32(app.get_length_s().max(0.0))),
                cover_url: None,
            });
            let progress = Some(MediaPosition(Duration::from_secs_f32(
                app.get_elapsed_s().max(0.0),
            )));
            let _ = controls.set_playback(if app.get_playing() {
                MediaPlayback::Playing { progress }
            } else {
                MediaPlayback::Paused { progress }
            });
        },
    );
    Some(timer)
}

/// On macOS: hide the titlebar strip but keep the native traffic-light
/// buttons floating over the sidebar (transparent titlebar + full-size
/// content view). Must run before the first window is created.
fn setup_backend() {
    #[cfg(target_os = "macos")]
    {
        let backend = i_slint_backend_winit::Backend::builder()
            .with_window_attributes_hook(|attrs| {
                use winit::platform::macos::WindowAttributesExtMacOS;
                attrs
                    .with_titlebar_transparent(true)
                    .with_fullsize_content_view(true)
                    .with_title_hidden(true)
            })
            .build()
            .expect("winit backend");
        slint::platform::set_platform(Box::new(backend)).expect("set slint platform");
    }
}

fn main() -> Result<(), slint::PlatformError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .init();

    setup_backend();
    let app = AppWindow::new()?;
    #[cfg(target_os = "macos")]
    app.set_titlebar_inset(24.0);

    // ── Skins ───────────────────────────────────────────────────────────────
    let skins = Rc::new(skin::load_all());
    let current = Rc::new(RefCell::new(skin::load_selection(&skins)));
    if let Some(s) = skins.get(*current.borrow()) {
        skin::apply(s, &app);
    }
    {
        let app_weak = app.as_weak();
        let skins = skins.clone();
        let current = current.clone();
        app.on_cycle_skin(move || {
            let app = app_weak.unwrap();
            let mut idx = current.borrow_mut();
            *idx = (*idx + 1) % skins.len().max(1);
            if let Some(s) = skins.get(*idx) {
                skin::apply(s, &app);
                skin::save_selection(&s.name);
            }
        });
    }

    // ── Embedded daemon (if no music-player daemon is listening) ────────────
    if !daemon::is_running() {
        app.set_status_text("starting music-player…".into());
    }
    std::thread::spawn(|| {
        daemon::ensure_running();
    });

    // ── gRPC worker + commands ──────────────────────────────────────────────
    // The worker retries until the (embedded or external) daemon answers.
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<rpc::Cmd>();
    rpc::start(app.as_weak(), rx);

    // macOS Now Playing center (media keys, Control Center, AirPods).
    #[cfg(target_os = "macos")]
    let _media_controls_timer = setup_media_controls(&app, tx.clone());

    {
        let tx = tx.clone();
        app.on_play_pause(move || {
            let _ = tx.send(rpc::Cmd::PlayPause);
        });
    }
    {
        let tx = tx.clone();
        app.on_next(move || {
            let _ = tx.send(rpc::Cmd::Next);
        });
    }
    {
        let tx = tx.clone();
        app.on_previous(move || {
            let _ = tx.send(rpc::Cmd::Previous);
        });
    }
    {
        let tx = tx.clone();
        let app_weak = app.as_weak();
        app.on_seek(move |pct| {
            let app = app_weak.unwrap();
            let length_ms = (app.get_length_s() as f64 * 1000.0) as u32;
            if length_ms > 0 {
                let ms = (length_ms as f32 * pct.clamp(0.0, 1.0)) as u32;
                // Optimistic local update so the bar doesn't snap back while
                // the daemon catches up.
                app.set_elapsed_s(ms as f32 / 1000.0);
                app.set_progress(pct);
                let _ = tx.send(rpc::Cmd::SeekMs(ms));
            }
        });
    }
    {
        let tx = tx.clone();
        app.on_set_volume(move |pct| {
            let _ = tx.send(rpc::Cmd::SetVolume(pct));
        });
    }
    {
        let tx = tx.clone();
        app.on_play_album(move |id| {
            let _ = tx.send(rpc::Cmd::PlayAlbum(id.into()));
        });
    }
    {
        let tx = tx.clone();
        app.on_open_album(move |id| {
            let _ = tx.send(rpc::Cmd::OpenAlbum(id.into()));
        });
    }
    {
        let tx = tx.clone();
        app.on_play_album_track(move |id, pos| {
            let _ = tx.send(rpc::Cmd::PlayAlbumAt(id.into(), pos));
        });
    }
    {
        let tx = tx.clone();
        app.on_play_artist(move |id| {
            let _ = tx.send(rpc::Cmd::PlayArtist(id.into()));
        });
    }
    {
        let tx = tx.clone();
        app.on_play_track_at(move |i| {
            let _ = tx.send(rpc::Cmd::PlayAllAt(i));
        });
    }
    {
        let tx = tx.clone();
        app.on_play_liked_at(move |i| {
            let _ = tx.send(rpc::Cmd::PlayLikedAt(i));
        });
    }
    {
        let tx = tx.clone();
        app.on_play_queue_at(move |i| {
            let _ = tx.send(rpc::Cmd::QueueJump(i));
        });
    }
    {
        let tx = tx.clone();
        app.on_queue_clear(move || {
            let _ = tx.send(rpc::Cmd::QueueClear);
        });
    }
    {
        let tx = tx.clone();
        app.on_queue_remove(move |i| {
            let _ = tx.send(rpc::Cmd::QueueRemove(i));
        });
    }
    {
        let tx = tx.clone();
        let app_weak = app.as_weak();
        app.on_audio_set(move |key, value| {
            let app = app_weak.unwrap();
            // Optimistic: the daemon has no push channel, so mirror the
            // change locally right away.
            match key.as_str() {
                "eq_enabled" => app.set_eq_enabled(value != 0),
                "eq_precut" => app.set_eq_precut(value),
                "bass" => app.set_bass(value),
                "treble" => app.set_treble(value),
                "balance" => app.set_balance(value),
                "rg_type" => app.set_rg_type(value),
                "rg_preamp" => app.set_rg_preamp(value),
                "rg_noclip" => app.set_rg_noclip(value != 0),
                "crossfade" => app.set_crossfade(value),
                "fade_in_delay" => app.set_fade_in_delay(value),
                "fade_in_duration" => app.set_fade_in_duration(value),
                "fade_out_delay" => app.set_fade_out_delay(value),
                "fade_out_duration" => app.set_fade_out_duration(value),
                "fade_out_mixmode" => app.set_fade_out_mixmode(value),
                "dithering" => app.set_dithering(value != 0),
                other => tracing::warn!("unknown audio setting key: {other}"),
            }
            let _ = tx.send(rpc::Cmd::AudioSet(key.into(), value));
        });
    }
    {
        let tx = tx.clone();
        let app_weak = app.as_weak();
        app.on_eq_band_set(move |index, gain| {
            let app = app_weak.unwrap();
            let gain = gain.clamp(-240, 240);
            let model = app.get_eq_bands();
            if let Some(mut row) = model.row_data(index as usize) {
                row.gain = gain;
                model.set_row_data(index as usize, row);
            }
            let _ = tx.send(rpc::Cmd::EqBandSet(index as usize, gain));
        });
    }
    {
        let tx = tx.clone();
        app.on_play_album_shuffled(move |id| {
            let _ = tx.send(rpc::Cmd::PlayAlbumShuffled(id.into()));
        });
    }
    {
        let tx = tx.clone();
        let app_weak = app.as_weak();
        app.on_toggle_shuffle(move || {
            let app = app_weak.unwrap();
            let enabled = !app.get_shuffle();
            app.set_shuffle(enabled); // optimistic
            let _ = tx.send(rpc::Cmd::SetShuffle(enabled));
        });
    }
    {
        let tx = tx.clone();
        let app_weak = app.as_weak();
        app.on_cycle_repeat(move || {
            let app = app_weak.unwrap();
            let mode = (app.get_repeat_mode() + 1) % 3;
            app.set_repeat_mode(mode); // optimistic
            let _ = tx.send(rpc::Cmd::SetRepeat(mode));
        });
    }
    // ── Server switcher ─────────────────────────────────────────────────────
    STATE.with(|s| s.borrow_mut().active_server = rpc::endpoints().display);
    {
        let tx = tx.clone();
        app.on_switcher_opened(move || {
            let _ = tx.send(rpc::Cmd::DiscoverServers);
        });
    }
    {
        let app_weak = app.as_weak();
        app.on_switcher_query(move |q| {
            let app = app_weak.unwrap();
            STATE.with(|s| s.borrow_mut().switcher_query = q.to_string());
            app.set_switcher_results(ModelRc::new(VecModel::from(switcher_results(&q))));
        });
    }
    {
        let tx = tx.clone();
        let app_weak = app.as_weak();
        app.on_switcher_activate(move |item| {
            let app = app_weak.unwrap();
            let kind = item.kind.to_string();
            let id = item.id.to_string();
            if kind == "server" {
                let (host, port) = match id.rsplit_once(':') {
                    Some((h, p)) => (h.to_string(), p.parse().unwrap_or(5051)),
                    None => (id.clone(), 5051),
                };
                let display = format!("{host}:{port}");
                STATE.with(|s| s.borrow_mut().active_server = display.clone());
                app.set_status_text(display.into());
                app.set_connected(false);
                let _ = tx.send(rpc::Cmd::SwitchServer {
                    host,
                    grpc_port: port,
                });
            } else if let Some(idx) = id
                .strip_prefix("srv:")
                .and_then(|i| i.parse::<usize>().ok())
            {
                // Saved Subsonic / Jellyfin server → open the browse flow.
                let srv = STATE.with(|s| {
                    let mut st = s.borrow_mut();
                    st.browse_stack.clear();
                    st.servers.get(idx).cloned()
                });
                if let Some(srv) = srv {
                    app.set_current_tab(4);
                    app.set_browse_error("".into());
                    app.set_browse_loading(true);
                    let _ = tx.send(rpc::Cmd::ConnectServer(srv));
                }
            }
        });
    }

    // ── Track / album context actions ───────────────────────────────────────
    {
        let tx = tx.clone();
        app.on_track_like(move |id| {
            let id: String = id.into();
            let like = !is_liked(&id);
            let _ = tx.send(rpc::Cmd::LikeTrack { id, like });
        });
    }
    {
        let tx = tx.clone();
        app.on_track_insert(move |id, position| {
            let _ = tx.send(rpc::Cmd::InsertTracks {
                position,
                tracks: vec![id.into()],
            });
        });
    }
    {
        let tx = tx.clone();
        app.on_album_insert(move |id, position| {
            let _ = tx.send(rpc::Cmd::InsertAlbum {
                album_id: id.into(),
                position,
            });
        });
    }
    {
        let tx = tx.clone();
        app.on_album_like(move |id| {
            let _ = tx.send(rpc::Cmd::LikeAlbum(id.into()));
        });
    }
    {
        let tx = tx.clone();
        app.on_track_add_to_playlist(move |playlist_id, track_id| {
            let _ = tx.send(rpc::Cmd::PlaylistAddTrack {
                playlist_id: playlist_id.into(),
                track_id: track_id.into(),
            });
        });
    }

    // ── Playlists ───────────────────────────────────────────────────────────
    {
        let tx = tx.clone();
        app.on_playlists_open(move |id| {
            let _ = tx.send(rpc::Cmd::OpenPlaylist(id.into()));
        });
    }
    {
        let tx = tx.clone();
        app.on_playlist_play(move |id| {
            let _ = tx.send(rpc::Cmd::PlaySavedPlaylist(id.into()));
        });
    }
    {
        let tx = tx.clone();
        let app_weak = app.as_weak();
        app.on_playlist_delete(move |id| {
            let app = app_weak.unwrap();
            let id: String = id.into();
            if app.get_pl_detail().id.as_str() == id {
                app.set_pl_detail_open(false);
            }
            let _ = tx.send(rpc::Cmd::PlaylistDelete(id));
        });
    }
    {
        let tx = tx.clone();
        app.on_playlist_form_submit(move |id, name, desc| {
            let cmd = if id.is_empty() {
                rpc::Cmd::PlaylistCreate {
                    name: name.into(),
                    description: desc.into(),
                }
            } else {
                rpc::Cmd::PlaylistUpdate {
                    id: id.into(),
                    name: name.into(),
                    description: desc.into(),
                }
            };
            let _ = tx.send(cmd);
        });
    }
    {
        let tx = tx.clone();
        app.on_playlist_remove_track(move |track_id| {
            let playlist_id = STATE.with(|s| s.borrow().pl_detail_id.clone());
            if let Some(playlist_id) = playlist_id {
                let _ = tx.send(rpc::Cmd::PlaylistRemoveTrack {
                    playlist_id,
                    track_id: track_id.into(),
                });
            }
        });
    }
    {
        let app_weak = app.as_weak();
        app.on_plpicker_query(move |q| {
            let app = app_weak.unwrap();
            let q = q.to_lowercase();
            let items: Vec<PaletteItem> = STATE.with(|s| {
                s.borrow()
                    .playlists
                    .iter()
                    .filter(|p| {
                        q.is_empty()
                            || p.name.to_lowercase().contains(&q)
                            || p.description.to_lowercase().contains(&q)
                    })
                    .map(|p| PaletteItem {
                        kind: "playlist".into(),
                        id: p.id.clone().into(),
                        title: p.name.clone().into(),
                        subtitle: if p.track_count == 1 {
                            "1 track".into()
                        } else {
                            format!("{} tracks", p.track_count).into()
                        },
                        index: -1,
                        has_art: false,
                        art: slint::Image::default(),
                    })
                    .collect()
            });
            app.set_plpicker_results(ModelRc::new(VecModel::from(items)));
        });
    }
    {
        let app_weak = app.as_weak();
        app.on_picker_query(move |q| {
            let app = app_weak.unwrap();
            STATE.with(|s| s.borrow_mut().picker_query = q.to_string());
            app.set_picker_results(ModelRc::new(VecModel::from(picker_results(&q))));
        });
    }
    {
        let tx = tx.clone();
        app.on_picker_add(move |track_id| {
            let track_id: String = track_id.into();
            let playlist_id = STATE.with(|s| {
                let mut st = s.borrow_mut();
                // Optimistic: hide it from the picker immediately.
                st.pl_detail_track_ids.push(track_id.clone());
                st.pl_detail_id.clone()
            });
            if let Some(playlist_id) = playlist_id {
                let _ = tx.send(rpc::Cmd::PlaylistAddTrack {
                    playlist_id,
                    track_id,
                });
            }
        });
    }

    // ── Remote servers ──────────────────────────────────────────────────────
    STATE.with(|s| s.borrow_mut().servers = servers::load());
    refresh_servers_model(&app);
    {
        let app_weak = app.as_weak();
        app.on_server_add(move |kind, name, url, user, pass| {
            let app = app_weak.unwrap();
            STATE.with(|s| {
                let mut st = s.borrow_mut();
                st.servers.push(servers::SavedServer {
                    kind: kind.into(),
                    name: name.into(),
                    url: url.into(),
                    username: user.into(),
                    password: pass.into(),
                });
                servers::save(&st.servers);
            });
            refresh_servers_model(&app);
        });
    }
    {
        let app_weak = app.as_weak();
        app.on_server_delete(move |idx| {
            let app = app_weak.unwrap();
            STATE.with(|s| {
                let mut st = s.borrow_mut();
                if (idx as usize) < st.servers.len() {
                    st.servers.remove(idx as usize);
                    servers::save(&st.servers);
                }
            });
            refresh_servers_model(&app);
        });
    }
    {
        let tx = tx.clone();
        let app_weak = app.as_weak();
        app.on_server_connect(move |idx| {
            let app = app_weak.unwrap();
            let srv = STATE.with(|s| {
                let mut st = s.borrow_mut();
                st.browse_stack.clear();
                st.servers.get(idx as usize).cloned()
            });
            if let Some(srv) = srv {
                app.set_browse_loading(true);
                let _ = tx.send(rpc::Cmd::ConnectServer(srv));
            }
        });
    }
    {
        let tx = tx.clone();
        app.on_browse_open(move |path, title| {
            let _ = tx.send(rpc::Cmd::Browse {
                title: title.into(),
                path: path.into(),
                push: true,
            });
        });
    }
    {
        let tx = tx.clone();
        app.on_browse_play_dir(move |path| {
            let _ = tx.send(rpc::Cmd::PlayDir(path.into()));
        });
    }
    {
        let tx = tx.clone();
        app.on_browse_play_at(move |idx| {
            let dir = STATE.with(|s| s.borrow().browse_stack.last().map(|(_, p)| p.clone()));
            if let Some(dir) = dir {
                let _ = tx.send(rpc::Cmd::PlayDirAt(dir, idx));
            }
        });
    }
    {
        let tx = tx.clone();
        let app_weak = app.as_weak();
        app.on_browse_back(move || {
            let app = app_weak.unwrap();
            let target = STATE.with(|s| {
                let mut st = s.borrow_mut();
                st.browse_stack.pop();
                st.browse_stack.last().cloned()
            });
            match target {
                Some((title, path)) => {
                    app.set_browse_loading(true);
                    let _ = tx.send(rpc::Cmd::Browse {
                        title,
                        path,
                        push: false,
                    });
                }
                None => {
                    app.set_browsing(false);
                    app.set_browse_entries(ModelRc::new(VecModel::from(Vec::<BrowseItem>::new())));
                }
            }
        });
    }
    {
        let app_weak = app.as_weak();
        app.on_palette_query(move |text| {
            let app = app_weak.unwrap();
            app.set_palette_results(ModelRc::new(VecModel::from(palette_results(&text))));
        });
    }
    {
        let tx = tx.clone();
        app.on_palette_activate(move |item| {
            let cmd = match item.kind.as_str() {
                "track" => rpc::Cmd::PlayAllAt(item.index),
                "album" => rpc::Cmd::PlayAlbum(item.id.into()),
                "playlist" => rpc::Cmd::PlaySavedPlaylist(item.id.into()),
                _ => rpc::Cmd::PlayArtist(item.id.into()),
            };
            let _ = tx.send(cmd);
        });
    }
    {
        let app_weak = app.as_weak();
        app.on_open_artist(move |name| {
            let app = app_weak.unwrap();
            app.invoke_open_palette_with(name);
        });
    }

    app.run()
}
