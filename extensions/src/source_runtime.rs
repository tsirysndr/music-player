//! Bounded, instance-affine source dispatch. Register this module publicly.
//! One dedicated thread per live instance; no guest call runs on an async worker.
//! Cancellation linearizes at the last `is_closed` check before dispatch. After
//! dispatch it suppresses delivery, not execution. We never use CancelHandle.
//! A wedged guest pins its thread/budget; an outer timeout is not proof of exit.
//! Failed cleanup quarantines and retains the instance and obligation. This is
//! intentionally fail-closed: no automatic reset can discard remote ownership.
//! Limits are 32 worker lifetimes process-wide and 16 reservations per instance
//! (queued/running calls, active leases, queued/running/failed cleanup combined).
//! These are count limits, not a replacement for guest memory/response limits.
//! Instance bookkeeping is also capped at 32 entries. A disabled weak tombstone
//! occupies an entry until the last external Extension owner drops the instance;
//! it retains neither a worker nor the Extension itself after cleanup finishes.
use crate::{
    abi::source_v2::*,
    host::{Extension, SourceInstanceBusy},
};
use std::{
    collections::{HashSet, VecDeque},
    fmt,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Condvar, Mutex, OnceLock, Weak,
    },
    time::Duration,
};
use tokio::sync::oneshot;

mod v1;
pub use v1::{LegacySourceLease, ResolvedLegacySource};

const INSTANCE_LIMIT: usize = 32;
const SLOT_LIMIT: usize = 16;
const ATTEMPTS: usize = 3;
static INSTANCES: OnceLock<Mutex<Vec<Instance>>> = OnceLock::new();
// A disabled instance keeps a weak tombstone after its worker exits. Reusing
// that same Arc cannot undo disable; dropping the Extension removes the tombstone.
struct Instance {
    extension: Weak<Extension>,
    core: Weak<Core>,
    disabled: bool,
}
static WORKERS: AtomicUsize = AtomicUsize::new(0);
struct WorkerBudget;
impl Drop for WorkerBudget {
    fn drop(&mut self) {
        WORKERS.fetch_sub(1, Ordering::SeqCst);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeError {
    Busy,
    Unavailable,
    InvalidRequest,
    CallFailed,
    CleanupFailed,
}
impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Busy => "source instance capacity unavailable; no guest call executed",
            Self::Unavailable => "source scheduler unavailable",
            Self::InvalidRequest => "invalid or duplicate source attempt",
            Self::CallFailed => "source call failed",
            Self::CleanupFailed => "source cleanup failed; instance quarantined",
        })
    }
}
impl std::error::Error for RuntimeError {}

/// Counts only; never includes remote IDs, credentials or guest error strings.
#[derive(Clone, Copy, Debug)]
pub struct RuntimeStatus {
    pub reserved: usize,
    pub queued: usize,
    pub pending_cleanup: usize,
    pub failed_cleanup: usize,
    pub quarantined: bool,
    pub disabled: bool,
    pub retrying_cleanup: usize,
}

type Job = Box<dyn FnOnce() + Send>;
struct State {
    handles: usize,
    jobs: VecDeque<Job>,
    cleanup: VecDeque<Cleanup>,
    failed: Vec<Cleanup>,
    ids: HashSet<String>,
    reserved: usize,
    quarantined: bool,
    stopped: bool,
    disabled: bool,
    retry: Option<CleanupRetry>,
}
struct CleanupRetry {
    remaining: usize,
    total: usize,
    failed: bool,
    reply: oneshot::Sender<Result<usize, RuntimeError>>,
}
struct Core {
    extension: Arc<Extension>,
    state: Mutex<State>,
    wake: Condvar,
    // Fields drop in declaration order. Keep capacity through Extension teardown,
    // even if the last handle's Drop briefly outlives the worker itself.
    _worker_budget: WorkerBudget,
    #[cfg(test)]
    before_resolve_delivery: Mutex<Option<Job>>,
    #[cfg(test)]
    lifetime_observer: Mutex<Option<oneshot::Sender<()>>>,
    #[cfg(test)]
    before_idle_wait: Mutex<Option<oneshot::Sender<()>>>,
}

