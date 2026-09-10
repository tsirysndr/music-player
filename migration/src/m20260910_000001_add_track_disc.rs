use sea_orm_migration::prelude::*;

/// Which disc of a set a track sits on.
///
/// The tag has always been read and then thrown away at this boundary, so the
/// local library reported every track as disc zero and a multi-disc album was
/// shown as one long run of tracks — while the same album on a remote server,
/// whose provider does carry the number, grouped correctly. The two now agree.
///
/// Nullable, and left null for a single-disc release: "which disc" is not a
/// question most albums answer, and a default of 1 would be a claim rather than
/// an absence. Existing rows fill in on the next scan.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Track::Table)
                    .add_column(ColumnDef::new(Track::Disc).unsigned())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Track::Table)
                    .drop_column(Track::Disc)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Track {
    Table,
    Disc,
}
