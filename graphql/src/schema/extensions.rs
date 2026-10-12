//! Listing the installed WebAssembly extensions, and switching them on and off.
//!
//! Deliberately narrow: a UI may list an extension and toggle it, and that is
//! all. Installing and scaffolding change what code the daemon runs from a
//! source the user has to have chosen deliberately, so those stay with the CLI
//! (`music-player extension install`) rather than being reachable from a
//! browser tab.

use async_graphql::*;
use chrono::Utc;
use music_player_extensions::{catalog, search_paths};
use music_player_settings::{get_application_directory, read_settings, Settings};
use music_player_storage::{extension_state, Database};
use std::path::PathBuf;

use super::objects::extension::Extension;

#[cfg(test)]
mod tests;

#[derive(Default)]
pub struct ExtensionsQuery;

#[derive(Default)]
pub struct ExtensionsMutation;

/// The configured extension paths, or none when settings cannot be read —
/// which only means falling back to the default location.
fn extension_paths() -> Vec<String> {
    read_settings()
        .ok()
        .and_then(|config| config.try_deserialize::<Settings>().ok())
        .map(|settings| settings.extension_paths)
        .unwrap_or_default()
}

/// Scan every search path, apply the stored enabled flags, and filter.
async fn scan(db: &Database, paths: &[PathBuf], filter: Option<String>) -> Vec<Extension> {
    let enabled = extension_state::enabled_map(db.get_connection()).await;
    let installed = catalog::installed(paths, &enabled);

    let needle = filter
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty());

    installed
        .into_iter()
        .map(Extension::from)
        .filter(|extension| match &needle {
            None => true,
            Some(needle) => matches(extension, needle),
        })
        .collect()
}

/// The ids currently on disk, for checking an id before storing a flag for it.
fn installed_ids(paths: &[PathBuf]) -> Vec<String> {
    catalog::installed(paths, &Default::default())
        .into_iter()
        .map(|installed| installed.manifest.id)
        .collect()
}

fn catalog_paths(_ctx: &Context<'_>) -> Vec<PathBuf> {
    // Test-only directory configuration: still scan real manifests, without
    // changing process environment or reading/writing the user's settings.
    #[cfg(test)]
    if let Some(paths) = _ctx.data_opt::<tests::CatalogPaths>() {
        return paths.0.clone();
    }
    search_paths(&get_application_directory(), &extension_paths())
}

#[Object]
impl ExtensionsQuery {
    /// Every installed extension, in id order. `filter` is a case-insensitive
    /// search over the id, name, description, author, topics and capabilities.
    ///
    /// This reads the manifests rather than loading the modules, so opening an
    /// extensions page costs a directory walk instead of one WebAssembly
    /// instantiation per extension.
    async fn extensions(
        &self,
        ctx: &Context<'_>,
        filter: Option<String>,
    ) -> Result<Vec<Extension>, Error> {
        let db = ctx.data::<Database>()?;
        Ok(scan(db, &catalog_paths(ctx), filter).await)
    }
}

#[Object]
impl ExtensionsMutation {
    /// Switch an extension on or off, and return it as it now stands.
    ///
    /// Source account runtimes stop accepting new calls when disabled. Stale
    /// results are suppressed, while existing leases retain their original
    /// instance for cleanup. Non-source modules keep their existing flag and
    /// startup-loading behavior; this does not unload them mid-call.
    async fn set_extension_enabled(
        &self,
        ctx: &Context<'_>,
        id: ID,
        enabled: bool,
    ) -> Result<Extension, Error> {
        let db = ctx.data::<Database>()?;
        let manager = crate::source_resolver(ctx)?.extension_accounts()?;
        let id = id.to_string();
        let paths = catalog_paths(ctx);

        // Refuse an id that is not installed rather than storing a row that
        // will never match anything.
        if !installed_ids(&paths).contains(&id) {
            return Err(Error::new(format!("no extension '{id}' is installed")));
        }

        manager
            .set_enabled(&id, enabled, &Utc::now().to_rfc3339())
            .await
            .map_err(|e| Error::new(e.to_string()))?;

        scan(db, &paths, None)
            .await
            .into_iter()
            .find(|extension| extension.id.as_str() == id)
            .ok_or_else(|| Error::new(format!("no extension '{id}' is installed")))
    }

    /// Re-read the extension directories and return what is installed now.
    ///
    /// Also drops the stored flags for extensions that are no longer there: a
    /// leftover row is harmless while it matches nothing, but it would switch
    /// an extension straight back off if it were ever reinstalled, which is
    /// not what deleting it meant.
    async fn rescan_extensions(
        &self,
        ctx: &Context<'_>,
        filter: Option<String>,
    ) -> Result<Vec<Extension>, Error> {
        let db = ctx.data::<Database>()?;
        // Generation invalidation and stale preference pruning share the
        // manager's mutation gate. Never prune independently here.
        crate::source_resolver(ctx)?
            .extension_accounts()?
            .rescan()
            .await
            .map_err(|e| Error::new(e.to_string()))?;
        Ok(scan(db, &catalog_paths(ctx), filter).await)
    }
}

/// Whether an extension matches a search term.
///
/// Deliberately generous — id, name, description, author and topics — because
/// people search for what an extension *does* ("lyrics") at least as often as
/// for what it is called.
fn matches(extension: &Extension, needle: &str) -> bool {
    let haystacks = [
        extension.id.as_str(),
        extension.name.as_str(),
        extension.description.as_str(),
        extension.author.as_str(),
    ];
    haystacks
        .iter()
        .any(|value| value.to_lowercase().contains(needle))
        || extension
            .topics
            .iter()
            .any(|topic| topic.to_lowercase().contains(needle))
        || extension
            .capabilities
            .iter()
            .any(|capability| capability.contains(needle))
}
