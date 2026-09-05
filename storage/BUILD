package(default_visibility = ["//visibility:public"])

load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "music_player_storage",
    srcs = [
      "src/lib.rs",
      "src/repo/mod.rs",
      "src/repo/album.rs",
      "src/repo/artist.rs",
      "src/repo/track.rs",
      "src/repo/playlist.rs",
      "src/repo/folder.rs",
      "src/searcher/mod.rs",
    ],
    deps = [
      "//settings:music_player_settings",
      "//types:music_player_types",
      "//entity:music_player_entity",
      "@crate_index//:itertools",
    ] + all_crate_deps(),
)