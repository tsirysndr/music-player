use music_player_entity::saved_radio;
use music_player_settings::{
    read_settings, Settings, DEFAULT_RADIO_BROWSER_URL, DEFAULT_TUNEIN_URL,
};
use music_player_storage::Database;
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const CATEGORIES: &[(&str, &str)] = &[
    ("Synthwave", "synthwave"),
    ("Lo-fi", "lofi"),
    ("Jazz", "jazz"),
    ("Techno", "techno"),
    ("Ambient", "ambient"),
    ("Classical", "classical"),
    ("Rock", "rock"),
    ("Pop", "pop"),
    ("Electronic", "electronic"),
    ("Hip-Hop", "hip hop"),
    ("Chillout", "chill"),
    ("Dance", "dance"),
    ("Reggae", "reggae"),
    ("Metal", "metal"),
    ("News", "news"),
    ("World", "world"),
    ("House", "house"),
    ("Trance", "trance"),
    ("Drum & Bass", "drum and bass"),
    ("Disco", "disco"),
    ("Funk", "funk"),
    ("Soul", "soul"),
    ("R&B", "r&b"),
    ("Blues", "blues"),
    ("Country", "country"),
    ("Folk", "folk"),
    ("Punk", "punk"),
    ("Indie", "indie"),
    ("Latin", "latin"),
    ("K-Pop", "k-pop"),
    ("Gospel", "gospel"),
    ("Oldies", "oldies"),
    ("Soundtrack", "soundtrack"),
];

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
        false
    } else {
        let row = saved_radio::ActiveModel {
            id: ActiveValue::Set(station.id.clone()),
            name: ActiveValue::Set(station.name.clone()),
            stream_url: ActiveValue::Set(station.stream_url.clone()),
            source: ActiveValue::Set(station.source.clone()),
            genre: ActiveValue::Set(station.genre.clone()),
            country: ActiveValue::Set(station.country.clone()),
            logo: ActiveValue::Set(station.logo.clone()),
            bitrate: ActiveValue::Set(station.bitrate),
        };
        row.insert(conn).await.is_ok()
    }
}
