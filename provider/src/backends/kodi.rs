//! Kodi (and XBMC, which is what it still calls itself over mDNS).
//!
//! JSON-RPC over HTTP at `/jsonrpc`. Unlike the other backends this one has no
//! search endpoint — its idiom is a `contains` filter on each list call, which
//! is exactly what the trait's default [`search`](crate::MusicProvider::search)
//! does, so it is not overridden.
//!
//! Kodi's ids are integers per media type (`albumid`, `artistid`, `songid`)
//! while the rest of the app uses strings. They are stored as the bare number
//! and parsed back per method; the three id spaces never mix, because
//! `album`, `artist` and `track` are separate calls.

use crate::{
    Album, Artist, MusicProvider, Page, Playlist, ProviderCapabilities, ProviderConfig,
    ProviderError, ProviderFactory, Track,
};
use async_trait::async_trait;
use base64::Engine;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::core::params::ObjectParams;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;

/// What every song listing asks for.
const SONG_PROPERTIES: &[&str] = &[
    "title",
    "artist",
    "artistid",
    "album",
    "albumid",
    "track",
    "disc",
    "duration",
    "year",
    "thumbnail",
    "file",
];

const ALBUM_PROPERTIES: &[&str] = &["title", "artist", "artistid", "year", "thumbnail"];
const ARTIST_PROPERTIES: &[&str] = &["thumbnail"];

pub struct Kodi {
    client: HttpClient,
    base_url: String,
    host: String,
    /// Prefixed onto media urls so the playback engine can fetch them without
    /// knowing anything about Kodi's auth.
    credentials: Option<(String, String)>,
}

impl Kodi {
    fn params(fields: Value) -> Result<ObjectParams, ProviderError> {
        let mut params = ObjectParams::new();
        if let Some(map) = fields.as_object() {
            for (key, value) in map {
                params
                    .insert(key.as_str(), value)
                    .map_err(ProviderError::other)?;
            }
        }
        Ok(params)
    }

    async fn call<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        fields: Value,
    ) -> Result<T, ProviderError> {
        self.client
            .request(method, Self::params(fields)?)
            .await
            .map_err(|e| {
                let message = e.to_string();
                if message.contains("401") || message.to_lowercase().contains("unauthor") {
                    ProviderError::Auth(message)
                } else {
                    ProviderError::Transport(message)
                }
            })
    }

    /// Kodi serves any library file at `/vfs/<url-encoded path>`, which is what
    /// `Files.PrepareDownload` hands back — building it directly saves a round
    /// trip per track.
    ///
    /// The credentials go in the url because whatever plays this is an HTTP
    /// client that knows nothing about Kodi.
    fn media_url(&self, path: &str, prefix: &str) -> String {
        if path.is_empty() {
            return String::new();
        }
        let encoded = urlencoding::encode(path);
        match (&self.credentials, self.base_url.split_once("://")) {
            (Some((user, password)), Some((scheme, rest))) => {
                let user = urlencoding::encode(user);
                let password = urlencoding::encode(password);
                format!("{scheme}://{user}:{password}@{rest}/{prefix}/{encoded}")
            }
            _ => format!("{}/{prefix}/{encoded}", self.base_url),
        }
    }

    fn cover(&self, thumbnail: &str) -> Option<String> {
        if thumbnail.is_empty() {
            return None;
        }
        Some(self.media_url(thumbnail, "image"))
    }

    /// Kodi pages with an inclusive-exclusive window rather than a count.
    fn limits(page: Page) -> Value {
        let start = page.offset.max(0);
        if page.limit <= 0 {
            return json!({ "start": start });
        }
        json!({ "start": start, "end": start + page.limit })
    }

    fn filter(field: &str, query: Option<&str>) -> Option<Value> {
        query
            .map(str::trim)
            .filter(|query| !query.is_empty())
            .map(|query| json!({ "field": field, "operator": "contains", "value": query }))
    }

    fn map_album(&self, album: &KodiAlbum) -> Album {
        Album {
            id: album.albumid.to_string(),
            title: album.title.clone().unwrap_or_else(|| album.label.clone()),
            artist: album.artist.join(", "),
            artist_id: album.artistid.first().map(|id| id.to_string()),
            year: album.year.filter(|year| *year > 0),
            cover: self.cover(&album.thumbnail),
            tracks: vec![],
        }
    }

    fn map_artist(&self, artist: &KodiArtist) -> Artist {
        Artist {
            id: artist.artistid.to_string(),
            name: artist
                .artist
                .clone()
                .unwrap_or_else(|| artist.label.clone()),
            picture: self.cover(&artist.thumbnail),
            albums: vec![],
            songs: vec![],
        }
    }

    fn map_song(&self, song: &KodiSong) -> Track {
        let artist = song.artist.join(", ");
        Track {
            // Not reported on a listing by this server.
            bitrate: None,
            sample_rate: None,
            id: song.songid.to_string(),
            title: song.title.clone().unwrap_or_else(|| song.label.clone()),
            duration: song.duration.map(|seconds| seconds as f32),
            // Kodi counts discs from 1 but reports 0 for a single-disc album.
            disc_number: song.disc.unwrap_or(0).max(0) as u32,
            track_number: song.track.filter(|track| *track > 0).map(|t| t as u32),
            uri: self.media_url(&song.file, "vfs"),
            artists: song
                .artistid
                .iter()
                .zip(song.artist.iter())
                .map(|(id, name)| Artist {
                    id: id.to_string(),
                    name: name.clone(),
                    ..Default::default()
                })
                .collect(),
            album: song.albumid.filter(|id| *id > 0).map(|id| Album {
                id: id.to_string(),
                title: song.album.clone().unwrap_or_default(),
                artist: artist.clone(),
                year: song.year.filter(|year| *year > 0),
                cover: self.cover(&song.thumbnail),
                ..Default::default()
            }),
            artist,
        }
    }
}

