//! Tests for the analytics engine.
//!
//! These are hermetic: every fixture is written into a temporary directory, so
//! nothing here depends on the machine having a music library, a Rocksky login
//! or a network connection.

use crate::sql;
use crate::Analytics;

fn db() -> Analytics {
    Analytics::open_in_memory().expect("an in-memory analytics database")
}

/// Insert a listen with only the fields a test cares about.
fn insert(a: &Analytics, origin: &str, key: &str, at: &str, artist: &str, title: &str) {
    a.conn()
        .execute(
            "INSERT OR REPLACE INTO listens
               (origin, origin_key, played_at, title, artist, match_key)
             VALUES (?, ?, ?::TIMESTAMPTZ, ?, ?, match_key(?, ?))",
            duckdb::params![origin, key, at, title, artist, artist, title],
        )
        .expect("the listen inserts");
}

fn scalar<T: duckdb::types::FromSql>(a: &Analytics, sql: &str) -> T {
    a.conn()
        .query_row(sql, [], |row| row.get(0))
        .unwrap_or_else(|e| panic!("{sql}: {e}"))
}

// ── the link itself ─────────────────────────────────────────────────────────

/// The statically linked library is the version the bindings were generated
/// against. A mismatch links cleanly and then misbehaves at runtime, so it is
/// worth one assertion.
#[test]
fn links_against_the_expected_duckdb() {
    let version: String = scalar(&db(), "SELECT version()");
    assert!(
        version.starts_with("v1.5."),
        "linked against DuckDB {version}, expected v1.5.x"
    );
}

