package(default_visibility = ["//visibility:public"])

load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "music_player_playback",
    srcs = [
      "src/audio_backend/mod.rs",    
      "src/audio_backend/rodio.rs",  
      "src/audio_backend/sdl.rs",
      "src/decoder/mod.rs",
      "src/decoder/symphonia_decoder.rs",
      "src/metadata/mod.rs",
      "src/metadata/audio/file.rs",  
      "src/metadata/audio/mod.rs",
      "src/config.rs",     
      "src/dither.rs",
      "src/lib.rs",        
      "src/tests.rs",
      "src/convert.rs",    
      "src/formatter.rs",  
      "src/player.rs",
    ],
    deps = [
      "//audio:music_player_audio",
      "//tracklist:music_player_tracklist",
      "//entity:music_player_entity",
      "@crate_index//:rand",
      "@crate_index//:rodio",
      "@crate_index//:log",
      "@crate_index//:thiserror",
      "@crate_index//:symphonia",
      "@crate_index//:futures-util",
      "@crate_index//:cpal",
      "@crate_index//:parking_lot",
      "@crate_index//:rand_distr",
      "@crate_index//:zerocopy",
      "@crate_index//:librespot-protocol",
      "@crate_index//:url",
    ] + all_crate_deps(),
    proc_macro_deps = [
        "@crate_index//:async-trait",
    ],
)
