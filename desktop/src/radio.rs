use music_player_entity::saved_radio;
use music_player_settings::{
    read_settings, Settings, DEFAULT_RADIO_BROWSER_URL, DEFAULT_TUNEIN_URL,
};
use music_player_storage::Database;
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub stream_url: String,
    pub source: String,
    #[serde(default)]
    pub genre: String,
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub logo: String,
    #[serde(default)]
    pub bitrate: u32,
}

#[derive(Deserialize)]
struct RbStation {
    stationuuid: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    url_resolved: String,
    #[serde(default)]
    favicon: String,
    #[serde(default)]
    tags: String,
    #[serde(default)]
    country: String,
    #[serde(default)]
    bitrate: u32,
}

#[derive(Deserialize)]
struct TuneInResponse {
    #[serde(default)]
    body: Vec<TuneInItem>,
}

#[derive(Deserialize)]
struct TuneInItem {
    #[serde(default)]
    r#type: String,
    #[serde(default)]
    text: String,
    #[serde(default, rename = "URL")]
    url: String,
    #[serde(default)]
    guide_id: String,
    #[serde(default)]
    subtext: String,
    #[serde(default)]
    image: String,
    #[serde(default)]
    bitrate: String,
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .user_agent(concat!("music-player-desktop/", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("radio HTTP client")
}

fn api_urls() -> (String, String) {
    read_settings()
        .ok()
        .and_then(|config| config.try_deserialize::<Settings>().ok())
        .map(|settings| {
            (
                settings.radio_browser_url.trim_end_matches('/').to_owned(),
                settings.tunein_url.trim_end_matches('/').to_owned(),
            )
        })
        .unwrap_or_else(|| (DEFAULT_RADIO_BROWSER_URL.into(), DEFAULT_TUNEIN_URL.into()))
}

pub async fn search(query: &str) -> Vec<Station> {
    let (radio_browser, tunein) = tokio::join!(search_radio_browser(query), search_tunein(query));
    let mut stations = radio_browser.unwrap_or_default();
    stations.extend(tunein.unwrap_or_default());
    stations
}

pub async fn browse(tag: &str) -> anyhow::Result<Vec<Station>> {
    let (radio_browser, _) = api_urls();
    let rows: Vec<RbStation> = client()
        .get(format!("{radio_browser}/json/stations/search"))
        .query(&[
            ("tag", tag),
            ("limit", "100"),
            ("hidebroken", "true"),
            ("order", "clickcount"),
            ("reverse", "true"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(rows.into_iter().map(from_rb).collect())
}

async fn search_radio_browser(query: &str) -> anyhow::Result<Vec<Station>> {
    let (radio_browser, _) = api_urls();
    let rows: Vec<RbStation> = client()
        .get(format!("{radio_browser}/json/stations/search"))
        .query(&[
            ("name", query),
            ("limit", "50"),
            ("hidebroken", "true"),
            ("order", "clickcount"),
            ("reverse", "true"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(rows.into_iter().map(from_rb).collect())
}

async fn search_tunein(query: &str) -> anyhow::Result<Vec<Station>> {
    let (_, tunein) = api_urls();
    let response: TuneInResponse = client()
        .get(format!("{tunein}/Search.ashx"))
        .query(&[("query", query), ("render", "json"), ("formats", "mp3,aac")])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(response
        .body
        .into_iter()
        .filter(|item| item.r#type == "audio" && !item.url.is_empty())
        .map(|item| Station {
            id: format!(
                "tunein:{}",
                if item.guide_id.is_empty() {
                    &item.url
                } else {
                    &item.guide_id
                }
            ),
            name: item.text,
            stream_url: item.url,
            source: "TuneIn".into(),
            genre: item.subtext,
            country: String::new(),
            logo: item.image,
            bitrate: item.bitrate.parse().unwrap_or_default(),
        })
        .collect())
}

fn from_rb(row: RbStation) -> Station {
    Station {
        id: format!("rb:{}", row.stationuuid),
        name: row.name.trim().to_owned(),
        stream_url: if row.url_resolved.is_empty() {
            row.url
        } else {
            row.url_resolved
        },
        source: "Radio Browser".into(),
        genre: row
            .tags
            .split(',')
            .next()
            .unwrap_or_default()
            .trim()
            .to_owned(),
        country: row.country,
        logo: row.favicon,
        bitrate: row.bitrate,
    }
}

fn logo_cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|dir| dir.join("music-player").join("radio-logos"))
}

/// Station logo bytes, cached on disk between runs. Directories hand out a
/// couple hundred logos per search and most of them come back on the next
/// search too, so re-downloading them every time is pure latency.
pub async fn logo_bytes(url: &str) -> Option<Vec<u8>> {
    if url.is_empty() {
        return None;
    }
    let cached = logo_cache_dir().map(|dir| dir.join(format!("{:x}", md5::compute(url))));
    if let Some(path) = &cached {
        if let Ok(bytes) = tokio::fs::read(path).await {
            if !bytes.is_empty() {
                return Some(bytes);
            }
        }
    }

    let bytes = client()
        .get(url)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .bytes()
        .await
        .ok()?;
    if bytes.is_empty() {
        return None;
    }
    if let Some(path) = &cached {
        if let Some(parent) = path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }
        if let Err(e) = tokio::fs::write(path, &bytes).await {
            tracing::debug!("could not cache radio logo {url}: {e}");
        }
    }
    Some(bytes.to_vec())
}

pub async fn resolve_stream(station: &Station) -> String {
    let lower = station.stream_url.to_lowercase();
    let playlist = station.id.starts_with("tunein:")
        || lower.contains("tune.ashx")
        || lower
            .split('?')
            .next()
            .is_some_and(|p| p.ends_with(".pls") || p.ends_with(".m3u"));
    if !playlist {
        return station.stream_url.clone();
    }
    let Ok(response) = client().get(&station.stream_url).send().await else {
        return station.stream_url.clone();
    };
    let Ok(body) = response.text().await else {
        return station.stream_url.clone();
    };
    for line in body.lines() {
        let line = line.trim();
        let candidate = line
            .strip_prefix("File1=")
            .or_else(|| (!line.starts_with('#')).then_some(line));
        if let Some(url) =
            candidate.filter(|url| url.starts_with("http://") || url.starts_with("https://"))
        {
            return url.to_owned();
        }
    }
    station.stream_url.clone()
}

pub async fn load_bookmarks() -> Vec<Station> {
    let db = Database::new().await;
    saved_radio::Entity::find()
        .all(db.get_connection())
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|row| Station {
            id: row.id,
            name: row.name,
            stream_url: row.stream_url,
            source: row.source,
            genre: row.genre,
            country: row.country,
            logo: row.logo,
            bitrate: row.bitrate,
        })
        .collect()
}

pub async fn toggle_bookmark(station: &Station) -> bool {
    let db = Database::new().await;
    let conn = db.get_connection();
    let row = saved_radio::Model {
        id: station.id.clone(),
        name: station.name.clone(),
        stream_url: station.stream_url.clone(),
        source: station.source.clone(),
        genre: station.genre.clone(),
        country: station.country.clone(),
        logo: station.logo.clone(),
        bitrate: station.bitrate,
    };
    if saved_radio::Entity::find_by_id(station.id.clone())
        .one(conn)
        .await
        .ok()
        .flatten()
        .is_some()
    {
        let _ = saved_radio::Entity::delete_by_id(station.id.clone())
            .exec(conn)
            .await;
        // Mirror into the user's atproto repo; a no-op when signed out.
        if let Err(e) = music_player_storage::atradio::unfavorite(&row).await {
            tracing::warn!("could not remove the bookmark on atradio.fm: {e}");
        }
        false
    } else {
        let saved = saved_radio::ActiveModel {
            id: ActiveValue::Set(row.id.clone()),
            name: ActiveValue::Set(row.name.clone()),
            stream_url: ActiveValue::Set(row.stream_url.clone()),
            source: ActiveValue::Set(row.source.clone()),
            genre: ActiveValue::Set(row.genre.clone()),
            country: ActiveValue::Set(row.country.clone()),
            logo: ActiveValue::Set(row.logo.clone()),
            bitrate: ActiveValue::Set(row.bitrate),
        }
        .insert(conn)
        .await
        .is_ok();
        if saved {
            if let Err(e) = music_player_storage::atradio::favorite(&row).await {
                tracing::warn!("could not mirror the bookmark to atradio.fm: {e}");
            }
        }
        saved
    }
}
