package(default_visibility = ["//visibility:public"])

load("@crate_index//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "music_player_addons",
    srcs = [
      "src/airplay.rs",
      "src/chromecast.rs",
      "src/datpiff.rs",
      "src/deezer.rs",
      "src/dlna.rs",
      "src/genius.rs",
      "src/kodi.rs",
      "src/lastfm.rs",
      "src/lib.rs",
      "src/local.rs",
      "src/musicbrainz.rs",
      "src/myvazo.rs",
      "src/tononkira.rs",
      ],
      deps = [
        "//client:music_player_client",
        "//types:music_player_types",
        "//tracklist:music_player_tracklist",
        "@crate_index//:upnp-client",
        "@crate_index//:local-ip-addr",
        "@crate_index//:rust_cast",
      ] + all_crate_deps(),
      proc_macro_deps = [
        "@crate_index//:async-trait",
      ],
)