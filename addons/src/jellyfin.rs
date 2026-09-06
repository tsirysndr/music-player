use super::{Addon, Browsable};
use anyhow::Error;
use async_trait::async_trait;
use music_player_settings::read_settings;
use music_player_settings::Settings;
use music_player_types::types::{Album, Artist, Device, Playlist, Track};
use serde::{Deserialize, Serialize};
use url::Url;

const CLIENT_NAME: &str = "music-player";
const CLIENT_VERSION: &str = "0.2.1";
/// Jellyfin RunTimeTicks are expressed in 100ns units.
const TICKS_PER_SECOND: f64 = 10_000_000.0;

/// Jellyfin source addon.
pub struct Jellyfin {
    name: String,
    version: String,
    author: String,
    description: String,
    enabled: bool,
    base_url: String,
    username: String,
    password: String,
    token: Option<String>,
    user_id: String,
    host: String,
}

impl Jellyfin {
    pub fn new() -> Self {
        Self {
            name: "Jellyfin".to_string(),
            version: "0.1.0".to_string(),
            author: "Tsiry Sandratraina".to_string(),
            description: "Jellyfin addon".to_string(),
            enabled: true,
            base_url: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            token: None,
            user_id: "".to_string(),
            host: "".to_string(),
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
            ..Jellyfin::new()
        }
    }

    pub async fn connect(&mut self) -> Result<(), Error> {
        if self.base_url.is_empty() {
            return Err(Error::msg("Jellyfin server url is not configured"));
        }
        let url = format!("{}/Users/AuthenticateByName", self.base_url);
        let body = AuthenticateByName {
            username: self.username.clone(),
            pw: self.password.clone(),
        };
        let mut response = surf::post(&url)
            .header(
                "X-Emby-Authorization",
                format!(
                    "MediaBrowser Client=\"{}\", Device=\"{}\", DeviceId=\"{}\", Version=\"{}\"",
                    CLIENT_NAME, CLIENT_NAME, CLIENT_NAME, CLIENT_VERSION
                ),
            )
            .body_json(&body)
            .map_err(|err| Error::msg(err.to_string()))?
            .await
            .map_err(|err| Error::msg(err.to_string()))?;
        if !response.status().is_success() {
            return Err(Error::msg(format!(
                "Jellyfin authentication failed with status {}",
                response.status()
            )));
        }
        let auth: AuthenticationResult = response
            .body_json()
            .await
            .map_err(|err| Error::msg(err.to_string()))?;
        self.user_id = auth.user.id;
        self.token = Some(auth.access_token);
        Ok(())
    }

    fn token(&self) -> String {
        self.token.clone().unwrap_or_default()
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, url: Url) -> Result<T, Error> {
        let mut response = surf::get(url.as_str())
            .header("X-Emby-Token", self.token())
            .await
            .map_err(|err| Error::msg(err.to_string()))?;
        if !response.status().is_success() {
            return Err(Error::msg(format!(
                "Jellyfin request failed with status {}",
                response.status()
            )));
        }
        response
            .body_json()
            .await
            .map_err(|err| Error::msg(err.to_string()))
    }

    fn url(&self, path: &str, params: &[(&str, &str)]) -> Result<Url, Error> {
        let mut url = Url::parse(&format!("{}{}", self.base_url, path))?;
        {
            let mut query = url.query_pairs_mut();
            for (key, value) in params {
                query.append_pair(key, value);
            }
        }
        Ok(url)
    }

    async fn items(&self, params: &[(&str, &str)]) -> Result<ItemsResult, Error> {
        let url = self.url(&format!("/Users/{}/Items", self.user_id), params)?;
        self.get_json(url).await
    }

    async fn item(&self, id: &str) -> Result<BaseItem, Error> {
        let url = self.url(&format!("/Users/{}/Items/{}", self.user_id, id), &[])?;
        self.get_json(url).await
    }

    /// Universal audio endpoint: streams the original container when the
    /// client supports it and transparently falls back to transcoding.
    pub fn stream_url(&self, item_id: &str) -> String {
        match self.url(
            &format!("/Audio/{}/universal", item_id),
            &[
                ("userId", &self.user_id),
                ("api_key", &self.token()),
                ("container", "mp3,aac,m4a,flac,ogg,opus,wav"),
                ("transcodingContainer", "mp3"),
                ("audioCodec", "mp3"),
            ],
        ) {
            Ok(url) => url.to_string(),
            Err(_) => "".to_string(),
        }
    }

