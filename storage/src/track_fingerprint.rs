//! Stored acoustic fingerprints, and the identities they resolve to.
//!
//! A fingerprint is the one thing about a track that is neither a tag nor a
//! measurement: it is derived from the audio alone, so it survives a file
//! being re-tagged, re-encoded or renamed, and it is the same string every
//! other Chromaprint user would compute. That is what makes it usable as a
//! question to ask a stranger — "what is this recording?" — which is what the
//! identification pass does with it.
//!
//! Computing one costs a decode, and the answer never changes, so it is
//! computed once and kept. The AcoustID answer is kept beside it for the same
//! reason, and a *miss* is kept too: without recording that the question was
//! asked, every scan would ask it again about every track the database has
//! never heard of.

use anyhow::Result;
use music_player_analysis::Fingerprint;
use music_player_entity::{track, track_fingerprint};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};

pub use crate::track_analysis::TrackRef;

/// What AcoustID said a fingerprint was.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Identity {
    /// AcoustID's own id for the recording — stable, and what a second lookup
    /// would return, so it is worth keeping even when the MusicBrainz ids are
    /// missing (a fingerprint can be known to AcoustID and linked to nothing).
    pub acoustid: String,
    pub recording_mbid: Option<String>,
    pub release_mbid: Option<String>,
    pub score: f32,
}

/// The stored fingerprint for a track, if it has one.
pub async fn get(db: &DatabaseConnection, track_id: &str) -> Option<track_fingerprint::Model> {
    track_fingerprint::Entity::find_by_id(track_id.to_string())
        .one(db)
        .await
        .ok()?
}

/// Store a fingerprint, replacing any earlier one for the same track.
///
/// The lookup columns are deliberately left alone. Re-fingerprinting happens
/// when a file is replaced by a better rip of the same recording, and throwing
/// away an identity that is still correct would mean asking AcoustID for it
/// again.
pub async fn put(db: &DatabaseConnection, track_id: &str, fingerprint: &Fingerprint) -> Result<()> {
    let row = track_fingerprint::ActiveModel {
        id: Set(track_id.to_string()),
        fingerprint: Set(fingerprint.fingerprint.clone()),
        duration: Set(fingerprint.duration as i32),
        acoustid: Set(None),
        recording_mbid: Set(None),
        release_mbid: Set(None),
        score: Set(None),
        looked_up_at: Set(None),
        fingerprinted_at: Set(chrono::Utc::now().to_rfc3339()),
    };

    track_fingerprint::Entity::insert(row)
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(track_fingerprint::Column::Id)
                .update_columns([
                    track_fingerprint::Column::Fingerprint,
                    track_fingerprint::Column::Duration,
                    track_fingerprint::Column::FingerprintedAt,
                ])
                .to_owned(),
        )
        .exec(db)
        .await?;
    Ok(())
}

/// Record what AcoustID answered — including that it answered nothing.
///
/// `None` is a real result and is stored as one: the time is written either
/// way, and it is the time, not the identity, that keeps the next scan from
/// asking again. A library of home recordings would otherwise spend every scan
/// re-asking about tracks no database will ever have.
pub async fn record_lookup(
    db: &DatabaseConnection,
    track_id: &str,
    identity: Option<&Identity>,
) -> Result<()> {
    let update = track_fingerprint::ActiveModel {
        id: Set(track_id.to_string()),
        acoustid: Set(identity.map(|found| found.acoustid.clone())),
        recording_mbid: Set(identity.and_then(|found| found.recording_mbid.clone())),
        release_mbid: Set(identity.and_then(|found| found.release_mbid.clone())),
        score: Set(identity.map(|found| found.score)),
        looked_up_at: Set(Some(chrono::Utc::now().to_rfc3339())),
        ..Default::default()
    };
    track_fingerprint::Entity::update(update).exec(db).await?;
    Ok(())
}

