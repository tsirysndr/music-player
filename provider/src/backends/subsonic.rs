//! Subsonic, Navidrome, Airsonic, gonic — anything speaking the Subsonic API.
//!
//! Auth is Subsonic's salted-token scheme, so the password never crosses the
//! wire and every stream url is pre-authenticated. That is why
//! [`crate::url::decorate`] leaves these uris alone: rewriting one would strip
//! the token off it.

use crate::{
    http, Album, Artist, MusicProvider, Page, Playlist, ProviderCapabilities, ProviderConfig,
    ProviderError, ProviderFactory, Track,
};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;
use url::Url;

const API_VERSION: &str = "1.16.1";
const CLIENT_NAME: &str = "music-player";

/// Subsonic (Navidrome, Airsonic, gonic, ...) source addon.
pub struct Subsonic {
    base_url: String,
    username: String,
    password: String,
    salt: String,
    host: String,
    connected: bool,
}

impl Subsonic {
    pub fn new() -> Self {
        let salt = generate_salt();
        Self {
            base_url: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            salt,
            host: "".to_string(),
            connected: false,
        }
    }

    pub fn with_credentials(base_url: &str, username: &str, password: &str) -> Self {
        let base_url = base_url.trim().trim_end_matches('/').to_string();
        let host = Url::parse(&base_url)
            .ok()
            .and_then(|url| url.host_str().map(|host| host.to_string()))
            .unwrap_or_default();
        Self {
            base_url,
            username: username.to_string(),
            password: password.to_string(),
            host,
            ..Subsonic::new()
        }
    }

    pub async fn connect(&mut self) -> Result<(), ProviderError> {
        if self.base_url.is_empty() {
            return Err(ProviderError::Other(
                "a Subsonic server needs a url".to_string(),
            ));
        }
        let url = self.api_url("ping", &[])?;
        let _response = request(url).await?;
        self.connected = true;
        Ok(())
    }

    fn token(&self) -> String {
        format!(
            "{:x}",
            md5::compute(format!("{}{}", self.password, self.salt))
        )
    }

