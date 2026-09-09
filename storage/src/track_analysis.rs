//! Stored track analysis, and the work of producing it.
//!
//! Analysing a track means decoding all of it — seconds of CPU for a local
//! file, and a download first for a remote one. The answer never changes,
//! because the same audio always analyses the same, so it is computed once and
//! kept. Everything else here follows from that: rows are looked up by a
//! derived id rather than searched for, work is skipped when a row exists, and
//! one analysis runs at a time.

use anyhow::{Error, Result};
use music_player_analysis::Analysis;
use music_player_entity::track_analysis::{self, id_for};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};

/// Enough of a track to analyse it and to remember what it was.
///
/// The artist and title are copied into the stored row rather than looked up
/// later: matching needs an artist for every candidate, and fetching those from
/// a remote library would be a request per track to answer a question about a
/// handful of them.
#[derive(Clone, Debug, Default)]
pub struct TrackRef {
    pub id: String,
    pub uri: String,
    pub artist: String,
    pub title: String,
}

/// The largest track worth downloading to analyse.
///
/// A guard against a mis-tagged uri that turns out to be a video, or a live
/// stream that slipped past the finite check: without a limit, "analyse this"
/// could mean an unbounded download.
const MAX_FETCH_BYTES: u64 = 120 * 1024 * 1024;

/// One analysis at a time, across the process.
///
/// Analysis is a whole core of decoding plus, for a remote track, a download
/// competing with the stream that is *playing*. Running several would make
/// listening worse to fill a table faster, which is the wrong trade — nothing
/// here is urgent.
fn slot() -> &'static tokio::sync::Semaphore {
    static SLOT: std::sync::OnceLock<tokio::sync::Semaphore> = std::sync::OnceLock::new();
    SLOT.get_or_init(|| tokio::sync::Semaphore::new(1))
}

/// The stored analysis for a track, if it has been analysed.
pub async fn get(db: &DatabaseConnection, source: &str, track_id: &str) -> Option<Analysis> {
    let row = track_analysis::Entity::find_by_id(id_for(source, track_id))
        .one(db)
        .await
        .ok()??;
    Some(from_row(row))
}

/// Store an analysis, replacing any earlier one for the same track.
pub async fn put(
    db: &DatabaseConnection,
    source: &str,
    track: &TrackRef,
    analysis: &Analysis,
) -> Result<()> {
    let row = track_analysis::ActiveModel {
        id: Set(id_for(source, &track.id)),
        track_id: Set(track.id.clone()),
        artist: Set(Some(track.artist.clone())),
        title: Set(Some(track.title.clone())),
        source: Set(source.to_string()),
        waveform: Set(Some(analysis.waveform.clone())),
        bpm: Set(analysis.bpm),
        bpm_confidence: Set(analysis.bpm_confidence),
        key: Set(analysis.key.clone()),
        key_confidence: Set(analysis.key_confidence),
        valence: Set(analysis.valence),
        arousal: Set(analysis.arousal),
        moods: Set(serde_json::to_string(&analysis.moods).ok()),
        lufs: Set(analysis.lufs),
        true_peak_db: Set(analysis.true_peak_db),
        duration: Set(Some(analysis.duration)),
        analyzed_at: Set(chrono::Utc::now().to_rfc3339()),
    };

    // Replace rather than insert: re-analysing is how a bad result is
    // corrected, and it must not need a delete first.
    track_analysis::Entity::insert(row)
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(track_analysis::Column::Id)
                .update_columns([
                    track_analysis::Column::Artist,
                    track_analysis::Column::Title,
                    track_analysis::Column::Waveform,
                    track_analysis::Column::Bpm,
                    track_analysis::Column::BpmConfidence,
                    track_analysis::Column::Key,
                    track_analysis::Column::KeyConfidence,
                    track_analysis::Column::Valence,
                    track_analysis::Column::Arousal,
                    track_analysis::Column::Moods,
                    track_analysis::Column::Lufs,
                    track_analysis::Column::TruePeakDb,
                    track_analysis::Column::Duration,
                    track_analysis::Column::AnalyzedAt,
                ])
                .to_owned(),
        )
        .exec(db)
        .await?;
    Ok(())
}

