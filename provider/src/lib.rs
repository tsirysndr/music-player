//! What a remote music server has to be, and nothing else.
//!
//! A *provider* is where the library screens read from: another music-player
//! daemon, a Subsonic/Navidrome server, Jellyfin, Kodi. It is deliberately not
//! a *sink* — where the audio comes out is [`music_player_renderer::Player`], a
//! separate trait behind separate state. Keeping them apart is what lets you
//! switch servers without interrupting whatever is playing: nothing reachable
//! from here can send a `PlayerCommand` or touch the tracklist.
//!
//! This crate exists rather than living in `addons` because of the dependency
//! graph: `addons -> client -> server`, so `server` can never depend on
//! `addons`. Both the gRPC server and the GraphQL schema need to route reads
//! through a connected provider, so the trait has to sit below both of them.
//! `addons` implements it; the binaries wire the two together.
//!
//! Adding a backend is one file and one line — see [`registry`].

pub mod backends;
pub mod config;
pub mod http;
pub mod registry;
pub mod state;
pub mod url;

pub use backends::{builtin_registry, register_builtin};
pub use config::ProviderConfig;
pub use registry::{ProviderFactory, ProviderKindInfo, ProviderRegistry};
pub use state::{ConnectedProvider, ProviderState};

pub use music_player_types::types::{Album, Artist, Genre, Playlist, Track};
use std::fmt;

/// A slice of a listing. `limit <= 0` means "as many as the backend will give".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Page {
    pub offset: i32,
    pub limit: i32,
}

impl Page {
    pub fn new(offset: i32, limit: i32) -> Self {
        Self {
            offset: offset.max(0),
            limit,
        }
    }

    /// No paging — used where a caller wants everything a backend has.
    pub fn all() -> Self {
        Self {
            offset: 0,
            limit: -1,
        }
    }

    /// Applies the page to an already-materialised list, for backends whose
    /// API cannot paginate (Mopidy-style `browse`, Kodi playlist files).
    pub fn slice<T>(&self, items: Vec<T>) -> Vec<T> {
        let offset = self.offset.max(0) as usize;
        let mut items: Vec<T> = items.into_iter().skip(offset).collect();
        if self.limit > 0 {
            items.truncate(self.limit as usize);
        }
        items
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::new(0, 100)
    }
}

/// What a backend can actually do, so callers can ask before they offer it.
///
/// Everything defaults to "no": a new backend claims a feature by naming it,
/// which is harder to get wrong than remembering to opt out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProviderCapabilities {
    /// Whether this server can list genres. False means the Genres screen
    /// shows the local library instead of an empty page.
    pub genres: bool,
    pub playlists: bool,
    pub liked: bool,
    /// The backend has a real search endpoint. When false the default
    /// [`MusicProvider::search`] fans out to three filtered list calls instead.
    pub native_search: bool,
}

#[derive(Debug, Default)]
pub struct SearchResults {
    pub artists: Vec<Artist>,
    pub albums: Vec<Album>,
    pub tracks: Vec<Track>,
}

#[derive(Debug)]
pub enum ProviderError {
    /// The backend has no such concept — likes on Kodi, playlists on a server
    /// that does not keep any. Distinct from a failure: there is nothing to
    /// retry and nothing to fix.
    Unsupported {
        kind: &'static str,
        feature: &'static str,
    },
    NotFound(String),
    Auth(String),
    Transport(String),
    Other(String),
}

impl ProviderError {
    pub fn transport(e: impl fmt::Display) -> Self {
        ProviderError::Transport(e.to_string())
    }

    pub fn other(e: impl fmt::Display) -> Self {
        ProviderError::Other(e.to_string())
    }
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderError::Unsupported { kind, feature } => {
                write!(f, "{kind} does not support {feature}")
            }
            ProviderError::NotFound(what) => write!(f, "not found: {what}"),
            ProviderError::Auth(why) => write!(f, "authentication failed: {why}"),
            ProviderError::Transport(why) => write!(f, "{why}"),
            ProviderError::Other(why) => write!(f, "{why}"),
        }
    }
}

impl std::error::Error for ProviderError {}

