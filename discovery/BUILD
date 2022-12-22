package(default_visibility = ["//visibility:public"])

load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "music_player_discovery",
    srcs = ["src/lib.rs"],
    deps = [
        "//settings:music_player_settings",
        "@crate_index//:env_logger",
        "@crate_index//:libmdns",
        "@crate_index//:mdns",
        "@crate_index//:mdns-sd",
        "@crate_index//:async-stream",
        "@crate_index//:futures-util",
    ] + all_crate_deps(),
)