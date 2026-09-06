use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SavedRadio::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SavedRadio::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SavedRadio::Name).string().not_null())
                    .col(ColumnDef::new(SavedRadio::StreamUrl).string().not_null())
                    .col(ColumnDef::new(SavedRadio::Source).string().not_null())
                    .col(ColumnDef::new(SavedRadio::Genre).string().not_null())
                    .col(ColumnDef::new(SavedRadio::Country).string().not_null())
                    .col(ColumnDef::new(SavedRadio::Logo).string().not_null())
                    .col(ColumnDef::new(SavedRadio::Bitrate).integer().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SavedRadio::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum SavedRadio {
    Table,
    Id,
    Name,
    StreamUrl,
    Source,
    Genre,
    Country,
    Logo,
    Bitrate,
}
