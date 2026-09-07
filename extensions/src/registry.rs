//! Discovering, enabling and calling the installed extensions.
//!
//! Extensions live one directory each under `<app dir>/extensions`. The
//! registry scans that on startup, loads the enabled ones, and fans calls out
//! to whichever declared the matching capability.
//!
//! Fan-out is deliberately forgiving: one extension failing a call is logged
//! and skipped, never propagated. A broken third-party plugin must not stop a
//! scan, a play or a playlist from working.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Error;

use crate::abi::{
    CommandRequest, CommandResponse, CommandSpec, Event, MetadataResponse, PredicateRequest,
    PredicateSpec, SourceInfo, TrackInfo,
};
use crate::host::{Extension, HostContext};
use crate::manifest::{Capability, Manifest, MANIFEST_FILES};

/// Where extensions are installed, under the application directory.
pub const EXTENSIONS_DIR: &str = "extensions";

/// The prefix that marks a smart-playlist field as coming from an extension,
/// e.g. `ext:mood==chill`.
pub const PREDICATE_PREFIX: &str = "ext:";

/// An installed extension, loaded or not.
pub struct Installed {
    pub manifest: Manifest,
    pub dir: PathBuf,
    /// `None` when the extension is disabled, or when loading it failed.
    pub extension: Option<Arc<Extension>>,
    /// Why it is not loaded, when that was a failure rather than a choice.
    pub error: Option<String>,
    pub enabled: bool,
}

impl Installed {
    pub fn id(&self) -> &str {
        &self.manifest.id
    }
}

/// Every installed extension, and the fan-out over them.
#[derive(Default)]
pub struct Registry {
    extensions: Vec<Installed>,
}

impl Registry {
    /// Scan `dir` for extensions and load the enabled ones.
    ///
    /// `enabled` decides which to load — typically read from the `addon` table.
    /// An extension missing from the map is loaded: an extension the user just
    /// dropped in should work without a second step.
    ///
    /// `config` holds per-extension config overrides, keyed by extension id.
    pub fn load(
        dir: &Path,
        enabled: &BTreeMap<String, bool>,
        config: &BTreeMap<String, BTreeMap<String, String>>,
        context: HostContext,
    ) -> Self {
        Self::load_all(&[dir.to_path_buf()], enabled, config, context)
    }

    /// Scan several directories, in priority order.
    ///
    /// An extension id found in more than one is loaded from the first, so a
    /// user's own copy shadows a system-wide one rather than colliding with it.
    pub fn load_all(
        dirs: &[PathBuf],
        enabled: &BTreeMap<String, bool>,
        config: &BTreeMap<String, BTreeMap<String, String>>,
        context: HostContext,
    ) -> Self {
        tracing::info!(
            paths = ?dirs.iter().map(|d| d.display().to_string()).collect::<Vec<_>>(),
            "searching for extensions"
        );
        let mut extensions: Vec<Installed> = Vec::new();
        for dir in dirs {
            let found = Self::scan(dir, enabled, config, &context);
            for installed in found {
                if let Some(existing) = extensions.iter().find(|e| e.id() == installed.id()) {
                    tracing::info!(
                        extension = %installed.id(),
                        shadowed = %installed.dir.display(),
                        loaded = %existing.dir.display(),
                        "ignoring a duplicate extension: an earlier search path already provides it"
                    );
                    continue;
                }
                extensions.push(installed);
            }
        }
        extensions.sort_by(|a, b| a.manifest.id.cmp(&b.manifest.id));

        let loaded: Vec<&str> = extensions
            .iter()
            .filter(|e| e.extension.is_some())
            .map(|e| e.id())
            .collect();
        if loaded.is_empty() {
            tracing::info!(installed = extensions.len(), "no extensions loaded");
        } else {
            tracing::info!(
                count = loaded.len(),
                extensions = ?loaded,
                "extensions ready"
            );
        }
        Self { extensions }
    }