/// `read_json_auto`, `read_csv` and Parquet all come from extensions that are
/// separate archives in the prebuilt release. Linking DuckDB without them
/// succeeds and only fails when an import is actually attempted, which is a
/// miserable way to find out.
#[test]
fn json_csv_and_parquet_extensions_are_available() {
    let dir = tempfile::tempdir().unwrap();
    let a = db();

    let json = dir.path().join("listens.json");
    std::fs::write(&json, r#"[{"ts":"2025-01-01T00:00:00Z","ms_played":1000}]"#).unwrap();
    let rows: i64 = scalar(
        &a,
        &format!("SELECT count(*) FROM read_json_auto('{}')", json.display()),
    );
    assert_eq!(rows, 1, "read_json_auto should read the fixture");

    let csv = dir.path().join("scrobbles.csv");
    std::fs::write(
        &csv,
        "uts,artist,track\n1700000000,Boards of Canada,Roygbiv\n",
    )
    .unwrap();
    let rows: i64 = scalar(
        &a,
        &format!(
            "SELECT count(*) FROM read_csv('{}', header := true)",
            csv.display()
        ),
    );
    assert_eq!(rows, 1, "read_csv should read the fixture");

    let parquet = dir.path().join("out.parquet");
    a.conn()
        .execute_batch(&format!(
            "COPY (SELECT 1 AS a) TO '{}' (FORMAT PARQUET)",
            parquet.display()
        ))
        .expect("parquet export works");
    let rows: i64 = scalar(
        &a,
        &format!("SELECT count(*) FROM read_parquet('{}')", parquet.display()),
    );
    assert_eq!(rows, 1, "parquet should round-trip");
}

// ── normalisation ───────────────────────────────────────────────────────────

/// The bug that a real Last.fm export caught: folding with `[^a-z0-9]+` erases
/// a title with no ASCII letters down to nothing, so every Japanese, Cyrillic
/// or Greek title collapsed onto one key and looked like the same song.
#[test]
fn non_latin_titles_keep_distinct_keys() {
    let a = db();
    let distinct: i64 = scalar(
        &a,
        "SELECT count(DISTINCT match_key('サカナクション', x))
         FROM (VALUES ('新宝島'), ('ミュージック'), ('夜の踊り子')) AS t(x)",
    );
    assert_eq!(distinct, 3, "three different titles, three different keys");

    let empty: i64 = scalar(
        &a,
        "SELECT count(*) FROM (VALUES ('新宝島'), ('Привет'), ('Ωμέγα')) AS t(x)
         WHERE nullif(trim(norm(x)), '') IS NULL",
    );
    assert_eq!(empty, 0, "no non-Latin title should normalise away");
}

/// Spotify records the album artist alone; Last.fm records the full credit.
/// Matching the whole string finds nothing, so only the primary credit counts.
#[test]
fn primary_artist_matches_across_catalogues() {
    let a = db();
    let same: bool = scalar(
        &a,
        "SELECT match_key('Riton', 'Turn Me On (feat. Vula)')
              = match_key('Riton, Oliver Heldens, Vula', 'Turn Me On')",
    );
    assert!(
        same,
        "the same listen should key identically in both exports"
    );

    let different: bool = scalar(
        &a,
        "SELECT match_key('Riton', 'Turn Me On') <> match_key('Riton', 'Rinse & Repeat')",
    );
    assert!(different, "different songs must not collide");
}

/// Remaster and edition suffixes describe a release, not a different song.
#[test]
fn edition_suffixes_fold_together() {
    let a = db();
    let same: bool = scalar(
        &a,
        "SELECT match_key('Bowie', 'Heroes - 2017 Remaster') = match_key('Bowie', 'Heroes')",
    );
    assert!(same, "a remaster is the same song");
}

/// A title that is nothing but punctuation still has to produce a usable key
/// rather than a NULL that fails the table's NOT NULL constraint.
#[test]
fn degenerate_titles_still_key() {
    let a = db();
    let key: String = scalar(&a, "SELECT match_key('!!!', '...')");
    assert!(!key.is_empty(), "a key is always produced");
}

// ── deduplication ───────────────────────────────────────────────────────────

/// The real finding, reduced to a fixture: a Last.fm export whose scrobbles
/// all sit exactly three hours after their Spotify twin, because the scrobbler
/// submitted local time as UTC. The offset must be *detected* from the data —
/// hard-coding +3h would be right for one timezone and wrong everywhere else.
#[test]
fn detects_a_clock_offset_and_drops_the_duplicates() {
    let a = db();

    // 300 pairs, three hours apart: above the agreement threshold.
    for i in 0..300 {
        let hour = i / 60;
        let minute = i % 60;
        let spotify = format!("2025-06-02 {:02}:{:02}:07+00", hour, minute);
        let lastfm = format!("2025-06-02 {:02}:{:02}:07+00", hour + 3, minute);
        let artist = format!("Artist {i}");
        insert(&a, "spotify", &format!("s{i}"), &spotify, &artist, "Track");
        insert(&a, "lastfm", &format!("l{i}"), &lastfm, &artist, "Track");
    }
    a.refresh_dedup().unwrap();

    let offset: i64 = scalar(
        &a,
        "SELECT offset_s::BIGINT FROM origin_offset WHERE lo = 'lastfm' AND hi = 'spotify'",
    );
    assert_eq!(offset, 3 * 3600, "the three-hour offset should be detected");

    assert_eq!(a.count().unwrap(), 600, "600 raw listens");
    let canonical: i64 = scalar(&a, "SELECT count(*) FROM canonical_listens");
    assert_eq!(
        canonical, 300,
        "each pair collapses to one canonical listen"
    );

    // Spotify outranks Last.fm, so the surviving row is the richer one.
    let surviving: i64 = scalar(
        &a,
        "SELECT count(*) FROM canonical_listens WHERE origin = 'spotify'",
    );
    assert_eq!(surviving, 300, "the more trusted origin is the one kept");
}

/// Without enough agreeing pairs an apparent offset is coincidence. Assuming
/// one would shift unrelated listens on top of each other and delete real
/// history, so a thin signal must be ignored.
#[test]
fn ignores_an_offset_too_thin_to_trust() {
    let a = db();
    for i in 0..5 {
        let artist = format!("Artist {i}");
        insert(
            &a,
            "spotify",
            &format!("s{i}"),
            &format!("2025-06-02 01:{i:02}:00+00"),
            &artist,
            "Track",
        );
        insert(
            &a,
            "lastfm",
            &format!("l{i}"),
            &format!("2025-06-02 04:{i:02}:00+00"),
            &artist,
            "Track",
        );
    }
    a.refresh_dedup().unwrap();

    let offset: i64 = scalar(
        &a,
        "SELECT coalesce(max(offset_s), 0)::BIGINT FROM origin_offset
         WHERE lo = 'lastfm' AND hi = 'spotify'",
    );
    assert_eq!(offset, 0, "five pairs is not evidence of a clock offset");

    let canonical: i64 = scalar(&a, "SELECT count(*) FROM canonical_listens");
    assert_eq!(canonical, 10, "so nothing is dropped");
}

/// Two genuinely separate plays of the same track, hours apart with no
/// systematic offset in play, are not duplicates.
#[test]
fn keeps_genuine_repeat_plays() {
    let a = db();
    insert(
        &a,
        "local",
        "1",
        "2025-06-02 09:00:00+00",
        "Burial",
        "Archangel",
    );
    insert(
        &a,
        "local",
        "2",
        "2025-06-02 15:00:00+00",
        "Burial",
        "Archangel",
    );
    a.refresh_dedup().unwrap();

    let canonical: i64 = scalar(&a, "SELECT count(*) FROM canonical_listens");
    assert_eq!(canonical, 2, "playing a track twice is two listens");
}

/// Re-importing the same export must not grow the table — the whole point of
/// keying on (origin, origin_key).
#[test]
fn reimport_is_idempotent() {
    let a = db();
    for _ in 0..3 {
        insert(
            &a,
            "spotify",
            "k1",
            "2025-06-02 09:00:00+00",
            "Aphex Twin",
            "Xtal",
        );
    }
    assert_eq!(
        a.count().unwrap(),
        1,
        "the same listen imported thrice is one row"
    );
}

// ── schema ──────────────────────────────────────────────────────────────────

/// `migrate` runs on every open, so it has to survive being run repeatedly
/// against a database that already has data in it.
#[test]
fn migrate_is_idempotent_and_preserves_data() {
    let a = db();
    insert(
        &a,
        "local",
        "1",
        "2025-06-02 09:00:00+00",
        "Four Tet",
        "Two Thousand and Seventeen",
    );
    a.migrate().unwrap();
    a.migrate().unwrap();
    assert_eq!(a.count().unwrap(), 1, "re-migrating keeps the data");
}

/// The dedup views are defined against tables the schema creates; building
/// them on an empty database must work, because that is what the first run
/// after installing does.
#[test]
fn dedup_works_on_an_empty_database() {
    let a = db();
    a.refresh_dedup().unwrap();
    let canonical: i64 = scalar(&a, "SELECT count(*) FROM canonical_listens");
    assert_eq!(canonical, 0);
}

/// `enriched_listens` has to survive a listen with no local track and no
/// resolved match — which is every row of a fresh import.
#[test]
fn enrichment_tolerates_unmatched_listens() {
    let a = db();
    insert(
        &a,
        "spotify",
        "k1",
        "2025-06-02 09:00:00+00",
        "Unknown",
        "Untitled",
    );
    a.refresh_dedup().unwrap();
    let rows: i64 = scalar(
        &a,
        "SELECT count(*) FROM enriched_listens WHERE album IS NULL",
    );
    assert_eq!(
        rows, 1,
        "an unmatched listen still appears, just without metadata"
    );
}

/// The schema text is applied as one batch; a syntax error in any statement
/// would only surface at runtime.
#[test]
fn schema_and_dedup_sql_are_valid() {
    let a = db();
    a.conn().execute_batch(sql::SCHEMA).unwrap();
    a.conn().execute_batch(sql::DEDUP).unwrap();
}

// ── syncing from the player's SQLite database ───────────────────────────────

/// A real migrated SQLite database with a small library and listen log.
///
/// Built with the project's own migrations rather than hand-written DDL, so
/// this test fails if the sync queries drift away from the real schema —
/// which is the failure worth catching.
async fn sqlite_fixture() -> (tempfile::TempDir, sea_orm::DatabaseConnection) {
    use migration::{Migrator, MigratorTrait};
    use sea_orm::{ConnectionTrait, DbBackend, Statement};

    let dir = tempfile::tempdir().unwrap();
    let url = format!(
        "sqlite://{}?mode=rwc",
        dir.path().join("music-player.sqlite3").display()
    );
    let conn = sea_orm::Database::connect(&url).await.unwrap();
    Migrator::up(&conn, None).await.unwrap();

    for sql in [
        "INSERT INTO artist (id, name) VALUES ('a1', 'Boards of Canada')",
        "INSERT INTO album (id, title, artist, artist_id, year) \
         VALUES ('al1', 'Music Has the Right to Children', 'Boards of Canada', 'a1', 1998)",
        "INSERT INTO track (id, title, artist, genre, duration, uri, album_id, artist_id, bpm, key) \
         VALUES ('t1', 'Roygbiv', 'Boards of Canada', 'Electronic', 170.5, '/m/roygbiv.mp3', 'al1', 'a1', 101.5, '8A')",
        "INSERT INTO track (id, title, artist, genre, duration, uri, album_id, artist_id) \
         VALUES ('t2', 'Olson', 'Boards of Canada', 'Electronic', 90.0, '/m/olson.mp3', 'al1', 'a1')",
        // A local play, a remote one carrying its own snapshot, and a skip.
        "INSERT INTO play_history (track_id, played_at, ms_played, length_ms, skipped, source, title, artist) \
         VALUES ('t1', 1750000000, 170000, 170500, 0, 'local', '', '')",
        "INSERT INTO play_history (track_id, played_at, ms_played, length_ms, skipped, source, title, artist) \
         VALUES ('t2', 1750000300, 4000, 90000, 1, 'local', '', '')",
        "INSERT INTO play_history (track_id, played_at, ms_played, length_ms, skipped, source, title, artist) \
         VALUES ('remote-9', 1750000600, 200000, 210000, 0, 'navidrome.example', 'Sun Ra', 'Space Is the Place')",
    ] {
        conn.execute_raw(Statement::from_string(DbBackend::Sqlite, sql.to_string()))
            .await
            .unwrap();
    }
    (dir, conn)
}

#[tokio::test]
async fn sync_mirrors_listens_and_tracks() {
    let (_dir, sqlite) = sqlite_fixture().await;
    let a = db();

    let report = crate::sync::sync(&a, &sqlite).await.unwrap();
    assert_eq!(report.listens, 3, "three listens mirrored");
    assert_eq!(report.tracks, 2, "two library tracks mirrored");

    // The local rows take their title from the `track` row, not the blank
    // snapshot; the remote row has only its snapshot to go on.
    let title: String = scalar(&a, "SELECT title FROM listens WHERE track_id = 't1'");
    assert_eq!(title, "Roygbiv");
    let remote: String = scalar(&a, "SELECT artist FROM listens WHERE track_id = 'remote-9'");
    assert_eq!(remote, "Space Is the Place");

    // Audio features ride along for joins.
    let bpm: f64 = scalar(&a, "SELECT bpm FROM tracks WHERE track_id = 't1'");
    assert!((bpm - 101.5).abs() < 0.01, "bpm survives the mirror");
    let duration: i64 = scalar(&a, "SELECT duration_ms FROM tracks WHERE track_id = 't1'");
    assert_eq!(duration, 170_500, "seconds become milliseconds");

    let skipped: bool = scalar(&a, "SELECT skipped FROM listens WHERE track_id = 't2'");
    assert!(skipped, "a skip stays a skip");
}

/// The watermark is the whole point of the design: a second sync with nothing
/// new must be a no-op, not a re-import.
#[tokio::test]
async fn sync_is_incremental() {
    let (_dir, sqlite) = sqlite_fixture().await;
    let a = db();

    assert_eq!(crate::sync::sync(&a, &sqlite).await.unwrap().listens, 3);
    assert_eq!(
        crate::sync::sync(&a, &sqlite).await.unwrap().listens,
        0,
        "nothing new to mirror"
    );
    assert_eq!(a.count().unwrap(), 3, "and no duplicate rows");

    use sea_orm::{ConnectionTrait, DbBackend, Statement};
    sqlite
        .execute_raw(Statement::from_string(
            DbBackend::Sqlite,
            "INSERT INTO play_history (track_id, played_at, ms_played, length_ms, skipped, source, title, artist) \
             VALUES ('t1', 1750001000, 170000, 170500, 0, 'local', '', '')"
                .to_string(),
        ))
        .await
        .unwrap();

    assert_eq!(
        crate::sync::sync(&a, &sqlite).await.unwrap().listens,
        1,
        "only the new listen is mirrored"
    );
    assert_eq!(a.count().unwrap(), 4);
}

// ── imports ─────────────────────────────────────────────────────────────────

/// A miniature Spotify export: a music listen, a podcast episode (no track
/// metadata), and a track abandoned after four seconds.
fn spotify_export(dir: &std::path::Path) -> std::path::PathBuf {
    let file = dir.join("Streaming_History_Audio_2025.json");
    std::fs::write(
        &file,
        r#"[
          {"ts":"2025-06-02T07:25:07Z","platform":"osx","ms_played":205000,
           "conn_country":"MG","master_metadata_track_name":"Raiso",
           "master_metadata_album_artist_name":"Ceasar",
           "master_metadata_album_album_name":"Sample","spotify_track_uri":"spotify:track:abc",
           "episode_name":null,"reason_start":"trackdone","reason_end":"trackdone",
           "shuffle":true,"skipped":false},
          {"ts":"2025-06-02T08:00:00Z","platform":"osx","ms_played":900000,
           "conn_country":"MG","master_metadata_track_name":null,
           "master_metadata_album_artist_name":null,
           "master_metadata_album_album_name":null,"spotify_track_uri":null,
           "episode_name":"Some Podcast","reason_start":"trackdone","reason_end":"endplay",
           "shuffle":false,"skipped":false},
          {"ts":"2025-06-02T09:00:00Z","platform":"osx","ms_played":4000,
           "conn_country":"MG","master_metadata_track_name":"Skipped Song",
           "master_metadata_album_artist_name":"Someone",
           "master_metadata_album_album_name":"Album","spotify_track_uri":"spotify:track:def",
           "episode_name":null,"reason_start":"trackdone","reason_end":"fwdbtn",
           "shuffle":false,"skipped":true}
        ]"#,
    )
    .unwrap();
    file
}

