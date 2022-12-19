package(default_visibility = ["//visibility:public"])

load("//cargo:crates.bzl", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "graphql_lib",
    srcs = [
      "src/schema/objects/album.rs",
      "src/schema/objects/artist.rs",
      "src/schema/objects/current_track.rs",
      "src/schema/objects/folder.rs",
      "src/schema/objects/lyrics.rs",
      "src/schema/objects/mod.rs",
      "src/schema/objects/player_state.rs",
      "src/schema/objects/playlist.rs",
      "src/schema/objects/search_result.rs",
      "src/schema/objects/track.rs",
      "src/schema/objects/tracklist.rs",
      "src/schema/addons.rs",
      "src/schema/core.rs",
      "src/schema/history.rs",
      "src/schema/library.rs",
      "src/schema/mixer.rs",
      "src/schema/mod.rs",
      "src/schema/playback.rs",
      "src/schema/playlist.rs",
      "src/schema/tracklist.rs",
      "src/lib.rs",
      "src/simple_broker.rs",
    ],
    deps = [
      "//playback:playback_lib",
      "//scanner:scanner_lib",
      "//settings:settings_lib",
      "//storage:storage_lib",
      "//entity:entity_lib",
      "//tracklist:tracklist_lib",
      "//types:types_lib",
    ] + all_crate_deps(),
)