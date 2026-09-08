//! Loading and calling WebAssembly extensions.
//!
//! An extension is a directory containing a manifest and a `.wasm`. The host
//! reads the manifest, grants exactly what it declared, and calls only the
//! exports its capabilities cover.
//!
//! # Isolation
//!
//! Each extension is a separate Extism plugin with its own linear memory. It
//! gets no filesystem, no clock and no sockets; network access is limited to
//! the hosts in its manifest, and the only host functions it can call are the
//! three declared in `schema.yaml`. A panicking, looping or malformed
//! extension fails its own call — the host logs it and carries on — rather
//! than taking the player down.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{anyhow, Context, Error};
use extism::{Manifest as ExtismManifest, Plugin, PluginBuilder, UserData, Wasm, PTR};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::abi::{
    BrowseRequest, CommandRequest, CommandResponse, CommandSpec, Event, LibraryAlbum,
    LibraryArtist, LibraryPlaylist, MetadataRequest, MetadataResponse, PredicateRequest,
    PredicateResponse, PredicateSpec, SavedRadio, SourceAlbum, SourceArtist, SourceInfo,
    SourcePlaylist, StreamRequest, StreamResponse, TrackInfo,
};
use crate::manifest::{Capability, Manifest};

/// How long one call into an extension may take before it is cut off. Generous
/// enough for a metadata provider or a source to make an HTTP request, short
/// enough that a runaway loop does not wedge a scan.
const CALL_TIMEOUT: Duration = Duration::from_secs(10);

/// The user's library, as the `query_*` host functions need it.
///
/// A trait so this crate does not depend on the database directly, and so tests
/// can answer with fixed lists. Every method defaults to empty: a host that
/// only wants to expose tracks implements one method, not six.
pub trait LibraryAccess: Send + Sync {
    /// Tracks matching an RSQL filter, capped at `limit` (0 = no cap).
    fn query(&self, filter: &str, limit: u32) -> Result<Vec<TrackInfo>, Error>;

    /// Albums matching an RSQL filter over the album fields.
    fn albums(&self, _filter: &str, _limit: u32) -> Result<Vec<LibraryAlbum>, Error> {
        Ok(Vec::new())
    }

    /// Artists matching an RSQL filter over the artist fields.
    fn artists(&self, _filter: &str, _limit: u32) -> Result<Vec<LibraryArtist>, Error> {
        Ok(Vec::new())
    }

    /// Playlists matching an RSQL filter over the playlist fields.
    fn playlists(&self, _filter: &str, _limit: u32) -> Result<Vec<LibraryPlaylist>, Error> {
        Ok(Vec::new())
    }

    /// The tracks of one playlist, in order.
    fn playlist_tracks(&self, _playlist_id: &str) -> Result<Vec<TrackInfo>, Error> {
        Ok(Vec::new())
    }

    /// The user's bookmarked radio stations.
    fn saved_radios(&self) -> Result<Vec<SavedRadio>, Error> {
        Ok(Vec::new())
    }
}

/// What the host functions need in order to answer. Shared by every loaded
/// extension, hence the `Arc`.
#[derive(Clone, Default)]
pub struct HostContext {
    /// Answers `query_library`. `None` leaves extensions with no library
    /// access at all.
    pub library: Option<Arc<dyn LibraryAccess>>,
}

/// One loaded extension.
pub struct Extension {
    pub manifest: Manifest,
    pub dir: PathBuf,
    /// The Extism plugin. Behind a mutex because a plugin owns a single
    /// WebAssembly instance, so calls into it must be serialized.
    plugin: Mutex<Plugin>,
    /// The config the extension can see: its manifest defaults with the user's
    /// values layered on top.
    config: BTreeMap<String, String>,
}

impl Extension {
    /// Load the extension in `dir`, layering `overrides` over its declared
    /// config defaults.
    pub fn load(
        dir: &Path,
        overrides: &BTreeMap<String, String>,
        context: HostContext,
    ) -> Result<Self, Error> {
        let manifest = Manifest::load(dir)?;
        let wasm_path = manifest.entry_path(dir);

        // Only declared keys are visible: an extension cannot read settings it
        // never asked for, and the user can see the full list of what it wants
        // before enabling it.
        let mut config = manifest.permissions.config.clone();
        for (key, value) in overrides {
            if config.contains_key(key) {
                config.insert(key.clone(), value.clone());
            }
        }

        let mut extism_manifest =
            ExtismManifest::new([Wasm::file(&wasm_path)]).with_timeout(CALL_TIMEOUT);
        for host in &manifest.permissions.allowed_hosts {
            extism_manifest = extism_manifest.with_allowed_host(host);
        }

        let plugin = build_plugin(extism_manifest, &manifest, config.clone(), context)
            .with_context(|| format!("loading {}", wasm_path.display()))?;

        Ok(Self {
            manifest,
            dir: dir.to_path_buf(),
            plugin: Mutex::new(plugin),
            config,
        })
    }