#[test]
fn imports_a_spotify_export() {
    let dir = tempfile::tempdir().unwrap();
    spotify_export(dir.path());
    let a = db();

    let report = crate::import::spotify::import(
        &a,
        dir.path(),
        crate::import::spotify::DEFAULT_MIN_SECONDS,
        &crate::progress::Silent,
    )
    .unwrap();

    assert_eq!(report.read, 3, "all three records were examined");
    assert_eq!(
        report.inserted, 1,
        "only the real, long-enough listen is a listen"
    );
    assert_eq!(
        report
            .skipped
            .get("not a track (podcast, audiobook, video)"),
        Some(&1),
        "the podcast is reported, not silently dropped"
    );
    assert_eq!(report.skipped.get("played under 30s"), Some(&1));

    // Spotify's own end-reason is kept: it is a better skip signal than any
    // duration heuristic the player could apply after the fact.
    let reason: String = scalar(&a, "SELECT reason_end FROM listens WHERE title = 'Raiso'");
    assert_eq!(reason, "trackdone");
    let shuffle: bool = scalar(&a, "SELECT shuffle FROM listens WHERE title = 'Raiso'");
    assert!(shuffle);
}

/// Re-running an import over the same export must not double the history.
#[test]
fn spotify_import_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    spotify_export(dir.path());
    let a = db();

    for _ in 0..3 {
        crate::import::spotify::import(&a, dir.path(), 30, &crate::progress::Silent).unwrap();
    }
    assert_eq!(a.count().unwrap(), 1, "three imports, one listen");
}

/// A lower threshold admits the short play; the default would drop it.
#[test]
fn spotify_min_seconds_is_respected() {
    let dir = tempfile::tempdir().unwrap();
    spotify_export(dir.path());
    let a = db();

    let report =
        crate::import::spotify::import(&a, dir.path(), 0, &crate::progress::Silent).unwrap();
    assert_eq!(
        report.inserted, 2,
        "no threshold, so the 4s play counts too"
    );
}

/// Pointing the importer at a folder with no history files should say so
/// rather than silently importing nothing.
#[test]
fn spotify_import_rejects_an_unrelated_directory() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("notes.txt"), "hello").unwrap();
    let err = crate::import::spotify::import(&db(), dir.path(), 30, &crate::progress::Silent)
        .unwrap_err()
        .to_string();
    assert!(err.contains("Streaming_History_Audio"), "got: {err}");
}

/// The video history sits in the same export and has the same shape, but a
/// watched video is not a listen.
#[test]
fn spotify_import_ignores_the_video_history() {
    let dir = tempfile::tempdir().unwrap();
    spotify_export(dir.path());
    std::fs::write(
        dir.path().join("Streaming_History_Video_2025.json"),
        r#"[{"ts":"2025-06-02T10:00:00Z","ms_played":500000,
             "master_metadata_track_name":"A Video",
             "master_metadata_album_artist_name":"Someone"}]"#,
    )
    .unwrap();

    let a = db();
    let report =
        crate::import::spotify::import(&a, dir.path(), 30, &crate::progress::Silent).unwrap();
    assert_eq!(report.files.len(), 1, "only the audio history is read");
    assert_eq!(report.inserted, 1);
}

