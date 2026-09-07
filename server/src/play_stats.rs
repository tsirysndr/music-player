//! Local play counts, for smart playlists.
//!
//! Separate from the scrobbler on purpose: scrobbling needs a Rocksky account
//! and the network, while "played more than five times" has to work on a laptop
//! that has never been signed in to anything. Both use the same threshold — half
//! the track or four minutes — so the two counts agree when both are on.
//!
//! A track that is passed over before that threshold is counted as a skip
//! instead, which is what makes a "songs I keep skipping" filter possible.

use std::sync::{Arc, Mutex};

use music_player_entity::track_stats;
use music_player_tracklist::Tracklist;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};

use crate::scrobbler::{current_track, unix_now, Current, Watcher, TICK};

/// Start recording play statistics. Runs until the process exits.
pub fn spawn(tracklist: Arc<Mutex<Tracklist>>) {
    tokio::spawn(record_loop(tracklist));
}

async fn record_loop(tracklist: Arc<Mutex<Tracklist>>) {
    let db = music_player_storage::shared().await;
    let conn = db.get_connection().clone();
    let mut watcher = Watcher::default();
    // The play in progress: remembered so that when it is replaced without ever
    // crossing the threshold, it can be counted as a skip.
    let mut pending: Option<Pending> = None;

    loop {
        tokio::time::sleep(TICK).await;

        let Some(current) = current_track(&tracklist) else {
            // Playback stopped. Whatever was in flight was not finished, but a
            // deliberate stop is not a skip either — drop it.
            watcher.key = None;
            pending = None;
            continue;
        };

        let counted = watcher.advance(&current);
        if let Some(previous) = pending.take() {
            if previous.key != current.key {
                // A new play started while the old one was still short of the
                // threshold: the user moved on.
                if let Err(e) = bump(&conn, &previous.id, Kind::Skip).await {
                    tracing::debug!(track = %previous.id, "could not record a skip: {e}");
                }
            } else {
                pending = Some(previous);
            }
        }

        if counted {
            if let Err(e) = bump(&conn, &current.id, Kind::Play).await {
                tracing::debug!(track = %current.id, "could not record a play: {e}");
            }
            pending = None;
            continue;
        }
        if pending.is_none() && !current.id.is_empty() {
            pending = Some(Pending::of(&current));
        }
    }
}

struct Pending {
    key: String,
    id: String,
}

impl Pending {
    fn of(current: &Current) -> Self {
        Self {
            key: current.key.clone(),
            id: current.id.clone(),
        }
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Play,
    Skip,
}

/// Add one to a track's play or skip count, creating the row on first sight.
async fn bump(conn: &DatabaseConnection, track_id: &str, kind: Kind) -> Result<(), anyhow::Error> {
    if track_id.is_empty() {
        return Ok(());
    }
    let now = unix_now();
    let existing = track_stats::Entity::find_by_id(track_id.to_owned())
        .one(conn)
        .await?;
    match existing {
        Some(row) => {
            let mut update = track_stats::ActiveModel {
                track_id: ActiveValue::Unchanged(row.track_id),
                updated_at: ActiveValue::Set(now),
                ..Default::default()
            };
            match kind {
                Kind::Play => {
                    update.play_count = ActiveValue::Set(row.play_count + 1);
                    update.last_played = ActiveValue::Set(Some(now));
                }
                Kind::Skip => {
                    update.skip_count = ActiveValue::Set(row.skip_count + 1);
                    update.last_skipped = ActiveValue::Set(Some(now));
                }
            }
            update.update(conn).await?;
        }
        None => {
            let (play_count, skip_count, last_played, last_skipped) = match kind {
                Kind::Play => (1, 0, Some(now), None),
                Kind::Skip => (0, 1, None, Some(now)),
            };
            track_stats::ActiveModel {
                track_id: ActiveValue::Set(track_id.to_owned()),
                play_count: ActiveValue::Set(play_count),
                skip_count: ActiveValue::Set(skip_count),
                last_played: ActiveValue::Set(last_played),
                last_skipped: ActiveValue::Set(last_skipped),
                updated_at: ActiveValue::Set(now),
            }
            .insert(conn)
            .await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;

    async fn temp_db() -> (tempfile::TempDir, DatabaseConnection) {
        use migration::{Migrator, MigratorTrait};
        let dir = tempfile::tempdir().unwrap();
        let url = format!(
            "sqlite://{}?mode=rwc",
            dir.path().join("music-player.sqlite3").display()
        );
        let conn = Database::connect(&url).await.unwrap();
        Migrator::up(&conn, None).await.unwrap();
        (dir, conn)
    }

    async fn stats(conn: &DatabaseConnection, id: &str) -> track_stats::Model {
        track_stats::Entity::find_by_id(id.to_owned())
            .one(conn)
            .await
            .unwrap()
            .unwrap()
    }

    #[tokio::test]
    async fn counts_plays_and_skips_separately() {
        let (_dir, conn) = temp_db().await;

        bump(&conn, "t1", Kind::Play).await.unwrap();
        bump(&conn, "t1", Kind::Play).await.unwrap();
        bump(&conn, "t1", Kind::Skip).await.unwrap();

        let row = stats(&conn, "t1").await;
        assert_eq!(row.play_count, 2);
        assert_eq!(row.skip_count, 1);
        assert!(row.last_played.is_some());
        assert!(row.last_skipped.is_some());
    }

    /// Incrementing a skip must not disturb the play count, or a track skipped
    /// once would look like it had never been played.
    #[tokio::test]
    async fn a_skip_leaves_the_play_count_alone() {
        let (_dir, conn) = temp_db().await;
        bump(&conn, "t1", Kind::Play).await.unwrap();
        bump(&conn, "t1", Kind::Skip).await.unwrap();
        let row = stats(&conn, "t1").await;
        assert_eq!(row.play_count, 1);
        assert!(row.last_played.is_some());
    }

    #[tokio::test]
    async fn ignores_tracks_with_no_id() {
        let (_dir, conn) = temp_db().await;
        bump(&conn, "", Kind::Play).await.unwrap();
        assert_eq!(
            track_stats::Entity::find().all(&conn).await.unwrap().len(),
            0
        );
    }
}
