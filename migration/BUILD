package(default_visibility = ["//visibility:public"])

load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "migration",
    srcs = [
        "src/lib.rs", 
        "src/m20220101_000001_create_table.rs",
        "src/m20221115_220318_add_folder_table.rs",
        "src/m20221117_211308_add_created_at_column.rs",
        ],
    deps = ["//settings:music_player_settings"] + all_crate_deps(),
)