#[test]
fn imports_a_lastfm_csv() {
    let dir = tempfile::tempdir().unwrap();
    let csv = dir.path().join("scrobbles.csv");
    std::fs::write(
        &csv,
        "uts,utc_time,artist,artist_mbid,album,album_mbid,track,track_mbid\n\
         \"1779022496\",\"17 May 2026, 12:54\",\"Nelly Furtado\",\"mb1\",\"Loose\",\"mb2\",\"Say It Right\",\"\"\n\
         \"1779022363\",\"17 May 2026, 12:52\",\"Ofenbach, Svea\",\"\",\"I\",\"\",\"Body Talk\",\"\"\n\
         \"\",\"bad row\",\"\",\"\",\"\",\"\",\"\",\"\"\n",
    )
    .unwrap();

    let a = db();
    let report = crate::import::lastfm::import(&a, &csv, &crate::progress::Silent).unwrap();
    assert_eq!(report.inserted, 2, "two usable scrobbles");
    assert_eq!(
        report.skipped.get("missing track, artist or timestamp"),
        Some(&1)
    );

    // The CSV's own utc_time column says 12:54 UTC; the uts must agree, or
    // every chart is shifted.
    let when: String = scalar(
        &a,
        "SELECT strftime(played_at AT TIME ZONE 'UTC', '%Y-%m-%d %H:%M')
         FROM listens WHERE title = 'Say It Right'",
    );
    assert_eq!(when, "2026-05-17 12:54");
}

#[test]
fn imports_a_lastfm_json_dump() {
    let dir = tempfile::tempdir().unwrap();
    let json = dir.path().join("recent.json");
    // The getRecentTracks shape, including a now-playing entry with no date
    // and an artist given as an object rather than a string.
    std::fs::write(
        &json,
        r##"{"recenttracks":{"track":[
            {"name":"Roygbiv","artist":{"#text":"Boards of Canada"},
             "album":{"#text":"Music Has the Right to Children"},"date":{"uts":"1700000000"}},
            {"name":"Olson","artist":"Boards of Canada","date":1700000300},
            {"name":"Now Playing","artist":"Someone","@attr":{"nowplaying":"true"}}
        ]}}"##,
    )
    .unwrap();

    let a = db();
    let report = crate::import::lastfm::import(&a, &json, &crate::progress::Silent).unwrap();
    assert_eq!(report.inserted, 2, "the now-playing entry is not a listen");
    assert_eq!(
        report.skipped.get("now playing (no timestamp yet)"),
        Some(&1)
    );

    let artist: String = scalar(&a, "SELECT artist FROM listens WHERE title = 'Roygbiv'");
    assert_eq!(artist, "Boards of Canada", "the #text form is unwrapped");
    let artist: String = scalar(&a, "SELECT artist FROM listens WHERE title = 'Olson'");
    assert_eq!(artist, "Boards of Canada", "the bare-string form works too");
}

// ── format detection ────────────────────────────────────────────────────────

#[test]
fn detects_export_formats() {
    use crate::import::{detect, Format};

    let dir = tempfile::tempdir().unwrap();
    spotify_export(dir.path());
    assert_eq!(
        detect(dir.path()).unwrap(),
        Format::Spotify,
        "a directory is a Spotify export"
    );
    assert_eq!(
        detect(&dir.path().join("Streaming_History_Audio_2025.json")).unwrap(),
        Format::Spotify
    );

    let csv = dir.path().join("scrobbles.csv");
    std::fs::write(&csv, "uts,utc_time,artist,album,track\n1,2,3,4,5\n").unwrap();
    assert_eq!(detect(&csv).unwrap(), Format::Lastfm);

    let json = dir.path().join("recent.json");
    std::fs::write(&json, r#"{"recenttracks":{"track":[]}}"#).unwrap();
    assert_eq!(detect(&json).unwrap(), Format::Lastfm);
}

#[test]
fn unrecognised_exports_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let odd = dir.path().join("mystery.txt");
    std::fs::write(&odd, "who knows\n1,2,3\n").unwrap();
    assert!(crate::import::detect(&odd).is_err());
}

/// A path with a quote in it must not be able to terminate the SQL string
/// literal that DuckDB's file readers require.
#[test]
fn paths_with_quotes_are_escaped() {
    let dir = tempfile::tempdir().unwrap();
    let odd = dir.path().join("Tsiry's export");
    std::fs::create_dir(&odd).unwrap();
    spotify_export(&odd);

    let a = db();
    let report = crate::import::spotify::import(&a, &odd, 30, &crate::progress::Silent).unwrap();
    assert_eq!(report.inserted, 1, "a quoted path imports normally");
}

// ── the query engine ────────────────────────────────────────────────────────

/// A fixture with a known shape, so every assertion below is a number that can
/// be worked out by hand rather than a snapshot of whatever the code did.
///
/// Two sessions on 2025-06-02 separated by a four-hour gap, and one listen a
/// year earlier:
///   * session A, 09:00 — five tracks, one of them skipped
///   * session B, 14:00 — two tracks
fn listening_fixture() -> Analytics {
    let a = db();
    let session_a = [
        ("09:00:00", "Boards of Canada", "Roygbiv", 170_000i64, false),
        ("09:03:00", "Boards of Canada", "Olson", 90_000, false),
        ("09:05:00", "Aphex Twin", "Xtal", 4_000, true),
        ("09:10:00", "Burial", "Archangel", 240_000, false),
        ("09:15:00", "Boards of Canada", "Roygbiv", 170_000, false),
    ];
    for (i, (time, artist, title, ms, skipped)) in session_a.iter().enumerate() {
        a.conn()
            .execute(
                "INSERT OR REPLACE INTO listens
                   (origin, origin_key, played_at, title, artist, ms_played,
                    length_ms, skipped, source, match_key)
                 VALUES ('local', ?, ?::TIMESTAMPTZ, ?, ?, ?, 180000, ?, 'local', match_key(?, ?))",
                duckdb::params![
                    format!("a{i}"),
                    format!("2025-06-02 {time}+00"),
                    title,
                    artist,
                    ms,
                    skipped,
                    artist,
                    title
                ],
            )
            .unwrap();
    }
    for (i, (time, artist, title)) in [
        ("14:00:00", "Four Tet", "Two Thousand and Seventeen"),
        ("14:06:00", "Burial", "Archangel"),
    ]
    .iter()
    .enumerate()
    {
        a.conn()
            .execute(
                "INSERT OR REPLACE INTO listens
                   (origin, origin_key, played_at, title, artist, ms_played,
                    length_ms, skipped, source, match_key)
                 VALUES ('local', ?, ?::TIMESTAMPTZ, ?, ?, 180000, 180000, false, 'local',
                         match_key(?, ?))",
                duckdb::params![
                    format!("b{i}"),
                    format!("2025-06-02 {time}+00"),
                    title,
                    artist,
                    artist,
                    title
                ],
            )
            .unwrap();
    }
    a.conn()
        .execute(
            "INSERT OR REPLACE INTO listens
               (origin, origin_key, played_at, title, artist, ms_played, length_ms,
                skipped, source, match_key)
             VALUES ('local', 'old', '2024-06-02 09:00:00+00'::TIMESTAMPTZ, 'Xtal',
                     'Aphex Twin', 300000, 300000, false, 'local',
                     match_key('Aphex Twin', 'Xtal'))",
            [],
        )
        .unwrap();
    a.refresh_dedup().unwrap();
    a
}

