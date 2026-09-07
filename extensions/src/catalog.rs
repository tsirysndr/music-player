//! Listing what is installed, without running any of it.
//!
//! [`Registry::load_all`](crate::registry::Registry::load_all) instantiates
//! every enabled module so it can be called; that is the right thing for the
//! daemon at startup and the wrong thing for a UI that only wants to draw a
//! list. Both clients' extension views ask this instead: it reads the
//! manifests and stops there, so opening the page costs a directory walk
//! rather than a WebAssembly instantiation per extension.
//!
//! The one thing it cannot know is whether a module *would* load — that needs
//! the module — so [`Installed::status`] reports `Enabled` where the registry
//! would report `Loaded` or `Failed`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::manifest::{Manifest, MANIFEST_FILES};

/// Whether an extension is switched on, as far as a manifest scan can tell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Switched on. Whether its module loads is only known once it is loaded.
    Enabled,
    /// Turned off by the user, so it is listed but never called.
    Disabled,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Enabled => "enabled",
            Status::Disabled => "disabled",
        }
    }
}

/// One installed extension, as read from its manifest.
#[derive(Clone, Debug)]
pub struct Installed {
    pub manifest: Manifest,
    /// The directory the manifest was read from.
    pub dir: PathBuf,
    pub status: Status,
}

impl Installed {
    pub fn id(&self) -> &str {
        &self.manifest.id
    }

    pub fn enabled(&self) -> bool {
        self.status == Status::Enabled
    }
}

/// Every extension installed under `dirs`, in id order.
///
/// Directories are searched in the order given and the first copy of an id
/// wins, matching how the registry resolves a shadowed extension. Anything
/// unreadable or malformed is skipped with a log line rather than failing the
/// whole scan — one broken third-party manifest must not empty the list.
pub fn installed(dirs: &[PathBuf], enabled: &BTreeMap<String, bool>) -> Vec<Installed> {
    let mut found: Vec<Installed> = Vec::new();
    for dir in dirs {
        for candidate in scan(dir, enabled) {
            if found.iter().any(|existing| existing.id() == candidate.id()) {
                tracing::debug!(
                    extension = %candidate.id(),
                    shadowed = %candidate.dir.display(),
                    "skipping a duplicate extension: an earlier search path provides it"
                );
                continue;
            }
            found.push(candidate);
        }
    }
    found.sort_by(|a, b| a.manifest.id.cmp(&b.manifest.id));
    found
}

fn scan(dir: &Path, enabled: &BTreeMap<String, bool>) -> Vec<Installed> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        // A configured path that does not exist yet is normal, not an error.
        Err(e) => {
            tracing::debug!(path = %dir.display(), "no extensions here: {e}");
            return Vec::new();
        }
    };

    let mut found = Vec::new();
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
        // An extension the user just dropped in is on until they say otherwise,
        // which is the same default the registry applies.
        let status = match enabled.get(&manifest.id).copied().unwrap_or(true) {
            true => Status::Enabled,
            false => Status::Disabled,
        };
        found.push(Installed {
            manifest,
            dir: path,
            status,
        });
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn install(root: &Path, id: &str) -> PathBuf {
        let dir = root.join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("plugin.json"),
            format!(r#"{{"id":"{id}","name":"{id}","capabilities":["events"]}}"#),
        )
        .unwrap();
        std::fs::write(dir.join("plugin.wasm"), b"\0asm\x01\0\0\0").unwrap();
        dir
    }

    #[test]
    fn a_missing_directory_is_simply_no_extensions() {
        let found = installed(
            &[PathBuf::from("/nonexistent/extensions")],
            &BTreeMap::new(),
        );
        assert!(found.is_empty());
    }

    #[test]
    fn reads_every_manifest_in_id_order() {
        let root = tempfile::tempdir().unwrap();
        install(root.path(), "com.example.zebra");
        install(root.path(), "com.example.alpha");

        let found = installed(&[root.path().to_path_buf()], &BTreeMap::new());
        let ids: Vec<&str> = found.iter().map(|e| e.id()).collect();
        assert_eq!(ids, ["com.example.alpha", "com.example.zebra"]);
        assert!(found.iter().all(|e| e.enabled()));
    }

    /// A disabled extension is still listed — the UI has to show it to let the
    /// user turn it back on.
    #[test]
    fn lists_disabled_extensions() {
        let root = tempfile::tempdir().unwrap();
        install(root.path(), "com.example.off");
        let enabled = BTreeMap::from([("com.example.off".to_string(), false)]);

        let found = installed(&[root.path().to_path_buf()], &enabled);
        assert_eq!(found.len(), 1);
        assert!(!found[0].enabled());
        assert_eq!(found[0].status.as_str(), "disabled");
    }

    /// A directory that is not an extension, and one whose manifest is broken,
    /// are both skipped rather than emptying the list.
    #[test]
    fn skips_what_is_not_an_extension() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("not-an-extension")).unwrap();
        std::fs::create_dir_all(root.path().join("broken")).unwrap();
        std::fs::write(root.path().join("broken/plugin.json"), "{ oops").unwrap();
        install(root.path(), "com.example.good");

        let found = installed(&[root.path().to_path_buf()], &BTreeMap::new());
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id(), "com.example.good");
    }

    /// The first search path wins, so a user's own copy shadows a system one
    /// instead of appearing twice.
    #[test]
    fn the_first_search_path_wins() {
        let user = tempfile::tempdir().unwrap();
        let system = tempfile::tempdir().unwrap();
        install(user.path(), "com.example.dup");
        install(system.path(), "com.example.dup");

        let found = installed(
            &[user.path().to_path_buf(), system.path().to_path_buf()],
            &BTreeMap::new(),
        );
        assert_eq!(found.len(), 1);
        assert!(found[0].dir.starts_with(user.path()));
    }
}
