package(default_visibility = ["//visibility:public"])

load("//cargo:crates.bzl", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "settings_lib",
    srcs = ["src/lib.rs"],
    deps = all_crate_deps(),
)