#[test]
fn overview_counts_what_it_says() {
    let a = listening_fixture();
    let o = crate::query::overview(&a, crate::query::Scope::all()).unwrap();

    assert_eq!(o.listens, 8, "eight listens in the fixture");
    // Roygbiv twice and Xtal twice, so five distinct tracks: Roygbiv,
    // Olson, Xtal, Archangel, Two Thousand and Seventeen.
    assert_eq!(o.distinct_tracks, 5);
    // Boards of Canada, Aphex Twin, Burial, Four Tet.
    assert_eq!(o.distinct_artists, 4);
    assert_eq!(o.active_days, 2, "two days with listening");
    assert_eq!(o.sessions, 3, "two on the day, one a year before");
    assert_eq!(o.longest_streak, 1, "no two consecutive days");
}

/// A window must actually restrict, or every "this month" figure is wrong.
#[test]
fn overview_respects_the_window() {
    let a = listening_fixture();
    let window = crate::Window {
        since: Some(
            chrono::DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z")
                .unwrap()
                .timestamp(),
        ),
        until: None,
    };
    let o = crate::query::overview(&a, window.into()).unwrap();
    assert_eq!(o.listens, 7, "the 2024 listen is outside the window");
    assert_eq!(o.sessions, 2);
}

#[test]
fn sessions_split_on_a_long_gap() {
    let a = listening_fixture();
    let stats = crate::query::session_stats(&a, crate::query::Scope::all(), 5).unwrap();

    assert_eq!(stats.sessions, 3);
    // The four-hour gap between 09:15 and 14:00 splits the day in two; the
    // two-minute gaps inside session A do not.
    let longest = stats.longest_minutes.round() as i64;
    assert_eq!(longest, 15, "session A runs 09:00 to 09:15");

    // Session A opens with Roygbiv, session B with Four Tet, and the 2024
    // session with Xtal — each opens exactly one session.
    assert_eq!(stats.openers.len(), 3);
    assert!(stats.openers.iter().all(|o| o.listens == 1));
}

#[test]
fn top_artists_ranks_by_listens() {
    let a = listening_fixture();
    let top = crate::query::top(
        &a,
        crate::query::TopKind::Artists,
        crate::query::Scope::all(),
        10,
    )
    .unwrap();

    assert_eq!(top[0].name, "Boards of Canada");
    assert_eq!(top[0].listens, 3);
    assert_eq!(top.len(), 4);
}

#[test]
fn top_tracks_folds_repeats_together() {
    let a = listening_fixture();
    let top = crate::query::top(
        &a,
        crate::query::TopKind::Tracks,
        crate::query::Scope::all(),
        10,
    )
    .unwrap();

    let roygbiv = top.iter().find(|r| r.name == "Roygbiv").unwrap();
    assert_eq!(roygbiv.listens, 2, "the same track twice is one row of two");
}

#[test]
fn skips_use_the_recorded_signal() {
    let a = listening_fixture();
    let rows = crate::query::skips(&a, crate::query::Scope::all(), 10, 1).unwrap();

    let xtal = rows.iter().find(|r| r.name == "Xtal").unwrap();
    assert_eq!(xtal.listens, 2, "played twice across the fixture");
    assert_eq!(xtal.skips, 1);
    assert!((xtal.skip_rate - 0.5).abs() < 1e-9);

    // Tracks that were never skipped should rank below it.
    assert_eq!(rows[0].name, "Xtal", "the most-skipped track leads");
}

#[test]
fn transitions_are_within_a_session_only() {
    let a = listening_fixture();
    let t = crate::query::transitions(&a, crate::query::Scope::all(), 50).unwrap();

    // Session A gives four transitions, session B one. The gap between them
    // must not produce a Roygbiv -> Four Tet edge.
    assert_eq!(t.iter().map(|t| t.times).sum::<i64>(), 5);
    assert!(
        !t.iter()
            .any(|t| t.from_title == "Roygbiv" && t.to_title == "Two Thousand and Seventeen"),
        "a session boundary is not a transition"
    );
}

#[test]
fn clock_buckets_by_local_hour() {
    let a = listening_fixture();
    let cells = crate::query::clock(&a, crate::query::Scope::all()).unwrap();
    let total: i64 = cells.iter().map(|c| c.listens).sum();
    assert_eq!(total, 8, "every listen lands in exactly one cell");
}

#[test]
fn drift_buckets_and_measures_discovery() {
    let a = listening_fixture();
    let points = crate::query::drift(&a, crate::query::Scope::all(), "year").unwrap();

    assert_eq!(points.len(), 2, "2024 and 2025");
    // Every track in 2024 was heard for the first time.
    assert!((points[0].discovery_rate - 1.0).abs() < 1e-9);
    // In 2025 only the tracks not already seen in 2024 are new: Xtal is a
    // repeat, the other five are firsts, over seven listens.
    assert!(points[1].discovery_rate < 1.0);

    assert!(crate::query::drift(&a, crate::query::Scope::all(), "fortnight").is_err());
}

#[test]
fn rotation_measures_concentration() {
    let a = listening_fixture();
    let r = crate::query::rotation(&a, crate::query::Scope::all(), 10).unwrap();

    assert!(r.gini > 0.0 && r.gini < 1.0, "gini was {}", r.gini);
    assert!(r.top_10_percent_share > 0.0);
}

/// An empty database must answer every question with zeroes rather than an
/// error — this is what the screens show before anything has been imported.
#[test]
fn every_query_survives_an_empty_database() {
    let a = db();
    a.refresh_dedup().unwrap();
    let w = crate::query::Scope::all();

    assert_eq!(crate::query::overview(&a, w).unwrap().listens, 0);
    assert!(crate::query::top(&a, crate::query::TopKind::Artists, w, 10)
        .unwrap()
        .is_empty());
    assert!(crate::query::clock(&a, w).unwrap().is_empty());
    assert_eq!(crate::query::session_stats(&a, w, 5).unwrap().sessions, 0);
    assert!(crate::query::skips(&a, w, 10, 1).unwrap().is_empty());
    assert!(crate::query::drift(&a, w, "month").unwrap().is_empty());
    assert!(crate::query::transitions(&a, w, 10).unwrap().is_empty());
    assert!(crate::query::on_this_day(&a, 10).unwrap().is_empty());
    assert!(crate::query::origins(&a).unwrap().is_empty());

    let r = crate::query::rotation(&a, w, 10).unwrap();
    assert_eq!(r.gini, 0.0);
    assert_eq!(r.library_tracks, 0);
}

// ── bulk staging ────────────────────────────────────────────────────────────

/// A feed page can overlap the previous one, so the same key can appear twice
/// in a single batch. `INSERT OR REPLACE` rejects a batch that conflicts with
/// itself, so the staging fold has to collapse those first.
#[test]
fn staging_tolerates_duplicates_within_a_batch() {
    use crate::staging::{Row, Staging, When};

    let a = db();
    let mut staging = Staging::new(&a);
    for _ in 0..3 {
        staging
            .push(Row {
                origin: "rocksky".into(),
                origin_key: "at://same".into(),
                played_at: When::Rfc3339("2025-06-02T09:00:00Z".into()),
                title: "Archangel".into(),
                artist: "Burial".into(),
                source: "rocksky".into(),
                ..Default::default()
            })
            .unwrap();
    }
    staging.flush().unwrap();

    assert_eq!(a.count().unwrap(), 1, "the batch collapses to one row");
    assert_eq!(staging.inserted, 1);
}

