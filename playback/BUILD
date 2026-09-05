package(default_visibility = ["//visibility:public"])

load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "music_player_playback",
    srcs = [
      "src/lib.rs",
      "src/player.rs",
      "src/tests.rs",
    ],
    deps = [
      "//tracklist:music_player_tracklist",
      "//entity:music_player_entity",
      "//settings:music_player_settings",
      "@crate_index//:anyhow",
      "@crate_index//:log",
      "@crate_index//:futures-util",
      "@crate_index//:rockbox-playback",
      "@crate_index//:tokio",
    ] + all_crate_deps(),
    proc_macro_deps = [
        "@crate_index//:async-trait",
    ],
)
