package(default_visibility = ["//visibility:public"])

load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "music_player_types",
    srcs = [
      "src/lib.rs",
      "src/types.rs",
    ],
   deps = [
    "@crate_index//:tantivy",
   ] + all_crate_deps(),
)