/// Analyse a track, unless it already has been.
///
/// Returns the analysis either way, so a caller does not have to ask twice.
/// The stored-row check happens again *after* the queue for the work slot: by
/// the time a queued request is reached, the track it wanted may already have
/// been analysed by whatever was ahead of it.
pub async fn ensure(db: &DatabaseConnection, source: &str, track: &TrackRef) -> Result<Analysis> {
    if let Some(analysis) = get(db, source, &track.id).await {
        return Ok(analysis);
    }

    let _permit = slot().acquire().await?;
    if let Some(analysis) = get(db, source, &track.id).await {
        return Ok(analysis);
    }

    let bytes = fetch(&track.uri).await?;
    let hint = extension_of(&track.uri);
    // Decoding is CPU-bound and takes seconds. On the async runtime it would
    // stall every other task on that thread, including playback's own timer.
    let analysis = tokio::task::spawn_blocking(move || {
        music_player_analysis::analyze(&bytes, hint.as_deref())
    })
    .await??;

    put(db, source, track, &analysis).await?;
    // The local `track` row carries key and tempo too, so a track listing is
    // one query rather than a join per page. Only the local library has rows to
    // write to; for a remote provider this finds nothing and does nothing.
    write_back_key_and_bpm(db, &track.id, &analysis).await;
    Ok(analysis)
}

/// Copy the key and tempo onto the track row, if there is one.
///
/// Best-effort: the analysis is already stored and useful, and a track that is
/// not in the local table — every track of a remote provider — is the normal
/// case rather than a failure.
async fn write_back_key_and_bpm(db: &DatabaseConnection, track_id: &str, analysis: &Analysis) {
    use music_player_entity::track;

    if analysis.key.is_none() && analysis.bpm.is_none() {
        return;
    }
    let update = track::ActiveModel {
        id: Set(track_id.to_string()),
        key: match &analysis.key {
            Some(key) => Set(Some(key.clone())),
            // Nothing found is not a reason to clear what is there: an earlier
            // pass may have got an answer this one did not.
            None => sea_orm::ActiveValue::NotSet,
        },
        bpm: match analysis.bpm {
            Some(bpm) => Set(Some(bpm)),
            None => sea_orm::ActiveValue::NotSet,
        },
        ..Default::default()
    };
    if let Err(cause) = track::Entity::update(update).exec(db).await {
        tracing::debug!(%track_id, %cause, "could not store key and bpm on the track");
    }
}

/// How many local tracks still have no key or tempo.
///
/// Counted separately from the listing so a pass can report real progress —
/// "3 of 4812" — rather than progress through whichever page it happens to
/// be holding.
pub async fn unanalysed_local_count(db: &DatabaseConnection) -> u64 {
    use music_player_entity::track;

    track::Entity::find()
        .filter(
            sea_orm::Condition::any()
                .add(track::Column::Key.is_null())
                .add(track::Column::Bpm.is_null()),
        )
        .count(db)
        .await
        .unwrap_or(0) as u64
}

/// Tracks in the local library that have no key or tempo yet.
///
/// The list the background pass works through. Ordered oldest-first so a
/// resumed pass continues where it left off rather than starting again.
pub async fn unanalysed_local_tracks(db: &DatabaseConnection, limit: u64) -> Vec<TrackRef> {
    use music_player_entity::track;

    track::Entity::find()
        .filter(
            sea_orm::Condition::any()
                .add(track::Column::Key.is_null())
                .add(track::Column::Bpm.is_null()),
        )
        .order_by_asc(track::Column::Id)
        .limit(limit)
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|row| TrackRef {
            id: row.id,
            uri: row.uri,
            artist: row.artist,
            title: row.title,
        })
        .collect()
}

/// How much of a library has been analysed.
pub async fn coverage(db: &DatabaseConnection, source: &str) -> u64 {
    track_analysis::Entity::find()
        .filter(track_analysis::Column::Source.eq(source))
        .count(db)
        .await
        .unwrap_or(0) as u64
}

/// A track's analysis, alongside the id it belongs to.
pub struct Analysed {
    pub track_id: String,
    /// As it was when analysed. Only used to keep one artist from playing twice
    /// in a row, so a rename since is of no consequence.
    pub artist: String,
    pub title: String,
    pub analysis: Analysis,
}

