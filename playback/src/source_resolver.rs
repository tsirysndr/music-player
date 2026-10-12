//! Resolve queued sources by their saved account, independent of library browsing.
//! Native operations authenticate fresh snapshots. Extensions share the daemon
//! account manager, which owns configured instances and their cleanup lifetimes.

use crate::source_descriptor::SourceDescriptor;
use crate::{extension_accounts::ExtensionAccounts, source_extension, source_extension_v1};
use music_player_provider::{
    backends::emby::Emby,
    emby_playback::{AudioSelection, ResolvedAudio},
    MusicProvider, ProviderConfig, ProviderError,
};
use music_player_storage::{saved_servers, saved_servers::SavedServer, Database};
use music_player_types::{
    audio::AudioOptions,
    source::{ResourceKind, SourceHandle, SourceRef},
    types::Track,
};
use std::{sync::Arc, time::Duration};

const RESOLVE_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct SourceResolver {
    db: Database,
    device_id: String,
    follow_redirects: bool,
    extensions: Option<ExtensionDispatch>,
}

#[derive(Clone)]
struct ExtensionDispatch {
    accounts: ExtensionAccounts,
    format_exts: Vec<String>,
}

/// One complete plugin page, with host-bound stable handles.
pub struct SourceMediaPage {
    pub items: Vec<music_player_provider::MediaEntry>,
    pub next_offset: Option<u64>,
}

impl SourceResolver {
    /// The host supplies its device identity and redirect setting (default true).
    pub fn new(db: Database, device_id: String, follow_redirects: bool) -> Self {
        Self {
            db,
            device_id,
            follow_redirects,
            extensions: None,
        }
    }

    /// Inject the SAME manager into Player and API resolvers. The daemon supplies
    /// formats supported by its forward-only decoder; file formats requiring
    /// random access must not be advertised here merely because files play.
    pub fn with_extensions(
        mut self,
        accounts: ExtensionAccounts,
        format_exts: Vec<String>,
    ) -> Self {
        self.extensions = Some(ExtensionDispatch {
            accounts,
            format_exts,
        });
        self
    }

    /// API mutations must use the same account manager as playback.
    pub fn extension_accounts(&self) -> Result<&ExtensionAccounts, ProviderError> {
        Ok(&self.extension_dispatch()?.accounts)
    }

    fn extension_dispatch(&self) -> Result<&ExtensionDispatch, ProviderError> {
        self.extensions
            .as_ref()
            .ok_or_else(|| failure("extension source runtime is not configured"))
    }

    /// The deadline covers storage, authentication, identity verification and
    /// playback resolution together. Dropping the future cancels this work; an
    /// acquired encoding lease retains the provider's bounded Drop cleanup.
    pub async fn resolve(
        &self,
        source: &SourceRef,
        selection: &AudioSelection,
        offset_ms: u64,
    ) -> Result<ResolvedAudio, ProviderError> {
        tokio::time::timeout(RESOLVE_TIMEOUT, async {
            self.authenticated(source)
                .await?
                .resolve_audio(source, selection, offset_ms)
                .await
        })
        .await
        .map_err(|_| failure("saved source resolution timed out"))?
    }

    /// Dispatch through the saved source namespace, never the browsing provider.
    pub(crate) async fn resolve_playback(
        &self,
        source: &SourceHandle,
        selection: &AudioSelection,
        offset_ms: u64,
    ) -> Result<SourceDescriptor, ProviderError> {
        require_handle_item(source)?;
        let source = match source {
            SourceHandle::Remote(source) => source,
            SourceHandle::Local(source) => {
                // V1 cannot express selection or an offset. Reject before loading
                // the guest, and never reinterpret a failed v2 call as v1.
                source_extension_v1::request(&source.item_id, selection, offset_ms)?;
                return tokio::time::timeout(RESOLVE_TIMEOUT, async {
                    let dispatch = self.extension_dispatch()?;
                    let runtime = dispatch
                        .accounts
                        .for_local_source(source)
                        .await
                        .map_err(extension_failure)?;
                    let resolved = runtime
                        .resolve_legacy(source)
                        .await
                        .map_err(extension_failure)?;
                    source_extension_v1::descriptor(
                        resolved,
                        &dispatch.format_exts,
                        self.follow_redirects,
                    )
                })
                .await
                .map_err(|_| failure("saved source resolution timed out"))?;
            }
        };
        if source.extension_id().is_some() {
            return tokio::time::timeout(RESOLVE_TIMEOUT, async {
                let dispatch = self.extension_dispatch()?;
                let request = source_extension::resolve_request(
                    source,
                    selection,
                    offset_ms,
                    dispatch.format_exts.clone(),
                    self.follow_redirects,
                )?;
                let runtime = dispatch
                    .accounts
                    .for_source(source)
                    .await
                    .map_err(extension_failure)?;
                let resolved = runtime
                    .resolve(source, request)
                    .await
                    .map_err(extension_failure)?;
                source_extension::descriptor(resolved, self.follow_redirects)
            })
            .await
            .map_err(|_| failure("saved source resolution timed out"))?;
        }
        self.resolve(source, selection, offset_ms)
            .await
            .map(SourceDescriptor::from)
    }