/// A remote music library.
///
/// **Six required methods.** Everything else has a default, so a new backend
/// starts small and grows only into what its server actually offers.
///
/// `&self`, not `&mut self`: a source is shared behind an `Arc` and called
/// from every resolver at once. Anything that needs a mutable client (a tonic
/// channel, say) keeps its own lock inside, so one slow backend cannot stall
/// requests to a different one — or to the local library.
#[async_trait::async_trait]
pub trait MusicProvider: Send + Sync + 'static {
    /// The registry key — `"subsonic"`, `"jellyfin"`, `"kodi"`, …
    fn kind(&self) -> &'static str;

    /// Absolute base url, scheme included, no trailing slash. Non-empty by
    /// construction: [`ProviderFactory::connect`] refuses an empty one, which is
    /// what stops relative track and cover uris from escaping unresolved.
    fn base_url(&self) -> &str;

    /// Host only. Used when a track's uri has to be rewritten for a cast
    /// device that has to reach the server itself.
    fn host(&self) -> &str;

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities::default()
    }

    // ── required ────────────────────────────────────────────────────────────

    async fn albums(&self, filter: Option<&str>, page: Page) -> Result<Vec<Album>, ProviderError>;
    async fn artists(&self, filter: Option<&str>, page: Page)
        -> Result<Vec<Artist>, ProviderError>;
    async fn tracks(&self, filter: Option<&str>, page: Page) -> Result<Vec<Track>, ProviderError>;
    async fn album(&self, id: &str) -> Result<Album, ProviderError>;
    async fn artist(&self, id: &str) -> Result<Artist, ProviderError>;
    async fn track(&self, id: &str) -> Result<Track, ProviderError>;

    // ── provided ────────────────────────────────────────────────────────────

    /// Empty rather than an error: a server with no playlist concept should
    /// show an empty Playlists screen, not a red banner.
    async fn playlists(&self, _page: Page) -> Result<Vec<Playlist>, ProviderError> {
        Ok(vec![])
    }

    async fn playlist(&self, _id: &str) -> Result<Playlist, ProviderError> {
        Err(ProviderError::Unsupported {
            kind: self.kind(),
            feature: "playlists",
        })
    }

    /// Three filtered list calls, which is exactly the idiom for a backend
    /// whose API has no search endpoint. Override when there is one.
    ///
    /// A failing leg yields an empty list rather than failing the search: a
    /// server that can list albums but not artists should still find albums.
    async fn search(&self, keyword: &str, page: Page) -> Result<SearchResults, ProviderError> {
        Ok(SearchResults {
            artists: self.artists(Some(keyword), page).await.unwrap_or_default(),
            albums: self.albums(Some(keyword), page).await.unwrap_or_default(),
            tracks: self.tracks(Some(keyword), page).await.unwrap_or_default(),
        })
    }

    /// Empty by default. A liked list must never fall back to the *local*
    /// likes: those are ids from a different library, and rendering them over
    /// a remote source gives rows that cannot be played.
    async fn liked_tracks(&self, _page: Page) -> Result<Vec<Track>, ProviderError> {
        Ok(vec![])
    }

    async fn set_liked(&self, _id: &str, _liked: bool) -> Result<(), ProviderError> {
        Err(ProviderError::Unsupported {
            kind: self.kind(),
            feature: "likes",
        })
    }

    /// The genres this server knows about.
    ///
    /// Every backend that has them has its own endpoint for it, and none of
    /// them expose the local `genre` table — so without this the Genres screen
    /// would go blank the moment a server was connected, which is the shape of
    /// bug this trait exists to prevent.
    ///
    /// The default is empty rather than an error: a server without genres has
    /// none, which is a fact, not a failure.
    async fn genres(&self, _page: Page) -> Result<Vec<Genre>, ProviderError> {
        Ok(vec![])
    }

    /// The tracks in one genre.
    async fn genre_tracks(&self, _genre: &str, _page: Page) -> Result<Vec<Track>, ProviderError> {
        Ok(vec![])
    }

    /// Add a track to one of this server's playlists.
    ///
    /// A remote track cannot go into a *local* playlist — the row would point
    /// at an id the local library has never heard of — so with a provider
    /// connected the playlist is its own, and so is the track.
    async fn add_to_playlist(
        &self,
        _playlist_id: &str,
        _track_id: &str,
    ) -> Result<(), ProviderError> {
        Err(ProviderError::Unsupported {
            kind: self.kind(),
            feature: "editing playlists",
        })
    }

    async fn remove_from_playlist(
        &self,
        _playlist_id: &str,
        _track_id: &str,
    ) -> Result<(), ProviderError> {
        Err(ProviderError::Unsupported {
            kind: self.kind(),
            feature: "editing playlists",
        })
    }

    /// A cheap round trip, used to check a server is reachable before it is
    /// made current.
    async fn ping(&self) -> Result<(), ProviderError> {
        Ok(())
    }
}
