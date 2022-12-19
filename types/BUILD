package(default_visibility = ["//visibility:public"])

load("//cargo:crates.bzl", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "types_lib",
    srcs = [
      "src/lib.rs",
      "src/types.rs",
    ],
   deps = all_crate_deps(),
)