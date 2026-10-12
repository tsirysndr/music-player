use super::*;
use std::sync::Mutex;

#[derive(Clone)]
struct FakeSession(Arc<Mutex<FakeState>>);
struct FakeState {
    generation: u64,
    frames: u64,
    phase: StreamPhase,
    boundary: StreamOutputBoundary,
    input_released: bool,
    events: Arc<Mutex<Vec<&'static str>>>,
}
impl FakeSession {
    fn new(generation: u64, events: Arc<Mutex<Vec<&'static str>>>) -> Self {
        Self(Arc::new(Mutex::new(FakeState {
            generation,
            frames: 0,
            phase: StreamPhase::Opening,
            boundary: StreamOutputBoundary::ByteStream,
            input_released: false,
            events,
        })))
    }
    fn deliver(&self, frames: u64) {
        let mut state = self.0.lock().unwrap();
        assert_ne!(state.phase, StreamPhase::Cancelled);
        state.frames += frames;
    }
}
impl SessionControl for FakeSession {
    fn cancel(&self) {
        let mut state = self.0.lock().unwrap();
        state.events.lock().unwrap().push("cancel");
        state.phase = StreamPhase::Cancelled;
    }
    fn output(&self) -> StreamOutputSnapshot {
        let state = self.0.lock().unwrap();
        StreamOutputSnapshot {
            generation: state.generation,
            boundary: state.boundary,
            frames: state.frames,
            sample_rate: 44100,
        }
    }
    fn phase(&self) -> StreamPhase {
        self.0.lock().unwrap().phase
    }
    fn generation(&self) -> u64 {
        self.0.lock().unwrap().generation
    }
    fn input_released(&self) -> bool {
        self.0.lock().unwrap().input_released
    }
}

type Control = Coordinator<&'static str, FakeSession>;
fn caps(timeline: Timeline, seek_policy: SeekPolicy) -> PlaybackCapabilities {
    PlaybackCapabilities {
        timeline,
        seek_policy,
    }
}
fn with_caps(
    events: Arc<Mutex<Vec<&'static str>>>,
    capabilities: PlaybackCapabilities,
) -> Resolved<()> {
    let mut result = descriptor(events, StartPosition::RequestedOnly);
    result.capabilities = capabilities;
    result
}
fn descriptor(events: Arc<Mutex<Vec<&'static str>>>, start: StartPosition) -> Resolved<()> {
    Resolved {
        capabilities: caps(Timeline::Unknown, SeekPolicy::ServerOffset),
        payload: (),
        start,
        lease: PlaybackLease::new(move || events.lock().unwrap().push("release")),
    }
}
fn accept(
    c: &mut Control,
    request: &ResolveRequest<&'static str>,
    s: &FakeSession,
    start: StartPosition,
) {
    let events = s.0.lock().unwrap().events.clone();
    assert_eq!(
        c.resolved(request, descriptor(events, start), |_| Ok::<_, ()>(
            s.clone()
        )),
        ResolveOutcome::Started
    );
}
fn reject(c: &mut Control, request: &ResolveRequest<&'static str>) {
    let events = Arc::new(Mutex::new(Vec::new()));
    assert_eq!(
        c.resolved(
            request,
            descriptor(events.clone(), StartPosition::RequestedOnly),
            |_| -> Result<FakeSession, ()> { panic!("stale result started output") }
        ),
        ResolveOutcome::Stale
    );
    assert_eq!(*events.lock().unwrap(), ["release"]);
}

#[test]
fn seek_pause_late_resolution_keeps_paused_target() {
    let mut c = Control::default();
    let initial = c
        .load("episode:version:audio1", Duration::ZERO, true)
        .unwrap();
    let seek = c.seek(Duration::from_secs(7200)).unwrap().unwrap();
    c.pause();
    let before = c.snapshot();
    reject(&mut c, &initial);
    reject(&mut c, &seek);
    assert_eq!(c.resolve_failed(&seek), ResolveOutcome::Stale);
    assert_eq!(c.snapshot(), before);
    assert_eq!(before.desired, DesiredState::Paused);
    assert_eq!(before.target, Duration::from_secs(7200));
    assert!(before.position.is_none());
    assert!(initial.is_cancelled() && seek.is_cancelled());
    let resume = c.play().unwrap().unwrap();
    assert_eq!(*resume.key(), "episode:version:audio1");
    assert_eq!(resume.target(), Duration::from_secs(7200));
    assert!(!resume.is_cancelled());
}

