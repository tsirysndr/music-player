use std::sync::mpsc::Sender;

use music_player_server::metadata::v1alpha1::{Album, Artist, Track};
use tui::layout::Rect;

use crate::{network::IoEvent, user_config::UserConfig};

pub struct Playlist {
    pub items: Vec<PlaylistItem>,
}

pub struct PlaylistItem {
    pub name: String,
}

#[derive(Default)]
pub struct TrackTable {
    pub tracks: Vec<Track>,
    pub selected_index: usize,
}

#[derive(Default)]
pub struct ArtistTable {
    pub artists: Vec<Artist>,
    pub selected_index: usize,
}

#[derive(Default)]
pub struct AlbumTable {
    pub albums: Vec<Album>,
    pub selected_index: usize,
}

pub struct App {
    pub size: Rect,
    navigation_stack: Vec<Route>,
    pub library: Library,
    pub user_config: UserConfig,
    pub input: Vec<char>,
    pub input_idx: usize,
    pub input_cursor_position: u16,
    io_tx: Option<Sender<IoEvent>>,
    is_loading: bool,
    pub playlists: Option<Playlist>,
    pub selected_playlist_index: Option<usize>,
    pub active_playlist_index: Option<usize>,
    pub artist_table: ArtistTable,
    pub album_table: AlbumTable,
    pub track_table: TrackTable,
    pub selected_album: Option<Album>,
}

impl App {
    pub fn new(io_tx: Sender<IoEvent>) -> Self {
        Self {
            size: Rect::default(),
            navigation_stack: vec![DEFAULT_ROUTE],
            library: Library { selected_index: 0 },
            user_config: UserConfig::new(),
            input: vec![],
            input_idx: 0,
            input_cursor_position: 0,
            selected_playlist_index: None,
            active_playlist_index: None,
            playlists: None,
            io_tx: Some(io_tx),
            is_loading: false,
            track_table: Default::default(),
            artist_table: Default::default(),
            album_table: Default::default(),
            selected_album: None,
        }
    }

    // Send a network event to the network thread
    pub fn dispatch(&mut self, action: IoEvent) {
        // `is_loading` will be set to false again after the async action has finished in network.rs
        self.is_loading = true;
        if let Some(io_tx) = &self.io_tx {
            if let Err(e) = io_tx.send(action) {
                self.is_loading = false;
                println!("Error from dispatch {}", e);
                // TODO: handle error
            };
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
        let mut current_route = self.get_current_route_mut();
        if let Some(active_block) = active_block {
            current_route.active_block = active_block;
        }
        if let Some(hovered_block) = hovered_block {
            current_route.hovered_block = hovered_block;
        }
    }

    pub fn reset_route_state(&mut self) {
        self.navigation_stack = vec![DEFAULT_ROUTE];
    }

    pub fn decrease_volume(&mut self) {}

    pub fn increase_volume(&mut self) {}

    pub fn toggle_playback(&mut self) {
        self.dispatch(IoEvent::TogglePlayback);
    }

    pub fn seek_forwards(&mut self) {}

    pub fn seek_backwards(&mut self) {}

    pub fn shuffle(&mut self) {}

    pub fn repeat(&mut self) {}
}

pub const LIBRARY_OPTIONS: [&str; 4] = ["Tracks", "Albums", "Artists", "Play Queue"];

const DEFAULT_ROUTE: Route = Route {
    id: RouteId::TrackTable,
    active_block: ActiveBlock::Empty,
    hovered_block: ActiveBlock::Library,
};

#[derive(Clone)]
pub struct ScrollableResultPages<T> {
    index: usize,
    pub pages: Vec<T>,
}

impl<T> ScrollableResultPages<T> {
    pub fn new() -> ScrollableResultPages<T> {
        ScrollableResultPages {
            index: 0,
            pages: vec![],
        }
    }

    pub fn get_results(&self, at_index: Option<usize>) -> Option<&T> {
        self.pages.get(at_index.unwrap_or(self.index))
    }

    pub fn get_mut_results(&mut self, at_index: Option<usize>) -> Option<&mut T> {
        self.pages.get_mut(at_index.unwrap_or(self.index))
    }

    pub fn add_pages(&mut self, new_pages: T) {
        self.pages.push(new_pages);
        // Whenever a new page is added, set the active index to the end of the vector
        self.index = self.pages.len() - 1;
    }
}

#[derive(Clone)]
pub struct Library {
    pub selected_index: usize,
}

#[derive(PartialEq, Debug)]
pub enum SearchResultBlock {
    AlbumSearch,
    SongSearch,
    ArtistSearch,
    PlaylistSearch,
    Empty,
}

#[derive(PartialEq, Debug, Clone)]
pub enum ArtistBlock {
    Tracks,
    Albums,
    Empty,
}

#[derive(Clone, PartialEq, Debug)]
pub enum RouteId {
    AlbumTracks,
    AlbumList,
    Artist,
    Search,
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
    SearchResultBlock,
    Input,
    PlayQueue,
    Empty,
}

#[derive(Debug)]
pub struct Route {
    pub id: RouteId,
    pub active_block: ActiveBlock,
    pub hovered_block: ActiveBlock,
}