/// Epoch seconds and RFC-3339 strings must land on the same instant, or the
/// SQLite mirror and the Rocksky feed disagree about when things happened.
#[test]
fn staging_accepts_both_timestamp_forms() {
    use crate::staging::{Row, Staging, When};

    let a = db();
    let mut staging = Staging::new(&a);
    // 2025-06-02T09:00:00Z
    staging
        .push(Row {
            origin: "local".into(),
            origin_key: "1".into(),
            played_at: When::Epoch(1_748_854_800),
            title: "Roygbiv".into(),
            artist: "Boards of Canada".into(),
            source: "local".into(),
            ..Default::default()
        })
        .unwrap();
    staging
        .push(Row {
            origin: "rocksky".into(),
            origin_key: "2".into(),
            played_at: When::Rfc3339("2025-06-02T09:00:00Z".into()),
            title: "Roygbiv".into(),
            artist: "Boards of Canada".into(),
            source: "rocksky".into(),
            ..Default::default()
        })
        .unwrap();
    staging.flush().unwrap();

    let same: bool = scalar(&a, "SELECT count(DISTINCT played_at) = 1 FROM listens");
    assert!(same, "both forms describe the same instant");
}

/// A row with no title cannot be keyed or displayed; it must not reach
/// `listens` and trip the NOT NULL constraint.
#[test]
fn staging_drops_untitled_rows() {
    use crate::staging::{Row, Staging, When};

    let a = db();
    let mut staging = Staging::new(&a);
    staging
        .push(Row {
            origin: "local".into(),
            origin_key: "1".into(),
            played_at: When::Epoch(1_748_854_800),
            title: "   ".into(),
            artist: "Nobody".into(),
            source: "local".into(),
            ..Default::default()
        })
        .unwrap();
    staging.flush().unwrap();
    assert_eq!(a.count().unwrap(), 0);
}

/// A radio stream is recorded as one row per announced song while `ms_played`
/// keeps counting for the whole session, so a single row can claim hours. On a
/// real library that put more than half of all reported listening time onto
/// one station. A listen may contribute at most one track's length.
#[test]
fn a_single_listen_cannot_claim_hours() {
    let a = db();
    // Eight hours of stream attributed to one announcement, with no length.
    a.conn()
        .execute(
            "INSERT INTO listens (origin, origin_key, played_at, title, artist,
                                  ms_played, length_ms, source, match_key)
             VALUES ('local', 'radio', '2026-09-18 12:00:00+00'::TIMESTAMPTZ,
                     'Advertisement', 'Some Station', 28800000, 0, 'stream',
                     match_key('Some Station', 'Advertisement'))",
            [],
        )
        .unwrap();
    // A normal four-minute play that knows its own length.
    a.conn()
        .execute(
            "INSERT INTO listens (origin, origin_key, played_at, title, artist,
                                  ms_played, length_ms, source, match_key)
             VALUES ('local', 'track', '2026-09-18 20:00:00+00'::TIMESTAMPTZ,
                     'Archangel', 'Burial', 240000, 240000, 'local',
                     match_key('Burial', 'Archangel'))",
            [],
        )
        .unwrap();
    a.refresh_dedup().unwrap();

    let overview = crate::query::overview(&a, crate::query::Scope::all()).unwrap();
    // 20 minutes for the uncapped stream row + 4 real minutes.
    assert!(
        (overview.hours_played - (20.0 + 4.0) / 60.0).abs() < 1e-6,
        "hours_played was {}",
        overview.hours_played
    );

    // The real play is not clipped: it is under the cap and knows its length.
    let hours: f64 = scalar(
        &a,
        "SELECT sum(ms_counted) / 3600000.0 FROM enriched_listens WHERE title = 'Archangel'",
    );
    assert!(
        (hours - 4.0 / 60.0).abs() < 1e-9,
        "a normal play is untouched"
    );
}

/// A play that overruns its own track length (a stream tagged with a duration,
/// a rounding artefact) contributes the track length, not the overrun.
#[test]
fn overruns_are_clamped_to_the_track_length() {
    let a = db();
    a.conn()
        .execute(
            "INSERT INTO listens (origin, origin_key, played_at, title, artist,
                                  ms_played, length_ms, source, match_key)
             VALUES ('local', '1', '2026-09-18 12:00:00+00'::TIMESTAMPTZ,
                     'Loop', 'Someone', 3600000, 180000, 'local',
                     match_key('Someone', 'Loop'))",
            [],
        )
        .unwrap();
    a.refresh_dedup().unwrap();

    let counted: i64 = scalar(&a, "SELECT ms_counted FROM enriched_listens");
    assert_eq!(
        counted, 180_000,
        "an hour on a three-minute track counts as three minutes"
    );
}

/// Radio announcements carry no duration and all look "skipped" when the
/// station moves on. Left in, station idents and adverts take every top place
/// with a 100% rate and bury the tracks actually being abandoned.
#[test]
fn skips_ignore_streams_with_no_duration() {
    let a = db();
    for i in 0..10 {
        a.conn()
            .execute(
                "INSERT INTO listens (origin, origin_key, played_at, title, artist,
                                      ms_played, length_ms, skipped, source, match_key)
                 VALUES ('local', ?, ?::TIMESTAMPTZ, 'Advertisement', 'Live365',
                         30000, 0, true, 'stream', match_key('Live365', 'Advertisement'))",
                duckdb::params![format!("r{i}"), format!("2026-09-18 1{i}:00:00+00")],
            )
            .unwrap();
    }
    // A real track, skipped half the time.
    for i in 0..4 {
        a.conn()
            .execute(
                "INSERT INTO listens (origin, origin_key, played_at, title, artist,
                                      ms_played, length_ms, skipped, source, match_key)
                 VALUES ('local', ?, ?::TIMESTAMPTZ, 'Memory', 'Windser',
                         20000, 200000, ?, 'local', match_key('Windser', 'Memory'))",
                duckdb::params![
                    format!("t{i}"),
                    format!("2026-09-19 1{i}:00:00+00"),
                    i % 2 == 0
                ],
            )
            .unwrap();
    }
    a.refresh_dedup().unwrap();

    let rows = crate::query::skips(&a, crate::query::Scope::all(), 20, 1).unwrap();
    assert!(
        !rows.iter().any(|r| r.name == "Advertisement"),
        "a station ident is not a skipped track"
    );
    let memory = rows
        .iter()
        .find(|r| r.name == "Memory")
        .expect("the real track");
    assert_eq!(memory.skips, 2);
    assert!((memory.skip_rate - 0.5).abs() < 1e-9);
    assert!(
        memory.median_completion.is_some(),
        "completion is computable"
    );
}

// ── local-library scope ─────────────────────────────────────────────────────