/// Every analysed track in a library that is usable for matching.
///
/// Loaded whole rather than queried per candidate: the rows are tiny without
/// their waveforms, a large library is tens of thousands of them, and the
/// alternative is a query per comparison. Waveforms are left in the database —
/// they are the only large column and matching never looks at them.
pub async fn matchable(db: &DatabaseConnection, source: &str) -> Vec<Analysed> {
    let rows = track_analysis::Entity::find()
        .filter(track_analysis::Column::Source.eq(source))
        .filter(track_analysis::Column::Bpm.is_not_null())
        .order_by_asc(track_analysis::Column::Bpm)
        .all(db)
        .await
        .unwrap_or_default();

    rows.into_iter()
        .map(|row| Analysed {
            track_id: row.track_id.clone(),
            artist: row.artist.clone().unwrap_or_default(),
            title: row.title.clone().unwrap_or_default(),
            analysis: Analysis {
                // Dropped deliberately: a listing of every analysed track would
                // otherwise carry a waveform per row that nothing reads.
                waveform: Vec::new(),
                ..from_row(row)
            },
        })
        .filter(|analysed| analysed.analysis.is_useful_for_matching())
        .collect()
}

fn from_row(row: track_analysis::Model) -> Analysis {
    Analysis {
        waveform: row.waveform.unwrap_or_default(),
        bpm: row.bpm,
        bpm_confidence: row.bpm_confidence,
        key: row.key,
        key_confidence: row.key_confidence,
        valence: row.valence,
        arousal: row.arousal,
        moods: row
            .moods
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default(),
        lufs: row.lufs,
        true_peak_db: row.true_peak_db,
        duration: row.duration.unwrap_or_default(),
    }
}

/// The audio behind a uri.
///
/// A cached copy is used when there is one — the bytes are identical and the
/// download is already paid for. Otherwise a local path is read and a url is
/// fetched.
async fn fetch(uri: &str) -> Result<Vec<u8>> {
    if let Some(path) = crate::track_cache::cached_path(uri) {
        return Ok(tokio::fs::read(path).await?);
    }

    if !uri.starts_with("http://") && !uri.starts_with("https://") {
        let path = uri.strip_prefix("file://").unwrap_or(uri);
        return Ok(tokio::fs::read(path).await?);
    }

    if !crate::track_cache::is_cacheable(uri) {
        // The same test the cache uses, for the same reason: a live stream has
        // no end, so "download it" has no meaning.
        return Err(Error::msg("not a finite stream"));
    }

    let response = crate::atproto::http()?
        .get(uri)
        .send()
        .await?
        .error_for_status()?;

    if let Some(length) = response.content_length() {
        if length > MAX_FETCH_BYTES {
            return Err(Error::msg(format!(
                "{length} bytes is too large to analyse"
            )));
        }
    }

    let bytes = response.bytes().await?;
    if bytes.len() as u64 > MAX_FETCH_BYTES {
        // A server that sent no content-length can still send too much.
        return Err(Error::msg("too large to analyse"));
    }
    Ok(bytes.to_vec())
}

