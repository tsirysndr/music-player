//! What an extension declares about itself.
//!
//! Every extension ships a `plugin.json` next to its `.wasm`. Declaring
//! capabilities up front — rather than probing the module for exported
//! functions — means the host knows what an extension is for before it runs any
//! of its code, so a metadata provider is never asked to handle an event and a
//! broken module cannot masquerade as something it is not.
//!
//! The layout borrows from Kodi's `addon.xml` and Navidrome's plugin manifests:
//! identity, permissions, and a list of what it plugs into.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Error};
use serde::{Deserialize, Serialize};

/// The manifest file names an extension directory may use, in the order they
/// are looked for. TOML reads better by hand — comments, no trailing-comma
/// traps — while JSON is what a build step or a registry emits; both describe
/// exactly the same [`Manifest`].
pub const MANIFEST_FILES: &[&str] = &["plugin.toml", "plugin.json"];

/// The JSON manifest name, kept for callers that write one out.
pub const MANIFEST_FILE: &str = "plugin.json";

/// What an extension is allowed to plug into. An extension only ever receives
/// the calls for capabilities it declared.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// Reacts to things happening: plays, likes, scans. Export one function per
    /// [`Event`](crate::host::Event) name it cares about.
    Events,
    /// Supplies data the player does not have: lyrics, artwork, bios. Exports
    /// `get_lyrics`, `get_artwork`, `get_artist_bio`.
    Metadata,
    /// Registers named actions the UI can invoke. Exports `commands` (listing
    /// them) and `run_command`.
    Commands,
    /// Adds predicates usable in smart-playlist filters. Exports `predicates`
    /// and `evaluate`.
    Predicates,
    /// Provides a browsable media source of its own — a streaming service, a
    /// remote share, anything with albums and playable tracks. Exports
    /// `source_info`, `list_albums`, `list_artists`, `list_tracks`,
    /// `list_playlists` and `get_stream_url`.
    Source,
}

impl Capability {
    pub fn as_str(self) -> &'static str {
        match self {
            Capability::Events => "events",
            Capability::Metadata => "metadata",
            Capability::Commands => "commands",
            Capability::Predicates => "predicates",
            Capability::Source => "source",
        }
    }
}

/// What an extension may reach outside its sandbox.
///
/// WebAssembly gives nothing by default — no network, no disk, no clock. Each
/// field here opens exactly one door, and the manifest is where the user can
/// read what an extension asked for before enabling it.
///
/// Calling an external API lives here rather than in [`Capability`] on purpose:
/// a capability says what an extension *does for the player*, a permission says
/// what it may *touch*. Any of the five capabilities can legitimately need the
/// network — a metadata provider fetching lyrics, a source listing a catalogue,
/// an events extension scrobbling — so gating it on one would either block
/// honest extensions or wave through the rest.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permissions {
    /// Hosts the extension may make HTTP requests to. Exact hostnames or
    /// `*.example.com`. Empty means no network at all.
    #[serde(default)]
    pub allowed_hosts: Vec<String>,
    /// Config keys the extension reads, with their defaults. The user's own
    /// values override these; a key that is not declared is not visible.
    #[serde(default)]
    pub config: BTreeMap<String, String>,
    /// Whether the extension may query the library (tracks, albums, artists)
    /// through the host functions.
    #[serde(default)]
    pub library_read: bool,
}