/// "In my library only" matches on the normalised title+artist key, not on the
/// listen carrying a local track id. A play imported from Spotify counts when
/// the same track sits in the library — the question is "my history, for music
/// I own", not "plays that went through this player".
#[test]
fn local_only_counts_imported_plays_of_owned_tracks() {
    let a = db();
    a.conn()
        .execute(
            "INSERT INTO tracks (track_id, title, artist, match_key)
             VALUES ('t1', 'Roygbiv', 'Boards of Canada',
                     match_key('Boards of Canada', 'Roygbiv'))",
            [],
        )
        .unwrap();

    // Owned, but the listen came from a Spotify export with no track id.
    insert(
        &a,
        "spotify",
        "s1",
        "2025-06-02 09:00:00+00",
        "Boards of Canada",
        "Roygbiv",
    );
    // Not in the library at all.
    insert(
        &a,
        "spotify",
        "s2",
        "2025-06-02 10:00:00+00",
        "Some Other Band",
        "Unowned",
    );
    a.refresh_dedup().unwrap();

    let everything = crate::query::Scope::all();
    let owned = crate::query::Scope {
        window: crate::Window::all(),
        local_only: true,
    };

    assert_eq!(crate::query::overview(&a, everything).unwrap().listens, 2);
    assert_eq!(
        crate::query::overview(&a, owned).unwrap().listens,
        1,
        "only the track the library has"
    );

    let top = crate::query::top(&a, crate::query::TopKind::Tracks, owned, 10).unwrap();
    assert_eq!(top.len(), 1);
    assert_eq!(top[0].name, "Roygbiv");
}

/// The two restrictions are independent and must compose.
#[test]
fn local_only_composes_with_the_window() {
    let a = db();
    a.conn()
        .execute(
            "INSERT INTO tracks (track_id, title, artist, match_key)
             VALUES ('t1', 'Roygbiv', 'Boards of Canada',
                     match_key('Boards of Canada', 'Roygbiv'))",
            [],
        )
        .unwrap();
    insert(
        &a,
        "local",
        "old",
        "2020-01-01 09:00:00+00",
        "Boards of Canada",
        "Roygbiv",
    );
    insert(
        &a,
        "local",
        "new",
        "2025-06-02 09:00:00+00",
        "Boards of Canada",
        "Roygbiv",
    );
    insert(
        &a,
        "local",
        "other",
        "2025-06-02 10:00:00+00",
        "Nobody",
        "Unowned",
    );
    a.refresh_dedup().unwrap();

    let since = chrono::DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z")
        .unwrap()
        .timestamp();
    let scope = crate::query::Scope {
        window: crate::Window {
            since: Some(since),
            until: None,
        },
        local_only: true,
    };
    assert_eq!(
        crate::query::overview(&a, scope).unwrap().listens,
        1,
        "owned, and inside the window"
    );
}

/// With nothing in the library the restricted view is empty rather than an
/// error — that is what the toggle shows before a scan has run.
#[test]
fn local_only_on_an_empty_library_is_empty() {
    let a = db();
    insert(
        &a,
        "spotify",
        "s1",
        "2025-06-02 09:00:00+00",
        "Someone",
        "A Song",
    );
    a.refresh_dedup().unwrap();

    let owned = crate::query::Scope {
        window: crate::Window::all(),
        local_only: true,
    };
    assert_eq!(crate::query::overview(&a, owned).unwrap().listens, 0);
    assert!(
        crate::query::top(&a, crate::query::TopKind::Artists, owned, 10)
            .unwrap()
            .is_empty()
    );
    assert!(crate::query::clock(&a, owned).unwrap().is_empty());
}

/// Internet radio records one row per announced song with an explicit zero
/// length. Those are station idents and adverts, not tracks; the catalogue
/// answers a 500 for them, which is not cached as a miss, so left in they are
/// re-asked on every run forever.
#[test]
fn enrichment_skips_zero_length_stream_rows() {
    let a = db();
    a.conn()
        .execute(
            "INSERT INTO listens (origin, origin_key, played_at, title, artist,
                                  ms_played, length_ms, source, match_key)
             VALUES ('local', 'r1', now(), 'Advertisement', 'Live365',
                     30000, 0, 'stream', match_key('Live365', 'Advertisement'))",
            [],
        )
        .unwrap();
    a.conn()
        .execute(
            "INSERT INTO listens (origin, origin_key, played_at, title, artist,
                                  ms_played, length_ms, source, match_key)
             VALUES ('spotify', 's1', now(), 'Xtal', 'Aphex Twin',
                     300000, NULL, 'spotify', match_key('Aphex Twin', 'Xtal'))",
            [],
        )
        .unwrap();

    let pending = crate::enrich::pending_for_test(&a, 100).unwrap();
    let titles: Vec<&str> = pending.iter().map(String::as_str).collect();
    assert_eq!(titles, ["Xtal"], "the stream row is not a track to resolve");
}

/// The catalogue answers the literal string "None" for a track it has no
/// genre for. Left alone it ranks as a genre, above several real ones.
#[test]
fn a_literal_none_genre_is_treated_as_missing() {
    let a = db();
    a.conn()
        .execute(
            "INSERT INTO song_match (match_key, resolved, genre, matched_at)
             VALUES (match_key('Someone', 'A Song'), true, 'None', now())",
            [],
        )
        .unwrap();
    insert(
        &a,
        "spotify",
        "s1",
        "2025-06-02 09:00:00+00",
        "Someone",
        "A Song",
    );
    a.refresh_dedup().unwrap();

    let genre: Option<String> = scalar(&a, "SELECT genre FROM enriched_listens");
    assert_eq!(genre, None, "\"None\" is the absence of a genre, not one");

    let top = crate::query::top(
        &a,
        crate::query::TopKind::Genres,
        crate::query::Scope::all(),
        10,
    )
    .unwrap();
    assert!(top.is_empty(), "and it does not rank");
}

// ── artist pictures ─────────────────────────────────────────────────────────

/// Enrichment already stores an artist picture per track; those are folded in
/// without asking the network at all.
#[test]
fn artist_pictures_come_from_enrichment_when_known() {
    let a = db();
    insert(
        &a,
        "spotify",
        "s1",
        "2025-06-02 09:00:00+00",
        "Boards of Canada",
        "Roygbiv",
    );
    a.conn()
        .execute(
            "INSERT INTO song_match (match_key, resolved, artist_picture, matched_at)
             VALUES (match_key('Boards of Canada', 'Roygbiv'), true, 'https://art/boc.jpg', now())",
            [],
        )
        .unwrap();
    a.refresh_dedup().unwrap();

    let pending = crate::artwork::pending_artists(&a, &["Boards of Canada".to_string()]).unwrap();
    assert!(pending.is_empty(), "nothing to look up: enrichment knew it");

    let picture: String = scalar(
        &a,
        "SELECT picture FROM artist_art WHERE artist_key = 'boards of canada'",
    );
    assert_eq!(picture, "https://art/boc.jpg");
}

/// An artist with no stored picture is queued with several of their tracks,
/// most-played first. The catalogue answers per track, so one entry having no
/// picture says nothing about the artist — giving up after the first would
/// cache a miss that is not true.
#[test]
fn artists_are_queued_with_several_candidate_titles() {
    let a = db();
    for (i, (title, plays)) in [
        ("Hypnotized", 5),
        ("Bad Company", 3),
        ("Body Funk", 2),
        ("Dopamine", 1),
    ]
    .iter()
    .enumerate()
    {
        for play in 0..*plays {
            insert(
                &a,
                "spotify",
                &format!("s{i}-{play}"),
                &format!("2025-06-0{} 09:0{}:00+00", i + 1, play),
                "Purple Disco Machine",
                title,
            );
        }
    }
    a.refresh_dedup().unwrap();

    let pending =
        crate::artwork::pending_artists(&a, &["Purple Disco Machine".to_string()]).unwrap();
    assert_eq!(pending.len(), 1);
    let (key, artist, titles) = &pending[0];
    assert_eq!(key, "purple disco machine");
    assert_eq!(artist, "Purple Disco Machine");
    assert_eq!(
        titles,
        &["Hypnotized", "Bad Company", "Body Funk"],
        "three most-played titles, most-played first"
    );
}

