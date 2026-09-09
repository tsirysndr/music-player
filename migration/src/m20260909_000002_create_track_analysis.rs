use sea_orm_migration::prelude::*;

/// What a track sounds like: waveform, loudness, tempo, mood.
///
/// A cache, not a fact about the library. Computing a row means decoding the
/// whole track — seconds of CPU for a local file, and a download first for a
/// remote one — while the answer never changes, because the same audio always
/// analyses the same. Storing it is the difference between a waveform that
/// appears instantly and one that costs a decode every time the player is
/// opened.
///
/// Keyed by track id and kept in its own table rather than added to `track`,
/// for two reasons. The rows have a different lifetime: a rescan rewrites
/// `track` and would throw away analysis that is still perfectly valid. And
/// most tracks have no row at all until something asks — analysis is
/// opportunistic, and a column per feature on `track` would suggest otherwise.
///
/// `source` records which library the id belongs to. Ids are only unique within
/// a provider, so without it a Navidrome track and a local one could collide
/// and a waveform would be drawn for the wrong song.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TrackAnalysis::Table)
                    .if_not_exists()
                    // `md5(source + "\0" + track_id)`, so re-analysing the same
                    // track replaces its row rather than adding one.
                    .col(
                        ColumnDef::new(TrackAnalysis::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TrackAnalysis::TrackId).string().not_null())
                    // Copied from the library at analysis time. Denormalised on
                    // purpose: auto-DJ needs an artist for every candidate to
                    // avoid playing one twice in a row, and looking those up
                    // would be a request per track against a remote server —
                    // thousands of them, to answer a question about a set of
                    // five. A stale name here costs a slightly worse pick.
                    .col(ColumnDef::new(TrackAnalysis::Artist).string())
                    .col(ColumnDef::new(TrackAnalysis::Title).string())
                    // Empty for the daemon's own library.
                    .col(ColumnDef::new(TrackAnalysis::Source).string().not_null())
                    // Peak per bar, one byte each, drawn as-is.
                    .col(
                        ColumnDef::new(TrackAnalysis::Waveform)
                            .blob(sea_orm_migration::sea_query::BlobSize::Blob(None)),
                    )
                    .col(ColumnDef::new(TrackAnalysis::Bpm).float())
                    .col(ColumnDef::new(TrackAnalysis::BpmConfidence).float())
                    .col(ColumnDef::new(TrackAnalysis::Valence).float())
                    .col(ColumnDef::new(TrackAnalysis::Arousal).float())
                    // A json array of [label, confidence] pairs. Read whole and
                    // never queried into, so a table of its own would be three
                    // joins to say what one string says.
                    .col(ColumnDef::new(TrackAnalysis::Moods).string())
                    .col(ColumnDef::new(TrackAnalysis::Lufs).float())
                    .col(ColumnDef::new(TrackAnalysis::TruePeakDb).float())
                    // As decoded, which is not always what the tags claim.
                    .col(ColumnDef::new(TrackAnalysis::Duration).float())
                    .col(
                        ColumnDef::new(TrackAnalysis::AnalyzedAt)
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // The id is derived from these two, so this is what the derivation
        // promises — enforced here rather than trusted to every writer.
        manager
            .create_index(
                Index::create()
                    .name("idx_track_analysis_track")
                    .table(TrackAnalysis::Table)
                    .col(TrackAnalysis::Source)
                    .col(TrackAnalysis::TrackId)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        // Auto-DJ asks "what is near this tempo", over the whole library, every
        // time it tops the queue up. Without an index that is a full scan of
        // every analysed track.
        manager
            .create_index(
                Index::create()
                    .name("idx_track_analysis_bpm")
                    .table(TrackAnalysis::Table)
                    .col(TrackAnalysis::Source)
                    .col(TrackAnalysis::Bpm)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TrackAnalysis::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum TrackAnalysis {
    Table,
    Id,
    TrackId,
    Artist,
    Title,
    Source,
    Waveform,
    Bpm,
    BpmConfidence,
    Valence,
    Arousal,
    Moods,
    Lufs,
    TruePeakDb,
    Duration,
    AnalyzedAt,
}
