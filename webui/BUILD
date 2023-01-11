package(default_visibility = ["//visibility:public"])

load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "music_player_webui",
    srcs = ["src/lib.rs"],
    compile_data = glob([
        "musicplayer/build/**",
    ]),
    data = glob([
        "musicplayer/build/**",
    ]),
    deps = [
        "//graphql:music_player_graphql",
        "//settings:music_player_settings",
        "//storage:music_player_storage",
        "//playback:music_player_playback",
        "//tracklist:music_player_tracklist",
        "//entity:music_player_entity",
        "//scanner:music_player_scanner",
        "//types:music_player_types",
        "//addons:music_player_addons",
        "@crate_index//:actix-web",
        "@crate_index//:async-graphql",
        "@crate_index//:async-graphql-actix-web",
        "@crate_index//:mime_guess",
        "@crate_index//:rust-embed",
        "@crate_index//:actix-files",
        "@crate_index//:actix-cors",
    ] + all_crate_deps(),
)
