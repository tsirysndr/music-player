//! Filling a smart playlist from its RSQL filter.
//!
//! A smart playlist stores a query, not a track list. Regenerating it runs that
//! query and rewrites `playlist_tracks` — so everything downstream (the queue,
//! the detail view, gRPC, the web UI) sees an ordinary playlist and needs to
//! know nothing about any of this.

use anyhow::{anyhow, Error};
use music_player_entity::{playlist, playlist_tracks};
use music_player_rsql::{build_at, QuerySpec, SortOrder, Value, TRACKS};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend,
    EntityTrait, QueryFilter, Statement, TransactionTrait,
};

/// Turn a playlist row into the query it stands for.
pub fn spec_of(row: &playlist::Model) -> QuerySpec {
    QuerySpec {
        filter: row.rsql.clone().unwrap_or_default(),
        sort_by: row.sort_by.clone().filter(|s| !s.trim().is_empty()),
        sort_order: match row.sort_order.as_deref() {
            Some(value) if value.eq_ignore_ascii_case("desc") => SortOrder::Desc,
            _ => SortOrder::Asc,
        },
        limit: row.max_tracks,
    }
}

/// Run `spec` against the library and return the matching track ids, in order.
///
/// This is also what the "N tracks match" preview under the filter box calls,
/// so it must be safe to run on every keystroke: it selects ids only, and the
/// filter's own limit caps the work.
pub async fn matching_track_ids(
    conn: &DatabaseConnection,
    spec: &QuerySpec,
) -> Result<Vec<String>, Error> {
    matching_track_ids_at(conn, spec, unix_now()).await
}

async fn matching_track_ids_at(
    conn: &DatabaseConnection,
    spec: &QuerySpec,
    now: i64,
) -> Result<Vec<String>, Error> {
    let query = build_at(spec, &TRACKS, now).map_err(|e| anyhow!("{e}"))?;
    let params: Vec<sea_orm::Value> = query
        .params
        .into_iter()
        .map(|value| match value {
            Value::Text(text) => text.into(),
            Value::Integer(number) => number.into(),
        })
        .collect();
    let rows = conn
        .query_all(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            &query.sql,
            params,
        ))
        .await?;
    rows.into_iter()
        .map(|row| row.try_get::<String>("", "id").map_err(Error::from))
        .collect()
}

/// Regenerate `playlist_id`'s tracks from its filter. Returns how many tracks
/// it now holds.
///
/// The rewrite is one transaction: a smart playlist is never briefly empty
/// while something else is reading it.
pub async fn regenerate(conn: &DatabaseConnection, playlist_id: &str) -> Result<usize, Error> {
    let row = playlist::Entity::find_by_id(playlist_id.to_owned())
        .one(conn)
        .await?
        .ok_or_else(|| anyhow!("no playlist {playlist_id}"))?;
    if !row.is_smart {
        return Err(anyhow!("playlist {playlist_id} is not a smart playlist"));
    }

    let ids = matching_track_ids(conn, &spec_of(&row)).await?;
    let count = ids.len();

    let txn = conn.begin().await?;
    playlist_tracks::Entity::delete_many()
        .filter(playlist_tracks::Column::PlaylistId.eq(playlist_id.to_owned()))
        .exec(&txn)
        .await?;
    let now = chrono::Utc::now();
    for (position, track_id) in ids.into_iter().enumerate() {
        playlist_tracks::ActiveModel {
            // The row id has to be stable per (playlist, position) so a
            // regeneration replaces rows rather than growing the table.
            id: ActiveValue::Set(format!(
                "{:x}",
                md5::compute(format!("{playlist_id}:{position}"))
            )),
            playlist_id: ActiveValue::Set(playlist_id.to_owned()),
            track_id: ActiveValue::Set(track_id),
            created_at: ActiveValue::Set(now),
        }
        .insert(&txn)
        .await?;
    }
    playlist::ActiveModel {
        id: ActiveValue::Unchanged(playlist_id.to_owned()),
        refreshed_at: ActiveValue::Set(Some(now)),
        ..Default::default()
    }
    .update(&txn)
    .await?;
    txn.commit().await?;

    tracing::info!(
        playlist = playlist_id,
        count,
        "regenerated a smart playlist"
    );
    Ok(count)
}