#[test]
fn paused_seeks_never_resolve_and_resume_uses_latest_non_aligned_target() {
    let mut c = Control::default();
    assert!(c.load("movie", Duration::ZERO, false).is_none());
    for ms in [7_200_000, 7_205_123, 8_000_789] {
        assert!(c.seek(Duration::from_millis(ms)).unwrap().is_none());
        assert_eq!(c.snapshot().phase, Phase::Paused);
    }
    c.stop();
    assert_eq!(c.snapshot().target, Duration::from_millis(8_000_789));
    let request = c.play().unwrap().unwrap();
    assert_eq!(request.target(), Duration::from_millis(8_000_789));
    assert!(c.play().unwrap().is_none());
}

#[test]
fn stop_and_next_reject_all_old_success_and_error_results() {
    let mut c = Control::default();
    let a = c
        .load("server-a/account-a/item1", Duration::ZERO, true)
        .unwrap();
    c.stop();
    reject(&mut c, &a);
    assert_eq!(c.snapshot().desired, DesiredState::Stopped);
    let b = c
        .load("server-b/account-b/item1", Duration::from_secs(12), true)
        .unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    let s = FakeSession::new(42, events.clone());
    accept(&mut c, &b, &s, StartPosition::RequestedOnly);
    let before = c.snapshot();
    reject(&mut c, &a);
    assert_eq!(c.resolve_failed(&a), ResolveOutcome::Stale);
    reject(&mut c, &b); // Duplicate successful completion must not replace B.
    assert_eq!(c.snapshot(), before);
    assert!(events.lock().unwrap().is_empty());
    assert_eq!(s.phase(), StreamPhase::Opening);
}

#[test]
fn only_last_seek_can_start_for_every_completion_order() {
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let mut c = Control::default();
        let requests = [
            c.load("episode", Duration::ZERO, true).unwrap(),
            c.seek(Duration::from_secs(60)).unwrap().unwrap(),
            c.seek(Duration::from_secs(7200)).unwrap().unwrap(),
        ];
        let events = Arc::new(Mutex::new(Vec::new()));
        let s = FakeSession::new(99, events);
        for index in order {
            if index == 2 {
                accept(&mut c, &requests[index], &s, StartPosition::RequestedOnly);
            } else {
                reject(&mut c, &requests[index]);
            }
        }
        assert_eq!(c.snapshot().generation, Some(99));
        assert_eq!(c.snapshot().target, Duration::from_secs(7200));
    }
}

#[test]
fn ready_is_not_position_and_pause_uses_final_delivered_frames() {
    let mut c = Control::default();
    let request = c.load("episode", Duration::from_secs(7200), true).unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    let s = FakeSession::new(37, events.clone());
    accept(&mut c, &request, &s, StartPosition::RequestedOnly);
    c.observe();
    assert_eq!(c.snapshot().phase, Phase::Opening);
    assert!(c.snapshot().position.is_none());
    s.deliver(44100);
    c.observe();
    assert_eq!(
        c.snapshot().position.unwrap().absolute,
        Duration::from_secs(7201)
    );
    s.deliver(44100); // Arrived since last host tick; pause must capture this too.
    c.pause();
    assert_eq!(*events.lock().unwrap(), ["cancel", "release"]);
    assert_eq!(c.snapshot().target, Duration::from_secs(7202));
    assert_eq!(
        c.snapshot().position.unwrap().boundary,
        StreamOutputBoundary::ByteStream
    );
    let resume = c.play().unwrap().unwrap();
    assert_eq!(resume.target(), Duration::from_secs(7202));
    let s2 = FakeSession::new(38, events.clone());
    accept(&mut c, &resume, &s2, StartPosition::RequestedOnly);
    c.observe();
    assert_eq!(c.snapshot().phase, Phase::Opening);
    s2.deliver(22050);
    c.observe();
    assert_eq!(
        c.snapshot().position.unwrap().absolute,
        Duration::from_millis(7_202_500)
    );
    assert_eq!(c.snapshot().position.unwrap().generation, 38);
}

#[test]
fn calibrated_origin_preserves_requested_target_and_error_bound() {
    let mut c = Control::default();
    let request = c
        .load("episode", Duration::from_millis(7_205_123), true)
        .unwrap();
    let s = FakeSession::new(1, Arc::new(Mutex::new(Vec::new())));
    let origin = StartPosition::Calibrated {
        offset: Duration::from_millis(7_205_100),
        max_error: Duration::from_millis(20),
    };
    accept(&mut c, &request, &s, origin);
    s.0.lock().unwrap().boundary = StreamOutputBoundary::DeviceBuffer;
    s.deliver(88200);
    c.observe();
    let position = c.snapshot().position.unwrap();
    assert_eq!(position.absolute, Duration::from_millis(7_207_100));
    assert_eq!(position.requested_start, request.target());
    assert_eq!(position.start, origin);
    assert_eq!(position.boundary, StreamOutputBoundary::DeviceBuffer);
}

