//! Typesense search backend. Active when settings.toml has a `[typesense]`
//! table (url + api_key); the `songs`, `albums` and `artists` collections
//! are rebuilt from the database after every library scan (`reindex`) and
//! queried by `Searcher` in place of the SQLite FTS5 tables.

use std::time::Duration;

use anyhow::Error;
use music_player_settings::TypesenseSettings;
use music_player_types::types::{Album, Artist, SimplifiedSong};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use serde::Deserialize;
use serde_json::{json, Value};

const RESULT_LIMIT: u32 = 20;

pub const SONGS: &str = "songs";
pub const ALBUMS: &str = "albums";
pub const ARTISTS: &str = "artists";

#[derive(Clone)]
pub struct Typesense {
    http: reqwest::Client,
    url: String,
    api_key: String,
}

#[derive(Deserialize)]
struct SearchHit {
    document: Value,
}

#[derive(Deserialize)]
struct SearchResponse {
    #[serde(default)]
    hits: Vec<SearchHit>,
}

impl Typesense {
    pub fn new(settings: &TypesenseSettings) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("http client"),
            url: settings.url.trim_end_matches('/').to_string(),
            api_key: settings.api_key.clone(),
        }
    }

    async fn search(
        &self,
        collection: &str,
        term: &str,
        query_by: &str,
    ) -> Result<Vec<Value>, Error> {
        let resp = self
            .http
            .get(format!(
                "{}/collections/{collection}/documents/search",
                self.url
            ))
            .header("x-typesense-api-key", &self.api_key)
            .query(&[
                ("q", term),
                ("query_by", query_by),
                ("per_page", &RESULT_LIMIT.to_string()),
                ("prefix", "true"),
            ])
            .send()
            .await?
            .error_for_status()?;
        let body: SearchResponse = resp.json().await?;
        Ok(body.hits.into_iter().map(|h| h.document).collect())
    }

    pub async fn search_song(&self, term: &str) -> Result<Vec<SimplifiedSong>, Error> {
        let docs = self.search(SONGS, term, "title,artist,album,genre").await?;
        Ok(docs
            .iter()
            .map(|d| SimplifiedSong {
                id: str_field(d, "id"),
                title: str_field(d, "title"),
                artist: str_field(d, "artist"),
                album: str_field(d, "album"),
                genre: str_field(d, "genre"),
                duration: Duration::from_secs_f32(
                    d.get("duration").and_then(Value::as_f64).unwrap_or(0.0) as f32,
                ),
                cover: opt_field(d, "cover"),
                artist_id: str_field(d, "artist_id"),
                album_id: str_field(d, "album_id"),
            })
            .collect())
    }

    pub async fn search_album(&self, term: &str) -> Result<Vec<Album>, Error> {
        let docs = self.search(ALBUMS, term, "title,artist").await?;
        Ok(docs
            .iter()
            .map(|d| Album {
                id: str_field(d, "id"),
                title: str_field(d, "title"),
                artist: str_field(d, "artist"),
                artist_id: opt_field(d, "artist_id"),
                year: d.get("year").and_then(Value::as_i64).map(|y| y as u32),
                cover: opt_field(d, "cover"),
                ..Default::default()
            })
            .collect())
    }

    pub async fn search_artist(&self, term: &str) -> Result<Vec<Artist>, Error> {
        let docs = self.search(ARTISTS, term, "name").await?;
        Ok(docs
            .iter()
            .map(|d| Artist {
                id: str_field(d, "id"),
                name: str_field(d, "name"),
                ..Default::default()
            })
            .collect())
    }

    /// Rebuild a collection: drop (ignoring "not found"), create with an
    /// auto schema, then bulk-upsert `docs` as JSONL.
    async fn rebuild(&self, collection: &str, docs: Vec<Value>) -> Result<(), Error> {
        let _ = self
            .http
            .delete(format!("{}/collections/{collection}", self.url))
            .header("x-typesense-api-key", &self.api_key)
            .send()
            .await;
        self.http
            .post(format!("{}/collections", self.url))
            .header("x-typesense-api-key", &self.api_key)
            .json(&json!({
                "name": collection,
                "fields": [{ "name": ".*", "type": "auto" }],
            }))
            .send()
            .await?
            .error_for_status()?;
        if docs.is_empty() {
            return Ok(());
        }
        let body: String = docs
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        self.http
            .post(format!(
                "{}/collections/{collection}/documents/import?action=upsert",
                self.url
            ))
            .header("x-typesense-api-key", &self.api_key)
            .header("Content-Type", "text/plain")
            .body(body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Full sync from the database — called after every library scan.
    pub async fn reindex(&self, connection: &DatabaseConnection) -> Result<(), Error> {
        let songs = connection
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                r#"SELECT t.id, t.title, t.artist, t.genre, t.duration, t.artist_id, t.album_id,
                    al.title AS album_title, al.cover AS album_cover
                FROM track t LEFT JOIN album al ON al.id = t.album_id"#
                    .to_owned(),
            ))
            .await?
            .iter()
            .map(|row| {
                Ok(json!({
                    "id": row.try_get::<String>("", "id")?,
                    "title": row.try_get::<String>("", "title")?,
                    "artist": row.try_get::<String>("", "artist")?,
                    "genre": row.try_get::<String>("", "genre")?,
                    "duration": row.try_get::<Option<f32>>("", "duration")?.unwrap_or(0.0),
                    "artist_id": row.try_get::<Option<String>>("", "artist_id")?.unwrap_or_default(),
                    "album_id": row.try_get::<Option<String>>("", "album_id")?.unwrap_or_default(),
                    "album": row.try_get::<Option<String>>("", "album_title")?.unwrap_or_default(),
                    "cover": row.try_get::<Option<String>>("", "album_cover")?.unwrap_or_default(),
                }))
            })
            .collect::<Result<Vec<Value>, Error>>()?;

        let albums = connection
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT id, title, artist, artist_id, year, cover FROM album".to_owned(),
            ))
            .await?
            .iter()
            .map(|row| {
                Ok(json!({
                    "id": row.try_get::<String>("", "id")?,
                    "title": row.try_get::<String>("", "title")?,
                    "artist": row.try_get::<String>("", "artist")?,
                    "artist_id": row.try_get::<Option<String>>("", "artist_id")?.unwrap_or_default(),
                    "year": row.try_get::<Option<i32>>("", "year")?.unwrap_or(0),
                    "cover": row.try_get::<Option<String>>("", "cover")?.unwrap_or_default(),
                }))
            })
            .collect::<Result<Vec<Value>, Error>>()?;

        let artists = connection
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT id, name FROM artist".to_owned(),
            ))
            .await?
            .iter()
            .map(|row| {
                Ok(json!({
                    "id": row.try_get::<String>("", "id")?,
                    "name": row.try_get::<String>("", "name")?,
                }))
            })
            .collect::<Result<Vec<Value>, Error>>()?;

        let counts = (songs.len(), albums.len(), artists.len());
        self.rebuild(SONGS, songs).await?;
        self.rebuild(ALBUMS, albums).await?;
        self.rebuild(ARTISTS, artists).await?;
        tracing::info!(
            songs = counts.0,
            albums = counts.1,
            artists = counts.2,
            "typesense reindex complete"
        );
        Ok(())
    }
}

fn str_field(doc: &Value, field: &str) -> String {
    doc.get(field)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn opt_field(doc: &Value, field: &str) -> Option<String> {
    doc.get(field)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(String::from)
}