    pub fn cover_url(&self, item_id: &str) -> String {
        match self.url(
            &format!("/Items/{}/Images/Primary", item_id),
            &[("api_key", &self.token())],
        ) {
            Ok(url) => url.to_string(),
            Err(_) => "".to_string(),
        }
    }

    fn map_track(&self, item: &BaseItem) -> Track {
        let artist = item
            .artist_items
            .first()
            .or_else(|| item.album_artists.first())
            .map(|artist| artist.name.clone())
            .unwrap_or_else(|| "None".to_string());
        let artist_id = item
            .artist_items
            .first()
            .or_else(|| item.album_artists.first())
            .map(|artist| artist.id.clone())
            .unwrap_or_else(|| format!("{:x}", md5::compute(&artist)));
        let album_title = item.album.clone().unwrap_or_else(|| "None".to_string());
        let album_id = item
            .album_id
            .clone()
            .unwrap_or_else(|| format!("{:x}", md5::compute(&album_title)));
        let cover_item_id = item.album_id.clone().unwrap_or_else(|| item.id.clone());
        Track {
            id: item.id.clone(),
            title: item.name.clone(),
            duration: item
                .run_time_ticks
                .map(|ticks| (ticks as f64 / TICKS_PER_SECOND) as f32),
            disc_number: item.parent_index_number.unwrap_or(1),
            track_number: item.index_number,
            uri: self.stream_url(&item.id),
            artists: vec![Artist {
                id: artist_id,
                name: artist.clone(),
                ..Default::default()
            }],
            album: Some(Album {
                id: album_id,
                title: album_title,
                artist: artist.clone(),
                year: item.production_year,
                cover: Some(self.cover_url(&cover_item_id)),
                ..Default::default()
            }),
            artist,
        }
    }

    fn map_album(&self, item: &BaseItem, tracks: Vec<Track>) -> Album {
        Album {
            id: item.id.clone(),
            title: item.name.clone(),
            artist: item
                .album_artists
                .first()
                .or_else(|| item.artist_items.first())
                .map(|artist| artist.name.clone())
                .unwrap_or_else(|| "None".to_string()),
            artist_id: item
                .album_artists
                .first()
                .or_else(|| item.artist_items.first())
                .map(|artist| artist.id.clone()),
            year: item.production_year,
            cover: Some(self.cover_url(&item.id)),
            tracks,
        }
    }

    fn map_artist(&self, item: &BaseItem, albums: Vec<Album>) -> Artist {
        Artist {
            id: item.id.clone(),
            name: item.name.clone(),
            picture: Some(self.cover_url(&item.id)),
            albums,
            songs: vec![],
        }
    }
}

