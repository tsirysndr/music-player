package(default_visibility = ["//visibility:public"])

load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "music_player_settings",
    srcs = ["src/lib.rs"],
    deps = [
        "@crate_index//:config",
        "@crate_index//:uuid",
        "@crate_index//:toml",
        "@crate_index//:serde",
        "@crate_index//:dirs",
        "@crate_index//:md5",
    ] + all_crate_deps(),
)