/// The file extension in a uri, as a hint to the decoder.
///
/// Taken from the path only. A Subsonic stream url ends in a query string full
/// of tokens, and reading an extension out of one would hand the prober
/// nonsense.
fn extension_of(uri: &str) -> Option<String> {
    let path = uri.split(['?', '#']).next()?;
    let extension = path.rsplit('/').next()?.rsplit_once('.')?.1;
    if extension.is_empty() || extension.len() > 5 {
        return None;
    }
    Some(extension.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use music_player_entity::track_analysis::id_for;
    use sea_orm::{ConnectionTrait, Database, DbBackend, Schema};

    /// An in-memory database with just the analysis table.
    async fn memory_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let schema = Schema::new(DbBackend::Sqlite);
        db.execute(
            db.get_database_backend()
                .build(&schema.create_table_from_entity(track_analysis::Entity)),
        )
        .await
        .unwrap();
        db
    }

    fn track(id: &str) -> TrackRef {
        TrackRef {
            id: id.into(),
            uri: format!("https://nas.lan/rest/stream?id={id}"),
            artist: "Someone".into(),
            title: id.into(),
        }
    }

    fn sample() -> Analysis {
        Analysis {
            waveform: vec![1, 2, 3, 250],
            bpm: Some(128.0),
            bpm_confidence: Some(0.8),
            valence: Some(0.3),
            arousal: Some(0.7),
            moods: vec![("energetic".into(), 0.9), ("bright".into(), 0.4)],
            key: Some("8A".into()),
            key_confidence: Some(0.7),
            lufs: Some(-9.5),
            true_peak_db: Some(-0.8),
            duration: 214.0,
        }
    }

    #[tokio::test]
    async fn what_goes_in_comes_back_out() {
        let db = memory_db().await;
        put(&db, "", &track("track-1"), &sample()).await.unwrap();

        let stored = get(&db, "", "track-1").await.expect("it was just stored");
        assert_eq!(stored, sample());
    }

    #[tokio::test]
    async fn an_unanalysed_track_has_nothing() {
        let db = memory_db().await;
        assert!(get(&db, "", "never-seen").await.is_none());
    }

    /// The same id on two servers is two tracks, and must not share a row.
    #[tokio::test]
    async fn libraries_do_not_share_rows() {
        let db = memory_db().await;
        put(&db, "", &track("shared-id"), &sample()).await.unwrap();

        assert!(get(&db, "navidrome", "shared-id").await.is_none());
        assert_eq!(coverage(&db, "").await, 1);
        assert_eq!(coverage(&db, "navidrome").await, 0);
    }

    /// Re-analysing corrects a bad result, so it must replace rather than
    /// collide — the row id is derived, so an insert would be a key conflict.
    #[tokio::test]
    async fn re_analysing_replaces_the_row() {
        let db = memory_db().await;
        put(&db, "", &track("track-1"), &sample()).await.unwrap();

        let corrected = Analysis {
            bpm: Some(64.0),
            ..sample()
        };
        put(&db, "", &track("track-1"), &corrected).await.unwrap();

        assert_eq!(get(&db, "", "track-1").await.unwrap().bpm, Some(64.0));
        assert_eq!(coverage(&db, "").await, 1);
    }

    /// Matching loads every usable row, without the waveforms — the one large
    /// column, and the one nothing matching does looks at.
    #[tokio::test]
    async fn matching_skips_waveforms_and_unusable_rows() {
        let db = memory_db().await;
        put(&db, "", &track("usable"), &sample()).await.unwrap();
        put(
            &db,
            "",
            &track("waveform-only"),
            &Analysis {
                waveform: vec![9; 400],
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let matchable = matchable(&db, "").await;
        assert_eq!(matchable.len(), 1);
        assert_eq!(matchable[0].track_id, "usable");
        assert!(matchable[0].analysis.waveform.is_empty());
        // The row is still there and still has its waveform to draw.
        assert_eq!(
            get(&db, "", "waveform-only").await.unwrap().waveform.len(),
            400
        );
    }

    #[test]
    fn the_extension_comes_from_the_path_not_the_query() {
        assert_eq!(extension_of("/music/song.flac"), Some("flac".into()));
        assert_eq!(extension_of("https://nas.lan/a/b.mp3"), Some("mp3".into()));
        // A Subsonic stream url: the extension is in the query, and the path
        // has none. Guessing from the query would hand the prober a token.
        assert_eq!(
            extension_of("https://nas.lan/rest/stream?id=7&format=raw&t=abc.def"),
            None
        );
        assert_eq!(extension_of("https://nas.lan/rest/stream?id=7"), None);
        // A dot in a directory name is not the file's extension.
        assert_eq!(extension_of("/my.music/track"), None);
    }

    #[tokio::test]
    async fn a_uri_that_is_not_audio_fails_rather_than_storing_nothing() {
        let db = memory_db().await;
        let missing = TrackRef {
            id: "missing".into(),
            uri: "/nowhere/at/all.mp3".into(),
            ..Default::default()
        };
        assert!(ensure(&db, "", &missing).await.is_err());
        // Nothing was written, so a later attempt will try again rather than
        // find an empty row and believe the track has been analysed.
        assert!(get(&db, "", "missing").await.is_none());
    }

    /// A live stream has no end; downloading one to analyse it would not stop.
    #[tokio::test]
    async fn a_live_stream_is_refused() {
        let db = memory_db().await;
        let radio = TrackRef {
            id: "radio".into(),
            uri: "https://example.com/listen.m3u".into(),
            ..Default::default()
        };
        assert!(ensure(&db, "", &radio).await.is_err());
    }

    #[test]
    fn the_row_id_matches_the_entity() {
        assert_eq!(id_for("", "a"), id_for("", "a"));
        assert_ne!(id_for("", "a"), id_for("s", "a"));
    }
}
