//! Plex Media Server.
//!
//! Plain HTTP with `Accept: application/json` and an `X-Plex-Token`. Plex is
//! XML-first, so every response has to be asked for as JSON explicitly and
//! arrives wrapped in a `MediaContainer`.
//!
//! Plex has no username/password on the server itself — a token is what
//! authenticates, and it is obtained out of band (Plex Web → any item → Get
//! Info → View XML, or `plex.tv`). The add-server form's password field is
//! reused for it rather than inventing a third credential shape; the label a
//! client shows comes from the registry, not from here.

use crate::{
    http, Album, Artist, MusicProvider, Page, Playlist, ProviderCapabilities, ProviderConfig,
    ProviderError, ProviderFactory, SearchResults, Track,
};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

/// Plex's own type numbers, which every query is keyed on.
const TYPE_ARTIST: u32 = 8;
const TYPE_ALBUM: u32 = 9;
const TYPE_TRACK: u32 = 10;

pub struct Plex {
    base_url: String,
    host: String,
    token: String,
    /// The music library's section id. Found once at connect, because every
    /// listing is scoped to a section and Plex has no "all music" endpoint.
    section: String,
}

impl Plex {
    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T, ProviderError> {
        let mut url =
            url::Url::parse(&format!("{}{path}", self.base_url)).map_err(ProviderError::other)?;
        {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in query {
                pairs.append_pair(key, value);
            }
        }
        let response = http::client()
            .get(url.as_str())
            .header("X-Plex-Token", &self.token)
            // Plex speaks XML unless asked otherwise.
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(ProviderError::transport)?;

        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                401 | 403 => ProviderError::Auth("the server refused the token".to_string()),
                404 => ProviderError::NotFound(path.to_string()),
                _ => ProviderError::Transport(format!("the server answered {status}")),
            });
        }
        response.json().await.map_err(ProviderError::transport)
    }

    /// Plex serves art and audio from its own paths, and both need the token —
    /// whatever plays or renders this is an HTTP client that knows nothing
    /// about Plex.
    fn signed(&self, path: &str) -> String {
        if path.is_empty() {
            return String::new();
        }
        let separator = if path.contains('?') { '&' } else { '?' };
        format!(
            "{}{path}{separator}X-Plex-Token={}",
            self.base_url,
            urlencoding::encode(&self.token)
        )
    }

    fn cover(&self, thumb: &Option<String>) -> Option<String> {
        thumb
            .as_deref()
            .filter(|thumb| !thumb.is_empty())
            .map(|thumb| self.signed(thumb))
    }

    /// `/library/sections/<id>/all`, which is how every listing is spelled.
    async fn section_items(
        &self,
        kind: u32,
        query: Option<&str>,
        page: Page,
    ) -> Result<Vec<Metadata>, ProviderError> {
        let kind = kind.to_string();
        let start = page.offset.max(0).to_string();
        let size = page.limit.max(0).to_string();
        let mut params: Vec<(&str, &str)> = vec![
            ("type", kind.as_str()),
            ("X-Plex-Container-Start", start.as_str()),
        ];
        if page.limit > 0 {
            params.push(("X-Plex-Container-Size", size.as_str()));
        }
        if let Some(query) = query.map(str::trim).filter(|q| !q.is_empty()) {
            params.push(("title", query));
        }
        let container: Container = self
            .get(&format!("/library/sections/{}/all", self.section), &params)
            .await?;
        Ok(container.media_container.metadata)
    }

    fn map_album(&self, item: &Metadata) -> Album {
        Album {
            id: item.rating_key.clone(),
            title: item.title.clone(),
            artist: item.parent_title.clone().unwrap_or_default(),
            artist_id: item.parent_rating_key.clone(),
            year: item.year,
            cover: self.cover(&item.thumb),
            tracks: vec![],
        }
    }

    fn map_artist(&self, item: &Metadata) -> Artist {
        Artist {
            id: item.rating_key.clone(),
            name: item.title.clone(),
            picture: self.cover(&item.thumb),
            albums: vec![],
            songs: vec![],
        }
    }

    fn map_track(&self, item: &Metadata) -> Track {
        // The playable path is buried two levels down, in the first part of
        // the first media entry.
        let file = item
            .media
            .first()
            .and_then(|media| media.part.first())
            .map(|part| part.key.clone())
            .unwrap_or_default();
        let artist = item.grandparent_title.clone().unwrap_or_default();
        Track {
            id: item.rating_key.clone(),
            title: item.title.clone(),
            // Plex reports milliseconds.
            duration: item.duration.map(|ms| ms as f32 / 1000.0),
            disc_number: item.parent_index.unwrap_or(0),
            track_number: item.index,
            uri: self.signed(&file),
            artists: match (&item.grandparent_rating_key, artist.is_empty()) {
                (Some(id), false) => vec![Artist {
                    id: id.clone(),
                    name: artist.clone(),
                    ..Default::default()
                }],
                _ => vec![],
            },
            album: item.parent_rating_key.as_ref().map(|id| Album {
                id: id.clone(),
                title: item.parent_title.clone().unwrap_or_default(),
                artist: artist.clone(),
                year: item.parent_year.or(item.year),
                cover: self
                    .cover(&item.parent_thumb)
                    .or_else(|| self.cover(&item.thumb)),
                ..Default::default()
            }),
            artist,
        }
    }
}