/// A parsed `plugin.json`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    /// Stable id, e.g. `com.example.lyrics`. Also the row key in the database.
    pub id: String,
    /// Display name, shown wherever the extension is listed.
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
    /// The project's website.
    #[serde(default)]
    pub homepage: String,
    /// Source repository, so a user can read what they are about to run.
    #[serde(default)]
    pub repository: String,
    /// SPDX identifier, e.g. `MIT` or `Apache-2.0`.
    #[serde(default)]
    pub license: String,
    /// An icon for the extensions list. A URL, or a file name relative to the
    /// manifest.
    #[serde(default)]
    pub logo: String,
    /// Free-form tags for searching and grouping, e.g. `["lyrics", "offline"]`.
    #[serde(default)]
    pub topics: Vec<String>,
    /// A README shipped with the extension, relative to the manifest. What the
    /// details page shows so a user can read what an extension does — and how
    /// to configure it — without leaving the app.
    #[serde(default)]
    pub readme: String,
    /// The `.wasm` file, relative to the manifest.
    #[serde(default = "default_entry")]
    pub entry: String,
    /// What this extension plugs into.
    #[serde(default)]
    pub capabilities: Vec<Capability>,
    #[serde(default)]
    pub permissions: Permissions,
}

fn default_entry() -> String {
    "plugin.wasm".to_string()
}

/// Where an extension's logo comes from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogoSource {
    /// A remote image.
    Url(String),
    /// An image shipped inside the extension directory.
    File(PathBuf),
}

/// Parse a manifest, picking the format from the file extension.
fn parse(raw: &str, path: &Path) -> Result<Manifest, Error> {
    if path.extension().is_some_and(|ext| ext == "toml") {
        Ok(toml::from_str(raw)?)
    } else {
        Ok(serde_json::from_str(raw)?)
    }
}

impl Manifest {
    /// Read and check the manifest in `dir` — `plugin.toml` if present,
    /// otherwise `plugin.json`.
    pub fn load(dir: &Path) -> Result<Self, Error> {
        let path = MANIFEST_FILES
            .iter()
            .map(|name| dir.join(name))
            .find(|path| path.exists())
            .ok_or_else(|| anyhow!("{} has no {}", dir.display(), MANIFEST_FILES.join(" or ")))?;
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let manifest: Manifest =
            parse(&raw, &path).with_context(|| format!("parsing {}", path.display()))?;
        manifest.validate()?;
        if !dir.join(&manifest.entry).exists() {
            return Err(anyhow!(
                "{} declares entry '{}', which does not exist",
                path.display(),
                manifest.entry
            ));
        }
        Ok(manifest)
    }

    fn validate(&self) -> Result<(), Error> {
        if self.id.trim().is_empty() {
            return Err(anyhow!("the manifest needs an id"));
        }
        // The id becomes a database key and a directory name; keeping it to a
        // reverse-DNS-ish charset avoids having to escape it anywhere.
        if !self
            .id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        {
            return Err(anyhow!(
                "the id '{}' may only contain letters, digits, '.', '-' and '_'",
                self.id
            ));
        }
        if self.name.trim().is_empty() {
            return Err(anyhow!("the manifest needs a name"));
        }
        if self.capabilities.is_empty() {
            return Err(anyhow!(
                "'{}' declares no capabilities, so nothing would ever call it",
                self.id
            ));
        }
        Ok(())
    }

    pub fn has(&self, capability: Capability) -> bool {
        self.capabilities.contains(&capability)
    }

    /// Absolute path to the `.wasm`, given the directory the manifest is in.
    pub fn entry_path(&self, dir: &Path) -> PathBuf {
        dir.join(&self.entry)
    }

    /// The bundled README, if the manifest names one that exists.
    ///
    /// Resolved against the extension's own directory and refused if it points
    /// outside — a manifest must not be able to make the details page display
    /// an arbitrary file from the user's disk.
    pub fn readme_path(&self, dir: &Path) -> Option<PathBuf> {
        let readme = self.readme.trim();
        if readme.is_empty() || readme.contains("..") || Path::new(readme).is_absolute() {
            return None;
        }
        let path = dir.join(readme);
        path.is_file().then_some(path)
    }

