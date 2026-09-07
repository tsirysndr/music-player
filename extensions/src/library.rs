//! The real library, behind the extension host's [`LibraryAccess`] trait.
//!
//! Extensions filter with RSQL — the same language smart playlists use — so an
//! author learns one vocabulary and the host reuses one compiler. Every filter
//! is compiled against the schema for the table being read, which is what keeps
//! a guest from reaching columns (or tables) it was not offered.
//!
//! Calls arrive from WebAssembly, which is synchronous, while sea-orm is async.
//! Each query is therefore handed to the runtime this was built on rather than
//! blocking whatever thread the guest happens to be running on.

use anyhow::{anyhow, Error};
use music_player_entity::{album, artist, playlist, playlist_tracks, saved_radio, track};
use music_player_rsql::{build_at, QuerySpec, Value, ALBUMS, ARTISTS, PLAYLISTS, TRACKS};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Statement,
};

use crate::abi::{LibraryAlbum, LibraryArtist, LibraryPlaylist, SavedRadio, TrackInfo};
use crate::host::LibraryAccess;

/// Reads the user's library for extensions.
pub struct Library {
    conn: DatabaseConnection,
    /// The runtime the async queries are driven on. Held rather than created
    /// per call: building a runtime per query would be both slow and a good way
    /// to run out of file descriptors.
    handle: tokio::runtime::Handle,
}

impl Library {
    /// Wrap a connection, driving queries on the current runtime.
    ///
    /// Must be called from inside a Tokio runtime — the daemon and the desktop
    /// app both are.
    pub fn new(conn: DatabaseConnection) -> Result<Self, Error> {
        let handle = tokio::runtime::Handle::try_current()
            .map_err(|_| anyhow!("the extension library needs a Tokio runtime"))?;
        Ok(Self { conn, handle })
    }

    /// Run an async query from a synchronous guest call.
    fn block_on<F: std::future::Future>(&self, future: F) -> F::Output {
        // `block_in_place` moves this off the async worker so the runtime keeps
        // serving everything else while the guest waits.
        tokio::task::block_in_place(|| self.handle.block_on(future))
    }

    /// Compile an RSQL filter into the ids it selects.
    async fn ids(
        &self,
        filter: &str,
        limit: u32,
        schema: &music_player_rsql::Schema,
    ) -> Result<Vec<String>, Error> {
        let spec = QuerySpec {
            filter: filter.to_owned(),
            limit: (limit > 0).then_some(limit),
            ..Default::default()
        };
        let query = build_at(&spec, schema, unix_now()).map_err(|e| anyhow!("{e}"))?;
        let params: Vec<sea_orm::Value> = query
            .params
            .into_iter()
            .map(|value| match value {
                Value::Text(text) => text.into(),
                Value::Integer(number) => number.into(),
            })
            .collect();
        let rows = self
            .conn
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

    /// Reorder `rows` to match `ids` — `is_in` returns table order, but the
    /// filter's order is what the caller asked for.
    fn in_filter_order<T, K: Fn(&T) -> &str>(mut rows: Vec<T>, ids: &[String], key: K) -> Vec<T> {
        rows.sort_by_key(|row| {
            ids.iter()
                .position(|id| id == key(row))
                .unwrap_or(usize::MAX)
        });
        rows
    }
}

impl LibraryAccess for Library {
    fn query(&self, filter: &str, limit: u32) -> Result<Vec<TrackInfo>, Error> {
        self.block_on(async {
            let ids = self.ids(filter, limit, &TRACKS).await?;
            if ids.is_empty() {
                return Ok(Vec::new());
            }
            let rows = track::Entity::find()
                .filter(track::Column::Id.is_in(ids.clone()))
                .all(&self.conn)
                .await?;
            Ok(Self::in_filter_order(rows, &ids, |row| &row.id)
                .iter()
                .map(TrackInfo::from)
                .collect())
        })
    }

    fn albums(&self, filter: &str, limit: u32) -> Result<Vec<LibraryAlbum>, Error> {
        self.block_on(async {
            let ids = self.ids(filter, limit, &ALBUMS).await?;
            if ids.is_empty() {
                return Ok(Vec::new());
            }
            let rows = album::Entity::find()
                .filter(album::Column::Id.is_in(ids.clone()))
                .all(&self.conn)
                .await?;
            Ok(Self::in_filter_order(rows, &ids, |row| &row.id)
                .iter()
                .map(LibraryAlbum::from)
                .collect())
        })
    }

    fn artists(&self, filter: &str, limit: u32) -> Result<Vec<LibraryArtist>, Error> {
        self.block_on(async {
            let ids = self.ids(filter, limit, &ARTISTS).await?;
            if ids.is_empty() {
                return Ok(Vec::new());
            }
            let rows = artist::Entity::find()
                .filter(artist::Column::Id.is_in(ids.clone()))
                .all(&self.conn)
                .await?;
            Ok(Self::in_filter_order(rows, &ids, |row| &row.id)
                .iter()
                .map(LibraryArtist::from)
                .collect())
        })
    }

    fn playlists(&self, filter: &str, limit: u32) -> Result<Vec<LibraryPlaylist>, Error> {
        self.block_on(async {
            let ids = self.ids(filter, limit, &PLAYLISTS).await?;
            if ids.is_empty() {
                return Ok(Vec::new());
            }
            let rows = playlist::Entity::find()
                .filter(playlist::Column::Id.is_in(ids.clone()))
                .all(&self.conn)
                .await?;
            let ordered = Self::in_filter_order(rows, &ids, |row| &row.id);
            let mut out = Vec::with_capacity(ordered.len());
            for row in &ordered {
                // The row itself carries no track count; counting is one cheap
                // query per playlist and the list is short.
                let count = playlist_tracks::Entity::find()
                    .filter(playlist_tracks::Column::PlaylistId.eq(row.id.clone()))
                    .count(&self.conn)
                    .await
                    .unwrap_or_default();
                let mut playlist = LibraryPlaylist::from(row);
                playlist.track_count = count as u32;
                out.push(playlist);
            }
            Ok(out)
        })
    }

