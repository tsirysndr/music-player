use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Playlist::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Playlist::CreatedAt).date_time().not_null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Folder::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Folder::CreatedAt).date_time().not_null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(PlaylistTrack::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(PlaylistTrack::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        todo!();
    }
}

#[derive(Iden)]
enum Playlist {
    Table,
    CreatedAt,
}

#[derive(Iden)]
enum Folder {
    Table,
    CreatedAt,
}

#[derive(Iden)]
enum PlaylistTrack {
    Table,
    CreatedAt,
}