#[async_trait]
impl MusicProvider for Kodi {
    fn kind(&self) -> &'static str {
        "kodi"
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
            // Kodi has star ratings, not likes; showing one as the other would
            // be worse than showing nothing.
            liked: false,
            // No search endpoint — the trait's three filtered calls *are* the
            // Kodi idiom.
            native_search: false,
        }
    }

    async fn albums(&self, filter: Option<&str>, page: Page) -> Result<Vec<Album>, ProviderError> {
        let mut fields = json!({
            "properties": ALBUM_PROPERTIES,
            "limits": Self::limits(page),
            "sort": { "method": "label", "order": "ascending" },
        });
        if let Some(filter) = Self::filter("album", filter) {
            fields["filter"] = filter;
        }
        let result: AlbumsResult = self.call("AudioLibrary.GetAlbums", fields).await?;
        Ok(result
            .albums
            .iter()
            .map(|album| self.map_album(album))
            .collect())
    }

    async fn artists(
        &self,
        filter: Option<&str>,
        page: Page,
    ) -> Result<Vec<Artist>, ProviderError> {
        let mut fields = json!({
            "properties": ARTIST_PROPERTIES,
            "limits": Self::limits(page),
            "sort": { "method": "label", "order": "ascending" },
        });
        if let Some(filter) = Self::filter("artist", filter) {
            fields["filter"] = filter;
        }
        let result: ArtistsResult = self.call("AudioLibrary.GetArtists", fields).await?;
        Ok(result
            .artists
            .iter()
            .map(|artist| self.map_artist(artist))
            .collect())
    }

    async fn tracks(&self, filter: Option<&str>, page: Page) -> Result<Vec<Track>, ProviderError> {
        let mut fields = json!({
            "properties": SONG_PROPERTIES,
            "limits": Self::limits(page),
            "sort": { "method": "label", "order": "ascending" },
        });
        if let Some(filter) = Self::filter("title", filter) {
            fields["filter"] = filter;
        }
        let result: SongsResult = self.call("AudioLibrary.GetSongs", fields).await?;
        Ok(result
            .songs
            .iter()
            .map(|song| self.map_song(song))
            .collect())
    }

    async fn album(&self, id: &str) -> Result<Album, ProviderError> {
        let albumid = parse_id(id, "album")?;
        let details: AlbumDetails = self
            .call(
                "AudioLibrary.GetAlbumDetails",
                json!({ "albumid": albumid, "properties": ALBUM_PROPERTIES }),
            )
            .await?;
        let songs: SongsResult = self
            .call(
                "AudioLibrary.GetSongs",
                json!({
                    "properties": SONG_PROPERTIES,
                    "filter": { "albumid": albumid },
                    "sort": { "method": "track", "order": "ascending" },
                }),
            )
            .await?;
        let mut album = self.map_album(&details.albumdetails);
        album.tracks = songs.songs.iter().map(|song| self.map_song(song)).collect();
        Ok(album)
    }

    async fn artist(&self, id: &str) -> Result<Artist, ProviderError> {
        let artistid = parse_id(id, "artist")?;
        let details: ArtistDetails = self
            .call(
                "AudioLibrary.GetArtistDetails",
                json!({ "artistid": artistid, "properties": ARTIST_PROPERTIES }),
            )
            .await?;
        let albums: AlbumsResult = self
            .call(
                "AudioLibrary.GetAlbums",
                json!({
                    "properties": ALBUM_PROPERTIES,
                    "filter": { "artistid": artistid },
                    "sort": { "method": "year", "order": "ascending" },
                }),
            )
            .await?;
        let mut artist = self.map_artist(&details.artistdetails);
        artist.albums = albums
            .albums
            .iter()
            .map(|album| self.map_album(album))
            .collect();
        Ok(artist)
    }

    async fn track(&self, id: &str) -> Result<Track, ProviderError> {
        let songid = parse_id(id, "track")?;
        let details: SongDetails = self
            .call(
                "AudioLibrary.GetSongDetails",
                json!({ "songid": songid, "properties": SONG_PROPERTIES }),
            )
            .await?;
        Ok(self.map_song(&details.songdetails))
    }

    /// Kodi keeps music playlists as files in the profile directory rather
    /// than as library rows, so this is a directory listing.
    async fn playlists(&self, page: Page) -> Result<Vec<Playlist>, ProviderError> {
        let result: DirectoryResult = self
            .call(
                "Files.GetDirectory",
                json!({
                    "directory": "special://profile/playlists/music",
                    "media": "music",
                }),
            )
            .await?;
        Ok(page.slice(
            result
                .files
                .into_iter()
                .filter(|file| file.filetype != "directory")
                .map(|file| Playlist {
                    id: file.file,
                    name: file.label,
                    description: None,
                    track_count: None,
                    tracks: vec![],
                })
                .collect(),
        ))
    }

    async fn playlist(&self, id: &str) -> Result<Playlist, ProviderError> {
        let result: DirectoryResult = self
            .call(
                "Files.GetDirectory",
                json!({ "directory": id, "media": "music", "properties": SONG_PROPERTIES }),
            )
            .await?;
        Ok(Playlist {
            id: id.to_string(),
            name: id.rsplit('/').next().unwrap_or(id).to_string(),
            description: None,
            track_count: None,
            tracks: result
                .files
                .iter()
                .filter(|file| file.filetype != "directory")
                .map(|file| Track {
                    id: file.id.unwrap_or_default().to_string(),
                    title: file.label.clone(),
                    uri: self.media_url(&file.file, "vfs"),
                    ..Default::default()
                })
                .collect(),
        })
    }

    async fn ping(&self) -> Result<(), ProviderError> {
        let _: Value = self.call("JSONRPC.Ping", json!({})).await?;
        Ok(())
    }
}

