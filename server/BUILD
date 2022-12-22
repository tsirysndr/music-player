package(default_visibility = ["//visibility:public"])

load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "music_player_server",
    srcs = [
      "src/api/metadata.v1alpha1.rs",
      "src/api/music.v1alpha1.rs",
      "src/api/objects.v1alpha1.rs",
      "src/addons.rs",
      "src/history.rs",
      "src/playlist.rs",
      "src/core.rs",
      "src/lib.rs",        
      "src/mixer.rs",
      "src/server.rs",
      "src/event.rs",
      "src/library.rs",
      "src/playback.rs",
      "src/tracklist.rs"
    ],
    deps = [
      "//playback:music_player_playback",
      "//scanner:music_player_scanner",
      "//settings:music_player_settings",
      "//storage:music_player_storage",
      "//entity:music_player_entity",
      "//tracklist:music_player_tracklist",
      "@crate_index//:tonic",
      "@crate_index//:tonic-web",
      "@crate_index//:futures-util",
      "@crate_index//:prost",
      "@crate_index//:serde",
      "@crate_index//:tokio-tungstenite",
      "@crate_index//:tungstenite",
      "@crate_index//:tokio",
      "@crate_index//:chrono",
      "@crate_index//:uuid",
      "@crate_index//:cuid",
    ] + all_crate_deps(),
)