#[async_trait]
impl MusicProvider for Plex {
    fn kind(&self) -> &'static str {
        "plex"
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
            // Plex has star ratings, not likes.
            liked: false,
            native_search: true,
        }
    }

    async fn albums(&self, filter: Option<&str>, page: Page) -> Result<Vec<Album>, ProviderError> {
        let items = self.section_items(TYPE_ALBUM, filter, page).await?;
        Ok(items.iter().map(|item| self.map_album(item)).collect())
    }

    async fn artists(
        &self,
        filter: Option<&str>,
        page: Page,
    ) -> Result<Vec<Artist>, ProviderError> {
        let items = self.section_items(TYPE_ARTIST, filter, page).await?;
        Ok(items.iter().map(|item| self.map_artist(item)).collect())
    }

    async fn tracks(&self, filter: Option<&str>, page: Page) -> Result<Vec<Track>, ProviderError> {
        let items = self.section_items(TYPE_TRACK, filter, page).await?;
        Ok(items.iter().map(|item| self.map_track(item)).collect())
    }

    async fn album(&self, id: &str) -> Result<Album, ProviderError> {
        let container: Container = self.get(&format!("/library/metadata/{id}"), &[]).await?;
        let item = container
            .media_container
            .metadata
            .first()
            .ok_or_else(|| ProviderError::NotFound(format!("album {id}")))?;
        let mut album = self.map_album(item);

        let children: Container = self
            .get(&format!("/library/metadata/{id}/children"), &[])
            .await?;
        album.tracks = children
            .media_container
            .metadata
            .iter()
            .map(|track| self.map_track(track))
            .collect();
        Ok(album)
    }

    async fn artist(&self, id: &str) -> Result<Artist, ProviderError> {
        let container: Container = self.get(&format!("/library/metadata/{id}"), &[]).await?;
        let item = container
            .media_container
            .metadata
            .first()
            .ok_or_else(|| ProviderError::NotFound(format!("artist {id}")))?;
        let mut artist = self.map_artist(item);

        let children: Container = self
            .get(&format!("/library/metadata/{id}/children"), &[])
            .await?;
        artist.albums = children
            .media_container
            .metadata
            .iter()
            .map(|album| self.map_album(album))
            .collect();
        Ok(artist)
    }

    async fn track(&self, id: &str) -> Result<Track, ProviderError> {
        let container: Container = self.get(&format!("/library/metadata/{id}"), &[]).await?;
        container
            .media_container
            .metadata
            .first()
            .map(|item| self.map_track(item))
            .ok_or_else(|| ProviderError::NotFound(format!("track {id}")))
    }

    async fn playlists(&self, page: Page) -> Result<Vec<Playlist>, ProviderError> {
        let container: Container = self.get("/playlists", &[("playlistType", "audio")]).await?;
        Ok(page.slice(
            container
                .media_container
                .metadata
                .iter()
                .map(|item| Playlist {
                    id: item.rating_key.clone(),
                    name: item.title.clone(),
                    description: item.summary.clone().filter(|s| !s.is_empty()),
                    tracks: vec![],
                    // Reported on the listing; the items are a second call.
                    track_count: item.leaf_count,
                })
                .collect(),
        ))
    }

    async fn playlist(&self, id: &str) -> Result<Playlist, ProviderError> {
        let container: Container = self.get(&format!("/playlists/{id}"), &[]).await?;
        let item = container
            .media_container
            .metadata
            .first()
            .ok_or_else(|| ProviderError::NotFound(format!("playlist {id}")))?;
        let name = item.title.clone();
        let description = item.summary.clone().filter(|s| !s.is_empty());

        let items: Container = self.get(&format!("/playlists/{id}/items"), &[]).await?;
        Ok(Playlist {
            id: id.to_string(),
            name,
            description,
            // The items are right here, so counting them is exact.
            track_count: None,
            tracks: items
                .media_container
                .metadata
                .iter()
                .map(|track| self.map_track(track))
                .collect(),
        })
    }

    /// One call across every type, rather than the trait's three.
    async fn search(&self, keyword: &str, page: Page) -> Result<SearchResults, ProviderError> {
        let limit = page.limit.max(1).to_string();
        let container: Container = self
            .get(
                "/hubs/search",
                &[("query", keyword), ("limit", limit.as_str())],
            )
            .await?;

        let mut results = SearchResults::default();
        // Search comes back as hubs — one per type — rather than a flat list.
        for hub in &container.media_container.hub {
            for item in &hub.metadata {
                match item.item_type.as_deref() {
                    Some("artist") => results.artists.push(self.map_artist(item)),
                    Some("album") => results.albums.push(self.map_album(item)),
                    Some("track") => results.tracks.push(self.map_track(item)),
                    _ => {}
                }
            }
        }
        Ok(results)
    }

    async fn ping(&self) -> Result<(), ProviderError> {
        let _: Container = self.get("/identity", &[]).await?;
        Ok(())
    }
}