    fn api_url(&self, endpoint: &str, params: &[(&str, &str)]) -> Result<Url, ProviderError> {
        let mut url = Url::parse(&format!("{}/rest/{}", self.base_url, endpoint))
            .map_err(ProviderError::other)?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("u", &self.username);
            query.append_pair("t", &self.token());
            query.append_pair("s", &self.salt);
            query.append_pair("v", API_VERSION);
            query.append_pair("c", CLIENT_NAME);
            query.append_pair("f", "json");
            for (key, value) in params {
                query.append_pair(key, value);
            }
        }
        Ok(url)
    }

    pub fn stream_url(&self, song_id: &str) -> String {
        match Url::parse(&format!("{}/rest/stream", self.base_url)) {
            Ok(mut url) => {
                {
                    let mut query = url.query_pairs_mut();
                    query.append_pair("id", song_id);
                    query.append_pair("u", &self.username);
                    query.append_pair("t", &self.token());
                    query.append_pair("s", &self.salt);
                    query.append_pair("v", API_VERSION);
                    query.append_pair("c", CLIENT_NAME);
                }
                url.to_string()
            }
            Err(_) => "".to_string(),
        }
    }

    pub fn cover_art_url(&self, cover_art_id: &str) -> String {
        match Url::parse(&format!("{}/rest/getCoverArt", self.base_url)) {
            Ok(mut url) => {
                {
                    let mut query = url.query_pairs_mut();
                    query.append_pair("id", cover_art_id);
                    query.append_pair("u", &self.username);
                    query.append_pair("t", &self.token());
                    query.append_pair("s", &self.salt);
                    query.append_pair("v", API_VERSION);
                    query.append_pair("c", CLIENT_NAME);
                }
                url.to_string()
            }
            Err(_) => "".to_string(),
        }
    }

    fn map_song(&self, song: &Child) -> Track {
        let artist = song.artist.clone().unwrap_or_else(|| "None".to_string());
        let artist_id = song
            .artist_id
            .clone()
            .unwrap_or_else(|| format!("{:x}", md5::compute(&artist)));
        let album_title = song.album.clone().unwrap_or_else(|| "None".to_string());
        let album_id = song
            .album_id
            .clone()
            .unwrap_or_else(|| format!("{:x}", md5::compute(&album_title)));
        Track {
            id: song.id.clone(),
            title: song.title.clone(),
            duration: song.duration.map(|duration| duration as f32),
            disc_number: song.disc_number.unwrap_or(1),
            track_number: song.track,
            uri: self.stream_url(&song.id),
            artists: vec![Artist {
                id: artist_id,
                name: artist.clone(),
                ..Default::default()
            }],
            album: Some(Album {
                id: album_id,
                title: album_title,
                artist: artist.clone(),
                year: song.year,
                cover: song
                    .cover_art
                    .as_ref()
                    .map(|cover_art| self.cover_art_url(cover_art)),
                ..Default::default()
            }),
            artist,
        }
    }

    fn map_album(&self, album: &AlbumID3) -> Album {
        Album {
            id: album.id.clone(),
            title: album.name.clone(),
            artist: album.artist.clone().unwrap_or_else(|| "None".to_string()),
            artist_id: album.artist_id.clone(),
            year: album.year,
            cover: album
                .cover_art
                .as_ref()
                .map(|cover_art| self.cover_art_url(cover_art)),
            tracks: album.song.iter().map(|song| self.map_song(song)).collect(),
        }
    }

    fn map_artist(&self, artist: &ArtistID3) -> Artist {
        Artist {
            id: artist.id.clone(),
            name: artist.name.clone(),
            picture: artist
                .cover_art
                .as_ref()
                .map(|cover_art| self.cover_art_url(cover_art)),
            albums: artist
                .album
                .iter()
                .map(|album| self.map_album(album))
                .collect(),
            songs: vec![],
        }
    }

    fn map_playlist(&self, playlist: &PlaylistID3) -> Playlist {
        Playlist {
            id: playlist.id.clone(),
            name: playlist.name.clone(),
            description: playlist.comment.clone(),
            tracks: playlist
                .entry
                .iter()
                .map(|song| self.map_song(song))
                .collect(),
        }
    }
}

