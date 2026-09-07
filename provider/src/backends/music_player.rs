//! Another music-player daemon.
//!
//! Over the peer's **GraphQL** endpoint rather than its gRPC one. Two reasons:
//! the gRPC client lives in `music-player-client`, which depends on
//! `music-player-server`, and this crate has to stay *below* the server crate
//! so the gRPC layer can route reads through a provider — using it here would
//! close the cycle. And the url a saved server carries is the web endpoint
//! anyway, which is the same address a queued track's `/tracks/<id>` has to
//! resolve against, so there is nothing left to guess.

use crate::{
    http, Album, Artist, MusicProvider, Page, Playlist, ProviderCapabilities, ProviderConfig,
    ProviderError, ProviderFactory, SearchResults, Track,
};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

/// The port a daemon serves its web UI and GraphQL API on.
const WEB_PORT: u16 = 5053;

pub struct MusicPlayer {
    base_url: String,
    host: String,
}

impl MusicPlayer {
    pub fn new(base_url: &str) -> Self {
        let base_url = base_url.trim_end_matches('/').to_string();
        let host = url::Url::parse(&base_url)
            .ok()
            .and_then(|url| url.host_str().map(str::to_owned))
            .unwrap_or_default();
        Self { base_url, host }
    }

    async fn query<T: serde::de::DeserializeOwned>(
        &self,
        query: &str,
        variables: Value,
        field: &str,
    ) -> Result<T, ProviderError> {
        let response = http::client()
            .post(format!("{}/graphql", self.base_url))
            .json(&json!({ "query": query, "variables": variables }))
            .send()
            .await
            .map_err(ProviderError::transport)?;
        let status = response.status();
        if !status.is_success() {
            return Err(ProviderError::Transport(format!(
                "the peer answered {status}"
            )));
        }

        let mut body: GraphQlResponse = response.json().await.map_err(ProviderError::transport)?;
        // GraphQL reports failures in the body with a 200, so the errors array
        // is the only thing that says a query did not work.
        if let Some(errors) = body.errors.filter(|errors| !errors.is_empty()) {
            let message = errors
                .into_iter()
                .map(|error| error.message)
                .collect::<Vec<_>>()
                .join("; ");
            return Err(ProviderError::Other(message));
        }
        let data = body
            .data
            .take()
            .ok_or_else(|| ProviderError::Other("the peer returned no data".to_string()))?;
        let field = data
            .get(field)
            .cloned()
            .ok_or_else(|| ProviderError::NotFound(field.to_string()))?;
        if field.is_null() {
            return Err(ProviderError::NotFound(field.to_string()));
        }
        serde_json::from_value(field).map_err(ProviderError::other)
    }
}

/// The peer's own field selection, kept to what [`Track`] needs.
const TRACK_FIELDS: &str = r#"
    id title duration discNumber trackNumber uri artist
    artists { id name }
    album { id title artist year cover }
"#;

const ALBUM_FIELDS: &str = "id title artist year cover";
const ARTIST_FIELDS: &str = "id name picture";

