use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Artist::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Artist::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Artist::Name).string().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Album::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Album::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Album::Title).string().not_null())
                    .col(ColumnDef::new(Album::Artist).string().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Track::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Track::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Track::Title).string().not_null())
                    .col(ColumnDef::new(Track::Artist).string().not_null())
                    .col(ColumnDef::new(Track::Album).string().not_null())
                    .col(ColumnDef::new(Track::Genre).string().not_null())
                    .col(ColumnDef::new(Track::Year).integer())
                    .col(ColumnDef::new(Track::Track).integer())
                    .col(ColumnDef::new(Track::Bitrate).integer())
                    .col(ColumnDef::new(Track::SampleRate).integer())
                    .col(ColumnDef::new(Track::BitDepth).integer())
                    .col(ColumnDef::new(Track::Channels).integer())
                    .col(ColumnDef::new(Track::Duration).integer())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Artist::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Album::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Track::Table).if_exists().to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Album {
    Table,
    Id,
    Title,
    Artist,
}

#[derive(Iden)]
enum Artist {
    Table,
    Id,
    Name,
}

#[derive(Iden)]
enum Track {
    Table,
    Id,
    Title,
    Artist,
    Album,
    Genre,
    Year,
    Track,
    Bitrate,
    SampleRate,
    BitDepth,
    Channels,
    Duration,
}