#[async_trait]
impl MusicProvider for Subsonic {
    fn kind(&self) -> &'static str {
        "subsonic"
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
        let limit = normalize_limit(page.limit);
        let offset = page.offset.max(0);
        match filter {
            Some(query) if !query.trim().is_empty() => {
                let url = self.api_url(
                    "search3",
                    &[
                        ("query", query),
                        ("artistCount", "0"),
                        ("songCount", "0"),
                        ("albumCount", &limit.to_string()),
                        ("albumOffset", &offset.to_string()),
                    ],
                )?;
                let response = request(url).await?;
                let result = response.search_result3.unwrap_or_default();
                Ok(result
                    .album
                    .iter()
                    .map(|album| self.map_album(album))
                    .collect())
            }
            _ => {
                let url = self.api_url(
                    "getAlbumList2",
                    &[
                        ("type", "alphabeticalByName"),
                        ("size", &limit.to_string()),
                        ("offset", &offset.to_string()),
                    ],
                )?;
                let response = request(url).await?;
                let list = response.album_list2.unwrap_or_default();
                Ok(list
                    .album
                    .iter()
                    .map(|album| self.map_album(album))
                    .collect())
            }
        }
    }

    /// `getArtists` returns the whole index in one go — there is no server-side
    /// paging for it — so the filter and the page are applied here.
    async fn artists(&self, filter: Option<&str>, page: Page) -> Result<Vec<Artist>, ProviderError> {
        let url = self.api_url("getArtists", &[])?;
        let response = request(url).await?;
        let artists = response.artists.unwrap_or_default();
        let filter = filter.unwrap_or_default().trim().to_lowercase();
        Ok(artists
            .index
            .iter()
            .flat_map(|index| index.artist.iter())
            .filter(|artist| filter.is_empty() || artist.name.to_lowercase().contains(&filter))
            .skip(page.offset.max(0) as usize)
            .take(normalize_limit(page.limit) as usize)
            .map(|artist| self.map_artist(artist))
            .collect())
    }

    async fn tracks(&self, filter: Option<&str>, page: Page) -> Result<Vec<Track>, ProviderError> {
        let limit = normalize_limit(page.limit);
        let offset = page.offset.max(0);
        let url = self.api_url(
            "search3",
            &[
                ("query", filter.unwrap_or_default()),
                ("artistCount", "0"),
                ("albumCount", "0"),
                ("songCount", &limit.to_string()),
                ("songOffset", &offset.to_string()),
            ],
        )?;
        let response = request(url).await?;
        let result = response.search_result3.unwrap_or_default();
        Ok(result.song.iter().map(|song| self.map_song(song)).collect())
    }

    async fn album(&self, id: &str) -> Result<Album, ProviderError> {
        let url = self.api_url("getAlbum", &[("id", id)])?;
        let response = request(url).await?;
        match response.album {
            Some(ref album) => Ok(self.map_album(album)),
            None => Err(ProviderError::NotFound(format!("album {id}"))),
        }
    }

    async fn artist(&self, id: &str) -> Result<Artist, ProviderError> {
        let url = self.api_url("getArtist", &[("id", id)])?;
        let response = request(url).await?;
        match response.artist {
            Some(ref artist) => Ok(self.map_artist(artist)),
            None => Err(ProviderError::NotFound(format!("artist {id}"))),
        }
    }

    async fn track(&self, id: &str) -> Result<Track, ProviderError> {
        let url = self.api_url("getSong", &[("id", id)])?;
        let response = request(url).await?;
        match response.song {
            Some(ref song) => Ok(self.map_song(song)),
            None => Err(ProviderError::NotFound(format!("track {id}"))),
        }
    }

    async fn playlists(&self, page: Page) -> Result<Vec<Playlist>, ProviderError> {
        let url = self.api_url("getPlaylists", &[])?;
        let response = request(url).await?;
        let playlists = response.playlists.unwrap_or_default();
        Ok(playlists
            .playlist
            .iter()
            .skip(page.offset.max(0) as usize)
            .take(normalize_limit(page.limit) as usize)
            .map(|playlist| self.map_playlist(playlist))
            .collect())
    }

    async fn playlist(&self, id: &str) -> Result<Playlist, ProviderError> {
        let url = self.api_url("getPlaylist", &[("id", id)])?;
        let response = request(url).await?;
        match response.playlist {
            Some(ref playlist) => Ok(self.map_playlist(playlist)),
            None => Err(ProviderError::NotFound(format!("playlist {id}"))),
        }
    }

    /// Subsonic's starred songs, which is what a "like" is on this server.
    async fn liked_tracks(&self, page: Page) -> Result<Vec<Track>, ProviderError> {
        let url = self.api_url("getStarred2", &[])?;
        let response = request(url).await?;
        let starred = response.starred2.unwrap_or_default();
        Ok(starred
            .song
            .iter()
            .skip(page.offset.max(0) as usize)
            .take(normalize_limit(page.limit) as usize)
            .map(|song| self.map_song(song))
            .collect())
    }

    async fn set_liked(&self, id: &str, liked: bool) -> Result<(), ProviderError> {
        let endpoint = if liked { "star" } else { "unstar" };
        let url = self.api_url(endpoint, &[("id", id)])?;
        request(url).await.map(|_| ())
    }

    /// One search call rather than the trait's three.
    async fn search(&self, keyword: &str, page: Page) -> Result<crate::SearchResults, ProviderError> {
        let limit = normalize_limit(page.limit);
        let url = self.api_url(
            "search3",
            &[
                ("query", keyword),
                ("artistCount", &limit.to_string()),
                ("albumCount", &limit.to_string()),
                ("songCount", &limit.to_string()),
            ],
        )?;
        let response = request(url).await?;
        let result = response.search_result3.unwrap_or_default();
        Ok(crate::SearchResults {
            artists: result
                .artist
                .iter()
                .map(|artist| self.map_artist(artist))
                .collect(),
            albums: result
                .album
                .iter()
                .map(|album| self.map_album(album))
                .collect(),
            tracks: result.song.iter().map(|song| self.map_song(song)).collect(),
        })
    }

    async fn ping(&self) -> Result<(), ProviderError> {
        let url = self.api_url("ping", &[])?;
        request(url).await.map(|_| ())
    }
}

