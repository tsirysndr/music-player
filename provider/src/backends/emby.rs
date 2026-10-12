//! Native Emby metadata client. Playback is resolved separately; metadata never
//! contains authenticated stream URLs. The host must bind the authenticated
//! remote identity to the saved account before publishing this provider.

use crate::{
    emby_playback::MediaSource, Album, Artist, MediaEntry, MusicProvider, Page,
    ProviderCapabilities, ProviderConfig, ProviderError, ProviderFactory, SearchResults, Track,
};
use music_player_settings::EmbyRuntimeSettings;
use music_player_types::source::{RemoteIdentity, ResourceKind, SourceRef};
use reqwest::{
    header::{HeaderMap, HeaderValue, LOCATION},
    Method, StatusCode,
};
use serde::{de::DeserializeOwned, Deserialize};
use std::{collections::HashSet, sync::Arc, time::Duration};
use url::Url;

const PAGE_SIZE: usize = 500;
const MAX_JSON_BYTES: usize = 8 * 1024 * 1024;

/// Native Emby authentication for the saved-server registry. The host still
/// verifies and binds the returned identity before publishing the provider.
pub struct EmbyFactory {
    device_id: String,
    follow_redirects: bool,
}

impl EmbyFactory {
    pub fn new(device_id: String, follow_redirects: bool) -> Self {
        Self {
            device_id,
            follow_redirects,
        }
    }
}

impl Default for EmbyFactory {
    fn default() -> Self {
        let settings = EmbyRuntimeSettings::read();
        Self::new(settings.device_id, settings.follow_redirects)
    }
}

#[async_trait::async_trait]
impl ProviderFactory for EmbyFactory {
    fn kind(&self) -> &'static str {
        "emby"
    }

    fn display_name(&self) -> &'static str {
        "Emby"
    }

    fn default_port(&self) -> u16 {
        8096
    }

    async fn connect(
        &self,
        config: &ProviderConfig,
    ) -> Result<Arc<dyn MusicProvider>, ProviderError> {
        // Empty passwords are valid: pass the original account config through.
        Ok(Arc::new(
            Emby::authenticate(config, &self.device_id, self.follow_redirects).await?,
        ))
    }
}

pub struct Emby {
    client: reqwest::Client,
    base: Url,
    base_text: String,
    host: String,
    account_id: String,
    identity: RemoteIdentity,
    headers: HeaderMap,
    follow_redirects: bool,
    device_id: String,
}