/// A miss is cached, so an artist the catalogue does not know is asked about
/// once rather than on every repaint.
#[test]
fn artist_misses_are_cached_and_not_retried() {
    let a = db();
    insert(
        &a,
        "spotify",
        "s1",
        "2025-06-02 09:00:00+00",
        "Nobody At All",
        "A Song",
    );
    a.refresh_dedup().unwrap();

    let names = vec!["Nobody At All".to_string()];
    assert_eq!(
        crate::artwork::pending_artists(&a, &names).unwrap().len(),
        1
    );

    crate::artwork::store_pictures(&a, &[("nobody at all".to_string(), None)]).unwrap();
    assert!(
        crate::artwork::pending_artists(&a, &names)
            .unwrap()
            .is_empty(),
        "a recorded miss is not asked about again"
    );

    // And it does not masquerade as a picture on the leaderboard.
    let top = crate::query::top(
        &a,
        crate::query::TopKind::Artists,
        crate::query::Scope::all(),
        5,
    )
    .unwrap();
    assert_eq!(top[0].art, None);
}

/// A stored picture reaches the leaderboard row that asked for it.
#[test]
fn a_resolved_picture_reaches_the_leaderboard() {
    let a = db();
    insert(
        &a,
        "spotify",
        "s1",
        "2025-06-02 09:00:00+00",
        "Four Tet",
        "Two Thousand and Seventeen",
    );
    a.refresh_dedup().unwrap();
    crate::artwork::store_pictures(
        &a,
        &[(
            "four tet".to_string(),
            Some("https://art/ft.jpg".to_string()),
        )],
    )
    .unwrap();

    let top = crate::query::top(
        &a,
        crate::query::TopKind::Artists,
        crate::query::Scope::all(),
        5,
    )
    .unwrap();
    assert_eq!(top[0].name, "Four Tet");
    assert_eq!(top[0].art.as_deref(), Some("https://art/ft.jpg"));
}

/// Tracks wear the album cover the catalogue resolved, where the local
/// library has none of its own.
#[test]
fn tracks_fall_back_to_the_catalogue_cover() {
    let a = db();
    insert(
        &a,
        "spotify",
        "s1",
        "2025-06-02 09:00:00+00",
        "Burial",
        "Archangel",
    );
    a.conn()
        .execute(
            "INSERT INTO song_match (match_key, resolved, album_art, matched_at)
             VALUES (match_key('Burial', 'Archangel'), true, 'https://art/untrue.jpg', now())",
            [],
        )
        .unwrap();
    a.refresh_dedup().unwrap();

    let top = crate::query::top(
        &a,
        crate::query::TopKind::Tracks,
        crate::query::Scope::all(),
        5,
    )
    .unwrap();
    assert_eq!(top[0].art.as_deref(), Some("https://art/untrue.jpg"));
}

/// A local cover wins over the catalogue's: it is the file the user actually
/// has, and the cover server can serve it without the network.
#[test]
fn a_local_cover_wins_over_the_catalogue() {
    let a = db();
    a.conn()
        .execute(
            "INSERT INTO tracks (track_id, title, artist, cover, match_key)
             VALUES ('t1', 'Archangel', 'Burial', 'local-cover.jpg',
                     match_key('Burial', 'Archangel'))",
            [],
        )
        .unwrap();
    a.conn()
        .execute(
            "INSERT INTO listens (origin, origin_key, played_at, track_id, title, artist,
                                  ms_played, length_ms, source, match_key)
             VALUES ('local', '1', now(), 't1', 'Archangel', 'Burial', 240000, 240000,
                     'local', match_key('Burial', 'Archangel'))",
            [],
        )
        .unwrap();
    a.conn()
        .execute(
            "INSERT INTO song_match (match_key, resolved, album_art, matched_at)
             VALUES (match_key('Burial', 'Archangel'), true, 'https://art/remote.jpg', now())",
            [],
        )
        .unwrap();
    a.refresh_dedup().unwrap();

    let top = crate::query::top(
        &a,
        crate::query::TopKind::Tracks,
        crate::query::Scope::all(),
        5,
    )
    .unwrap();
    assert_eq!(top[0].art.as_deref(), Some("local-cover.jpg"));
}

/// The album steers a match toward the right release, and when it disagrees
/// with the catalogue's spelling it does not merely fail to help — it narrows
/// the search until nothing matches. `Good Ones` by Charli xcx resolves with
/// no album and misses with `CRASH`, the album an export holds for it. So a
/// miss with an album must be asked again without one.
#[tokio::test]
async fn an_album_that_does_not_match_is_dropped_and_retried() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let seen = Arc::new(AtomicUsize::new(0));
    let albums = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));

    // A catalogue that only answers when no album is given.
    let (addr, shutdown) = {
        let seen = Arc::clone(&seen);
        let albums = Arc::clone(&albums);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();
        tokio::spawn(async move {
            loop {
                let accept = tokio::select! {
                    a = listener.accept() => a,
                    _ = &mut rx => break,
                };
                let Ok((mut stream, _)) = accept else { break };
                let seen = Arc::clone(&seen);
                let albums = Arc::clone(&albums);
                tokio::spawn(async move {
                    use tokio::io::{AsyncReadExt, AsyncWriteExt};
                    let mut buf = vec![0u8; 4096];
                    let n = stream.read(&mut buf).await.unwrap_or(0);
                    let request = String::from_utf8_lossy(&buf[..n]).to_string();
                    seen.fetch_add(1, Ordering::SeqCst);

                    let album = request
                        .split("album=")
                        .nth(1)
                        .and_then(|rest| rest.split([' ', '&']).next())
                        .unwrap_or("")
                        .to_string();
                    albums.lock().unwrap().push(album.clone());

                    let body = if album.is_empty() {
                        r#"{"title":"Good Ones","artist":"Charli xcx","albumArt":"https://art/x.jpg"}"#
                    } else {
                        "{}"
                    };
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                });
            }
        });
        (addr, tx)
    };

    std::env::set_var("ROCKSKY_API_URL", format!("http://{addr}"));

    let a = db();
    a.conn()
        .execute(
            "INSERT INTO listens (origin, origin_key, played_at, title, artist, album,
                                  ms_played, length_ms, source, match_key)
             VALUES ('spotify', 's1', now(), 'Good Ones', 'Charli xcx', 'CRASH (Deluxe)',
                     200000, 200000, 'spotify', match_key('Charli xcx', 'Good Ones'))",
            [],
        )
        .unwrap();
    a.refresh_dedup().unwrap();

    let report = crate::enrich::enrich(&a, 10, false, &crate::progress::Silent)
        .await
        .unwrap();
    let _ = shutdown.send(());
    std::env::remove_var("ROCKSKY_API_URL");

    assert_eq!(report.resolved, 1, "the retry without an album finds it");
    let asked = albums.lock().unwrap().clone();
    assert_eq!(asked.len(), 2, "asked twice: with the album, then without");
    assert!(!asked[0].is_empty(), "the album is tried first");
    assert!(asked[1].is_empty(), "then dropped");

    let art: String = scalar(&a, "SELECT album_art FROM song_match");
    assert_eq!(art, "https://art/x.jpg");
}