pub struct SubsonicFactory;

#[async_trait]
impl ProviderFactory for SubsonicFactory {
    fn kind(&self) -> &'static str {
        "subsonic"
    }

    /// Navidrome speaks this API and advertises under its own name.
    fn aliases(&self) -> &'static [&'static str] {
        &["navidrome", "airsonic", "gonic"]
    }

    fn display_name(&self) -> &'static str {
        "Subsonic / Navidrome"
    }

    fn default_port(&self) -> u16 {
        4533
    }

    async fn connect(&self, config: &ProviderConfig) -> Result<Arc<dyn MusicProvider>, ProviderError> {
        let mut client = Subsonic::with_credentials(
            &config.url,
            config.username.as_deref().unwrap_or_default(),
            config.password.as_deref().unwrap_or_default(),
        );
        client.connect().await?;
        Ok(Arc::new(client))
    }
}

fn normalize_limit(limit: i32) -> i32 {
    if limit <= 0 {
        500
    } else {
        limit.min(500)
    }
}

fn generate_salt() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let seed = format!("{}-{:?}", now.as_nanos(), std::time::Instant::now());
    format!("{:x}", md5::compute(seed))[..12].to_string()
}

async fn request(url: Url) -> Result<ResponseBody, ProviderError> {
    let response = http::client()
        .get(url.as_str())
        .send()
        .await
        .map_err(ProviderError::transport)?;
    let status = response.status();
    if !status.is_success() {
        return Err(match status.as_u16() {
            401 | 403 => ProviderError::Auth(format!("the server refused the login ({status})")),
            _ => ProviderError::Transport(format!("the server answered {status}")),
        });
    }
    let envelope: SubsonicResponse = response.json().await.map_err(ProviderError::transport)?;
    envelope.into_result()
}

#[derive(Debug, Deserialize)]
pub struct SubsonicResponse {
    #[serde(rename = "subsonic-response")]
    pub subsonic_response: ResponseBody,
}

