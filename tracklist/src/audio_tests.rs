use super::*;
use music_player_types::source::{RemoteIdentity, ResourceKind, SourceRef};

fn episode() -> Track {
    let handle = SourceRef {
        resolver: "emby".into(),
        account_id: "saved-family".into(),
        remote: RemoteIdentity {
            server_id: "server-a".into(),
            user_id: "family".into(),
        },
        kind: ResourceKind::Item,
        item_id: "55508".into(),
    }
    .to_handle();
    Track {
        id: handle.clone(),
        uri: handle,
        title: "Episode".into(),
        ..Default::default()
    }
}

fn choice(index: i32) -> AudioSelection {
    AudioSelection::Explicit {
        media_source_id: "version-a".into(),
        audio_stream_index: index,
    }
}

#[test]
fn selected_next_preserves_current_and_duplicates_and_rejects_before_insertion() {
    let mut queue = Tracklist::new(vec![episode(), episode()]);
    queue.next_track();
    let current = queue.current_entry().unwrap();
    let original_next = queue.entries().1;
    let inserted = queue
        .insert_next_with_selection(episode(), choice(0))
        .unwrap();
    assert_eq!(queue.current_entry().unwrap(), current);
    let upcoming = queue.entries().1;
    assert_eq!(upcoming[0].occurrence_id, inserted);
    assert_ne!(inserted, current.occurrence_id);
    assert_eq!(upcoming[0].selection, choice(0));
    assert_eq!(&upcoming[1..], original_next.as_slice());
    let before = queue.entries();
    assert!(queue
        .insert_next_with_selection(episode(), choice(-1))
        .is_err());
    assert_eq!(queue.entries(), before);
    queue.next_track();
    assert_eq!(queue.current_entry().unwrap().occurrence_id, inserted);
    assert_eq!(queue.current_entry().unwrap().selection, choice(0));
}

fn pin(index: i32) -> AudioPin {
    AudioPin {
        media_source_id: "version-a".into(),
        audio_stream_index: index,
        runtime_ticks: Some(191429666670),
        etag: Some("revision-a".into()),
        codec: Some("aac".into()),
        channels: Some(2),
        sample_rate: Some(44100),
    }
}

#[test]
fn legacy_entries_keep_browse_tags_and_scope_and_reject_choices_atomically() {
    let handle = music_player_types::source::LocalSourceRef::for_extension(
        "legacy.fixture",
        "saved-account".into(),
        7,
        ResourceKind::Item,
        "item".into(),
    )
    .unwrap()
    .to_handle();
    let track = Track {
        id: handle.clone(),
        uri: handle.clone(),
        title: "Actual browse title".into(),
        artist: "Actual artist".into(),
        ..Default::default()
    };
    let mut queue = Tracklist::new_empty();
    let ids = queue
        .queue_with_selection(vec![(track.clone(), AudioSelection::Auto)])
        .unwrap();
    queue.next_track().unwrap();
    let before = queue.entries();
    let current = queue.current_entry().unwrap();
    let restored: QueueEntry =
        serde_json::from_str(&serde_json::to_string(&current).unwrap()).unwrap();
    restored.validate().unwrap();
    assert_eq!(restored.track.id, handle);
    assert_eq!(restored.track.title, "Actual browse title");
    assert_eq!(restored.track.artist, "Actual artist");
    for selection in [AudioSelection::Auto, choice(1)] {
        assert!(queue.select_audio(&ids[0], selection).is_err());
        assert_eq!(queue.entries(), before);
    }
    assert!(queue.accept_current_pin(&ids[0], pin(0)).is_err());
    assert!(queue
        .queue_with_selection(vec![(episode(), AudioSelection::Auto), (track, choice(1))])
        .is_err());
    assert_eq!(queue.entries(), before);
    let mut wrong_scope = restored;
    wrong_scope.track.uri = wrong_scope.track.uri.replace("revision=7", "revision=8");
    assert!(wrong_scope.validate().is_err());
}

#[test]
fn repeated_episode_keeps_independent_choices_through_navigation_and_shuffle() {
    let mut queue = Tracklist::new_empty();
    let ids = queue
        .queue_with_selection(vec![(episode(), choice(0)), (episode(), choice(3))])
        .unwrap();
    assert_ne!(ids[0], ids[1]);
    queue.next_track().unwrap();
    queue.accept_current_pin(&ids[0], pin(0)).unwrap();
    queue.next_track().unwrap();
    assert_eq!(
        queue.current_entry().unwrap().effective_selection(),
        choice(3)
    );
    queue.accept_current_pin(&ids[1], pin(3)).unwrap();
    queue.previous_track().unwrap();
    assert_eq!(queue.current_entry().unwrap().occurrence_id, ids[0]);
    assert_eq!(
        queue.current_entry().unwrap().effective_selection(),
        AudioSelection::Pinned(pin(0))
    );
    queue.play_track_at(0);
    queue
        .queue_with_selection(vec![(episode(), choice(4)), (episode(), choice(5))])
        .unwrap();
    let (history, mut expected) = queue.entries();
    queue.shuffle();
    let (after_history, mut actual) = queue.entries();
    assert_eq!(after_history, history);
    expected.sort_by(|a, b| a.occurrence_id.cmp(&b.occurrence_id));
    actual.sort_by(|a, b| a.occurrence_id.cmp(&b.occurrence_id));
    assert_eq!(actual, expected);
    let second = actual
        .iter()
        .find(|entry| entry.occurrence_id == ids[1])
        .unwrap();
    assert_eq!(second.effective_selection(), AudioSelection::Pinned(pin(3)));
}