    /// The logo as something a UI can load: a URL passed through as-is, or a
    /// bundled file resolved against the extension's own directory.
    pub fn logo_source(&self, dir: &Path) -> Option<LogoSource> {
        let logo = self.logo.trim();
        if logo.is_empty() {
            return None;
        }
        if logo.starts_with("http://") || logo.starts_with("https://") {
            return Some(LogoSource::Url(logo.to_owned()));
        }
        // A bundled logo may not escape the extension's directory.
        if logo.contains("..") || Path::new(logo).is_absolute() {
            return None;
        }
        Some(LogoSource::File(dir.join(logo)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, manifest: &str) {
        write_as(dir, MANIFEST_FILE, manifest);
    }

    fn write_as(dir: &Path, name: &str, manifest: &str) {
        std::fs::write(dir.join(name), manifest).unwrap();
        std::fs::write(dir.join("plugin.wasm"), b"\0asm").unwrap();
    }

    #[test]
    fn reads_a_minimal_manifest() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            r#"{"id":"com.example.hello","name":"Hello","capabilities":["events"]}"#,
        );
        let manifest = Manifest::load(dir.path()).unwrap();
        assert_eq!(manifest.id, "com.example.hello");
        assert_eq!(manifest.entry, "plugin.wasm");
        assert!(manifest.has(Capability::Events));
        assert!(!manifest.has(Capability::Metadata));
        // Nothing is granted unless it is asked for.
        assert!(manifest.permissions.allowed_hosts.is_empty());
        assert!(!manifest.permissions.library_read);
    }

