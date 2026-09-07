//! Where extensions are found, and installing one from a URL.
//!
//! # Search paths
//!
//! By default extensions live under `<app dir>/extensions`. `extension_paths`
//! in `settings.toml` replaces that with an ordered list, so a user's own
//! extensions can sit apart from ones a package manager installed:
//!
//! ```toml
//! extension_paths = ["~/.config/music-player/extensions", "/usr/share/music-player/extensions"]
//! ```
//!
//! Earlier paths win: an extension id found in two places is loaded from the
//! first, which is what makes a local copy able to shadow a system one.
//!
//! # Installing from a URL
//!
//! [`install_from_url`] downloads a module into the cache directory and writes
//! a manifest beside it, so an extension published as a bare `.wasm` — the
//! usual shape — can be installed by pasting a link. A URL pointing at a
//! manifest instead installs the module it names.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Error};

use crate::manifest::{Manifest, MANIFEST_FILE, MANIFEST_FILES};
use crate::registry::EXTENSIONS_DIR;

/// Where extensions downloaded from a URL are kept. Under the app directory
/// rather than a settings path: those may be read-only or shared, and a
/// downloaded extension belongs to this machine.
pub const CACHE_DIR: &str = "extensions-cache";

/// A downloaded module larger than this is refused. An extension is a few
/// hundred kilobytes of WebAssembly; anything at this scale is a mistake or an
/// attempt to fill the disk.
const MAX_DOWNLOAD_BYTES: u64 = 64 * 1024 * 1024;

/// Expand a leading `~` and any `$VAR` in a configured path.
fn expand(path: &str) -> PathBuf {
    let mut expanded = path.trim().to_string();
    if let Some(rest) = expanded.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            expanded = home.join(rest).to_string_lossy().into_owned();
        }
    } else if expanded == "~" {
        if let Some(home) = dirs::home_dir() {
            expanded = home.to_string_lossy().into_owned();
        }
    }
    // `$VAR` and `${VAR}`; an unset variable expands to nothing, matching what
    // a shell would do.
    while let Some(start) = expanded.find('$') {
        let rest = &expanded[start + 1..];
        let (name, len) = if let Some(inner) = rest.strip_prefix('{') {
            match inner.find('}') {
                Some(end) => (&inner[..end], end + 3),
                None => break,
            }
        } else {
            let end = rest
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .unwrap_or(rest.len());
            if end == 0 {
                break;
            }
            (&rest[..end], end + 1)
        };
        let value = std::env::var(name).unwrap_or_default();
        expanded.replace_range(start..start + len, &value);
    }
    PathBuf::from(expanded)
}

/// Every directory to search for extensions, in priority order.
///
/// `configured` is `extension_paths` from settings; empty falls back to
/// `<app_dir>/extensions`. The download cache is always searched last, so an
/// extension installed from a URL is found without having to be configured,
/// but never shadows one the user placed deliberately.
pub fn search_paths(app_dir: &str, configured: &[String]) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = configured
        .iter()
        .filter(|path| !path.trim().is_empty())
        .map(|path| expand(path))
        .collect();
    if paths.is_empty() {
        paths.push(Path::new(app_dir).join(EXTENSIONS_DIR));
    }
    let cache = cache_dir(app_dir);
    if !paths.contains(&cache) {
        paths.push(cache);
    }
    // A path listed twice would load the same extension twice; the first
    // mention wins.
    let mut seen = Vec::new();
    paths.retain(|path| {
        if seen.contains(path) {
            return false;
        }
        seen.push(path.clone());
        true
    });
    paths
}

/// The directory holding extensions downloaded from a URL.
pub fn cache_dir(app_dir: &str) -> PathBuf {
    Path::new(app_dir).join(CACHE_DIR)
}

/// A URL that names a manifest rather than a module.
fn is_manifest_url(url: &str) -> bool {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    MANIFEST_FILES
        .iter()
        .any(|name| path.to_ascii_lowercase().ends_with(name))
}

