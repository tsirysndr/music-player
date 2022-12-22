package(default_visibility = ["//visibility:public"])

load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "music_player_client",
    srcs = [
      "src/lib.rs",
      "src/library.rs",
      "src/playback.rs",
      "src/playlist.rs",
      "src/tracklist.rs",
      "src/ws_client.rs",
    ],
    deps = [
      "//server:music_player_server",
      "//settings:music_player_settings",
      "//playback:music_player_playback",
      "//storage:music_player_storage",
      "//tracklist:music_player_tracklist",
      "@crate_index//:tonic",
      "@crate_index//:futures-util",
      "@crate_index//:url",
      "@crate_index//:serde",
      "@crate_index//:tokio-tungstenite",
      "@crate_index//:tungstenite",
      "@crate_index//:tokio",
    ] + all_crate_deps(),
)