    #[test]
    fn reads_permissions_and_config_defaults() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            r#"{
                "id": "com.example.lyrics",
                "name": "Lyrics",
                "capabilities": ["metadata"],
                "permissions": {
                    "allowedHosts": ["api.example.com"],
                    "config": {"apiKey": ""},
                    "libraryRead": true
                }
            }"#,
        );
        let manifest = Manifest::load(dir.path()).unwrap();
        assert_eq!(manifest.permissions.allowed_hosts, ["api.example.com"]);
        assert!(manifest.permissions.config.contains_key("apiKey"));
        assert!(manifest.permissions.library_read);
    }

    /// An extension that declares nothing would be loaded and then never
    /// called — better to say so than to leave the author guessing.
    #[test]
    fn rejects_a_manifest_with_no_capabilities() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), r#"{"id":"x","name":"X","capabilities":[]}"#);
        let e = Manifest::load(dir.path()).unwrap_err();
        assert!(e.to_string().contains("no capabilities"));
    }

    #[test]
    fn rejects_ids_that_would_need_escaping() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            r#"{"id":"../etc/passwd","name":"X","capabilities":["events"]}"#,
        );
        assert!(Manifest::load(dir.path())
            .unwrap_err()
            .to_string()
            .contains("may only contain"));
    }

    /// The descriptive fields are what a listing shows before anyone runs the
    /// extension, so they all have to survive a round trip.
    #[test]
    fn reads_the_descriptive_fields() {
        let dir = tempfile::tempdir().unwrap();
        write_as(
            dir.path(),
            "plugin.toml",
            r#"
                id = "com.example.lyrics"
                name = "Lyrics"
                version = "1.2.0"
                author = "Someone"
                description = "Fetches lyrics"
                homepage = "https://example.com"
                repository = "https://github.com/someone/lyrics"
                license = "MIT"
                logo = "icon.png"
                topics = ["lyrics", "offline"]
                capabilities = ["metadata"]
            "#,
        );
        let manifest = Manifest::load(dir.path()).unwrap();
        assert_eq!(manifest.version, "1.2.0");
        assert_eq!(manifest.author, "Someone");
        assert_eq!(manifest.description, "Fetches lyrics");
        assert_eq!(manifest.repository, "https://github.com/someone/lyrics");
        assert_eq!(manifest.license, "MIT");
        assert_eq!(manifest.topics, ["lyrics", "offline"]);
    }

    /// A bundled README is shown on the details page, so a manifest must not be
    /// able to point it at an arbitrary file on the user's disk.
    #[test]
    fn resolves_the_readme_without_escaping_the_directory() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            r#"{"id":"x","name":"X","capabilities":["events"],"readme":"README.md"}"#,
        );
        std::fs::write(dir.path().join("README.md"), "# X").unwrap();

        let manifest = Manifest::load(dir.path()).unwrap();
        assert_eq!(
            manifest.readme_path(dir.path()),
            Some(dir.path().join("README.md"))
        );

        for bad in ["../../../etc/passwd", "/etc/passwd", "nope.md", ""] {
            let manifest = Manifest {
                readme: bad.into(),
                ..manifest.clone()
            };
            assert!(
                manifest.readme_path(dir.path()).is_none(),
                "{bad} should not resolve"
            );
        }
    }

    /// A bundled logo resolves inside the extension directory; a remote one is
    /// passed through; and neither may point outside.
    #[test]
    fn resolves_the_logo() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            r#"{"id":"x","name":"X","capabilities":["events"],"logo":"icon.png"}"#,
        );
        let manifest = Manifest::load(dir.path()).unwrap();
        assert_eq!(
            manifest.logo_source(dir.path()),
            Some(LogoSource::File(dir.path().join("icon.png")))
        );

        let remote = Manifest {
            logo: "https://example.com/icon.png".into(),
            ..manifest.clone()
        };
        assert_eq!(
            remote.logo_source(dir.path()),
            Some(LogoSource::Url("https://example.com/icon.png".into()))
        );

        // No logo, and no escaping the directory.
        let none = Manifest {
            logo: String::new(),
            ..manifest.clone()
        };
        assert!(none.logo_source(dir.path()).is_none());
        let escaping = Manifest {
            logo: "../../etc/passwd".into(),
            ..manifest
        };
        assert!(escaping.logo_source(dir.path()).is_none());
    }

    /// TOML and JSON describe the same manifest; an author picks whichever
    /// they prefer to write by hand.
    #[test]
    fn reads_a_toml_manifest() {
        let dir = tempfile::tempdir().unwrap();
        write_as(
            dir.path(),
            "plugin.toml",
            r#"
                id = "com.example.lyrics"
                name = "Lyrics"
                capabilities = ["metadata"]

                [permissions]
                allowedHosts = ["api.example.com"]
                libraryRead = true

                [permissions.config]
                apiKey = ""
            "#,
        );
        let manifest = Manifest::load(dir.path()).unwrap();
        assert_eq!(manifest.id, "com.example.lyrics");
        assert!(manifest.has(Capability::Metadata));
        assert_eq!(manifest.permissions.allowed_hosts, ["api.example.com"]);
        assert!(manifest.permissions.library_read);
        assert!(manifest.permissions.config.contains_key("apiKey"));
    }

    /// With both present the TOML wins, so an author editing the readable one
    /// is not silently overridden by a stale generated JSON.
    #[test]
    fn toml_takes_precedence_over_json() {
        let dir = tempfile::tempdir().unwrap();
        write_as(
            dir.path(),
            "plugin.json",
            r#"{"id":"from.json","name":"JSON","capabilities":["events"]}"#,
        );
        write_as(
            dir.path(),
            "plugin.toml",
            "id = \"from.toml\"\nname = \"TOML\"\ncapabilities = [\"events\"]\n",
        );
        assert_eq!(Manifest::load(dir.path()).unwrap().id, "from.toml");
    }

    #[test]
    fn reports_a_directory_with_no_manifest() {
        let dir = tempfile::tempdir().unwrap();
        assert!(Manifest::load(dir.path())
            .unwrap_err()
            .to_string()
            .contains("plugin.toml or plugin.json"));
    }

    #[test]
    fn rejects_a_missing_wasm_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(MANIFEST_FILE),
            r#"{"id":"x","name":"X","capabilities":["events"],"entry":"nope.wasm"}"#,
        )
        .unwrap();
        assert!(Manifest::load(dir.path())
            .unwrap_err()
            .to_string()
            .contains("does not exist"));
    }
}