    fn playlist_tracks(&self, playlist_id: &str) -> Result<Vec<TrackInfo>, Error> {
        self.block_on(async {
            let entries = playlist_tracks::Entity::find()
                .filter(playlist_tracks::Column::PlaylistId.eq(playlist_id.to_owned()))
                .order_by_asc(playlist_tracks::Column::CreatedAt)
                .all(&self.conn)
                .await?;
            let ids: Vec<String> = entries.iter().map(|row| row.track_id.clone()).collect();
            if ids.is_empty() {
                return Ok(Vec::new());
            }
            let rows = track::Entity::find()
                .filter(track::Column::Id.is_in(ids.clone()))
                .all(&self.conn)
                .await?;
            Ok(Self::in_filter_order(rows, &ids, |row| &row.id)
                .iter()
                .map(TrackInfo::from)
                .collect())
        })
    }

    fn saved_radios(&self) -> Result<Vec<SavedRadio>, Error> {
        self.block_on(async {
            let rows = saved_radio::Entity::find().all(&self.conn).await?;
            Ok(rows.iter().map(SavedRadio::from).collect())
        })
    }
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
    use sea_orm::{ActiveModelTrait, ActiveValue, Database};

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
            id: ActiveValue::Set("al-1".into()),
            title: ActiveValue::Set("OK Computer".into()),
            artist: ActiveValue::Set("Radiohead".into()),
            year: ActiveValue::Set(Some(1997)),
            ..Default::default()
        }
        .insert(&conn)
        .await
        .unwrap();
        artist::ActiveModel {
            id: ActiveValue::Set("ar-1".into()),
            name: ActiveValue::Set("Radiohead".into()),
            ..Default::default()
        }
        .insert(&conn)
        .await
        .unwrap();
        for (id, title, genre) in [("t1", "Airbag", "rock"), ("t2", "So What", "jazz")] {
            track::ActiveModel {
                id: ActiveValue::Set(id.into()),
                title: ActiveValue::Set(title.into()),
                artist: ActiveValue::Set("Radiohead".into()),
                genre: ActiveValue::Set(genre.into()),
                uri: ActiveValue::Set(format!("/music/{id}.mp3")),
                album_id: ActiveValue::Set(Some("al-1".into())),
                ..Default::default()
            }
            .insert(&conn)
            .await
            .unwrap();
        }
        playlist::ActiveModel {
            id: ActiveValue::Set("pl-1".into()),
            name: ActiveValue::Set("Mix".into()),
            created_at: ActiveValue::Set(chrono::Utc::now()),
            ..Default::default()
        }
        .insert(&conn)
        .await
        .unwrap();
        playlist_tracks::ActiveModel {
            id: ActiveValue::Set("pt-1".into()),
            playlist_id: ActiveValue::Set("pl-1".into()),
            track_id: ActiveValue::Set("t1".into()),
            created_at: ActiveValue::Set(chrono::Utc::now()),
        }
        .insert(&conn)
        .await
        .unwrap();
        saved_radio::ActiveModel {
            id: ActiveValue::Set("rb:1".into()),
            name: ActiveValue::Set("Jazz FM".into()),
            stream_url: ActiveValue::Set("https://example/stream".into()),
            source: ActiveValue::Set("Radio Browser".into()),
            genre: ActiveValue::Set("jazz".into()),
            country: ActiveValue::Set("".into()),
            logo: ActiveValue::Set("".into()),
            bitrate: ActiveValue::Set(128),
        }
        .insert(&conn)
        .await
        .unwrap();
        (dir, conn)
    }

    /// `block_in_place` needs the multi-thread runtime, which is what the
    /// daemon and desktop app both use.
    #[tokio::test(flavor = "multi_thread")]
    async fn reads_every_part_of_the_library() {
        let (_dir, conn) = library().await;
        let library = Library::new(conn).unwrap();

        assert_eq!(library.query("genre==rock", 0).unwrap().len(), 1);
        assert_eq!(library.query("", 0).unwrap().len(), 2);
        assert_eq!(
            library.albums("year>1990", 0).unwrap()[0].title,
            "OK Computer"
        );
        assert_eq!(library.artists("name==Radiohead", 0).unwrap().len(), 1);

        let playlists = library.playlists("", 0).unwrap();
        assert_eq!(playlists.len(), 1);
        // The count comes from the join table, not from the playlist row.
        assert_eq!(playlists[0].track_count, 1);

        let tracks = library.playlist_tracks("pl-1").unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].title, "Airbag");

        let radios = library.saved_radios().unwrap();
        assert_eq!(radios.len(), 1);
        assert_eq!(radios[0].name, "Jazz FM");
    }

    /// Each table is filtered against its own schema, so a track field is not
    /// silently accepted on an album query.
    #[tokio::test(flavor = "multi_thread")]
    async fn a_filter_is_checked_against_the_right_schema() {
        let (_dir, conn) = library().await;
        let library = Library::new(conn).unwrap();

        let e = library.albums("playcount>1", 0).unwrap_err();
        assert!(e.to_string().contains("unknown field"));
        // And the same filter is fine on tracks.
        assert!(library.query("playcount>1", 0).is_ok());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_limit_caps_the_result() {
        let (_dir, conn) = library().await;
        let library = Library::new(conn).unwrap();
        assert_eq!(library.query("", 1).unwrap().len(), 1);
    }
}