/// Derive an extension id from a URL, for a bare `.wasm` that brings no
/// manifest. The host and path both matter: two different projects can each
/// publish `plugin.wasm`.
fn id_from_url(url: &str) -> String {
    let trimmed = url
        .split(['?', '#'])
        .next()
        .unwrap_or(url)
        .trim_end_matches('/');
    let stem = trimmed
        .rsplit('/')
        .next()
        .unwrap_or("extension")
        .trim_end_matches(".wasm");
    let host = trimmed
        .split("://")
        .nth(1)
        .and_then(|rest| rest.split('/').next())
        .unwrap_or("url");
    let safe = |value: &str| -> String {
        value
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                    c
                } else {
                    '-'
                }
            })
            .collect()
    };
    let id = format!("{}.{}", safe(host), safe(stem));
    if id.trim_matches(['.', '-'].as_ref()).is_empty() {
        "downloaded.extension".to_string()
    } else {
        id
    }
}

/// Install an extension from `url` into the cache under `app_dir`, returning
/// the directory it was written to.
///
/// Accepts either a bare `.wasm` — in which case a minimal manifest is written
/// for it — or a `plugin.toml` / `plugin.json`, whose `entry` is then fetched
/// from the same location.
///
/// The download is staged in a temporary directory and moved into place only
/// once it is complete and its manifest parses, so a failed or interrupted
/// install cannot leave a half-written extension to be loaded next start.
pub async fn install_from_url(
    app_dir: &str,
    url: &str,
    capabilities: &[crate::manifest::Capability],
) -> Result<PathBuf, Error> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(anyhow!(
            "an extension url must start with http:// or https://"
        ));
    }
    let client = reqwest::Client::builder()
        .user_agent(concat!(
            "music-player-extensions/",
            env!("CARGO_PKG_VERSION")
        ))
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let staging = tempfile::tempdir()?;
    let manifest = if is_manifest_url(url) {
        install_manifest(&client, url, staging.path()).await?
    } else {
        install_bare_wasm(&client, url, staging.path(), capabilities).await?
    };

    let target = cache_dir(app_dir).join(&manifest.id);
    // Loading the staged copy is the last check: a module that will not open
    // is rejected here rather than on the next start.
    Manifest::load(staging.path())
        .with_context(|| format!("the extension downloaded from {url} is not usable"))?;

    if target.exists() {
        std::fs::remove_dir_all(&target)
            .with_context(|| format!("replacing {}", target.display()))?;
    }
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // `rename` across filesystems fails, and the temp dir often is one, so
    // copy the (few) files rather than moving the directory.
    std::fs::create_dir_all(&target)?;
    for entry in std::fs::read_dir(staging.path())?.flatten() {
        std::fs::copy(entry.path(), target.join(entry.file_name()))?;
    }
    tracing::info!(extension = %manifest.id, %url, path = %target.display(), "installed extension");
    Ok(target)
}

