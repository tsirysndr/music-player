//! `music-player extension` — scaffold, list and install WebAssembly extensions.
//!
//! `init` writes a complete, buildable extension: a manifest filled in from the
//! answers, and — when the `xtp` CLI is on PATH — typed bindings and a skeleton
//! generated from `schema.yaml` for the chosen language. Without `xtp` it still
//! writes the manifest and a README explaining the one command needed to get
//! the rest, rather than failing and leaving nothing behind.

use std::path::{Path, PathBuf};

use music_player_extensions::manifest::MANIFEST_FILES;
use music_player_extensions::manifest::{Capability, Permissions};
use music_player_extensions::{install_from_url, search_paths, Manifest, Registry};
use music_player_settings::{get_application_directory, read_settings, Settings};
use owo_colors::OwoColorize;

type CmdResult = Result<(), Box<dyn std::error::Error>>;

/// The languages `xtp plugin init` can generate bindings for.
const LANGUAGES: &[&str] = &["rust", "go", "typescript", "python", "csharp", "zig", "cpp"];

/// Every capability, with the one-line description the prompt shows.
const CAPABILITIES: &[(&str, Capability, &str)] = &[
    (
        "events",
        Capability::Events,
        "React to plays, skips, likes and scans",
    ),
    (
        "metadata",
        Capability::Metadata,
        "Supply lyrics, artwork, bios or genres",
    ),
    ("commands", Capability::Commands, "Add actions to the UI"),
    (
        "predicates",
        Capability::Predicates,
        "Add smart-playlist filter terms",
    ),
    (
        "source",
        Capability::Source,
        "Provide a browsable media source",
    ),
];

