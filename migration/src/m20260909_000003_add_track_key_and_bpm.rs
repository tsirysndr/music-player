use sea_orm_migration::prelude::*;

/// A track's musical key and tempo, on the track itself.
///
/// Both are already measured by the analysis cache, so this is a second home
/// for two of its fields — deliberately. `track_analysis` is keyed by library
/// and holds a waveform and a mood: it answers "how does this sound" for the
/// player. These two columns answer a different question, "what does this row
/// say", for every listing that shows a table of tracks. Putting them here
/// means a track list is one query rather than a join per page, and means the
/// scanner can fill them in as part of indexing the library.
///
/// Only for the local library. A remote provider's tracks are not rows in this
/// table, which is why the clients only offer these columns when reading from
/// a music-player daemon.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Track::Table)
                    // Camelot notation, e.g. "8A". Null means not analysed —
                    // which is different from a track that has no clear key,
                    // and the clients show nothing at all for it either way.
                    .add_column(ColumnDef::new(Track::Key).string())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Track::Table)
                    .add_column(ColumnDef::new(Track::Bpm).float())
                    .to_owned(),
            )
            .await?;

        // The analysis cache learns the key at the same time it learns the
        // tempo, so it keeps it too — along with a confidence the track row has
        // no room for. The two are not duplicates: this is the measurement,
        // the track column is the copy a listing reads.
        manager
            .alter_table(
                Table::alter()
                    .table(TrackAnalysis::Table)
                    .add_column(ColumnDef::new(TrackAnalysis::Key).string())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(TrackAnalysis::Table)
                    .add_column(ColumnDef::new(TrackAnalysis::KeyConfidence).float())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Track::Table)
                    .drop_column(Track::Key)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Track::Table)
                    .drop_column(Track::Bpm)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Track {
    Table,
    Key,
    Bpm,
}

#[derive(Iden)]
enum TrackAnalysis {
    Table,
    Key,
    KeyConfidence,
}
