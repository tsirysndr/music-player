use sea_orm_migration::prelude::*;

/// Genres, and what belongs to them.
///
/// `track.genre` is already a free-text column filled from the file's tag, but
/// it cannot answer "what genres are in this library" without a scan-and-split
/// on every read, and it holds duplicates that differ only in case or spacing.
/// A table of distinct genres, keyed by a normalised name, does.
///
/// Two link tables rather than one, because the two sources disagree in useful
/// ways:
///
/// - `track_genres` comes from the file's own tag. Precise when present, and
///   often absent or wrong.
/// - `artist_genres` comes from the Rocksky enrichment the scanner already
///   runs for artist pictures. Much better coverage — most files ship no
///   usable genre tag at all — but it describes the artist, not the track.
///
/// Keeping both lets a genre listing use whichever it has, rather than
/// choosing one source and being wrong for half the library.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Genre::Table)
                    .if_not_exists()
                    // `md5(lowercased, trimmed name)`: deterministic, so the
                    // same genre found twice is the same row rather than a
                    // duplicate, and a rescan is idempotent.
                    .col(ColumnDef::new(Genre::Id).string().not_null().primary_key())
                    // As first seen, for display.
                    .col(ColumnDef::new(Genre::Name).string().not_null())
                    .to_owned(),
            )
            .await?;

        // The name is what dedupes, so the database enforces it rather than
        // trusting every writer to check first.
        manager
            .create_index(
                Index::create()
                    .name("idx_genre_name")
                    .table(Genre::Table)
                    .col(Genre::Name)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TrackGenres::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TrackGenres::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TrackGenres::TrackId).string().not_null())
                    .col(ColumnDef::new(TrackGenres::GenreId).string().not_null())
                    .to_owned(),
            )
            .await?;

        // One row per pair. The id is derived from both, so re-linking is a
        // no-op instead of a second row.
        manager
            .create_index(
                Index::create()
                    .name("idx_track_genres_pair")
                    .table(TrackGenres::Table)
                    .col(TrackGenres::TrackId)
                    .col(TrackGenres::GenreId)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_track_genres_genre")
                    .table(TrackGenres::Table)
                    .col(TrackGenres::GenreId)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ArtistGenres::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ArtistGenres::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ArtistGenres::ArtistId).string().not_null())
                    .col(ColumnDef::new(ArtistGenres::GenreId).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_artist_genres_pair")
                    .table(ArtistGenres::Table)
                    .col(ArtistGenres::ArtistId)
                    .col(ArtistGenres::GenreId)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_artist_genres_genre")
                    .table(ArtistGenres::Table)
                    .col(ArtistGenres::GenreId)
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ArtistGenres::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TrackGenres::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Genre::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Genre {
    Table,
    Id,
    Name,
}

#[derive(Iden)]
enum TrackGenres {
    Table,
    Id,
    TrackId,
    GenreId,
}

#[derive(Iden)]
enum ArtistGenres {
    Table,
    Id,
    ArtistId,
    GenreId,
}
