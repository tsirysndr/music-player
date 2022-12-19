package(default_visibility = ["//visibility:public"])

load("//cargo:crates.bzl", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "storage_lib",
    srcs = [
      "src/lib.rs",
      "src/searcher/album.rs",
      "src/searcher/artist.rs",
      "src/searcher/mod.rs",
      "src/searcher/track.rs",
    ],
    deps = [
      "//settings:settings_lib",
      "//types:types_lib",
    ] + all_crate_deps(),
)