use super::*;
use music_player_storage::Database;
use music_player_types::audio::AudioPin;
use music_player_types::source::{RemoteIdentity, SourceRef};

async fn playback() -> SourcePlayback {
    let connection = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
    SourcePlayback::new(SourceResolver::new(
        Database { connection },
        "fixture".into(),
        true,
    ))
}

fn source(id: &str) -> SourceHandle {
    SourceHandle::Remote(SourceRef {
        resolver: "emby".into(),
        account_id: "saved-account".into(),
        remote: RemoteIdentity {
            server_id: "server-a".into(),
            user_id: "family-id".into(),
        },
        kind: ResourceKind::Item,
        item_id: id.into(),
    })
}

fn roundtrip(value: SourceCheckpoint) -> SourceCheckpoint {
    let raw = serde_json::to_string(&value).unwrap();
    // No authenticated request/transport state belongs in this document.
    assert!(!raw.contains("http:"));
    assert!(!raw.contains("headers"));
    serde_json::from_str(&raw).unwrap()
}

fn legacy_source(revision: u64) -> SourceHandle {
    music_player_types::source::LocalSourceRef::for_extension(
        "legacy.fixture",
        "local-account".into(),
        revision,
        ResourceKind::Item,
        "item".into(),
    )
    .unwrap()
    .into()
}

#[tokio::test]
async fn legacy_load_knows_no_seek_before_resolution_and_rejects_invalid_replacement() {
    let mut player = playback().await;
    player
        .load(legacy_source(1), Duration::ZERO, false)
        .unwrap();
    let checkpoint = roundtrip(player.checkpoint().unwrap());
    assert_eq!(checkpoint.capabilities, Some(legacy_capabilities()));
    for target in [Duration::ZERO, Duration::from_millis(1200)] {
        assert_eq!(player.seek(target), Err(ControlError::SeekUnsupported));
        assert_eq!(player.checkpoint().unwrap().offset_ms, 0);
        assert!(player.task.is_none());
    }
    for (selection, target) in [
        (AudioSelection::Auto, Duration::from_millis(1)),
        (
            AudioSelection::Explicit {
                media_source_id: "version".into(),
                audio_stream_index: 0,
            },
            Duration::ZERO,
        ),
    ] {
        assert!(player
            .load_occurrence(
                legacy_source(2),
                uuid::Uuid::new_v4().to_string(),
                selection,
                target,
                true
            )
            .is_err());
        let unchanged = player.checkpoint().unwrap();
        assert_eq!(unchanged.occurrence_id, checkpoint.occurrence_id);
        assert_eq!(unchanged.source, checkpoint.source);
        assert!(player.task.is_none());
    }
}

#[tokio::test]
async fn legacy_checkpoint_retains_revision_and_position_without_silent_restart() {
    let mut player = playback().await;
    player
        .load(legacy_source(3), Duration::ZERO, false)
        .unwrap();
    let mut checkpoint = roundtrip(player.checkpoint().unwrap());
    checkpoint.offset_ms = 7_200_123;
    // A missing capabilities field does not grant seeking to a legacy source.
    checkpoint.capabilities = None;
    let mut restored = playback().await;
    restored
        .restore(legacy_source(3), checkpoint.clone())
        .unwrap();
    assert_eq!(restored.snapshot().target, Duration::from_millis(7_200_123));
    assert_eq!(restored.play(), Err(ControlError::ResumeUnsupported));
    assert_eq!(restored.snapshot().desired, DesiredState::Paused);
    assert!(restored.task.is_none());
    assert_eq!(
        restored.checkpoint().unwrap().source,
        legacy_source(3).to_handle()
    );
    assert!(restored
        .restore(legacy_source(4), checkpoint.clone())
        .is_err());
    let mut incompatible = checkpoint.clone();
    incompatible.capabilities = Some(PlaybackCapabilities {
        timeline: Timeline::Unknown,
        seek_policy: SeekPolicy::ServerOffset,
    });
    assert!(restored.restore(legacy_source(3), incompatible).is_err());
    checkpoint.version = 1;
    assert!(restored.restore(legacy_source(3), checkpoint).is_err());
    assert_eq!(restored.checkpoint().unwrap().offset_ms, 7_200_123);
}

