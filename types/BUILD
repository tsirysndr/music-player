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
    "//settings:music_player_settings",
    "@crate_index//:tantivy",
    "@crate_index//:mdns-sd",
    "@crate_index//:local-ip-addr",
    "@crate_index//:upnp-client",
    "@crate_index//:url",
   ] + all_crate_deps(),
)