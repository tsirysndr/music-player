package(default_visibility = ["//visibility:public"])

load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "music_player_scanner",
    srcs = ["src/lib.rs"],
    deps = [
      "//settings:music_player_settings",
      "//entity:music_player_entity",
      "//storage:music_player_storage",
      "//types:music_player_types",
      "//migration:migration",
      "@crate_index//:walkdir",
      "@crate_index//:mime_guess",
    ] + all_crate_deps(),
)