    /// Read selectable metadata through the source's saved account. Browsing
    /// another server never changes this lookup, and no playback lease is opened.
    pub async fn audio_options(&self, source: &SourceRef) -> Result<AudioOptions, ProviderError> {
        tokio::time::timeout(RESOLVE_TIMEOUT, async {
            if source.extension_id().is_some() {
                require_item(source)?;
                let runtime = self
                    .extension_dispatch()?
                    .accounts
                    .for_source(source)
                    .await
                    .map_err(extension_failure)?;
                let response = runtime
                    .audio_options(source)
                    .await
                    .map_err(extension_failure)?;
                return source_extension::audio_options(source, response);
            }
            self.authenticated(source)
                .await?
                .audio_options(source)
                .await
        })
        .await
        .map_err(|_| failure("saved source audio options lookup timed out"))?
    }

    /// Legacy sources have no audio-options export; do not resolve a stream to
    /// fabricate a candidate. Clients may still queue their browse metadata.
    pub async fn audio_options_handle(
        &self,
        source: &SourceHandle,
    ) -> Result<AudioOptions, ProviderError> {
        require_handle_item(source)?;
        match source {
            SourceHandle::Remote(source) => self.audio_options(source).await,
            SourceHandle::Local(_) => Err(failure("source v1 does not support audio selection")),
        }
    }

    /// Resolve an id-only queue entry through its stable handle, never a bare
    /// item ID interpreted against whichever provider is currently being browsed.
    pub async fn track(&self, source: &SourceRef) -> Result<Track, ProviderError> {
        tokio::time::timeout(RESOLVE_TIMEOUT, async {
            if source.extension_id().is_some() {
                require_item(source)?;
                let runtime = self
                    .extension_dispatch()?
                    .accounts
                    .for_source(source)
                    .await
                    .map_err(extension_failure)?;
                let response = runtime.item(source).await.map_err(extension_failure)?;
                return source_extension::media_track(source, response);
            }
            self.authenticated(source)
                .await?
                .track(&source.to_handle())
                .await
        })
        .await
        .map_err(|_| failure("saved source metadata lookup timed out"))?
    }

    /// V1 has no item lookup. A caller must carry the original browse Track
    /// through AddTracks instead of replacing its title with an ID or scanning.
    pub async fn track_handle(&self, source: &SourceHandle) -> Result<Track, ProviderError> {
        require_handle_item(source)?;
        match source {
            SourceHandle::Remote(source) => self.track(source).await,
            SourceHandle::Local(_) => Err(failure(
                "source v1 requires browse metadata when adding tracks",
            )),
        }
    }

    /// Validate queue ownership without authentication or any network request.
    /// Playback still checks a fresh snapshot when it actually opens the source.
    pub async fn validate_saved(&self, source: &SourceRef) -> Result<(), ProviderError> {
        if source.extension_id().is_some() {
            require_item(source)?;
            return tokio::time::timeout(
                RESOLVE_TIMEOUT,
                self.extension_dispatch()?.accounts.validate_saved(source),
            )
            .await
            .map_err(|_| failure("saved source account lookup timed out"))?
            .map_err(extension_failure);
        }
        tokio::time::timeout(RESOLVE_TIMEOUT, self.saved_snapshot(source))
            .await
            .map_err(|_| failure("saved source account lookup timed out"))??;
        Ok(())
    }

    /// Shared queue boundary, including the persisted scope of a v1 account.
    pub async fn validate_handle(&self, source: &SourceHandle) -> Result<(), ProviderError> {
        require_handle_item(source)?;
        match source {
            SourceHandle::Remote(source) => self.validate_saved(source).await,
            SourceHandle::Local(_) => tokio::time::timeout(
                RESOLVE_TIMEOUT,
                self.extension_dispatch()?.accounts.validate_handle(source),
            )
            .await
            .map_err(|_| failure("saved source account lookup timed out"))?
            .map_err(extension_failure),
        }
    }

