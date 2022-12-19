package(default_visibility = ["//visibility:public"])

load("//cargo:crates.bzl", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "client_lib",
    srcs = [
      "src/lib.rs",
      "src/library.rs",
      "src/playback.rs",
      "src/playlist.rs",
      "src/tracklist.rs",
      "src/ws_client.rs",
    ],
    deps = [
      "//server:server_lib",
      "//settings:settings_lib",
      "//playback:playback_lib",
      "//storage:storage_lib",
      "//tracklist:tracklist_lib",
    ] + all_crate_deps(),
)