#[test]
fn unavailable_clock_does_not_invent_resume_position() {
    let mut c = Control::default();
    let request = c.load("episode", Duration::from_secs(7200), true).unwrap();
    let s = FakeSession::new(1, Arc::new(Mutex::new(Vec::new())));
    accept(&mut c, &request, &s, StartPosition::RequestedOnly);
    s.0.lock().unwrap().boundary = StreamOutputBoundary::Unavailable;
    s.deliver(441000);
    c.observe();
    c.pause();
    assert!(c.snapshot().position.is_none());
    assert_eq!(
        c.play().unwrap().unwrap().target(),
        Duration::from_secs(7200)
    );
}

#[test]
fn end_unconfirmed_releases_finished_input_without_revoking_output() {
    let mut c = Control::default();
    let request = c.load("episode", Duration::from_secs(7200), true).unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    let s = FakeSession::new(1, events.clone());
    accept(&mut c, &request, &s, StartPosition::RequestedOnly);
    s.0.lock().unwrap().phase = StreamPhase::EndUnconfirmed;
    for _ in 0..3 {
        s.deliver(44100);
        c.observe();
        assert_eq!(c.snapshot().phase, Phase::EndUnconfirmed);
    }
    assert!(events.lock().unwrap().is_empty());
    assert_eq!(
        c.snapshot().position.unwrap().absolute,
        Duration::from_secs(7203)
    );
    s.0.lock().unwrap().input_released = true;
    c.observe();
    c.observe();
    assert_eq!(*events.lock().unwrap(), ["release"]);
    s.deliver(44100); // Queued PCM can still drain after provider cleanup.
    c.observe();
    assert_eq!(
        c.snapshot().position.unwrap().absolute,
        Duration::from_secs(7204)
    );
    assert_eq!(c.snapshot().phase, Phase::EndUnconfirmed);
    c.stop();
    assert_eq!(*events.lock().unwrap(), ["release", "cancel"]);
}

#[test]
fn open_failure_releases_once_and_retry_retains_target() {
    let mut c = Control::default();
    let request = c.load("episode", Duration::from_secs(7200), true).unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    assert_eq!(
        c.resolved(
            &request,
            descriptor(events.clone(), StartPosition::RequestedOnly),
            |_| Err::<FakeSession, ()>(())
        ),
        ResolveOutcome::Failed
    );
    assert_eq!(*events.lock().unwrap(), ["release"]);
    assert_eq!(c.snapshot().phase, Phase::Failed);
    let retry = c.play().unwrap().unwrap();
    assert_eq!(retry.target(), request.target());
    assert_eq!(retry.key(), request.key());
    assert_eq!(c.resolve_failed(&request), ResolveOutcome::Stale);
    assert_eq!(c.resolve_failed(&retry), ResolveOutcome::Failed);
}

#[test]
fn stream_failure_cancels_resources_and_preserves_output_position() {
    let mut c = Control::default();
    let request = c.load("episode", Duration::from_secs(7200), true).unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    let s = FakeSession::new(1, events.clone());
    accept(&mut c, &request, &s, StartPosition::RequestedOnly);
    s.deliver(88200);
    s.0.lock().unwrap().phase = StreamPhase::Failed;
    c.observe();
    assert_eq!(*events.lock().unwrap(), ["cancel", "release"]);
    assert_eq!(c.snapshot().phase, Phase::Failed);
    assert_eq!(
        c.play().unwrap().unwrap().target(),
        Duration::from_secs(7202)
    );
}

#[test]
fn source_time_overflow_fails_without_wrapping_or_replacing_checkpoint() {
    let mut c = Control::default();
    let request = c.load("episode", Duration::MAX, true).unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    let s = FakeSession::new(1, events.clone());
    accept(&mut c, &request, &s, StartPosition::RequestedOnly);
    s.deliver(44100);
    c.observe();
    assert_eq!(c.snapshot().phase, Phase::Failed);
    assert_eq!(c.snapshot().target, Duration::MAX);
    assert!(c.snapshot().position.is_none());
    assert_eq!(*events.lock().unwrap(), ["cancel", "release"]);
}

