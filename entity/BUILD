package(default_visibility = ["//visibility:public"])

load("//cargo:crates.bzl", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "entity_lib",
    srcs = [
      "src/addon.rs", 
      "src/album.rs",
      "src/artist_tracks.rs",
      "src/artist.rs",
      "src/folder.rs",
      "src/lib.rs",
      "src/playlist_tracks.rs",
      "src/playlist.rs",
      "src/track.rs",
    ],
    deps = ["//types:types_lib"] + all_crate_deps(),
)