pub struct SourceRuntime {
    core: Arc<Core>,
}
impl Clone for SourceRuntime {
    fn clone(&self) -> Self {
        self.core.state.lock().unwrap().handles += 1;
        Self {
            core: self.core.clone(),
        }
    }
}
impl Drop for SourceRuntime {
    fn drop(&mut self) {
        let mut state = self.core.state.lock().unwrap();
        state.handles -= 1;
        self.core.wake.notify_one();
    }
}
struct Permit {
    core: Arc<Core>,
    id: Option<String>,
}
impl Drop for Permit {
    fn drop(&mut self) {
        let mut state = self.core.state.lock().unwrap();
        state.reserved -= 1;
        if let Some(id) = &self.id {
            state.ids.remove(id);
        }
        self.core.wake.notify_one();
    }
}
struct Cleanup {
    request: ReleaseRequest,
    permit: Permit,
    reply: Option<oneshot::Sender<Result<ReleaseResponse, RuntimeError>>>,
    retry: bool,
}
/// Neither this lease nor its response implements Debug. Drop does not require
/// a Tokio runtime, a UI receiver, or the continued existence of SourceRuntime.
pub struct SourceLease {
    cleanup: Option<Cleanup>,
}
impl Drop for SourceLease {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            enqueue_cleanup(cleanup);
        }
    }
}
pub struct ResolvedSource {
    pub response: ResolveResponse,
    pub lease: SourceLease,
}

fn enqueue_cleanup(cleanup: Cleanup) {
    let core = cleanup.permit.core.clone();
    // Uses the resolve reservation: cleanup admission cannot fail due to a full queue.
    core.state.lock().unwrap().cleanup.push_back(cleanup);
    core.wake.notify_one();
}

