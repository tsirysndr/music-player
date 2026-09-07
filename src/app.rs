use std::{sync::mpsc::Sender, time::Instant};

use music_player_server::api::{
    metadata::v1alpha1::{Album, Artist, Track},
    music::v1alpha1::GetCurrentlyPlayingSongResponse,
};
use nucleo_matcher::{
    pattern::{CaseMatching, Normalization, Pattern},
    Config, Matcher, Utf32Str,
};
use ratatui::layout::Rect;

use crate::{network::IoEvent, user_config::UserConfig};

pub type CurrentlyPlaybackContext = GetCurrentlyPlayingSongResponse;

pub const SEEK_STEP_MS: u32 = 5_000;
pub const VOLUME_STEP: u32 = 5;

/// Number of items fetched per page for the browse tables (tracks/albums/artists).
pub const PAGE_SIZE: usize = 200;
/// Number of items fetched per request while building the fuzzy-search index.
pub const SEARCH_PAGE_SIZE: usize = 500;
/// When the selection gets within this many rows of the end of a partially
/// loaded collection, the next page is requested.
pub const LOAD_MORE_THRESHOLD: usize = 20;

/// The three independently paged browse collections.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PagedCollection {
    Tracks,
    Albums,
    Artists,
}

/// Incremental-loading state for one paged collection.
#[derive(Default, Debug)]
pub struct Pagination {
    /// Offset to request for the next page.
    pub next_offset: usize,
    /// Whether the server may have more items past `next_offset`.
    pub has_more: bool,
    /// A page request is currently in flight (prevents duplicate requests).
    pub loading: bool,
}

impl Pagination {
    /// State after receiving one page: a full page means there may be more.
    pub fn after_page(next_offset: usize, page_len: usize) -> Self {
        Self {
            next_offset,
            has_more: page_len >= PAGE_SIZE,
            loading: false,
        }
    }
}

#[derive(Clone)]
pub struct PlaylistItem {
    pub id: String,
    pub name: String,
}

#[derive(Default)]
pub struct TrackTable {
    pub tracks: Vec<Track>,
    pub selected_index: usize,
    /// Paging state; only meaningful when the table holds the library's
    /// track list (album/artist/queue views load everything at once and
    /// leave this in its default, exhausted state).
    pub pagination: Pagination,
}

#[derive(Default)]
pub struct ArtistTable {
    pub artists: Vec<Artist>,
    pub selected_index: usize,
    pub pagination: Pagination,
}

#[derive(Default)]
pub struct AlbumTable {
    pub albums: Vec<Album>,
    pub selected_index: usize,
    pub pagination: Pagination,
}

/// Cached copy of the whole library, used as the haystack for fuzzy search.
#[derive(Default)]
pub struct LibraryCache {
    pub tracks: Vec<Track>,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SearchScope {
    Tracks,
    Albums,
    Artists,
}

impl SearchScope {
    pub fn next(self) -> Self {
        match self {
            SearchScope::Tracks => SearchScope::Albums,
            SearchScope::Albums => SearchScope::Artists,
            SearchScope::Artists => SearchScope::Tracks,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            SearchScope::Tracks => "Tracks",
            SearchScope::Albums => "Albums",
            SearchScope::Artists => "Artists",
        }
    }
}

/// A single row in the fuzzy finder result list.
pub struct SearchResult {
    pub id: String,
    pub display: String,
    /// Char indices (into `display`) that matched the query.
    pub indices: Vec<u32>,
    pub score: u32,
}

/// State of the fzf-style fuzzy finder overlay.
pub struct SearchState {
    pub active: bool,
    pub query: String,
    pub scope: SearchScope,
    pub results: Vec<SearchResult>,
    pub selected_index: usize,
    pub loading: bool,
    matcher: Matcher,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            active: false,
            query: String::new(),
            scope: SearchScope::Tracks,
            results: vec![],
            selected_index: 0,
            loading: false,
            matcher: Matcher::new(Config::DEFAULT),
        }
    }
}

