package(default_visibility = ["//visibility:public"])

load("//cargo:crates.bzl", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "playback_lib",
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
      "//tracklist:tracklist_lib",
      "//entity:entity_lib",
    ] + all_crate_deps(),
)