#[tokio::test]
async fn pending_seek_survives_restart_before_first_resolution_without_network() {
    let mut first = playback().await;
    first.load(source("55508"), Duration::ZERO, false).unwrap();
    first.seek(Duration::from_millis(7_200_123)).unwrap();
    let checkpoint = roundtrip(first.checkpoint().unwrap());
    assert!(matches!(checkpoint.selection, AudioSelection::Auto));
    let mut restored = playback().await;
    restored.restore(source("55508"), checkpoint).unwrap();
    assert_eq!(restored.snapshot().desired, DesiredState::Paused);
    assert_eq!(restored.snapshot().target, Duration::from_millis(7_200_123));
    assert!(restored.task.is_none());
    assert!(restored.snapshot().position.is_none());
    assert!(restored.completed.try_recv().is_err());
}

#[tokio::test]
async fn pinned_selection_and_u64_position_survive_restore_seek_and_stop() {
    let pin = AudioPin {
        media_source_id: "version-a".into(),
        audio_stream_index: 1,
        runtime_ticks: Some(100_000_000_000_000),
        etag: Some("revision-a".into()),
        codec: Some("aac".into()),
        channels: Some(2),
        sample_rate: Some(44100),
    };
    let position = u64::from(u32::MAX) + 7_200_123;
    let checkpoint = SourceCheckpoint {
        version: 1,
        capabilities: None,
        occurrence_id: uuid::Uuid::new_v4().to_string(),
        source: source("55508").to_handle(),
        offset_ms: position,
        selection: AudioSelection::Pinned(pin.clone()),
    };
    let mut player = playback().await;
    player
        .restore(source("55508"), roundtrip(checkpoint))
        .unwrap();
    assert_eq!(player.snapshot().target.as_millis(), u128::from(position));
    player.seek(Duration::from_millis(position + 321)).unwrap();
    player.stop();
    let saved = roundtrip(player.checkpoint().unwrap());
    assert_eq!(saved.offset_ms, position + 321);
    assert_eq!(saved.selection, AudioSelection::Pinned(pin));
    assert_eq!(player.snapshot().desired, DesiredState::Stopped);
    assert!(player.task.is_none());
}

#[tokio::test]
async fn incompatible_checkpoint_cannot_replace_the_current_occurrence() {
    let mut player = playback().await;
    player
        .load(source("current"), Duration::ZERO, false)
        .unwrap();
    let original = player.checkpoint().unwrap();
    for case in 0..3 {
        let mut invalid = original.clone();
        match case {
            0 => invalid.source = source("other").to_handle(),
            1 => invalid.version = 99,
            _ => invalid.occurrence_id = "invalid-uuid".into(),
        }
        assert!(player.restore(source("current"), invalid).is_err());
        assert_eq!(
            player.checkpoint().unwrap().occurrence_id,
            original.occurrence_id
        );
    }
}

#[tokio::test]
async fn restored_no_seek_checkpoint_rejects_controls_without_losing_position() {
    let mut player = playback().await;
    player.load(source("55508"), Duration::ZERO, false).unwrap();
    let mut checkpoint = player.checkpoint().unwrap();
    checkpoint.offset_ms = 7_200_123;
    checkpoint.capabilities = Some(PlaybackCapabilities {
        timeline: Timeline::Unknown,
        seek_policy: SeekPolicy::None,
    });
    player
        .restore(source("55508"), roundtrip(checkpoint.clone()))
        .unwrap();
    assert!(player.seek(Duration::ZERO).is_err());
    assert!(player.play().is_err());
    let saved = roundtrip(player.checkpoint().unwrap());
    assert_eq!(saved.version, 2);
    assert_eq!(saved.offset_ms, checkpoint.offset_ms);
    assert_eq!(saved.occurrence_id, checkpoint.occurrence_id);
    assert_eq!(saved.capabilities, checkpoint.capabilities);
    assert_eq!(player.snapshot().desired, DesiredState::Paused);
    assert!(player.task.is_none());
    assert!(player.completed.try_recv().is_err());
}

#[tokio::test]
async fn legacy_unknown_duration_checkpoint_keeps_server_offset_seek() {
    let mut player = playback().await;
    player.load(source("55508"), Duration::ZERO, false).unwrap();
    let mut checkpoint = player.checkpoint().unwrap();
    checkpoint.version = 1;
    checkpoint.capabilities = None;
    checkpoint.offset_ms = 7_200_123;
    player
        .restore(source("55508"), roundtrip(checkpoint))
        .unwrap();
    player.seek(Duration::from_millis(7_200_456)).unwrap();
    let saved = roundtrip(player.checkpoint().unwrap());
    assert_eq!(saved.version, 2);
    assert_eq!(saved.offset_ms, 7_200_456);
    assert_eq!(
        saved.capabilities,
        Some(PlaybackCapabilities {
            timeline: Timeline::Unknown,
            seek_policy: SeekPolicy::ServerOffset,
        })
    );
    assert!(player.task.is_none());
}