impl SubsonicResponse {
    pub fn into_result(self) -> Result<ResponseBody, ProviderError> {
        let body = self.subsonic_response;
        if body.status != "ok" {
            let code = body.error.as_ref().map(|error| error.code);
            let message = body
                .error
                .map(|error| error.message)
                .unwrap_or_else(|| "unknown Subsonic error".to_string());
            // A Subsonic server reports a bad login as a 200 with an error
            // body, so the code is the only way to tell it from a real fault.
            return Err(match code {
                Some(40) | Some(41) | Some(50) => ProviderError::Auth(message),
                Some(70) => ProviderError::NotFound(message),
                _ => ProviderError::Other(message),
            });
        }
        Ok(body)
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseBody {
    pub status: String,
    pub error: Option<SubsonicError>,
    pub artists: Option<ArtistsID3>,
    pub artist: Option<ArtistID3>,
    pub album_list2: Option<AlbumList2>,
    pub album: Option<AlbumID3>,
    pub song: Option<Child>,
    pub search_result3: Option<SearchResult3>,
    #[serde(rename = "starred2")]
    pub starred2: Option<Starred2>,
    pub playlists: Option<Playlists>,
    pub playlist: Option<PlaylistID3>,
}

#[derive(Debug, Deserialize)]
pub struct SubsonicError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct ArtistsID3 {
    #[serde(default)]
    pub index: Vec<IndexID3>,
}

#[derive(Debug, Deserialize)]
pub struct IndexID3 {
    pub name: String,
    #[serde(default)]
    pub artist: Vec<ArtistID3>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistID3 {
    pub id: String,
    pub name: String,
    pub cover_art: Option<String>,
    pub album_count: Option<i32>,
    #[serde(default)]
    pub album: Vec<AlbumID3>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumID3 {
    pub id: String,
    pub name: String,
    pub artist: Option<String>,
    pub artist_id: Option<String>,
    pub cover_art: Option<String>,
    pub song_count: Option<i32>,
    pub duration: Option<i32>,
    pub year: Option<u32>,
    #[serde(default)]
    pub song: Vec<Child>,
}

#[derive(Debug, Default, Deserialize)]
pub struct AlbumList2 {
    #[serde(default)]
    pub album: Vec<AlbumID3>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Child {
    pub id: String,
    pub title: String,
    pub album: Option<String>,
    pub album_id: Option<String>,
    pub artist: Option<String>,
    pub artist_id: Option<String>,
    pub track: Option<u32>,
    pub disc_number: Option<u32>,
    pub year: Option<u32>,
    pub duration: Option<i32>,
    pub cover_art: Option<String>,
    pub content_type: Option<String>,
    pub suffix: Option<String>,
    pub path: Option<String>,
}

/// `getStarred2` — the songs the user has starred on this server.
#[derive(Debug, Default, Deserialize)]
pub struct Starred2 {
    #[serde(default)]
    pub song: Vec<Child>,
}

#[derive(Debug, Default, Deserialize)]
pub struct SearchResult3 {
    #[serde(default)]
    pub artist: Vec<ArtistID3>,
    #[serde(default)]
    pub album: Vec<AlbumID3>,
    #[serde(default)]
    pub song: Vec<Child>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Playlists {
    #[serde(default)]
    pub playlist: Vec<PlaylistID3>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistID3 {
    pub id: String,
    pub name: String,
    pub comment: Option<String>,
    pub song_count: Option<i32>,
    pub duration: Option<i32>,
    #[serde(default)]
    pub entry: Vec<Child>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client() -> Subsonic {
        let mut client = Subsonic::with_credentials("https://music.example.com/", "demo", "demo");
        client.salt = "abcdef123456".to_string();
        client
    }

    #[test]
    fn parses_ping_failure() {
        let json = r#"{
            "subsonic-response": {
                "status": "failed",
                "version": "1.16.1",
                "error": { "code": 40, "message": "Wrong username or password" }
            }
        }"#;
        let envelope: SubsonicResponse = serde_json::from_str(json).unwrap();
        let err = envelope.into_result().unwrap_err();
        assert!(err.to_string().contains("Wrong username or password"));
    }

    #[test]
    fn stream_url_contains_token_auth_params() {
        let client = client();
        let url = client.stream_url("track-1");
        assert!(url.starts_with("https://music.example.com/rest/stream?id=track-1"));
        assert!(url.contains("u=demo"));
        assert!(url.contains(&format!("t={}", client.token())));
        assert!(url.contains("s=abcdef123456"));
        assert!(url.contains("v=1.16.1"));
        assert!(url.contains("c=music-player"));
        // token = md5(password + salt)
        assert_eq!(
            client.token(),
            format!("{:x}", md5::compute("demoabcdef123456"))
        );
    }

    #[test]
    fn parses_get_artists_and_flattens_indexes() {
        let json = r#"{
            "subsonic-response": {
                "status": "ok",
                "version": "1.16.1",
                "artists": {
                    "ignoredArticles": "The El La",
                    "index": [
                        { "name": "B", "artist": [
                            { "id": "ar-1", "name": "Bonobo", "coverArt": "ar-1", "albumCount": 2 }
                        ]},
                        { "name": "D", "artist": [
                            { "id": "ar-2", "name": "Daft Punk", "albumCount": 4 },
                            { "id": "ar-3", "name": "Deluan", "albumCount": 1 }
                        ]}
                    ]
                }
            }
        }"#;
        let envelope: SubsonicResponse = serde_json::from_str(json).unwrap();
        let body = envelope.into_result().unwrap();
        let client = client();
        let artists: Vec<Artist> = body
            .artists
            .unwrap()
            .index
            .iter()
            .flat_map(|index| index.artist.iter())
            .map(|artist| client.map_artist(artist))
            .collect();
        assert_eq!(artists.len(), 3);
        assert_eq!(artists[0].id, "ar-1");
        assert_eq!(artists[0].name, "Bonobo");
        assert!(artists[0]
            .picture
            .as_ref()
            .unwrap()
            .starts_with("https://music.example.com/rest/getCoverArt?id=ar-1"));
        assert_eq!(artists[2].name, "Deluan");
    }

    #[test]
    fn parses_get_album_with_songs() {
        let json = r#"{
            "subsonic-response": {
                "status": "ok",
                "version": "1.16.1",
                "album": {
                    "id": "al-1",
                    "name": "Discovery",
                    "artist": "Daft Punk",
                    "artistId": "ar-2",
                    "coverArt": "al-1",
                    "songCount": 2,
                    "duration": 620,
                    "year": 2001,
                    "song": [
                        {
                            "id": "tr-1",
                            "title": "One More Time",
                            "album": "Discovery",
                            "albumId": "al-1",
                            "artist": "Daft Punk",
                            "artistId": "ar-2",
                            "track": 1,
                            "discNumber": 1,
                            "year": 2001,
                            "duration": 320,
                            "coverArt": "al-1",
                            "contentType": "audio/mpeg",
                            "suffix": "mp3",
                            "path": "Daft Punk/Discovery/01 - One More Time.mp3"
                        },
                        {
                            "id": "tr-2",
                            "title": "Aerodynamic",
                            "album": "Discovery",
                            "albumId": "al-1",
                            "artist": "Daft Punk",
                            "artistId": "ar-2",
                            "track": 2,
                            "duration": 300
                        }
                    ]
                }
            }
        }"#;
        let envelope: SubsonicResponse = serde_json::from_str(json).unwrap();
        let body = envelope.into_result().unwrap();
        let client = client();
        let album = client.map_album(&body.album.unwrap());
        assert_eq!(album.id, "al-1");
        assert_eq!(album.title, "Discovery");
        assert_eq!(album.artist, "Daft Punk");
        assert_eq!(album.artist_id, Some("ar-2".to_string()));
        assert_eq!(album.year, Some(2001));
        assert!(album
            .cover
            .as_ref()
            .unwrap()
            .starts_with("https://music.example.com/rest/getCoverArt?id=al-1"));
        assert_eq!(album.tracks.len(), 2);
        let track = &album.tracks[0];
        assert_eq!(track.id, "tr-1");
        assert_eq!(track.title, "One More Time");
        assert_eq!(track.duration, Some(320.0));
        assert_eq!(track.track_number, Some(1));
        assert!(track
            .uri
            .starts_with("https://music.example.com/rest/stream?id=tr-1"));
        assert!(track.uri.contains("c=music-player"));
        assert_eq!(track.album.as_ref().unwrap().id, "al-1");
        assert_eq!(track.artists[0].id, "ar-2");
    }

    #[test]
    fn parses_search3_songs() {
        let json = r#"{
            "subsonic-response": {
                "status": "ok",
                "version": "1.16.1",
                "searchResult3": {
                    "song": [
                        {
                            "id": "tr-9",
                            "title": "Kiara",
                            "album": "Black Sands",
                            "albumId": "al-9",
                            "artist": "Bonobo",
                            "artistId": "ar-1",
                            "duration": 234
                        }
                    ]
                }
            }
        }"#;
        let envelope: SubsonicResponse = serde_json::from_str(json).unwrap();
        let body = envelope.into_result().unwrap();
        let client = client();
        let result = body.search_result3.unwrap();
        let tracks: Vec<Track> = result
            .song
            .iter()
            .map(|song| client.map_song(song))
            .collect();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].artist, "Bonobo");
        assert_eq!(tracks[0].duration, Some(234.0));
        assert!(tracks[0]
            .uri
            .starts_with("https://music.example.com/rest/stream?id=tr-9"));
    }

    /// Live smoke test against a real Subsonic-compatible server.
    ///
    /// Start one with:
    ///   docker run -d -p 4544:4533 -v /path/to/music:/music:ro deluan/navidrome
    ///   curl -X POST http://localhost:4544/auth/createAdmin \
    ///     -d '{"username":"admin","password":"admin"}'
    /// Override with SUBSONIC_TEST_URL / SUBSONIC_TEST_USERNAME / SUBSONIC_TEST_PASSWORD.
    /// Run with: cargo test -p music-player-provider -- --ignored
    #[test]
    #[ignore = "requires a running Navidrome/Subsonic server"]
    fn live_navidrome_smoke() {
        let url = std::env::var("SUBSONIC_TEST_URL")
            .unwrap_or_else(|_| "http://localhost:4544".to_string());
        let username =
            std::env::var("SUBSONIC_TEST_USERNAME").unwrap_or_else(|_| "admin".to_string());
        let password =
            std::env::var("SUBSONIC_TEST_PASSWORD").unwrap_or_else(|_| "admin".to_string());
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async move {
            let mut client = Subsonic::with_credentials(&url, &username, &password);
            client.connect().await.expect("ping failed");

            let artists = client
                .artists(None, Page::new(0, 50))
                .await
                .expect("getArtists failed");
            assert!(!artists.is_empty(), "expected at least one artist");

            let albums = client
                .albums(None, Page::new(0, 50))
                .await
                .expect("getAlbumList2 failed");
            assert!(!albums.is_empty(), "expected at least one album");

            let tracks = client.tracks(None, Page::new(0, 50)).await.expect("search3 failed");
            assert!(!tracks.is_empty(), "expected at least one track");
            for track in &tracks {
                assert!(track.uri.contains("/rest/stream?id="));
                assert!(track.album.is_some());
            }

            let album = client.album(&albums[0].id).await.expect("getAlbum failed");
            assert!(!album.tracks.is_empty(), "expected album songs");

            let artist = client
                .artist(&artists[0].id)
                .await
                .expect("getArtist failed");
            assert!(!artist.albums.is_empty(), "expected artist albums");

            let track = client.track(&tracks[0].id).await.expect("getSong failed");

            // The stream uri must be directly fetchable (this is what the
            // playback engine will request).
            let response = http::client()
                .get(&track.uri)
                .send()
                .await
                .expect("stream request failed");
            assert!(
                response.status().is_success(),
                "stream url returned {}",
                response.status()
            );
            let body = response.bytes().await.expect("stream body failed");
            assert!(!body.is_empty(), "expected stream bytes");

            client.playlists(Page::new(0, 50)).await.expect("getPlaylists failed");
        });
    }

    #[test]
    fn parses_playlists() {
        let json = r#"{
            "subsonic-response": {
                "status": "ok",
                "version": "1.16.1",
                "playlists": {
                    "playlist": [
                        { "id": "pl-1", "name": "Chill", "comment": "chill vibes", "songCount": 10, "duration": 2400 },
                        { "id": "pl-2", "name": "Focus", "songCount": 5, "duration": 1200 }
                    ]
                }
            }
        }"#;
        let envelope: SubsonicResponse = serde_json::from_str(json).unwrap();
        let body = envelope.into_result().unwrap();
        let client = client();
        let playlists: Vec<Playlist> = body
            .playlists
            .unwrap()
            .playlist
            .iter()
            .map(|playlist| client.map_playlist(playlist))
            .collect();
        assert_eq!(playlists.len(), 2);
        assert_eq!(playlists[0].id, "pl-1");
        assert_eq!(playlists[0].name, "Chill");
        assert_eq!(playlists[0].description, Some("chill vibes".to_string()));
    }
}