    pub fn id(&self) -> &str {
        &self.manifest.id
    }

    /// The config this extension sees, after the user's overrides.
    pub fn config(&self) -> &BTreeMap<String, String> {
        &self.config
    }

    /// Call an export with a JSON body and decode its JSON reply.
    ///
    /// A missing export is `Ok(None)`: an extension with the `events`
    /// capability need not handle *every* event, so "did not implement this
    /// one" is a normal answer rather than a failure.
    pub fn call_json<I: Serialize, O: DeserializeOwned>(
        &self,
        function: &str,
        input: &I,
    ) -> Result<Option<O>, Error> {
        let mut plugin = self
            .plugin
            .lock()
            .map_err(|_| anyhow!("the '{}' extension panicked earlier", self.id()))?;
        if !plugin.function_exists(function) {
            return Ok(None);
        }
        let body = serde_json::to_vec(input)?;
        let raw = plugin
            .call::<&[u8], &[u8]>(function, &body)
            .map_err(|e| anyhow!("{}::{function} failed: {e}", self.manifest.id))?;
        if raw.is_empty() {
            return Ok(None);
        }
        serde_json::from_slice(raw).map(Some).map_err(|e| {
            anyhow!(
                "{}::{function} returned invalid JSON: {e}",
                self.manifest.id
            )
        })
    }

    /// Call an export that takes no input.
    fn call_empty<O: DeserializeOwned>(&self, function: &str) -> Result<Option<O>, Error> {
        self.call_json::<(), O>(function, &())
    }

    // ── Events ─────────────────────────────────────────────────────────────

    /// Deliver an event. No-op unless the extension declared `events` and
    /// exports the matching handler.
    pub fn notify(&self, event: &Event) -> Result<(), Error> {
        if !self.manifest.has(Capability::Events) {
            return Ok(());
        }
        self.call_json::<_, serde_json::Value>(event.handler(), event)?;
        Ok(())
    }

    // ── Metadata ───────────────────────────────────────────────────────────

    /// Ask for metadata about a track.
    pub fn metadata(&self, track: &TrackInfo) -> Result<Option<MetadataResponse>, Error> {
        if !self.manifest.has(Capability::Metadata) {
            return Ok(None);
        }
        let request = MetadataRequest {
            track: track.clone(),
        };
        self.call_json("get_metadata", &request)
    }

    // ── Commands ───────────────────────────────────────────────────────────

    /// The actions this extension offers.
    pub fn commands(&self) -> Result<Vec<CommandSpec>, Error> {
        if !self.manifest.has(Capability::Commands) {
            return Ok(Vec::new());
        }
        Ok(self.call_empty("commands")?.unwrap_or_default())
    }

    /// Run one of those actions.
    pub fn run_command(&self, request: &CommandRequest) -> Result<CommandResponse, Error> {
        if !self.manifest.has(Capability::Commands) {
            return Err(anyhow!("'{}' does not provide commands", self.manifest.id));
        }
        self.call_json("run_command", request)?
            .ok_or_else(|| anyhow!("'{}' has no run_command export", self.manifest.id))
    }

    // ── Smart-playlist predicates ──────────────────────────────────────────

    /// The predicates this extension adds to the filter vocabulary.
    pub fn predicates(&self) -> Result<Vec<PredicateSpec>, Error> {
        if !self.manifest.has(Capability::Predicates) {
            return Ok(Vec::new());
        }
        Ok(self.call_empty("predicates")?.unwrap_or_default())
    }

    /// Ask whether one track satisfies one predicate. A predicate that errors
    /// or is missing does not match, so a broken extension narrows a playlist
    /// rather than filling it with everything.
    pub fn evaluate(&self, request: &PredicateRequest) -> Result<bool, Error> {
        if !self.manifest.has(Capability::Predicates) {
            return Ok(false);
        }
        Ok(self
            .call_json::<_, PredicateResponse>("evaluate", request)?
            .map(|response| response.matches)
            .unwrap_or(false))
    }

    // ── Media source ───────────────────────────────────────────────────────

