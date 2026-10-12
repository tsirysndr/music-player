//! Shared daemon account boundary; never expose an unguarded SourceRuntime.
//! All account/enable mutations must use this SAME manager (including remote UI
//! and CLI). Direct DB/file writes cannot participate in its linearization; DB
//! snapshots are still checked on each call, and installation changes require
//! rescan. Native saved-server accounts are deliberately outside this module.
//!
//! The gate covers DB checks and the FIRST poll that enqueues a scheduler call,
//! never guest completion. Disable can therefore reject queued work while an
//! executing call finishes. A final gated check suppresses stale results; dropping
//! a stale resolved value releases its owned lease on the original runtime. A
//! result already delivered before invalidation is the caller's responsibility
//! (Player's occurrence/generation check remains required).
use music_player_extensions::{
    abi::{self as legacy, source_v2 as wire},
    catalog,
    host::{Extension, HostContext},
    manifest::SourceProtocol,
    source_runtime::{
        ResolvedLegacySource, ResolvedSource, RuntimeError, RuntimeStatus, SourceRuntime,
    },
};
use music_player_storage::{
    extension_source_accounts::{self as accounts, AccountProtocol, ConfigChange, SourceAccount},
    extension_state, Database,
};
use music_player_types::source::{
    LocalSourceRef, RemoteIdentity, ResourceKind, SourceHandle, SourceRef,
};
use std::{
    collections::BTreeMap,
    fmt,
    future::{poll_fn, Future},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, OnceLock,
    },
    task::Poll,
};
use tokio::sync::{oneshot, OwnedSemaphorePermit, Semaphore};

const INSTANCE_LIMIT: usize = 16; // includes loading, retired leases and failed cleanup
const CALL_LIMIT: usize = 32;
static LOADS: OnceLock<Arc<Semaphore>> = OnceLock::new();
// All playback test modules exercising this process-wide budget share this gate.
#[cfg(test)]
pub(crate) static TEST_LOAD_SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
fn load_permit() -> Result<OwnedSemaphorePermit, AccountError> {
    LOADS
        .get_or_init(|| Arc::new(Semaphore::new(2)))
        .clone()
        .try_acquire_owned()
        .map_err(|_| AccountError::Busy)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountError {
    Busy,
    Storage,
    Missing,
    Disabled,
    InvalidSource,
    Identity,
    Changed,
    Load,
    Runtime(RuntimeError),
}
impl fmt::Display for AccountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Busy => "extension account capacity unavailable",
            Self::Storage => "extension account storage operation failed",
            Self::Missing => "extension source account not found",
            Self::Disabled => "source extension disabled",
            Self::InvalidSource => "invalid extension source or configuration",
            Self::Identity => "extension account identity missing or different",
            Self::Changed => "extension account or installation changed",
            Self::Load => "source extension load failed",
            Self::Runtime(_) => "extension source operation failed",
        })
    }
}
impl std::error::Error for AccountError {}
impl From<RuntimeError> for AccountError {
    fn from(value: RuntimeError) -> Self {
        Self::Runtime(value)
    }
}

struct Entry {
    runtime: SourceRuntime,
    protocol: AccountProtocol,
    effective_config: BTreeMap<String, String>,
    valid: Arc<AtomicBool>,
}
struct Slot {
    account_id: String,
    extension_id: String,
    revision: i64,
    generation: u64,
    valid: Arc<AtomicBool>,
    entry: Option<Arc<Entry>>,
}
struct State {
    generation: u64,
    admission_epoch: Arc<()>,
    slots: Vec<Slot>,
}
struct Inner {
    db: Database,
    paths: Vec<PathBuf>,
    context: HostContext,
    gate: tokio::sync::Mutex<()>,
    state: Mutex<State>,
    calls: Arc<Semaphore>,
    mutations: Arc<Semaphore>,
    #[cfg(test)]
    hooks: Mutex<TestHooks>,
}
#[derive(Clone)]
pub struct ExtensionAccounts {
    inner: Arc<Inner>,
}
/// Manifest-only form information. Configuration values may be credentials,
/// so clients receive declared keys and submit changes without echoing values.
pub struct SourceDefinition {
    pub extension_id: String,
    pub name: String,
    pub enabled: bool,
    pub protocol: AccountProtocol,
    pub config_keys: Vec<String>,
}
/// A verified snapshot, not a raw scheduler handle. Calls always recheck storage.
#[derive(Clone)]
pub struct AccountRuntime {
    inner: Arc<Inner>,
    snapshot: SourceAccount,
    entry: Arc<Entry>,
    // Only source_info during explicit connect may precede a local binding.
    connecting: bool,
}
#[derive(Clone, Debug)]
pub struct AccountInstanceStatus {
    pub account_id: String,
    pub config_revision: i64,
    pub installation_generation: u64,
    pub retired: bool,
    pub runtime: RuntimeStatus,
}