fn parse_capabilities(raw: &str) -> Result<Vec<Capability>, String> {
    let mut parsed = Vec::new();
    for name in raw.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        match CAPABILITIES
            .iter()
            .find(|(label, _, _)| *label == name.to_ascii_lowercase())
        {
            Some((_, capability, _)) => {
                if !parsed.contains(capability) {
                    parsed.push(*capability);
                }
            }
            None => {
                return Err(format!(
                    "unknown capability '{name}' — try one of: {}",
                    CAPABILITIES
                        .iter()
                        .map(|(label, _, _)| *label)
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            }
        }
    }
    Ok(parsed)
}

/// The configured extension paths, or none when settings cannot be read —
/// which only means falling back to the default location.
fn extension_paths() -> Vec<String> {
    read_settings()
        .ok()
        .and_then(|config| config.try_deserialize::<Settings>().ok())
        .map(|settings| settings.extension_paths)
        .unwrap_or_default()
}

/// `extension init` — scaffold a new extension.
pub async fn init(
    id: &str,
    name: Option<&str>,
    capabilities_arg: &str,
    language: &str,
    path: Option<&str>,
) -> CmdResult {
    let capabilities = parse_capabilities(capabilities_arg)?;
    if capabilities.is_empty() {
        return Err("an extension needs at least one capability (--capabilities)".into());
    }
    if !LANGUAGES.contains(&language) {
        return Err(format!(
            "unknown language '{language}' — try one of: {}",
            LANGUAGES.join(", ")
        )
        .into());
    }

    let name = name.unwrap_or_else(|| id.rsplit('.').next().unwrap_or(id));
    let dir = PathBuf::from(path.unwrap_or(name));
    if dir.exists()
        && dir
            .read_dir()
            .map(|mut d| d.next().is_some())
            .unwrap_or(false)
    {
        return Err(format!("{} already exists and is not empty", dir.display()).into());
    }
    std::fs::create_dir_all(&dir)?;

    let manifest = Manifest {
        id: id.to_owned(),
        name: name.to_owned(),
        version: "0.1.0".to_owned(),
        author: String::new(),
        description: String::new(),
        homepage: String::new(),
        repository: String::new(),
        license: "MIT".to_owned(),
        logo: String::new(),
        topics: Vec::new(),
        readme: String::new(),
        entry: "plugin.wasm".to_owned(),
        capabilities: capabilities.clone(),
        permissions: Permissions::default(),
    };
    std::fs::write(dir.join("plugin.toml"), manifest_toml(&manifest))?;

    let generated = generate_bindings(&dir, language)?;
    std::fs::write(
        dir.join("README.md"),
        readme(&manifest, language, generated),
    )?;

    println!("{} {}", "Created".green().bold(), dir.display());
    println!("  id            {id}");
    println!(
        "  capabilities  {}",
        capabilities
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!();
    if generated {
        println!("Bindings generated for {language}. Next:");
        println!("  cd {}", dir.display());
        println!("  {}", build_command(language));
    } else {
        println!(
            "{} the {} CLI was not found, so only the manifest was written.",
            "Note:".yellow().bold(),
            "xtp".bold()
        );
        println!("Install it from https://docs.xtp.dylibso.com, then run:");
        println!(
            "  xtp plugin init --schema-file extensions/schema.yaml \\\n    \
             --template {language} --path {} --yes",
            dir.display()
        );
    }
    println!();
    println!("Then copy plugin.toml and the built .wasm into:");
    println!("  {}", install_dir().display());
    Ok(())
}

/// `extension list` — what is installed, and whether it loaded.
pub async fn list() -> CmdResult {
    let app_dir = get_application_directory();
    let paths = search_paths(&app_dir, &extension_paths());

    println!("{}", "Search paths".bold());
    for path in &paths {
        let marker = if path.exists() { " " } else { "?" };
        println!("  {marker} {}", path.display());
    }
    println!();

    // Nothing is enabled or configured from here: this is a report of what the
    // daemon would load, not a second source of truth.
    let registry = Registry::load_all(
        &paths,
        &Default::default(),
        &Default::default(),
        Default::default(),
    );
    if registry.is_empty() {
        println!("No extensions installed.");
        println!(
            "Create one with {} or install one with {}.",
            "music-player extension init <id>".bold(),
            "music-player extension install <url>".bold()
        );
        return Ok(());
    }

    println!("{}", "Installed".bold());
    for installed in registry.installed() {
        let status = match (&installed.extension, &installed.error) {
            (Some(_), _) => "loaded".green().to_string(),
            (None, Some(e)) => format!("{} {e}", "failed".red()),
            (None, None) => "disabled".yellow().to_string(),
        };
        println!(
            "  {} {} {}",
            installed.id().bold(),
            installed.manifest.version,
            status
        );
        println!(
            "      {} · {}",
            installed
                .manifest
                .capabilities
                .iter()
                .map(|c| c.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            installed.dir.display()
        );
        if !installed.manifest.description.is_empty() {
            println!("      {}", installed.manifest.description);
        }
    }
    Ok(())
}

/// `extension install <url>` — download an extension into the cache.
pub async fn install(url: &str, capabilities_arg: &str) -> CmdResult {
    let capabilities = parse_capabilities(capabilities_arg)?;
    let app_dir = get_application_directory();
    let dir = install_from_url(&app_dir, url, &capabilities).await?;
    let manifest = Manifest::load(&dir)?;
    println!(
        "{} {} {} into {}",
        "Installed".green().bold(),
        manifest.id.bold(),
        manifest.version,
        dir.display()
    );
    println!("Restart music-player to load it.");
    Ok(())
}

/// `extension uninstall <id>` — remove an installed extension.
pub async fn uninstall(id: &str, yes: bool) -> CmdResult {
    let app_dir = get_application_directory();
    let paths = search_paths(&app_dir, &extension_paths());

    // Every copy, not just the first: an id can exist in several search paths,
    // and removing only the shadowing one would leave it apparently installed.
    let found: Vec<PathBuf> = paths
        .iter()
        .map(|path| path.join(id))
        .filter(|dir| dir.is_dir() && MANIFEST_FILES.iter().any(|name| dir.join(name).exists()))
        .collect();

    if found.is_empty() {
        return Err(format!(
            "'{id}' is not installed — run `music-player extension list` to see what is"
        )
        .into());
    }

    for dir in &found {
        println!("  {}", dir.display());
    }
    if !yes {
        print!(
            "Remove {} {}? [y/N] ",
            found.len(),
            if found.len() == 1 {
                "directory"
            } else {
                "directories"
            }
        );
        use std::io::Write;
        std::io::stdout().flush()?;
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer)?;
        if !matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
            println!("Left alone.");
            return Ok(());
        }
    }

    for dir in &found {
        std::fs::remove_dir_all(dir)
            .map_err(|e| format!("could not remove {}: {e}", dir.display()))?;
    }
    println!("{} {}", "Uninstalled".green().bold(), id.bold());
    println!("Restart music-player to unload it.");
    Ok(())
}

/// Where a built extension is copied to be picked up.
fn install_dir() -> PathBuf {
    let app_dir = get_application_directory();
    search_paths(&app_dir, &extension_paths())
        .into_iter()
        .next()
        .unwrap_or_else(|| PathBuf::from(app_dir).join("extensions"))
}

/// The host/guest contract, compiled into the binary.
///
/// Embedded rather than read from disk so `extension init` works from any
/// directory — an installed binary has no source checkout to look in, and the
/// schema is the one thing scaffolding cannot do without.
const SCHEMA: &str = include_str!("../extensions/schema.yaml");

/// A source checkout's schema, if this is running in one. Preferred over the
/// embedded copy so someone editing the schema scaffolds against their edit.
fn checkout_schema() -> Option<PathBuf> {
    let path = PathBuf::from("extensions/schema.yaml");
    path.exists().then_some(path)
}

/// Run `xtp plugin init` to generate bindings. Returns whether it ran.
fn generate_bindings(dir: &Path, language: &str) -> Result<bool, Box<dyn std::error::Error>> {
    // The embedded schema is written out only when there is no checkout copy,
    // and into a temp dir that is cleaned up on the way out.
    let staging = tempfile::tempdir()?;
    let schema = match checkout_schema() {
        Some(path) => path,
        None => {
            let path = staging.path().join("schema.yaml");
            std::fs::write(&path, SCHEMA)?;
            path
        }
    };
    let status = std::process::Command::new("xtp")
        .args([
            "plugin",
            "init",
            "--schema-file",
            &schema.to_string_lossy(),
            "--template",
            language,
            "--path",
            &dir.to_string_lossy(),
            "--yes",
        ])
        .status();
    match status {
        Ok(status) if status.success() => Ok(true),
        Ok(status) => {
            tracing::debug!("xtp plugin init exited with {status}");
            Ok(false)
        }
        // `xtp` not being installed is the common case, not an error: the
        // manifest is written either way and the README says what to run.
        Err(e) => {
            tracing::debug!("could not run xtp: {e}");
            Ok(false)
        }
    }
}

fn build_command(language: &str) -> &'static str {
    match language {
        "rust" => "cargo build --release --target wasm32-wasip1",
        "go" => "tinygo build -o plugin.wasm -target wasi .",
        "typescript" => "npm install && npm run build",
        "python" => "pip install -r requirements.txt && ./build.sh",
        "csharp" => "dotnet build -c Release",
        "zig" => "zig build -Dtarget=wasm32-wasi -Doptimize=ReleaseSmall",
        _ => "see the generated build script",
    }
}