/// Download a `.wasm` and write a manifest for it.
async fn install_bare_wasm(
    client: &reqwest::Client,
    url: &str,
    into: &Path,
    capabilities: &[crate::manifest::Capability],
) -> Result<Manifest, Error> {
    if capabilities.is_empty() {
        return Err(anyhow!(
            "a bare .wasm carries no manifest, so its capabilities have to be given"
        ));
    }
    let bytes = download(client, url).await?;
    std::fs::write(into.join("plugin.wasm"), &bytes)?;

    let id = id_from_url(url);
    let manifest = Manifest {
        id: id.clone(),
        name: id,
        version: String::new(),
        author: String::new(),
        description: format!("Installed from {url}"),
        homepage: url.to_owned(),
        repository: String::new(),
        license: String::new(),
        logo: String::new(),
        topics: Vec::new(),
        readme: String::new(),
        entry: "plugin.wasm".to_string(),
        capabilities: capabilities.to_vec(),
        permissions: Default::default(),
    };
    std::fs::write(
        into.join(MANIFEST_FILE),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    Ok(manifest)
}

/// Download a manifest, then the module it names from the same location.
async fn install_manifest(
    client: &reqwest::Client,
    url: &str,
    into: &Path,
) -> Result<Manifest, Error> {
    let raw = download(client, url).await?;
    let name = url
        .split(['?', '#'])
        .next()
        .unwrap_or(url)
        .rsplit('/')
        .next()
        .unwrap_or(MANIFEST_FILE);
    std::fs::write(into.join(name), &raw)?;

    // Parsed only to learn the entry file name; `Manifest::load` re-reads and
    // validates it once the module is in place.
    let manifest: Manifest = if name.ends_with(".toml") {
        toml::from_str(std::str::from_utf8(&raw)?)?
    } else {
        serde_json::from_slice(&raw)?
    };

    // The entry is resolved against the manifest's own location, so a manifest
    // cannot point the download at an unrelated host.
    if manifest.entry.contains("://") || manifest.entry.contains("..") {
        return Err(anyhow!(
            "the manifest's entry '{}' must be a plain file name",
            manifest.entry
        ));
    }
    let base = url
        .rsplit_once('/')
        .map(|(base, _)| base)
        .ok_or_else(|| anyhow!("cannot resolve '{}' against {url}", manifest.entry))?;
    let wasm = download(client, &format!("{base}/{}", manifest.entry)).await?;
    std::fs::write(into.join(&manifest.entry), wasm)?;
    Ok(manifest)
}

/// Fetch a URL, refusing anything implausibly large.
async fn download(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, Error> {
    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("downloading {url}"))?
        .error_for_status()
        .with_context(|| format!("downloading {url}"))?;
    // Checked before reading when the server declares it, and again after, for
    // a server that does not.
    if let Some(length) = response.content_length() {
        if length > MAX_DOWNLOAD_BYTES {
            return Err(anyhow!("{url} is {length} bytes, which is too large"));
        }
    }
    let bytes = response.bytes().await?;
    if bytes.len() as u64 > MAX_DOWNLOAD_BYTES {
        return Err(anyhow!("{url} is too large"));
    }
    Ok(bytes.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::Capability;

    #[test]
    fn falls_back_to_the_app_directory() {
        let paths = search_paths("/app", &[]);
        assert_eq!(paths[0], PathBuf::from("/app/extensions"));
        // The download cache is always searched, without being configured.
        assert!(paths.contains(&PathBuf::from("/app/extensions-cache")));
    }

    /// Configured paths replace the default and keep their order, so a local
    /// copy can shadow a system-wide one.
    #[test]
    fn configured_paths_replace_the_default_and_keep_their_order() {
        let paths = search_paths("/app", &["/first".to_string(), "/second".to_string()]);
        assert_eq!(paths[0], PathBuf::from("/first"));
        assert_eq!(paths[1], PathBuf::from("/second"));
        assert!(!paths.contains(&PathBuf::from("/app/extensions")));
    }

    /// A path listed twice would load the same extension twice.
    #[test]
    fn duplicate_paths_are_collapsed() {
        let paths = search_paths("/app", &["/same".into(), "/same".into()]);
        assert_eq!(paths.iter().filter(|p| *p == Path::new("/same")).count(), 1);
    }

    #[test]
    fn expands_home_and_environment_variables() {
        std::env::set_var("MP_TEST_EXT_DIR", "/opt/ext");
        assert_eq!(
            expand("$MP_TEST_EXT_DIR/here"),
            PathBuf::from("/opt/ext/here")
        );
        assert_eq!(expand("${MP_TEST_EXT_DIR}"), PathBuf::from("/opt/ext"));
        // An unset variable expands to nothing, as a shell would.
        assert_eq!(expand("$MP_TEST_UNSET_VAR/x"), PathBuf::from("/x"));

        if let Some(home) = dirs::home_dir() {
            assert_eq!(expand("~/ext"), home.join("ext"));
        }
    }

    #[test]
    fn recognises_manifest_urls() {
        assert!(is_manifest_url("https://example.com/plugin.toml"));
        assert!(is_manifest_url("https://example.com/plugin.json?v=2"));
        assert!(!is_manifest_url("https://example.com/plugin.wasm"));
    }

    /// Two projects can each publish `plugin.wasm`; the host has to be part of
    /// the derived id or the second install would overwrite the first.
    #[test]
    fn derives_a_distinct_id_per_host() {
        let a = id_from_url("https://alice.example/plugin.wasm");
        let b = id_from_url("https://bob.example/plugin.wasm");
        assert_ne!(a, b);
        assert!(a.starts_with("alice.example"));
        // And the result is a legal manifest id.
        assert!(a
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_')));
    }

    #[tokio::test]
    async fn refuses_a_non_http_url() {
        let e = install_from_url("/app", "file:///etc/passwd", &[Capability::Events])
            .await
            .unwrap_err();
        assert!(e.to_string().contains("must start with http"));
    }

    /// Without a manifest there is nothing to say what the module is for, so
    /// the caller has to supply it.
    #[tokio::test]
    async fn refuses_a_bare_wasm_with_no_capabilities() {
        let e = install_from_url("/app", "https://example.com/plugin.wasm", &[])
            .await
            .unwrap_err();
        assert!(e.to_string().contains("capabilities have to be given"));
    }
}