    /// Read one directory. Anything unreadable or malformed is skipped with a
    /// log line rather than failing the whole scan.
    fn scan(
        dir: &Path,
        enabled: &BTreeMap<String, bool>,
        config: &BTreeMap<String, BTreeMap<String, String>>,
        context: &HostContext,
    ) -> Vec<Installed> {
        let mut extensions = Vec::new();
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            // A configured path that does not exist yet is normal, not an error.
            Err(e) => {
                tracing::debug!(path = %dir.display(), "no extensions here: {e}");
                return extensions;
            }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() || !MANIFEST_FILES.iter().any(|name| path.join(name).exists()) {
                continue;
            }
            let manifest = match Manifest::load(&path) {
                Ok(manifest) => manifest,
                Err(e) => {
                    tracing::warn!(dir = %path.display(), "ignoring an extension: {e}");
                    continue;
                }
            };
            let is_enabled = enabled.get(&manifest.id).copied().unwrap_or(true);
            let empty = BTreeMap::new();
            let overrides = config.get(&manifest.id).unwrap_or(&empty);

            let (extension, error) = if is_enabled {
                match Extension::load(&path, overrides, context.clone()) {
                    Ok(extension) => {
                        tracing::info!(
                            extension = %manifest.id,
                            version = %manifest.version,
                            capabilities = %manifest
                                .capabilities
                                .iter()
                                .map(|c| c.as_str())
                                .collect::<Vec<_>>()
                                .join(", "),
                            path = %path.display(),
                            "loaded extension"
                        );
                        (Some(Arc::new(extension)), None)
                    }
                    Err(e) => {
                        tracing::warn!(
                            extension = %manifest.id,
                            path = %path.display(),
                            "could not load: {e}"
                        );
                        (None, Some(e.to_string()))
                    }
                }
            } else {
                tracing::info!(extension = %manifest.id, "extension disabled, not loading");
                (None, None)
            };
            extensions.push(Installed {
                manifest,
                dir: path,
                extension,
                error,
                enabled: is_enabled,
            });
        }
        extensions
    }

    pub fn installed(&self) -> &[Installed] {
        &self.extensions
    }

    pub fn get(&self, id: &str) -> Option<&Installed> {
        self.extensions.iter().find(|e| e.manifest.id == id)
    }

    pub fn is_empty(&self) -> bool {
        self.extensions.is_empty()
    }

    /// The loaded extensions declaring `capability`.
    fn with(&self, capability: Capability) -> impl Iterator<Item = &Arc<Extension>> {
        self.extensions
            .iter()
            .filter(move |installed| installed.manifest.has(capability))
            .filter_map(|installed| installed.extension.as_ref())
    }

    // ── Events ─────────────────────────────────────────────────────────────

    /// Deliver `event` to every extension that handles it. Failures are logged
    /// and skipped — an event is a notification, not a transaction.
    pub fn notify(&self, event: &Event) {
        for extension in self.with(Capability::Events) {
            if let Err(e) = extension.notify(event) {
                tracing::warn!(extension = %extension.id(), "event handler failed: {e}");
            }
        }
    }

    // ── Metadata ───────────────────────────────────────────────────────────

    /// Ask each metadata provider in turn and merge what they return, first
    /// non-empty answer per field winning. Ordering is by extension id, so the
    /// result does not depend on directory iteration order.
    pub fn metadata(&self, track: &TrackInfo) -> MetadataResponse {
        let mut merged = MetadataResponse::default();
        for extension in self.with(Capability::Metadata) {
            let answer = match extension.metadata(track) {
                Ok(Some(answer)) => answer,
                Ok(None) => continue,
                Err(e) => {
                    tracing::warn!(extension = %extension.id(), "metadata lookup failed: {e}");
                    continue;
                }
            };
            merged.lyrics = merged.lyrics.or(answer.lyrics);
            merged.artwork_url = merged.artwork_url.or(answer.artwork_url);
            merged.bio = merged.bio.or(answer.bio);
            if merged.genres.is_empty() {
                merged.genres = answer.genres;
            }
        }
        merged
    }

    // ── Commands ───────────────────────────────────────────────────────────

    /// Every command on offer, paired with the extension id that provides it.
    pub fn commands(&self) -> Vec<(String, CommandSpec)> {
        let mut all = Vec::new();
        for extension in self.with(Capability::Commands) {
            match extension.commands() {
                Ok(specs) => all.extend(
                    specs
                        .into_iter()
                        .map(|spec| (extension.id().to_owned(), spec)),
                ),
                Err(e) => {
                    tracing::warn!(extension = %extension.id(), "could not list commands: {e}")
                }
            }
        }
        all
    }

    /// Run a command on a named extension.
    pub fn run_command(
        &self,
        extension_id: &str,
        request: &CommandRequest,
    ) -> Result<CommandResponse, Error> {
        let installed = self
            .get(extension_id)
            .ok_or_else(|| anyhow::anyhow!("no extension '{extension_id}'"))?;
        let extension = installed
            .extension
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("'{extension_id}' is not loaded"))?;
        extension.run_command(request)
    }

    // ── Smart-playlist predicates ──────────────────────────────────────────

    /// Every predicate on offer, as `(extension id, spec)`.
    pub fn predicates(&self) -> Vec<(String, PredicateSpec)> {
        let mut all = Vec::new();
        for extension in self.with(Capability::Predicates) {
            match extension.predicates() {
                Ok(specs) => all.extend(
                    specs
                        .into_iter()
                        .map(|spec| (extension.id().to_owned(), spec)),
                ),
                Err(e) => {
                    tracing::warn!(extension = %extension.id(), "could not list predicates: {e}")
                }
            }
        }
        all
    }

    /// Whether `track` satisfies `ext:<name> <op> <value>`.
    ///
    /// The first extension that offers `name` answers. An unknown predicate or
    /// a failing one does not match — a smart playlist that quietly loses a
    /// filter and grows to the whole library is worse than one that comes back
    /// short.
    pub fn evaluate(&self, name: &str, op: &str, value: &str, track: &TrackInfo) -> bool {
        let name = name.strip_prefix(PREDICATE_PREFIX).unwrap_or(name);
        for extension in self.with(Capability::Predicates) {
            let offers = extension
                .predicates()
                .map(|specs| specs.iter().any(|spec| spec.name == name))
                .unwrap_or(false);
            if !offers {
                continue;
            }
            let request = PredicateRequest {
                name: name.to_owned(),
                op: op.to_owned(),
                value: value.to_owned(),
                track: track.clone(),
            };
            match extension.evaluate(&request) {
                Ok(matches) => return matches,
                Err(e) => {
                    tracing::warn!(extension = %extension.id(), "predicate '{name}' failed: {e}");
                    return false;
                }
            }
        }
        tracing::debug!("no extension provides the predicate '{name}'");
        false
    }

    // ── Media sources ──────────────────────────────────────────────────────

    /// Every media source on offer, as `(extension id, info)`.
    pub fn sources(&self) -> Vec<(String, SourceInfo)> {
        let mut all = Vec::new();
        for extension in self.with(Capability::Source) {
            match extension.source_info() {
                Ok(Some(info)) => all.push((extension.id().to_owned(), info)),
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!(extension = %extension.id(), "could not read source info: {e}")
                }
            }
        }
        all
    }

    /// The extension providing a named source, if it is loaded.
    pub fn source(&self, extension_id: &str) -> Option<&Arc<Extension>> {
        self.get(extension_id)
            .filter(|installed| installed.manifest.has(Capability::Source))
            .and_then(|installed| installed.extension.as_ref())
    }
}