// ── Plex's wire shapes ──────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
struct Container {
    #[serde(rename = "MediaContainer", default)]
    media_container: MediaContainer,
}

#[derive(Deserialize, Default)]
struct MediaContainer {
    #[serde(rename = "Metadata", default)]
    metadata: Vec<Metadata>,
    #[serde(rename = "Directory", default)]
    directory: Vec<Directory>,
    #[serde(rename = "Hub", default)]
    hub: Vec<Hub>,
}

#[derive(Deserialize, Default)]
struct Hub {
    #[serde(rename = "Metadata", default)]
    metadata: Vec<Metadata>,
}

/// A library section. Only the music one is of interest, and only its key.
#[derive(Deserialize, Default)]
struct Directory {
    key: String,
    #[serde(rename = "type", default)]
    kind: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct Metadata {
    #[serde(default)]
    rating_key: String,
    #[serde(default)]
    title: String,
    #[serde(rename = "type")]
    item_type: Option<String>,
    summary: Option<String>,
    thumb: Option<String>,
    parent_thumb: Option<String>,
    parent_title: Option<String>,
    parent_rating_key: Option<String>,
    parent_year: Option<u32>,
    /// The disc, for a track.
    parent_index: Option<u32>,
    grandparent_title: Option<String>,
    grandparent_rating_key: Option<String>,
    /// The track number.
    index: Option<u32>,
    year: Option<u32>,
    /// Milliseconds.
    duration: Option<u64>,
    /// How many tracks a playlist holds.
    leaf_count: Option<u32>,
    #[serde(rename = "Media", default)]
    media: Vec<Media>,
}

#[derive(Deserialize, Default)]
struct Media {
    #[serde(rename = "Part", default)]
    part: Vec<Part>,
}

#[derive(Deserialize, Default)]
struct Part {
    #[serde(default)]
    key: String,
}

pub struct PlexFactory;

#[async_trait]
impl ProviderFactory for PlexFactory {
    fn kind(&self) -> &'static str {
        "plex"
    }

