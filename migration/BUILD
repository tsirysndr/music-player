package(default_visibility = ["//visibility:public"])

load("//cargo:crates.bzl", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_library")

rust_library(
    name = "migration_lib",
    srcs = ["src/lib.rs"],
    deps = ["//settings:settings_lib"] + all_crate_deps(),
)