#[async_trait]
impl MusicProvider for MusicPlayer {
    fn kind(&self) -> &'static str {
        "music-player"
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn host(&self) -> &str {
        &self.host
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            playlists: true,
            liked: true,
            native_search: true,
        }
    }

    async fn albums(&self, filter: Option<&str>, page: Page) -> Result<Vec<Album>, ProviderError> {
        let query = format!(
            "query($filter: String, $offset: Int, $limit: Int) {{ albums(filter: $filter, offset: $offset, limit: $limit) {{ {ALBUM_FIELDS} }} }}"
        );
        let albums: Vec<RemoteAlbum> = self
            .query(&query, page.with_filter(filter), "albums")
            .await?;
        Ok(albums.into_iter().map(Into::into).collect())
    }

    async fn artists(
        &self,
        filter: Option<&str>,
        page: Page,
    ) -> Result<Vec<Artist>, ProviderError> {
        let query = format!(
            "query($filter: String, $offset: Int, $limit: Int) {{ artists(filter: $filter, offset: $offset, limit: $limit) {{ {ARTIST_FIELDS} }} }}"
        );
        let artists: Vec<RemoteArtist> = self
            .query(&query, page.with_filter(filter), "artists")
            .await?;
        Ok(artists.into_iter().map(Into::into).collect())
    }

    async fn tracks(&self, filter: Option<&str>, page: Page) -> Result<Vec<Track>, ProviderError> {
        let query = format!(
            "query($filter: String, $offset: Int, $limit: Int) {{ tracks(filter: $filter, offset: $offset, limit: $limit) {{ {TRACK_FIELDS} }} }}"
        );
        let tracks: Vec<RemoteTrack> = self
            .query(&query, page.with_filter(filter), "tracks")
            .await?;
        Ok(tracks.into_iter().map(Into::into).collect())
    }

    async fn album(&self, id: &str) -> Result<Album, ProviderError> {
        let query = format!(
            "query($id: ID!) {{ album(id: $id) {{ {ALBUM_FIELDS} tracks {{ {TRACK_FIELDS} }} }} }}"
        );
        let album: RemoteAlbum = self.query(&query, json!({ "id": id }), "album").await?;
        Ok(album.into())
    }

    async fn artist(&self, id: &str) -> Result<Artist, ProviderError> {
        let query = format!(
            "query($id: ID!) {{ artist(id: $id) {{ {ARTIST_FIELDS} albums {{ {ALBUM_FIELDS} }} songs {{ {TRACK_FIELDS} }} }} }}"
        );
        let artist: RemoteArtist = self.query(&query, json!({ "id": id }), "artist").await?;
        Ok(artist.into())
    }

    async fn track(&self, id: &str) -> Result<Track, ProviderError> {
        let query = format!("query($id: ID!) {{ track(id: $id) {{ {TRACK_FIELDS} }} }}");
        let track: RemoteTrack = self.query(&query, json!({ "id": id }), "track").await?;
        Ok(track.into())
    }

    async fn playlists(&self, page: Page) -> Result<Vec<Playlist>, ProviderError> {
        let query = "query { playlists { id name description } }";
        let playlists: Vec<RemotePlaylist> = self.query(query, json!({}), "playlists").await?;
        Ok(page
            .slice(playlists)
            .into_iter()
            .map(Into::into)
            .collect())
    }

    async fn playlist(&self, id: &str) -> Result<Playlist, ProviderError> {
        let query = format!(
            "query($id: ID!) {{ playlist(id: $id) {{ id name description tracks {{ {TRACK_FIELDS} }} }} }}"
        );
        let playlist: RemotePlaylist = self.query(&query, json!({ "id": id }), "playlist").await?;
        Ok(playlist.into())
    }

    async fn liked_tracks(&self, page: Page) -> Result<Vec<Track>, ProviderError> {
        let query = format!(
            "query($offset: Int, $limit: Int) {{ likedTracks(offset: $offset, limit: $limit) {{ {TRACK_FIELDS} }} }}"
        );
        let tracks: Vec<RemoteTrack> = self
            .query(
                &query,
                json!({ "offset": page.offset, "limit": page.limit }),
                "likedTracks",
            )
            .await?;
        Ok(tracks.into_iter().map(Into::into).collect())
    }

    async fn set_liked(&self, id: &str, liked: bool) -> Result<(), ProviderError> {
        let query = "mutation($id: String!, $like: Boolean!) { likeTrack(id: $id, like: $like) }";
        let _: Value = self
            .query(query, json!({ "id": id, "like": liked }), "likeTrack")
            .await?;
        Ok(())
    }

    async fn search(&self, keyword: &str, _page: Page) -> Result<SearchResults, ProviderError> {
        let query = format!(
            "query($keyword: String!) {{ search(keyword: $keyword) {{ artists {{ {ARTIST_FIELDS} }} albums {{ {ALBUM_FIELDS} }} tracks {{ {TRACK_FIELDS} }} }} }}"
        );
        let results: RemoteSearch = self
            .query(&query, json!({ "keyword": keyword }), "search")
            .await?;
        Ok(SearchResults {
            artists: results.artists.into_iter().map(Into::into).collect(),
            albums: results.albums.into_iter().map(Into::into).collect(),
            tracks: results.tracks.into_iter().map(Into::into).collect(),
        })
    }

    async fn ping(&self) -> Result<(), ProviderError> {
        self.albums(None, Page::new(0, 1)).await.map(|_| ())
    }
}

impl Page {
    /// The variables every filtered listing query takes.
    fn with_filter(&self, filter: Option<&str>) -> Value {
        json!({ "filter": filter, "offset": self.offset, "limit": self.limit })
    }
}

// ── The peer's wire shapes ──────────────────────────────────────────────────

#[derive(Deserialize)]
struct GraphQlResponse {
    data: Option<Value>,
    #[serde(default)]
    errors: Option<Vec<GraphQlError>>,
}