// Owned by the blocking load, then its returned result, until publication. Even
// an abandoned JoinHandle cannot free the slot while instantiation is running.
struct Loading {
    inner: Arc<Inner>,
    valid: Arc<AtomicBool>,
}
impl Drop for Loading {
    fn drop(&mut self) {
        self.inner
            .state
            .lock()
            .unwrap()
            .slots
            .retain(|slot| !Arc::ptr_eq(&slot.valid, &self.valid) || slot.entry.is_some());
    }
}
struct CancelLoad(Arc<AtomicBool>);
impl Drop for CancelLoad {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

impl Inner {
    fn call_permit(&self) -> Result<OwnedSemaphorePermit, AccountError> {
        self.calls
            .clone()
            .try_acquire_owned()
            .map_err(|_| AccountError::Busy)
    }
    fn invalidate(&self, matches: impl Fn(&Slot) -> bool) {
        let mut state = self.state.lock().unwrap();
        // Also invalidate acquisitions that read their row but have not yet
        // reserved a slot. Arc identity avoids a wrapping intent counter.
        state.admission_epoch = Arc::new(());
        for slot in &state.slots {
            if matches(slot) {
                slot.valid.store(false, Ordering::SeqCst);
                if let Some(entry) = &slot.entry {
                    entry.runtime.disable();
                }
            }
        }
    }
    fn prune(state: &mut State) {
        state.slots.retain(|slot| {
            let Some(entry) = &slot.entry else {
                return true;
            };
            slot.valid.load(Ordering::SeqCst)
                || Arc::strong_count(entry) != 1
                || entry.runtime.status().reserved != 0
        });
    }
    async fn snapshot(&self, id: &str) -> Result<SourceAccount, AccountError> {
        let row = accounts::get(self.db.get_connection(), id)
            .await
            .map_err(|_| AccountError::Storage)?
            .ok_or(AccountError::Missing)?;
        if !extension_state::source_enabled(self.db.get_connection(), &row.extension_id)
            .await
            .map_err(|_| AccountError::Storage)?
        {
            return Err(AccountError::Disabled);
        }
        if row.config_revision < 1 {
            return Err(AccountError::InvalidSource);
        }
        binding_shape(&row)?;
        Ok(row)
    }
    async fn check(&self, snapshot: &SourceAccount, entry: &Entry) -> Result<(), AccountError> {
        if !entry.valid.load(Ordering::SeqCst) {
            return Err(AccountError::Changed);
        }
        let row = self.snapshot(&snapshot.id).await?;
        if !same_config(snapshot, &row) {
            return Err(AccountError::Changed);
        }
        if account_protocol(snapshot)? != entry.protocol {
            return Err(AccountError::Changed);
        }
        // An unbound connect may race another successful first bind; the later
        // bind SQL still only permits the same pair. Bound requests require equality.
        if snapshot.remote_server_id.is_some() || snapshot.remote_user_id.is_some() {
            if identity(snapshot)? != identity(&row)? {
                return Err(AccountError::Identity);
            }
        }
        if snapshot.local_bound_revision.is_some()
            && (snapshot.local_bound_revision != row.local_bound_revision
                || snapshot.local_config_json != row.local_config_json)
        {
            return Err(AccountError::Identity);
        }
        Ok(())
    }
}
fn same_config(a: &SourceAccount, b: &SourceAccount) -> bool {
    a.id == b.id
        && a.extension_id == b.extension_id
        && a.source_api_version == b.source_api_version
        && a.config_revision == b.config_revision
        && a.config_json == b.config_json
        && a.created_at == b.created_at
}
fn identity(row: &SourceAccount) -> Result<wire::SourceIdentity, AccountError> {
    if account_protocol(row)? != AccountProtocol::V2 {
        return Err(AccountError::Identity);
    }
    match (&row.remote_server_id, &row.remote_user_id) {
        (Some(server), Some(user)) if !server.trim().is_empty() && !user.trim().is_empty() => {
            Ok(wire::SourceIdentity {
                server_id: server.clone(),
                user_id: user.clone(),
            })
        }
        _ => Err(AccountError::Identity),
    }
}
fn account_protocol(row: &SourceAccount) -> Result<AccountProtocol, AccountError> {
    accounts::protocol(row).map_err(|_| AccountError::InvalidSource)
}
fn declared_protocol(protocol: Option<SourceProtocol>) -> Result<AccountProtocol, AccountError> {
    match protocol {
        Some(SourceProtocol::V1) => Ok(AccountProtocol::V1),
        Some(SourceProtocol::V2) => Ok(AccountProtocol::V2),
        None => Err(AccountError::InvalidSource),
    }
}
fn binding_shape(row: &SourceAccount) -> Result<(), AccountError> {
    let valid = match account_protocol(row)? {
        AccountProtocol::V1 => {
            row.remote_server_id.is_none()
                && row.remote_user_id.is_none()
                && row.local_bound_revision.is_some() == row.local_config_json.is_some()
                && row
                    .local_bound_revision
                    .is_none_or(|revision| revision == row.config_revision)
        }
        AccountProtocol::V2 => {
            row.local_bound_revision.is_none()
                && row.local_config_json.is_none()
                && row.remote_server_id.is_some() == row.remote_user_id.is_some()
        }
    };
    if valid {
        Ok(())
    } else {
        Err(AccountError::Identity)
    }
}
fn local_binding(row: &SourceAccount) -> Result<i64, AccountError> {
    binding_shape(row)?;
    if account_protocol(row)? != AccountProtocol::V1 {
        return Err(AccountError::Identity);
    }
    row.local_bound_revision.ok_or(AccountError::Identity)
}
fn check_local_source(row: &SourceAccount, source: &LocalSourceRef) -> Result<(), AccountError> {
    SourceHandle::parse(&source.to_handle()).map_err(|_| AccountError::InvalidSource)?;
    if source.extension_id() != Some(row.extension_id.as_str())
        || source.account_id != row.id
        || source.config_revision != local_binding(row)? as u64
    {
        return Err(AccountError::Identity);
    }
    Ok(())
}
fn check_handle(row: &SourceAccount, source: &SourceHandle) -> Result<(), AccountError> {
    match source {
        SourceHandle::Remote(source) => check_source(row, source),
        SourceHandle::Local(source) => check_local_source(row, source),
    }
}
fn check_source(row: &SourceAccount, source: &SourceRef) -> Result<(), AccountError> {
    SourceRef::parse(&source.to_handle()).map_err(|_| AccountError::InvalidSource)?;
    if source.extension_id() != Some(row.extension_id.as_str()) || source.account_id != row.id {
        return Err(AccountError::Identity);
    }
    let actual = identity(row)?;
    if source.remote.server_id != actual.server_id || source.remote.user_id != actual.user_id {
        return Err(AccountError::Identity);
    }
    Ok(())
}

impl ExtensionAccounts {
    /// Construct ONCE per daemon and share clones with Player and all API surfaces.
    pub fn new(db: Database, paths: Vec<PathBuf>, context: HostContext) -> Self {
        Self {
            inner: Arc::new(Inner {
                db,
                paths,
                context,
                gate: tokio::sync::Mutex::new(()),
                state: Mutex::new(State {
                    generation: 1,
                    admission_epoch: Arc::new(()),
                    slots: Vec::new(),
                }),
                calls: Arc::new(Semaphore::new(CALL_LIMIT)),
                mutations: Arc::new(Semaphore::new(1)),
                #[cfg(test)]
                hooks: Mutex::new(TestHooks::default()),
            }),
        }
    }
    pub fn status(&self) -> Vec<AccountInstanceStatus> {
        let mut state = self.inner.state.lock().unwrap();
        Inner::prune(&mut state);
        state
            .slots
            .iter()
            .filter_map(|slot| {
                slot.entry.as_ref().map(|entry| AccountInstanceStatus {
                    account_id: slot.account_id.clone(),
                    config_revision: slot.revision,
                    installation_generation: slot.generation,
                    retired: !slot.valid.load(Ordering::SeqCst),
                    runtime: entry.runtime.status(),
                })
            })
            .collect()
    }
    /// Read from the same database as playback, without loading a guest.
    pub async fn list(&self) -> Result<Vec<SourceAccount>, AccountError> {
        let _permit = self.inner.call_permit()?;
        let _gate = self.inner.gate.lock().await;
        accounts::list(self.inner.db.get_connection())
            .await
            .map_err(|_| AccountError::Storage)
    }
    pub async fn get(&self, id: &str) -> Result<SourceAccount, AccountError> {
        let _permit = self.inner.call_permit()?;
        let _gate = self.inner.gate.lock().await;
        // A disabled/uninstalled source must remain inspectable and removable.
        accounts::get(self.inner.db.get_connection(), id)
            .await
            .map_err(|_| AccountError::Storage)?
            .ok_or(AccountError::Missing)
    }
    /// The daemon catalog, including non-source extensions. Reading this does
    /// not instantiate a guest or claim that an enabled module has loaded.
    pub async fn installed(&self) -> Result<Vec<catalog::Installed>, AccountError> {
        let permit = self.inner.call_permit()?;
        let enabled = extension_state::enabled_map(self.inner.db.get_connection()).await;
        let paths = self.inner.paths.clone();
        tokio::task::spawn_blocking(move || {
            // Catalog scans are bounded calls, not WASM instantiations. Keep
            // the permit until the scan exits even if its caller disappears.
            let _permit = permit;
            catalog::installed(&paths, &enabled)
        })
        .await
        .map_err(|_| AccountError::Load)
    }
    pub async fn definitions(&self) -> Result<Vec<SourceDefinition>, AccountError> {
        Ok(self
            .installed()
            .await?
            .into_iter()
            .filter_map(|item| {
                let protocol = item.manifest.source_protocol().ok()?;
                let protocol = declared_protocol(protocol).ok()?;
                Some(SourceDefinition {
                    enabled: item.enabled(),
                    extension_id: item.manifest.id,
                    name: item.manifest.name,
                    protocol,
                    config_keys: item.manifest.permissions.config.into_keys().collect(),
                })
            })
            .collect())
    }
    /// DB-only preflight for batch validation. Does not bind, load or run a guest.
    pub async fn validate_saved(&self, source: &SourceRef) -> Result<(), AccountError> {
        self.validate_handle(&SourceHandle::Remote(source.clone()))
            .await
    }
    pub async fn validate_handle(&self, source: &SourceHandle) -> Result<(), AccountError> {
        let _permit = self.inner.call_permit()?;
        let _gate = self.inner.gate.lock().await;
        let row = self.inner.snapshot(source.account_id()).await?;
        check_handle(&row, source)
    }
    /// The sole operation allowed to bind a previously unbound account.
    pub async fn connect(&self, id: &str) -> Result<SourceAccount, AccountError> {
        Ok(self.acquire(id, None, true).await?.snapshot)
    }
    pub async fn for_source(&self, source: &SourceRef) -> Result<AccountRuntime, AccountError> {
        self.for_handle(&SourceHandle::Remote(source.clone())).await
    }
    pub async fn for_local_source(
        &self,
        source: &LocalSourceRef,
    ) -> Result<AccountRuntime, AccountError> {
        self.for_handle(&SourceHandle::Local(source.clone())).await
    }
    pub async fn for_handle(&self, source: &SourceHandle) -> Result<AccountRuntime, AccountError> {
        // Malformed/non-extension references fail before DB/guest work.
        SourceHandle::parse(&source.to_handle()).map_err(|_| AccountError::InvalidSource)?;
        if source.extension_id().is_none() {
            return Err(AccountError::InvalidSource);
        }
        self.acquire(source.account_id(), Some(source), false).await
    }
    /// Root browse for an explicitly selected, already bound saved account.
    pub async fn for_account(&self, id: &str) -> Result<AccountRuntime, AccountError> {
        self.acquire(id, None, false).await
    }
    async fn acquire(
        &self,
        id: &str,
        source: Option<&SourceHandle>,
        connect: bool,
    ) -> Result<AccountRuntime, AccountError> {
        let _permit = self.inner.call_permit()?;
        let (snapshot, epoch) = {
            let _gate = self.inner.gate.lock().await;
            let row = self.inner.snapshot(id).await?;
            if let Some(source) = source {
                check_handle(&row, source)?;
            }
            if !connect {
                match account_protocol(&row)? {
                    AccountProtocol::V1 => {
                        local_binding(&row)?;
                    }
                    AccountProtocol::V2 => {
                        identity(&row)?;
                    }
                }
            }
            (
                row,
                self.inner.state.lock().unwrap().admission_epoch.clone(),
            )
        };
        let entry = self.load(&snapshot, &epoch).await?;
        let mut bound = AccountRuntime {
            inner: self.inner.clone(),
            snapshot,
            entry,
            connecting: connect,
        };
        if bound.entry.protocol == AccountProtocol::V1 {
            if connect {
                bound.dispatch(bound.entry.runtime.source_info_v1()).await?;
                #[cfg(test)]
                self.test_pause(TestPoint::Identity).await;
                let snapshot = bound.snapshot.clone();
                let entry = bound.entry.clone();
                // The binding and retirement remain under one owned mutation
                // even if connect loses its waiting caller during the SQL.
                bound.snapshot = self
                    .mutation(move |inner| async move {
                        inner.check(&snapshot, &entry).await?;
                        let updated = accounts::bind_local_configuration(
                            inner.db.get_connection(),
                            &snapshot,
                            &entry.effective_config,
                        )
                        .await
                        .map_err(|_| AccountError::Changed)?;
                        if updated.config_revision != snapshot.config_revision {
                            inner.invalidate(|slot| slot.account_id == snapshot.id);
                        }
                        Ok(updated)
                    })
                    .await?;
            } else {
                let _gate = self.inner.gate.lock().await;
                bound.check().await?;
            }
            // Only connect consumes the newly bound snapshot; an updated scope
            // retires this entry and the next ordinary acquisition loads afresh.
            bound.connecting = false;
            return Ok(bound);
        }
        let actual = bound
            .dispatch(
                bound
                    .entry
                    .runtime
                    .source_identity_v2(wire::IdentityRequest {}),
            )
            .await?;
        #[cfg(test)]
        self.test_pause(TestPoint::Identity).await;
        if !connect {
            let _gate = self.inner.gate.lock().await;
            bound.check().await?;
            if actual != identity(&bound.snapshot)? {
                return Err(AccountError::Identity);
            }
            return Ok(bound);
        }
        let snapshot = bound.snapshot.clone();
        let entry = bound.entry.clone();
        bound.snapshot = self
            .mutation(move |inner| async move {
                inner.check(&snapshot, &entry).await?;
                accounts::bind_remote_identity(
                    inner.db.get_connection(),
                    &snapshot,
                    &RemoteIdentity {
                        server_id: actual.server_id,
                        user_id: actual.user_id,
                    },
                )
                .await
                .map_err(|_| AccountError::Identity)
            })
            .await?;
        bound.connecting = false;
        Ok(bound)
    }
    async fn load(
        &self,
        snapshot: &SourceAccount,
        epoch: &Arc<()>,
    ) -> Result<Arc<Entry>, AccountError> {
        let (loading, permit) = {
            let _gate = self.inner.gate.lock().await;
            let current = self.inner.snapshot(&snapshot.id).await?;
            if !same_config(snapshot, &current) {
                return Err(AccountError::Changed);
            }
            let mut state = self.inner.state.lock().unwrap();
            if !Arc::ptr_eq(epoch, &state.admission_epoch) {
                return Err(AccountError::Changed);
            }
            Inner::prune(&mut state);
            if let Some(slot) = state.slots.iter().find(|slot| {
                slot.account_id == snapshot.id
                    && slot.revision == snapshot.config_revision
                    && slot.generation == state.generation
                    && slot.valid.load(Ordering::SeqCst)
            }) {
                return slot.entry.clone().ok_or(AccountError::Busy);
            }
            // Evict only an idle entry with no external guarded handle or lease.
            if state.slots.len() >= INSTANCE_LIMIT {
                if let Some(index) = state.slots.iter().position(|slot| {
                    slot.entry.as_ref().is_some_and(|entry| {
                        Arc::strong_count(entry) == 1 && entry.runtime.status().reserved == 0
                    })
                }) {
                    let slot = state.slots.remove(index);
                    slot.valid.store(false, Ordering::SeqCst);
                    slot.entry.unwrap().runtime.disable();
                } else {
                    return Err(AccountError::Busy);
                }
            }
            let permit = load_permit()?;
            let valid = Arc::new(AtomicBool::new(true));
            let generation = state.generation;
            state.slots.push(Slot {
                account_id: snapshot.id.clone(),
                extension_id: snapshot.extension_id.clone(),
                revision: snapshot.config_revision,
                generation,
                valid: valid.clone(),
                entry: None,
            });
            (
                Loading {
                    inner: self.inner.clone(),
                    valid,
                },
                permit,
            )
        };
        let cancelled = Arc::new(AtomicBool::new(false));
        let _cancel = CancelLoad(cancelled.clone());
        let snapshot_for_load = snapshot.clone();
        #[cfg(test)]
        let hook = self.inner.hooks.lock().unwrap().load.take();
        let task = tokio::task::spawn_blocking(move || {
            let _permit = permit; // remains owned until load/destruction actually finishes
            #[cfg(test)]
            if let Some(hook) = hook {
                hook();
            }
            if cancelled.load(Ordering::SeqCst) || !loading.valid.load(Ordering::SeqCst) {
                return Err(AccountError::Changed);
            }
            let overrides =
                accounts::config(&snapshot_for_load).map_err(|_| AccountError::InvalidSource)?;
            let installed = catalog::installed(&loading.inner.paths, &BTreeMap::new())
                .into_iter()
                .find(|item| item.manifest.id == snapshot_for_load.extension_id)
                .ok_or(AccountError::Load)?;
            let protocol = account_protocol(&snapshot_for_load)?;
            if declared_protocol(
                installed
                    .manifest
                    .source_protocol()
                    .map_err(|_| AccountError::Load)?,
            )? != protocol
                || !overrides
                    .keys()
                    .all(|key| installed.manifest.permissions.config.contains_key(key))
            {
                return Err(AccountError::InvalidSource);
            }
            let extension =
                Extension::load(&installed.dir, &overrides, loading.inner.context.clone())
                    .map_err(|_| AccountError::Load)?;
            if extension.manifest.id != snapshot_for_load.extension_id
                || declared_protocol(extension.loaded_source_protocol())? != protocol
                || !overrides
                    .keys()
                    .all(|key| extension.manifest.permissions.config.contains_key(key))
            {
                return Err(AccountError::InvalidSource);
            }
            if cancelled.load(Ordering::SeqCst) || !loading.valid.load(Ordering::SeqCst) {
                return Err(AccountError::Changed);
            }
            let effective_config = extension.config().clone();
            let runtime = SourceRuntime::new(Arc::new(extension))?;
            let entry = Arc::new(Entry {
                runtime,
                protocol,
                effective_config,
                valid: loading.valid.clone(),
            });
            Ok((loading, entry))
        });
        let (loading, entry) = task.await.map_err(|_| AccountError::Load)??;
        let _gate = self.inner.gate.lock().await;
        self.inner.check(snapshot, &entry).await?;
        let mut state = self.inner.state.lock().unwrap();
        let slot = state
            .slots
            .iter_mut()
            .find(|slot| Arc::ptr_eq(&slot.valid, &loading.valid))
            .ok_or(AccountError::Changed)?;
        slot.entry = Some(entry.clone());
        drop(state);
        drop(loading);
        Ok(entry)
    }