/// Serialize a manifest as commented TOML — friendlier to edit than the
/// round-tripped output `toml::to_string` would give.
fn manifest_toml(manifest: &Manifest) -> String {
    let capabilities = manifest
        .capabilities
        .iter()
        .map(|c| format!("\"{}\"", c.as_str()))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        r#"# What the host reads before it runs any of your code.
# See https://github.com/tsirysndr/music-player/tree/master/extensions

id = "{id}"
name = "{name}"
version = "{version}"
author = ""
description = ""
homepage = ""
repository = ""
license = "{license}"
# A bundled file next to this manifest, or an https:// url.
logo = ""
# Free-form tags, for searching and grouping.
topics = []
# A README shipped alongside, shown on the extension's details page.
readme = "README.md"

# The module, relative to this file.
entry = "{entry}"

# The host calls only the exports these cover.
capabilities = [{capabilities}]

[permissions]
# Hosts this extension may reach. Empty means no network at all.
allowedHosts = []
# Whether it may read the user's library (tracks, albums, artists, playlists,
# saved radios).
libraryRead = false

# Only keys declared here are visible to get_config, and only these are offered
# in the settings UI.
[permissions.config]
"#,
        id = manifest.id,
        name = manifest.name,
        version = manifest.version,
        license = manifest.license,
        entry = manifest.entry,
        capabilities = capabilities,
    )
}

