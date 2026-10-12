use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ExtensionSourceAccount::Table)
                    .col(
                        ColumnDef::new(ExtensionSourceAccount::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ExtensionSourceAccount::ExtensionId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExtensionSourceAccount::Name)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExtensionSourceAccount::SourceApiVersion)
                            .integer()
                            .not_null()
                            .default(2),
                    )
                    .col(
                        ColumnDef::new(ExtensionSourceAccount::ConfigJson)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExtensionSourceAccount::ConfigRevision)
                            .big_integer()
                            .not_null()
                            .default(1),
                    )
                    .col(
                        ColumnDef::new(ExtensionSourceAccount::RemoteServerId)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ExtensionSourceAccount::RemoteUserId)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ExtensionSourceAccount::CreatedAt)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ExtensionSourceAccount::LocalBoundRevision)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ExtensionSourceAccount::LocalConfigJson)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ExtensionSourceAccount::UpdatedAt)
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(ExtensionSourceAccount::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum ExtensionSourceAccount {
    Table,
    Id,
    ExtensionId,
    SourceApiVersion,
    Name,
    ConfigJson,
    ConfigRevision,
    RemoteServerId,
    RemoteUserId,
    LocalBoundRevision,
    LocalConfigJson,
    CreatedAt,
    UpdatedAt,
}
