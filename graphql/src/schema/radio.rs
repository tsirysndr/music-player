use async_graphql::*;
use music_player_entity::{album, saved_radio, track};
use music_player_playback::player::PlayerCommand;
use music_player_settings::{read_settings, Settings};
use music_player_storage::Database;
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, SimpleObject, InputObject)]
#[graphql(input_name = "RadioStationInput")]
pub struct RadioStation {
    pub id: String,
    pub name: String,
    pub stream_url: String,
    pub source: String,
    pub genre: String,
    pub country: String,
    pub logo: String,
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

fn from_row(row: saved_radio::Model) -> RadioStation {
    RadioStation {
        id: row.id,
        name: row.name,
        stream_url: row.stream_url,
        source: row.source,
        genre: row.genre,
        country: row.country,
        logo: row.logo,
        bitrate: row.bitrate,
    }
}

#[derive(Default)]
pub struct RadioQuery;

#[Object]
impl RadioQuery {
    async fn saved_radios(&self, ctx: &Context<'_>) -> Result<Vec<RadioStation>> {
        let db = ctx.data::<Database>()?;
        Ok(saved_radio::Entity::find()
            .all(db.get_connection())
            .await?
            .into_iter()
            .map(from_row)
            .collect())
    }

    async fn radios(
        &self,
        query: Option<String>,
        category: Option<String>,
    ) -> Result<Vec<RadioStation>> {
        let settings = read_settings()
            .map_err(|error| Error::new(error.to_string()))?
            .try_deserialize::<Settings>()
            .map_err(|error| Error::new(error.to_string()))?;
        let radio_browser = settings.radio_browser_url.trim_end_matches('/');
        let tunein = settings.tunein_url.trim_end_matches('/');
        let client = reqwest::Client::builder()
            .user_agent("music-player/0.2.1")
            .build()?;
        let rows: Vec<RbStation> = if let Some(ref category) = category {
            client
                .get(format!("{radio_browser}/json/stations/search"))
                .query(&[
                    ("tag", category.as_str()),
                    ("limit", "100"),
                    ("hidebroken", "true"),
                    ("order", "clickcount"),
                    ("reverse", "true"),
                ])
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?
        } else {
            client
                .get(format!("{radio_browser}/json/stations/search"))
                .query(&[
                    ("name", query.clone().unwrap_or_default()),
                    ("limit", "100".into()),
                    ("hidebroken", "true".into()),
                    ("order", "clickcount".into()),
                    ("reverse", "true".into()),
                ])
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?
        };
        let mut result: Vec<RadioStation> = rows
            .into_iter()
            .map(|row| RadioStation {
                id: format!("rb:{}", row.stationuuid),
                name: row.name,
                stream_url: if row.url_resolved.is_empty() {
                    row.url
                } else {
                    row.url_resolved
                },
                source: "Radio Browser".into(),
                genre: row.tags.split(',').next().unwrap_or_default().into(),
                country: row.country,
                logo: row.favicon,
                bitrate: row.bitrate,
            })
            .collect();
        if category.is_none() {
            let tunein = client.get(format!("{tunein}/Search.ashx")).query(&[
                ("query", query.clone().unwrap_or_default()),
                ("render", "json".to_string()),
                ("formats", "mp3,aac".to_string()),
            ]);
            if let Ok(response) = tunein.send().await {
                if let Ok(response) = response.error_for_status() {
                    if let Ok(response) = response.json::<TuneInResponse>().await {
                        result.extend(
                            response
                                .body
                                .into_iter()
                                .filter(|item| item.r#type == "audio" && !item.url.is_empty())
                                .map(|item| RadioStation {
                                    id: format!(
                                        "tunein:{}",
                                        if item.guide_id.is_empty() {
                                            item.url.clone()
                                        } else {
                                            item.guide_id
                                        }
                                    ),
                                    name: item.text,
                                    stream_url: item.url,
                                    source: "TuneIn".into(),
                                    genre: item.subtext,
                                    country: String::new(),
                                    logo: item.image,
                                    bitrate: item.bitrate.parse().unwrap_or_default(),
                                }),
                        );
                    }
                }
            }
        }
        Ok(result)
    }
}

#[derive(Default)]
pub struct RadioMutation;

#[Object]
impl RadioMutation {
    async fn save_radio(&self, ctx: &Context<'_>, station: RadioStation) -> Result<bool> {
        let db = ctx.data::<Database>()?;
        let row = saved_radio::Model {
            id: station.id,
            name: station.name,
            stream_url: station.stream_url,
            source: station.source,
            genre: station.genre,
            country: station.country,
            logo: station.logo,
            bitrate: station.bitrate,
        };
        saved_radio::ActiveModel {
            id: ActiveValue::Set(row.id.clone()),
            name: ActiveValue::Set(row.name.clone()),
            stream_url: ActiveValue::Set(row.stream_url.clone()),
            source: ActiveValue::Set(row.source.clone()),
            genre: ActiveValue::Set(row.genre.clone()),
            country: ActiveValue::Set(row.country.clone()),
            logo: ActiveValue::Set(row.logo.clone()),
            bitrate: ActiveValue::Set(row.bitrate),
        }
        .insert(db.get_connection())
        .await?;
        // Mirror the bookmark into the user's atproto repo. Signed out, this
        // does nothing; a failure there must not fail the local bookmark.
        if let Err(e) = music_player_storage::atradio::favorite(&row).await {
            tracing::warn!("could not mirror the bookmark to atradio.fm: {e}");
        }
        Ok(true)
    }