/// Local tracks that have never been fingerprinted.
///
/// "Never fingerprinted" — no row at all — rather than "has no fingerprint",
/// and the distinction is the one `track_analysis` had to make for the same
/// reason: a track that cannot be fingerprinted must leave the queue, or the
/// pass offers it again for ever. A row is written on success and the track
/// leaves; a failure is walked past by the caller's offset.
///
/// Local only. A fingerprint comes from the audio, and the only library whose
/// audio is on this machine is this one.
fn never_fingerprinted() -> sea_orm::Select<track::Entity> {
    use sea_orm::sea_query::{Expr, ExprTrait, Query};

    let fingerprinted = Query::select()
        .column(track_fingerprint::Column::Id)
        .from(track_fingerprint::Entity)
        .to_owned();

    track::Entity::find()
        .filter(Expr::col(track::Column::Id).not_in_subquery(fingerprinted))
        .filter(track::Column::Uri.starts_with("/"))
        // Stable, so paging by offset walks the list rather than shuffling it.
        .order_by_asc(track::Column::Id)
}

/// How many local tracks have never been fingerprinted.
pub async fn unfingerprinted_count(db: &DatabaseConnection) -> u64 {
    never_fingerprinted().count(db).await.unwrap_or(0)
}

/// A page of local tracks that have never been fingerprinted.
pub async fn unfingerprinted_tracks(
    db: &DatabaseConnection,
    offset: u64,
    limit: u64,
) -> Vec<TrackRef> {
    never_fingerprinted()
        .offset(offset)
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

/// The placeholders a missing tag turns into.
///
/// The scanner writes `"None"` for a tag a file does not have, and taggers
/// before it wrote their own — so "unknown" here means any of the handful of
/// strings that all mean the same nothing. Compared lower-cased and trimmed,
/// because `"Unknown Artist"` and `"unknown artist "` are the same absence.
pub const UNKNOWN_TAGS: [&str; 8] = [
    "",
    "none",
    "unknown",
    "unknown artist",
    "unknown album",
    "unknown title",
    "[unknown]",
    "<unknown>",
];

/// Whether a tag says nothing.
///
/// The same test the query below makes, from the same list — the pass that
/// decides *which* field to overwrite and the query that decides which tracks
/// to ask about must agree, or a track is fetched and then found to have
/// nothing worth filling in.
pub fn is_unknown(value: &str) -> bool {
    UNKNOWN_TAGS.contains(&value.trim().to_lowercase().as_str())
}

fn unknown_tag_list() -> String {
    UNKNOWN_TAGS
        .iter()
        .map(|tag| format!("'{tag}'"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Whether any of a track's identifying tags is missing.
///
/// Title, artist and album only. Year and track number are missing from
/// plenty of perfectly well tagged files, and treating those as unknown would
/// send the whole library to AcoustID to fill in a number.
///
/// Parenthesised as a whole, because it is raw sql dropped into a `WHERE` that
/// already has other clauses ANDed onto it — without the brackets the trailing
/// `OR` would swallow them and the pass would offer up the entire library.
fn has_unknown_tags() -> sea_orm::sea_query::SimpleExpr {
    let unknown = unknown_tag_list();
    sea_orm::sea_query::Expr::cust(format!(
        "(LOWER(TRIM(track.title)) IN ({unknown}) \
       OR LOWER(TRIM(track.artist)) IN ({unknown}) \
       OR track.album_id IN (SELECT id FROM album WHERE LOWER(TRIM(title)) IN ({unknown})))"
    ))
}

/// A track that is fingerprinted but not yet identified, and does not know
/// what it is.
pub struct Unidentified {
    pub track: track::Model,
    pub fingerprint: String,
    pub duration: u32,
}

/// How many tracks are waiting to be identified.
pub async fn unidentified_count(db: &DatabaseConnection) -> u64 {
    unidentified_select().count(db).await.unwrap_or(0)
}

fn unidentified_select() -> sea_orm::Select<track::Entity> {
    use sea_orm::sea_query::{Expr, ExprTrait, Query};

    let never_asked = Query::select()
        .column(track_fingerprint::Column::Id)
        .from(track_fingerprint::Entity)
        .and_where(Expr::col(track_fingerprint::Column::LookedUpAt).is_null())
        .to_owned();

    track::Entity::find()
        .filter(Expr::col(track::Column::Id).in_subquery(never_asked))
        .filter(has_unknown_tags())
        .order_by_asc(track::Column::Id)
}

/// A page of tracks with a fingerprint, no identity, and something missing
/// from their tags — the ones worth asking AcoustID about.
///
/// A track with complete tags is deliberately not here. Its fingerprint is
/// still stored and still useful (it is what a duplicate check compares), but
/// the point of the lookup is to fill in what the file does not say, and
/// asking about a file that says everything spends someone else's rate limit
/// to learn nothing.
pub async fn unidentified_tracks(db: &DatabaseConnection, limit: u64) -> Vec<Unidentified> {
    let tracks = unidentified_select()
        .limit(limit)
        .all(db)
        .await
        .unwrap_or_default();
    if tracks.is_empty() {
        return Vec::new();
    }

    // One query for the fingerprints rather than one per track: the ids are
    // already in hand and this page is a few hundred of them at most.
    let ids: Vec<String> = tracks.iter().map(|row| row.id.clone()).collect();
    let prints: std::collections::HashMap<String, track_fingerprint::Model> =
        track_fingerprint::Entity::find()
            .filter(track_fingerprint::Column::Id.is_in(ids))
            .all(db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|row| (row.id.clone(), row))
            .collect();

    tracks
        .into_iter()
        .filter_map(|track| {
            let print = prints.get(&track.id)?;
            Some(Unidentified {
                fingerprint: print.fingerprint.clone(),
                duration: print.duration.max(0) as u32,
                track,
            })
        })
        .collect()
}

/// How many tracks in the library have been fingerprinted.
pub async fn coverage(db: &DatabaseConnection) -> u64 {
    track_fingerprint::Entity::find()
        .count(db)
        .await
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use migration::{Migrator, MigratorTrait};
    use sea_orm::{ConnectionTrait, DbBackend, Statement};

    /// A library with one well-tagged track and one that knows nothing about
    /// itself — the two cases the identification pass has to tell apart.
    async fn library() -> (tempfile::TempDir, DatabaseConnection) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("music-player.sqlite3");
        let db = sea_orm::Database::connect(format!("sqlite://{}?mode=rwc", path.display()))
            .await
            .unwrap();
        Migrator::up(&db, None).await.unwrap();

        for sql in [
            "INSERT INTO artist (id, name) VALUES ('a1', 'Lil Uzi Vert')",
            "INSERT INTO artist (id, name) VALUES ('a2', 'None')",
            "INSERT INTO album (id, title, artist, artist_id) VALUES ('al1', 'Eternal Atake', 'Lil Uzi Vert', 'a1')",
            "INSERT INTO album (id, title, artist, artist_id) VALUES ('al2', 'None', 'None', 'a2')",
            r#"INSERT INTO track (id, title, artist, genre, uri, album_id, artist_id)
                VALUES ('known', 'Futsal Shuffle 2020', 'Lil Uzi Vert', 'Hip Hop', '/music/futsal.mp3', 'al1', 'a1')"#,
            r#"INSERT INTO track (id, title, artist, genre, uri, album_id, artist_id)
                VALUES ('nameless', 'None', 'None', 'None', '/music/track03.mp3', 'al2', 'a2')"#,
            r#"INSERT INTO track (id, title, artist, genre, uri, album_id, artist_id)
                VALUES ('remote', 'Streamed', 'Someone', 'Rock', 'https://example.test/s.mp3', 'al1', 'a1')"#,
        ] {
            db.execute_raw(Statement::from_string(DbBackend::Sqlite, sql.to_string()))
                .await
                .unwrap();
        }
        (dir, db)
    }

    fn print(duration: u32) -> Fingerprint {
        Fingerprint {
            fingerprint: "AQADtNKaSBqTBD8eZcfDhzB34aFDxEeSP8eTBf7xHH_Rx2i2".to_string(),
            duration,
        }
    }

    /// Only local files, and only ones with nothing stored yet. A remote
    /// track has no audio on this machine to fingerprint.
    #[tokio::test]
    async fn the_queue_is_local_tracks_with_no_fingerprint() {
        let (_dir, db) = library().await;

        let waiting = unfingerprinted_tracks(&db, 0, 100).await;
        let ids: Vec<&str> = waiting.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids, ["known", "nameless"]);
        assert_eq!(unfingerprinted_count(&db).await, 2);

        put(&db, "known", &print(192)).await.unwrap();

        let waiting = unfingerprinted_tracks(&db, 0, 100).await;
        assert_eq!(waiting.len(), 1);
        assert_eq!(waiting[0].id, "nameless");
        assert_eq!(unfingerprinted_count(&db).await, 1);
    }

    /// Fingerprinting the same track twice replaces the row rather than
    /// failing on the primary key.
    #[tokio::test]
    async fn re_fingerprinting_replaces() {
        let (_dir, db) = library().await;
        put(&db, "known", &print(192)).await.unwrap();
        put(&db, "known", &print(200)).await.unwrap();
        assert_eq!(get(&db, "known").await.unwrap().duration, 200);
    }

    /// The whole point of the pass: a file that does not know what it is gets
    /// asked about, and a file that does is left alone.
    #[tokio::test]
    async fn only_tracks_with_unknown_tags_are_asked_about() {
        let (_dir, db) = library().await;
        put(&db, "known", &print(192)).await.unwrap();
        put(&db, "nameless", &print(240)).await.unwrap();

        let pending = unidentified_tracks(&db, 100).await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].track.id, "nameless");
        assert_eq!(pending[0].duration, 240);
        assert_eq!(unidentified_count(&db).await, 1);
    }

    /// An album nobody named is a missing tag too, even when the title and
    /// artist on the track itself are fine.
    #[tokio::test]
    async fn an_unknown_album_counts_as_unknown() {
        let (_dir, db) = library().await;
        db.execute_raw(Statement::from_string(
            DbBackend::Sqlite,
            "UPDATE track SET title = 'Real Title', artist = 'Real Artist' WHERE id = 'nameless'"
                .to_string(),
        ))
        .await
        .unwrap();
        put(&db, "nameless", &print(240)).await.unwrap();

        let pending = unidentified_tracks(&db, 100).await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].track.id, "nameless");
    }

    /// The rust test and the sql have to agree about what "unknown" means.
    #[test]
    fn the_placeholders_are_recognised_however_they_are_written() {
        assert!(is_unknown("None"));
        assert!(is_unknown("  unknown artist "));
        assert!(is_unknown(""));
        assert!(!is_unknown("Nonesuch"));
        assert!(!is_unknown("Various Artists"));
    }

    /// A miss is an answer. Without storing one, every scan would ask
    /// AcoustID the same unanswerable question again.
    #[tokio::test]
    async fn a_miss_is_recorded_so_it_is_not_asked_twice() {
        let (_dir, db) = library().await;
        put(&db, "nameless", &print(240)).await.unwrap();
        assert_eq!(unidentified_tracks(&db, 100).await.len(), 1);

        record_lookup(&db, "nameless", None).await.unwrap();

        assert!(unidentified_tracks(&db, 100).await.is_empty());
        let row = get(&db, "nameless").await.unwrap();
        assert!(row.looked_up_at.is_some());
        assert!(row.acoustid.is_none());
    }

    /// A hit keeps the identity, and re-fingerprinting the audio later must
    /// not throw it away — the recording is the same recording.
    #[tokio::test]
    async fn an_identity_survives_re_fingerprinting() {
        let (_dir, db) = library().await;
        put(&db, "nameless", &print(240)).await.unwrap();
        record_lookup(
            &db,
            "nameless",
            Some(&Identity {
                acoustid: "acoust-1".to_string(),
                recording_mbid: Some("rec-1".to_string()),
                release_mbid: Some("rel-1".to_string()),
                score: 0.97,
            }),
        )
        .await
        .unwrap();

        put(&db, "nameless", &print(241)).await.unwrap();

        let row = get(&db, "nameless").await.unwrap();
        assert_eq!(row.duration, 241);
        assert_eq!(row.acoustid.as_deref(), Some("acoust-1"));
        assert_eq!(row.recording_mbid.as_deref(), Some("rec-1"));
    }
}