    // Metadata inspection also consumes the process-wide blocking-load budget.
    // No gate is held while it scans; generation must match again at commit.
    async fn declared(
        &self,
        plugin: &str,
        values: &BTreeMap<String, String>,
    ) -> Result<(u64, AccountProtocol), AccountError> {
        let generation = self.inner.state.lock().unwrap().generation;
        let paths = self.inner.paths.clone();
        let plugin = plugin.to_owned();
        let keys: Vec<_> = values.keys().cloned().collect();
        let permit = load_permit()?;
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let installed = catalog::installed(&paths, &BTreeMap::new())
                .into_iter()
                .find(|item| item.manifest.id == plugin)
                .ok_or(AccountError::Load)?;
            let protocol = declared_protocol(
                installed
                    .manifest
                    .source_protocol()
                    .map_err(|_| AccountError::Load)?,
            )?;
            if !keys
                .iter()
                .all(|key| installed.manifest.permissions.config.contains_key(key))
            {
                return Err(AccountError::InvalidSource);
            }
            Ok((generation, protocol))
        })
        .await
        .map_err(|_| AccountError::Load)?
    }
    /// DB mutations keep their single bounded task and gate if the caller drops.
    /// A lost reply means outcome unknown, not rollback; never auto-retry create.
    async fn mutation<T, F, Fut>(&self, action: F) -> Result<T, AccountError>
    where
        T: Send + 'static,
        F: FnOnce(Arc<Inner>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, AccountError>> + Send + 'static,
    {
        let permit = self
            .inner
            .mutations
            .clone()
            .try_acquire_owned()
            .map_err(|_| AccountError::Busy)?;
        let inner = self.inner.clone();
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            let _permit = permit;
            let _gate = inner.gate.lock().await;
            // No mutation began if its waiting caller already disappeared.
            if tx.is_closed() {
                return;
            }
            let result = action(inner.clone()).await;
            let _ = tx.send(result);
        });
        rx.await.map_err(|_| AccountError::Storage)?
    }
    pub async fn create(
        &self,
        plugin: &str,
        name: &str,
        values: BTreeMap<String, String>,
        now: &str,
    ) -> Result<SourceAccount, AccountError> {
        let (generation, protocol) = self.declared(plugin, &values).await?;
        let (plugin, name, now) = (plugin.to_owned(), name.to_owned(), now.to_owned());
        self.mutation(move |inner| async move {
            if inner.state.lock().unwrap().generation != generation {
                return Err(AccountError::Changed);
            }
            accounts::create_for_protocol(
                inner.db.get_connection(),
                &plugin,
                &name,
                &values,
                &now,
                protocol,
            )
            .await
            .map_err(|_| AccountError::Storage)
        })
        .await
    }
    pub async fn update(
        &self,
        snapshot: &SourceAccount,
        name: &str,
        changes: BTreeMap<String, ConfigChange>,
        now: &str,
    ) -> Result<SourceAccount, AccountError> {
        // Renaming a saved account does not depend on its plugin still being
        // installed. Configuration edits still require the current declaration.
        let generation = if changes.is_empty() {
            None
        } else {
            let mut values = accounts::config(snapshot).map_err(|_| AccountError::InvalidSource)?;
            for (key, change) in &changes {
                match change {
                    ConfigChange::Set(value) => {
                        values.insert(key.clone(), value.clone());
                    }
                    ConfigChange::Remove => {
                        values.remove(key);
                    }
                }
            }
            let (generation, protocol) = self.declared(&snapshot.extension_id, &values).await?;
            if protocol != account_protocol(snapshot)? {
                return Err(AccountError::Changed);
            }
            Some(generation)
        };
        let (snapshot, name, now) = (snapshot.clone(), name.to_owned(), now.to_owned());
        self.mutation(move |inner| async move {
            if generation.is_some_and(|expected| inner.state.lock().unwrap().generation != expected)
            {
                return Err(AccountError::Changed);
            }
            // The owned mutation task holds the gate through SQL and invalidation,
            // even if its caller disappears. No final publication can cross a
            // successful config edit; rename/no-op/failed CAS must preserve handles.
            let updated =
                accounts::update(inner.db.get_connection(), &snapshot, &name, &changes, &now)
                    .await
                    .map_err(|_| AccountError::Changed)?;
            if updated.config_revision != snapshot.config_revision {
                inner.invalidate(|slot| slot.account_id == snapshot.id);
            }
            Ok(updated)
        })
        .await
    }
    pub async fn remove(&self, id: &str) -> Result<(), AccountError> {
        let id = id.to_owned();
        self.mutation(move |inner| async move {
            inner.invalidate(|slot| slot.account_id == id);
            accounts::remove(inner.db.get_connection(), &id)
                .await
                .map_err(|_| AccountError::Storage)
        })
        .await
    }
    pub async fn set_enabled(
        &self,
        plugin: &str,
        enabled: bool,
        now: &str,
    ) -> Result<(), AccountError> {
        let (plugin, now) = (plugin.to_owned(), now.to_owned());
        self.mutation(move |inner| async move {
            inner.invalidate(|slot| slot.extension_id == plugin);
            extension_state::set_enabled(inner.db.get_connection(), &plugin, enabled, &now)
                .await
                .map_err(|_| AccountError::Storage)
        })
        .await
    }
    /// Catalog changes are lazy: invalidate every old installation generation,
    /// then the next acquire scans paths again. Never prune saved accounts here.
    pub async fn rescan(&self) -> Result<(), AccountError> {
        self.mutation(|inner| async move {
            {
                let mut state = inner.state.lock().unwrap();
                state.generation = state
                    .generation
                    .checked_add(1)
                    .ok_or(AccountError::Changed)?;
            }
            inner.invalidate(|_| true);
            // Preference pruning participates in the same mutation gate as
            // enable/disable. API callers must not prune independently after
            // rescan, where a concurrent toggle could otherwise be erased.
            let installed: Vec<String> = catalog::installed(&inner.paths, &BTreeMap::new())
                .into_iter()
                .map(|entry| entry.manifest.id)
                .collect();
            if extension_state::prune(inner.db.get_connection(), &installed)
                .await
                .is_err()
            {
                tracing::warn!("could not prune stale extension preferences during rescan");
            }
            Ok(())
        })
        .await
    }
    /// Retry a finite snapshot of original runtimes, including deleted accounts.
    /// No entry is removed on error and disabled runtimes are never re-enabled.
    pub async fn retry_failed_cleanup(&self, account_id: &str) -> Result<usize, AccountError> {
        let _permit = self.inner.call_permit()?;
        let entries: Vec<_> = self
            .inner
            .state
            .lock()
            .unwrap()
            .slots
            .iter()
            .filter(|slot| slot.account_id == account_id)
            .filter_map(|slot| slot.entry.clone())
            .collect();
        let mut recovered = 0;
        let mut failed = None;
        for entry in entries {
            match entry.runtime.retry_failed_cleanup().await {
                Ok(count) => recovered += count,
                Err(error) => failed = Some(error),
            }
        }
        if let Some(error) = failed {
            Err(error.into())
        } else {
            Ok(recovered)
        }
    }
}

