//! WebAssembly extensions, powered by [Extism](https://extism.org).
//!
//! An extension is a `.wasm` module plus a manifest, dropped into
//! `<app dir>/extensions/<id>/`. It declares what it plugs into — events,
//! metadata, commands, smart-playlist predicates, or a media source of its own
//! — and the host grants exactly that and nothing more.
//!
//! The host/guest contract lives in `schema.yaml`, an
//! [XTP schema](https://docs.xtp.dylibso.com). It is the source of truth: run
//! `xtp plugin init --schema-file schema.yaml --template <language>` to
//! generate typed bindings and a working skeleton in Rust, Go, TypeScript,
//! Python, C#, Zig or C++. [`abi`] mirrors the same types on the host side.
//!
//! See `extensions/examples/` for one worked example per capability.

pub mod abi;
pub mod catalog;
pub mod host;
pub mod install;
pub mod library;
pub mod manifest;
pub mod registry;

pub use abi::{Event, TrackInfo};
pub use catalog::{installed, Installed, Status};
pub use host::{Extension, HostContext, LibraryAccess};
pub use install::{cache_dir, install_from_url, search_paths};
pub use library::Library;
pub use manifest::{Capability, LogoSource, Manifest};
pub use registry::{extensions_dir, Registry, EXTENSIONS_DIR, PREDICATE_PREFIX};