    /// Describe the source this extension provides.
    pub fn source_info(&self) -> Result<Option<SourceInfo>, Error> {
        if !self.manifest.has(Capability::Source) {
            return Ok(None);
        }
        self.call_empty("source_info")
    }

    pub fn list_albums(&self, request: &BrowseRequest) -> Result<Vec<SourceAlbum>, Error> {
        self.browse("list_albums", request)
    }

    pub fn list_artists(&self, request: &BrowseRequest) -> Result<Vec<SourceArtist>, Error> {
        self.browse("list_artists", request)
    }

    pub fn list_tracks(&self, request: &BrowseRequest) -> Result<Vec<TrackInfo>, Error> {
        self.browse("list_tracks", request)
    }

    pub fn list_playlists(&self, request: &BrowseRequest) -> Result<Vec<SourcePlaylist>, Error> {
        self.browse("list_playlists", request)
    }

    fn browse<O: DeserializeOwned>(
        &self,
        function: &str,
        request: &BrowseRequest,
    ) -> Result<Vec<O>, Error> {
        if !self.manifest.has(Capability::Source) {
            return Ok(Vec::new());
        }
        Ok(self
            .call_json::<_, Vec<O>>(function, request)?
            .unwrap_or_default())
    }

    /// Resolve one of the source's track ids to a playable url.
    pub fn stream_url(&self, track_id: &str) -> Result<Option<StreamResponse>, Error> {
        if !self.manifest.has(Capability::Source) {
            return Ok(None);
        }
        let request = StreamRequest {
            track_id: track_id.to_owned(),
        };
        self.call_json("get_stream_url", &request)
    }
}

/// Build the Extism plugin, wiring up the host functions from `schema.yaml`.
fn build_plugin(
    extism_manifest: ExtismManifest,
    manifest: &Manifest,
    config: BTreeMap<String, String>,
    context: HostContext,
) -> Result<Plugin, Error> {
    let log_id = manifest.id.clone();
    let library_read = manifest.permissions.library_read;
    let library = context.library.clone();

    let mut builder = PluginBuilder::new(extism_manifest)
        .with_wasi(true)
        .with_function(
            "log",
            [PTR],
            [],
            UserData::new(()),
            move |plugin, inputs, _outputs, _user_data| {
                let message: String = plugin.memory_get_val(&inputs[0])?;
                tracing::info!(extension = %log_id, "{message}");
                Ok(())
            },
        )
        .with_function(
            "get_config",
            [PTR],
            [PTR],
            UserData::new(()),
            move |plugin, inputs, outputs, _user_data| {
                let key: String = plugin.memory_get_val(&inputs[0])?;
                // An undeclared key reads as empty rather than erroring, so the
                // extension gets a defined answer either way.
                let value = config.get(&key).cloned().unwrap_or_default();
                let handle = plugin.memory_new(&value)?;
                outputs[0] = plugin.memory_to_val(handle);
                Ok(())
            },
        );

    // The library reads. Each is registered the same way: decode the request,
    // check the permission, hand back JSON. A denied or failing call answers
    // with an empty list rather than trapping — an extension should degrade,
    // not crash, when it is not allowed to look.
    macro_rules! library_query {
        ($name:literal, $method:ident) => {{
            let id = manifest.id.clone();
            let library = library.clone();
            builder = builder.with_function(
                $name,
                [PTR],
                [PTR],
                UserData::new(()),
                move |plugin, inputs, outputs, _user_data| {
                    let raw: String = plugin.memory_get_val(&inputs[0])?;
                    let body = if let Some(library) = guard(&id, library_read, library.as_deref()) {
                        let query: LibraryQuery = serde_json::from_str(&raw).unwrap_or_default();
                        match library.$method(&query.filter, query.limit) {
                            Ok(rows) => serde_json::to_string(&rows)?,
                            Err(e) => {
                                tracing::warn!(extension = %id, "{} failed: {e}", $name);
                                "[]".to_string()
                            }
                        }
                    } else {
                        "[]".to_string()
                    };
                    let handle = plugin.memory_new(&body)?;
                    outputs[0] = plugin.memory_to_val(handle);
                    Ok(())
                },
            );
        }};
    }

    library_query!("query_library", query);
    library_query!("query_albums", albums);
    library_query!("query_artists", artists);
    library_query!("query_playlists", playlists);

    {
        let id = manifest.id.clone();
        let library = library.clone();
        builder = builder.with_function(
            "get_playlist_tracks",
            [PTR],
            [PTR],
            UserData::new(()),
            move |plugin, inputs, outputs, _user_data| {
                let playlist_id: String = plugin.memory_get_val(&inputs[0])?;
                let body = match guard(&id, library_read, library.as_deref()) {
                    Some(library) => match library.playlist_tracks(&playlist_id) {
                        Ok(tracks) => serde_json::to_string(&tracks)?,
                        Err(e) => {
                            tracing::warn!(extension = %id, "get_playlist_tracks failed: {e}");
                            "[]".to_string()
                        }
                    },
                    None => "[]".to_string(),
                };
                let handle = plugin.memory_new(&body)?;
                outputs[0] = plugin.memory_to_val(handle);
                Ok(())
            },
        );
    }

    {
        let id = manifest.id.clone();
        let library = library.clone();
        builder = builder.with_function(
            "get_saved_radios",
            [],
            [PTR],
            UserData::new(()),
            move |plugin, _inputs, outputs, _user_data| {
                let body = match guard(&id, library_read, library.as_deref()) {
                    Some(library) => match library.saved_radios() {
                        Ok(radios) => serde_json::to_string(&radios)?,
                        Err(e) => {
                            tracing::warn!(extension = %id, "get_saved_radios failed: {e}");
                            "[]".to_string()
                        }
                    },
                    None => "[]".to_string(),
                };
                let handle = plugin.memory_new(&body)?;
                outputs[0] = plugin.memory_to_val(handle);
                Ok(())
            },
        );
    }

    builder.build()
}

