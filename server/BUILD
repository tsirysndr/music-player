package(default_visibility = ["//visibility:public"])

load("//cargo:crates.bzl", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")
load("@rules_rust//cargo:defs.bzl", "cargo_build_script")

proto_library(
    name = "protos",
    srcs = ["proto/*/*/*.proto"],
    visibility = ["//visibility:public"],
)

cargo_build_script(
    name = "build_script",
    srcs = ["build.rs"],
    data = [":protos"],
)

rust_library(
    name = "server_lib",
    srcs = [
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
      "//playback:playback_lib",
      "//scanner:scanner_lib",
      "//settings:settings_lib",
      "//storage:storage_lib",
      "//entity:entity_lib",
      "//tracklist:tracklist_lib",
    ] + all_crate_deps(),
)