//! Serial control for capability-described audio at 1x playback speed.
//!
//! The host drives commands and resolve completions on the same event loop. A
//! request identity is independent of the engine generation, which exists only
//! after a resolution is accepted. K is the pinned source/track/version identity;
//! it must not be a temporary URL or the currently browsed provider.
//!
//! This component does not resolve sources, schedule retries, cache media or
//! advance queues. The daemon adapter supplies those policies. Output observations
//! are delivery estimates, never listening statistics or proof of completion.
use rockbox_playback::{StreamOutputBoundary, StreamOutputSnapshot, StreamPhase, StreamSession};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::sync::Notify;

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Timeline {
    Finite { duration_ticks: u64 },
    Live,
    Unknown,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeekPolicy {
    ServerOffset,
    None,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlaybackCapabilities {
    pub timeline: Timeline,
    pub seek_policy: SeekPolicy,
}
impl PlaybackCapabilities {
    fn compatible_with(self, next: Self) -> bool {
        self.seek_policy == next.seek_policy
            && match (self.timeline, next.timeline) {
                (Timeline::Unknown, Timeline::Unknown | Timeline::Finite { .. }) => true,
                (Timeline::Live, Timeline::Live) => true,
                (
                    Timeline::Finite { duration_ticks: a },
                    Timeline::Finite { duration_ticks: b },
                ) => a == b,
                _ => false,
            }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlError {
    SeekUnsupported,
    ResumeUnsupported,
}
impl std::fmt::Display for ControlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::SeekUnsupported => "source does not support seeking",
            Self::ResumeUnsupported => "source does not support resuming at this position",
        })
    }
}
impl std::error::Error for ControlError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesiredState {
    Playing,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Stopped,
    Paused,
    Resolving,
    Opening,
    Streaming,
    Failed,
    EndUnconfirmed,
}

