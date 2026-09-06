pub mod typesense;

use std::time::Duration;

use anyhow::Error;
use music_player_settings::read_typesense_settings;
use music_player_types::types::{Album, Artist, SimplifiedSong};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};

use self::typesense::Typesense;

const RESULT_LIMIT: u32 = 20;

/// Library full-text search.
///
/// Default backend: the `track_search`, `album_search` and `artist_search`
/// SQLite FTS5 tables, kept in sync with `track`, `album` and `artist` by
/// triggers created in the `m20260905_000001_create_search_index` migration.
///
/// When settings.toml has a `[typesense]` table (url + api_key), queries go
/// to Typesense instead — with a silent fallback to FTS5 when the server is
/// unreachable, so search never breaks outright. `reindex` re-syncs the
/// Typesense collections and is called after every library scan.
#[derive(Clone)]
pub struct Searcher {
    connection: DatabaseConnection,
    typesense: Option<Typesense>,
}

impl Searcher {
    pub fn new(connection: DatabaseConnection) -> Self {
        let typesense = read_typesense_settings().as_ref().map(Typesense::new);
        Self {
            connection,
            typesense,
        }
    }

    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    /// Re-sync the Typesense collections from the database. No-op on the
    /// FTS5 backend (its triggers keep the index current).
    pub async fn reindex(&self) -> Result<(), Error> {
        if let Some(ts) = &self.typesense {
            ts.reindex(&self.connection).await?;
        }
        Ok(())
    }

    pub async fn search_song(&self, term: &str) -> Result<Vec<SimplifiedSong>, Error> {
        if let Some(ts) = &self.typesense {
            match ts.search_song(term).await {
                Ok(songs) => return Ok(songs),
                Err(e) => tracing::warn!("typesense song search failed, using FTS5: {e}"),
            }
        }
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
        if let Some(ts) = &self.typesense {
            match ts.search_album(term).await {
                Ok(albums) => return Ok(albums),
                Err(e) => tracing::warn!("typesense album search failed, using FTS5: {e}"),
            }
        }
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
        if let Some(ts) = &self.typesense {
            match ts.search_artist(term).await {
                Ok(artists) => return Ok(artists),
                Err(e) => tracing::warn!("typesense artist search failed, using FTS5: {e}"),
            }
        }
        let match_query = match build_match_query(term) {
            Some(match_query) => match_query,
            None => return Ok(vec![]),
        };
        let sql = format!(
            r#"SELECT ar.id, ar.name, ar.picture
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
                picture: row.try_get::<Option<String>>("", "picture")?,
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
