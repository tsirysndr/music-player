use sea_orm_migration::prelude::*;

/// Which extensions the user has switched off.
///
/// Only the flag lives here: everything else about an extension — name,
/// version, capabilities, permissions — is read from its manifest on disk,
/// which is the thing that can change under the daemon between runs. A copy in
/// the database would be a second source of truth that goes stale the moment
/// an extension is upgraded.
///
/// The `addon` table is deliberately left alone. It describes the built-in
/// playback addons (Subsonic, Jellyfin, Chromecast, DLNA) named in
/// `settings.toml`, which are a different thing from a WebAssembly extension
/// and carry entirely different columns.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Extension::Table)
                    .if_not_exists()
                    // The manifest id, e.g. `com.example.lyrics`.
                    .col(
                        ColumnDef::new(Extension::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    // An extension the user has never touched has no row at
                    // all, so the default matches what the registry assumes
                    // for one that was simply dropped into the directory.
                    .col(
                        ColumnDef::new(Extension::Enabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(ColumnDef::new(Extension::UpdatedAt).string().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Extension::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Extension {
    Table,
    Id,
    Enabled,
    UpdatedAt,
}