#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct LibraryQuery {
    #[serde(default)]
    filter: String,
    #[serde(default)]
    limit: u32,
}

/// The library an extension may read, or `None` when it may not.
///
/// The permission is enforced here, host-side — never trusted from the guest,
/// which could simply lie about what its manifest said.
fn guard<'a>(
    id: &str,
    permitted: bool,
    library: Option<&'a dyn LibraryAccess>,
) -> Option<&'a dyn LibraryAccess> {
    if !permitted {
        tracing::warn!(
            extension = %id,
            "library read denied: the manifest does not request the libraryRead permission"
        );
        return None;
    }
    library
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedLibrary {
        tracks: Vec<TrackInfo>,
    }

    impl LibraryAccess for FixedLibrary {
        fn query(&self, _filter: &str, limit: u32) -> Result<Vec<TrackInfo>, Error> {
            let mut tracks = self.tracks.clone();
            if limit > 0 {
                tracks.truncate(limit as usize);
            }
            Ok(tracks)
        }
    }

    fn track(title: &str) -> TrackInfo {
        TrackInfo {
            id: title.to_lowercase(),
            title: title.into(),
            artist: "Someone".into(),
            ..Default::default()
        }
    }

    /// The permission is enforced host-side, not trusted from the guest: an
    /// extension that never asked for library access is handed nothing.
    #[test]
    fn the_guard_refuses_without_the_permission() {
        let library = FixedLibrary {
            tracks: vec![track("Airbag")],
        };
        assert!(guard("x", false, Some(&library)).is_none());
        assert!(guard("x", true, Some(&library)).is_some());
    }

    #[test]
    fn the_guard_passes_through_a_missing_library() {
        assert!(guard("x", true, None).is_none());
    }

    #[test]
    fn a_query_honours_the_limit() {
        let library = FixedLibrary {
            tracks: vec![track("A"), track("B"), track("C")],
        };
        assert_eq!(library.query("", 2).unwrap().len(), 2);
        assert_eq!(library.query("", 0).unwrap().len(), 3);
    }

    /// A malformed request body reads as an empty query rather than failing
    /// the extension's call.
    #[test]
    fn a_bad_request_body_reads_as_an_empty_query() {
        let query: LibraryQuery = serde_json::from_str("not json").unwrap_or_default();
        assert!(query.filter.is_empty());
        assert_eq!(query.limit, 0);
    }

    /// A host that only exposes tracks implements one method; the rest default
    /// to empty rather than forcing every host to write five stubs.
    #[test]
    fn unimplemented_library_reads_default_to_empty() {
        let library = FixedLibrary { tracks: vec![] };
        assert!(library.albums("", 0).unwrap().is_empty());
        assert!(library.artists("", 0).unwrap().is_empty());
        assert!(library.playlists("", 0).unwrap().is_empty());
        assert!(library.playlist_tracks("pl-1").unwrap().is_empty());
        assert!(library.saved_radios().unwrap().is_empty());
    }
}
