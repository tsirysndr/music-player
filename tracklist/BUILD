package(default_visibility = ["//visibility:public"])

load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "music_player_tracklist",
    srcs = ["src/lib.rs"],
    deps = [
        "//entity:music_player_entity", 
        "@crate_index//:rand"
    ] + all_crate_deps(),
)