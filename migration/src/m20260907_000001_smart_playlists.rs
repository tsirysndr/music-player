//! Smart playlists, and the play statistics they filter on.
//!
//! A smart playlist is an ordinary playlist that knows how to refill itself:
//! the same row gains an RSQL filter plus a sort and a cap. Keeping it in
//! `playlist` rather than a table of its own means every surface that already
//! lists, opens, plays or deletes a playlist handles smart ones with no extra
//! code — only the refill is new.
//!
//! `track_stats` and `track.created_at` exist for the filters people actually
//! write: "played more than five times", "never played", "added this month".

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for mut column in [
            ColumnDef::new(Playlist::IsSmart)
                .boolean()
                .not_null()
                .default(false)
                .to_owned(),
            ColumnDef::new(Playlist::Rsql).string().to_owned(),
            ColumnDef::new(Playlist::SortBy).string().to_owned(),
            ColumnDef::new(Playlist::SortOrder).string().to_owned(),
            // Not `limit`: a reserved word in SQL, and quoting it everywhere
            // to save four characters is a poor trade.
            ColumnDef::new(Playlist::MaxTracks).integer().to_owned(),
            ColumnDef::new(Playlist::RefreshedAt).date_time().to_owned(),
        ] {
            manager
                .alter_table(
                    Table::alter()
                        .table(Playlist::Table)
                        .add_column_if_not_exists(&mut column)
                        .to_owned(),
                )
                .await?;
        }

        // Nullable: every track already in the library predates this column and
        // has no honest value to give it.
        manager
            .alter_table(
                Table::alter()
                    .table(Track::Table)
                    .add_column_if_not_exists(ColumnDef::new(Track::CreatedAt).date_time())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TrackStats::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TrackStats::TrackId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TrackStats::PlayCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(TrackStats::SkipCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    // Unix seconds rather than a datetime: the RSQL compiler
                    // resolves "in the last 30 days" to a plain integer, and
                    // comparing that against a text date would not work.
                    .col(ColumnDef::new(TrackStats::LastPlayed).big_integer())
                    .col(ColumnDef::new(TrackStats::LastSkipped).big_integer())
                    .col(
                        ColumnDef::new(TrackStats::UpdatedAt)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_track_stats_play_count")
                    .table(TrackStats::Table)
                    .col(TrackStats::PlayCount)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TrackStats::Table).to_owned())
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Track::Table)
                    .drop_column(Track::CreatedAt)
                    .to_owned(),
            )
            .await?;
        for column in [
            Playlist::IsSmart,
            Playlist::Rsql,
            Playlist::SortBy,
            Playlist::SortOrder,
            Playlist::MaxTracks,
            Playlist::RefreshedAt,
        ] {
            manager
                .alter_table(
                    Table::alter()
                        .table(Playlist::Table)
                        .drop_column(column)
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }
}

#[derive(Iden)]
enum Playlist {
    Table,
    IsSmart,
    Rsql,
    SortBy,
    SortOrder,
    MaxTracks,
    RefreshedAt,
}

#[derive(Iden)]
enum Track {
    Table,
    CreatedAt,
}

#[derive(Iden)]
enum TrackStats {
    Table,
    TrackId,
    PlayCount,
    SkipCount,
    LastPlayed,
    LastSkipped,
    UpdatedAt,
}
