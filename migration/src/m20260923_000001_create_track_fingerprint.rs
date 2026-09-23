use sea_orm_migration::prelude::*;

/// What a track *is*: its Chromaprint fingerprint, and the AcoustID and
/// MusicBrainz identity that fingerprint resolves to.
///
/// Separate from `track_analysis` because the two answer different questions
/// and are produced at different costs. Analysis measures a recording — how
/// loud, how fast, what key — and every field of it is a number about the
/// audio. A fingerprint is an identity: it is what lets a file tagged
/// "track03.mp3" be recognised as a specific recording, and it is the only
/// thing here that is worth sending to a server.
///
/// Separate from `track` for the same reason analysis is: a rescan rewrites
/// `track` rows, and a fingerprint that cost a decode to produce must survive
/// that. Nothing about the audio changed.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TrackFingerprint::Table)
                    .if_not_exists()
                    // The track id itself. Fingerprinting needs the audio, and
                    // the only library whose audio is on this machine is the
                    // local one — so unlike `track_analysis` there is no
                    // second library whose ids could collide with these.
                    .col(
                        ColumnDef::new(TrackFingerprint::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TrackFingerprint::Fingerprint)
                            .string()
                            .not_null(),
                    )
                    // Seconds, whole. Sent to AcoustID alongside the
                    // fingerprint, which matches on both.
                    .col(
                        ColumnDef::new(TrackFingerprint::Duration)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TrackFingerprint::Acoustid).string())
                    .col(ColumnDef::new(TrackFingerprint::RecordingMbid).string())
                    .col(ColumnDef::new(TrackFingerprint::ReleaseMbid).string())
                    .col(ColumnDef::new(TrackFingerprint::Score).float())
                    // Null means never asked. A time with no `acoustid` is a
                    // miss that has been recorded as one, so the next scan does
                    // not ask again about a track the database has never heard
                    // of.
                    .col(ColumnDef::new(TrackFingerprint::LookedUpAt).string())
                    .col(
                        ColumnDef::new(TrackFingerprint::FingerprintedAt)
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Two files of the same recording share a fingerprint prefix but not
        // the whole string, so this does not find duplicates on its own. What
        // it does answer cheaply is "which tracks are this MusicBrainz
        // recording" — the question every "I already have this" check asks.
        manager
            .create_index(
                Index::create()
                    .name("idx_track_fingerprint_recording")
                    .table(TrackFingerprint::Table)
                    .col(TrackFingerprint::RecordingMbid)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TrackFingerprint::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum TrackFingerprint {
    Table,
    Id,
    Fingerprint,
    Duration,
    Acoustid,
    RecordingMbid,
    ReleaseMbid,
    Score,
    LookedUpAt,
    FingerprintedAt,
}