pub struct App {
    /// The smart-playlist form overlay. Swallows keys while `active`.
    pub smart_playlist_form: crate::smart_playlist_form::SmartPlaylistForm,
    pub instant_since_last_current_playback_poll: Instant,
    pub size: Rect,
    navigation_stack: Vec<Route>,
    pub library: Library,
    pub user_config: UserConfig,
    io_tx: Option<Sender<IoEvent>>,
    pub playlists: Vec<PlaylistItem>,
    pub selected_playlist_index: Option<usize>,
    pub artist_table: ArtistTable,
    pub album_table: AlbumTable,
    pub track_table: TrackTable,
    pub artist_albums: Vec<Album>,
    pub selected_album: Option<Album>,
    pub selected_artist_name: Option<String>,
    pub current_playback_context: Option<CurrentlyPlaybackContext>,
    pub song_progress_ms: u128,
    pub is_fetching_current_playback: bool,
    pub volume: u32,
    /// Muted at the daemon, which keeps `volume` so unmuting restores it.
    pub muted: bool,
    pub show_help: bool,
    pub help_scroll: u16,
    pub search: SearchState,
    pub library_cache: Option<LibraryCache>,
    pub server_addr: String,
}

impl App {
    pub fn new(io_tx: Sender<IoEvent>) -> Self {
        Self {
            size: Rect::default(),
            navigation_stack: vec![DEFAULT_ROUTE],
            library: Library { selected_index: 0 },
            user_config: UserConfig::new(),
            selected_playlist_index: None,
            playlists: vec![],
            io_tx: Some(io_tx),
            track_table: Default::default(),
            artist_table: Default::default(),
            album_table: Default::default(),
            artist_albums: vec![],
            selected_album: None,
            selected_artist_name: None,
            current_playback_context: None,
            song_progress_ms: 0,
            is_fetching_current_playback: false,
            instant_since_last_current_playback_poll: Instant::now(),
            volume: 100,
            muted: false,
            show_help: false,
            help_scroll: 0,
            search: SearchState::default(),
            smart_playlist_form: Default::default(),
            library_cache: None,
            server_addr: "localhost:5051".to_string(),
        }
    }

    // Send a network event to the network thread
    pub fn dispatch(&mut self, action: IoEvent) {
        if let Some(io_tx) = &self.io_tx {
            if io_tx.send(action).is_err() {
                // The network thread is gone; nothing we can do from here.
            }
        }
    }

    pub fn get_current_route(&self) -> &Route {
        self.navigation_stack.last().unwrap_or(&DEFAULT_ROUTE)
    }

    pub fn push_navigation_stack(
        &mut self,
        next_route_id: RouteId,
        next_active_block: ActiveBlock,
    ) {
        if !self
            .navigation_stack
            .last()
            .map(|last_route| last_route.id == next_route_id)
            .unwrap_or(false)
        {
            self.navigation_stack.push(Route {
                id: next_route_id,
                active_block: next_active_block,
                hovered_block: next_active_block,
            });
        }
    }

    pub fn pop_navigation_stack(&mut self) -> Option<Route> {
        if self.navigation_stack.len() == 1 {
            None
        } else {
            self.navigation_stack.pop()
        }
    }

    fn get_current_route_mut(&mut self) -> &mut Route {
        self.navigation_stack.last_mut().unwrap()
    }

    pub fn set_current_route_state(
        &mut self,
        active_block: Option<ActiveBlock>,
        hovered_block: Option<ActiveBlock>,
    ) {
        let current_route = self.get_current_route_mut();
        if let Some(active_block) = active_block {
            current_route.active_block = active_block;
        }
        if let Some(hovered_block) = hovered_block {
            current_route.hovered_block = hovered_block;
        }
    }