    /// Browse an explicitly selected extension account. Root browsing needs no
    /// invented item ID; the returned guest IDs are bound to the verified account.
    pub async fn extension_media(
        &self,
        account_id: &str,
        parent: Option<&SourceRef>,
        query: Option<String>,
        offset: u64,
        limit: u32,
    ) -> Result<SourceMediaPage, ProviderError> {
        tokio::time::timeout(RESOLVE_TIMEOUT, async {
            if let Some(parent) = parent {
                if parent.account_id != account_id || parent.kind != ResourceKind::Container {
                    return Err(failure("invalid extension browse parent"));
                }
            }
            let accounts = &self.extension_dispatch()?.accounts;
            let runtime = match parent {
                Some(parent) => accounts.for_source(parent).await,
                None => accounts.for_account(account_id).await,
            }
            .map_err(extension_failure)?;
            let response = runtime
                .browse(parent, query, offset, limit)
                .await
                .map_err(extension_failure)?;
            let remote = runtime.remote_identity().map_err(extension_failure)?;
            let resolver = SourceRef::extension_resolver(runtime.extension_id())
                .map_err(|_| failure("invalid extension source identity"))?;
            let items = response
                .items
                .into_iter()
                .map(|item| {
                    let source = SourceRef {
                        resolver: resolver.clone(),
                        account_id: runtime.account_id().to_owned(),
                        remote: remote.clone(),
                        item_id: item.item_id.clone(),
                        kind: if item.is_container {
                            ResourceKind::Container
                        } else {
                            ResourceKind::Item
                        },
                    };
                    source_extension::media_entry(&source, item)
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(SourceMediaPage {
                items,
                next_offset: response.next_offset,
            })
        })
        .await
        .map_err(|_| failure("saved source browse timed out"))?
    }

    async fn saved_snapshot(&self, source: &SourceRef) -> Result<SavedServer, ProviderError> {
        // SourceRef has public fields: validate constructed values as well as
        // handles already parsed by an API. Reject before making any request.
        SourceRef::parse(&source.to_handle())
            .map_err(|_| failure("invalid saved source reference"))?;
        if source.resolver != "emby" {
            return Err(failure("source is not owned by the native Emby resolver"));
        }
        if source.kind != ResourceKind::Item {
            return Err(failure("saved source is not an item"));
        }
        let snapshot = saved_servers::get(self.db.get_connection(), &source.account_id)
            .await
            .map_err(|_| failure("saved source account lookup failed"))?
            .ok_or_else(|| failure("saved source account is missing"))?;
        if snapshot.kind != "emby"
            || snapshot.remote_server_id.as_deref() != Some(source.remote.server_id.as_str())
            || snapshot.remote_user_id.as_deref() != Some(source.remote.user_id.as_str())
        {
            return Err(failure(
                "saved source account identity is not bound or does not match",
            ));
        }
        Ok(snapshot)
    }

    async fn authenticated(&self, source: &SourceRef) -> Result<Arc<Emby>, ProviderError> {
        let snapshot = self.saved_snapshot(source).await?;
        let config = ProviderConfig {
            id: snapshot.id.clone(),
            kind: snapshot.kind.clone(),
            name: snapshot.name.clone(),
            url: snapshot.url.clone(),
            username: snapshot.username.clone(),
            password: snapshot.password.clone(),
        };
        let client = Emby::authenticate(&config, &self.device_id, self.follow_redirects).await?;
        if client.identity() != &source.remote {
            return Err(failure(
                "authenticated source identity does not match the saved source",
            ));
        }
        // Use the PRE-authentication snapshot, not a refreshed row. This checks
        // URL/credentials/deletion atomically before requesting item metadata or
        // a playback session. Storage deliberately permits name-only changes.
        // This playback path never establishes an initially absent binding.
        saved_servers::bind_remote_identity(self.db.get_connection(), &snapshot, client.identity())
            .await
            .map_err(|_| failure("saved source account changed during authentication"))?;
        Ok(Arc::new(client))
    }
}

fn failure(message: &'static str) -> ProviderError {
    ProviderError::Other(message.into())
}

fn require_item(source: &SourceRef) -> Result<(), ProviderError> {
    SourceRef::parse(&source.to_handle()).map_err(|_| failure("invalid saved source reference"))?;
    if source.kind != ResourceKind::Item {
        return Err(failure("saved source is not an item"));
    }
    Ok(())
}

fn require_handle_item(source: &SourceHandle) -> Result<(), ProviderError> {
    SourceHandle::parse(&source.to_handle())
        .map_err(|_| failure("invalid saved source reference"))?;
    if source.kind() != ResourceKind::Item {
        return Err(failure("saved source is not an item"));
    }
    Ok(())
}

fn extension_failure(error: crate::extension_accounts::AccountError) -> ProviderError {
    // AccountError Display deliberately excludes guest/driver/credential details.
    ProviderError::Other(error.to_string().into())
}

#[cfg(test)]
mod legacy_tests;
#[cfg(test)]
mod tests;

mod legacy;
pub use legacy::LegacySourceList;