/// The extensions directory under `app_dir`.
pub fn extensions_dir(app_dir: &str) -> PathBuf {
    Path::new(app_dir).join(EXTENSIONS_DIR)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An empty but structurally valid WebAssembly module: the magic bytes
    /// and version, no sections. Enough to load, with no exports — which is
    /// exactly a well-formed extension that implements nothing.
    const EMPTY_WASM: &[u8] = b"\0asm\x01\0\0\0";

    fn install(root: &Path, id: &str, capabilities: &str) -> PathBuf {
        install_wasm(root, id, capabilities, EMPTY_WASM)
    }

    fn install_wasm(root: &Path, id: &str, capabilities: &str, wasm: &[u8]) -> PathBuf {
        let dir = root.join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("plugin.json"),
            format!(r#"{{"id":"{id}","name":"{id}","capabilities":{capabilities}}}"#),
        )
        .unwrap();
        std::fs::write(dir.join("plugin.wasm"), wasm).unwrap();
        dir
    }

    #[test]
    fn a_missing_directory_is_simply_no_extensions() {
        let registry = Registry::load(
            Path::new("/nonexistent/extensions"),
            &BTreeMap::new(),
            &BTreeMap::new(),
            HostContext::default(),
        );
        assert!(registry.is_empty());
    }

    /// A directory that is not an extension, and one whose manifest is broken,
    /// are both skipped rather than failing the whole scan.
    #[test]
    fn skips_what_is_not_an_extension() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("not-an-extension")).unwrap();
        std::fs::create_dir_all(root.path().join("broken")).unwrap();
        std::fs::write(root.path().join("broken/plugin.json"), "{ oops").unwrap();
        install(root.path(), "com.example.good", r#"["events"]"#);

        let registry = Registry::load(
            root.path(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            HostContext::default(),
        );
        assert_eq!(registry.installed().len(), 1);
        assert_eq!(registry.installed()[0].id(), "com.example.good");
    }

    /// A disabled extension is still listed — the settings UI has to show it
    /// to let the user turn it back on — but is not loaded.
    #[test]
    fn lists_disabled_extensions_without_loading_them() {
        let root = tempfile::tempdir().unwrap();
        install(root.path(), "com.example.off", r#"["events"]"#);
        let enabled = BTreeMap::from([("com.example.off".to_string(), false)]);

        let registry = Registry::load(
            root.path(),
            &enabled,
            &BTreeMap::new(),
            HostContext::default(),
        );
        let installed = &registry.installed()[0];
        assert!(!installed.enabled);
        assert!(installed.extension.is_none());
        // Not loading it was a choice, not a failure.
        assert!(installed.error.is_none());
    }

    /// An extension whose wasm will not load is recorded with its error rather
    /// than vanishing, so the user can see why it is not working.
    #[test]
    fn records_why_an_extension_failed_to_load() {
        let root = tempfile::tempdir().unwrap();
        install_wasm(
            root.path(),
            "com.example.broken",
            r#"["events"]"#,
            b"this is not webassembly",
        );

        let registry = Registry::load(
            root.path(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            HostContext::default(),
        );
        let installed = &registry.installed()[0];
        assert!(installed.enabled);
        assert!(installed.extension.is_none());
        assert!(installed.error.is_some());
    }

    /// A valid module that exports nothing is not an error: an extension may
    /// legitimately handle only some of its capability's functions, and a
    /// missing export is "not implemented", not "broken".
    #[test]
    fn an_extension_with_no_exports_still_loads() {
        let root = tempfile::tempdir().unwrap();
        install(root.path(), "com.example.empty", r#"["events"]"#);

        let registry = Registry::load(
            root.path(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            HostContext::default(),
        );
        let installed = &registry.installed()[0];
        assert!(installed.extension.is_some(), "{:?}", installed.error);
        // Delivering an event it does not handle is a no-op, not a failure.
        registry.notify(&Event::ScanCompleted {
            tracks_added: 0,
            total_tracks: 0,
        });
    }

    /// Nothing is loaded, so nothing can answer — but the fan-out must be a
    /// no-op rather than a panic.
    #[test]
    fn fan_out_over_nothing_is_harmless() {
        let registry = Registry::default();
        registry.notify(&Event::ScanCompleted {
            tracks_added: 1,
            total_tracks: 1,
        });
        assert!(registry.metadata(&TrackInfo::default()).is_empty());
        assert!(registry.commands().is_empty());
        assert!(registry.predicates().is_empty());
        assert!(registry.sources().is_empty());
        // An unknown predicate does not match, rather than matching everything.
        assert!(!registry.evaluate("mood", "==", "chill", &TrackInfo::default()));
    }

    #[test]
    fn predicate_names_may_carry_the_prefix() {
        let registry = Registry::default();
        // Both spellings resolve to the same lookup; neither matches here.
        assert!(!registry.evaluate("ext:mood", "==", "chill", &TrackInfo::default()));
        assert!(!registry.evaluate("mood", "==", "chill", &TrackInfo::default()));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::abi::BrowseRequest;

    /// Every shipped example, with the capability its manifest declares.
    const EXAMPLES: &[(&str, Capability)] = &[
        ("scrobble-logger", Capability::Events),
        ("lyrics-provider", Capability::Metadata),
        ("library-commands", Capability::Commands),
        ("mood-predicate", Capability::Predicates),
        ("radio-source", Capability::Source),
    ];

    fn examples_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("examples")
    }

    /// Install the built examples into a temp directory. Returns `None` when
    /// they have not been built — a checkout without a wasm toolchain must
    /// still be able to run the test suite.
    fn install_examples() -> Option<tempfile::TempDir> {
        let root = tempfile::tempdir().ok()?;
        let mut installed = 0;
        for (name, _) in EXAMPLES {
            let source = examples_dir().join(name);
            if !source.join("plugin.wasm").exists() {
                continue;
            }
            let target = root.path().join(name);
            std::fs::create_dir_all(&target).ok()?;
            for file in ["plugin.toml", "plugin.wasm"] {
                std::fs::copy(source.join(file), target.join(file)).ok()?;
            }
            installed += 1;
        }
        (installed == EXAMPLES.len()).then_some(root)
    }

    /// Loads every shipped example and calls into each one, so the host
    /// functions, the manifest gating and the JSON ABI are exercised together
    /// against real WebAssembly rather than a stub.
    #[test]
    fn loads_and_calls_every_example() {
        let Some(root) = install_examples() else {
            eprintln!("skipping: build the examples first (see extensions/README.md)");
            return;
        };
        let registry = Registry::load_all(
            &[root.path().to_path_buf()],
            &BTreeMap::new(),
            &BTreeMap::new(),
            HostContext::default(),
        );
        assert_eq!(registry.installed().len(), EXAMPLES.len());
        for installed in registry.installed() {
            assert!(
                installed.extension.is_some(),
                "{} failed to load: {:?}",
                installed.id(),
                installed.error
            );
        }

        // Each example answers for its own capability and no other. This is
        // the manifest gate doing its job: every module exports every function
        // in the schema, so without it they would all answer everything.
        let commands = registry.commands();
        assert_eq!(commands.len(), 3, "only library-commands provides commands");
        assert!(commands
            .iter()
            .all(|(id, _)| id.contains("library-commands")));

        let predicates = registry.predicates();
        assert_eq!(
            predicates.len(),
            2,
            "only mood-predicate provides predicates"
        );
        assert!(predicates.iter().any(|(_, spec)| spec.name == "mood"));

        let sources = registry.sources();
        assert_eq!(sources.len(), 1, "only radio-source provides a source");
        assert_eq!(sources[0].1.name, "Radio Browser");
    }

    /// The predicate example, end to end: a real filter term evaluated against
    /// a real track, inside the sandbox.
    #[test]
    fn evaluates_a_predicate_from_an_example() {
        let Some(root) = install_examples() else {
            eprintln!("skipping: build the examples first");
            return;
        };
        let registry = Registry::load_all(
            &[root.path().to_path_buf()],
            &BTreeMap::new(),
            &BTreeMap::new(),
            HostContext::default(),
        );
        let ambient = TrackInfo {
            id: "t1".into(),
            title: "Weightless".into(),
            artist: "Marconi Union".into(),
            genre: "ambient".into(),
            year: Some(2011),
            ..Default::default()
        };
        assert!(registry.evaluate("ext:mood", "==", "calm", &ambient));
        assert!(!registry.evaluate("mood", "==", "energetic", &ambient));
        // `!=` is the complement, not a no-op.
        assert!(registry.evaluate("mood", "!=", "energetic", &ambient));
        assert!(registry.evaluate("era", "==", "twentytens", &ambient));
    }

    /// The source example answers a browse that needs no network.
    #[test]
    fn browses_the_source_example() {
        let Some(root) = install_examples() else {
            eprintln!("skipping: build the examples first");
            return;
        };
        let registry = Registry::load_all(
            &[root.path().to_path_buf()],
            &BTreeMap::new(),
            &BTreeMap::new(),
            HostContext::default(),
        );
        let source = registry.source("fm.atradio.radio-browser").unwrap();
        let albums = source.list_albums(&BrowseRequest::default()).unwrap();
        assert!(albums.iter().any(|album| album.id == "jazz"));
    }

    /// A metadata provider that cannot reach the network still answers — an
    /// empty response, not an error, so other providers get their turn.
    #[test]
    fn a_metadata_provider_degrades_to_an_empty_answer() {
        let Some(root) = install_examples() else {
            eprintln!("skipping: build the examples first");
            return;
        };
        let registry = Registry::load_all(
            &[root.path().to_path_buf()],
            &BTreeMap::new(),
            &BTreeMap::new(),
            HostContext::default(),
        );
        // No artist or title: the provider returns early without a request.
        let merged = registry.metadata(&TrackInfo::default());
        assert!(merged.is_empty());
    }
}