#[tokio::test]
async fn repeated_item_is_a_new_occurrence_and_old_error_does_not_replace_it() {
    let mut player = playback().await;
    player.load(source("55508"), Duration::ZERO, false).unwrap();
    // Supply controlled late completions on the actual bounded mailbox. No
    // engine may start for an error, and no network timing is needed here.
    let old_id = player.occurrence.as_ref().unwrap().id.clone();
    let old = player.coordinator.play().unwrap().unwrap();
    player.load(source("55508"), Duration::ZERO, false).unwrap();
    assert_ne!(old_id, player.occurrence.as_ref().unwrap().id);
    let current = player.coordinator.play().unwrap().unwrap();
    assert!(old.is_cancelled());
    assert!(player
        .sender
        .try_send((old, Err(ProviderError::Other("old failure".into()))))
        .is_ok());
    player.poll_with(|_| panic!("error completion attempted to open a stream"));
    assert_eq!(player.snapshot().phase, Phase::Resolving);
    assert!(player.error().is_none());
    assert!(player
        .sender
        .try_send((current, Err(ProviderError::Other("current failure".into()))))
        .is_ok());
    player.poll_with(|_| panic!("error completion attempted to open a stream"));
    assert_eq!(player.snapshot().phase, Phase::Failed);
    assert_eq!(player.error(), Some("current failure"));
}

// Exercises the real mailbox/coordinator bridge, not decoding or listening.
// The unchanged HTTP -> PCM fixture remains the native engine evidence.
struct BridgeSession;
impl SessionControl for BridgeSession {
    fn cancel(&self) {}
    fn output(&self) -> rockbox_playback::StreamOutputSnapshot {
        rockbox_playback::StreamOutputSnapshot {
            generation: 7,
            boundary: rockbox_playback::StreamOutputBoundary::ByteStream,
            frames: 44100,
            sample_rate: 44100,
        }
    }
    fn phase(&self) -> rockbox_playback::StreamPhase {
        rockbox_playback::StreamPhase::Opening
    }
    fn generation(&self) -> u64 {
        7
    }
    fn input_released(&self) -> bool {
        false
    }
}

async fn bridge_player() -> SourcePlayback<BridgeSession> {
    bridge_player_with_session().await
}

async fn bridge_player_with_session<S: SessionControl>() -> SourcePlayback<S> {
    let resolver = playback().await.resolver.clone();
    let (sender, completed) = mpsc::channel(1);
    SourcePlayback {
        resolver,
        coordinator: Coordinator::default(),
        occurrence: None,
        selection: AudioSelection::Auto,
        task: None,
        completed,
        sender,
        error: None,
    }
}

#[tokio::test]
async fn rejected_seek_keeps_bridge_session_task_and_lease_alive() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    struct CountedSession {
        cancelled: Arc<AtomicUsize>,
        frames: Arc<AtomicUsize>,
    }
    impl SessionControl for CountedSession {
        fn cancel(&self) {
            self.cancelled.fetch_add(1, Ordering::SeqCst);
        }
        fn output(&self) -> rockbox_playback::StreamOutputSnapshot {
            let mut output = BridgeSession.output();
            output.frames = self.frames.load(Ordering::SeqCst) as u64;
            output
        }
        fn phase(&self) -> rockbox_playback::StreamPhase {
            BridgeSession.phase()
        }
        fn generation(&self) -> u64 {
            7
        }
        fn input_released(&self) -> bool {
            false
        }
    }
    let mut player = bridge_player_with_session::<CountedSession>().await;
    player.load(source("55508"), Duration::ZERO, false).unwrap();
    let request = player.coordinator.play().unwrap().unwrap();
    let released = Arc::new(AtomicUsize::new(0));
    let cancelled = Arc::new(AtomicUsize::new(0));
    let frames = Arc::new(AtomicUsize::new(44100));
    let mut descriptor = bridge_descriptor(None, released.clone());
    descriptor.requested_offset_ms = 0;
    descriptor.start = StartPosition::RequestedOnly;
    descriptor.capabilities.seek_policy = SeekPolicy::None;
    assert!(player
        .sender
        .try_send((request.clone(), Ok(descriptor)))
        .is_ok());
    player.poll_with(|_| {
        Ok(CountedSession {
            cancelled: cancelled.clone(),
            frames: frames.clone(),
        })
    });
    let before = player.snapshot();
    assert_eq!(before.phase, Phase::Streaming);
    // Stand in for a still-running resolver epilogue owned by this attempt.
    let (finish, finished) = tokio::sync::oneshot::channel::<()>();
    player.task = Some(tokio::spawn(async move { finished.await.unwrap() }));
    let task_id = player.task.as_ref().unwrap().id();
    for target in [Duration::ZERO, Duration::from_secs(7200)] {
        assert_eq!(player.seek(target), Err(ControlError::SeekUnsupported));
        assert_eq!(player.snapshot(), before);
        assert_eq!(player.task.as_ref().unwrap().id(), task_id);
        assert!(!player.task.as_ref().unwrap().is_finished());
        assert!(!request.is_cancelled());
        assert_eq!(cancelled.load(Ordering::SeqCst), 0);
        assert_eq!(released.load(Ordering::SeqCst), 0);
    }
    finish.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(1), player.task.take().unwrap())
        .await
        .expect("retained task did not finish")
        .expect("rejected seek aborted the retained task");
    frames.store(88200, Ordering::SeqCst);
    player.poll_with(|_| panic!("rejected seek started another session"));
    assert_eq!(
        player.snapshot().position.unwrap().absolute,
        Duration::from_secs(2)
    );
    player.stop();
    assert_eq!(cancelled.load(Ordering::SeqCst), 1);
    assert_eq!(released.load(Ordering::SeqCst), 1);
    drop(player);
    assert_eq!(cancelled.load(Ordering::SeqCst), 1);
    assert_eq!(released.load(Ordering::SeqCst), 1);
}

