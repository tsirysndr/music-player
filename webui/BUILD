package(default_visibility = ["//visibility:public"])

load("//cargo:crates.bzl", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "webui_lib",
    srcs = ["src/lib.rs"],
    deps = [
      "//graphql:graphql_lib",
      "//settings:settings_lib",
      "//storage:storage_lib",
      "//playback:playback_lib",
      "//tracklist:tracklist_lib",
      "//entity:entity_lib",
      "//scanner:scanner_lib",
    ] + all_crate_deps(),
)