fn parse_id(id: &str, what: &str) -> Result<i64, ProviderError> {
    id.parse()
        .map_err(|_| ProviderError::NotFound(format!("{what} {id}")))
}

// ── Kodi's wire shapes ──────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
struct AlbumsResult {
    #[serde(default)]
    albums: Vec<KodiAlbum>,
}

#[derive(Deserialize)]
struct AlbumDetails {
    albumdetails: KodiAlbum,
}

#[derive(Deserialize, Default)]
struct ArtistsResult {
    #[serde(default)]
    artists: Vec<KodiArtist>,
}

#[derive(Deserialize)]
struct ArtistDetails {
    artistdetails: KodiArtist,
}

#[derive(Deserialize, Default)]
struct SongsResult {
    #[serde(default)]
    songs: Vec<KodiSong>,
}

#[derive(Deserialize)]
struct SongDetails {
    songdetails: KodiSong,
}

#[derive(Deserialize, Default)]
struct DirectoryResult {
    #[serde(default)]
    files: Vec<KodiFile>,
}

#[derive(Deserialize, Default)]
struct KodiFile {
    #[serde(default)]
    label: String,
    #[serde(default)]
    file: String,
    #[serde(default)]
    filetype: String,
    id: Option<i64>,
}

#[derive(Deserialize, Default)]
struct KodiAlbum {
    albumid: i64,
    #[serde(default)]
    label: String,
    title: Option<String>,
    #[serde(default)]
    artist: Vec<String>,
    #[serde(default)]
    artistid: Vec<i64>,
    year: Option<u32>,
    #[serde(default)]
    thumbnail: String,
}