impl SourceRuntime {
    /// Repeated construction for the SAME Arc instance shares all reservations.
    /// Separate instances also have a process-wide cap on dedicated workers.
    pub fn new(extension: Arc<Extension>) -> Result<Self, RuntimeError> {
        let mut registry = INSTANCES
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .unwrap();
        registry.retain(|entry| {
            (entry.disabled && entry.extension.strong_count() > 0)
                || entry.core.upgrade().is_some_and(|core| {
                    let state = core.state.lock().unwrap();
                    !state.stopped
                })
        });
        for entry in registry.iter() {
            if !entry.extension.ptr_eq(&Arc::downgrade(&extension)) {
                continue;
            }
            if let Some(core) = entry.core.upgrade() {
                let mut state = core.state.lock().unwrap();
                if !state.stopped && Arc::ptr_eq(&core.extension, &extension) {
                    state.handles += 1;
                    drop(state);
                    return Ok(Self { core });
                }
            }
            if entry.disabled {
                return Err(RuntimeError::Unavailable);
            }
        }
        if registry.len() >= INSTANCE_LIMIT {
            return Err(RuntimeError::Busy);
        }
        WORKERS
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |count| {
                (count < INSTANCE_LIMIT).then_some(count + 1)
            })
            .map_err(|_| RuntimeError::Busy)?;
        let worker_budget = WorkerBudget;
        let core = Arc::new(Core {
            extension,
            state: Mutex::new(State {
                handles: 1,
                jobs: VecDeque::new(),
                cleanup: VecDeque::new(),
                failed: Vec::new(),
                ids: HashSet::new(),
                reserved: 0,
                quarantined: false,
                stopped: false,
                disabled: false,
                retry: None,
            }),
            wake: Condvar::new(),
            _worker_budget: worker_budget,
            #[cfg(test)]
            before_resolve_delivery: Mutex::new(None),
            #[cfg(test)]
            lifetime_observer: Mutex::new(None),
            #[cfg(test)]
            before_idle_wait: Mutex::new(None),
        });
        let worker_core = core.clone();
        std::thread::Builder::new()
            .name("source-runtime".into())
            .spawn(move || worker(worker_core))
            .map_err(|_| RuntimeError::Unavailable)?;
        registry.push(Instance {
            extension: Arc::downgrade(&core.extension),
            core: Arc::downgrade(&core),
            disabled: false,
        });
        Ok(Self { core })
    }

    pub fn status(&self) -> RuntimeStatus {
        let state = self.core.state.lock().unwrap();
        RuntimeStatus {
            reserved: state.reserved,
            queued: state.jobs.len(),
            pending_cleanup: state.cleanup.len(),
            failed_cleanup: state.failed.len(),
            quarantined: state.quarantined,
            disabled: state.disabled,
            retrying_cleanup: state.retry.as_ref().map_or(0, |retry| retry.remaining),
        }
    }

    /// Permanently stop normal admissions for this exact Extension instance.
    /// Queued jobs observe disable at dispatch; an already dispatched call is
    /// not interrupted. Existing leases and all cleanup remain owned and runnable.
    pub fn disable(&self) {
        // Same lock ordering as new(): registry, then instance state.
        let mut registry = INSTANCES
            .get()
            .expect("registered source runtime")
            .lock()
            .unwrap();
        if let Some(entry) = registry
            .iter_mut()
            .find(|entry| entry.core.ptr_eq(&Arc::downgrade(&self.core)))
        {
            entry.disabled = true;
        }
        self.core.state.lock().unwrap().disabled = true;
        self.core.wake.notify_one();
    }

    /// Retry the failed obligations present at admission, once each (at most
    /// three guest calls each). Concurrent retry requests return Busy, not another
    /// batch. Dropping the waiter does not cancel cleanup or release reservations.
    /// Returns the number recovered; any failure returns CleanupFailed. Observe
    /// status for remaining obligations. Disable is never undone by recovery.
    pub async fn retry_failed_cleanup(&self) -> Result<usize, RuntimeError> {
        let (tx, rx) = oneshot::channel();
        {
            let mut state = self.core.state.lock().unwrap();
            if state.retry.is_some() {
                return Err(RuntimeError::Busy);
            }
            let total = state.failed.len();
            if total == 0 {
                return Ok(0);
            }
            let failed = std::mem::take(&mut state.failed);
            state.retry = Some(CleanupRetry {
                remaining: total,
                total,
                failed: false,
                reply: tx,
            });
            for mut cleanup in failed {
                cleanup.retry = true;
                state.cleanup.push_back(cleanup);
            }
        }
        self.core.wake.notify_one();
        rx.await.unwrap_or(Err(RuntimeError::CleanupFailed))
    }

    fn reserve(&self, id: Option<String>) -> Result<Permit, RuntimeError> {
        let mut state = self.core.state.lock().unwrap();
        if state.quarantined || state.stopped || state.disabled {
            return Err(RuntimeError::Unavailable);
        }
        if state.reserved >= SLOT_LIMIT {
            return Err(RuntimeError::Busy);
        }
        if let Some(id) = &id {
            if id.trim().is_empty() || !state.ids.insert(id.clone()) {
                return Err(RuntimeError::InvalidRequest);
            }
        }
        state.reserved += 1;
        Ok(Permit {
            core: self.core.clone(),
            id,
        })
    }
    fn enqueue(&self, job: Job) {
        self.core.state.lock().unwrap().jobs.push_back(job);
        self.core.wake.notify_one();
    }
    async fn call<T: Send + 'static>(
        &self,
        call: impl Fn(&Extension) -> anyhow::Result<T> + Send + 'static,
    ) -> Result<T, RuntimeError> {
        let permit = self.reserve(None)?;
        let core = self.core.clone();
        let (tx, rx) = oneshot::channel();
        self.enqueue(Box::new(move || {
            let result = invoke(&core, &tx, call);
            drop(permit);
            let _ = tx.send(result.map_err(|failure| failure.error));
        }));
        rx.await.unwrap_or(Err(RuntimeError::Unavailable))
    }
    pub async fn source_identity_v2(
        &self,
        request: IdentityRequest,
    ) -> Result<SourceIdentity, RuntimeError> {
        self.call(move |ext| ext.source_identity_v2(&request)).await
    }
    pub async fn browse_media_v2(
        &self,
        request: BrowseRequest,
    ) -> Result<BrowseResponse, RuntimeError> {
        self.call(move |ext| ext.browse_media_v2(&request)).await
    }
    pub async fn get_media_item_v2(
        &self,
        request: ItemRequest,
    ) -> Result<ItemResponse, RuntimeError> {
        self.call(move |ext| ext.get_media_item_v2(&request)).await
    }
    pub async fn get_audio_options_v2(
        &self,
        request: AudioOptionsRequest,
    ) -> Result<AudioOptionsResponse, RuntimeError> {
        self.call(move |ext| ext.get_audio_options_v2(&request))
            .await
    }
    /// Callers must use unique request IDs per attempt. Executed errors and lost
    /// replies keep cleanup ownership here, including invalid responses with no lease ID.
    pub async fn resolve_audio_v2(
        &self,
        request: ResolveRequest,
    ) -> Result<ResolvedSource, RuntimeError> {
        if request.identity.server_id.trim().is_empty()
            || request.identity.user_id.trim().is_empty()
            || request.item_id.trim().is_empty()
        {
            return Err(RuntimeError::InvalidRequest);
        }
        let permit = self.reserve(Some(request.request_id.clone()))?;
        let core = self.core.clone();
        let (tx, rx) = oneshot::channel();
        self.enqueue(Box::new(move || {
            let mut cleanup = Cleanup {
                request: ReleaseRequest {
                    request_id: request.request_id.clone(),
                    identity: request.identity.clone(),
                    item_id: request.item_id.clone(),
                    lease_id: None,
                },
                permit,
                reply: None,
                retry: false,
            };
            match invoke(&core, &tx, |ext| ext.resolve_audio_v2(&request)) {
                Ok(response) => {
                    cleanup.request.lease_id = response.lease_id.clone();
                    #[cfg(test)]
                    if let Some(hook) = core.before_resolve_delivery.lock().unwrap().take() {
                        hook();
                    }
                    // Failed send drops the owned lease, scheduling original-instance cleanup.
                    let _ = tx.send(Ok(ResolvedSource {
                        response,
                        lease: SourceLease {
                            cleanup: Some(cleanup),
                        },
                    }));
                }
                Err(failure) => {
                    if failure.executed {
                        enqueue_cleanup(cleanup);
                    } else {
                        drop(cleanup);
                    }
                    let _ = tx.send(Err(failure.error));
                }
            }
        }));
        rx.await.unwrap_or(Err(RuntimeError::Unavailable))
    }
    /// Consumes ownership, not an arbitrary caller-supplied requestId. Dropping
    /// this future after admission only drops the waiter; release still runs.
    pub async fn release_audio_v2(
        &self,
        mut lease: SourceLease,
    ) -> Result<ReleaseResponse, RuntimeError> {
        let cleanup = lease.cleanup.as_ref().ok_or(RuntimeError::InvalidRequest)?;
        if !Arc::ptr_eq(&self.core, &cleanup.permit.core) {
            return Err(RuntimeError::InvalidRequest);
        }
        let (tx, rx) = oneshot::channel();
        let mut cleanup = lease.cleanup.take().ok_or(RuntimeError::InvalidRequest)?;
        cleanup.reply = Some(tx);
        enqueue_cleanup(cleanup);
        rx.await.unwrap_or(Err(RuntimeError::CleanupFailed))
    }
}