fn bridge_descriptor(
    pin: Option<&str>,
    released: std::sync::Arc<std::sync::atomic::AtomicUsize>,
) -> SourceDescriptor {
    let mut request = HttpRequest::new("http://127.0.0.1:1/synthetic".into());
    request
        .headers
        .insert("x-fixture", "synthetic-token".parse().unwrap());
    request.follow_redirects = false;
    request.header_timeout = Duration::from_secs(3);
    SourceDescriptor {
        request,
        format_ext: "flac".into(),
        requested_offset_ms: 1200,
        capabilities: PlaybackCapabilities {
            timeline: Timeline::Unknown,
            seek_policy: SeekPolicy::ServerOffset,
        },
        start: StartPosition::Calibrated {
            offset: Duration::from_millis(1100),
            max_error: Duration::from_millis(5),
        },
        pin: pin.map(|id| AudioPin {
            media_source_id: id.into(),
            audio_stream_index: 0,
            runtime_ticks: None,
            etag: None,
            codec: None,
            channels: None,
            sample_rate: None,
        }),
        lease: PlaybackLease::new(move || {
            released.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }),
    }
}

#[tokio::test]
async fn bridge_releases_stale_and_duplicate_descriptors_and_pins_only_current_occurrence() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    let mut player = bridge_player().await;
    let target = Duration::from_millis(1200);
    player.load(source("same-item"), target, false).unwrap();
    let old = player.coordinator.play().unwrap().unwrap();
    player.load(source("same-item"), target, false).unwrap();
    let current = player.coordinator.play().unwrap().unwrap();
    assert_ne!(old.key().id, current.key().id);
    let stale_released = Arc::new(AtomicUsize::new(0));
    assert!(player
        .sender
        .try_send((
            old,
            Ok(bridge_descriptor(Some("old"), stale_released.clone()))
        ))
        .is_ok());
    player.poll_with(|_| panic!("stale descriptor started a session"));
    assert_eq!(stale_released.load(Ordering::SeqCst), 1);
    assert!(player.accepted_pin().is_none());

    let active_released = Arc::new(AtomicUsize::new(0));
    assert!(player
        .sender
        .try_send((
            current.clone(),
            Ok(bridge_descriptor(Some("current"), active_released.clone()))
        ))
        .is_ok());
    let mut starts = 0;
    player.poll_with(|(http, format_ext)| {
        starts += 1;
        assert_eq!(format_ext, "flac");
        assert_eq!(http.url, "http://127.0.0.1:1/synthetic");
        assert_eq!(http.headers["x-fixture"], "synthetic-token");
        assert!(!http.follow_redirects);
        assert_eq!(http.header_timeout, Duration::from_secs(3));
        Ok(BridgeSession)
    });
    assert_eq!(starts, 1);
    let (id, pin) = player.accepted_pin().unwrap();
    assert_eq!(id, current.key().id);
    assert_eq!(pin.media_source_id, "current");
    assert_eq!(pin.audio_stream_index, 0);
    let position = player.snapshot().position.unwrap();
    assert_eq!(position.requested_start, target);
    assert_eq!(
        position.start,
        StartPosition::Calibrated {
            offset: Duration::from_millis(1100),
            max_error: Duration::from_millis(5),
        }
    );
    assert_eq!(position.absolute, Duration::from_millis(2100));
    assert_eq!(active_released.load(Ordering::SeqCst), 0);

    let duplicate_released = Arc::new(AtomicUsize::new(0));
    assert!(player
        .sender
        .try_send((
            current,
            Ok(bridge_descriptor(
                Some("duplicate"),
                duplicate_released.clone()
            ))
        ))
        .is_ok());
    player.poll_with(|_| panic!("duplicate descriptor started a session"));
    assert_eq!(duplicate_released.load(Ordering::SeqCst), 1);
    assert_eq!(player.accepted_pin().unwrap().1.media_source_id, "current");
    player.stop();
    assert_eq!(active_released.load(Ordering::SeqCst), 1);
    drop(player);
    assert_eq!(active_released.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn bridge_rejects_mismatched_target_and_does_not_invent_an_optional_pin() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    let mut player = bridge_player().await;
    player
        .load(source("old-item"), Duration::from_millis(500), false)
        .unwrap();
    let stale = player.coordinator.play().unwrap().unwrap();
    player
        .load(source("item"), Duration::from_millis(999), false)
        .unwrap();
    let request = player.coordinator.play().unwrap().unwrap();
    let released = Arc::new(AtomicUsize::new(0));
    assert!(player
        .sender
        .try_send((request, Ok(bridge_descriptor(None, released.clone()))))
        .is_ok());
    player.poll_with(|_| panic!("mismatched target started a session"));
    assert_eq!(player.snapshot().phase, Phase::Failed);
    assert!(player.error().is_some());
    assert_eq!(released.load(Ordering::SeqCst), 1);
    let current_error = player.error().unwrap().to_owned();
    let stale_released = Arc::new(AtomicUsize::new(0));
    assert!(player
        .sender
        .try_send((
            stale,
            Ok(bridge_descriptor(Some("stale"), stale_released.clone()))
        ))
        .is_ok());
    player.poll_with(|_| panic!("stale mismatched descriptor started a session"));
    assert_eq!(stale_released.load(Ordering::SeqCst), 1);
    assert_eq!(player.error(), Some(current_error.as_str()));
    assert_eq!(player.snapshot().phase, Phase::Failed);
    assert!(player.accepted_pin().is_none());
    assert_eq!(player.selection, AudioSelection::Auto);

    player
        .load(source("item"), Duration::from_millis(1200), false)
        .unwrap();
    let request = player.coordinator.play().unwrap().unwrap();
    assert!(player
        .sender
        .try_send((request, Ok(bridge_descriptor(None, released.clone()))))
        .is_ok());
    player.poll_with(|_| Ok(BridgeSession));
    assert_eq!(player.snapshot().phase, Phase::Streaming);
    assert!(player.accepted_pin().is_none());
    assert_eq!(player.checkpoint().unwrap().selection, AudioSelection::Auto);
    player.stop();
    assert_eq!(released.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn bridge_open_failure_releases_once_and_preserves_requested_selection() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    let mut player = bridge_player().await;
    let selection = AudioSelection::Explicit {
        media_source_id: "requested-version".into(),
        audio_stream_index: 0,
    };
    let occurrence_id = uuid::Uuid::new_v4().to_string();
    player
        .load_occurrence(
            source("item"),
            occurrence_id.clone(),
            selection.clone(),
            Duration::from_millis(1200),
            false,
        )
        .unwrap();
    let request = player.coordinator.play().unwrap().unwrap();
    let released = Arc::new(AtomicUsize::new(0));
    assert!(player
        .sender
        .try_send((
            request,
            Ok(bridge_descriptor(Some("failed-result"), released.clone(),))
        ))
        .is_ok());

    let mut starts = 0;
    player.poll_with(|_| {
        starts += 1;
        Err("synthetic stream open failure".into())
    });
    assert_eq!(starts, 1);
    assert_eq!(released.load(Ordering::SeqCst), 1);
    assert_eq!(player.snapshot().phase, Phase::Failed);
    assert_eq!(player.error(), Some("synthetic stream open failure"));
    assert!(player.snapshot().generation.is_none());
    assert!(player.accepted_pin().is_none());
    let checkpoint = player.checkpoint().unwrap();
    assert_eq!(checkpoint.occurrence_id, occurrence_id);
    assert_eq!(checkpoint.selection, selection);

    player.stop();
    assert_eq!(released.load(Ordering::SeqCst), 1);
    assert_eq!(player.checkpoint().unwrap().selection, selection);
    drop(player);
    assert_eq!(released.load(Ordering::SeqCst), 1);
}