/// Regenerate every smart playlist. Called after a library scan, when the
/// answers have changed underneath every stored filter.
pub async fn regenerate_all(conn: &DatabaseConnection) -> Result<usize, Error> {
    let rows = playlist::Entity::find()
        .filter(playlist::Column::IsSmart.eq(true))
        .all(conn)
        .await?;
    let mut refreshed = 0;
    for row in rows {
        match regenerate(conn, &row.id).await {
            Ok(_) => refreshed += 1,
            // One broken filter must not stop the rest from refreshing.
            Err(e) => {
                tracing::warn!(playlist = %row.id, name = %row.name, "could not regenerate: {e}")
            }
        }
    }
    Ok(refreshed)
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use music_player_entity::{album, track, track_stats};
    use sea_orm::Database;

    async fn library() -> (tempfile::TempDir, DatabaseConnection) {
        use migration::{Migrator, MigratorTrait};
        let dir = tempfile::tempdir().unwrap();
        let url = format!(
            "sqlite://{}?mode=rwc",
            dir.path().join("music-player.sqlite3").display()
        );
        let conn = Database::connect(&url).await.unwrap();
        Migrator::up(&conn, None).await.unwrap();

        album::ActiveModel {
            id: ActiveValue::Set("album-1".into()),
            title: ActiveValue::Set("OK Computer".into()),
            artist: ActiveValue::Set("Radiohead".into()),
            year: ActiveValue::Set(Some(1997)),
            ..Default::default()
        }
        .insert(&conn)
        .await
        .unwrap();

        for (id, title, artist, genre, year, plays) in [
            ("t1", "Airbag", "Radiohead", "rock", 1997, 12),
            ("t2", "Karma Police", "Radiohead", "rock", 1997, 30),
            ("t3", "So What", "Miles Davis", "jazz", 1959, 3),
            ("t4", "Untagged", "Nobody", "", 2020, 0),
        ] {
            track::ActiveModel {
                id: ActiveValue::Set(id.into()),
                title: ActiveValue::Set(title.into()),
                artist: ActiveValue::Set(artist.into()),
                genre: ActiveValue::Set(genre.into()),
                year: ActiveValue::Set(Some(year)),
                uri: ActiveValue::Set(format!("/music/{id}.mp3")),
                album_id: ActiveValue::Set(Some("album-1".into())),
                created_at: ActiveValue::Set(Some(chrono::Utc::now())),
                ..Default::default()
            }
            .insert(&conn)
            .await
            .unwrap();
            if plays > 0 {
                track_stats::ActiveModel {
                    track_id: ActiveValue::Set(id.into()),
                    play_count: ActiveValue::Set(plays),
                    skip_count: ActiveValue::Set(0),
                    last_played: ActiveValue::Set(Some(1_700_000_000)),
                    last_skipped: ActiveValue::Set(None),
                    updated_at: ActiveValue::Set(1_700_000_000),
                }
                .insert(&conn)
                .await
                .unwrap();
            }
        }
        (dir, conn)
    }

    async fn ids(conn: &DatabaseConnection, spec: QuerySpec) -> Vec<String> {
        matching_track_ids_at(conn, &spec, 1_700_000_000)
            .await
            .unwrap()
    }

    fn spec(filter: &str) -> QuerySpec {
        QuerySpec {
            filter: filter.into(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn filters_on_track_and_joined_album_fields() {
        let (_dir, conn) = library().await;
        assert_eq!(ids(&conn, spec("genre==rock")).await, vec!["t1", "t2"]);
        assert_eq!(ids(&conn, spec("artist=='Miles Davis'")).await, vec!["t3"]);
        // `album` lives on the joined album row, not on track.
        assert_eq!(ids(&conn, spec("album=='OK Computer'")).await.len(), 4);
    }

    /// A track with no `track_stats` row has been played zero times, and has to
    /// come back from `playcount==0` — the COALESCE in the schema is what makes
    /// "never played" work at all.
    #[tokio::test]
    async fn tracks_with_no_stats_count_as_never_played() {
        let (_dir, conn) = library().await;
        assert_eq!(ids(&conn, spec("playcount==0")).await, vec!["t4"]);
        assert_eq!(ids(&conn, spec("playcount>5")).await, vec!["t1", "t2"]);
    }

    /// In SQL `NULL <> 'rock'` is NULL. Without the compiler's explicit
    /// IS NULL arm the untagged track would silently vanish here.
    #[tokio::test]
    async fn negation_keeps_untagged_tracks() {
        let (_dir, conn) = library().await;
        let mut found = ids(&conn, spec("genre!=rock")).await;
        found.sort();
        assert_eq!(found, vec!["t3", "t4"]);
    }

    #[tokio::test]
    async fn sorts_and_limits() {
        let (_dir, conn) = library().await;
        let top = ids(
            &conn,
            QuerySpec {
                filter: "playcount>0".into(),
                sort_by: Some("playcount".into()),
                sort_order: SortOrder::Desc,
                limit: Some(2),
            },
        )
        .await;
        assert_eq!(top, vec!["t2", "t1"]);
    }

    #[tokio::test]
    async fn regenerating_replaces_the_tracks() {
        let (_dir, conn) = library().await;
        playlist::ActiveModel {
            id: ActiveValue::Set("pl-1".into()),
            name: ActiveValue::Set("Rock".into()),
            created_at: ActiveValue::Set(chrono::Utc::now()),
            is_smart: ActiveValue::Set(true),
            rsql: ActiveValue::Set(Some("genre==rock".into())),
            ..Default::default()
        }
        .insert(&conn)
        .await
        .unwrap();

        assert_eq!(regenerate(&conn, "pl-1").await.unwrap(), 2);
        // Running it again must replace, not accumulate.
        assert_eq!(regenerate(&conn, "pl-1").await.unwrap(), 2);
        let rows = playlist_tracks::Entity::find()
            .filter(playlist_tracks::Column::PlaylistId.eq("pl-1"))
            .all(&conn)
            .await
            .unwrap();
        assert_eq!(rows.len(), 2);

        let row = playlist::Entity::find_by_id("pl-1".to_owned())
            .one(&conn)
            .await
            .unwrap()
            .unwrap();
        assert!(row.refreshed_at.is_some());
    }

    #[tokio::test]
    async fn refuses_to_regenerate_an_ordinary_playlist() {
        let (_dir, conn) = library().await;
        playlist::ActiveModel {
            id: ActiveValue::Set("pl-2".into()),
            name: ActiveValue::Set("Hand picked".into()),
            created_at: ActiveValue::Set(chrono::Utc::now()),
            is_smart: ActiveValue::Set(false),
            ..Default::default()
        }
        .insert(&conn)
        .await
        .unwrap();
        assert!(regenerate(&conn, "pl-2").await.is_err());
    }

    #[tokio::test]
    async fn a_bad_filter_is_an_error_not_a_panic() {
        let (_dir, conn) = library().await;
        let e = matching_track_ids(&conn, &spec("nonsense==1"))
            .await
            .unwrap_err();
        assert!(e.to_string().contains("unknown field"));
    }
}