#[test]
fn dropping_coordinator_invalidates_pending_and_releases_active() {
    let mut c = Control::default();
    let request = c.load("a", Duration::ZERO, true).unwrap();
    drop(c);
    assert!(request.is_cancelled());
    let mut c = Control::default();
    let request = c.load("b", Duration::ZERO, true).unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    let s = FakeSession::new(1, events.clone());
    accept(&mut c, &request, &s, StartPosition::RequestedOnly);
    drop(c);
    assert_eq!(*events.lock().unwrap(), ["cancel", "release"]);
}

#[tokio::test]
async fn cancelling_resolve_wakes_waiters_before_and_after_invalidation() {
    let mut c = Control::default();
    let request = c.load("episode", Duration::ZERO, true).unwrap();
    let clone = request.clone();
    let waiting = request.cancelled();
    tokio::pin!(waiting);
    // Poll once before pause, just as a resolver does beside its HTTP future.
    tokio::select! {
        biased;
        _ = &mut waiting => panic!("cancelled before a command"),
        _ = std::future::ready(()) => {}
    }
    c.pause();
    tokio::time::timeout(Duration::from_secs(1), waiting)
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(1), clone.cancelled())
        .await
        .unwrap();
}

#[test]
fn rejected_seek_does_not_cancel_active_or_pending_even_at_zero() {
    let capabilities = caps(Timeline::Unknown, SeekPolicy::None);
    let mut c = Control::default();
    let request = c
        .load_with_capabilities("item", Duration::ZERO, true, Some(capabilities))
        .unwrap();
    let before = c.snapshot();
    assert!(matches!(
        c.seek(Duration::ZERO),
        Err(ControlError::SeekUnsupported)
    ));
    assert!(!request.is_cancelled());
    assert_eq!(c.snapshot(), before);
    let events = Arc::new(Mutex::new(Vec::new()));
    let s = FakeSession::new(1, events.clone());
    assert_eq!(
        c.resolved(&request, with_caps(events.clone(), capabilities), |_| Ok::<
            _,
            (),
        >(
            s.clone()
        )),
        ResolveOutcome::Started
    );
    s.deliver(44100);
    c.observe();
    let before = c.snapshot();
    for target in [Duration::ZERO, Duration::from_secs(2)] {
        assert!(matches!(c.seek(target), Err(ControlError::SeekUnsupported)));
        assert_eq!(c.snapshot(), before);
        assert!(events.lock().unwrap().is_empty());
    }
    s.deliver(44100);
    c.pause();
    let paused = c.snapshot();
    assert_eq!(paused.target, Duration::from_secs(2));
    assert!(matches!(c.play(), Err(ControlError::ResumeUnsupported)));
    assert_eq!(c.snapshot(), paused);
    c.stop();
    let stopped = c.snapshot();
    assert!(matches!(c.play(), Err(ControlError::ResumeUnsupported)));
    assert_eq!(c.snapshot(), stopped);
    drop(c);
    assert_eq!(*events.lock().unwrap(), ["cancel", "release"]);
}

#[test]
fn server_offset_resume_and_seek_do_not_require_finite_duration() {
    for timeline in [Timeline::Unknown, Timeline::Live] {
        let capabilities = caps(timeline, SeekPolicy::ServerOffset);
        let mut c = Control::default();
        let request = c
            .load_with_capabilities(
                "item",
                Duration::from_millis(7_200_123),
                true,
                Some(capabilities),
            )
            .unwrap();
        let events = Arc::new(Mutex::new(Vec::new()));
        let s = FakeSession::new(1, events.clone());
        assert_eq!(
            c.resolved(&request, with_caps(events, capabilities), |_| Ok::<_, ()>(
                s.clone()
            )),
            ResolveOutcome::Started
        );
        s.deliver(44100);
        c.pause();
        assert_eq!(c.snapshot().capabilities, Some(capabilities));
        let resume = c.play().unwrap().unwrap();
        assert_eq!(resume.target(), Duration::from_millis(7_201_123));
        let seek = c.seek(Duration::from_secs(9000)).unwrap().unwrap();
        assert!(resume.is_cancelled());
        assert_eq!(seek.target(), Duration::from_secs(9000));
        c.pause();
        assert!(c.seek(Duration::from_secs(9100)).unwrap().is_none());
        c.stop();
        assert_eq!(
            c.play().unwrap().unwrap().target(),
            Duration::from_secs(9100)
        );
        assert_eq!(c.snapshot().capabilities, Some(capabilities));
        c.load("new occurrence", Duration::ZERO, false);
        assert_eq!(c.snapshot().capabilities, None);
    }
}

