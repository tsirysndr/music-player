package(default_visibility = ["//visibility:public"])

load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "music_player_entity",
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
    deps = [
      "//types:music_player_types",
      "@crate_index//:serde",
      "@crate_index//:chrono",
      "@crate_index//:md5"
      ] + all_crate_deps(),
)