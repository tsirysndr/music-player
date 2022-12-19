package(default_visibility = ["//visibility:public"])

load("//cargo:crates.bzl", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "addons_lib",
    srcs = [
      "src/datpiff.rs",
      "src/deezer.rs",
      "src/genius.rs",
      "src/lastfm.rs",
      "src/lib.rs",
      "src/local.rs",
      "src/musicbrainz.rs",
      "src/myvazo.rs",
      "src/tononkira.rs",
      ],
      deps = all_crate_deps(),
)