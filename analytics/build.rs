//! Link DuckDB against the upstream prebuilt static libraries.
//!
//! The obvious alternative — `duckdb/bundled` — compiles the whole DuckDB C++
//! amalgamation from source. That is minutes of clang on every clean build and
//! a large chunk of the CI budget, for a library we never patch. Upstream
//! already publishes exactly what we want per platform as
//! `static-libs-<platform>.zip` on the GitHub release.
//!
//! Static rather than the `libduckdb-<platform>.zip` shared library on
//! purpose: a `.dylib` has to be found again at *run* time, which means either
//! an rpath pointing into this checkout's `target/` — which breaks the moment
//! the binary is copied to `~/.local/bin` — or shipping the library next to
//! every artifact. A static archive keeps `music-player` one self-contained
//! file, as it was before DuckDB entered the tree.
//!
//! **This script does not download anything**; `scripts/fetch-duckdb.sh` does,
//! and has to, because `libduckdb-sys` resolves `-l static=duckdb_static` when
//! its own rlib is compiled and Cargo offers no way to order our build script
//! ahead of that. All that is left here is to name the support archives that
//! `libduckdb-sys` does not know about.
//!
//! `libduckdb-sys` is pointed at `vendor/duckdb/lib` by `DUCKDB_LIB_DIR` in
//! `.cargo/config.toml`, set there because a build script cannot export
//! environment variables to another crate's build script. Without it the sys
//! crate falls through to `pkg-config` and links whichever DuckDB happens to
//! be installed system-wide — a silent mismatch against the pregenerated
//! bindings.

use std::path::{Path, PathBuf};
use std::{env, fs};

/// Must match `VERSION` in `scripts/fetch-duckdb.sh`, and the DuckDB wrapped
/// by the `duckdb` crate: duckdb-rs encodes it in its own version as
/// `1.<major><minor:02><patch:02>.x`, so `1.10505.0` is DuckDB 1.5.5.
const DUCKDB_VERSION: &str = "1.5.5";

/// Benchmark data generators (`dsdgen`/`dbgen`). Nothing here calls them and
/// they are ~3.5 MB of archive.
const SKIP: &[&str] = &["tpcds", "tpch"];

fn main() {
    println!("cargo:rerun-if-env-changed=DUCKDB_LIB_DIR");

    let Some(lib_dir) = vendored_lib_dir() else {
        // Not fatal. A build that overrides DUCKDB_LIB_DIR/DUCKDB_STATIC to
        // use a shared library, or targets a platform with no prebuilt static
        // release, is legitimate and already has its own link flags from the
        // sys crate; only the support archives below would be missing, and
        // those come with the shared library.
        println!(
            "cargo:warning=vendor/duckdb/lib is missing or stale — run scripts/fetch-duckdb.sh"
        );
        return;
    };
    println!(
        "cargo:rerun-if-changed={}",
        lib_dir.join(".stamp").display()
    );
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    // `libduckdb-sys` emits `-l static=duckdb_static` and nothing else, but
    // the release splits DuckDB's vendored dependencies (zstd, re2, utf8proc,
    // pg_query, …) and its built-in extensions into separate archives. They
    // have to be named or the core archive's references go unresolved.
    let mut archives = Vec::new();
    for entry in fs::read_dir(&lib_dir).expect("the vendored library directory is readable") {
        let path = entry.expect("a readable directory entry").path();
        let Some(name) = archive_name(&path) else {
            continue;
        };
        if name == "duckdb_static" || SKIP.iter().any(|skip| name.contains(skip)) {
            continue;
        }
        archives.push(name);
    }
    // `read_dir` yields filesystem order, and a link order that differs
    // between machines produces failures that reproduce for exactly one
    // developer.
    archives.sort();

    for archive in &archives {
        println!("cargo:rustc-link-lib=static={archive}");
    }
    // The core archive references the support archives and the extensions
    // reference the core back to register themselves, which one left-to-right
    // pass cannot satisfy. Naming the core again is the standard resolution
    // for a circular static dependency.
    println!("cargo:rustc-link-lib=static=duckdb_static");

    // DuckDB is C++; the Rust driver links through `cc`, which does not pull
    // in the C++ runtime by itself.
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-link-lib=c++");
        // `__dynamic_cast` and `__gxx_personality_v0` live in libc++abi, not
        // libc++. Most links never notice, because only the DuckDB objects
        // that use RTTI or unwinding reference them and a small query pulls
        // none of those in — so omitting this links a trivial smoke test fine
        // and then fails the moment anything touches the storage layer.
        println!("cargo:rustc-link-lib=c++abi");
    } else {
        println!("cargo:rustc-link-lib=stdc++");
    }
}

/// `libduckdb_re2.a` -> `duckdb_re2`; `None` for anything not a static archive.
fn archive_name(path: &Path) -> Option<String> {
    if path.extension()? != "a" {
        return None;
    }
    let stem = path.file_stem()?.to_str()?;
    Some(stem.strip_prefix("lib").unwrap_or(stem).to_owned())
}

/// `vendor/duckdb/lib`, but only when its stamp says it holds a complete set
/// of archives for this platform and DuckDB version.
fn vendored_lib_dir() -> Option<PathBuf> {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").ok()?);
    let root = manifest
        .ancestors()
        .find(|dir| dir.join("Cargo.toml").exists() && dir.join("migration").is_dir())?;
    let lib_dir = root.join("vendor/duckdb/lib");

    let platform = match env::var("TARGET").ok()?.as_str() {
        "aarch64-apple-darwin" => "osx-arm64",
        "x86_64-apple-darwin" => "osx-amd64",
        "x86_64-unknown-linux-gnu" => "linux-amd64",
        "aarch64-unknown-linux-gnu" => "linux-arm64",
        _ => return None,
    };
    let want = format!("{DUCKDB_VERSION}-{platform}");
    let stamp = fs::read_to_string(lib_dir.join(".stamp")).ok()?;
    (stamp.trim() == want).then_some(lib_dir)
}