impl Emby {
    pub async fn authenticate(
        config: &ProviderConfig,
        device_id: &str,
        follow_redirects: bool,
    ) -> Result<Self, ProviderError> {
        if config.kind != "emby" || config.id.is_empty() || device_id.is_empty() {
            return Err(other("Emby requires an account and device identity"));
        }
        // A caller-owned random device ID is also used for per-session cleanup.
        // Never interpolate a remote string into the authorization grammar.
        if !device_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err(other("invalid Emby device identity"));
        }
        let base = Url::parse(config.url.trim()).map_err(|_| other("invalid Emby server URL"))?;
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(20))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .map_err(network)?;
        let mut headers = HeaderMap::new();
        headers.insert("X-Emby-Authorization", HeaderValue::from_str(&format!(
            "MediaBrowser Client=\"music-player\", Device=\"music-player\", DeviceId=\"{device_id}\", Version=\"{}\"",
            env!("CARGO_PKG_VERSION")
        )).map_err(|_| other("invalid Emby client identity"))?);
        let mut value = Self {
            client,
            host: base.host_str().unwrap_or_default().to_owned(),
            base_text: config.url.clone(),
            base,
            account_id: config.id.clone(),
            identity: RemoteIdentity {
                server_id: String::new(),
                user_id: String::new(),
            },
            headers,
            follow_redirects,
            device_id: device_id.to_owned(),
        };
        let body = serde_json::json!({
            "Username": config.username.as_deref().unwrap_or_default(),
            "Pw": config.password.as_deref().unwrap_or_default(),
        });
        let auth: Authentication = value
            .json(
                Method::POST,
                &["Users", "AuthenticateByName"],
                &[],
                Some(&body),
            )
            .await?;
        value.identity.user_id = auth.user.id;
        let mut token = HeaderValue::from_str(&auth.access_token)
            .map_err(|_| other("invalid Emby session token"))?;
        token.set_sensitive(true);
        value.headers.insert("X-Emby-Token", token);
        value.identity.server_id = match auth.server_id.filter(|id| !id.is_empty()) {
            Some(id) => id,
            None => {
                let info: SystemInfo = value
                    .json(Method::GET, &["System", "Info"], &[], None)
                    .await?;
                info.id
            }
        };
        if value.identity.server_id.trim().is_empty()
            || value.identity.user_id.trim().is_empty()
            || auth.access_token.is_empty()
        {
            return Err(other("Emby did not confirm its server and user identity"));
        }
        Ok(value)
    }

    pub fn identity(&self) -> &RemoteIdentity {
        &self.identity
    }
    pub fn device_id(&self) -> &str {
        &self.device_id
    }
    pub fn follows_redirects(&self) -> bool {
        self.follow_redirects
    }
    /// Headers are transient. Callers must not serialize them into Track/queue.
    pub fn playback_headers(&self) -> HeaderMap {
        self.headers.clone()
    }

    pub fn reference(&self, kind: ResourceKind, item_id: &str) -> Result<SourceRef, ProviderError> {
        let source = SourceRef {
            resolver: "emby".into(),
            account_id: self.account_id.clone(),
            remote: self.identity.clone(),
            kind,
            item_id: item_id.to_owned(),
        };
        SourceRef::parse(&source.to_handle()).map_err(ProviderError::other)
    }

    pub fn check_reference(
        &self,
        value: &str,
        kind: ResourceKind,
    ) -> Result<SourceRef, ProviderError> {
        let source = SourceRef::parse(value).map_err(ProviderError::other)?;
        if source.resolver != "emby"
            || source.account_id != self.account_id
            || source.remote != self.identity
            || source.kind != kind
        {
            return Err(other(
                "source identity or resource kind does not match this Emby account",
            ));
        }
        Ok(source)
    }

    pub(crate) fn endpoint(
        &self,
        segments: &[&str],
        query: &[(&str, String)],
    ) -> Result<Url, ProviderError> {
        let mut url = self.base.clone();
        url.path_segments_mut()
            .map_err(|_| other("invalid Emby base path"))?
            .pop_if_empty()
            .extend(segments.iter().copied());
        url.query_pairs_mut()
            .extend_pairs(query.iter().map(|(key, value)| (*key, value)));
        Ok(url)
    }

    async fn response(
        &self,
        mut method: Method,
        mut url: Url,
        mut body: Option<serde_json::Value>,
    ) -> Result<reqwest::Response, ProviderError> {
        for hop in 0..=10 {
            let mut request = self
                .client
                .request(method.clone(), url.clone())
                .headers(self.headers.clone());
            if let Some(body) = &body {
                request = request.json(body);
            }
            let response = request.send().await.map_err(network)?;
            let status = response.status();
            if matches!(status.as_u16(), 301 | 302 | 303 | 307 | 308) {
                if !self.follow_redirects {
                    return Err(other("Emby HTTP redirects are disabled"));
                }
                if hop == 10 {
                    return Err(other("too many Emby HTTP redirects"));
                }
                let location = response
                    .headers()
                    .get(LOCATION)
                    .and_then(|v| v.to_str().ok())
                    .ok_or_else(|| other("Emby redirect has no valid Location"))?;
                url = url
                    .join(location)
                    .map_err(|_| other("invalid Emby redirect Location"))?;
                if (status == StatusCode::SEE_OTHER && method != Method::HEAD)
                    || (matches!(status.as_u16(), 301 | 302) && method == Method::POST)
                {
                    method = Method::GET;
                    body = None;
                }
                // Preserve the caller's auth headers, including across origins.
                continue;
            }
            return match status.as_u16() {
                200..=299 => Ok(response),
                401 | 403 => Err(ProviderError::Auth("Emby refused this session".into())),
                404 => Err(ProviderError::NotFound("Emby resource".into())),
                _ => Err(other(format!("Emby answered HTTP {}", status.as_u16()))),
            };
        }
        unreachable!()
    }

    pub(crate) async fn json<T: DeserializeOwned>(
        &self,
        method: Method,
        segments: &[&str],
        query: &[(&str, String)],
        body: Option<&serde_json::Value>,
    ) -> Result<T, ProviderError> {
        let mut response = self
            .response(method, self.endpoint(segments, query)?, body.cloned())
            .await?;
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(network)? {
            if chunk.len() > MAX_JSON_BYTES.saturating_sub(bytes.len()) {
                return Err(other("Emby metadata response is too large"));
            }
            bytes.extend_from_slice(&chunk);
        }
        // Serde errors can quote server-provided values; use a fixed diagnostic.
        serde_json::from_slice(&bytes).map_err(|_| other("invalid Emby metadata response"))
    }

    pub(crate) async fn delete_encoding(&self, play_session_id: &str) -> Result<(), ProviderError> {
        if play_session_id.is_empty() {
            return Err(other("missing playback session identity"));
        }
        self.response(
            Method::DELETE,
            self.endpoint(
                &["Videos", "ActiveEncodings"],
                &[
                    ("DeviceId", self.device_id.clone()),
                    ("PlaySessionId", play_session_id.to_owned()),
                ],
            )?,
            None,
        )
        .await?;
        Ok(())
    }

    pub(crate) async fn item(&self, source: &SourceRef) -> Result<Item, ProviderError> {
        self.check_reference(&source.to_handle(), source.kind)?;
        let item: Item = self
            .json(
                Method::GET,
                &["Users", &self.identity.user_id, "Items", &source.item_id],
                &[("Fields", "MediaSources,MediaStreams".into())],
                None,
            )
            .await?;
        if item.id != source.item_id {
            return Err(other("Emby returned a different item"));
        }
        Ok(item)
    }

    async fn items(
        &self,
        mut query: Vec<(&str, String)>,
        page: Page,
    ) -> Result<Vec<Item>, ProviderError> {
        let mut cursor = page.offset.max(0) as usize;
        let wanted = (page.limit > 0).then_some(page.limit as usize);
        let mut items = Vec::new();
        let mut seen = HashSet::new();
        query.push(("EnableTotalRecordCount", "true".into()));
        query.push(("SortBy", "SortName,Id".into()));
        loop {
            let limit = wanted
                .map(|n| n - items.len())
                .unwrap_or(PAGE_SIZE)
                .min(PAGE_SIZE);
            if limit == 0 {
                break;
            }
            let mut params = query.clone();
            params.extend([
                ("StartIndex", cursor.to_string()),
                ("Limit", limit.to_string()),
            ]);
            let result: Items = self
                .json(
                    Method::GET,
                    &["Users", &self.identity.user_id, "Items"],
                    &params,
                    None,
                )
                .await?;
            if result.start_index.is_some_and(|start| start != cursor) || result.items.len() > limit
            {
                return Err(other("Emby returned an inconsistent page"));
            }
            if result.items.is_empty() {
                if result
                    .total_record_count
                    .is_some_and(|total| cursor < total)
                {
                    return Err(other("Emby listing changed; retry the listing"));
                }
                break;
            }
            cursor = cursor
                .checked_add(result.items.len())
                .ok_or_else(|| other("Emby page offset overflow"))?;
            for item in result.items {
                if !seen.insert(item.id.clone()) {
                    return Err(other("Emby listing repeated an item; retry the listing"));
                }
                items.push(item);
            }
            if result
                .total_record_count
                .is_some_and(|total| cursor >= total)
            {
                break;
            }
        }
        Ok(items)
    }

    async fn views(&self, page: Page) -> Result<Vec<Item>, ProviderError> {
        // Emby's Views endpoint has no StartIndex/Limit in its API contract.
        // Fetch the whole root list once and apply the caller's page locally.
        let result: Items = self
            .json(
                Method::GET,
                &["Users", &self.identity.user_id, "Views"],
                &[],
                None,
            )
            .await?;
        if result.start_index.is_some_and(|start| start != 0)
            || result
                .total_record_count
                .is_some_and(|total| total != result.items.len())
        {
            return Err(other("Emby returned an incomplete root listing"));
        }
        let mut seen = HashSet::new();
        for item in &result.items {
            if !seen.insert(&item.id) {
                return Err(other("Emby root listing repeated an item"));
            }
        }
        Ok(page.slice(result.items))
    }

    fn map_entry(&self, item: &Item) -> Result<MediaEntry, ProviderError> {
        let is_container = item.is_container();
        let kind = if is_container {
            ResourceKind::Container
        } else {
            ResourceKind::Item
        };
        Ok(MediaEntry {
            id: self.reference(kind, &item.id)?.to_handle(),
            title: item.name.clone(),
            item_type: item.item_type.clone(),
            media_type: item.media_type.clone(),
            is_container,
            season_number: match item.item_type.as_deref() {
                Some("Season") => item.index_number,
                Some("Episode") => item.parent_index_number,
                _ => None,
            },
            episode_number: if item.item_type.as_deref() == Some("Episode") {
                item.index_number
            } else {
                None
            },
            track: if item.is_media_leaf() {
                Some(self.map_track(item)?)
            } else {
                None
            },
        })
    }

    pub(crate) fn map_track(&self, item: &Item) -> Result<Track, ProviderError> {
        if !item.is_media_leaf() {
            return Err(other("this Emby item is not an audio/video leaf"));
        }
        let handle = self.reference(ResourceKind::Item, &item.id)?.to_handle();
        let music = item.media_type.as_deref() == Some("Audio");
        let artists = item
            .artist_items
            .iter()
            .map(|artist| {
                Ok(Artist {
                    id: self
                        .reference(ResourceKind::Artist, &artist.id)?
                        .to_handle(),
                    name: artist.name.clone(),
                    ..Default::default()
                })
            })
            .collect::<Result<Vec<_>, ProviderError>>()?;
        let artist = artists
            .first()
            .map(|a| a.name.clone())
            .or_else(|| item.series_name.clone())
            .unwrap_or_default();
        let album = match (&item.album_id, &item.album) {
            (Some(id), Some(title)) if music => Some(Album {
                id: self.reference(ResourceKind::Album, id)?.to_handle(),
                title: title.clone(),
                artist: artist.clone(),
                ..Default::default()
            }),
            _ => None,
        };
        Ok(Track {
            id: handle.clone(),
            uri: handle,
            title: item.name.clone(),
            artist,
            artists,
            album,
            duration: item
                .run_time_ticks
                .map(|ticks| (ticks as f64 / 10_000_000.0) as f32),
            disc_number: if music {
                // Legacy music fields are u32. Never wrap a larger index;
                // episode/season numbers live losslessly on MediaEntry.
                u32::try_from(item.parent_index_number.unwrap_or(1)).unwrap_or(0)
            } else {
                0
            },
            track_number: if music {
                item.index_number
                    .and_then(|index| u32::try_from(index).ok())
            } else {
                None
            },
            liked: item.user_data.as_ref().map(|user| user.is_favorite),
            ..Default::default()
        })
    }
}

