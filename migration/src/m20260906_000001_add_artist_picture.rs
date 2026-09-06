use sea_orm_migration::prelude::*;

/// Adds `artist.picture` — a URL filled in batch from the Rocksky API after
/// library scans (music_player_scanner::update_artist_pictures).
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Artist::Table)
                    .add_column_if_not_exists(ColumnDef::new(Artist::Picture).string().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Artist::Table)
                    .drop_column(Artist::Picture)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Artist {
    Table,
    Picture,
}