#[test]
fn late_pin_cannot_change_an_identical_next_occurrence() {
    let mut queue = Tracklist::new_empty();
    let ids = queue
        .queue_with_selection(vec![(episode(), choice(0)), (episode(), choice(0))])
        .unwrap();
    queue.next_track();
    queue.next_track();
    let before = queue.entries();
    assert!(queue.accept_current_pin(&ids[0], pin(0)).is_err());
    assert_eq!(queue.entries(), before);
    assert_eq!(queue.current_entry().unwrap().pin, None);
    assert!(queue.accept_current_pin(&ids[1], pin(3)).is_err());
    assert_eq!(queue.entries(), before);
}

#[test]
fn changing_one_occurrence_clears_only_its_pin() {
    let mut queue = Tracklist::new_empty();
    let ids = queue
        .queue_with_selection(vec![(episode(), choice(0)), (episode(), choice(0))])
        .unwrap();
    queue.next_track();
    queue.accept_current_pin(&ids[0], pin(0)).unwrap();
    queue.next_track();
    queue.accept_current_pin(&ids[1], pin(0)).unwrap();
    assert!(!queue.select_audio(&ids[0], choice(3)).unwrap());
    assert_eq!(queue.entries().0[0].effective_selection(), choice(3));
    assert_eq!(
        queue.current_entry().unwrap().effective_selection(),
        AudioSelection::Pinned(pin(0))
    );
    assert!(queue.select_audio(&ids[1], AudioSelection::Auto).unwrap());
    assert_eq!(
        queue.current_entry().unwrap().effective_selection(),
        AudioSelection::Auto
    );
    assert_eq!(queue.entries().0[1], queue.current_entry().unwrap());
}

#[test]
fn invalid_batch_or_snapshot_leaves_existing_queue_intact() {
    let mut queue = Tracklist::new(vec![episode()]);
    queue.next_track();
    let before = queue.entries();
    let current = queue.current_entry();
    assert!(queue
        .queue_with_selection(vec![(episode(), choice(0)), (episode(), choice(-1))])
        .is_err());
    assert_eq!(queue.entries(), before);
    let duplicate = before.0[0].clone();
    assert!(queue
        .restore_entries(vec![duplicate.clone()], vec![duplicate], 9000)
        .is_err());
    assert_eq!(queue.entries(), before);
    assert_eq!(queue.current_entry(), current);
    assert_eq!(queue.playback_state().position_ms, 0);
    assert!(queue
        .select_audio(
            &current.unwrap().occurrence_id,
            AudioSelection::Pinned(pin(0))
        )
        .is_err());
    assert_eq!(queue.entries(), before);
}

#[test]
fn restoring_entries_preserves_choices_pins_and_paused_position() {
    let mut queue = Tracklist::new_empty();
    let ids = queue
        .queue_with_selection(vec![(episode(), choice(0)), (episode(), choice(3))])
        .unwrap();
    queue.next_track();
    queue.accept_current_pin(&ids[0], pin(0)).unwrap();
    let (played, upcoming) = queue.entries();
    let mut restored = Tracklist::new_empty();
    restored
        .restore_entries(played.clone(), upcoming.clone(), 7200123)
        .unwrap();
    assert_eq!(restored.entries(), (played, upcoming));
    assert_eq!(restored.current_entry(), queue.current_entry());
    assert_eq!(
        restored.playback_state(),
        PlaybackState {
            position_ms: 7200123,
            is_playing: false
        }
    );
    restored.next_track();
    assert_eq!(restored.current_entry().unwrap().occurrence_id, ids[1]);
    assert_eq!(
        restored.current_entry().unwrap().effective_selection(),
        choice(3)
    );
}

#[test]
fn previous_after_removing_current_returns_nearest_retained_occurrence() {
    for count in [2, 3] {
        let mut queue = Tracklist::new_empty();
        let ids = queue
            .queue_with_selection(
                (0..count)
                    .map(|index| (episode(), choice(index as i32)))
                    .collect(),
            )
            .unwrap();
        for (index, id) in ids.iter().enumerate() {
            queue.next_track();
            queue.accept_current_pin(id, pin(index as i32)).unwrap();
        }
        queue.remove_track_at(count - 1);
        assert_eq!(queue.current_entry().unwrap().occurrence_id, ids[count - 1]);
        queue.previous_track().unwrap();
        assert_eq!(queue.current_entry().unwrap().occurrence_id, ids[count - 2]);
        assert_eq!(
            queue.current_entry().unwrap().effective_selection(),
            AudioSelection::Pinned(pin((count - 2) as i32))
        );
        assert!(
            queue.entries().1.is_empty(),
            "removed current must not be queued again"
        );
    }
}