fn readme(manifest: &Manifest, language: &str, generated: bool) -> String {
    let exports = manifest
        .capabilities
        .iter()
        .map(|capability| match capability {
            Capability::Events => {
                "- `on_track_played`, `on_track_skipped`, `on_track_liked`, \
                 `on_playlist_created`, `on_scan_completed`"
            }
            Capability::Metadata => "- `get_metadata`",
            Capability::Commands => "- `commands`, `run_command`",
            Capability::Predicates => "- `predicates`, `evaluate`",
            Capability::Source => {
                "- `source_info`, `list_albums`, `list_artists`, `list_tracks`, \
                 `list_playlists`, `get_stream_url`"
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let next = if generated {
        format!("```sh\n{}\n```", build_command(language))
    } else {
        format!(
            "Bindings were not generated — the `xtp` CLI was not found.\n\n\
             ```sh\n\
             xtp plugin init --schema-file extensions/schema.yaml \\\n  \
             --template {language} --path . --yes\n\
             {}\n\
             ```",
            build_command(language)
        )
    };

    format!(
        "# {name}\n\n\
         A music-player extension.\n\n\
         ## Implement\n\n\
         {exports}\n\n\
         `pdk` exports every function in the schema, so stub the ones you do not\n\
         implement. That is harmless: `plugin.toml` is what decides which are\n\
         ever called.\n\n\
         ## Build\n\n\
         {next}\n\n\
         ## Install\n\n\
         Copy `plugin.toml` and the built `.wasm` into a directory named after\n\
         the extension id, under one of music-player's extension paths:\n\n\
         ```sh\n\
         music-player extension list   # shows where those are\n\
         ```\n\n\
         Then restart music-player.\n",
        name = manifest.name,
        exports = exports,
        next = next,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_capability_lists() {
        assert_eq!(
            parse_capabilities("events,metadata").unwrap(),
            vec![Capability::Events, Capability::Metadata]
        );
        // Case and spacing are forgiving; duplicates collapse.
        assert_eq!(
            parse_capabilities(" Events , events ").unwrap(),
            vec![Capability::Events]
        );
        assert!(parse_capabilities("").unwrap().is_empty());
    }

    /// A typo should name the alternatives rather than just failing.
    #[test]
    fn rejects_an_unknown_capability_helpfully() {
        let e = parse_capabilities("event").unwrap_err();
        assert!(e.contains("unknown capability 'event'"));
        assert!(e.contains("events"));
        assert!(e.contains("source"));
    }

    /// The generated manifest has to be a manifest the loader accepts — a
    /// scaffold that does not load is worse than no scaffold.
    #[test]
    fn the_generated_manifest_round_trips() {
        let manifest = Manifest {
            id: "com.example.test".into(),
            name: "Test".into(),
            version: "0.1.0".into(),
            author: String::new(),
            description: String::new(),
            homepage: String::new(),
            repository: String::new(),
            license: "MIT".into(),
            logo: String::new(),
            topics: Vec::new(),
            readme: String::new(),
            readme: String::new(),
            entry: "plugin.wasm".into(),
            capabilities: vec![Capability::Metadata, Capability::Source],
            permissions: Permissions::default(),
        };
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("plugin.toml"), manifest_toml(&manifest)).unwrap();
        std::fs::write(dir.path().join("plugin.wasm"), b"\0asm\x01\0\0\0").unwrap();

        let loaded = Manifest::load(dir.path()).unwrap();
        assert_eq!(loaded.id, "com.example.test");
        assert_eq!(loaded.capabilities, manifest.capabilities);
        assert_eq!(loaded.license, "MIT");
        // Nothing is granted by default.
        assert!(loaded.permissions.allowed_hosts.is_empty());
        assert!(!loaded.permissions.library_read);
    }

    #[test]
    fn the_readme_names_the_exports_for_the_declared_capabilities() {
        let manifest = Manifest {
            id: "x".into(),
            name: "X".into(),
            version: "0.1.0".into(),
            author: String::new(),
            description: String::new(),
            homepage: String::new(),
            repository: String::new(),
            license: String::new(),
            logo: String::new(),
            topics: Vec::new(),
            readme: String::new(),
            readme: String::new(),
            entry: "plugin.wasm".into(),
            capabilities: vec![Capability::Predicates],
            permissions: Permissions::default(),
        };
        let readme = readme(&manifest, "rust", true);
        assert!(readme.contains("`predicates`, `evaluate`"));
        assert!(!readme.contains("get_metadata"));
        assert!(readme.contains("cargo build --release --target wasm32-wasip1"));
    }
}
