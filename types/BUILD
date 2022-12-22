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
    "//discovery:music_player_discovery",
    "@crate_index//:tantivy",
    "@crate_index//:mdns-sd",
   ] + all_crate_deps(),
)