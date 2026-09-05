use std::time::Duration;

use anyhow::Error;
use music_player_types::types::{Album, Artist, SimplifiedSong};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};

const RESULT_LIMIT: u32 = 20;

/// Full-text search over the `track_search`, `album_search` and
/// `artist_search` SQLite FTS5 tables. The tables are kept in sync with
/// `track`, `album` and `artist` by triggers created in the
/// `m20260905_000001_create_search_index` migration, so this type is a pure
/// read-only query layer.
#[derive(Clone)]
pub struct Searcher {
    connection: DatabaseConnection,
}

impl Searcher {
    pub fn new(connection: DatabaseConnection) -> Self {
        Self { connection }
    }

    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    pub async fn search_song(&self, term: &str) -> Result<Vec<SimplifiedSong>, Error> {
        let match_query = match build_match_query(term) {
            Some(match_query) => match_query,
            None => return Ok(vec![]),
        };
        let sql = format!(
            r#"SELECT t.id, t.title, t.artist, t.genre, t.duration, t.artist_id, t.album_id,
                al.title AS album_title, al.cover AS album_cover
            FROM (SELECT id, rank FROM track_search WHERE track_search MATCH ? ORDER BY rank LIMIT {}) s
            JOIN track t ON t.id = s.id
            LEFT JOIN album al ON al.id = t.album_id
            ORDER BY s.rank"#,
            RESULT_LIMIT
        );
        let rows = self
            .connection
            .query_all(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                &sql,
                vec![match_query.into()],
            ))
            .await?;

        let mut songs = Vec::with_capacity(rows.len());
        for row in rows {
            songs.push(SimplifiedSong {
                id: row.try_get("", "id")?,
                title: row.try_get("", "title")?,
                artist: row.try_get("", "artist")?,
                album: row
                    .try_get::<Option<String>>("", "album_title")?
                    .unwrap_or_default(),
                genre: row.try_get("", "genre")?,
                duration: Duration::from_secs_f32(
                    row.try_get::<Option<f32>>("", "duration")?.unwrap_or(0.0),
                ),
                cover: normalize_cover(row.try_get("", "album_cover")?),
                artist_id: row
                    .try_get::<Option<String>>("", "artist_id")?
                    .unwrap_or_default(),
                album_id: row
                    .try_get::<Option<String>>("", "album_id")?
                    .unwrap_or_default(),
            });
        }
        Ok(songs)
    }

    pub async fn search_album(&self, term: &str) -> Result<Vec<Album>, Error> {
        let match_query = match build_match_query(term) {
            Some(match_query) => match_query,
            None => return Ok(vec![]),
        };
        let sql = format!(
            r#"SELECT al.id, al.title, al.artist, al.artist_id, al.year, al.cover
            FROM (SELECT id, rank FROM album_search WHERE album_search MATCH ? ORDER BY rank LIMIT {}) s
            JOIN album al ON al.id = s.id
            ORDER BY s.rank"#,
            RESULT_LIMIT
        );
        let rows = self
            .connection
            .query_all(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                &sql,
                vec![match_query.into()],
            ))
            .await?;

        let mut albums = Vec::with_capacity(rows.len());
        for row in rows {
            albums.push(Album {
                id: row.try_get("", "id")?,
                title: row.try_get("", "title")?,
                artist: row.try_get("", "artist")?,
                artist_id: row.try_get("", "artist_id")?,
                year: row
                    .try_get::<Option<i32>>("", "year")?
                    .map(|year| year as u32),
                cover: normalize_cover(row.try_get("", "cover")?),
                ..Default::default()
            });
        }
        Ok(albums)
    }

    pub async fn search_artist(&self, term: &str) -> Result<Vec<Artist>, Error> {
        let match_query = match build_match_query(term) {
            Some(match_query) => match_query,
            None => return Ok(vec![]),
        };
        let sql = format!(
            r#"SELECT ar.id, ar.name
            FROM (SELECT id, rank FROM artist_search WHERE artist_search MATCH ? ORDER BY rank LIMIT {}) s
            JOIN artist ar ON ar.id = s.id
            ORDER BY s.rank"#,
            RESULT_LIMIT
        );
        let rows = self
            .connection
            .query_all(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                &sql,
                vec![match_query.into()],
            ))
            .await?;

        let mut artists = Vec::with_capacity(rows.len());
        for row in rows {
            artists.push(Artist {
                id: row.try_get("", "id")?,
                name: row.try_get("", "name")?,
                ..Default::default()
            });
        }
        Ok(artists)
    }
}

/// Builds a safe FTS5 MATCH expression from arbitrary user input.
///
/// The input is split on any non-alphanumeric character, each token is
/// wrapped in double quotes (so it can never be interpreted as FTS5 query
/// syntax) and given a `*` suffix for prefix matching. Returns `None` when
/// the input contains no usable token.
fn build_match_query(term: &str) -> Option<String> {
    let tokens: Vec<String> = term
        .split(|c: char| !c.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| format!("\"{}\"*", token))
        .collect();
    if tokens.is_empty() {
        None
    } else {
        Some(tokens.join(" "))
    }
}

fn normalize_cover(cover: Option<String>) -> Option<String> {
    match cover.as_deref() {
        Some("") | None => None,
        Some(_) => cover,
    }
}