#[async_trait::async_trait]
impl MusicProvider for Emby {
    fn kind(&self) -> &'static str {
        "emby"
    }
    fn remote_identity(&self) -> Option<RemoteIdentity> {
        Some(self.identity.clone())
    }
    fn base_url(&self) -> &str {
        &self.base_text
    }
    fn host(&self) -> &str {
        &self.host
    }
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            native_search: true,
            media_browse: true,
            ..Default::default()
        }
    }
    async fn browse(
        &self,
        parent: Option<&str>,
        page: Page,
    ) -> Result<Vec<MediaEntry>, ProviderError> {
        let items = match parent {
            None => self.views(page).await?,
            Some(parent) => {
                // Validate before I/O: a handle from another account, an old
                // bare provider ID or a playable leaf is never a parent.
                let source = self.check_reference(parent, ResourceKind::Container)?;
                self.items(
                    vec![("ParentId", source.item_id), ("Recursive", "false".into())],
                    page,
                )
                .await?
            }
        };
        items.iter().map(|item| self.map_entry(item)).collect()
    }

    async fn container_tracks(
        &self,
        parent: &str,
        page: Page,
    ) -> Result<Vec<Track>, ProviderError> {
        let source = self.check_reference(parent, ResourceKind::Container)?;
        self.items(
            vec![
                ("ParentId", source.item_id),
                ("Recursive", "true".into()),
                ("MediaTypes", "Audio,Video".into()),
                ("IsFolder", "false".into()),
            ],
            page,
        )
        .await?
        .iter()
        // Keep the remote page cursor independent of this defensive filter.
        .filter(|item| item.is_media_leaf())
        .map(|item| self.map_track(item))
        .collect()
    }

    async fn search(&self, keyword: &str, page: Page) -> Result<SearchResults, ProviderError> {
        Ok(SearchResults {
            tracks: self.tracks(Some(keyword), page).await?,
            ..Default::default()
        })
    }
    async fn tracks(&self, filter: Option<&str>, page: Page) -> Result<Vec<Track>, ProviderError> {
        let mut query = vec![
            ("Recursive", "true".into()),
            ("MediaTypes", "Audio,Video".into()),
            ("IsFolder", "false".into()),
        ];
        if let Some(filter) = filter.filter(|filter| !filter.trim().is_empty()) {
            query.push(("SearchTerm", filter.into()));
        }
        self.items(query, page)
            .await?
            .iter()
            .map(|item| self.map_track(item))
            .collect()
    }
    async fn track(&self, id: &str) -> Result<Track, ProviderError> {
        let source = self.check_reference(id, ResourceKind::Item)?;
        self.map_track(&self.item(&source).await?)
    }
    async fn albums(&self, _: Option<&str>, _: Page) -> Result<Vec<Album>, ProviderError> {
        Err(ProviderError::Unsupported {
            kind: "emby",
            feature: "album browsing",
        })
    }
    async fn artists(&self, _: Option<&str>, _: Page) -> Result<Vec<Artist>, ProviderError> {
        Err(ProviderError::Unsupported {
            kind: "emby",
            feature: "artist browsing",
        })
    }
    async fn album(&self, _: &str) -> Result<Album, ProviderError> {
        Err(ProviderError::Unsupported {
            kind: "emby",
            feature: "album browsing",
        })
    }
    async fn artist(&self, _: &str) -> Result<Artist, ProviderError> {
        Err(ProviderError::Unsupported {
            kind: "emby",
            feature: "artist browsing",
        })
    }
}

