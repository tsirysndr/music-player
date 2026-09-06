use sea_orm_migration::prelude::*;

/// Remembers when each account's repo CAR archive was last downloaded.
///
/// The archive is the whole repo in one request, so it is only worth pulling
/// again once it has aged out — this row is what survives a restart and keeps
/// the daemon from re-downloading it on every boot.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AtprotoRepoSync::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtprotoRepoSync::Did)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    // RFC 3339, so the value is readable in the database too.
                    .col(
                        ColumnDef::new(AtprotoRepoSync::LastDownloadedAt)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtprotoRepoSync::Bytes)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtprotoRepoSync::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum AtprotoRepoSync {
    Table,
    Did,
    LastDownloadedAt,
    Bytes,
}
