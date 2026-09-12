use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::ConnectionTrait;

/// Throw away key and tempo that were guessed before the file was asked.
///
/// The first analysis build detected both from the audio without reading the
/// file's own tags. Correlation-based key detection gets the tonic right and
/// the mode wrong often enough to matter, and its tempo lands on half or double
/// the real one — measured against tagged files, roughly a third of the rows
/// disagreed with what the file plainly said, and every one of those was a
/// guess.
///
/// Those rows are identifiable exactly: that build wrote Camelot (`7B`, `8A`)
/// and every build since writes traditional notation (`F`, `Am`), so a Camelot
/// value in this column is a guess from before tags were read. Nothing written
/// since can match, which is what makes this safe to run over a whole library.
///
/// The analysis rows go too, not just the columns: the pass that refills them
/// looks for tracks with *no* analysis row, so leaving them would mean the same
/// wrong answers being copied back. Their waveforms are recomputed with them,
/// which is one decode for a correct result.
#[derive(DeriveMigrationName)]
pub struct Migration;

/// Camelot is one or two digits and an A or a B, and nothing else is.
const IS_CAMELOT: &str = r#"key GLOB '[0-9][AB]' OR key GLOB '[0-9][0-9][AB]'"#;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();
        let run = |sql: String| {
            db.execute(sea_orm_migration::sea_orm::Statement::from_string(
                backend, sql,
            ))
        };

        run(format!(
            "DELETE FROM track_analysis WHERE track_id IN \
             (SELECT id FROM track WHERE {IS_CAMELOT})"
        ))
        .await?;

        run(format!(
            "UPDATE track SET key = NULL, bpm = NULL WHERE {IS_CAMELOT}"
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Nothing to restore: what this removed was wrong, and the values that
        // replace it are recomputed from the files.
        Ok(())
    }
}