    pub fn decrease_volume(&mut self) {
        self.volume = self.volume.saturating_sub(VOLUME_STEP);
        let volume = self.volume;
        // Setting a level unmutes on the daemon; keep the flag in step so the
        // status line does not go on claiming to be muted.
        self.muted = false;
        self.dispatch(IoEvent::SetVolume(volume));
    }

    pub fn increase_volume(&mut self) {
        self.volume = (self.volume + VOLUME_STEP).min(100);
        let volume = self.volume;
        self.muted = false;
        self.dispatch(IoEvent::SetVolume(volume));
    }

    /// Silence without losing the level — unmuting restores it.
    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        let muted = self.muted;
        self.dispatch(IoEvent::SetMute(muted));
    }

    pub fn toggle_playback(&mut self) {
        self.dispatch(IoEvent::TogglePlayback);
    }

    fn current_track_duration_ms(&self) -> Option<u32> {
        self.current_playback_context
            .as_ref()
            .and_then(|ctx| ctx.track.as_ref())
            .map(|track| (track.duration * 1000.0) as u32)
    }

    pub fn seek_forwards(&mut self) {
        if let Some(duration_ms) = self.current_track_duration_ms() {
            let target = (self.song_progress_ms as u32)
                .saturating_add(SEEK_STEP_MS)
                .min(duration_ms);
            self.seek_to(target);
        }
    }

    pub fn seek_backwards(&mut self) {
        if self.current_track_duration_ms().is_some() {
            let target = (self.song_progress_ms as u32).saturating_sub(SEEK_STEP_MS);
            self.seek_to(target);
        }
    }

    fn seek_to(&mut self, position_ms: u32) {
        // Optimistically update the local progress so the UI reacts instantly.
        self.song_progress_ms = position_ms as u128;
        if let Some(ctx) = self.current_playback_context.as_mut() {
            ctx.position_ms = position_ms;
        }
        self.instant_since_last_current_playback_poll = Instant::now();
        self.dispatch(IoEvent::Seek(position_ms));
    }

    // ---- Incremental (paged) loading of the browse tables ----

    /// Requests the next page of `collection` when the selection is close to
    /// the end of what is loaded and more items are available. Safe to call
    /// after every selection change; it is a no-op while a page is already in
    /// flight or when the collection is exhausted.
    pub fn maybe_load_more(&mut self, collection: PagedCollection) {
        let (len, selected_index, pagination) = match collection {
            PagedCollection::Tracks => (
                self.track_table.tracks.len(),
                self.track_table.selected_index,
                &mut self.track_table.pagination,
            ),
            PagedCollection::Albums => (
                self.album_table.albums.len(),
                self.album_table.selected_index,
                &mut self.album_table.pagination,
            ),
            PagedCollection::Artists => (
                self.artist_table.artists.len(),
                self.artist_table.selected_index,
                &mut self.artist_table.pagination,
            ),
        };
        if pagination.has_more && !pagination.loading && selected_index + LOAD_MORE_THRESHOLD >= len
        {
            pagination.loading = true;
            self.dispatch(IoEvent::LoadMore(collection));
        }
    }

    // ---- Fuzzy search (fzf-style overlay) ----

    pub fn open_search(&mut self) {
        self.search.active = true;
        self.search.query.clear();
        self.search.selected_index = 0;
        if self.library_cache.is_none() && !self.search.loading {
            self.search.loading = true;
            self.dispatch(IoEvent::LoadSearchIndex);
        }
        self.refresh_search_results();
    }

    pub fn close_search(&mut self) {
        self.search.active = false;
        self.search.query.clear();
        self.search.results.clear();
        self.search.selected_index = 0;
    }

    /// Re-run the fuzzy matcher over the cached library for the current
    /// query/scope. Results are ranked by nucleo score (descending).
    pub fn refresh_search_results(&mut self) {
        let cache = match &self.library_cache {
            Some(cache) => cache,
            None => {
                self.search.results.clear();
                self.search.selected_index = 0;
                return;
            }
        };

        let candidates: Vec<(String, String)> = match self.search.scope {
            SearchScope::Tracks => cache
                .tracks
                .iter()
                .map(|t| {
                    let album = t
                        .album
                        .as_ref()
                        .map(|a| a.title.as_str())
                        .unwrap_or_default();
                    (
                        t.id.clone(),
                        format!("{} — {} — {}", t.title, t.artist, album),
                    )
                })
                .collect(),
            SearchScope::Albums => cache
                .albums
                .iter()
                .map(|a| (a.id.clone(), format!("{} — {}", a.title, a.artist)))
                .collect(),
            SearchScope::Artists => cache
                .artists
                .iter()
                .map(|a| (a.id.clone(), a.name.clone()))
                .collect(),
        };

        let query = self.search.query.clone();
        let mut results: Vec<SearchResult> = if query.is_empty() {
            candidates
                .into_iter()
                .map(|(id, display)| SearchResult {
                    id,
                    display,
                    indices: vec![],
                    score: 0,
                })
                .collect()
        } else {
            let pattern = Pattern::parse(&query, CaseMatching::Ignore, Normalization::Smart);
            let mut buf = Vec::new();
            let mut matched = Vec::new();
            for (id, display) in candidates {
                let haystack = Utf32Str::new(&display, &mut buf);
                let mut indices = Vec::new();
                if let Some(score) =
                    pattern.indices(haystack, &mut self.search.matcher, &mut indices)
                {
                    indices.sort_unstable();
                    indices.dedup();
                    matched.push(SearchResult {
                        id,
                        display,
                        indices,
                        score,
                    });
                }
            }
            matched.sort_by(|a, b| b.score.cmp(&a.score));
            matched
        };

        // Keep the list snappy even on huge libraries.
        results.truncate(500);
        self.search.results = results;
        if self.search.selected_index >= self.search.results.len() {
            self.search.selected_index = 0;
        }
    }

    pub fn poll_current_playback(&mut self) {
        // Poll every 5 seconds
        let poll_interval_ms = 5_000;

        let elapsed = self
            .instant_since_last_current_playback_poll
            .elapsed()
            .as_millis();

        if !self.is_fetching_current_playback && elapsed >= poll_interval_ms {
            self.is_fetching_current_playback = true;
            self.dispatch(IoEvent::GetCurrentPlayback);
        }
    }

    pub fn update_on_tick(&mut self) {
        self.poll_current_playback();
        if let Some(CurrentlyPlaybackContext {
            track: Some(track),
            position_ms,
            is_playing,
            ..
        }) = &self.current_playback_context
        {
            let elapsed = if *is_playing {
                self.instant_since_last_current_playback_poll
                    .elapsed()
                    .as_millis()
            } else {
                0u128
            } + u128::from(*position_ms);
            let duration_ms = (track.duration * 1000.0) as u128;
            if elapsed < duration_ms {
                self.song_progress_ms = elapsed;
            } else {
                self.song_progress_ms = duration_ms;
            }
        }
    }
}

pub const LIBRARY_OPTIONS: [&str; 4] = ["Tracks", "Albums", "Artists", "Play Queue"];

const DEFAULT_ROUTE: Route = Route {
    id: RouteId::TrackTable,
    active_block: ActiveBlock::Empty,
    hovered_block: ActiveBlock::Library,
};

#[derive(Clone)]
pub struct Library {
    pub selected_index: usize,
}

#[derive(Clone, PartialEq, Debug)]
pub enum RouteId {
    AlbumTracks,
    AlbumList,
    Artist,
    TrackTable,
    Artists,
    PlayQueue,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ActiveBlock {
    PlayBar,
    AlbumTracks,
    AlbumList,
    ArtistBlock,
    Library,
    Playlists,
    TrackTable,
    Artists,
    PlayQueue,
    Empty,
}

#[derive(Debug)]
pub struct Route {
    pub id: RouteId,
    pub active_block: ActiveBlock,
    pub hovered_block: ActiveBlock,
}