/// Evidence for the beginning of the returned audio, separate from the requested
/// seek target. Even a calibrated start cannot confirm downstream consumption.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartPosition {
    RequestedOnly,
    Calibrated {
        offset: Duration,
        max_error: Duration,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EstimatedPosition {
    pub absolute: Duration,
    pub requested_start: Duration,
    pub start: StartPosition,
    pub boundary: StreamOutputBoundary,
    pub generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Snapshot {
    /// None means no capability evidence yet, not SeekPolicy::None.
    pub capabilities: Option<PlaybackCapabilities>,
    pub desired: DesiredState,
    pub phase: Phase,
    /// The next resolve offset (including a paused seek), not a confirmed position.
    pub target: Duration,
    /// Last observed output for this source. May belong to the preceding session
    /// while a seek is resolving; generation makes that distinction explicit.
    pub position: Option<EstimatedPosition>,
    pub generation: Option<u64>,
}

/// A cloneable ticket for one resolve attempt. Fields are private so callers
/// cannot accidentally change the source/offset while retaining its identity.
#[derive(Clone)]
pub struct ResolveRequest<K> {
    key: K,
    target: Duration,
    signal: Arc<ResolveSignal>,
}
struct ResolveSignal {
    cancelled: AtomicBool,
    wake: Notify,
}
impl<K> ResolveRequest<K> {
    pub fn key(&self) -> &K {
        &self.key
    }
    pub fn target(&self) -> Duration {
        self.target
    }
    pub fn is_cancelled(&self) -> bool {
        self.signal.cancelled.load(Ordering::Acquire)
    }
    /// Use in the resolver's select loop so pause/next/stop interrupt a pending
    /// network resolve. Cancellation stays observable even to late subscribers.
    pub async fn cancelled(&self) {
        let wake = self.signal.wake.notified();
        tokio::pin!(wake);
        wake.as_mut().enable();
        if !self.is_cancelled() {
            wake.await;
        }
    }
    fn cancel(&self) {
        self.signal.cancelled.store(true, Ordering::Release);
        self.signal.wake.notify_waiters();
    }
    fn same_attempt(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.signal, &other.signal)
    }
}

/// Once-only resource release. The callback must enqueue bounded cleanup, not
/// perform network I/O or wait for a server on the command loop.
pub struct PlaybackLease(Option<Box<dyn FnOnce() + Send>>);
impl PlaybackLease {
    pub fn new(release: impl FnOnce() + Send + 'static) -> Self {
        Self(Some(Box::new(release)))
    }
}
impl Drop for PlaybackLease {
    fn drop(&mut self) {
        if let Some(release) = self.0.take() {
            release();
        }
    }
}

/// Payload is in-memory resolver state. A payload owning an already-open reader
/// must cancel/join it on Drop; preferably open the reader in the start callback.
pub struct Resolved<P> {
    pub capabilities: PlaybackCapabilities,
    pub payload: P,
    pub start: StartPosition,
    pub lease: PlaybackLease,
}

/// Small engine port so deterministic command ordering can be checked without
/// a sound device. The production implementation uses the actual StreamSession.
pub trait SessionControl {
    /// Idempotently revoke this session's output and freeze its media counter
    /// before returning; must not wait for remote resource cleanup.
    fn cancel(&self);
    fn output(&self) -> StreamOutputSnapshot;
    fn phase(&self) -> StreamPhase;
    fn generation(&self) -> u64;
    /// The input reader and decoder have both finished releasing their resources.
    /// This does not imply that output is drained or media is complete.
    fn input_released(&self) -> bool;
}
impl SessionControl for StreamSession {
    fn cancel(&self) {
        StreamSession::cancel(self);
    }
    fn output(&self) -> StreamOutputSnapshot {
        self.output_snapshot()
    }
    fn phase(&self) -> StreamPhase {
        self.snapshot().phase
    }
    fn generation(&self) -> u64 {
        StreamSession::generation(self)
    }
    fn input_released(&self) -> bool {
        let snapshot = self.snapshot();
        snapshot.reader_released && snapshot.decoder_joined
    }
}

struct Active<S: SessionControl> {
    session: S,
    requested_start: Duration,
    start: StartPosition,
    lease: Option<PlaybackLease>,
    cancelled: bool,
}
impl<S: SessionControl> Active<S> {
    fn cancel(&mut self) {
        if !self.cancelled {
            self.cancelled = true;
            self.session.cancel();
        }
    }
    fn position(&self) -> Result<Option<EstimatedPosition>, ()> {
        let output = self.session.output();
        if output.generation != self.session.generation() || output.frames == 0 {
            return Ok(None);
        }
        let Some(elapsed) = output.duration() else {
            return Ok(None);
        };
        let base = match self.start {
            StartPosition::RequestedOnly => self.requested_start,
            StartPosition::Calibrated { offset, .. } => offset,
        };
        Ok(Some(EstimatedPosition {
            absolute: base.checked_add(elapsed).ok_or(())?,
            requested_start: self.requested_start,
            start: self.start,
            boundary: output.boundary,
            generation: output.generation,
        }))
    }
}
impl<S: SessionControl> Drop for Active<S> {
    fn drop(&mut self) {
        // Revoke local output before asking the provider to release its lease.
        self.cancel();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolveOutcome {
    Started,
    Stale,
    Failed,
}

pub struct Coordinator<K, S: SessionControl = StreamSession> {
    capabilities: Option<PlaybackCapabilities>,
    key: Option<K>,
    desired: DesiredState,
    phase: Phase,
    target: Duration,
    position: Option<EstimatedPosition>,
    pending: Option<ResolveRequest<K>>,
    active: Option<Active<S>>,
}
impl<K, S: SessionControl> Default for Coordinator<K, S> {
    fn default() -> Self {
        Self {
            capabilities: None,
            key: None,
            desired: DesiredState::Stopped,
            phase: Phase::Stopped,
            target: Duration::ZERO,
            position: None,
            pending: None,
            active: None,
        }
    }
}
impl<K: Clone, S: SessionControl> Coordinator<K, S> {
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            capabilities: self.capabilities,
            desired: self.desired,
            phase: self.phase,
            target: self.target,
            position: self.position,
            generation: self.active.as_ref().map(|a| a.session.generation()),
        }
    }

    /// Load/next replaces the source even when the previous resolve is pending.
    pub fn load(&mut self, key: K, target: Duration, playing: bool) -> Option<ResolveRequest<K>> {
        self.load_with_capabilities(key, target, playing, None)
    }

    /// Restore retained evidence for this occurrence. A playing load still has
    /// its offset and capabilities checked before any engine start.
    pub fn load_with_capabilities(
        &mut self,
        key: K,
        target: Duration,
        playing: bool,
        capabilities: Option<PlaybackCapabilities>,
    ) -> Option<ResolveRequest<K>> {
        self.invalidate();
        self.active.take();
        self.capabilities = capabilities;
        self.key = Some(key);
        self.target = target;
        self.position = None;
        self.desired = if playing {
            DesiredState::Playing
        } else {
            DesiredState::Paused
        };
        self.begin()
    }

    pub fn play(&mut self) -> Result<Option<ResolveRequest<K>>, ControlError> {
        if self.key.is_none() {
            return Ok(None);
        }
        if self.desired == DesiredState::Playing
            && matches!(
                self.phase,
                Phase::Resolving | Phase::Opening | Phase::Streaming
            )
        {
            return Ok(None);
        }
        let resume_target = self
            .active
            .as_ref()
            .and_then(|active| active.position().ok().flatten())
            .map_or(self.target, |position| position.absolute);
        if self.seek_disabled() && !resume_target.is_zero() {
            return Err(ControlError::ResumeUnsupported);
        }
        self.capture_and_cancel();
        self.desired = DesiredState::Playing;
        Ok(self.begin())
    }

    pub fn seek(&mut self, target: Duration) -> Result<Option<ResolveRequest<K>>, ControlError> {
        if self.seek_disabled() {
            return Err(ControlError::SeekUnsupported);
        }
        if self.key.is_none() {
            return Ok(None);
        }
        self.invalidate();
        self.active.take();
        self.target = target;
        Ok(self.begin())
    }

    pub fn pause(&mut self) {
        self.capture_and_cancel();
        self.desired = DesiredState::Paused;
        self.phase = Phase::Paused;
    }

    pub fn stop(&mut self) {
        self.capture_and_cancel();
        self.desired = DesiredState::Stopped;
        self.phase = Phase::Stopped;
    }

    /// Deliver on the command loop. Crucially, the engine start callback is not
    /// invoked for a stale result, including duplicate delivery of the same ticket.
    /// The callback must only enqueue engine startup; do not await network I/O.
    pub fn resolved<P, E>(
        &mut self,
        request: &ResolveRequest<K>,
        resolved: Resolved<P>,
        start: impl FnOnce(P) -> Result<S, E>,
    ) -> ResolveOutcome {
        if !self.accepts(request) {
            return ResolveOutcome::Stale;
        }
        self.pending.take();
        if (resolved.capabilities.seek_policy == SeekPolicy::None && !request.target.is_zero())
            || self
                .capabilities
                .is_some_and(|previous| !previous.compatible_with(resolved.capabilities))
        {
            self.phase = Phase::Failed;
            return ResolveOutcome::Failed;
        }
        let Resolved {
            capabilities,
            payload,
            start: origin,
            lease,
        } = resolved;
        match start(payload) {
            Ok(session) => {
                self.capabilities = Some(capabilities);
                self.active = Some(Active {
                    session,
                    requested_start: request.target,
                    start: origin,
                    lease: Some(lease),
                    cancelled: false,
                });
                self.phase = Phase::Opening;
                ResolveOutcome::Started
            }
            Err(_) => {
                self.phase = Phase::Failed;
                ResolveOutcome::Failed
            }
        }
    }

    /// Errors from cancelled/older resolves must not replace current state.
    pub fn resolve_failed(&mut self, request: &ResolveRequest<K>) -> ResolveOutcome {
        if !self.accepts(request) {
            return ResolveOutcome::Stale;
        }
        self.pending.take();
        self.phase = Phase::Failed;
        ResolveOutcome::Failed
    }

    /// Called by the host tick. Zero delivered frames never confirm a seek.
    /// Decoder EOF leaves the stream alive to drain; it never advances a queue.
    pub fn observe(&mut self) {
        let Some(active) = self.active.as_mut() else {
            return;
        };
        // Once input is gone, provider cleanup cannot interrupt decoding. Keep
        // the session for output observation without retaining its server lease.
        if active.session.input_released() {
            active.lease.take();
        }
        match active.position() {
            Ok(Some(position)) => self.position = Some(position),
            Ok(None) => {}
            Err(()) => {
                self.active.take();
                self.phase = Phase::Failed;
                return;
            }
        }
        self.phase = match active.session.phase() {
            StreamPhase::Failed | StreamPhase::Cancelled => {
                self.capture_and_cancel();
                Phase::Failed
            }
            StreamPhase::EndUnconfirmed => Phase::EndUnconfirmed,
            _ if self
                .position
                .is_some_and(|p| p.generation == active.session.generation()) =>
            {
                Phase::Streaming
            }
            _ => Phase::Opening,
        };
    }

    fn seek_disabled(&self) -> bool {
        self.capabilities
            .is_some_and(|caps| caps.seek_policy == SeekPolicy::None)
    }

    fn accepts(&self, request: &ResolveRequest<K>) -> bool {
        self.desired == DesiredState::Playing
            && !request.is_cancelled()
            && self
                .pending
                .as_ref()
                .is_some_and(|p| p.same_attempt(request))
    }

    fn invalidate(&mut self) {
        if let Some(request) = self.pending.take() {
            request.cancel();
        }
    }

    fn capture_and_cancel(&mut self) {
        self.invalidate();
        if let Some(mut active) = self.active.take() {
            // The real session revokes output synchronously. Read its frozen
            // final counter, not an earlier tick or the decoder lead position.
            active.cancel();
            if let Ok(Some(position)) = active.position() {
                self.target = position.absolute;
                self.position = Some(position);
            }
        }
        // Without active output (e.g. seek -> pause), keep the pending target.
    }

    fn begin(&mut self) -> Option<ResolveRequest<K>> {
        self.invalidate();
        self.phase = match self.desired {
            DesiredState::Paused => Phase::Paused,
            DesiredState::Stopped => Phase::Stopped,
            DesiredState::Playing => {
                let request = ResolveRequest {
                    key: self.key.as_ref()?.clone(),
                    target: self.target,
                    signal: Arc::new(ResolveSignal {
                        cancelled: AtomicBool::new(false),
                        wake: Notify::new(),
                    }),
                };
                self.pending = Some(request.clone());
                self.phase = Phase::Resolving;
                return Some(request);
            }
        };
        None
    }
}
impl<K, S: SessionControl> Drop for Coordinator<K, S> {
    fn drop(&mut self) {
        if let Some(request) = self.pending.take() {
            request.cancel();
        }
        // Active::drop revokes output before dropping its lease.
    }
}

#[cfg(test)]
mod tests;