    /// Bookmark or unbookmark whatever station is playing, and report the new
    /// state — what the heart in the miniplayer needs.
    ///
    /// The station is rebuilt from the queued track rather than taken from the
    /// caller, so the miniplayer can toggle a bookmark without having to know
    /// where the station came from (it may be playing from a previous session).
    async fn toggle_current_radio_bookmark(&self, ctx: &Context<'_>) -> Result<bool> {
        let db = ctx.data::<Database>()?;
        let tracklist = ctx.data::<Arc<Mutex<music_player_tracklist::Tracklist>>>()?;
        let current = tracklist.lock().unwrap().current_track().0;
        let Some(current) = current else {
            return Ok(false);
        };
        let Some(id) = current.id.strip_prefix("radio:").map(str::to_owned) else {
            return Ok(false);
        };

        let conn = db.get_connection();
        if let Some(existing) = saved_radio::Entity::find_by_id(id.clone())
            .one(conn)
            .await?
        {
            saved_radio::Entity::delete_by_id(id).exec(conn).await?;
            if let Err(e) = music_player_storage::atradio::unfavorite(&existing).await {
                tracing::warn!("could not remove the bookmark on atradio.fm: {e}");
            }
            return Ok(false);
        }

        let row = saved_radio::Model {
            id: id.clone(),
            name: current.title.clone(),
            stream_url: current.uri.clone(),
            // The queue keeps the station's provider in the artist slot and its
            // logo as the album cover.
            source: current.artist.clone(),
            genre: String::new(),
            country: String::new(),
            logo: current.album.cover.clone().unwrap_or_default(),
            bitrate: current.bitrate.unwrap_or_default(),
        };
        saved_radio::ActiveModel {
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
        .await?;
        if let Err(e) = music_player_storage::atradio::favorite(&row).await {
            tracing::warn!("could not mirror the bookmark to atradio.fm: {e}");
        }
        Ok(true)
    }

    async fn remove_saved_radio(&self, ctx: &Context<'_>, id: String) -> Result<bool> {
        let db = ctx.data::<Database>()?;
        let row = saved_radio::Entity::find_by_id(id.clone())
            .one(db.get_connection())
            .await?;
        saved_radio::Entity::delete_by_id(id)
            .exec(db.get_connection())
            .await?;
        if let Some(row) = row {
            if let Err(e) = music_player_storage::atradio::unfavorite(&row).await {
                tracing::warn!("could not remove the bookmark on atradio.fm: {e}");
            }
        }
        Ok(true)
    }

    async fn play_radio(&self, ctx: &Context<'_>, station: RadioStation) -> Result<bool> {
        let sender = ctx.data::<Arc<Mutex<UnboundedSender<PlayerCommand>>>>()?;
        let model = track::Model {
            id: format!("radio:{}", station.id),
            title: station.name,
            artist: station.source,
            uri: station.stream_url,
            bitrate: Some(station.bitrate),
            album_id: Some("internet-radio".into()),
            album: album::Model {
                id: "internet-radio".into(),
                title: "Internet Radio".into(),
                cover: (!station.logo.is_empty()).then_some(station.logo),
                ..Default::default()
            },
            ..Default::default()
        };
        sender.lock().unwrap().send(PlayerCommand::LoadTracklist {
            tracks: vec![model],
        })?;
        Ok(true)
    }
}