impl Addon for Jellyfin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn author(&self) -> &str {
        &self.author
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[async_trait]
impl Browsable for Jellyfin {
    async fn albums(
        &mut self,
        filter: Option<String>,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<Album>, Error> {
        let offset = offset.max(0).to_string();
        let limit = normalize_limit(limit).to_string();
        let search_term = filter.unwrap_or_default();
        let mut params = vec![
            ("IncludeItemTypes", "MusicAlbum"),
            ("Recursive", "true"),
            ("SortBy", "SortName"),
            ("startIndex", offset.as_str()),
            ("limit", limit.as_str()),
        ];
        if !search_term.trim().is_empty() {
            params.push(("searchTerm", search_term.as_str()));
        }
        let result = self.items(&params).await?;
        Ok(result
            .items
            .iter()
            .map(|item| self.map_album(item, vec![]))
            .collect())
    }

    async fn artists(
        &mut self,
        filter: Option<String>,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<Artist>, Error> {
        let offset = offset.max(0).to_string();
        let limit = normalize_limit(limit).to_string();
        let search_term = filter.unwrap_or_default();
        let mut params = vec![
            ("userId", self.user_id.clone()),
            ("startIndex", offset),
            ("limit", limit),
        ];
        if !search_term.trim().is_empty() {
            params.push(("searchTerm", search_term));
        }
        let params: Vec<(&str, &str)> = params
            .iter()
            .map(|(key, value)| (*key, value.as_str()))
            .collect();
        let url = self.url("/Artists", &params)?;
        let result: ItemsResult = self.get_json(url).await?;
        Ok(result
            .items
            .iter()
            .map(|item| self.map_artist(item, vec![]))
            .collect())
    }

    async fn tracks(
        &mut self,
        filter: Option<String>,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<Track>, Error> {
        let offset = offset.max(0).to_string();
        let limit = normalize_limit(limit).to_string();
        let search_term = filter.unwrap_or_default();
        let mut params = vec![
            ("IncludeItemTypes", "Audio"),
            ("Recursive", "true"),
            ("SortBy", "SortName"),
            ("startIndex", offset.as_str()),
            ("limit", limit.as_str()),
        ];
        if !search_term.trim().is_empty() {
            params.push(("searchTerm", search_term.as_str()));
        }
        let result = self.items(&params).await?;
        Ok(result
            .items
            .iter()
            .map(|item| self.map_track(item))
            .collect())
    }

    async fn playlists(&mut self, offset: i32, limit: i32) -> Result<Vec<Playlist>, Error> {
        let offset = offset.max(0).to_string();
        let limit = normalize_limit(limit).to_string();
        let result = self
            .items(&[
                ("IncludeItemTypes", "Playlist"),
                ("Recursive", "true"),
                ("SortBy", "SortName"),
                ("startIndex", offset.as_str()),
                ("limit", limit.as_str()),
            ])
            .await?;
        Ok(result
            .items
            .iter()
            .map(|item| Playlist {
                id: item.id.clone(),
                name: item.name.clone(),
                description: None,
                tracks: vec![],
            })
            .collect())
    }

    async fn album(&mut self, id: &str) -> Result<Album, Error> {
        let item = self.item(id).await?;
        let songs = self
            .items(&[
                ("parentId", id),
                ("IncludeItemTypes", "Audio"),
                ("SortBy", "ParentIndexNumber,IndexNumber,SortName"),
            ])
            .await?;
        let tracks = songs
            .items
            .iter()
            .map(|song| self.map_track(song))
            .collect();
        Ok(self.map_album(&item, tracks))
    }

    async fn artist(&mut self, id: &str) -> Result<Artist, Error> {
        let item = self.item(id).await?;
        let albums = self
            .items(&[
                ("AlbumArtistIds", id),
                ("IncludeItemTypes", "MusicAlbum"),
                ("Recursive", "true"),
                ("SortBy", "SortName"),
            ])
            .await?;
        let albums = albums
            .items
            .iter()
            .map(|album| self.map_album(album, vec![]))
            .collect();
        Ok(self.map_artist(&item, albums))
    }

    async fn track(&mut self, id: &str) -> Result<Track, Error> {
        let item = self.item(id).await?;
        Ok(self.map_track(&item))
    }

    async fn playlist(&mut self, id: &str) -> Result<Playlist, Error> {
        let item = self.item(id).await?;
        let url = self.url(
            &format!("/Playlists/{}/Items", id),
            &[("userId", &self.user_id)],
        )?;
        let songs: ItemsResult = self.get_json(url).await?;
        Ok(Playlist {
            id: item.id.clone(),
            name: item.name.clone(),
            description: None,
            tracks: songs
                .items
                .iter()
                .map(|song| self.map_track(song))
                .collect(),
        })
    }

    fn device_ip(&self) -> String {
        self.host.clone()
    }
}

impl From<Device> for Jellyfin {
    fn from(device: Device) -> Self {
        let base_url = device
            .base_url
            .unwrap_or_else(|| format!("http://{}:{}", device.host, device.port));
        let (username, password) = match read_settings()
            .ok()
            .and_then(|config| config.try_deserialize::<Settings>().ok())
        {
            Some(settings) => (
                settings.jellyfin_username.unwrap_or_default(),
                settings.jellyfin_password.unwrap_or_default(),
            ),
            None => ("".to_string(), "".to_string()),
        };
        Jellyfin::with_credentials(&base_url, &username, &password)
    }
}

fn normalize_limit(limit: i32) -> i32 {
    if limit <= 0 {
        500
    } else {
        limit
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct AuthenticateByName {
    username: String,
    pw: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AuthenticationResult {
    pub access_token: String,
    pub user: AuthenticatedUser,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AuthenticatedUser {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ItemsResult {
    #[serde(default)]
    pub items: Vec<BaseItem>,
    #[serde(default)]
    pub total_record_count: i64,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BaseItem {
    pub id: String,
    pub name: String,
    #[serde(rename = "Type")]
    pub item_type: Option<String>,
    pub album: Option<String>,
    pub album_id: Option<String>,
    pub album_artist: Option<String>,
    #[serde(default)]
    pub album_artists: Vec<NameIdPair>,
    #[serde(default)]
    pub artist_items: Vec<NameIdPair>,
    pub run_time_ticks: Option<i64>,
    pub index_number: Option<u32>,
    pub parent_index_number: Option<u32>,
    pub production_year: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NameIdPair {
    pub id: String,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client() -> Jellyfin {
        let mut client =
            Jellyfin::with_credentials("https://jellyfin.example.com/", "demo", "demo");
        client.token = Some("test-token".to_string());
        client.user_id = "user-1".to_string();
        client
    }

    #[test]
    fn parses_authentication_result() {
        let json = r#"{
            "User": { "Id": "user-1", "Name": "demo" },
            "SessionInfo": { "Id": "session-1" },
            "AccessToken": "abcd1234",
            "ServerId": "server-1"
        }"#;
        let auth: AuthenticationResult = serde_json::from_str(json).unwrap();
        assert_eq!(auth.access_token, "abcd1234");
        assert_eq!(auth.user.id, "user-1");
    }

    #[test]
    fn maps_audio_item_to_track_with_universal_stream_url() {
        let json = r#"{
            "Items": [
                {
                    "Id": "song-1",
                    "Name": "One More Time",
                    "Type": "Audio",
                    "Album": "Discovery",
                    "AlbumId": "album-1",
                    "AlbumArtist": "Daft Punk",
                    "AlbumArtists": [{ "Id": "artist-1", "Name": "Daft Punk" }],
                    "ArtistItems": [{ "Id": "artist-1", "Name": "Daft Punk" }],
                    "RunTimeTicks": 3200000000,
                    "IndexNumber": 1,
                    "ParentIndexNumber": 1,
                    "ProductionYear": 2001
                }
            ],
            "TotalRecordCount": 1,
            "StartIndex": 0
        }"#;
        let result: ItemsResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.total_record_count, 1);
        let client = client();
        let track = client.map_track(&result.items[0]);
        assert_eq!(track.id, "song-1");
        assert_eq!(track.title, "One More Time");
        // 3_200_000_000 ticks / 10_000_000 = 320 seconds
        assert_eq!(track.duration, Some(320.0));
        assert_eq!(track.track_number, Some(1));
        assert_eq!(track.artist, "Daft Punk");
        assert_eq!(track.artists[0].id, "artist-1");
        assert!(track
            .uri
            .starts_with("https://jellyfin.example.com/Audio/song-1/universal?"));
        assert!(track.uri.contains("userId=user-1"));
        assert!(track.uri.contains("api_key=test-token"));
        assert!(track.uri.contains("transcodingContainer=mp3"));
        let album = track.album.as_ref().unwrap();
        assert_eq!(album.id, "album-1");
        assert_eq!(album.title, "Discovery");
        assert_eq!(
            album.cover,
            Some(
                "https://jellyfin.example.com/Items/album-1/Images/Primary?api_key=test-token"
                    .to_string()
            )
        );
    }

    #[test]
    fn maps_music_album_item() {
        let json = r#"{
            "Items": [
                {
                    "Id": "album-1",
                    "Name": "Discovery",
                    "Type": "MusicAlbum",
                    "AlbumArtists": [{ "Id": "artist-1", "Name": "Daft Punk" }],
                    "RunTimeTicks": 36000000000,
                    "ProductionYear": 2001
                }
            ],
            "TotalRecordCount": 1
        }"#;
        let result: ItemsResult = serde_json::from_str(json).unwrap();
        let client = client();
        let album = client.map_album(&result.items[0], vec![]);
        assert_eq!(album.id, "album-1");
        assert_eq!(album.title, "Discovery");
        assert_eq!(album.artist, "Daft Punk");
        assert_eq!(album.artist_id, Some("artist-1".to_string()));
        assert_eq!(album.year, Some(2001));
        assert_eq!(
            album.cover,
            Some(
                "https://jellyfin.example.com/Items/album-1/Images/Primary?api_key=test-token"
                    .to_string()
            )
        );
    }

    #[test]
    fn maps_artist_items() {
        let json = r#"{
            "Items": [
                { "Id": "artist-1", "Name": "Daft Punk", "Type": "MusicArtist" },
                { "Id": "artist-2", "Name": "Bonobo", "Type": "MusicArtist" }
            ],
            "TotalRecordCount": 2
        }"#;
        let result: ItemsResult = serde_json::from_str(json).unwrap();
        let client = client();
        let artists: Vec<Artist> = result
            .items
            .iter()
            .map(|item| client.map_artist(item, vec![]))
            .collect();
        assert_eq!(artists.len(), 2);
        assert_eq!(artists[0].id, "artist-1");
        assert_eq!(artists[1].name, "Bonobo");
    }
}
