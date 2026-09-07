use sea_orm_migration::prelude::*;

/// Remote music servers the user has saved.
///
/// One table in the daemon, replacing three stores that could not see each
/// other: the Slint desktop's `desktop_servers.json`, the single
/// `subsonic_*`/`jellyfin_*` pair in `settings.toml`, and the web client's
/// in-memory connection. Both of the former are imported once on first run.
///
/// This is not the `addon` table. That one describes the built-in playback
/// addons named in `settings.toml` and carries entirely different columns; a
/// saved server is a place to *read a library from*, which is a different
/// question from where the audio comes out.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SavedServer::Table)
                    .if_not_exists()
                    // `md5(kind + "\0" + url)` — deterministic, so re-adding a
                    // server updates its row instead of duplicating it.
                    .col(
                        ColumnDef::new(SavedServer::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    // The source-registry key. A string rather than an enum so
                    // a new backend needs no migration.
                    .col(ColumnDef::new(SavedServer::Kind).string().not_null())
                    .col(ColumnDef::new(SavedServer::Name).string().not_null())
                    .col(ColumnDef::new(SavedServer::Url).string().not_null())
                    .col(ColumnDef::new(SavedServer::Username).string().null())
                    .col(ColumnDef::new(SavedServer::Password).string().null())
                    .col(ColumnDef::new(SavedServer::CreatedAt).string().null())
                    .col(ColumnDef::new(SavedServer::UpdatedAt).string().null())
                    .to_owned(),
            )
            .await?;

        // Belt and braces with the derived primary key: two rows for the same
        // server would show up twice in every client's list.
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_saved_server_kind_url")
                    .table(SavedServer::Table)
                    .col(SavedServer::Kind)
                    .col(SavedServer::Url)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SavedServer::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum SavedServer {
    Table,
    Id,
    Kind,
    Name,
    Url,
    Username,
    Password,
    CreatedAt,
    UpdatedAt,
}
