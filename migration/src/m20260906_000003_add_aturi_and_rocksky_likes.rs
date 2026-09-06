use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, Statement};

/// Links the local library to the user's atproto repo.
///
/// * `aturi` on `track` / `album` / `artist` — the `at://` uri of the matching
///   `app.rocksky.song` / `.album` / `.artist` record, filled in when a like is
///   matched to a local file.
/// * `rocksky_like` — the likes extracted from the user's repo CAR, kept as
///   their own table so a like survives a library rescan and can be re-matched
///   later (a liked song may only be added to the library afterwards).
/// * A case-insensitive index over `track (title, artist)` so matching a few
///   thousand likes against the library does not turn into a table scan each.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Track::Table)
                    .add_column_if_not_exists(ColumnDef::new(Track::Aturi).string().null())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Album::Table)
                    .add_column_if_not_exists(ColumnDef::new(Album::Aturi).string().null())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Artist::Table)
                    .add_column_if_not_exists(ColumnDef::new(Artist::Aturi).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RockskyLike::Table)
                    .if_not_exists()
                    // The like record's own at:// uri.
                    .col(
                        ColumnDef::new(RockskyLike::Uri)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    // at:// uri of the app.rocksky.song it points at.
                    .col(ColumnDef::new(RockskyLike::SongUri).string().not_null())
                    .col(ColumnDef::new(RockskyLike::Title).string().not_null())
                    .col(ColumnDef::new(RockskyLike::Artist).string().not_null())
                    .col(ColumnDef::new(RockskyLike::Album).string().not_null())
                    .col(ColumnDef::new(RockskyLike::AlbumArtist).string().not_null())
                    .col(ColumnDef::new(RockskyLike::CreatedAt).string().not_null())
                    // Local track this like resolved to, when the song is in
                    // the library. Null while unmatched.
                    .col(ColumnDef::new(RockskyLike::TrackId).string().null())
                    .to_owned(),
            )
            .await?;

        // Expression indexes are not expressible through the schema builder.
        let db = manager.get_connection();
        for statement in [
            "CREATE INDEX IF NOT EXISTS idx_track_title_artist_nocase
                ON track (LOWER(title), LOWER(artist))",
            "CREATE INDEX IF NOT EXISTS idx_album_title_nocase ON album (LOWER(title))",
            "CREATE INDEX IF NOT EXISTS idx_track_aturi ON track (aturi)",
            "CREATE INDEX IF NOT EXISTS idx_rocksky_like_song_uri ON rocksky_like (song_uri)",
            "CREATE INDEX IF NOT EXISTS idx_rocksky_like_track_id ON rocksky_like (track_id)",
            "CREATE INDEX IF NOT EXISTS idx_rocksky_like_match
                ON rocksky_like (LOWER(title), LOWER(artist), LOWER(album))",
        ] {
            db.execute(Statement::from_string(
                manager.get_database_backend(),
                statement.to_owned(),
            ))
            .await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        for statement in [
            "DROP INDEX IF EXISTS idx_track_title_artist_nocase",
            "DROP INDEX IF EXISTS idx_album_title_nocase",
            "DROP INDEX IF EXISTS idx_track_aturi",
            "DROP INDEX IF EXISTS idx_rocksky_like_song_uri",
            "DROP INDEX IF EXISTS idx_rocksky_like_track_id",
            "DROP INDEX IF EXISTS idx_rocksky_like_match",
        ] {
            db.execute(Statement::from_string(
                manager.get_database_backend(),
                statement.to_owned(),
            ))
            .await?;
        }
        manager
            .drop_table(Table::drop().table(RockskyLike::Table).to_owned())
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Track::Table)
                    .drop_column(Track::Aturi)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Album::Table)
                    .drop_column(Album::Aturi)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Artist::Table)
                    .drop_column(Artist::Aturi)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Track {
    Table,
    Aturi,
}

#[derive(Iden)]
enum Album {
    Table,
    Aturi,
}

#[derive(Iden)]
enum Artist {
    Table,
    Aturi,
}

#[derive(Iden)]
enum RockskyLike {
    Table,
    Uri,
    SongUri,
    Title,
    Artist,
    Album,
    AlbumArtist,
    CreatedAt,
    TrackId,
}