fn network(error: reqwest::Error) -> ProviderError {
    ProviderError::transport(error.without_url())
}
fn other(error: impl Into<String>) -> ProviderError {
    ProviderError::Other(error.into())
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Authentication {
    access_token: String,
    user: User,
    server_id: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct User {
    id: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SystemInfo {
    id: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Items {
    items: Vec<Item>,
    total_record_count: Option<usize>,
    start_index: Option<usize>,
}

#[derive(Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Item {
    pub id: String,
    pub name: String,
    #[serde(rename = "Type")]
    pub item_type: Option<String>,
    pub media_type: Option<String>,
    pub is_folder: Option<bool>,
    pub run_time_ticks: Option<u64>,
    #[serde(default)]
    pub media_sources: Vec<MediaSource>,
    pub series_name: Option<String>,
    pub index_number: Option<u64>,
    pub parent_index_number: Option<u64>,
    pub album: Option<String>,
    pub album_id: Option<String>,
    #[serde(default)]
    pub artist_items: Vec<NameId>,
    pub user_data: Option<UserData>,
}
impl Item {
    fn is_container(&self) -> bool {
        self.is_folder == Some(true)
            || matches!(
                self.item_type.as_deref(),
                Some(
                    "CollectionFolder"
                        | "UserView"
                        | "Folder"
                        | "Series"
                        | "Season"
                        | "BoxSet"
                        | "MusicAlbum"
                        | "MusicArtist"
                        | "Playlist"
                )
            )
    }

    pub fn is_media_leaf(&self) -> bool {
        !self.is_container() && matches!(self.media_type.as_deref(), Some("Audio" | "Video"))
    }
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct NameId {
    pub id: String,
    pub name: String,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct UserData {
    pub is_favorite: bool,
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod factory_tests;

#[cfg(test)]
mod audio_options_tests;
