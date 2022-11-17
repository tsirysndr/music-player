use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Folder::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Folder::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Folder::Name).string().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Playlist::Table)
                    .add_column_if_not_exists(ColumnDef::new(Playlist::Description).string())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Folder::Table).if_exists().to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Playlist {
    Table,
    Description,
}

#[derive(Iden)]
enum Folder {
    Table,
    Id,
    Name,
}