#[derive(Deserialize)]
struct GraphQlError {
    message: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoteTrack {
    id: String,
    title: String,
    duration: Option<f32>,
    #[serde(default)]
    disc_number: u32,
    track_number: Option<u32>,
    #[serde(default)]
    uri: String,
    #[serde(default)]
    artist: String,
    #[serde(default)]
    artists: Vec<RemoteArtist>,
    album: Option<RemoteAlbum>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoteAlbum {
    id: String,
    title: String,
    #[serde(default)]
    artist: String,
    year: Option<u32>,
    cover: Option<String>,
    #[serde(default)]
    tracks: Vec<RemoteTrack>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoteArtist {
    id: String,
    name: String,
    picture: Option<String>,
    #[serde(default)]
    albums: Vec<RemoteAlbum>,
    #[serde(default)]
    songs: Vec<RemoteTrack>,
}

#[derive(Deserialize)]
struct RemotePlaylist {
    id: String,
    name: String,
    description: Option<String>,
    #[serde(default)]
    tracks: Vec<RemoteTrack>,
}

#[derive(Deserialize)]
struct RemoteSearch {
    #[serde(default)]
    artists: Vec<RemoteArtist>,
    #[serde(default)]
    albums: Vec<RemoteAlbum>,
    #[serde(default)]
    tracks: Vec<RemoteTrack>,
}

impl From<RemoteTrack> for Track {
    fn from(track: RemoteTrack) -> Self {
        Track {
            id: track.id,
            title: track.title,
            duration: track.duration,
            disc_number: track.disc_number,
            track_number: track.track_number,
            uri: track.uri,
            artist: track.artist,
            artists: track.artists.into_iter().map(Into::into).collect(),
            album: track.album.map(Into::into),
        }
    }
}

impl From<RemoteAlbum> for Album {
    fn from(album: RemoteAlbum) -> Self {
        Album {
            id: album.id,
            title: album.title,
            artist: album.artist,
            artist_id: None,
            year: album.year,
            cover: album.cover,
            tracks: album.tracks.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<RemoteArtist> for Artist {
    fn from(artist: RemoteArtist) -> Self {
        Artist {
            id: artist.id,
            name: artist.name,
            picture: artist.picture,
            albums: artist.albums.into_iter().map(Into::into).collect(),
            songs: artist.songs.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<RemotePlaylist> for Playlist {
    fn from(playlist: RemotePlaylist) -> Self {
        Playlist {
            id: playlist.id,
            name: playlist.name,
            description: playlist.description,
            tracks: playlist.tracks.into_iter().map(Into::into).collect(),
        }
    }
}

pub struct MusicPlayerFactory;

#[async_trait]
impl ProviderFactory for MusicPlayerFactory {
    fn kind(&self) -> &'static str {
        "music-player"
    }

    fn display_name(&self) -> &'static str {
        "music-player"
    }

    /// A peer daemon has no login.
    fn needs_credentials(&self) -> bool {
        false
    }

    fn default_port(&self) -> u16 {
        WEB_PORT
    }

    async fn connect(
        &self,
        config: &ProviderConfig,
    ) -> Result<Arc<dyn MusicProvider>, ProviderError> {
        Ok(Arc::new(MusicPlayer::new(&config.url)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_peers_web_url() {
        let peer = MusicPlayer::new("http://studio.lan:5053/");
        assert_eq!(peer.base_url(), "http://studio.lan:5053");
        assert_eq!(peer.host(), "studio.lan");
    }

    #[test]
    fn the_page_becomes_query_variables() {
        let variables = Page::new(20, 50).with_filter(Some("sabbath"));
        assert_eq!(variables["offset"], 20);
        assert_eq!(variables["limit"], 50);
        assert_eq!(variables["filter"], "sabbath");

        // No filter is a null, not an empty string: the peer treats those
        // differently, and "" would filter everything out.
        assert!(Page::default().with_filter(None)["filter"].is_null());
    }

    #[test]
    fn a_track_survives_the_round_trip() {
        let wire: RemoteTrack = serde_json::from_value(json!({
            "id": "t1",
            "title": "God Is Dead?",
            "duration": 532.3,
            "discNumber": 2,
            "trackNumber": 4,
            "uri": "http://peer:5053/tracks/t1",
            "artist": "Black Sabbath",
            "artists": [{ "id": "a1", "name": "Black Sabbath" }],
            "album": { "id": "al1", "title": "13", "artist": "Black Sabbath", "year": 2013, "cover": "al1.jpg" }
        }))
        .unwrap();

        let track: Track = wire.into();
        assert_eq!(track.disc_number, 2, "multi-disc info has to survive");
        assert_eq!(track.track_number, Some(4));
        assert_eq!(track.album.unwrap().title, "13");
        assert_eq!(track.artists.len(), 1);
    }

    /// The peer omits fields it has nothing for; none of them are required.
    #[test]
    fn a_sparse_track_still_parses() {
        let wire: RemoteTrack = serde_json::from_value(json!({
            "id": "t2",
            "title": "Untitled"
        }))
        .unwrap();
        let track: Track = wire.into();
        assert_eq!(track.disc_number, 0);
        assert!(track.uri.is_empty());
        assert!(track.album.is_none());
    }
}