struct Failure {
    error: RuntimeError,
    executed: bool,
}
fn invoke<T, R>(
    core: &Core,
    reply: &oneshot::Sender<R>,
    call: impl Fn(&Extension) -> anyhow::Result<T>,
) -> Result<T, Failure> {
    for attempt in 0..ATTEMPTS {
        let unavailable = {
            let state = core.state.lock().unwrap();
            state.quarantined || state.disabled
        };
        if reply.is_closed() || unavailable {
            return Err(Failure {
                error: RuntimeError::Unavailable,
                executed: false,
            });
        }
        match catch_unwind(AssertUnwindSafe(|| call(&core.extension))) {
            Ok(Ok(value)) => return Ok(value),
            Ok(Err(error)) if error.downcast_ref::<SourceInstanceBusy>().is_some() => {
                if attempt + 1 < ATTEMPTS {
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
            _ => {
                return Err(Failure {
                    error: RuntimeError::CallFailed,
                    executed: true,
                })
            }
        }
    }
    Err(Failure {
        error: RuntimeError::Busy,
        executed: false,
    })
}

fn worker(core: Arc<Core>) {
    loop {
        let mut state = core.state.lock().unwrap();
        if let Some(mut cleanup) = state.cleanup.pop_front() {
            drop(state);
            let retry = cleanup.retry;
            let mut result = None;
            for attempt in 0..ATTEMPTS {
                if let Ok(Ok(response)) = catch_unwind(AssertUnwindSafe(|| {
                    core.extension.release_audio_v2(&cleanup.request)
                })) {
                    result = Some(response);
                    break;
                }
                if attempt + 1 < ATTEMPTS {
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
            let succeeded = result.is_some();
            if let Some(response) = result {
                let reply = cleanup.reply.take();
                drop(cleanup);
                if let Some(reply) = reply {
                    let _ = reply.send(Ok(response));
                }
            } else {
                let reply = cleanup.reply.take();
                let mut state = core.state.lock().unwrap();
                state.quarantined = true;
                state.failed.push(cleanup); // retain reservation AND original instance
                drop(state);
                if let Some(reply) = reply {
                    let _ = reply.send(Err(RuntimeError::CleanupFailed));
                }
            }
            if retry {
                let mut state = core.state.lock().unwrap();
                let batch = state.retry.as_mut().expect("owned cleanup retry batch");
                batch.remaining -= 1;
                batch.failed |= !succeeded;
                if batch.remaining == 0 {
                    let batch = state.retry.take().unwrap();
                    state.quarantined = !state.failed.is_empty();
                    drop(state);
                    let _ = batch.reply.send(if batch.failed {
                        Err(RuntimeError::CleanupFailed)
                    } else {
                        Ok(batch.total)
                    });
                }
            }
            continue;
        }
        if let Some(job) = state.jobs.pop_front() {
            drop(state);
            job();
            continue;
        }
        if state.reserved == 0 && state.handles == 0 {
            state.stopped = true;
            return;
        }
        #[cfg(test)]
        if let Some(entered) = core.before_idle_wait.lock().unwrap().take() {
            let _ = entered.send(());
        }
        // Handles and reservations change under this same lock and notify the
        // worker; no polling delay or lost wake can accumulate idle instances.
        drop(core.wake.wait(state).unwrap());
    }
}

#[cfg(test)]
mod tests;
