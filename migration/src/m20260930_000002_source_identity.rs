use sea_orm_migration::prelude::*;

/// Existing saved IDs and credentials stay intact; old accounts start unbound.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite requires a separate ALTER TABLE for each added column.
        manager
            .alter_table(
                Table::alter()
                    .table(SavedServer::Table)
                    .add_column(ColumnDef::new(SavedServer::RemoteServerId).string().null())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(SavedServer::Table)
                    .add_column(ColumnDef::new(SavedServer::RemoteUserId).string().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(SavedServer::Table)
                    .drop_column(SavedServer::RemoteUserId)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(SavedServer::Table)
                    .drop_column(SavedServer::RemoteServerId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum SavedServer {
    Table,
    RemoteServerId,
    RemoteUserId,
}