#[derive(Deserialize, Default)]
struct KodiArtist {
    artistid: i64,
    #[serde(default)]
    label: String,
    artist: Option<String>,
    #[serde(default)]
    thumbnail: String,
}

#[derive(Deserialize, Default)]
struct KodiSong {
    songid: i64,
    #[serde(default)]
    label: String,
    title: Option<String>,
    #[serde(default)]
    artist: Vec<String>,
    #[serde(default)]
    artistid: Vec<i64>,
    album: Option<String>,
    albumid: Option<i64>,
    track: Option<i64>,
    disc: Option<i64>,
    duration: Option<i64>,
    year: Option<u32>,
    #[serde(default)]
    thumbnail: String,
    #[serde(default)]
    file: String,
}

pub struct KodiFactory;

#[async_trait]
impl ProviderFactory for KodiFactory {
    fn kind(&self) -> &'static str {
        "kodi"
    }

    /// Kodi still advertises itself as `xbmc` over mDNS, and that string is
    /// already flowing through discovery.
    fn aliases(&self) -> &'static [&'static str] {
        &["xbmc"]
    }

    fn display_name(&self) -> &'static str {
        "Kodi"
    }

    fn default_port(&self) -> u16 {
        8080
    }

    async fn connect(
        &self,
        config: &ProviderConfig,
    ) -> Result<Arc<dyn MusicProvider>, ProviderError> {
        let mut builder = HttpClientBuilder::default().request_timeout(Duration::from_secs(20));

        let credentials = match (&config.username, &config.password) {
            (Some(user), password) if !user.is_empty() => {
                let password = password.clone().unwrap_or_default();
                let raw = format!("{user}:{password}");
                let encoded = base64::engine::general_purpose::STANDARD.encode(raw);
                let mut headers = jsonrpsee::http_client::HeaderMap::new();
                headers.insert(
                    "authorization",
                    jsonrpsee::http_client::HeaderValue::from_str(&format!("Basic {encoded}"))
                        .map_err(ProviderError::other)?,
                );
                builder = builder.set_headers(headers);
                Some((user.clone(), password))
            }
            _ => None,
        };

        let client = builder
            .build(format!("{}/jsonrpc", config.url))
            .map_err(ProviderError::transport)?;

        let kodi = Kodi {
            client,
            base_url: config.url.clone(),
            host: config.host(),
            credentials,
        };
        kodi.ping().await?;
        Ok(Arc::new(kodi))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kodi(credentials: Option<(&str, &str)>) -> Kodi {
        Kodi {
            client: HttpClientBuilder::default()
                .build("http://kodi.lan:8080/jsonrpc")
                .unwrap(),
            base_url: "http://kodi.lan:8080".into(),
            host: "kodi.lan".into(),
            credentials: credentials.map(|(u, p)| (u.to_string(), p.to_string())),
        }
    }

    /// Kodi pages with a window, not a count.
    #[test]
    fn a_page_becomes_a_window() {
        assert_eq!(
            Kodi::limits(Page::new(20, 50)),
            json!({"start": 20, "end": 70})
        );
        // No limit means no upper bound rather than an end of zero.
        assert_eq!(Kodi::limits(Page::all()), json!({"start": 0}));
    }

    #[test]
    fn an_empty_filter_is_no_filter() {
        assert!(Kodi::filter("album", None).is_none());
        assert!(Kodi::filter("album", Some("   ")).is_none());
        assert_eq!(
            Kodi::filter("album", Some("sabbath")).unwrap(),
            json!({"field": "album", "operator": "contains", "value": "sabbath"})
        );
    }

    /// Whatever plays this is an HTTP client that knows nothing about Kodi, so
    /// the credentials have to be in the url.
    #[test]
    fn media_urls_carry_the_login() {
        let url = kodi(Some(("kodi", "pa ss"))).media_url("/music/a.mp3", "vfs");
        assert_eq!(
            url,
            "http://kodi:pa%20ss@kodi.lan:8080/vfs/%2Fmusic%2Fa.mp3"
        );

        let anonymous = kodi(None).media_url("/music/a.mp3", "vfs");
        assert_eq!(anonymous, "http://kodi.lan:8080/vfs/%2Fmusic%2Fa.mp3");
    }

    #[test]
    fn an_absent_file_yields_no_url() {
        assert!(kodi(None).media_url("", "vfs").is_empty());
        assert!(kodi(None).cover("").is_none());
    }

    #[test]
    fn a_song_maps_across() {
        let song: KodiSong = serde_json::from_value(json!({
            "songid": 42,
            "label": "God Is Dead?",
            "title": "God Is Dead?",
            "artist": ["Black Sabbath"],
            "artistid": [7],
            "album": "13",
            "albumid": 3,
            "track": 2,
            "disc": 1,
            "duration": 532,
            "year": 2013,
            "thumbnail": "image://foo/",
            "file": "/music/13/02.mp3"
        }))
        .unwrap();

        let track = kodi(None).map_song(&song);
        assert_eq!(track.id, "42");
        assert_eq!(track.track_number, Some(2));
        assert_eq!(track.disc_number, 1);
        assert_eq!(track.duration, Some(532.0));
        assert_eq!(track.artist, "Black Sabbath");
        assert_eq!(track.album.as_ref().unwrap().id, "3");
        assert!(track.uri.ends_with("/vfs/%2Fmusic%2F13%2F02.mp3"));
    }

    /// Kodi omits what it has nothing for; none of it is required.
    #[test]
    fn a_sparse_song_still_maps() {
        let song: KodiSong = serde_json::from_value(json!({
            "songid": 1,
            "label": "Untitled"
        }))
        .unwrap();
        let track = kodi(None).map_song(&song);
        assert_eq!(track.title, "Untitled");
        assert_eq!(track.track_number, None);
        assert!(track.album.is_none());
        assert!(track.uri.is_empty());
    }

    /// The three id spaces are separate calls, so a bare integer is enough —
    /// but it does have to be one.
    #[test]
    fn a_non_numeric_id_is_not_found() {
        assert!(parse_id("42", "album").is_ok());
        assert!(matches!(
            parse_id("abc", "album"),
            Err(ProviderError::NotFound(_))
        ));
    }

    /// Kodi reports 0 for a single-disc album; the app counts from 1 only when
    /// it means something.
    #[test]
    fn a_single_disc_album_reports_no_disc() {
        let song: KodiSong = serde_json::from_value(json!({ "songid": 1, "disc": 0 })).unwrap();
        assert_eq!(kodi(None).map_song(&song).disc_number, 0);
    }
}