impl AccountRuntime {
    pub fn protocol(&self) -> AccountProtocol {
        self.entry.protocol
    }
    /// Persist the guest item ID in the scope actually verified by this runtime.
    pub fn source_handle(
        &self,
        kind: ResourceKind,
        item_id: String,
    ) -> Result<SourceHandle, AccountError> {
        let resolver = SourceRef::extension_resolver(&self.snapshot.extension_id)
            .map_err(|_| AccountError::InvalidSource)?;
        let handle = match self.entry.protocol {
            AccountProtocol::V1 => SourceHandle::Local(LocalSourceRef {
                resolver,
                account_id: self.snapshot.id.clone(),
                config_revision: local_binding(&self.snapshot)? as u64,
                kind,
                item_id,
            }),
            AccountProtocol::V2 => SourceHandle::Remote(SourceRef {
                resolver,
                account_id: self.snapshot.id.clone(),
                remote: self.remote_identity()?,
                kind,
                item_id,
            }),
        };
        SourceHandle::parse(&handle.to_handle()).map_err(|_| AccountError::InvalidSource)?;
        Ok(handle)
    }
    async fn check(&self) -> Result<(), AccountError> {
        self.inner.check(&self.snapshot, &self.entry).await?;
        if self.entry.protocol == AccountProtocol::V1 && !self.connecting {
            local_binding(&self.snapshot)?;
            let bound: BTreeMap<String, String> = serde_json::from_str(
                self.snapshot
                    .local_config_json
                    .as_deref()
                    .ok_or(AccountError::Identity)?,
            )
            .map_err(|_| AccountError::Identity)?;
            if bound != self.entry.effective_config {
                return Err(AccountError::Changed);
            }
        }
        Ok(())
    }
    pub fn account_id(&self) -> &str {
        &self.snapshot.id
    }
    pub fn extension_id(&self) -> &str {
        &self.snapshot.extension_id
    }
    pub fn remote_identity(&self) -> Result<RemoteIdentity, AccountError> {
        let value = identity(&self.snapshot)?;
        Ok(RemoteIdentity {
            server_id: value.server_id,
            user_id: value.user_id,
        })
    }
    async fn dispatch<T>(
        &self,
        future: impl Future<Output = Result<T, RuntimeError>>,
    ) -> Result<T, AccountError> {
        let _permit = self.inner.call_permit()?;
        let mut future = std::pin::pin!(future);
        let first = {
            let _gate = self.inner.gate.lock().await;
            self.check().await?;
            // All SourceRuntime methods enqueue synchronously in their first poll.
            // Poll once under the gate; NEVER hold it waiting for a guest result.
            poll_fn(|cx| Poll::Ready(future.as_mut().poll(cx))).await
        };
        let result = match first {
            Poll::Ready(result) => result,
            Poll::Pending => future.await,
        };
        #[cfg(test)]
        ExtensionAccounts {
            inner: self.inner.clone(),
        }
        .test_pause(TestPoint::Delivery)
        .await;
        let _gate = self.inner.gate.lock().await;
        self.check().await?;
        result.map_err(Into::into)
    }
    pub async fn browse(
        &self,
        parent: Option<&SourceRef>,
        query: Option<String>,
        offset: u64,
        limit: u32,
    ) -> Result<wire::BrowseResponse, AccountError> {
        if let Some(parent) = parent {
            check_source(&self.snapshot, parent)?;
            if parent.kind != ResourceKind::Container {
                return Err(AccountError::InvalidSource);
            }
        }
        self.dispatch(self.entry.runtime.browse_media_v2(wire::BrowseRequest {
            identity: identity(&self.snapshot)?,
            parent_id: parent.map(|p| p.item_id.clone()),
            query,
            offset,
            limit,
        }))
        .await
    }
    fn item_id(&self, source: &SourceRef) -> Result<String, AccountError> {
        check_source(&self.snapshot, source)?;
        if source.kind != ResourceKind::Item {
            return Err(AccountError::InvalidSource);
        }
        Ok(source.item_id.clone())
    }
    pub async fn item(&self, source: &SourceRef) -> Result<wire::ItemResponse, AccountError> {
        self.dispatch(self.entry.runtime.get_media_item_v2(wire::ItemRequest {
            identity: identity(&self.snapshot)?,
            item_id: self.item_id(source)?,
        }))
        .await
    }
    pub async fn audio_options(
        &self,
        source: &SourceRef,
    ) -> Result<wire::AudioOptionsResponse, AccountError> {
        self.dispatch(
            self.entry
                .runtime
                .get_audio_options_v2(wire::AudioOptionsRequest {
                    identity: identity(&self.snapshot)?,
                    item_id: self.item_id(source)?,
                }),
        )
        .await
    }
    /// Request IDs/selection/offset/format validation remain in the typed host.
    /// The manager additionally prevents supplying another account/item identity.
    pub async fn resolve(
        &self,
        source: &SourceRef,
        request: wire::ResolveRequest,
    ) -> Result<ResolvedSource, AccountError> {
        if request.item_id != self.item_id(source)? || request.identity != identity(&self.snapshot)?
        {
            return Err(AccountError::Identity);
        }
        self.dispatch(self.entry.runtime.resolve_audio_v2(request))
            .await
    }
    pub async fn legacy_info(&self) -> Result<legacy::SourceInfo, AccountError> {
        local_binding(&self.snapshot)?;
        self.dispatch(self.entry.runtime.source_info_v1()).await
    }
    fn legacy_browse_request(
        &self,
        parent: Option<&LocalSourceRef>,
        query: Option<String>,
        offset: u64,
        limit: u32,
    ) -> Result<legacy::BrowseRequest, AccountError> {
        local_binding(&self.snapshot)?;
        if let Some(parent) = parent {
            check_local_source(&self.snapshot, parent)?;
            if parent.kind == ResourceKind::Item {
                return Err(AccountError::InvalidSource);
            }
        }
        if limit == 0 {
            return Err(AccountError::InvalidSource);
        }
        Ok(legacy::BrowseRequest {
            parent_id: parent
                .map(|value| value.item_id.clone())
                .unwrap_or_default(),
            query: query.unwrap_or_default(),
            offset: offset.try_into().map_err(|_| AccountError::InvalidSource)?,
            limit,
        })
    }
    pub async fn legacy_albums(
        &self,
        parent: Option<&LocalSourceRef>,
        query: Option<String>,
        offset: u64,
        limit: u32,
    ) -> Result<Vec<legacy::SourceAlbum>, AccountError> {
        let request = self.legacy_browse_request(parent, query, offset, limit)?;
        self.dispatch(self.entry.runtime.list_albums_v1(request))
            .await
    }
    pub async fn legacy_artists(
        &self,
        parent: Option<&LocalSourceRef>,
        query: Option<String>,
        offset: u64,
        limit: u32,
    ) -> Result<Vec<legacy::SourceArtist>, AccountError> {
        let request = self.legacy_browse_request(parent, query, offset, limit)?;
        self.dispatch(self.entry.runtime.list_artists_v1(request))
            .await
    }
    pub async fn legacy_tracks(
        &self,
        parent: Option<&LocalSourceRef>,
        query: Option<String>,
        offset: u64,
        limit: u32,
    ) -> Result<Vec<legacy::TrackInfo>, AccountError> {
        let request = self.legacy_browse_request(parent, query, offset, limit)?;
        self.dispatch(self.entry.runtime.list_tracks_v1(request))
            .await
    }
    pub async fn legacy_playlists(
        &self,
        parent: Option<&LocalSourceRef>,
        query: Option<String>,
        offset: u64,
        limit: u32,
    ) -> Result<Vec<legacy::SourcePlaylist>, AccountError> {
        let request = self.legacy_browse_request(parent, query, offset, limit)?;
        self.dispatch(self.entry.runtime.list_playlists_v1(request))
            .await
    }
    /// V1 resolves only an item ID; selection/offset capabilities are checked by
    /// the playback adapter before calling here. Its lease is local ownership.
    pub async fn resolve_legacy(
        &self,
        source: &LocalSourceRef,
    ) -> Result<ResolvedLegacySource, AccountError> {
        check_local_source(&self.snapshot, source)?;
        if source.kind != ResourceKind::Item {
            return Err(AccountError::InvalidSource);
        }
        self.dispatch(self.entry.runtime.resolve_audio_v1(legacy::StreamRequest {
            track_id: source.item_id.clone(),
        }))
        .await
    }
}

#[cfg(test)]
#[derive(Default)]
struct TestHooks {
    load: Option<Box<dyn FnOnce() + Send>>,
    identity: Option<TestPause>,
    delivery: Option<TestPause>,
}
#[cfg(test)]
struct TestPause {
    entered: oneshot::Sender<()>,
    resume: oneshot::Receiver<()>,
}
#[cfg(test)]
enum TestPoint {
    Identity,
    Delivery,
}
#[cfg(test)]
impl ExtensionAccounts {
    async fn test_pause(&self, point: TestPoint) {
        let hook = {
            let mut hooks = self.inner.hooks.lock().unwrap();
            match point {
                TestPoint::Identity => hooks.identity.take(),
                TestPoint::Delivery => hooks.delivery.take(),
            }
        };
        if let Some(hook) = hook {
            let _ = hook.entered.send(());
            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), hook.resume).await;
        }
    }
}
#[cfg(test)]
mod tests;