    fn display_name(&self) -> &'static str {
        "Plex (token in the password field)"
    }

    fn default_port(&self) -> u16 {
        32400
    }

    async fn connect(
        &self,
        config: &ProviderConfig,
    ) -> Result<Arc<dyn MusicProvider>, ProviderError> {
        let token = config
            .password
            .clone()
            .filter(|token| !token.is_empty())
            .ok_or_else(|| {
                ProviderError::Auth("Plex needs an X-Plex-Token, in the password field".into())
            })?;

        // Every listing is scoped to a section, and Plex has no "all music"
        // endpoint — so the music one is found once, here.
        let mut plex = Plex {
            base_url: config.url.clone(),
            host: config.host(),
            token,
            section: String::new(),
        };
        let sections: Container = plex.get("/library/sections", &[]).await?;
        plex.section = sections
            .media_container
            .directory
            .iter()
            .find(|section| section.kind == "artist")
            .map(|section| section.key.clone())
            .ok_or_else(|| ProviderError::Other("that Plex server has no music library".into()))?;

        Ok(Arc::new(plex))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn plex() -> Plex {
        Plex {
            base_url: "http://plex.lan:32400".into(),
            host: "plex.lan".into(),
            token: "tok en".into(),
            section: "3".into(),
        }
    }

    /// Whatever plays or renders this knows nothing about Plex, so the token
    /// has to travel in the url.
    #[test]
    fn media_urls_carry_the_token() {
        assert_eq!(
            plex().signed("/library/parts/1/file.mp3"),
            "http://plex.lan:32400/library/parts/1/file.mp3?X-Plex-Token=tok%20en"
        );
        // A path that already has a query gets the token appended, not a
        // second `?`.
        assert!(plex()
            .signed("/photo/:/transcode?url=x")
            .contains("?url=x&X-Plex-Token="));
    }

    #[test]
    fn an_absent_path_yields_no_url() {
        assert!(plex().signed("").is_empty());
        assert!(plex().cover(&None).is_none());
        assert!(plex().cover(&Some(String::new())).is_none());
    }

    #[test]
    fn a_track_maps_across() {
        let item: Metadata = serde_json::from_value(json!({
            "ratingKey": "1234",
            "title": "God Is Dead?",
            "type": "track",
            "index": 2,
            "parentIndex": 1,
            "duration": 532317,
            "parentTitle": "13",
            "parentRatingKey": "99",
            "parentThumb": "/library/metadata/99/thumb/1",
            "grandparentTitle": "Black Sabbath",
            "grandparentRatingKey": "7",
            "year": 2013,
            "Media": [{ "Part": [{ "key": "/library/parts/5/file.mp3" }] }]
        }))
        .unwrap();

        let track = plex().map_track(&item);
        assert_eq!(track.id, "1234");
        assert_eq!(track.track_number, Some(2));
        assert_eq!(track.disc_number, 1);
        // Plex reports milliseconds; the app wants seconds.
        assert_eq!(track.duration, Some(532.317));
        assert_eq!(track.artist, "Black Sabbath");
        assert_eq!(track.album.as_ref().unwrap().id, "99");
        assert!(track.uri.contains("/library/parts/5/file.mp3"));
        assert!(track.uri.contains("X-Plex-Token="));
    }

    /// Plex omits what it has nothing for, and a track with no media part is
    /// not playable but is still a row.
    #[test]
    fn a_sparse_track_still_maps() {
        let item: Metadata =
            serde_json::from_value(json!({ "ratingKey": "1", "title": "Untitled" })).unwrap();
        let track = plex().map_track(&item);
        assert_eq!(track.title, "Untitled");
        assert!(track.uri.is_empty());
        assert!(track.album.is_none());
        assert!(track.artists.is_empty());
    }

    #[test]
    fn an_album_maps_across() {
        let item: Metadata = serde_json::from_value(json!({
            "ratingKey": "99",
            "title": "13",
            "type": "album",
            "parentTitle": "Black Sabbath",
            "parentRatingKey": "7",
            "year": 2013,
            "thumb": "/library/metadata/99/thumb/1"
        }))
        .unwrap();

        let album = plex().map_album(&item);
        assert_eq!(album.id, "99");
        assert_eq!(album.artist, "Black Sabbath");
        assert_eq!(album.artist_id.as_deref(), Some("7"));
        assert_eq!(album.year, Some(2013));
        assert!(album.cover.unwrap().contains("X-Plex-Token="));
    }

    /// Search comes back as hubs, one per type, not a flat list.
    #[test]
    fn the_search_container_splits_by_type() {
        let container: Container = serde_json::from_value(json!({
            "MediaContainer": {
                "Hub": [
                    { "Metadata": [{ "ratingKey": "7", "title": "Black Sabbath", "type": "artist" }] },
                    { "Metadata": [{ "ratingKey": "99", "title": "13", "type": "album" }] }
                ]
            }
        }))
        .unwrap();
        assert_eq!(container.media_container.hub.len(), 2);
        assert_eq!(
            container.media_container.hub[1].metadata[0]
                .item_type
                .as_deref(),
            Some("album")
        );
    }

    /// An empty container is the normal shape of "nothing found".
    #[test]
    fn an_empty_container_parses() {
        let container: Container = serde_json::from_value(json!({ "MediaContainer": {} })).unwrap();
        assert!(container.media_container.metadata.is_empty());
        assert!(container.media_container.directory.is_empty());
    }
}