#[test]
fn changed_evidence_fails_before_start_and_releases_once() {
    let finite = Timeline::Finite {
        duration_ticks: 191429666670,
    };
    for (before, after) in [
        (
            caps(finite, SeekPolicy::ServerOffset),
            caps(
                Timeline::Finite { duration_ticks: 1 },
                SeekPolicy::ServerOffset,
            ),
        ),
        (
            caps(finite, SeekPolicy::ServerOffset),
            caps(Timeline::Unknown, SeekPolicy::ServerOffset),
        ),
        (
            caps(finite, SeekPolicy::ServerOffset),
            caps(Timeline::Live, SeekPolicy::ServerOffset),
        ),
        (
            caps(Timeline::Unknown, SeekPolicy::ServerOffset),
            caps(Timeline::Live, SeekPolicy::ServerOffset),
        ),
        (
            caps(Timeline::Live, SeekPolicy::ServerOffset),
            caps(Timeline::Unknown, SeekPolicy::ServerOffset),
        ),
        (
            caps(Timeline::Live, SeekPolicy::ServerOffset),
            caps(finite, SeekPolicy::ServerOffset),
        ),
        (
            caps(finite, SeekPolicy::ServerOffset),
            caps(finite, SeekPolicy::None),
        ),
        (
            caps(finite, SeekPolicy::None),
            caps(finite, SeekPolicy::ServerOffset),
        ),
    ] {
        let mut c = Control::default();
        let request = c
            .load_with_capabilities("item", Duration::ZERO, true, Some(before))
            .unwrap();
        let events = Arc::new(Mutex::new(Vec::new()));
        assert_eq!(
            c.resolved(
                &request,
                with_caps(events.clone(), after),
                |_| -> Result<FakeSession, ()> { panic!("incompatible evidence started") }
            ),
            ResolveOutcome::Failed
        );
        assert_eq!(c.snapshot().phase, Phase::Failed);
        assert_eq!(c.snapshot().capabilities, Some(before));
        c.stop();
        drop(c);
        assert_eq!(*events.lock().unwrap(), ["release"]);
    }
}

#[test]
fn unknown_can_gain_finite_evidence_only_after_successful_start() {
    let old = caps(Timeline::Unknown, SeekPolicy::ServerOffset);
    let new = caps(
        Timeline::Finite { duration_ticks: 1 },
        SeekPolicy::ServerOffset,
    );
    let mut c = Control::default();
    // No new finite-duration bound is imposed by this coordinator.
    let request = c
        .load_with_capabilities("item", Duration::from_secs(7200), true, Some(old))
        .unwrap();
    let failed = Arc::new(Mutex::new(Vec::new()));
    assert_eq!(
        c.resolved(&request, with_caps(failed.clone(), new), |_| Err::<
            FakeSession,
            (),
        >(())),
        ResolveOutcome::Failed
    );
    assert_eq!(c.snapshot().capabilities, Some(old));
    assert_eq!(*failed.lock().unwrap(), ["release"]);
    let retry = c.play().unwrap().unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    let s = FakeSession::new(1, events.clone());
    assert_eq!(
        c.resolved(&retry, with_caps(events, new), |_| Ok::<_, ()>(s)),
        ResolveOutcome::Started
    );
    assert_eq!(c.snapshot().capabilities, Some(new));
    let snapshot = c.snapshot();
    let stale = Arc::new(Mutex::new(Vec::new()));
    assert_eq!(
        c.resolved(
            &request,
            with_caps(stale.clone(), caps(Timeline::Live, SeekPolicy::None)),
            |_| -> Result<FakeSession, ()> { panic!("stale started") }
        ),
        ResolveOutcome::Stale
    );
    assert_eq!(c.snapshot(), snapshot);
    assert_eq!(*stale.lock().unwrap(), ["release"]);
}

#[test]
fn initially_unknown_policy_cannot_start_nonzero_unseekable_resolution() {
    let mut c = Control::default();
    let request = c.load("item", Duration::from_secs(7200), true).unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    assert_eq!(
        c.resolved(
            &request,
            with_caps(events.clone(), caps(Timeline::Unknown, SeekPolicy::None)),
            |_| -> Result<FakeSession, ()> { panic!("unsupported offset started") }
        ),
        ResolveOutcome::Failed
    );
    assert_eq!(c.snapshot().target, Duration::from_secs(7200));
    assert_eq!(c.snapshot().capabilities, None);
    assert_eq!(c.snapshot().phase, Phase::Failed);
    c.stop();
    drop(c);
    assert_eq!(*events.lock().unwrap(), ["release"]);
}
