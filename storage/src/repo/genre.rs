//! Genres, and what belongs to them.
//!
//! A track reaches a genre by either route: its own tag (`track_genres`), or
//! the genre its artist is tagged with (`artist_genres`). Most files carry no
//! usable genre tag, so relying on the first alone would show a nearly empty
//! screen; relying on the second alone would mis-file every compilation. The
//! queries below union them.

use anyhow::Error;
use music_player_entity::{genre, track};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait, Statement};

pub struct GenreRepository<'a> {
    conn: &'a DatabaseConnection,
}

impl<'a> GenreRepository<'a> {
    pub fn new(conn: &'a DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Every genre with at least one track, and how many it has.
    ///
    /// Genres with nothing in them are omitted rather than listed as empty:
    /// they are an artefact of an artist's tags outliving their tracks, and a
    /// row that opens onto nothing is worse than no row.
    pub async fn find_all(
        &self,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Vec<genre::Model>, Error> {
        let mut sql = String::from(
            r#"
            SELECT g.id, g.name, COUNT(DISTINCT t.id) AS track_count
            FROM genre g
            LEFT JOIN track_genres tg ON tg.genre_id = g.id
            LEFT JOIN artist_genres ag ON ag.genre_id = g.id
            LEFT JOIN track t
                   ON t.id = tg.track_id
                   OR t.artist_id = ag.artist_id
            GROUP BY g.id, g.name
            HAVING track_count > 0
            ORDER BY g.name COLLATE NOCASE ASC
            "#,
        );
        if let Some(limit) = limit {
            sql.push_str(&format!(" LIMIT {limit}"));
            sql.push_str(&format!(" OFFSET {}", offset.unwrap_or(0)));
        }

        let rows = self
            .conn
            .query_all(Statement::from_string(DbBackend::Sqlite, sql))
            .await?;
        rows.into_iter()
            .map(|row| {
                Ok(genre::Model {
                    id: row.try_get("", "id")?,
                    name: row.try_get("", "name")?,
                    track_count: row.try_get::<i64>("", "track_count").unwrap_or(0).max(0) as u32,
                })
            })
            .collect()
    }

    pub async fn find(&self, id: &str) -> Result<genre::Model, Error> {
        genre::Entity::find_by_id(id.to_owned())
            .one(self.conn)
            .await?
            .ok_or_else(|| Error::msg(format!("no such genre: {id}")))
    }

    /// The tracks in a genre, by either route.
    pub async fn tracks(
        &self,
        id: &str,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Vec<track::Model>, Error> {
        let mut sql = String::from(
            r#"
            SELECT DISTINCT t.id
            FROM track t
            LEFT JOIN track_genres tg ON tg.track_id = t.id
            LEFT JOIN artist_genres ag ON ag.artist_id = t.artist_id
            WHERE tg.genre_id = $1 OR ag.genre_id = $1
            ORDER BY t.artist COLLATE NOCASE, t.title COLLATE NOCASE
            "#,
        );
        if let Some(limit) = limit {
            sql.push_str(&format!(" LIMIT {limit}"));
            sql.push_str(&format!(" OFFSET {}", offset.unwrap_or(0)));
        }

        let rows = self
            .conn
            .query_all(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                &sql,
                [id.into()],
            ))
            .await?;
        let ids: Vec<String> = rows
            .into_iter()
            .filter_map(|row| row.try_get::<String>("", "id").ok())
            .collect();
        if ids.is_empty() {
            return Ok(vec![]);
        }

        // Fetched through the repository so the rows arrive with their album
        // and artists attached, as every other track listing does.
        let repository = super::track::TrackRepository::new(self.conn);
        let mut tracks = Vec::with_capacity(ids.len());
        for id in ids {
            if let Ok(track) = repository.find(&id).await {
                tracks.push(track);
            }
        }
        Ok(tracks)
    }
}
